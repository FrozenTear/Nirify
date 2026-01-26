//! Custom theme for Nirify
//!
//! A refined dark theme with warm undertones and precise spacing.
//! Inspired by professional creative tools and audio software.

use iced::widget::{button, container};
use iced::theme::Palette;
use iced::{Border, Color, Shadow, Theme, Vector};

/// Font constants for consistent typography
pub mod fonts {
    use iced::Font;
    use iced::font::{Family, Weight};

    /// Primary UI font - clean, modern sans-serif
    /// Using system defaults for maximum compatibility
    pub const UI_FONT: Font = Font {
        family: Family::SansSerif,
        weight: Weight::Normal,
        ..Font::DEFAULT
    };

    /// UI font (medium weight) for emphasis
    pub const UI_FONT_MEDIUM: Font = Font {
        family: Family::SansSerif,
        weight: Weight::Medium,
        ..Font::DEFAULT
    };

    /// UI font (semibold) for strong emphasis
    pub const UI_FONT_SEMIBOLD: Font = Font {
        family: Family::SansSerif,
        weight: Weight::Semibold,
        ..Font::DEFAULT
    };

    /// Monospace font for technical content (numbers, code, paths, hex values)
    pub const MONO_FONT: Font = Font {
        family: Family::Monospace,
        weight: Weight::Normal,
        ..Font::DEFAULT
    };

    /// Monospace medium weight
    pub const MONO_FONT_MEDIUM: Font = Font {
        family: Family::Monospace,
        weight: Weight::Medium,
        ..Font::DEFAULT
    };
}

/// Application theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    /// Follow system theme (portal or pywal/wallust)
    System,
    /// Default niri theme with warm amber/teal palette
    #[default]
    NiriAmber,
    /// Catppuccin Latte - Light, creamy pastels
    CatppuccinLatte,
    /// Catppuccin Frappe - Warm, muted dark
    CatppuccinFrappe,
    /// Catppuccin Macchiato - Rich, vibrant dark
    CatppuccinMacchiato,
    /// Catppuccin Mocha - Deep, cozy dark
    CatppuccinMocha,
    /// Dracula - Purple-tinted dark theme
    Dracula,
    /// Nord - Arctic, bluish dark theme
    Nord,
    /// Gruvbox Dark - Retro, earthy dark theme
    GruvboxDark,
    /// Gruvbox Light - Retro, earthy light theme
    GruvboxLight,
    /// Tokyo Night - Neon-accented dark theme
    TokyoNight,
    /// Solarized Dark - Classic low-contrast dark
    SolarizedDark,
    /// Solarized Light - Classic low-contrast light
    SolarizedLight,
}


impl AppTheme {
    /// Returns the iced Theme for this app theme
    ///
    /// Note: For `AppTheme::System`, this returns the NiriAmber fallback.
    /// The actual system theme should be resolved by the App using `SystemThemeState`.
    pub fn to_iced_theme(self) -> Theme {
        match self {
            AppTheme::System => build_niri_amber_theme(), // Fallback; App handles actual system theme
            AppTheme::NiriAmber => build_niri_amber_theme(),
            AppTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            AppTheme::Dracula => Theme::Dracula,
            AppTheme::Nord => Theme::Nord,
            AppTheme::GruvboxDark => Theme::GruvboxDark,
            AppTheme::GruvboxLight => Theme::GruvboxLight,
            AppTheme::TokyoNight => Theme::TokyoNight,
            AppTheme::SolarizedDark => Theme::SolarizedDark,
            AppTheme::SolarizedLight => Theme::SolarizedLight,
        }
    }

