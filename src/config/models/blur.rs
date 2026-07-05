//! Top-level background blur settings (niri >= 26.04)

/// Top-level background blur settings (niri >= 26.04)
#[derive(Debug, Clone, PartialEq)]
pub struct BlurSettings {
    /// false => emit `off` flag inside blur {}
    pub enabled: bool,
    /// Dual-kawase passes. niri: u8. Clamp 0..=255 (UI slider 0..=8).
    pub passes: i32,
    /// Pixel offset multiplier per pass. niri bound 0..=100.
    pub offset: f64,
    /// Noise to reduce banding. niri bound 0..=1000 (sensible values 0..=1).
    pub noise: f64,
    /// Saturation of blurred background. niri bound 0..=1000 (sensible 0..=3).
    pub saturation: f64,
}

impl Default for BlurSettings {
    fn default() -> Self {
        // Mirrors niri Blur::default()
        Self {
            enabled: true,
            passes: 3,
            offset: 3.0,
            noise: 0.02,
            saturation: 1.5,
        }
    }
}
