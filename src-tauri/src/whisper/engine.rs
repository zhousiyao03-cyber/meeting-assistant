use anyhow::{anyhow, Result};
use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig,
    SileroVadModelConfig, VadModelConfig, VoiceActivityDetector,
};
use std::cell::RefCell;
use std::path::Path;

/// ASR engine using sherpa-onnx with SenseVoice + Silero VAD.
///
/// Replaces the previous WhisperEngine. Uses VAD to automatically segment
/// speech, then runs SenseVoice offline recognition on each segment.
pub struct SherpaEngine {
    recognizer: OfflineRecognizer,
    vad: VoiceActivityDetector,
    remainder: RefCell<Vec<f32>>,
}

// Safety: The underlying C pointers are used from a single tokio task,
// matching the previous WhisperEngine usage pattern.
unsafe impl Send for SherpaEngine {}

impl SherpaEngine {
    /// Load SenseVoice model and Silero VAD from the given directory.
    ///
    /// Expected files in `model_dir`:
    /// - `model.int8.onnx` (SenseVoice int8 model)
    /// - `tokens.txt` (token vocabulary)
    /// - `silero_vad.onnx` (Silero VAD model)
    pub fn new(model_dir: &Path) -> Result<Self> {
        // Create Silero VAD
        let vad_config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: Some(model_dir.join("silero_vad.onnx").to_string_lossy().into_owned()),
                threshold: 0.5,
                min_silence_duration: 0.25,
                min_speech_duration: 0.25,
                max_speech_duration: 8.0,
                window_size: 512, // must be 512 for 16kHz
            },
            sample_rate: 16000,
            num_threads: 1,
            provider: Some("cpu".into()),
            ..Default::default()
        };
        let vad = VoiceActivityDetector::create(&vad_config, 30.0)
            .ok_or_else(|| anyhow!("Failed to create Silero VAD — check silero_vad.onnx path"))?;

        // Create SenseVoice offline recognizer
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
            model: Some(model_dir.join("model.int8.onnx").to_string_lossy().into_owned()),
            language: Some("auto".into()),
            use_itn: true,
        };
        config.model_config.tokens =
            Some(model_dir.join("tokens.txt").to_string_lossy().into_owned());
        config.model_config.num_threads = 2;

        let recognizer = OfflineRecognizer::create(&config)
            .ok_or_else(|| anyhow!("Failed to create SenseVoice recognizer — check model files"))?;

        eprintln!("[sherpa] SenseVoice + Silero VAD loaded from {:?}", model_dir);
        Ok(Self { recognizer, vad, remainder: RefCell::new(Vec::with_capacity(512)) })
    }

    /// Feed audio samples (16kHz mono f32) to the VAD, then recognize any
    /// complete speech segments. Returns zero or more transcribed text strings.
    pub fn process_audio(&self, audio: &[f32]) -> Vec<String> {
        let mut results = Vec::new();
        let mut remainder = self.remainder.borrow_mut();

        // Prepend leftover samples from last call
        remainder.extend_from_slice(audio);

        // Feed VAD in 512-sample windows
        let full_windows = remainder.len() / 512;
        for i in 0..full_windows {
            let start = i * 512;
            self.vad.accept_waveform(&remainder[start..start + 512]);
        }

        // Keep the remainder for next call
        let consumed = full_windows * 512;
        *remainder = remainder[consumed..].to_vec();

        // Recognize each complete speech segment
        while !self.vad.is_empty() {
            if let Some(segment) = self.vad.front() {
                let text = self.recognize_segment(segment.samples());
                if !text.is_empty() {
                    results.push(text);
                }
                self.vad.pop();
            }
        }

        results
    }

    /// Flush any remaining speech buffered in the VAD (call when recording stops).
    pub fn flush(&self) -> Vec<String> {
        // Feed any remaining samples before flushing
        {
            let mut remainder = self.remainder.borrow_mut();
            if remainder.len() >= 256 {
                let mut padded = remainder.drain(..).collect::<Vec<_>>();
                padded.resize(512, 0.0);
                self.vad.accept_waveform(&padded);
            }
            remainder.clear();
        }
        self.vad.flush();
        let mut results = Vec::new();
        while !self.vad.is_empty() {
            if let Some(segment) = self.vad.front() {
                let text = self.recognize_segment(segment.samples());
                if !text.is_empty() {
                    results.push(text);
                }
                self.vad.pop();
            }
        }
        results
    }

    /// Reset the VAD state (e.g., when resuming after pause).
    pub fn reset(&self) {
        self.vad.reset();
    }

    /// Run SenseVoice recognition on a single audio segment.
    fn recognize_segment(&self, samples: &[f32]) -> String {
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(16000, samples);
        self.recognizer.decode(&stream);

        match stream.get_result() {
            Some(result) => {
                let text = result.text.trim().to_string();
                if !text.is_empty() {
                    eprintln!("[sherpa] recognized: {}", &text);
                }
                text
            }
            None => String::new(),
        }
    }
}
