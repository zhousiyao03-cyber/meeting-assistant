//! NSWindow.sharingType controls whether the window is captured by
//! ScreenCaptureKit / NSScreenshot / Zoom screen sharing.
//!
//! Setting to NSWindowSharingType::None makes the window invisible to all
//! capture APIs while remaining visible to the user.

#![cfg(target_os = "macos")]

use objc2::msg_send;

/// Set the given NSWindow's sharing type.
///
/// # Safety
/// Caller must ensure `ns_window_ptr` points to a valid NSWindow object
/// (typically obtained from Tauri via window.ns_window()).
pub unsafe fn set_window_sharing_type(ns_window_ptr: *mut std::ffi::c_void, hidden: bool) {
    // NSWindowSharingType values:
    //   NSWindowSharingNone      = 0  (invisible to screen capture)
    //   NSWindowSharingReadOnly  = 1  (default; capturable)
    //   NSWindowSharingReadWrite = 2  (deprecated)
    let sharing_type: usize = if hidden { 0 } else { 1 };

    // Cast c_void pointer to anonymous Objective-C object pointer.
    // We don't need the strongly-typed NSWindow binding; the message send is
    // dynamic dispatch based on selector name.
    let obj = ns_window_ptr as *mut objc2::runtime::AnyObject;
    if obj.is_null() {
        return;
    }
    let _: () = msg_send![&*obj, setSharingType: sharing_type];
}
