//! Appearance settings (focus ring, border, gaps, corner radius)

use crate::constants::{
    DEFAULT_BORDER_COLOR, DEFAULT_BORDER_INACTIVE_COLOR, DEFAULT_BORDER_WIDTH,
    DEFAULT_CORNER_RADIUS, DEFAULT_FOCUS_RING_COLOR, DEFAULT_FOCUS_RING_INACTIVE_COLOR,
    DEFAULT_FOCUS_RING_WIDTH, DEFAULT_GAPS_OUTER, DEFAULT_GAP_SIZE,
};
use crate::types::{Color, ColorOrGradient};

/// Appearance settings (layout, focus ring, border, struts)
#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceSettings {
    // Focus ring
    pub focus_ring_enabled: bool,
    pub focus_ring_width: f32,
    pub focus_ring_active: ColorOrGradient,
    pub focus_ring_inactive: ColorOrGradient,
    pub focus_ring_urgent: ColorOrGradient,

    // Window border
    pub border_enabled: bool,
    pub border_thickness: f32,
    pub border_active: ColorOrGradient,
    pub border_inactive: ColorOrGradient,
    pub border_urgent: ColorOrGradient,

    // Gaps
    pub gaps_inner: f32,
    pub gaps_outer: f32,

    // Corner radius
    pub corner_radius: f32,

    // Background
    pub background_color: Option<Color>,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        // Note: unwrap_or_default() is used for color parsing because:
        // 1. The DEFAULT_* constants are compile-time verified hex strings
        // 2. If parsing somehow fails, Color::default() (black with 0 alpha) is a safe fallback
        // 3. This avoids panic in Default trait implementation
        Self {
            focus_ring_enabled: true,
            focus_ring_width: DEFAULT_FOCUS_RING_WIDTH as f32,
            focus_ring_active: ColorOrGradient::Color(
                Color::from_hex(DEFAULT_FOCUS_RING_COLOR).unwrap_or_default(),
            ),
            focus_ring_inactive: ColorOrGradient::Color(
                Color::from_hex(DEFAULT_FOCUS_RING_INACTIVE_COLOR).unwrap_or_default(),
            ),
            focus_ring_urgent: ColorOrGradient::Color(
                Color::from_hex("#eb6f92").unwrap_or_default(), // Rose Pine urgent
            ),
            border_enabled: false,
            border_thickness: DEFAULT_BORDER_WIDTH as f32,
            border_active: ColorOrGradient::Color(
                Color::from_hex(DEFAULT_BORDER_COLOR).unwrap_or_default(),
            ),
            border_inactive: ColorOrGradient::Color(
                Color::from_hex(DEFAULT_BORDER_INACTIVE_COLOR).unwrap_or_default(),
            ),
            border_urgent: ColorOrGradient::Color(
                Color::from_hex("#eb6f92").unwrap_or_default(),
            ),
            gaps_inner: DEFAULT_GAP_SIZE as f32,
            gaps_outer: DEFAULT_GAPS_OUTER,
            corner_radius: DEFAULT_CORNER_RADIUS,
            background_color: None,
        }
    }
}
