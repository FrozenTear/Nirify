//! Theme colors and styling helpers for the niri-settings UI
//!
//! Dynamic theming system with multiple color scheme support.
//! Built on Catppuccin with Nord alternative.

use floem::peniko::Color;
use floem::prelude::SignalGet;
use floem::reactive::{RwSignal, SignalUpdate};
use floem::style::Style;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// ============================================================================
// Theme Structure
// ============================================================================

/// Complete theme definition with all semantic colors
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // Background layers (darkest to lightest)
    pub bg_deep: Color,
    pub bg_base: Color,
    pub bg_surface: Color,
    pub bg_elevated: Color,
    pub bg_floating: Color,

    // Text hierarchy
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_muted: Color,
    pub text_ghost: Color,

    // Accent colors
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_muted: Color,
    pub secondary: Color,
    pub secondary_muted: Color,

    // Borders
    pub border: Color,
    pub border_subtle: Color,
    pub border_accent: Color,

    // Interactive states
    pub hover_bg: Color,
    pub active_bg: Color,
    pub focus_ring: Color,

    // Status
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Special: contrast color for text on accent background
    pub on_accent: Color,
}

// ============================================================================
// Theme Presets
// ============================================================================

/// Available theme presets
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreset {
    #[default]
    CatppuccinMocha,
    CatppuccinLatte,
    Nord,
    TokyoNight,
}

impl ThemePreset {
    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::Nord => "Nord",
            Self::TokyoNight => "Tokyo Night",
        }
    }

    /// Short description
    pub fn description(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "Warm dark theme with purple accents",
            Self::CatppuccinLatte => "Light theme with purple accents",
            Self::Nord => "Cool dark theme with blue accents",
            Self::TokyoNight => "Dark theme with cyan accents",
        }
    }

    /// Is this a dark theme?
    pub fn is_dark(&self) -> bool {
        match self {
            Self::CatppuccinMocha => true,
            Self::CatppuccinLatte => false,
            Self::Nord => true,
            Self::TokyoNight => true,
        }
    }

    /// Convert preset to actual Theme
    pub fn to_theme(&self) -> Theme {
        match self {
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
            Self::CatppuccinLatte => Theme::catppuccin_latte(),
            Self::Nord => Theme::nord(),
            Self::TokyoNight => Theme::tokyo_night(),
        }
    }

    /// All available presets
    pub fn all() -> &'static [ThemePreset] {
        &[
            Self::CatppuccinMocha,
            Self::CatppuccinLatte,
            Self::Nord,
            Self::TokyoNight,
        ]
    }
}

impl Theme {
    /// Catppuccin Mocha - Warm dark theme (default)
    pub fn catppuccin_mocha() -> Self {
        Self {
            // Background layers
            bg_deep: Color::from_rgb8(0x11, 0x11, 0x1b),    // Crust
            bg_base: Color::from_rgb8(0x1e, 0x1e, 0x2e),    // Base
            bg_surface: Color::from_rgb8(0x31, 0x32, 0x44), // Surface0
            bg_elevated: Color::from_rgb8(0x45, 0x47, 0x5a), // Surface1
            bg_floating: Color::from_rgb8(0x58, 0x5b, 0x70), // Surface2

            // Text
            text_primary: Color::from_rgb8(0xcd, 0xd6, 0xf4),   // Text
            text_secondary: Color::from_rgb8(0xba, 0xc2, 0xde), // Subtext1
            text_tertiary: Color::from_rgb8(0xa6, 0xad, 0xc8),  // Subtext0
            text_muted: Color::from_rgb8(0x7f, 0x84, 0x9c),     // Overlay1
            text_ghost: Color::from_rgb8(0x6c, 0x70, 0x86),     // Overlay0

            // Accents
            accent: Color::from_rgb8(0xcb, 0xa6, 0xf7),             // Mauve
            accent_hover: Color::from_rgb8(0xb4, 0xbe, 0xfe),       // Lavender
            accent_muted: Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x40),
            secondary: Color::from_rgb8(0x74, 0xc7, 0xec),          // Sapphire
            secondary_muted: Color::from_rgba8(0x74, 0xc7, 0xec, 0x40),

            // Borders
            border: Color::from_rgb8(0x45, 0x47, 0x5a),        // Surface1
            border_subtle: Color::from_rgb8(0x31, 0x32, 0x44), // Surface0
            border_accent: Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x60),

