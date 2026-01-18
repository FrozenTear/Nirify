//! Theme colors and styling helpers for the niri-settings UI
//!
//! Crystalline Dark - A refined glass-morphic aesthetic with subtle depth
//! Built on Catppuccin Mocha with enhanced visual hierarchy

use floem::peniko::Color;
use floem::style::Style;

// ============================================================================
// Catppuccin Mocha Palette
// ============================================================================

// Base colors - Deep background layers
pub const CRUST: Color = Color::from_rgb8(0x11, 0x11, 0x1b);
pub const MANTLE: Color = Color::from_rgb8(0x18, 0x18, 0x25);
pub const BASE: Color = Color::from_rgb8(0x1e, 0x1e, 0x2e);

// Surface colors - Elevated layers
pub const SURFACE0: Color = Color::from_rgb8(0x31, 0x32, 0x44);
pub const SURFACE1: Color = Color::from_rgb8(0x45, 0x47, 0x5a);
pub const SURFACE2: Color = Color::from_rgb8(0x58, 0x5b, 0x70);

// Overlay colors - Muted interactive states
pub const OVERLAY0: Color = Color::from_rgb8(0x6c, 0x70, 0x86);
pub const OVERLAY1: Color = Color::from_rgb8(0x7f, 0x84, 0x9c);
pub const OVERLAY2: Color = Color::from_rgb8(0x93, 0x99, 0xb2);

// Text colors - Typography hierarchy
pub const SUBTEXT0: Color = Color::from_rgb8(0xa6, 0xad, 0xc8);
pub const SUBTEXT1: Color = Color::from_rgb8(0xba, 0xc2, 0xde);
pub const TEXT: Color = Color::from_rgb8(0xcd, 0xd6, 0xf4);

// Accent colors - Vibrant highlights
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
// Semantic Aliases - Design System Tokens
// ============================================================================

// Background layers (darkest to lightest)
pub const BG_DEEP: Color = CRUST;
pub const BG_BASE: Color = BASE;
pub const BG_SURFACE: Color = SURFACE0;
pub const BG_ELEVATED: Color = SURFACE1;
pub const BG_FLOATING: Color = SURFACE2;

// Primary accent (used for active states, focus indicators)
pub const ACCENT: Color = MAUVE;
pub const ACCENT_HOVER: Color = LAVENDER;
pub const ACCENT_MUTED: Color = Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x40); // 25% opacity

// Secondary accent (used for secondary actions)
pub const SECONDARY: Color = SAPPHIRE;
pub const SECONDARY_MUTED: Color = Color::from_rgba8(0x74, 0xc7, 0xec, 0x40);

// Text hierarchy
pub const TEXT_PRIMARY: Color = TEXT;
pub const TEXT_SECONDARY: Color = SUBTEXT1;
pub const TEXT_TERTIARY: Color = SUBTEXT0;
pub const TEXT_MUTED: Color = OVERLAY1;
pub const TEXT_GHOST: Color = OVERLAY0;

// Interactive states
pub const HOVER_BG: Color = Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x08); // 3% white
pub const ACTIVE_BG: Color = Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x0f); // 6% white
pub const FOCUS_RING: Color = ACCENT_MUTED;

// Borders and dividers
pub const BORDER: Color = SURFACE1;
pub const BORDER_SUBTLE: Color = SURFACE0;
pub const BORDER_ACCENT: Color = Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x60); // 37% accent

// Status colors
pub const SUCCESS: Color = GREEN;
pub const WARNING: Color = YELLOW;
pub const ERROR: Color = RED;

// ============================================================================
// Spacing System - 4px base unit
// ============================================================================

pub const SPACING_2XS: f32 = 2.0;
pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;
pub const SPACING_2XL: f32 = 32.0;
pub const SPACING_3XL: f32 = 48.0;

// ============================================================================
// Border Radius - Consistent roundness
// ============================================================================

pub const RADIUS_XS: f32 = 2.0;
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;
pub const RADIUS_FULL: f32 = 9999.0;

