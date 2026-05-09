use anyhow::Result;

use super::buffer::SharedBuffer;

/// Backend-agnostic system audio capture.
/// Implementations push 16kHz mono f32 PCM into the buffer continuously
/// once `start()` returns Ok.
pub trait SystemAudioCapture: Send {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn backend_name(&self) -> &'static str;
}

#[derive(Clone, Debug)]
pub struct AppFilter {
    pub included_bundle_ids: Vec<String>,
    pub excluded_bundle_ids: Vec<String>,
}

impl Default for AppFilter {
    fn default() -> Self {
        Self {
            included_bundle_ids: vec![
                "us.zoom.xos".into(),
                "com.microsoft.teams2".into(),
                "com.google.Chrome".into(),
                "com.apple.Safari".into(),
                "com.hnc.Discord".into(),
                "com.apple.FaceTime".into(),
            ],
            excluded_bundle_ids: vec![
                "com.spotify.client".into(),
                "com.apple.Music".into(),
                "com.apple.Notes".into(),
            ],
        }
    }
}

/// Pick the best system-audio backend for this OS / version.
#[cfg(target_os = "macos")]
pub fn create_system_audio_capture(
    buffer: SharedBuffer,
    filter: AppFilter,
) -> Result<Box<dyn SystemAudioCapture>> {
    if super::permission::macos_version_at_least(13, 0) {
        return Ok(Box::new(
            super::screen_capture_kit::ScreenCaptureKitBackend::new(buffer, filter)?,
        ));
    }
    Err(anyhow::anyhow!(
        "Confide requires macOS 13.0 or later. macOS 12 support planned for v1.1."
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn create_system_audio_capture(
    _buffer: SharedBuffer,
    _filter: AppFilter,
) -> Result<Box<dyn SystemAudioCapture>> {
    Err(anyhow::anyhow!(
        "Confide currently supports macOS only. Windows planned for v1.2."
    ))
}