            // Interactive
            hover_bg: Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x08),
            active_bg: Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x0f),
            focus_ring: Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x40),

            // Status
            success: Color::from_rgb8(0xa6, 0xe3, 0xa1), // Green
            warning: Color::from_rgb8(0xf9, 0xe2, 0xaf), // Yellow
            error: Color::from_rgb8(0xf3, 0x8b, 0xa8),   // Red

            // On accent
            on_accent: Color::from_rgb8(0x11, 0x11, 0x1b), // Crust
        }
    }

    /// Catppuccin Latte - Light theme
    pub fn catppuccin_latte() -> Self {
        Self {
            // Background layers (reversed for light theme)
            bg_deep: Color::from_rgb8(0xdc, 0xe0, 0xe8),    // Crust
            bg_base: Color::from_rgb8(0xef, 0xf1, 0xf5),    // Base
            bg_surface: Color::from_rgb8(0xe6, 0xe9, 0xef), // Surface0
            bg_elevated: Color::from_rgb8(0xcc, 0xd0, 0xda), // Surface1
            bg_floating: Color::from_rgb8(0xbc, 0xc0, 0xcc), // Surface2

            // Text (dark on light)
            text_primary: Color::from_rgb8(0x4c, 0x4f, 0x69),   // Text
            text_secondary: Color::from_rgb8(0x5c, 0x5f, 0x77), // Subtext1
            text_tertiary: Color::from_rgb8(0x6c, 0x6f, 0x85),  // Subtext0
            text_muted: Color::from_rgb8(0x8c, 0x8f, 0xa1),     // Overlay1
            text_ghost: Color::from_rgb8(0x9c, 0xa0, 0xb0),     // Overlay0

            // Accents
            accent: Color::from_rgb8(0x88, 0x39, 0xef),             // Mauve
            accent_hover: Color::from_rgb8(0x72, 0x87, 0xfd),       // Lavender
            accent_muted: Color::from_rgba8(0x88, 0x39, 0xef, 0x30),
            secondary: Color::from_rgb8(0x20, 0x9f, 0xb5),          // Sapphire
            secondary_muted: Color::from_rgba8(0x20, 0x9f, 0xb5, 0x30),

            // Borders
            border: Color::from_rgb8(0xbc, 0xc0, 0xcc),        // Surface2
            border_subtle: Color::from_rgb8(0xcc, 0xd0, 0xda), // Surface1
            border_accent: Color::from_rgba8(0x88, 0x39, 0xef, 0x50),

            // Interactive
            hover_bg: Color::from_rgba8(0x4c, 0x4f, 0x69, 0x08),
            active_bg: Color::from_rgba8(0x4c, 0x4f, 0x69, 0x10),
            focus_ring: Color::from_rgba8(0x88, 0x39, 0xef, 0x30),

            // Status
            success: Color::from_rgb8(0x40, 0xa0, 0x2b), // Green
            warning: Color::from_rgb8(0xdf, 0x8e, 0x1d), // Yellow
            error: Color::from_rgb8(0xd2, 0x00, 0x65),   // Red

            // On accent (light text on dark accent)
            on_accent: Color::from_rgb8(0xef, 0xf1, 0xf5), // Base
        }
    }

    /// Nord - Cool dark theme with blue accents
    pub fn nord() -> Self {
        Self {
            // Background layers (Polar Night)
            bg_deep: Color::from_rgb8(0x2e, 0x34, 0x40),    // Nord0
            bg_base: Color::from_rgb8(0x3b, 0x42, 0x52),    // Nord1
            bg_surface: Color::from_rgb8(0x43, 0x4c, 0x5e), // Nord2
            bg_elevated: Color::from_rgb8(0x4c, 0x56, 0x6a), // Nord3
            bg_floating: Color::from_rgb8(0x54, 0x60, 0x74), // Nord3 lighter

            // Text (Snow Storm)
            text_primary: Color::from_rgb8(0xec, 0xef, 0xf4),   // Nord6
            text_secondary: Color::from_rgb8(0xe5, 0xe9, 0xf0), // Nord5
            text_tertiary: Color::from_rgb8(0xd8, 0xde, 0xe9),  // Nord4
            text_muted: Color::from_rgb8(0xa0, 0xa8, 0xb8),
            text_ghost: Color::from_rgb8(0x70, 0x78, 0x88),

            // Accents (Frost)
            accent: Color::from_rgb8(0x88, 0xc0, 0xd0),             // Nord8
            accent_hover: Color::from_rgb8(0x8f, 0xbc, 0xbb),       // Nord7
            accent_muted: Color::from_rgba8(0x88, 0xc0, 0xd0, 0x40),
            secondary: Color::from_rgb8(0x81, 0xa1, 0xc1),          // Nord9
            secondary_muted: Color::from_rgba8(0x81, 0xa1, 0xc1, 0x40),

            // Borders
            border: Color::from_rgb8(0x4c, 0x56, 0x6a),        // Nord3
            border_subtle: Color::from_rgb8(0x43, 0x4c, 0x5e), // Nord2
            border_accent: Color::from_rgba8(0x88, 0xc0, 0xd0, 0x60),

            // Interactive
            hover_bg: Color::from_rgba8(0xec, 0xef, 0xf4, 0x08),
            active_bg: Color::from_rgba8(0xec, 0xef, 0xf4, 0x0f),
            focus_ring: Color::from_rgba8(0x88, 0xc0, 0xd0, 0x40),

            // Status (Aurora)
            success: Color::from_rgb8(0xa3, 0xbe, 0x8c), // Nord14
            warning: Color::from_rgb8(0xeb, 0xcb, 0x8b), // Nord13
            error: Color::from_rgb8(0xbf, 0x61, 0x6a),   // Nord11

            // On accent
            on_accent: Color::from_rgb8(0x2e, 0x34, 0x40), // Nord0
        }
    }

    /// Tokyo Night - Dark theme with cyan accents
    pub fn tokyo_night() -> Self {
        Self {
            // Background layers
            bg_deep: Color::from_rgb8(0x16, 0x16, 0x1e),
            bg_base: Color::from_rgb8(0x1a, 0x1b, 0x26),
            bg_surface: Color::from_rgb8(0x24, 0x28, 0x3b),
            bg_elevated: Color::from_rgb8(0x2f, 0x33, 0x49),
            bg_floating: Color::from_rgb8(0x3b, 0x40, 0x5a),

            // Text
            text_primary: Color::from_rgb8(0xc0, 0xca, 0xf5),
            text_secondary: Color::from_rgb8(0xa9, 0xb1, 0xd6),
            text_tertiary: Color::from_rgb8(0x9a, 0xa5, 0xce),
            text_muted: Color::from_rgb8(0x56, 0x5f, 0x89),
            text_ghost: Color::from_rgb8(0x41, 0x4d, 0x68),

            // Accents
            accent: Color::from_rgb8(0x7d, 0xcf, 0xff),             // Cyan
            accent_hover: Color::from_rgb8(0xbb, 0x9a, 0xf7),       // Purple
            accent_muted: Color::from_rgba8(0x7d, 0xcf, 0xff, 0x40),
            secondary: Color::from_rgb8(0x7a, 0xa2, 0xf7),          // Blue
            secondary_muted: Color::from_rgba8(0x7a, 0xa2, 0xf7, 0x40),

            // Borders
            border: Color::from_rgb8(0x2f, 0x33, 0x49),
            border_subtle: Color::from_rgb8(0x24, 0x28, 0x3b),
            border_accent: Color::from_rgba8(0x7d, 0xcf, 0xff, 0x60),

            // Interactive
            hover_bg: Color::from_rgba8(0xc0, 0xca, 0xf5, 0x08),
            active_bg: Color::from_rgba8(0xc0, 0xca, 0xf5, 0x0f),
            focus_ring: Color::from_rgba8(0x7d, 0xcf, 0xff, 0x40),

            // Status
            success: Color::from_rgb8(0x9e, 0xce, 0x6a), // Green
            warning: Color::from_rgb8(0xe0, 0xaf, 0x68), // Orange
            error: Color::from_rgb8(0xf7, 0x76, 0x8e),   // Red

            // On accent
            on_accent: Color::from_rgb8(0x16, 0x16, 0x1e),
        }
    }
}

