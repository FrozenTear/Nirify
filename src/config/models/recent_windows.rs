//! Recent windows switcher settings (v25.05+)

use crate::types::Color;

/// Highlight style settings for recent windows switcher
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsHighlight {
    /// Active window highlight color
    pub active_color: Color,
    /// Urgent window highlight color
    pub urgent_color: Color,
    /// Highlight padding in logical pixels
    pub padding: i32,
    /// Corner radius for highlight
    pub corner_radius: i32,
}

impl Default for RecentWindowsHighlight {
    fn default() -> Self {
        Self {
            active_color: Color::from_hex("#7fc8ff").unwrap_or_default(),
            urgent_color: Color::from_hex("#eb6f92").unwrap_or_default(),
            padding: 8,
            corner_radius: 12,
        }
    }
}

/// Preview settings for recent windows switcher
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsPreviews {
    /// Maximum height of previews in logical pixels
    pub max_height: i32,
    /// Maximum scale factor for previews (0.0-1.0)
    pub max_scale: f64,
}

impl Default for RecentWindowsPreviews {
    fn default() -> Self {
        Self {
            max_height: 200,
            max_scale: 0.5,
        }
    }
}

/// Recent windows switcher settings (v25.05+)
///
/// Configures the recent windows (Alt-Tab) switcher appearance and behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsSettings {
    /// Whether the recent windows switcher is disabled
    pub off: bool,
    /// Delay before the window is committed to the recent list (ms)
    pub debounce_ms: i32,
    /// Delay before the switcher UI appears (ms)
    pub open_delay_ms: i32,
    /// Highlight styling
    pub highlight: RecentWindowsHighlight,
    /// Preview settings
    pub previews: RecentWindowsPreviews,
}

impl Default for RecentWindowsSettings {
    fn default() -> Self {
        Self {
            off: false,
            debounce_ms: 100,
            open_delay_ms: 200,
            highlight: RecentWindowsHighlight::default(),
            previews: RecentWindowsPreviews::default(),
        }
    }
}