    /// Returns all available themes for theme selector
    pub fn all() -> &'static [AppTheme] {
        &[
            // System theme first (follows desktop/wallpaper)
            AppTheme::System,
            // Custom theme
            AppTheme::NiriAmber,
            // Catppuccin family (most popular)
            AppTheme::CatppuccinMocha,
            AppTheme::CatppuccinMacchiato,
            AppTheme::CatppuccinFrappe,
            AppTheme::CatppuccinLatte,
            // Other popular dark themes
            AppTheme::Dracula,
            AppTheme::Nord,
            AppTheme::TokyoNight,
            AppTheme::GruvboxDark,
            AppTheme::SolarizedDark,
            // Light themes at the end
            AppTheme::GruvboxLight,
            AppTheme::SolarizedLight,
        ]
    }

    /// Human-readable name with light/dark indicator
    pub fn name(self) -> &'static str {
        match self {
            AppTheme::System => "System (Auto)",
            AppTheme::NiriAmber => "Niri Amber (Dark)",
            AppTheme::CatppuccinLatte => "Catppuccin Latte (Light)",
            AppTheme::CatppuccinFrappe => "Catppuccin Frappé (Dark)",
            AppTheme::CatppuccinMacchiato => "Catppuccin Macchiato (Dark)",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha (Dark)",
            AppTheme::Dracula => "Dracula (Dark)",
            AppTheme::Nord => "Nord (Dark)",
            AppTheme::GruvboxDark => "Gruvbox (Dark)",
            AppTheme::GruvboxLight => "Gruvbox (Light)",
            AppTheme::TokyoNight => "Tokyo Night (Dark)",
            AppTheme::SolarizedDark => "Solarized (Dark)",
            AppTheme::SolarizedLight => "Solarized (Light)",
        }
    }

    /// Returns whether this is a light theme
    ///
    /// Note: `System` returns false (dark) as a default; actual light/dark
    /// detection should be done via `SystemThemeState`.
    pub fn is_light(self) -> bool {
        matches!(
            self,
            AppTheme::CatppuccinLatte | AppTheme::GruvboxLight | AppTheme::SolarizedLight
        )
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for AppTheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "System" => Ok(Self::System),
            "NiriAmber" => Ok(Self::NiriAmber),
            "CatppuccinLatte" => Ok(Self::CatppuccinLatte),
            "CatppuccinFrappe" => Ok(Self::CatppuccinFrappe),
            "CatppuccinMacchiato" => Ok(Self::CatppuccinMacchiato),
            "CatppuccinMocha" => Ok(Self::CatppuccinMocha),
            "Dracula" => Ok(Self::Dracula),
            "Nord" => Ok(Self::Nord),
            "GruvboxDark" => Ok(Self::GruvboxDark),
            "GruvboxLight" => Ok(Self::GruvboxLight),
            "TokyoNight" => Ok(Self::TokyoNight),
            "SolarizedDark" => Ok(Self::SolarizedDark),
            "SolarizedLight" => Ok(Self::SolarizedLight),
            _ => Ok(Self::default()),
        }
    }
}

impl AppTheme {
    /// Converts theme to a string (for persistence)
    pub fn to_str(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::NiriAmber => "NiriAmber",
            Self::CatppuccinLatte => "CatppuccinLatte",
            Self::CatppuccinFrappe => "CatppuccinFrappe",
            Self::CatppuccinMacchiato => "CatppuccinMacchiato",
            Self::CatppuccinMocha => "CatppuccinMocha",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::GruvboxDark => "GruvboxDark",
            Self::GruvboxLight => "GruvboxLight",
            Self::TokyoNight => "TokyoNight",
            Self::SolarizedDark => "SolarizedDark",
            Self::SolarizedLight => "SolarizedLight",
        }
    }
}

/// Custom color palette for Nirify
pub struct NiriColors {
    // Background layers
    pub bg_base: Color,           // #1a1d23 - Deep charcoal base
    pub bg_surface: Color,        // #23272f - Elevated surfaces
    pub bg_surface_hover: Color,  // #2a2f38 - Hover state
    pub bg_input: Color,          // #2d323c - Input fields

    // Text hierarchy
    pub text_primary: Color,      // #e6e8eb - High contrast text
    pub text_secondary: Color,    // #9ca3af - Secondary text
    pub text_tertiary: Color,     // #6b7280 - Disabled/tertiary

    // Accent colors
    pub accent_primary: Color,    // #f59e42 - Warm amber
    pub accent_secondary: Color,  // #4fd1c5 - Teal cyan
    pub accent_tertiary: Color,   // #8b5cf6 - Purple (for special states)

    // Semantic colors
    pub success: Color,           // #10b981 - Green
    pub warning: Color,           // #f59e0b - Amber
    pub error: Color,             // #ef4444 - Red

    // Borders and dividers
    pub border_subtle: Color,     // #3a3f4b - Subtle borders
    pub border_strong: Color,     // #4b5563 - Strong borders

    // Special effects
    pub glow_accent: Color,       // #f59e42 with alpha - Warm glow
    pub shadow_color: Color,      // Black with alpha - Shadows
}