// Legacy aliases for compatibility
pub const BORDER_RADIUS_SM: f32 = RADIUS_SM;
pub const BORDER_RADIUS_MD: f32 = RADIUS_MD;
pub const BORDER_RADIUS_LG: f32 = RADIUS_LG;
pub const SPACING_XXL: f32 = SPACING_2XL;

// ============================================================================
// Typography Scale
// ============================================================================

pub const FONT_SIZE_XS: f32 = 10.0;
pub const FONT_SIZE_SM: f32 = 12.0;
pub const FONT_SIZE_BASE: f32 = 14.0;
pub const FONT_SIZE_LG: f32 = 16.0;
pub const FONT_SIZE_XL: f32 = 20.0;
pub const FONT_SIZE_2XL: f32 = 24.0;

// ============================================================================
// Component Style Helpers
// ============================================================================

/// Header container - the app title bar area
pub fn header_style(s: Style) -> Style {
    s.width_full()
        .items_center()
        .background(MANTLE)
        .padding_vert(SPACING_LG)
        .padding_horiz(SPACING_2XL)
}

/// Primary navigation tab - unselected state
pub fn nav_tab_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(TEXT_TERTIARY)
        .font_size(FONT_SIZE_BASE)
}

/// Primary navigation tab - selected state with underline
pub fn nav_tab_selected_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(TEXT_PRIMARY)
        .font_bold()
        .font_size(FONT_SIZE_BASE)
        .border_bottom(2.0)
        .border_color(ACCENT)
}

/// Secondary navigation tab - pill style unselected
pub fn secondary_tab_style(s: Style) -> Style {
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_XS)
        .border_radius(RADIUS_FULL)
        .color(TEXT_TERTIARY)
        .font_size(FONT_SIZE_SM)
}

/// Secondary navigation tab - pill style selected
pub fn secondary_tab_selected_style(s: Style) -> Style {
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_XS)
        .border_radius(RADIUS_FULL)
        .background(ACCENT)
        .color(CRUST)
        .font_size(FONT_SIZE_SM)
        .font_bold()
}

/// Secondary nav container
pub fn secondary_nav_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_vert(SPACING_MD)
        .gap(SPACING_SM)
        .background(MANTLE)
        .border_bottom(1.0)
        .border_color(BORDER_SUBTLE)
}

/// Section card - elevated glass-like panel
pub fn section_style(s: Style) -> Style {
    s.width_full()
        .background(BG_SURFACE)
        .border_radius(RADIUS_LG)
        .border(1.0)
        .border_color(BORDER_SUBTLE)
        .padding(SPACING_XL)
        .margin_bottom(SPACING_LG)
}

/// Section header text styling
pub fn section_header_style(s: Style) -> Style {
    s.font_size(FONT_SIZE_XS)
        .font_bold()
        .color(ACCENT)
        .margin_bottom(SPACING_LG)
}

/// Setting row - horizontal layout with label and control
pub fn setting_row_style(s: Style) -> Style {
    s.width_full()
        .padding_vert(SPACING_MD)
        .items_center()
        .gap(SPACING_LG)
}

/// Setting row with subtle divider between rows
pub fn setting_row_divided_style(s: Style) -> Style {
    setting_row_style(s)
        .border_bottom(1.0)
        .border_color(BORDER_SUBTLE)
        .padding_bottom(SPACING_LG)
        .margin_bottom(SPACING_MD)
}

/// Search bar container
pub fn search_bar_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_vert(SPACING_LG)
        .items_center()
        .gap(SPACING_MD)
        .background(BG_BASE)
}

/// Search input field
pub fn search_input_style(s: Style) -> Style {
    s.flex_grow(1.0)
        .background(BG_SURFACE)
        .border_radius(RADIUS_MD)
        .border(1.0)
        .border_color(BORDER_SUBTLE)
        .padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .color(TEXT_PRIMARY)
        .font_size(FONT_SIZE_BASE)
}

/// Main content area
pub fn content_style(s: Style) -> Style {
    s.background(BG_BASE)
        .flex_grow(1.0)
        .width_full()
        .padding(SPACING_2XL)
}

