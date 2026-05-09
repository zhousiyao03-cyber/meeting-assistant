//! OpenAI Realtime API streaming transcription client.
//!
//! Protocol reference: https://platform.openai.com/docs/guides/realtime-transcription
//! Endpoint: wss://api.openai.com/v1/realtime?model=gpt-realtime-whisper
//!
//! Frame schema (client→server):
//!   { "type": "input_audio_buffer.append", "audio": "<base64 PCM16 LE 16kHz mono>" }
//!
//! Frame schema (server→client):
//!   { "type": "conversation.item.input_audio_transcription.delta", "delta": "<text>" }
//!   { "type": "conversation.item.input_audio_transcription.completed", "transcript": "<final>" }

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::protocol::Message;

use super::{AsrConfig, AsrProvider, TranscriptCallback, TranscriptChunk};

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

pub struct OpenAiRealtimeWhisperProvider {
    config: AsrConfig,
    on_transcript: TranscriptCallback,
    sender: Option<Arc<TokioMutex<WsSink>>>,
    reader_handle: Option<JoinHandle<()>>,
    start_time: Option<std::time::Instant>,
}

impl OpenAiRealtimeWhisperProvider {
    pub fn new(config: &AsrConfig, on_transcript: TranscriptCallback) -> Result<Self> {
        if config.openai_api_key.is_empty() {
            return Err(anyhow!("OpenAI API key not configured"));
        }
        Ok(Self {
            config: config.clone(),
            on_transcript,
            sender: None,
            reader_handle: None,
            start_time: None,
        })
    }
}

#[async_trait::async_trait]
impl AsrProvider for OpenAiRealtimeWhisperProvider {
    async fn start(&mut self) -> Result<()> {
        let url = format!(
            "wss://api.openai.com/v1/realtime?model={}",
            self.config.openai_model
        );

        let request = Request::builder()
            .uri(&url)
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .header("OpenAI-Beta", "realtime=v1")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Host", "api.openai.com")
            .body(())?;

        let (ws, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| anyhow!("WebSocket connect to OpenAI Realtime failed: {}", e))?;

        let (sink, mut stream) = ws.split();
        let sink = Arc::new(TokioMutex::new(sink));

        let session_update = serde_json::json!({
            "type": "transcription_session.update",
            "session": {
                "input_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": self.config.openai_model,
                    "language": if self.config.language_hint == "auto" {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(self.config.language_hint.clone())
                    }
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "silence_duration_ms": 250
                }
            }
        });
        sink.lock()
            .await
            .send(Message::Text(session_update.to_string()))
            .await
            .map_err(|e| anyhow!("session update send: {}", e))?;

        let on_transcript = self.on_transcript.clone();
        let start_time = std::time::Instant::now();
        let reader = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => handle_event(&v, &on_transcript, start_time),
                        Err(e) => eprintln!("[asr] non-json frame: {} ({})", text, e),
                    },
                    Ok(Message::Close(_)) => {
                        eprintln!("[asr] WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[asr] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        self.sender = Some(sink);
        self.reader_handle = Some(reader);
        self.start_time = Some(start_time);
        eprintln!("[asr] OpenAI Realtime Whisper session started");
        Ok(())
    }

    async fn send_audio(&mut self, pcm_16k_mono: &[f32], _speaker: &str) -> Result<()> {
        let pcm16_bytes = f32_to_pcm16_bytes(pcm_16k_mono);
        let b64 = B64.encode(&pcm16_bytes);

        let frame = serde_json::json!({
            "type": "input_audio_buffer.append",
            "audio": b64
        });

        let sender = self.sender.as_ref().ok_or_else(|| anyhow!("ASR not started"))?;
        sender
            .lock()
            .await
            .send(Message::Text(frame.to_string()))
            .await
            .map_err(|e| anyhow!("audio send: {}", e))?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(sender) = self.sender.take() {
            let _ = sender.lock().await.close().await;
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.await;
        }
        eprintln!("[asr] OpenAI Realtime Whisper session stopped");
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "openai-realtime-whisper"
    }
}

fn handle_event(
    v: &serde_json::Value,
    on_transcript: &TranscriptCallback,
    start_time: std::time::Instant,
) {
    let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let offset = start_time.elapsed().as_secs_f64();

    match event_type {
        "conversation.item.input_audio_transcription.delta" => {
            if let Some(delta) = v.get("delta").and_then(|d| d.as_str()) {
                if !delta.is_empty() {
                    on_transcript(TranscriptChunk {
                        text: delta.to_string(),
                        speaker: "other".to_string(),
                        offset_secs: offset,
                        is_final: false,
                    });
                }
            }
        }
        "conversation.item.input_audio_transcription.completed" => {
            if let Some(text) = v.get("transcript").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    on_transcript(TranscriptChunk {
                        text: text.to_string(),
                        speaker: "other".to_string(),
                        offset_secs: offset,
                        is_final: true,
                    });
                }
            }
        }
        "error" => {
            eprintln!("[asr] OpenAI error event: {}", v);
        }
        _ => {
            // session.created, session.updated, etc.
        }
    }
}

fn f32_to_pcm16_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * 32767.0) as i16;
        bytes.extend_from_slice(&i16_val.to_le_bytes());
    }
    bytes
}
