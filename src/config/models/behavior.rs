//! Behavior settings (focus, workspace layout, struts, modifier keys)

use crate::constants::{DEFAULT_COLUMN_FIXED, DEFAULT_COLUMN_PROPORTION};
use crate::types::{CenterFocusedColumn, ModKey, WarpMouseMode};
use nirify_macros::SlintIndex;

/// Behavior settings (focus, workspace layout, struts)
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorSettings {
    // Focus
    pub focus_follows_mouse: bool,
    pub focus_follows_mouse_max_scroll_amount: Option<f32>, // percentage
    pub warp_mouse_to_focus: WarpMouseMode,

    // Workspace layout
    pub center_focused_column: CenterFocusedColumn,
    pub always_center_single_column: bool,
    pub empty_workspace_above_first: bool,

    // Default column width
    pub default_column_width_type: ColumnWidthType,
    pub default_column_width_proportion: f32,
    pub default_column_width_fixed: f32,

    // Struts (screen edge reserved space)
    pub strut_left: f32,
    pub strut_right: f32,
    pub strut_top: f32,
    pub strut_bottom: f32,

    // Modifier keys
    pub mod_key: ModKey,
    /// Modifier key when niri runs nested inside another compositor
    pub mod_key_nested: Option<ModKey>,
    pub workspace_auto_back_and_forth: bool,

    // Power handling
    /// Let the system handle the power button instead of niri
    pub disable_power_key_handling: bool,
    // Note: prefer_no_csd, screenshot_path, and hotkey_overlay_skip_at_startup
    // are in MiscSettings to avoid duplication
}

/// Column width type for new windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum ColumnWidthType {
    #[default]
    #[slint_index(default)]
    Proportion,
    Fixed,
    /// Empty `default-column-width {}` — new windows pick their own initial width.
    Auto,
}

impl std::fmt::Display for ColumnWidthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proportion => write!(f, "Proportion"),
            Self::Fixed => write!(f, "Fixed"),
            Self::Auto => write!(f, "Automatic"),
        }
    }
}

impl ColumnWidthType {
    pub fn all() -> &'static [Self] {
        &[Self::Proportion, Self::Fixed, Self::Auto]
    }
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            focus_follows_mouse: false,
            focus_follows_mouse_max_scroll_amount: None,
            warp_mouse_to_focus: WarpMouseMode::Off,
            center_focused_column: CenterFocusedColumn::Never,
            always_center_single_column: false,
            empty_workspace_above_first: false,
            default_column_width_type: ColumnWidthType::Proportion,
            default_column_width_proportion: DEFAULT_COLUMN_PROPORTION,
            default_column_width_fixed: DEFAULT_COLUMN_FIXED,
            strut_left: 0.0,
            strut_right: 0.0,
            strut_top: 0.0,
            strut_bottom: 0.0,
            mod_key: ModKey::default(),
            mod_key_nested: None,
            workspace_auto_back_and_forth: false,
            disable_power_key_handling: false,
        }
    }
}
