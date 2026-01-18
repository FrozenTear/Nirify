//! Theme colors and styling helpers for the niri-settings UI
//!
//! Catppuccin Mocha color palette

use floem::peniko::Color;
use floem::style::Style;

// ============================================================================
// Catppuccin Mocha Palette
// ============================================================================

// Base colors
pub const CRUST: Color = Color::from_rgb8(0x11, 0x11, 0x1b);
pub const MANTLE: Color = Color::from_rgb8(0x18, 0x18, 0x25);
pub const BASE: Color = Color::from_rgb8(0x1e, 0x1e, 0x2e);

// Surface colors
pub const SURFACE0: Color = Color::from_rgb8(0x31, 0x32, 0x44);
pub const SURFACE1: Color = Color::from_rgb8(0x45, 0x47, 0x5a);
pub const SURFACE2: Color = Color::from_rgb8(0x58, 0x5b, 0x70);

// Overlay colors
pub const OVERLAY0: Color = Color::from_rgb8(0x6c, 0x70, 0x86);
pub const OVERLAY1: Color = Color::from_rgb8(0x7f, 0x84, 0x9c);
pub const OVERLAY2: Color = Color::from_rgb8(0x93, 0x99, 0xb2);

// Text colors
pub const SUBTEXT0: Color = Color::from_rgb8(0xa6, 0xad, 0xc8);
pub const SUBTEXT1: Color = Color::from_rgb8(0xba, 0xc2, 0xde);
pub const TEXT: Color = Color::from_rgb8(0xcd, 0xd6, 0xf4);

// Accent colors
pub const LAVENDER: Color = Color::from_rgb8(0xb4, 0xbe, 0xfe);
pub const BLUE: Color = Color::from_rgb8(0x89, 0xb4, 0xfa);
pub const SAPPHIRE: Color = Color::from_rgb8(0x74, 0xc7, 0xec);
pub const SKY: Color = Color::from_rgb8(0x89, 0xdc, 0xeb);
pub const TEAL: Color = Color::from_rgb8(0x94, 0xe2, 0xd5);
pub const GREEN: Color = Color::from_rgb8(0xa6, 0xe3, 0xa1);
pub const YELLOW: Color = Color::from_rgb8(0xf9, 0xe2, 0xaf);
pub const PEACH: Color = Color::from_rgb8(0xfa, 0xb3, 0x87);
pub const MAROON: Color = Color::from_rgb8(0xeb, 0xa0, 0xac);
pub const RED: Color = Color::from_rgb8(0xf3, 0x8b, 0xa8);
pub const MAUVE: Color = Color::from_rgb8(0xcb, 0xa6, 0xf7);
pub const PINK: Color = Color::from_rgb8(0xf5, 0xc2, 0xe7);
pub const FLAMINGO: Color = Color::from_rgb8(0xf2, 0xcd, 0xcd);
pub const ROSEWATER: Color = Color::from_rgb8(0xf5, 0xe0, 0xdc);

// ============================================================================
// Semantic Aliases (for easier use)
// ============================================================================

pub const BG_DEEP: Color = CRUST;
pub const BG_BASE: Color = BASE;
pub const BG_SURFACE: Color = SURFACE0;
pub const BG_ELEVATED: Color = SURFACE1;

pub const ACCENT: Color = MAUVE;
pub const ACCENT_DIM: Color = LAVENDER;

pub const TEXT_PRIMARY: Color = TEXT;
pub const TEXT_SECONDARY: Color = SUBTEXT1;
pub const TEXT_MUTED: Color = OVERLAY1;

pub const BORDER: Color = SURFACE1;

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
        .color(SUBTEXT0)
}

/// Apply nav tab styling (selected)
pub fn nav_tab_selected_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(BORDER_RADIUS_LG)
        .background(ACCENT)
        .color(CRUST)
}

/// Apply secondary nav styling (subcategory bar)
pub fn secondary_nav_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_XL)
        .padding_vert(SPACING_SM)
        .gap(SPACING_MD)
        .border_bottom(1.0)
        .border_color(SURFACE0)
}

/// Apply search bar container styling
pub fn search_bar_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_XL)
        .padding_vert(SPACING_MD)
        .items_center()
        .gap(SPACING_MD)
        .border_bottom(1.0)
        .border_color(SURFACE0)
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
        .border_color(SURFACE0)
}
