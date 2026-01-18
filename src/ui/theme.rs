//! Theme colors and styling helpers for the niri-settings UI
//!
//! Dark theme inspired by Catppuccin Mocha with purple accent.

use floem::peniko::Color;
use floem::style::Style;

// ============================================================================
// Color Palette
// ============================================================================

/// Deep background (main app background)
pub const BG_DEEP: Color = Color::from_rgb8(0x11, 0x11, 0x14);

/// Base background (content area)
pub const BG_BASE: Color = Color::from_rgb8(0x18, 0x18, 0x1c);

/// Surface background (cards, sections)
pub const BG_SURFACE: Color = Color::from_rgb8(0x1e, 0x1e, 0x24);

/// Elevated surface (inputs, buttons)
pub const BG_ELEVATED: Color = Color::from_rgb8(0x28, 0x28, 0x30);

/// Primary accent color (selected tabs, toggles)
pub const ACCENT: Color = Color::from_rgb8(0x95, 0x80, 0xff);

/// Accent dimmed (for hover states)
pub const ACCENT_DIM: Color = Color::from_rgb8(0x7a, 0x68, 0xd9);

/// Primary text color
pub const TEXT_PRIMARY: Color = Color::from_rgb8(0xe8, 0xe8, 0xf0);

/// Secondary text color (descriptions)
pub const TEXT_SECONDARY: Color = Color::from_rgb8(0x88, 0x88, 0x98);

/// Muted text color (placeholders)
pub const TEXT_MUTED: Color = Color::from_rgb8(0x58, 0x58, 0x68);

/// Border color
pub const BORDER: Color = Color::from_rgb8(0x30, 0x30, 0x3a);

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
        .border_radius(BORDER_RADIUS_MD)
        .padding(SPACING_LG)
        .margin_bottom(SPACING_MD)
}

/// Apply setting row styling
pub fn setting_row_style(s: Style) -> Style {
    s.width_full()
        .padding_vert(SPACING_MD)
        .items_center()
        .gap(SPACING_LG)
}

/// Apply nav tab styling (unselected)
pub fn nav_tab_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(BORDER_RADIUS_LG)
        .color(TEXT_SECONDARY)
}

/// Apply nav tab styling (selected)
pub fn nav_tab_selected_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(BORDER_RADIUS_LG)
        .background(ACCENT)
        .color(BG_DEEP)
}

/// Apply secondary nav styling (subcategory bar)
pub fn secondary_nav_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_XL)
        .padding_vert(SPACING_SM)
        .gap(SPACING_MD)
        .border_bottom(1.0)
        .border_color(BORDER)
}

/// Apply search bar container styling
pub fn search_bar_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_XL)
        .padding_vert(SPACING_MD)
        .items_center()
        .gap(SPACING_MD)
        .border_bottom(1.0)
        .border_color(BORDER)
}

/// Apply main content area styling
pub fn content_style(s: Style) -> Style {
    s.background(BG_BASE)
        .flex_grow(1.0)
        .width_full()
        .padding(SPACING_XL)
}

/// Apply footer styling
pub fn footer_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_XL)
        .padding_vert(SPACING_MD)
        .items_center()
        .justify_between()
        .border_top(1.0)
        .border_color(BORDER)
}