// ============================================================================
// Global Theme State
// ============================================================================

thread_local! {
    /// Global reactive theme signal - changes propagate to all components
    static ACTIVE_THEME: RefCell<Option<RwSignal<Theme>>> = const { RefCell::new(None) };
    /// Global preset signal for persistence
    static ACTIVE_PRESET: RefCell<Option<RwSignal<ThemePreset>>> = const { RefCell::new(None) };
}

/// Initialize the theme system. Must be called once at app startup within the reactive scope.
pub fn init_theme_system() {
    ACTIVE_THEME.with(|cell| {
        if cell.borrow().is_none() {
            *cell.borrow_mut() = Some(RwSignal::new(Theme::catppuccin_mocha()));
        }
    });
    ACTIVE_PRESET.with(|cell| {
        if cell.borrow().is_none() {
            *cell.borrow_mut() = Some(RwSignal::new(ThemePreset::CatppuccinMocha));
        }
    });
}

/// Get the theme signal (for use in reactive contexts)
pub fn theme_signal() -> RwSignal<Theme> {
    ACTIVE_THEME.with(|cell| {
        cell.borrow()
            .expect("Theme system not initialized. Call init_theme_system() at app startup.")
    })
}

/// Get the preset signal (for use in reactive contexts)
pub fn preset_signal() -> RwSignal<ThemePreset> {
    ACTIVE_PRESET.with(|cell| {
        cell.borrow()
            .expect("Theme system not initialized. Call init_theme_system() at app startup.")
    })
}

