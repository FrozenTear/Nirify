//! UI index constants and helper functions
//!
//! This module contains:
//! - Constants for UI index values that aren't derived from enums
//! - Helper functions for boolean conversions
//!
//! Note: Enum-to-index conversions are now generated via #[derive(SlintIndex)]
//! in types.rs and models.rs. Use `EnumType::to_index()` and `EnumType::from_index()`
//! instead of the old `xxx_to_index()` and `index_to_xxx()` functions.

use crate::types::{CenterFocusedColumn, WarpMouseMode};

// ============================================================================
// TRACK LAYOUT (Keyboard)
// ============================================================================

/// Track layout index for "global"
pub const TRACK_LAYOUT_GLOBAL: i32 = 0;

/// Track layout index for "window"
pub const TRACK_LAYOUT_WINDOW: i32 = 1;

// ============================================================================
// OUTPUT SCALE LIMITS
// ============================================================================

/// Minimum output scale value
pub const OUTPUT_SCALE_MIN: f64 = 0.5;

/// Maximum output scale value
pub const OUTPUT_SCALE_MAX: f64 = 3.0;

// ============================================================================
// WARP MOUSE MODE (Behavior)
// ============================================================================

/// Check if WarpMouseMode is enabled (any non-Off value)
#[inline]
pub fn warp_mouse_is_enabled(mode: &WarpMouseMode) -> bool {
    !matches!(mode, WarpMouseMode::Off)
}

// ============================================================================
// CENTER FOCUSED COLUMN (Behavior)
// ============================================================================

/// Check if CenterFocusedColumn is enabled (any non-Never value)
#[inline]
pub fn center_focused_is_enabled(mode: &CenterFocusedColumn) -> bool {
    !matches!(mode, CenterFocusedColumn::Never)
}
