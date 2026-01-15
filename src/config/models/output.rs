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
    pub scale: f64,
    pub mode: String, // e.g., "1920x1080@60.000"
    /// Whether mode uses custom=true flag (v25.11+)
    pub mode_custom: bool,
    /// Custom modeline string (v25.11+) - WARNING: can damage monitors
    pub modeline: Option<String>,
    pub position_x: i32,
    pub position_y: i32,
    pub transform: Transform,
    pub vrr: VrrMode,
    pub focus_at_startup: bool,
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
            scale: 1.0,
            mode: String::new(),
            mode_custom: false,
            modeline: None,
            position_x: 0,
            position_y: 0,
            transform: Transform::Normal,
            vrr: VrrMode::Off,
            focus_at_startup: false,
            backdrop_color: None,
            hot_corners: None,
            layout_override: None,
        }
    }
}

/// Display/output settings - holds configured outputs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutputSettings {
    pub outputs: Vec<OutputConfig>,
}