/// Switch to a different theme
pub fn set_theme(preset: ThemePreset) {
    ACTIVE_PRESET.with(|cell| {
        if let Some(signal) = cell.borrow().as_ref() {
            signal.set(preset);
        }
    });
    ACTIVE_THEME.with(|cell| {
        if let Some(signal) = cell.borrow().as_ref() {
            signal.set(preset.to_theme());
        }
    });
}

/// Get current theme preset
pub fn current_preset() -> ThemePreset {
    ACTIVE_PRESET.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|s| s.get())
            .unwrap_or_default()
    })
}

// ============================================================================
// Dynamic Color Accessors
// ============================================================================

/// Get current theme (for use in style closures)
#[inline]
pub fn theme() -> Theme {
    ACTIVE_THEME.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|s| s.get())
            .unwrap_or_else(Theme::catppuccin_mocha)
    })
}

// Background colors
pub fn bg_deep() -> Color { theme().bg_deep }
pub fn bg_base() -> Color { theme().bg_base }
pub fn bg_surface() -> Color { theme().bg_surface }
pub fn bg_elevated() -> Color { theme().bg_elevated }
pub fn bg_floating() -> Color { theme().bg_floating }

// Text colors
pub fn text_primary() -> Color { theme().text_primary }
pub fn text_secondary() -> Color { theme().text_secondary }
pub fn text_tertiary() -> Color { theme().text_tertiary }
pub fn text_muted() -> Color { theme().text_muted }
pub fn text_ghost() -> Color { theme().text_ghost }

// Accent colors
pub fn accent() -> Color { theme().accent }
pub fn accent_hover() -> Color { theme().accent_hover }
pub fn accent_muted() -> Color { theme().accent_muted }
pub fn secondary() -> Color { theme().secondary }
pub fn secondary_muted() -> Color { theme().secondary_muted }

// Border colors
pub fn border() -> Color { theme().border }
pub fn border_subtle() -> Color { theme().border_subtle }
pub fn border_accent() -> Color { theme().border_accent }

// Interactive
pub fn hover_bg() -> Color { theme().hover_bg }
pub fn active_bg() -> Color { theme().active_bg }
pub fn focus_ring() -> Color { theme().focus_ring }

// Status colors
pub fn success() -> Color { theme().success }
pub fn warning() -> Color { theme().warning }
pub fn error() -> Color { theme().error }

// Special
pub fn on_accent() -> Color { theme().on_accent }

// ============================================================================
// Legacy Static Constants (for backward compatibility during migration)
// Keep these for now - they point to Catppuccin Mocha values
// ============================================================================

// Base colors - Deep background layers
pub const CRUST: Color = Color::from_rgb8(0x11, 0x11, 0x1b);
pub const MANTLE: Color = Color::from_rgb8(0x18, 0x18, 0x25);
pub const BASE: Color = Color::from_rgb8(0x1e, 0x1e, 0x2e);

// Surface colors - Elevated layers
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

// Accent colors (Catppuccin)
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

