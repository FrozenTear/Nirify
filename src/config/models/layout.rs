//! Layout settings (shadows, tab indicators, presets, overview)

use crate::constants::DEFAULT_OVERVIEW_ZOOM;
use crate::types::{CenterFocusedColumn, Color, ColorOrGradient};
use nirify_macros::SlintIndex;

/// Workspace shadow settings for overview (v25.05+)
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceShadow {
    /// Whether workspace shadow is enabled (off flag disables)
    pub enabled: bool,
    /// Shadow blur radius
    pub softness: i32,
    /// Shadow expansion
    pub spread: i32,
    /// Shadow X offset
    pub offset_x: i32,
    /// Shadow Y offset
    pub offset_y: i32,
    /// Shadow color with alpha
    pub color: Color,
}

impl Default for WorkspaceShadow {
    fn default() -> Self {
        Self {
            enabled: true,
            softness: 40,
            spread: 10,
            offset_x: 0,
            offset_y: 10,
            color: Color::from_hex("#00000050").unwrap_or_default(),
        }
    }
}

/// Overview settings
#[derive(Debug, Clone, PartialEq)]
pub struct OverviewSettings {
    pub zoom: f64,
    pub backdrop_color: Option<Color>,
    /// Workspace shadow in overview (v25.05+)
    pub workspace_shadow: Option<WorkspaceShadow>,
}

impl Default for OverviewSettings {
    fn default() -> Self {
        Self {
            zoom: DEFAULT_OVERVIEW_ZOOM,
            backdrop_color: None,
            workspace_shadow: None,
        }
    }
}

/// Shadow settings for windows
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowSettings {
    pub enabled: bool,
    pub softness: i32,
    pub spread: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub draw_behind_window: bool,
    pub color: Color,
    pub inactive_color: Color,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            softness: 30,
            spread: 5,
            offset_x: 0,
            offset_y: 5,
            draw_behind_window: false,
            color: Color::from_hex("#00000070").unwrap_or_default(),
            inactive_color: Color::from_hex("#00000050").unwrap_or_default(),
        }
    }
}

/// Tab indicator position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum TabIndicatorPosition {
    #[default]
    #[slint_index(default)]
    Left,
    Right,
    Top,
    Bottom,
}

impl std::fmt::Display for TabIndicatorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabIndicatorPosition::Left => write!(f, "Left"),
            TabIndicatorPosition::Right => write!(f, "Right"),
            TabIndicatorPosition::Top => write!(f, "Top"),
            TabIndicatorPosition::Bottom => write!(f, "Bottom"),
        }
    }
}

/// Tab indicator settings
#[derive(Debug, Clone, PartialEq)]
pub struct TabIndicatorSettings {
    pub enabled: bool,
    pub hide_when_single_tab: bool,
    pub place_within_column: bool,
    pub gap: i32,
    pub width: i32,
    pub length_proportion: f32,
    pub position: TabIndicatorPosition,
    pub gaps_between_tabs: i32,
    pub corner_radius: i32,
    pub active: ColorOrGradient,
    pub inactive: ColorOrGradient,
    pub urgent: ColorOrGradient,
}

impl Default for TabIndicatorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            hide_when_single_tab: false,
            place_within_column: false,
            gap: 5,
            width: 4,
            length_proportion: 1.0,
            position: TabIndicatorPosition::Left,
            gaps_between_tabs: 2,
            corner_radius: 8,
            active: ColorOrGradient::Color(Color::from_hex("#7fc8ff").unwrap_or_default()),
            inactive: ColorOrGradient::Color(Color::from_hex("#505050").unwrap_or_default()),
            urgent: ColorOrGradient::Color(Color::from_hex("#eb6f92").unwrap_or_default()),
        }
    }
}

/// Insert hint settings
#[derive(Debug, Clone, PartialEq)]
pub struct InsertHintSettings {
    pub enabled: bool,
    pub color: ColorOrGradient,
}

impl Default for InsertHintSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            color: ColorOrGradient::Color(Color::from_hex("#ffc87f80").unwrap_or_default()),
        }
    }
}

/// Preset column width entry
#[derive(Debug, Clone, PartialEq)]
pub enum PresetWidth {
    Proportion(f32),
    Fixed(i32),
}

/// Preset window height entry
#[derive(Debug, Clone, PartialEq)]
pub enum PresetHeight {
    Proportion(f32),
    Fixed(i32),
}

/// Default column display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefaultColumnDisplay {
    /// Normal column display
    #[default]
    Normal,
    /// Tabbed display (windows stacked as tabs)
    Tabbed,
}

impl std::fmt::Display for DefaultColumnDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultColumnDisplay::Normal => write!(f, "Normal"),
            DefaultColumnDisplay::Tabbed => write!(f, "Tabbed"),
        }
    }
}

