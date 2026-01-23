//! Window rules and layer rules

use crate::types::ColorOrGradient;
use nirify_macros::SlintIndex;

use super::layout::{DefaultColumnDisplay, ShadowSettings, TabIndicatorSettings};

// ============================================================================
// LAYER RULES
// ============================================================================

/// What to block layer surfaces from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum BlockOutFrom {
    /// Block from screencasts only
    #[default]
    #[slint_index(default)]
    Screencast,
    /// Block from all screen captures (screenshots and screencasts)
    ScreenCapture,
}

/// Match criteria for layer rules
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRuleMatch {
    /// Match by namespace (regex)
    pub namespace: Option<String>,
    /// Match only during first 60 seconds after niri launch
    pub at_startup: Option<bool>,
}

/// A single layer rule
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRule {
    /// Unique identifier for this rule
    pub id: u32,
    /// Display name for the rule
    pub name: String,
    /// Match criteria (multiple allowed - rule applies if ANY match)
    pub matches: Vec<LayerRuleMatch>,
    /// Block layer surface from screencasts/captures
    pub block_out_from: Option<BlockOutFrom>,
    /// Layer opacity (0.0-1.0)
    pub opacity: Option<f32>,
    /// Shadow settings (v25.02+)
    pub shadow: Option<ShadowSettings>,
    /// Corner radius for geometry (v25.02+)
    pub geometry_corner_radius: Option<i32>,
    /// Place within backdrop (v25.05+)
    pub place_within_backdrop: bool,
    /// Treat as floating for animations (v25.05+)
    pub baba_is_float: bool,
}

impl Default for LayerRule {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("New Layer Rule"),
            matches: vec![LayerRuleMatch::default()],
            block_out_from: None,
            opacity: None,
            shadow: None,
            geometry_corner_radius: None,
            place_within_backdrop: false,
            baba_is_float: false,
        }
    }
}

/// Layer rules settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRulesSettings {
    pub rules: Vec<LayerRule>,
    /// Counter for generating unique IDs
    pub next_id: u32,
}

impl LayerRulesSettings {
    /// Find a layer rule by ID (immutable)
    pub fn find(&self, id: u32) -> Option<&LayerRule> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Find a layer rule by ID (mutable)
    pub fn find_mut(&mut self, id: u32) -> Option<&mut LayerRule> {
        self.rules.iter_mut().find(|r| r.id == id)
    }

    /// Remove a rule by ID, returns true if removed
    pub fn remove(&mut self, id: u32) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() < len_before
    }
}

// ============================================================================
// WINDOW RULES
// ============================================================================

/// Match criteria for window rules
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowRuleMatch {
    /// Match by app-id (regex)
    pub app_id: Option<String>,
    /// Match by window title (regex)
    pub title: Option<String>,
    /// Match floating windows
    pub is_floating: Option<bool>,
    /// Match window with active border/focus ring
    pub is_active: Option<bool>,
    /// Match window with keyboard focus
    pub is_focused: Option<bool>,
    /// Match last-focused window in column (v0.1.6+)
    pub is_active_in_column: Option<bool>,
    /// Match window being screencast/recorded (v25.02+)
    pub is_window_cast_target: Option<bool>,
    /// Match window requesting attention/urgent (v25.05+)
    pub is_urgent: Option<bool>,
    /// Match only during first 60 seconds after niri launch (v0.1.6+)
    pub at_startup: Option<bool>,
}

/// Opening behavior for window rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum OpenBehavior {
    #[default]
    #[slint_index(default)]
    Normal,
    Maximized,
    Fullscreen,
    Floating,
}

/// Position reference point for floating windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum PositionRelativeTo {
    #[default]
    #[slint_index(default)]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl PositionRelativeTo {
    /// Convert to KDL string
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
        }
    }

    /// Parse from KDL string
    pub fn from_kdl(s: &str) -> Self {
        match s {
            "top-right" => Self::TopRight,
            "bottom-left" => Self::BottomLeft,
            "bottom-right" => Self::BottomRight,
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "right" => Self::Right,
            "center" => Self::Center,
            _ => Self::TopLeft,
        }
    }
}

/// Default floating window position
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FloatingPosition {
    /// X coordinate in logical pixels
    pub x: i32,
    /// Y coordinate in logical pixels
    pub y: i32,
    /// Which edge/corner the position is relative to
    pub relative_to: PositionRelativeTo,
}

