//! Catppuccin Mocha theme colors for Tessera UI
//!
//! Matches the theme used in the Slint version

/// Catppuccin Mocha color palette
pub struct Theme {
    // Base colors
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,

    // Text colors
    pub text: Color,
    pub subtext0: Color,
    pub subtext1: Color,

    // Accent colors
    pub blue: Color,
    pub lavender: Color,
    pub mauve: Color,
    pub red: Color,
    pub green: Color,

    // UI element colors
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,
    pub overlay0: Color,
}

/// RGB color representation
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Theme {
    /// Get the Catppuccin Mocha theme
    pub fn mocha() -> Self {
        Self {
            // Base colors
            base: Color { r: 30, g: 30, b: 46, a: 255 },
            mantle: Color { r: 24, g: 24, b: 37, a: 255 },
            crust: Color { r: 17, g: 17, b: 27, a: 255 },

            // Text
            text: Color { r: 205, g: 214, b: 244, a: 255 },
            subtext0: Color { r: 166, g: 173, b: 200, a: 255 },
            subtext1: Color { r: 186, g: 194, b: 222, a: 255 },

            // Accents
            blue: Color { r: 137, g: 180, b: 250, a: 255 },
            lavender: Color { r: 180, g: 190, b: 254, a: 255 },
            mauve: Color { r: 203, g: 166, b: 247, a: 255 },
            red: Color { r: 243, g: 139, b: 168, a: 255 },
            green: Color { r: 166, g: 227, b: 161, a: 255 },

            // Surfaces
            surface0: Color { r: 49, g: 50, b: 68, a: 255 },
            surface1: Color { r: 69, g: 71, b: 90, a: 255 },
            surface2: Color { r: 88, g: 91, b: 112, a: 255 },
            overlay0: Color { r: 108, g: 112, b: 134, a: 255 },
        }
    }
}
