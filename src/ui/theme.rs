//! Theme: Obsidian Editorial
//!
//! A refined, magazine-inspired aesthetic with dramatic negative space,
//! geometric precision, and a signature teal accent against deep charcoal.

// ============================================================================
// Core Palette - Obsidian Editorial
// ============================================================================

// Deep backgrounds - rich charcoal with subtle warmth
pub const VOID: (u8, u8, u8) = (0x0a, 0x0a, 0x0c);        // Deepest black
pub const OBSIDIAN: (u8, u8, u8) = (0x12, 0x12, 0x16);    // Primary background
pub const SLATE: (u8, u8, u8) = (0x1a, 0x1a, 0x20);       // Elevated surfaces
pub const GRAPHITE: (u8, u8, u8) = (0x24, 0x24, 0x2c);    // Cards/sections
pub const CHARCOAL: (u8, u8, u8) = (0x2e, 0x2e, 0x38);    // Borders, dividers

// Text hierarchy - warm grays
pub const TEXT_BRIGHT: (u8, u8, u8) = (0xf4, 0xf4, 0xf5); // Primary text
pub const TEXT_SOFT: (u8, u8, u8) = (0xb8, 0xb8, 0xbe);   // Secondary text
pub const TEXT_DIM: (u8, u8, u8) = (0x78, 0x78, 0x82);    // Tertiary/muted
pub const TEXT_GHOST: (u8, u8, u8) = (0x4a, 0x4a, 0x54);  // Ghost/disabled

// Signature accent - Electric Teal
pub const ACCENT_VIVID: (u8, u8, u8) = (0x00, 0xd4, 0xaa);    // Primary accent
pub const ACCENT_GLOW: (u8, u8, u8) = (0x00, 0xf0, 0xc0);     // Hover/active
pub const ACCENT_MUTED: (u8, u8, u8) = (0x00, 0x8a, 0x70);    // Subtle accent
pub const ACCENT_FAINT: (u8, u8, u8, u8) = (0x00, 0xd4, 0xaa, 0x15); // Background tint

// Secondary accent - Soft lavender for contrast
pub const SECONDARY: (u8, u8, u8) = (0xa8, 0x9c, 0xfa);
pub const SECONDARY_MUTED: (u8, u8, u8, u8) = (0xa8, 0x9c, 0xfa, 0x20);

// Status colors - refined tones
pub const SUCCESS: (u8, u8, u8) = (0x4a, 0xe0, 0x8a);
pub const WARNING: (u8, u8, u8) = (0xf4, 0xc4, 0x5c);
pub const ERROR: (u8, u8, u8) = (0xf4, 0x6c, 0x7c);

// ============================================================================
// Semantic Aliases - Design Tokens
// ============================================================================

// Background layers
pub const BG_DEEP: (u8, u8, u8) = OBSIDIAN;
pub const BG_BASE: (u8, u8, u8) = SLATE;
pub const BG_SURFACE: (u8, u8, u8) = GRAPHITE;
pub const BG_ELEVATED: (u8, u8, u8) = CHARCOAL;
pub const BG_FLOATING: (u8, u8, u8) = (0x38, 0x38, 0x44);

// Primary accent
pub const ACCENT: (u8, u8, u8) = ACCENT_VIVID;
pub const ACCENT_HOVER: (u8, u8, u8) = ACCENT_GLOW;

// Text hierarchy (aliases for compatibility)
pub const TEXT_PRIMARY: (u8, u8, u8) = TEXT_BRIGHT;
pub const TEXT_SECONDARY: (u8, u8, u8) = TEXT_SOFT;
pub const TEXT_TERTIARY: (u8, u8, u8) = TEXT_DIM;
pub const TEXT_MUTED: (u8, u8, u8) = TEXT_DIM;

// Interactive states
pub const HOVER_BG: (u8, u8, u8, u8) = (0xf4, 0xf4, 0xf5, 0x08);
pub const ACTIVE_BG: (u8, u8, u8, u8) = (0xf4, 0xf4, 0xf5, 0x12);
pub const SELECTED_BG: (u8, u8, u8, u8) = (0x00, 0xd4, 0xaa, 0x20);

