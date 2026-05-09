use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
}

/// Check whether the app has been granted Screen Recording permission.
/// Approach: try `SCShareableContent::get()` — if it returns a non-empty
/// display list, permission is granted. Empty list or error = denied.
#[cfg(target_os = "macos")]
pub fn check_screen_recording_permission() -> PermissionStatus {
    use screencapturekit::shareable_content::SCShareableContent;
    match SCShareableContent::get() {
        Ok(content) => {
            if content.displays().is_empty() {
                PermissionStatus::Denied
            } else {
                PermissionStatus::Granted
            }
        }
        Err(_) => PermissionStatus::Denied,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_screen_recording_permission() -> PermissionStatus {
    PermissionStatus::Denied
}

/// Open System Settings → Privacy & Security → Screen Recording.
pub fn open_settings_screen_recording() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        .spawn()
        .map(|_| ())
}

/// Get macOS version. Returns Some((major, minor)) or None on non-macOS.
pub fn macos_version() -> Option<(u32, u32)> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()?;
        let s = String::from_utf8(output.stdout).ok()?;
        let parts: Vec<&str> = s.trim().split('.').collect();
        let major: u32 = parts.first()?.parse().ok()?;
        let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        Some((major, minor))
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

pub fn macos_version_at_least(major: u32, minor: u32) -> bool {
    matches!(macos_version(), Some((m, n)) if m > major || (m == major && n >= minor))
}
