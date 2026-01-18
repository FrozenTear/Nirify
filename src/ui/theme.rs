//! Theme colors and styling helpers for the niri-settings UI
//!
//! Uses a dark theme inspired by Catppuccin Mocha.

use floem::peniko::Color;
use floem::style::Style;

// ============================================================================
// Color Palette - Catppuccin Mocha inspired
// ============================================================================

/// Deep background (sidebar, navigation)
pub const BG_DEEP: Color = Color::from_rgb8(0x0e, 0x0e, 0x12);

/// Base background (main content area)
pub const BG_BASE: Color = Color::from_rgb8(0x14, 0x14, 0x18);

/// Surface background (cards, sections)
pub const BG_SURFACE: Color = Color::from_rgb8(0x1a, 0x1a, 0x20);

/// Elevated surface (hover states, dropdowns)
pub const BG_ELEVATED: Color = Color::from_rgb8(0x20, 0x20, 0x28);

/// Primary accent color (buttons, focus)
pub const ACCENT: Color = Color::from_rgb8(0x95, 0x80, 0xff);

/// Accent hover state
pub const ACCENT_HOVER: Color = Color::from_rgb8(0xa5, 0x90, 0xff);

/// Success/positive color
pub const SUCCESS: Color = Color::from_rgb8(0x34, 0xc7, 0x59);

/// Warning color
pub const WARNING: Color = Color::from_rgb8(0xff, 0xb3, 0x4d);

/// Error/danger color
pub const ERROR: Color = Color::from_rgb8(0xf0, 0x56, 0x56);

/// Primary text color
pub const TEXT_PRIMARY: Color = Color::from_rgb8(0xe8, 0xe8, 0xf0);

/// Secondary text color (dimmed)
pub const TEXT_SECONDARY: Color = Color::from_rgb8(0x90, 0x90, 0xa0);

/// Muted text color (disabled, placeholders)
pub const TEXT_MUTED: Color = Color::from_rgb8(0x60, 0x60, 0x70);

/// Border color
pub const BORDER: Color = Color::from_rgb8(0x30, 0x30, 0x40);

/// Border color for focused elements
pub const BORDER_FOCUS: Color = Color::from_rgb8(0x95, 0x80, 0xff);

// ============================================================================
// Spacing Constants
// ============================================================================

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;
pub const SPACING_XXL: f32 = 32.0;

pub const BORDER_RADIUS_SM: f32 = 4.0;
pub const BORDER_RADIUS_MD: f32 = 8.0;
pub const BORDER_RADIUS_LG: f32 = 12.0;

// ============================================================================
// Style Helpers
// ============================================================================

/// Apply section card styling
pub fn section_style(s: Style) -> Style {
    s.background(BG_SURFACE)
        .border_radius(BORDER_RADIUS_LG)
        .padding(SPACING_LG)
        .margin_bottom(SPACING_LG)
        .border(1.0)
        .border_color(BORDER)
}

/// Apply setting row styling
pub fn setting_row_style(s: Style) -> Style {
    s.width_full()
        .padding_vert(SPACING_SM)
        .items_center()
        .gap(SPACING_MD)
}

/// Apply button base styling
pub fn button_style(s: Style) -> Style {
    s.background(BG_ELEVATED)
        .border_radius(BORDER_RADIUS_MD)
        .padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(TEXT_PRIMARY)
        .border(1.0)
        .border_color(BORDER)
}

/// Apply primary button styling
pub fn primary_button_style(s: Style) -> Style {
    s.background(ACCENT)
        .border_radius(BORDER_RADIUS_MD)
        .padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(BG_DEEP)
        .border(0.0)
}

/// Apply text input styling
pub fn text_input_style(s: Style) -> Style {
    s.background(BG_BASE)
        .border_radius(BORDER_RADIUS_SM)
        .padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .color(TEXT_PRIMARY)
        .border(1.0)
        .border_color(BORDER)
}

/// Apply sidebar styling
pub fn sidebar_style(s: Style) -> Style {
    s.background(BG_DEEP)
        .width(220.0)
        .height_full()
        .padding(SPACING_MD)
        .border_right(1.0)
        .border_color(BORDER)
}

/// Apply main content area styling
pub fn content_style(s: Style) -> Style {
    s.background(BG_BASE)
        .flex_grow(1.0)
        .height_full()
        .padding(SPACING_XL)
}
