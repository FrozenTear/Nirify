//! Miscellaneous settings (cursor, clipboard, hotkey overlay, xwayland)

use crate::constants::DEFAULT_CURSOR_SIZE;

/// Cursor settings
#[derive(Debug, Clone, PartialEq)]
pub struct CursorSettings {
    pub theme: String,
    pub size: i32,
    pub hide_when_typing: bool,
    pub hide_after_inactive_ms: Option<i32>,
}

impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            theme: String::new(), // Use system default
            size: DEFAULT_CURSOR_SIZE,
            hide_when_typing: false,
            hide_after_inactive_ms: None,
        }
    }
}

/// Miscellaneous settings
#[derive(Debug, Clone, PartialEq)]
pub struct MiscSettings {
    pub prefer_no_csd: bool,
    pub screenshot_path: String,
    pub disable_primary_clipboard: bool,
    pub hotkey_overlay_skip_at_startup: bool,
    /// Hide actions not bound to any key in hotkey overlay (v25.08+)
    pub hotkey_overlay_hide_not_bound: bool,
    /// Disable "Failed to parse config" notifications (v25.08+)
    pub config_notification_disable_failed: bool,
    /// Execute spawn-at-startup commands through shell (v25.08+)
    pub spawn_sh_at_startup: bool,
    /// XWayland satellite configuration (v25.08+)
    pub xwayland_satellite: XWaylandSatelliteConfig,
}

/// XWayland satellite configuration
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum XWaylandSatelliteConfig {
    /// Use default xwayland-satellite behavior
    #[default]
    Default,
    /// Disable xwayland-satellite
    Off,
    /// Use custom path for xwayland-satellite
    CustomPath(String),
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self {
            prefer_no_csd: false,
            screenshot_path: String::from(
                "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png",
            ),
            disable_primary_clipboard: false,
            hotkey_overlay_skip_at_startup: false,
            hotkey_overlay_hide_not_bound: false,
            config_notification_disable_failed: false,
            spawn_sh_at_startup: false,
            xwayland_satellite: XWaylandSatelliteConfig::Default,
        }
    }
}
