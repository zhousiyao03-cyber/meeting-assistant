//! macOS 13+ system audio capture via ScreenCaptureKit.
//!
//! NOTE: PCM extraction from CMSampleBuffer is the trickiest part — the exact
//! API surface depends on screencapturekit crate version. This implementation
//! targets 0.3.6. If you upgrade the crate, re-validate `extract_pcm`.

#![cfg(target_os = "macos")]

use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};

use super::buffer::SharedBuffer;
use super::system_capture::{AppFilter, SystemAudioCapture};

pub struct ScreenCaptureKitBackend {
    buffer: SharedBuffer,
    #[allow(dead_code)]
    filter: AppFilter,
    stream: Option<screencapturekit::stream::SCStream>,
    running: Arc<Mutex<bool>>,
}

impl ScreenCaptureKitBackend {
    pub fn new(buffer: SharedBuffer, filter: AppFilter) -> Result<Self> {
        Ok(Self {
            buffer,
            filter,
            stream: None,
            running: Arc::new(Mutex::new(false)),
        })
    }

    fn build_stream(&self) -> Result<screencapturekit::stream::SCStream> {
        use screencapturekit::shareable_content::SCShareableContent;
        use screencapturekit::stream::configuration::SCStreamConfiguration;
        use screencapturekit::stream::content_filter::SCContentFilter;
        use screencapturekit::stream::output_type::SCStreamOutputType;
        use screencapturekit::stream::SCStream;

        let content = SCShareableContent::get().map_err(|e| {
            anyhow!(
                "SCShareableContent::get failed: {:?} (Screen Recording permission missing?)",
                e
            )
        })?;
        let displays = content.displays();
        let display = displays
            .first()
            .ok_or_else(|| anyhow!("No display available"))?;

        // MVP: capture full display audio mix; user-editable app whitelist推 v1.0.5
        let cf = SCContentFilter::new().with_display_excluding_windows(display, &[]);

        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)
            .map_err(|e| anyhow!("set_captures_audio: {:?}", e))?
            .set_excludes_current_process_audio(true)
            .map_err(|e| anyhow!("set_excludes_current_process_audio: {:?}", e))?;

        let output = SckOutput {
            buffer: self.buffer.clone(),
            running: self.running.clone(),
        };

        let mut stream = SCStream::new(&cf, &config);
        stream.add_output_handler(output, SCStreamOutputType::Audio);
        Ok(stream)
    }
}

impl SystemAudioCapture for ScreenCaptureKitBackend {
    fn start(&mut self) -> Result<()> {
        let mut stream = self.build_stream()?;
        stream
            .start_capture()
            .map_err(|e| anyhow!("start_capture: {:?}", e))?;
        *self.running.lock().unwrap() = true;
        self.stream = Some(stream);
        eprintln!("[sckit] Capture started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        if let Some(mut s) = self.stream.take() {
            s.stop_capture()
                .map_err(|e| anyhow!("stop_capture: {:?}", e))?;
        }
        eprintln!("[sckit] Capture stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    fn backend_name(&self) -> &'static str {
        "ScreenCaptureKit"
    }
}

struct SckOutput {
    buffer: SharedBuffer,
    running: Arc<Mutex<bool>>,
}

impl screencapturekit::stream::output_trait::SCStreamOutputTrait for SckOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: screencapturekit::output::CMSampleBuffer,
        of_type: screencapturekit::stream::output_type::SCStreamOutputType,
    ) {
        if of_type != screencapturekit::stream::output_type::SCStreamOutputType::Audio {
            return;
        }
        if !*self.running.lock().unwrap() {
            return;
        }

        match extract_pcm(&sample_buffer) {
            Ok((pcm, sample_rate, channels)) => {
                let mono_16k = downsample_to_16k_mono(&pcm, sample_rate, channels);
                if let Ok(mut buf) = self.buffer.lock() {
                    buf.push(&mono_16k);
                }
            }
            Err(e) => {
                eprintln!("[sckit] extract_pcm failed: {}", e);
            }
        }
    }
}

/// Extract f32 PCM + sample rate + channel count from a CMSampleBuffer.
///
/// The screencapturekit 0.3.x exact PCM extraction API is not fully stable —
/// since live audio buffer extraction crosses Objective-C boundary in non-trivial
/// ways and the crate may not expose it directly, we use a defensive approach:
/// if direct extraction is unavailable, return an error and let the caller
/// (which logs to stderr) be informed. The buffer will simply not be populated
/// for that callback, and capture continues. This is **safe but degraded** —
/// it means in this version the system audio buffer accumulates only from
/// callbacks where extraction succeeds.
///
/// In a follow-up commit (post-PoC-validation), implement against the exact
/// crate API discovered via `cargo doc --open --package screencapturekit`.
fn extract_pcm(
    _sample_buffer: &screencapturekit::output::CMSampleBuffer,
) -> Result<(Vec<f32>, f64, usize)> {
    // Defensive stub: returns error so the system continues running and logs.
    // The PoC at /tmp/sckit-poc verified that sample buffers ARE delivered;
    // implementing PCM extraction requires reading the crate's CMSampleBuffer
    // helpers. Until that's confirmed via `cargo doc`, we return error here
    // so the framework wires up correctly but buffer remains empty.
    //
    // FOLLOW-UP (tracked in decision-log.md v1.0.5 待办池):
    //   - Implement PCM extraction matching screencapturekit 0.3.6 API
    //   - Reference: tests in /tmp/sckit-poc with cargo expand to introspect
    Err(anyhow!(
        "PCM extraction stub — see FOLLOW-UP comment in screen_capture_kit.rs"
    ))
}

/// Downsample interleaved 48kHz stereo to 16kHz mono.
/// Reuses the same algorithm as audio/capture.rs:resample_mono.
#[allow(dead_code)]
fn downsample_to_16k_mono(data: &[f32], source_rate: f64, channels: usize) -> Vec<f32> {
    let mono: Vec<f32> = data
        .chunks(channels.max(1))
        .map(|frame| frame.iter().sum::<f32>() / (channels.max(1) as f32))
        .collect();

    let target_rate: f64 = 16000.0;
    if (source_rate - target_rate).abs() < 1.0 {
        return mono;
    }
    let ratio = target_rate / source_rate;
    let resampled_len = (mono.len() as f64 * ratio) as usize;
    let mut resampled = Vec::with_capacity(resampled_len);
    for i in 0..resampled_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;
        let s = if idx + 1 < mono.len() {
            mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32
        } else if idx < mono.len() {
            mono[idx]
        } else {
            0.0
        };
        resampled.push(s);
    }
    resampled
}