// Legacy semantic aliases (static - for components not yet migrated)
pub const BG_DEEP: Color = CRUST;
pub const BG_BASE: Color = BASE;
pub const BG_SURFACE: Color = SURFACE0;
pub const BG_ELEVATED: Color = SURFACE1;
pub const BG_FLOATING: Color = SURFACE2;
pub const ACCENT: Color = MAUVE;
pub const ACCENT_HOVER: Color = LAVENDER;
pub const ACCENT_MUTED: Color = Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x40);
pub const SECONDARY: Color = SAPPHIRE;
pub const SECONDARY_MUTED: Color = Color::from_rgba8(0x74, 0xc7, 0xec, 0x40);
pub const TEXT_PRIMARY: Color = TEXT;
pub const TEXT_SECONDARY: Color = SUBTEXT1;
pub const TEXT_TERTIARY: Color = SUBTEXT0;
pub const TEXT_MUTED: Color = OVERLAY1;
pub const TEXT_GHOST: Color = OVERLAY0;
pub const HOVER_BG: Color = Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x08);
pub const ACTIVE_BG: Color = Color::from_rgba8(0xcd, 0xd6, 0xf4, 0x0f);
pub const FOCUS_RING: Color = ACCENT_MUTED;
pub const BORDER: Color = SURFACE1;
pub const BORDER_SUBTLE: Color = SURFACE0;
pub const BORDER_ACCENT: Color = Color::from_rgba8(0xcb, 0xa6, 0xf7, 0x60);
pub const SUCCESS: Color = GREEN;
pub const WARNING: Color = YELLOW;
pub const ERROR: Color = RED;

// ============================================================================
// Spacing System - 4px base unit with generous whitespace
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
// Component Style Helpers (Dynamic Theme-Aware)
// ============================================================================

/// Header container - clean, minimal title area
pub fn header_style(s: Style) -> Style {
    let t = theme();
    s.width_full()
        .items_center()
        .background(t.bg_base)
        .padding_vert(SPACING_XL)
        .padding_horiz(SPACING_2XL)
}

/// Primary navigation tab - unselected state
pub fn nav_tab_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(t.text_tertiary)
        .font_size(FONT_SIZE_BASE)
}

/// Primary navigation tab - selected state with underline
pub fn nav_tab_selected_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .color(t.text_primary)
        .font_bold()
        .font_size(FONT_SIZE_BASE)
        .border_bottom(2.0)
        .border_color(t.accent)
}

/// Secondary navigation tab - pill style unselected (subtle)
pub fn secondary_tab_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .color(t.text_tertiary)
        .font_size(FONT_SIZE_SM)
}

/// Secondary navigation tab - pill style selected (understated accent)
pub fn secondary_tab_selected_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .background(t.accent_muted)
        .color(t.text_primary)
        .font_size(FONT_SIZE_SM)
}

/// Secondary nav container - cleaner with less chrome
pub fn secondary_nav_style(s: Style) -> Style {
    let t = theme();
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_vert(SPACING_LG)
        .gap(SPACING_SM)
        .background(t.bg_base)
}

/// Section card - subtle grouping with minimal borders
pub fn section_style(s: Style) -> Style {
    let t = theme();
    s.width_full()
        .background(t.bg_surface)
        .border_radius(RADIUS_LG)
        .border(1.0)
        .border_color(t.border_subtle)
        .padding_vert(SPACING_LG)
        .padding_horiz(SPACING_XL)
        .margin_bottom(SPACING_XL)
}

/// Section header text styling - subtle, lowercase, refined
pub fn section_header_style(s: Style) -> Style {
    let t = theme();
    s.font_size(FONT_SIZE_SM)
        .color(t.text_muted)
        .margin_bottom(SPACING_LG)
        .padding_bottom(SPACING_SM)
        .border_bottom(1.0)
        .border_color(t.border_subtle)
}

/// Setting row - horizontal layout with more breathing room
pub fn setting_row_style(s: Style) -> Style {
    s.width_full()
        .padding_vert(SPACING_LG)
        .items_center()
        .gap(SPACING_XL)
}

/// Setting row with subtle divider between rows
pub fn setting_row_divided_style(s: Style) -> Style {
    let t = theme();
    setting_row_style(s)
        .border_bottom(1.0)
        .border_color(t.border_subtle)
        .padding_bottom(SPACING_XL)
        .margin_bottom(SPACING_MD)
}

/// Search bar container - integrated into content area
pub fn search_bar_style(s: Style) -> Style {
    let t = theme();
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_top(SPACING_LG)
        .padding_bottom(SPACING_XL)
        .items_center()
        .gap(SPACING_MD)
        .background(t.bg_base)
        .border_bottom(1.0)
        .border_color(t.border_subtle)
}

/// Search input field
pub fn search_input_style(s: Style) -> Style {
    let t = theme();
    s.flex_grow(1.0)
        .background(t.bg_surface)
        .border_radius(RADIUS_MD)
        .border(1.0)
        .border_color(t.border_subtle)
        .padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .color(t.text_primary)
        .font_size(FONT_SIZE_BASE)
}