impl Default for NiriColors {
    fn default() -> Self {
        Self {
            bg_base: Color::from_rgb(0.102, 0.114, 0.137),           // #1a1d23
            bg_surface: Color::from_rgb(0.137, 0.153, 0.184),        // #23272f
            bg_surface_hover: Color::from_rgb(0.165, 0.184, 0.220),  // #2a2f38
            bg_input: Color::from_rgb(0.176, 0.196, 0.235),          // #2d323c

            text_primary: Color::from_rgb(0.902, 0.910, 0.922),      // #e6e8eb
            text_secondary: Color::from_rgb(0.612, 0.639, 0.686),    // #9ca3af
            text_tertiary: Color::from_rgb(0.420, 0.447, 0.502),     // #6b7280

            accent_primary: Color::from_rgb(0.961, 0.620, 0.259),    // #f59e42
            accent_secondary: Color::from_rgb(0.310, 0.820, 0.773),  // #4fd1c5
            accent_tertiary: Color::from_rgb(0.545, 0.361, 0.965),   // #8b5cf6

            success: Color::from_rgb(0.063, 0.725, 0.506),           // #10b981
            warning: Color::from_rgb(0.961, 0.620, 0.043),           // #f59e0b
            error: Color::from_rgb(0.937, 0.267, 0.267),             // #ef4444

            border_subtle: Color::from_rgb(0.227, 0.247, 0.294),     // #3a3f4b
            border_strong: Color::from_rgb(0.294, 0.333, 0.388),     // #4b5563

            glow_accent: Color::from_rgba(0.961, 0.620, 0.259, 0.15), // #f59e42 with alpha
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        }
    }
}

/// Builds the custom NiriAmber theme from NiriColors palette
fn build_niri_amber_theme() -> Theme {
    let colors = NiriColors::default();

    // Create the iced Palette from our custom colors
    let palette = Palette {
        background: colors.bg_base,
        text: colors.text_primary,
        primary: colors.accent_primary,
        success: colors.success,
        warning: colors.warning,
        danger: colors.error,
    };

    // Create a custom theme with our palette
    Theme::custom("Niri Amber".to_string(), palette)
}

/// Returns the custom niri theme (uses default AppTheme)
pub fn niri_theme() -> Theme {
    AppTheme::default().to_iced_theme()
}

/// Custom button style for navigation tabs - uses theme palette
pub fn nav_tab_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();
        let primary = palette.primary;
        let bg = palette.background;
        let text = palette.text;

        // Derive surface colors from background
        let surface_hover = lighten(bg, 0.12);
        let surface = lighten(bg, 0.08);
        let border_subtle = lighten(bg, 0.15);
        let text_secondary = Color { a: 0.7, ..text };
        let glow = Color { a: 0.15, ..primary };

        let base_bg = if active { primary } else { Color::TRANSPARENT };
        let text_color = if active { bg } else { text_secondary };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(
                    if active {
                        Color { a: 0.9, ..primary }
                    } else {
                        surface_hover
                    }
                )),
                text_color,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: if active { glow } else { Color::TRANSPARENT },
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                snap: false,
            },
            button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(
                    if active {
                        Color { a: 0.8, ..primary }
                    } else {
                        surface
                    }
                )),
                text_color,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            _ => button::Style {
                background: Some(iced::Background::Color(base_bg)),
                text_color,
                border: Border {
                    color: if active { Color::TRANSPARENT } else { border_subtle },
                    width: if active { 0.0 } else { 1.0 },
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: if active { glow } else { Color::TRANSPARENT },
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                snap: false,
            },
        }
    }
}

/// Custom button style for sub-navigation tabs - uses theme palette
pub fn subnav_tab_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();
        let primary = palette.primary;
        let bg = palette.background;
        let text = palette.text;

        // Derive colors from theme
        let surface_hover = lighten(bg, 0.12);
        let surface = lighten(bg, 0.08);
        let text_secondary = Color { a: 0.7, ..text };

        let text_color = if active { primary } else { text_secondary };
        let border_color = if active { primary } else { Color::TRANSPARENT };

        match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(surface_hover)),
                text_color: if active { primary } else { text },
                border: Border {
                    color: border_color,
                    width: if active { 2.0 } else { 0.0 },
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(surface)),
                text_color,
                border: Border {
                    color: border_color,
                    width: if active { 2.0 } else { 0.0 },
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            _ => button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color,
                border: Border {
                    color: border_color,
                    width: if active { 2.0 } else { 0.0 },
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
        }
    }
}

