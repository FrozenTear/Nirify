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
    pub screenshot_path: ScreenshotPathConfig,
    pub disable_primary_clipboard: bool,
    pub hotkey_overlay_skip_at_startup: bool,
    /// Hide actions not bound to any key in hotkey overlay (v25.08+)
    pub hotkey_overlay_hide_not_bound: bool,
    /// Disable "Failed to parse config" notifications (v25.08+)
    pub config_notification_disable_failed: bool,
    /// Shell commands to run at startup (v25.08+)
    pub spawn_sh_at_startup: Vec<SpawnShCommand>,
    /// Counter for generating unique spawn-sh-at-startup IDs
    pub spawn_sh_next_id: u32,
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

/// A shell command run at startup via `spawn-sh-at-startup`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpawnShCommand {
    /// Unique identifier for UI
    pub id: u32,
    /// The shell command line
    pub command: String,
}

/// Screenshot save-path configuration (`screenshot-path`)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScreenshotPathConfig {
    /// Omit the node entirely; niri uses its built-in default path
    #[default]
    Default,
    /// `screenshot-path null` — screenshots are not saved to disk
    Disabled,
    /// `screenshot-path "..."` — custom path template
    Custom(String),
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self {
            prefer_no_csd: false,
            screenshot_path: ScreenshotPathConfig::Default,
            disable_primary_clipboard: false,
            hotkey_overlay_skip_at_startup: false,
            hotkey_overlay_hide_not_bound: false,
            config_notification_disable_failed: false,
            spawn_sh_at_startup: Vec::new(),
            spawn_sh_next_id: 0,
            xwayland_satellite: XWaylandSatelliteConfig::Default,
        }
    }
}
