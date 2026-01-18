//! Theme colors and styling helpers for the niri-settings UI
//!
//! Crystalline Dark - A refined glass-morphic aesthetic with subtle depth
//! Built on Catppuccin Mocha with enhanced visual hierarchy

// ============================================================================
// Catppuccin Mocha Palette - RGB tuples for Freya
// ============================================================================

// Base colors - Deep background layers
pub const CRUST: (u8, u8, u8) = (0x11, 0x11, 0x1b);
pub const MANTLE: (u8, u8, u8) = (0x18, 0x18, 0x25);
pub const BASE: (u8, u8, u8) = (0x1e, 0x1e, 0x2e);

// Surface colors - Elevated layers
pub const SURFACE0: (u8, u8, u8) = (0x31, 0x32, 0x44);
pub const SURFACE1: (u8, u8, u8) = (0x45, 0x47, 0x5a);
pub const SURFACE2: (u8, u8, u8) = (0x58, 0x5b, 0x70);

// Overlay colors - Muted interactive states
pub const OVERLAY0: (u8, u8, u8) = (0x6c, 0x70, 0x86);
pub const OVERLAY1: (u8, u8, u8) = (0x7f, 0x84, 0x9c);
pub const OVERLAY2: (u8, u8, u8) = (0x93, 0x99, 0xb2);

// Text colors - Typography hierarchy
pub const SUBTEXT0: (u8, u8, u8) = (0xa6, 0xad, 0xc8);
pub const SUBTEXT1: (u8, u8, u8) = (0xba, 0xc2, 0xde);
pub const TEXT: (u8, u8, u8) = (0xcd, 0xd6, 0xf4);

// Accent colors - Vibrant highlights
pub const LAVENDER: (u8, u8, u8) = (0xb4, 0xbe, 0xfe);
pub const BLUE: (u8, u8, u8) = (0x89, 0xb4, 0xfa);
pub const SAPPHIRE: (u8, u8, u8) = (0x74, 0xc7, 0xec);
pub const SKY: (u8, u8, u8) = (0x89, 0xdc, 0xeb);
pub const TEAL: (u8, u8, u8) = (0x94, 0xe2, 0xd5);
pub const GREEN: (u8, u8, u8) = (0xa6, 0xe3, 0xa1);
pub const YELLOW: (u8, u8, u8) = (0xf9, 0xe2, 0xaf);
pub const PEACH: (u8, u8, u8) = (0xfa, 0xb3, 0x87);
pub const MAROON: (u8, u8, u8) = (0xeb, 0xa0, 0xac);
pub const RED: (u8, u8, u8) = (0xf3, 0x8b, 0xa8);
pub const MAUVE: (u8, u8, u8) = (0xcb, 0xa6, 0xf7);
pub const PINK: (u8, u8, u8) = (0xf5, 0xc2, 0xe7);
pub const FLAMINGO: (u8, u8, u8) = (0xf2, 0xcd, 0xcd);
pub const ROSEWATER: (u8, u8, u8) = (0xf5, 0xe0, 0xdc);

// ============================================================================
// Semantic Aliases - Design System Tokens
// ============================================================================

// Background layers (darkest to lightest)
pub const BG_DEEP: (u8, u8, u8) = CRUST;
pub const BG_BASE: (u8, u8, u8) = BASE;
pub const BG_SURFACE: (u8, u8, u8) = SURFACE0;
pub const BG_ELEVATED: (u8, u8, u8) = SURFACE1;
pub const BG_FLOATING: (u8, u8, u8) = SURFACE2;

// Primary accent (used for active states, focus indicators)
pub const ACCENT: (u8, u8, u8) = MAUVE;
pub const ACCENT_HOVER: (u8, u8, u8) = LAVENDER;
pub const ACCENT_MUTED: (u8, u8, u8, u8) = (0xcb, 0xa6, 0xf7, 0x40); // 25% opacity

// Secondary accent (used for secondary actions)
pub const SECONDARY: (u8, u8, u8) = SAPPHIRE;
pub const SECONDARY_MUTED: (u8, u8, u8, u8) = (0x74, 0xc7, 0xec, 0x40);

// Text hierarchy
pub const TEXT_PRIMARY: (u8, u8, u8) = TEXT;
pub const TEXT_SECONDARY: (u8, u8, u8) = SUBTEXT1;
pub const TEXT_TERTIARY: (u8, u8, u8) = SUBTEXT0;
pub const TEXT_MUTED: (u8, u8, u8) = OVERLAY1;
pub const TEXT_GHOST: (u8, u8, u8) = OVERLAY0;

// Interactive states
pub const HOVER_BG: (u8, u8, u8, u8) = (0xcd, 0xd6, 0xf4, 0x08); // 3% white
pub const ACTIVE_BG: (u8, u8, u8, u8) = (0xcd, 0xd6, 0xf4, 0x0f); // 6% white

// Borders and dividers
pub const BORDER: (u8, u8, u8) = SURFACE1;
pub const BORDER_SUBTLE: (u8, u8, u8) = SURFACE0;
pub const BORDER_ACCENT: (u8, u8, u8, u8) = (0xcb, 0xa6, 0xf7, 0x60); // 37% accent

// Status colors
pub const SUCCESS: (u8, u8, u8) = GREEN;
pub const WARNING: (u8, u8, u8) = YELLOW;
pub const ERROR: (u8, u8, u8) = RED;

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
// Helper Functions
// ============================================================================

/// Parse hex color string into RGB tuple
pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        (r, g, b)
    } else {
        (128, 128, 128)
    }
}

/// Parse hex color string into RGBA tuple
pub fn parse_hex_color_alpha(hex: &str) -> (u8, u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
        (r, g, b, a)
    } else if hex.len() >= 6 {
        let (r, g, b) = parse_hex_color(hex);
        (r, g, b, 255)
    } else {
        (128, 128, 128, 255)
    }
}