/// Container style for search bar - uses theme palette
pub fn search_container_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let bg = palette.background;
    let text = palette.text;

    let input_bg = lighten(bg, 0.10);
    let border = lighten(bg, 0.15);

    container::Style {
        background: Some(iced::Background::Color(input_bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: Some(text),
        snap: false,
    }
}

/// Container style for the main navigation bar - uses theme palette
pub fn nav_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let bg = palette.background;
    let text = palette.text;

    let surface = lighten(bg, 0.05);

    container::Style {
        background: Some(iced::Background::Color(surface)),
        border: Border {
            color: lighten(bg, 0.12),
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 12.0,
        },
        text_color: Some(text),
        snap: false,
    }
}

/// Container style for the sub-navigation bar - uses theme palette
pub fn subnav_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let bg = palette.background;
    let text = palette.text;
    let text_secondary = Color { a: 0.7, ..text };

    container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: lighten(bg, 0.12),
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
        text_color: Some(text_secondary),
        snap: false,
    }
}

/// Container style for the status bar at bottom - uses theme palette
pub fn status_bar_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let bg = palette.background;
    let text = palette.text;

    let surface = lighten(bg, 0.05);
    let border = lighten(bg, 0.12);
    let text_secondary = Color { a: 0.7, ..text };

    container::Style {
        background: Some(iced::Background::Color(surface)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(text_secondary),
        snap: false,
    }
}

/// Container style for the search dropdown - uses theme palette
pub fn search_dropdown_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    let bg = palette.background;
    let text = palette.text;

    let surface = lighten(bg, 0.05);
    let border = lighten(bg, 0.18);

    container::Style {
        background: Some(iced::Background::Color(surface)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: Some(text),
        snap: false,
    }
}

/// Button style for search dropdown items - uses theme palette
pub fn search_dropdown_item_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();
        let bg = palette.background;
        let text = palette.text;

        let surface_hover = lighten(bg, 0.12);
        let input_bg = lighten(bg, 0.10);

        let item_bg = match status {
            button::Status::Hovered => surface_hover,
            button::Status::Pressed => input_bg,
            _ => Color::TRANSPARENT,
        };

        button::Style {
            background: Some(iced::Background::Color(item_bg)),
            text_color: text,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// Container style for setting cards - elevated surface with subtle border
/// Respects the current theme's color palette.
pub fn card_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    // Derive surface color: slightly lighter than background
    let surface = lighten(palette.background, 0.05);
    let border = lighten(palette.background, 0.12);

    container::Style {
        background: Some(iced::Background::Color(surface)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        text_color: Some(palette.text),
        snap: false,
    }
}

/// Container style for info/hint blocks
/// Uses the theme's success color for a subtle tint.
pub fn info_block_style(theme: &Theme) -> container::Style {
    let palette = theme.palette();
    // Use success color with low opacity for info blocks
    let tint = Color { a: 0.15, ..palette.success };
    let border = Color { a: 0.4, ..palette.success };

    container::Style {
        background: Some(iced::Background::Color(tint)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        text_color: Some(palette.text),
        snap: false,
    }
}

/// Helper: Lighten a color by a factor (0.0 = no change, 1.0 = white)
fn lighten(color: Color, factor: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * factor,
        g: color.g + (1.0 - color.g) * factor,
        b: color.b + (1.0 - color.b) * factor,
        a: color.a,
    }
}

/// Helper: Get the primary accent color from theme
pub fn accent_color(theme: &Theme) -> Color {
    theme.palette().primary
}

/// Helper: Get muted text color from theme
pub fn muted_text_color(theme: &Theme) -> Color {
    let text = theme.palette().text;
    Color { a: 0.5, ..text }
}

/// Helper: Get secondary text color from theme
pub fn secondary_text_color(theme: &Theme) -> Color {
    let text = theme.palette().text;
    Color { a: 0.7, ..text }
}

/// Container style for muted/description text - uses theme's text color with reduced opacity
pub fn muted_text_container(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(muted_text_color(theme)),
        ..Default::default()
    }
}

/// Container style for disabled text - uses theme's text color with very low opacity
pub fn disabled_text_container(theme: &Theme) -> container::Style {
    let text = theme.palette().text;
    container::Style {
        text_color: Some(Color { a: 0.35, ..text }),
        ..Default::default()
    }
}

/// Container style for secondary text (value displays, etc.)
pub fn secondary_text_container(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(secondary_text_color(theme)),
        ..Default::default()
    }
}

/// Container style for code/example text - uses theme's success color for a subtle green tint
pub fn code_text_container(theme: &Theme) -> container::Style {
    let success = theme.palette().success;
    container::Style {
        text_color: Some(Color { a: 0.85, ..success }),
        ..Default::default()
    }
}
