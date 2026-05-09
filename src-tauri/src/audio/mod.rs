pub mod buffer;
pub mod capture;
pub mod permission;
pub mod system_capture;

#[cfg(target_os = "macos")]
pub mod screen_capture_kit;
