use macroquad::{miniquad::conf::{Icon, Platform}, prelude::*};

pub struct WindowConfig {
    /// Window title. Defaults to an empty string.
    pub window_title: String,

    /// Preferred window width (ignored on WASM/Android).
    /// Defaults to `800`.
    pub window_width: i32,

    /// Preferred window height (ignored on WASM/Android).
    /// Defaults to `600`.
    pub window_height: i32,

    /// If `true`, the rendering canvas is scaled for HighDPI displays.
    /// Defaults to `false`.
    pub high_dpi: bool,

    /// If `true`, create the window in fullscreen mode (ignored on WASM/Android).
    /// Defaults to `false`.
    pub fullscreen: bool,

    /// MSAA sample count.
    /// Defaults to `1`.
    pub sample_count: i32,

    /// If `true`, the user can resize the window.
    pub window_resizable: bool,

    /// Optional icon data used by the OS where applicable:
    /// - On Windows, taskbar/title bar icon
    /// - On macOS, Dock/title bar icon
    /// - TODO: Favicon on HTML5
    /// - TODO: Taskbar/title bar icon on Linux (depends on WM)
    /// - Note: on gnome, icon is determined using `WM_CLASS` (can be set under [`Platform`]) and
    ///   an external `.desktop` file
    pub icon: Option<Icon>,

    /// Platform-specific hints (e.g., context creation, driver settings).
    pub platform: Platform,
}
