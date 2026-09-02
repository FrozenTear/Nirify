//! Output/display configuration

use crate::types::{Color, Transform, VrrMode};

use super::layout::LayoutOverride;

/// Per-output hot corners configuration (v25.11+)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutputHotCorners {
    /// Whether hot corners are enabled for this output (None = use global)
    pub enabled: Option<bool>,
    /// Top-left corner enabled
    pub top_left: bool,
    /// Top-right corner enabled
    pub top_right: bool,
    /// Bottom-left corner enabled
    pub bottom_left: bool,
    /// Bottom-right corner enabled
    pub bottom_right: bool,
}

impl OutputHotCorners {
    /// Returns true if any corner is enabled
    pub fn has_any_enabled(&self) -> bool {
        self.top_left || self.top_right || self.bottom_left || self.bottom_right
    }

    /// Returns true if this is just "off" (disabled)
    pub fn is_off(&self) -> bool {
        self.enabled == Some(false) && !self.has_any_enabled()
    }
}

/// Single output/display configuration
#[derive(Debug, Clone, PartialEq)]
pub struct OutputConfig {
    pub name: String,
    pub enabled: bool,
    /// Output scale. `None` = omit from KDL (niri auto-guesses from
    /// physical size/resolution). `Some(1.0)` is an explicit 1× and must
    /// be written — niri does **not** treat unset scale as 1.0.
    pub scale: Option<f64>,
    pub mode: String, // e.g., "1920x1080@60.000"
    /// Whether mode uses custom=true flag (v25.11+)
    pub mode_custom: bool,
    /// Custom modeline string (v25.11+) - WARNING: can damage monitors
    pub modeline: Option<String>,
    /// Explicit position; `None` means automatic placement by niri
    pub position: Option<(i32, i32)>,
    pub transform: Transform,
    pub vrr: VrrMode,
    pub focus_at_startup: bool,
    /// Per-output solid background color behind windows (niri Since 0.1.8)
    pub background_color: Option<Color>,
    pub backdrop_color: Option<Color>,
    /// Per-output hot corners (v25.11+)
    pub hot_corners: Option<OutputHotCorners>,
    /// Per-output layout override (v25.11+)
    pub layout_override: Option<LayoutOverride>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            scale: None,
            mode: String::new(),
            mode_custom: false,
            modeline: None,
            position: None,
            transform: Transform::Normal,
            vrr: VrrMode::Off,
            focus_at_startup: false,
            background_color: None,
            backdrop_color: None,
            hot_corners: None,
            layout_override: None,
        }
    }
}

impl OutputConfig {
    /// Scale for UI sliders and logical-size estimates.
    ///
    /// Unset (`None`, niri auto-guess) displays as 1.0 until the user picks
    /// an explicit value. Layout helpers should still prefer live IPC size
    /// when available.
    #[must_use]
    pub fn display_scale(&self) -> f64 {
        self.scale.unwrap_or(1.0)
    }
}

/// Display/output settings - holds configured outputs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutputSettings {
    pub outputs: Vec<OutputConfig>,
}