/// Footer container
pub fn footer_style(s: Style) -> Style {
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_vert(SPACING_LG)
        .items_center()
        .justify_between()
        .background(MANTLE)
        .border_top(1.0)
        .border_color(BORDER_SUBTLE)
}

/// Primary button - accent colored
pub fn button_primary_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .background(ACCENT)
        .color(CRUST)
        .font_size(FONT_SIZE_SM)
        .font_bold()
}

/// Secondary button - subtle surface
pub fn button_secondary_style(s: Style) -> Style {
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .background(BG_ELEVATED)
        .border(1.0)
        .border_color(BORDER)
        .color(TEXT_SECONDARY)
        .font_size(FONT_SIZE_SM)
}

/// Ghost button - transparent with text
pub fn button_ghost_style(s: Style) -> Style {
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_XS)
        .border_radius(RADIUS_SM)
        .color(TEXT_TERTIARY)
        .font_size(FONT_SIZE_SM)
}

/// Toggle track (off state)
pub fn toggle_track_style(s: Style) -> Style {
    s.width(44.0)
        .height(24.0)
        .border_radius(RADIUS_FULL)
        .background(SURFACE1)
        .border(1.0)
        .border_color(BORDER)
        .items_center()
        .padding_horiz(SPACING_2XS)
}

/// Toggle track (on state)
pub fn toggle_track_on_style(s: Style) -> Style {
    toggle_track_style(s)
        .background(ACCENT)
        .border_color(ACCENT)
}

/// Toggle knob
pub fn toggle_knob_style(s: Style) -> Style {
    s.width(18.0)
        .height(18.0)
        .border_radius(RADIUS_FULL)
        .background(TEXT_PRIMARY)
}

/// Slider track background
pub fn slider_track_style(s: Style) -> Style {
    s.height(4.0)
        .flex_grow(1.0)
        .border_radius(RADIUS_FULL)
        .background(SURFACE1)
}

/// Slider track fill (active portion)
pub fn slider_fill_style(s: Style) -> Style {
    s.height(4.0).border_radius(RADIUS_FULL).background(ACCENT)
}

/// Slider thumb
pub fn slider_thumb_style(s: Style) -> Style {
    s.width(16.0)
        .height(16.0)
        .border_radius(RADIUS_FULL)
        .background(TEXT_PRIMARY)
        .border(2.0)
        .border_color(ACCENT)
}

/// Color swatch display
pub fn color_swatch_style(s: Style) -> Style {
    s.width(28.0)
        .height(28.0)
        .border_radius(RADIUS_SM)
        .border(2.0)
        .border_color(BORDER)
}

/// Color input field container
pub fn color_input_container_style(s: Style) -> Style {
    s.background(BG_ELEVATED)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(BORDER_SUBTLE)
        .padding_left(SPACING_SM)
        .items_center()
}

/// Text input field
pub fn text_input_style(s: Style) -> Style {
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .background(BG_SURFACE)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(BORDER_SUBTLE)
        .color(TEXT_PRIMARY)
        .font_size(FONT_SIZE_SM)
}

/// Dropdown/select field
pub fn dropdown_style(s: Style) -> Style {
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .background(BG_ELEVATED)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(BORDER)
        .color(TEXT_PRIMARY)
        .font_size(FONT_SIZE_SM)
        .min_width(160.0)
}

/// Value display (like "4px")
pub fn value_badge_style(s: Style) -> Style {
    s.padding_horiz(SPACING_SM)
        .padding_vert(SPACING_2XS)
        .background(BG_ELEVATED)
        .border_radius(RADIUS_SM)
        .color(TEXT_SECONDARY)
        .font_size(FONT_SIZE_SM)
        .min_width(48.0)
}

/// Small icon button
pub fn icon_button_style(s: Style) -> Style {
    s.width(28.0)
        .height(28.0)
        .border_radius(RADIUS_SM)
        .items_center()
        .justify_center()
        .color(TEXT_MUTED)
}

/// Parse hex color string into Floem Color
pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        Color::from_rgb8(r, g, b)
    } else {
        Color::from_rgb8(128, 128, 128)
    }
}
