//! Window rules and layer rules

use crate::types::ColorOrGradient;
use nirify_macros::SlintIndex;

use super::layout::{DefaultColumnDisplay, ShadowSettings};

// ============================================================================
// SHARED VALUE TYPES
// ============================================================================

/// A default preset size for a window rule (column width / window height).
///
/// niri models these as `default-column-width { fixed N; }`,
/// `{ proportion F; }`, or an EMPTY block `{}` meaning "natural / unset".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleDefaultSize {
    /// Empty block `{}` — use niri's natural sizing.
    Natural,
    /// `proportion F` (0.0-1.0 of the working area).
    Proportion(f32),
    /// `fixed N` logical pixels.
    Fixed(i32),
}

/// Per-corner geometry corner radius.
///
/// niri accepts either one value (uniform) or four values in the order
/// top-left top-right bottom-right bottom-left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadiusValue {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadiusValue {
    pub fn uniform(r: f32) -> Self {
        Self {
            top_left: r,
            top_right: r,
            bottom_right: r,
            bottom_left: r,
        }
    }

    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }
}

/// Background effect override (blur / xray), shared by window and layer rules.
/// Since niri 26.04.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BackgroundEffectSettings {
    /// See through to the wallpaper (None = follow the window's request).
    pub xray: Option<bool>,
    /// Blur behind the surface (None = follow the window's request).
    pub blur: Option<bool>,
    /// Noise amount (niri: FloatOrInt<0,1000>).
    pub noise: Option<f32>,
    /// Saturation multiplier (niri: FloatOrInt<0,1000>).
    pub saturation: Option<f32>,
}

impl BackgroundEffectSettings {
    /// True when nothing is configured (so the block can be skipped).
    pub fn is_empty(&self) -> bool {
        self.xray.is_none()
            && self.blur.is_none()
            && self.noise.is_none()
            && self.saturation.is_none()
    }
}

/// Popup override block, shared by window and layer rules. Since niri 26.04.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PopupsSettings {
    pub opacity: Option<f32>,
    pub geometry_corner_radius: Option<CornerRadiusValue>,
    pub background_effect: Option<BackgroundEffectSettings>,
}

impl PopupsSettings {
    pub fn is_empty(&self) -> bool {
        self.opacity.is_none()
            && self.geometry_corner_radius.is_none()
            && self
                .background_effect
                .as_ref()
                .map(|b| b.is_empty())
                .unwrap_or(true)
    }
}

/// Per-window tab-indicator override.
///
/// niri's `TabIndicatorRule` only accepts colour/gradient children — NOT the
/// on/off/width/length/etc. that the global tab-indicator config accepts.
/// Gradients use the same [`ColorOrGradient`] model as focus-ring/border.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TabIndicatorOverride {
    pub active: Option<ColorOrGradient>,
    pub inactive: Option<ColorOrGradient>,
    pub urgent: Option<ColorOrGradient>,
}

impl TabIndicatorOverride {
    pub fn is_empty(&self) -> bool {
        self.active.is_none() && self.inactive.is_none() && self.urgent.is_none()
    }
}

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

/// Which layer-shell layer a layer rule matches. Since niri 26.04.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl LayerKind {
    pub fn all() -> &'static [Self] {
        &[Self::Background, Self::Bottom, Self::Top, Self::Overlay]
    }

    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Bottom => "bottom",
            Self::Top => "top",
            Self::Overlay => "overlay",
        }
    }

    pub fn from_kdl(s: &str) -> Option<Self> {
        match s {
            "background" => Some(Self::Background),
            "bottom" => Some(Self::Bottom),
            "top" => Some(Self::Top),
            "overlay" => Some(Self::Overlay),
            _ => None,
        }
    }
}

impl std::fmt::Display for LayerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Background => write!(f, "Background"),
            Self::Bottom => write!(f, "Bottom"),
            Self::Top => write!(f, "Top"),
            Self::Overlay => write!(f, "Overlay"),
        }
    }
}

/// Match criteria for layer rules
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRuleMatch {
    /// Match by namespace (regex)
    pub namespace: Option<String>,
    /// Match only during first 60 seconds after niri launch
    pub at_startup: Option<bool>,
    /// Match by layer-shell layer (Since niri 26.04)
    pub layer: Option<LayerKind>,
}