/// Layout extras settings
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutExtrasSettings {
    pub shadow: ShadowSettings,
    pub tab_indicator: TabIndicatorSettings,
    pub insert_hint: InsertHintSettings,
    pub preset_column_widths: Vec<PresetWidth>,
    pub preset_window_heights: Vec<PresetHeight>,
    pub default_column_display: DefaultColumnDisplay,
}

impl Default for LayoutExtrasSettings {
    fn default() -> Self {
        Self {
            shadow: ShadowSettings::default(),
            tab_indicator: TabIndicatorSettings::default(),
            insert_hint: InsertHintSettings::default(),
            preset_column_widths: vec![
                PresetWidth::Proportion(0.33333),
                PresetWidth::Proportion(0.5),
                PresetWidth::Proportion(0.66667),
            ],
            preset_window_heights: vec![
                PresetHeight::Proportion(0.33333),
                PresetHeight::Proportion(0.5),
                PresetHeight::Proportion(0.66667),
            ],
            default_column_display: DefaultColumnDisplay::Normal,
        }
    }
}

/// Layout overrides for per-output or per-workspace configuration (v25.11+)
///
/// All fields are optional - only set fields will be output to KDL.
/// Niri allows overriding "all the same options as the top-level layout {}"
/// for individual outputs and named workspaces.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayoutOverride {
    // === Gaps and Struts ===
    /// Gaps override (single value used for all gaps)
    pub gaps: Option<f32>,
    /// Strut left override
    pub strut_left: Option<f32>,
    /// Strut right override
    pub strut_right: Option<f32>,
    /// Strut top override
    pub strut_top: Option<f32>,
    /// Strut bottom override
    pub strut_bottom: Option<f32>,

    // === Column/Window Behavior ===
    /// Center focused column override
    pub center_focused_column: Option<CenterFocusedColumn>,
    /// Always center single column override
    pub always_center_single_column: Option<bool>,
    /// Default column display mode override (normal/tabbed)
    pub default_column_display: Option<DefaultColumnDisplay>,

    // === Default Sizing ===
    /// Default column width as proportion (0.0-1.0)
    pub default_column_width_proportion: Option<f32>,
    /// Default column width as fixed pixels
    pub default_column_width_fixed: Option<i32>,
    /// Preset column widths for switch-preset-column-width
    pub preset_column_widths: Option<Vec<PresetWidth>>,
    /// Preset window heights for switch-preset-window-height
    pub preset_window_heights: Option<Vec<PresetHeight>>,

    // === Focus Ring ===
    /// Focus ring enabled override (false = off)
    pub focus_ring_enabled: Option<bool>,
    /// Focus ring width override
    pub focus_ring_width: Option<i32>,
    /// Focus ring active color override
    pub focus_ring_active: Option<ColorOrGradient>,
    /// Focus ring inactive color override
    pub focus_ring_inactive: Option<ColorOrGradient>,

    // === Border ===
    /// Border enabled override (false = off)
    pub border_enabled: Option<bool>,
    /// Border width override
    pub border_width: Option<i32>,
    /// Border active color override
    pub border_active: Option<ColorOrGradient>,
    /// Border inactive color override
    pub border_inactive: Option<ColorOrGradient>,

    // === Shadow ===
    /// Shadow enabled override (false = off)
    pub shadow_enabled: Option<bool>,
    /// Shadow softness (blur radius) override
    pub shadow_softness: Option<i32>,
    /// Shadow spread override
    pub shadow_spread: Option<i32>,
    /// Shadow X offset override
    pub shadow_offset_x: Option<i32>,
    /// Shadow Y offset override
    pub shadow_offset_y: Option<i32>,
    /// Shadow color override
    pub shadow_color: Option<Color>,
}

impl LayoutOverride {
    /// Returns true if any field is set (not all None)
    pub fn has_any(&self) -> bool {
        self.gaps.is_some()
            || self.strut_left.is_some()
            || self.strut_right.is_some()
            || self.strut_top.is_some()
            || self.strut_bottom.is_some()
            || self.center_focused_column.is_some()
            || self.always_center_single_column.is_some()
            || self.default_column_display.is_some()
            || self.default_column_width_proportion.is_some()
            || self.default_column_width_fixed.is_some()
            || self.preset_column_widths.is_some()
            || self.preset_window_heights.is_some()
            || self.focus_ring_enabled.is_some()
            || self.focus_ring_width.is_some()
            || self.focus_ring_active.is_some()
            || self.focus_ring_inactive.is_some()
            || self.border_enabled.is_some()
            || self.border_width.is_some()
            || self.border_active.is_some()
            || self.border_inactive.is_some()
            || self.shadow_enabled.is_some()
            || self.shadow_softness.is_some()
            || self.shadow_spread.is_some()
            || self.shadow_offset_x.is_some()
            || self.shadow_offset_y.is_some()
            || self.shadow_color.is_some()
    }
}