/// Main content area - generous padding for breathing room
pub fn content_style(s: Style) -> Style {
    let t = theme();
    s.background(t.bg_deep)
        .flex_grow(1.0)
        .width_full()
        .padding_horiz(SPACING_3XL)
        .padding_vert(SPACING_2XL)
}

/// Footer container - subtle, unobtrusive
pub fn footer_style(s: Style) -> Style {
    let t = theme();
    s.width_full()
        .padding_horiz(SPACING_2XL)
        .padding_vert(SPACING_MD)
        .items_center()
        .justify_between()
        .background(t.bg_base)
}

/// Primary button - accent colored
pub fn button_primary_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .background(t.accent)
        .color(t.on_accent)
        .font_size(FONT_SIZE_SM)
        .font_bold()
}

/// Secondary button - subtle surface
pub fn button_secondary_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_LG)
        .padding_vert(SPACING_SM)
        .border_radius(RADIUS_MD)
        .background(t.bg_elevated)
        .border(1.0)
        .border_color(t.border)
        .color(t.text_secondary)
        .font_size(FONT_SIZE_SM)
}

/// Ghost button - transparent with text
pub fn button_ghost_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_XS)
        .border_radius(RADIUS_SM)
        .color(t.text_tertiary)
        .font_size(FONT_SIZE_SM)
}

/// Toggle track (off state)
pub fn toggle_track_style(s: Style) -> Style {
    let t = theme();
    s.width(44.0)
        .height(24.0)
        .border_radius(RADIUS_FULL)
        .background(t.bg_elevated)
        .border(1.0)
        .border_color(t.border)
        .items_center()
        .padding_horiz(SPACING_2XS)
}

/// Toggle track (on state)
pub fn toggle_track_on_style(s: Style) -> Style {
    let t = theme();
    toggle_track_style(s)
        .background(t.accent)
        .border_color(t.accent)
}

/// Toggle knob
pub fn toggle_knob_style(s: Style) -> Style {
    let t = theme();
    s.width(18.0)
        .height(18.0)
        .border_radius(RADIUS_FULL)
        .background(t.text_primary)
}

/// Slider track background
pub fn slider_track_style(s: Style) -> Style {
    let t = theme();
    s.height(4.0)
        .flex_grow(1.0)
        .border_radius(RADIUS_FULL)
        .background(t.bg_elevated)
}

/// Slider track fill (active portion)
pub fn slider_fill_style(s: Style) -> Style {
    let t = theme();
    s.height(4.0).border_radius(RADIUS_FULL).background(t.accent)
}

/// Slider thumb
pub fn slider_thumb_style(s: Style) -> Style {
    let t = theme();
    s.width(16.0)
        .height(16.0)
        .border_radius(RADIUS_FULL)
        .background(t.text_primary)
        .border(2.0)
        .border_color(t.accent)
}

/// Color swatch display
pub fn color_swatch_style(s: Style) -> Style {
    let t = theme();
    s.width(28.0)
        .height(28.0)
        .border_radius(RADIUS_SM)
        .border(2.0)
        .border_color(t.border)
}

/// Color input field container
pub fn color_input_container_style(s: Style) -> Style {
    let t = theme();
    s.background(t.bg_elevated)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(t.border_subtle)
        .padding_left(SPACING_SM)
        .items_center()
}

/// Text input field - uses elevated background for contrast in sections
pub fn text_input_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .background(t.bg_elevated)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(t.border)
        .color(t.text_primary)
        .font_size(FONT_SIZE_SM)
}

/// Dropdown/select field
pub fn dropdown_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_MD)
        .padding_vert(SPACING_SM)
        .background(t.bg_elevated)
        .border_radius(RADIUS_SM)
        .border(1.0)
        .border_color(t.border)
        .color(t.text_primary)
        .font_size(FONT_SIZE_SM)
        .min_width(160.0)
}

/// Value display (like "4px")
pub fn value_badge_style(s: Style) -> Style {
    let t = theme();
    s.padding_horiz(SPACING_SM)
        .padding_vert(SPACING_2XS)
        .background(t.bg_elevated)
        .border_radius(RADIUS_SM)
        .color(t.text_secondary)
        .font_size(FONT_SIZE_SM)
        .min_width(48.0)
}

/// Small icon button
pub fn icon_button_style(s: Style) -> Style {
    let t = theme();
    s.width(28.0)
        .height(28.0)
        .border_radius(RADIUS_SM)
        .items_center()
        .justify_center()
        .color(t.text_muted)
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