/// A single layer rule
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRule {
    /// Unique identifier for this rule
    pub id: u32,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Display name for the rule
    pub name: String,
    /// Match criteria (multiple allowed - rule applies if ANY match)
    pub matches: Vec<LayerRuleMatch>,
    /// Exclude criteria (rule does NOT apply if ANY exclude matches)
    pub excludes: Vec<LayerRuleMatch>,
    /// Block layer surface from screencasts/captures
    pub block_out_from: Option<BlockOutFrom>,
    /// Layer opacity (0.0-1.0)
    pub opacity: Option<f32>,
    /// Shadow settings (v25.02+)
    pub shadow: Option<ShadowSettings>,
    /// Corner radius for geometry (v25.02+)
    pub geometry_corner_radius: Option<CornerRadiusValue>,
    /// Place within backdrop (v25.05+)
    pub place_within_backdrop: bool,
    /// Treat as floating for animations (v25.05+)
    pub baba_is_float: bool,
    /// Background effect override (Since 26.04)
    pub background_effect: Option<BackgroundEffectSettings>,
    /// Popup override block (Since 26.04)
    pub popups: Option<PopupsSettings>,
}

impl Default for LayerRule {
    fn default() -> Self {
        Self {
            id: 0,
            enabled: true,
            name: String::from("New Layer Rule"),
            matches: vec![LayerRuleMatch::default()],
            excludes: vec![],
            block_out_from: None,
            opacity: None,
            shadow: None,
            geometry_corner_radius: None,
            place_within_backdrop: false,
            baba_is_float: false,
            background_effect: None,
            popups: None,
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

impl WindowRuleMatch {
    /// True when this match has no criteria (niri treats it as catch-all).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
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
}

impl std::fmt::Display for PositionRelativeTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TopLeft => write!(f, "Top Left"),
            Self::TopRight => write!(f, "Top Right"),
            Self::BottomLeft => write!(f, "Bottom Left"),
            Self::BottomRight => write!(f, "Bottom Right"),
            Self::Top => write!(f, "Top"),
            Self::Bottom => write!(f, "Bottom"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
        }
    }
}

impl PositionRelativeTo {
    pub fn all() -> &'static [Self] {
        &[
            Self::TopLeft,
            Self::TopRight,
            Self::BottomLeft,
            Self::BottomRight,
            Self::Top,
            Self::Bottom,
            Self::Left,
            Self::Right,
        ]
    }

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
        }
    }

    /// Parse from KDL string. Unknown values (incl. the removed "center") fall
    /// back to TopLeft.
    pub fn from_kdl(s: &str) -> Self {
        match s {
            "top-right" => Self::TopRight,
            "bottom-left" => Self::BottomLeft,
            "bottom-right" => Self::BottomRight,
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "right" => Self::Right,
            "top-left" => Self::TopLeft,
            _ => {
                log::warn!("Unknown relative-to value {:?}, using top-left", s);
                Self::TopLeft
            }
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
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Display name for the rule
    pub name: String,
    /// Match criteria (multiple allowed - rule applies if ANY match)
    pub matches: Vec<WindowRuleMatch>,
    /// Exclude criteria (multiple allowed - rule doesn't apply if ANY exclude matches)
    pub excludes: Vec<WindowRuleMatch>,

    // Opening behaviour — each independent (None = don't emit).
    /// Open maximized within the column
    pub open_maximized: Option<bool>,
    /// Open fullscreen
    pub open_fullscreen: Option<bool>,
    /// Open floating
    pub open_floating: Option<bool>,
    /// Maximize to screen edges instead of columns (v25.11+)
    pub open_maximized_to_edges: Option<bool>,
    /// Open focused
    pub open_focused: Option<bool>,

    /// Window opacity (0.0-1.0, None = default)
    pub opacity: Option<f32>,
    /// Block from screencasts / all captures
    pub block_out_from: Option<BlockOutFrom>,
    /// Custom corner radius (None = use global)
    pub corner_radius: Option<CornerRadiusValue>,
    /// Clip window to visual geometry (cuts shadows, rounds corners)
    pub clip_to_geometry: Option<bool>,
    /// Open on specific output
    pub open_on_output: Option<String>,
    /// Open on specific workspace
    pub open_on_workspace: Option<String>,
    /// Default floating window position
    pub default_floating_position: Option<FloatingPosition>,
    /// Default column width
    pub default_column_width: Option<RuleDefaultSize>,
    /// Default window height
    pub default_window_height: Option<RuleDefaultSize>,

    // Dynamic properties
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
    /// Focus ring enabled override (Some(false) = off)
    pub focus_ring_enabled: Option<bool>,
    /// Focus ring width override
    pub focus_ring_width: Option<i32>,
    /// Focus ring active color override
    pub focus_ring_active: Option<ColorOrGradient>,
    /// Focus ring inactive color override
    pub focus_ring_inactive: Option<ColorOrGradient>,
    /// Focus ring urgent color override
    pub focus_ring_urgent: Option<ColorOrGradient>,
    /// Border enabled override (Some(false) = off)
    pub border_enabled: Option<bool>,
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
    /// Custom tab indicator colours for this window
    pub tab_indicator: Option<TabIndicatorOverride>,
    /// Background effect override (Since 26.04)
    pub background_effect: Option<BackgroundEffectSettings>,
    /// Popup override block (Since 26.04)
    pub popups: Option<PopupsSettings>,
    /// Mark window as tiled (for X11 compatibility)
    pub tiled_state: Option<bool>,
    /// Animated floating effect (v25.05+)
    pub baba_is_float: Option<bool>,
}

impl Default for WindowRule {
    fn default() -> Self {
        Self {
            id: 0,
            enabled: true,
            name: String::from("New Rule"),
            matches: vec![WindowRuleMatch::default()],
            excludes: vec![],
            open_maximized: None,
            open_fullscreen: None,
            open_floating: None,
            open_maximized_to_edges: None,
            open_focused: None,
            opacity: None,
            block_out_from: None,
            corner_radius: None,
            clip_to_geometry: None,
            open_on_output: None,
            open_on_workspace: None,
            default_floating_position: None,
            default_column_width: None,
            default_window_height: None,
            scroll_factor: None,
            draw_border_with_background: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            focus_ring_enabled: None,
            focus_ring_width: None,
            focus_ring_active: None,
            focus_ring_inactive: None,
            focus_ring_urgent: None,
            border_enabled: None,
            border_width: None,
            border_active: None,
            border_inactive: None,
            border_urgent: None,
            variable_refresh_rate: None,
            default_column_display: None,
            shadow: None,
            tab_indicator: None,
            background_effect: None,
            popups: None,
            tiled_state: None,
            baba_is_float: None,
        }
    }
}

impl WindowRule {
    /// True when this rule has no match criteria (niri catch-all).
    ///
    /// The loader injects a default empty `match` when none is present, so
    /// "no criteria" and "no match nodes" are the same catch-all identity.
    #[must_use]
    pub fn is_catch_all(&self) -> bool {
        self.matches.iter().all(WindowRuleMatch::is_empty)
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

    /// First catch-all (no-match) window-rule, if any.
    #[must_use]
    pub fn catch_all(&self) -> Option<&WindowRule> {
        self.rules.iter().find(|r| r.is_catch_all())
    }

    /// First catch-all (no-match) window-rule, if any.
    pub fn catch_all_mut(&mut self) -> Option<&mut WindowRule> {
        self.rules.iter_mut().find(|r| r.is_catch_all())
    }

    /// True when a managed window-rule already acts as a niri catch-all.
    #[must_use]
    pub fn has_catch_all(&self) -> bool {
        self.rules.iter().any(WindowRule::is_catch_all)
    }
}

/// Copy `geometry-corner-radius` from the first catch-all into Appearance.
///
/// Appearance radius is a view over that distinguished catch-all. If the
/// catch-all has no radius, the Appearance field is left unchanged.
pub fn sync_appearance_radius_from_catch_all(
    appearance: &mut super::appearance::AppearanceSettings,
    window_rules: &WindowRulesSettings,
) {
    if let Some(rule) = window_rules.catch_all() {
        if let Some(cr) = rule.corner_radius {
            appearance.corner_radius = cr.top_left;
        }
    }
}

/// Write Appearance radius onto the first catch-all (creating none).
///
/// Used when the user edits the Appearance slider and a catch-all already
/// exists, so we do not emit a second radius-only rule from `appearance.kdl`.
pub fn apply_appearance_radius_to_catch_all(
    appearance: &super::appearance::AppearanceSettings,
    window_rules: &mut WindowRulesSettings,
) -> bool {
    let Some(rule) = window_rules.catch_all_mut() else {
        return false;
    };
    rule.corner_radius = if appearance.corner_radius > 0.0 {
        Some(CornerRadiusValue::uniform(appearance.corner_radius))
    } else {
        None
    };
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_to_center_falls_back_to_top_left() {
        assert_eq!(
            PositionRelativeTo::from_kdl("center"),
            PositionRelativeTo::TopLeft
        );
        assert_eq!(
            PositionRelativeTo::from_kdl("garbage"),
            PositionRelativeTo::TopLeft
        );
        assert_eq!(
            PositionRelativeTo::from_kdl("bottom-right"),
            PositionRelativeTo::BottomRight
        );
    }

    #[test]
    fn corner_radius_uniform_detection() {
        assert!(CornerRadiusValue::uniform(8.0).is_uniform());
        let cr = CornerRadiusValue {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        };
        assert!(!cr.is_uniform());
    }

    #[test]
    fn catch_all_treats_empty_injected_match_as_global() {
        let empty = WindowRule::default();
        assert!(empty.is_catch_all());
        let matched = WindowRule {
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(!matched.is_catch_all());
    }
}