// Borders
pub const BORDER: (u8, u8, u8) = CHARCOAL;
pub const BORDER_SUBTLE: (u8, u8, u8) = (0x28, 0x28, 0x32);
pub const BORDER_ACCENT: (u8, u8, u8, u8) = (0x00, 0xd4, 0xaa, 0x50);

// Legacy aliases for Catppuccin compat
pub const CRUST: (u8, u8, u8) = VOID;
pub const MANTLE: (u8, u8, u8) = OBSIDIAN;
pub const BASE: (u8, u8, u8) = SLATE;
pub const SURFACE0: (u8, u8, u8) = GRAPHITE;
pub const SURFACE1: (u8, u8, u8) = CHARCOAL;
pub const SURFACE2: (u8, u8, u8) = BG_FLOATING;
pub const OVERLAY0: (u8, u8, u8) = TEXT_GHOST;
pub const OVERLAY1: (u8, u8, u8) = TEXT_DIM;
pub const OVERLAY2: (u8, u8, u8) = TEXT_SOFT;
pub const SUBTEXT0: (u8, u8, u8) = TEXT_SOFT;
pub const SUBTEXT1: (u8, u8, u8) = TEXT_SOFT;
pub const TEXT: (u8, u8, u8) = TEXT_BRIGHT;
pub const LAVENDER: (u8, u8, u8) = SECONDARY;
pub const BLUE: (u8, u8, u8) = (0x6c, 0xb4, 0xf4);
pub const SAPPHIRE: (u8, u8, u8) = ACCENT_VIVID;
pub const SKY: (u8, u8, u8) = ACCENT_GLOW;
pub const TEAL: (u8, u8, u8) = ACCENT_VIVID;
pub const GREEN: (u8, u8, u8) = SUCCESS;
pub const YELLOW: (u8, u8, u8) = WARNING;
pub const PEACH: (u8, u8, u8) = (0xf4, 0xa8, 0x7c);
pub const MAROON: (u8, u8, u8) = (0xe8, 0x8c, 0x9c);
pub const RED: (u8, u8, u8) = ERROR;
pub const MAUVE: (u8, u8, u8) = SECONDARY;
pub const PINK: (u8, u8, u8) = (0xf4, 0xb8, 0xdc);
pub const FLAMINGO: (u8, u8, u8) = (0xf4, 0xc8, 0xc8);
pub const ROSEWATER: (u8, u8, u8) = (0xf4, 0xdc, 0xd8);

// ============================================================================
// Spacing System - 4px base, generous for editorial feel
// ============================================================================

pub const SPACING_2XS: f32 = 2.0;
pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;
pub const SPACING_2XL: f32 = 32.0;
pub const SPACING_3XL: f32 = 48.0;
pub const SPACING_4XL: f32 = 64.0;

// ============================================================================
// Border Radius - Subtle, refined
// ============================================================================

pub const RADIUS_XS: f32 = 2.0;
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;
pub const RADIUS_2XL: f32 = 20.0;
pub const RADIUS_FULL: f32 = 9999.0;

// ============================================================================
// Typography Scale - Clean hierarchy
// ============================================================================

pub const FONT_SIZE_2XS: f32 = 9.0;
pub const FONT_SIZE_XS: f32 = 10.0;
pub const FONT_SIZE_SM: f32 = 12.0;
pub const FONT_SIZE_BASE: f32 = 14.0;
pub const FONT_SIZE_LG: f32 = 16.0;
pub const FONT_SIZE_XL: f32 = 20.0;
pub const FONT_SIZE_2XL: f32 = 24.0;
pub const FONT_SIZE_3XL: f32 = 32.0;
pub const FONT_SIZE_4XL: f32 = 40.0;

// Letter spacing for display text
pub const LETTER_SPACING_TIGHT: f32 = -0.5;
pub const LETTER_SPACING_NORMAL: f32 = 0.0;
pub const LETTER_SPACING_WIDE: f32 = 1.0;
pub const LETTER_SPACING_ULTRA: f32 = 3.0;

// ============================================================================
// Component Heights
// ============================================================================

pub const NAV_HEIGHT: f32 = 72.0;
pub const SUBNAV_HEIGHT: f32 = 56.0;
pub const FOOTER_HEIGHT: f32 = 48.0;
pub const ROW_HEIGHT: f32 = 56.0;
pub const ROW_HEIGHT_COMPACT: f32 = 44.0;

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