/// A single window rule
#[derive(Debug, Clone, PartialEq)]
pub struct WindowRule {
    /// Unique identifier for this rule
    pub id: u32,
    /// Display name for the rule
    pub name: String,
    /// Match criteria (multiple allowed - rule applies if ANY match)
    pub matches: Vec<WindowRuleMatch>,
    /// Exclude criteria (multiple allowed - rule doesn't apply if ANY exclude matches)
    pub excludes: Vec<WindowRuleMatch>,
    /// Opening behavior
    pub open_behavior: OpenBehavior,
    /// Window opacity (0.0-1.0, None = default)
    pub opacity: Option<f32>,
    /// Block from screencasts
    pub block_out_from_screencast: bool,
    /// Custom corner radius (None = use global)
    pub corner_radius: Option<i32>,
    /// Clip window to visual geometry (cuts shadows, rounds corners)
    pub clip_to_geometry: Option<bool>,
    /// Open focused
    pub open_focused: Option<bool>,
    /// Open on specific output
    pub open_on_output: Option<String>,
    /// Open on specific workspace
    pub open_on_workspace: Option<String>,
    /// Default floating window position
    pub default_floating_position: Option<FloatingPosition>,
    /// Default column width proportion (0.0-1.0)
    pub default_column_width: Option<f32>,

    // New opening properties (v25.01+)
    /// Default window height proportion (0.0-1.0, None = auto)
    pub default_window_height: Option<f32>,
    /// Maximize to screen edges instead of columns (v25.11+)
    pub open_maximized_to_edges: Option<bool>,

    // New dynamic properties
    /// Per-window scroll factor (v25.02+)
    pub scroll_factor: Option<f64>,
    /// Draw border with background
    pub draw_border_with_background: Option<bool>,
    /// Minimum window width
    pub min_width: Option<i32>,
    /// Maximum window width
    pub max_width: Option<i32>,
    /// Minimum window height
    pub min_height: Option<i32>,
    /// Maximum window height
    pub max_height: Option<i32>,

    // Styling overrides
    /// Focus ring width override
    pub focus_ring_width: Option<i32>,
    /// Focus ring active color override
    pub focus_ring_active: Option<ColorOrGradient>,
    /// Focus ring inactive color override
    pub focus_ring_inactive: Option<ColorOrGradient>,
    /// Focus ring urgent color override
    pub focus_ring_urgent: Option<ColorOrGradient>,
    /// Border width override
    pub border_width: Option<i32>,
    /// Border active color override
    pub border_active: Option<ColorOrGradient>,
    /// Border inactive color override
    pub border_inactive: Option<ColorOrGradient>,
    /// Border urgent color override
    pub border_urgent: Option<ColorOrGradient>,

    // Additional dynamic properties
    /// Enable VRR for this window
    pub variable_refresh_rate: Option<bool>,
    /// Column display mode (Normal/Tabbed) for this window
    pub default_column_display: Option<DefaultColumnDisplay>,
    /// Custom shadow settings for this window
    pub shadow: Option<ShadowSettings>,
    /// Custom tab indicator for this window
    pub tab_indicator: Option<TabIndicatorSettings>,
    /// Mark window as tiled (for X11 compatibility)
    pub tiled_state: Option<bool>,
    /// Animated floating effect (v25.05+)
    pub baba_is_float: Option<bool>,
}

impl Default for WindowRule {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("New Rule"),
            matches: vec![WindowRuleMatch::default()],
            excludes: vec![],
            open_behavior: OpenBehavior::Normal,
            opacity: None,
            block_out_from_screencast: false,
            corner_radius: None,
            clip_to_geometry: None,
            open_focused: None,
            open_on_output: None,
            open_on_workspace: None,
            default_floating_position: None,
            default_column_width: None,
            // New opening properties
            default_window_height: None,
            open_maximized_to_edges: None,
            // New dynamic properties
            scroll_factor: None,
            draw_border_with_background: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            // Styling overrides
            focus_ring_width: None,
            focus_ring_active: None,
            focus_ring_inactive: None,
            focus_ring_urgent: None,
            border_width: None,
            border_active: None,
            border_inactive: None,
            border_urgent: None,
            // Additional dynamic properties
            variable_refresh_rate: None,
            default_column_display: None,
            shadow: None,
            tab_indicator: None,
            tiled_state: None,
            baba_is_float: None,
        }
    }
}

/// Window rules settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowRulesSettings {
    pub rules: Vec<WindowRule>,
    /// Counter for generating unique IDs
    pub next_id: u32,
}

impl WindowRulesSettings {
    /// Find a window rule by ID (immutable)
    pub fn find(&self, id: u32) -> Option<&WindowRule> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Find a window rule by ID (mutable)
    pub fn find_mut(&mut self, id: u32) -> Option<&mut WindowRule> {
        self.rules.iter_mut().find(|r| r.id == id)
    }

    /// Remove a rule by ID, returns true if removed
    pub fn remove(&mut self, id: u32) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() < len_before
    }
}
