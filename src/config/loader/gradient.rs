//! Gradient parsing helpers
//!
//! Functions for parsing gradient and color types from KDL documents.

use super::super::parser::get_string;
use super::helpers::parse_color;
use crate::types::{
    Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo, HueInterpolation,
};

/// Parse a gradient from a KDL node's entries.
///
/// Expected format:
/// `active-gradient from="#80c8ff" to="#bbddff" angle=45 relative-to="workspace-view" in="oklch shorter hue"`
pub fn parse_gradient_from_entries<'a>(
    entries: impl Iterator<Item = &'a kdl::KdlEntry>,
) -> Option<Gradient> {
    let mut from: Option<Color> = None;
    let mut to: Option<Color> = None;
    let mut angle = 180;
    let mut relative_to = GradientRelativeTo::Window;
    let mut color_space = ColorSpace::Srgb;
    let mut hue_interpolation: Option<HueInterpolation> = None;

    for entry in entries {
        if let Some(name) = entry.name() {
            match name.value() {
                "from" => {
                    if let Some(s) = entry.value().as_string() {
                        from = parse_color(s);
                    }
                }
                "to" => {
                    if let Some(s) = entry.value().as_string() {
                        to = parse_color(s);
                    }
                }
                "angle" => {
                    if let Some(v) = entry.value().as_integer() {
                        // Normalize angle to 0-359 range (handles negative values too)
                        angle = ((v % 360 + 360) % 360) as i32;
                    }
                }
                "relative-to" => {
                    if let Some(s) = entry.value().as_string() {
                        if let Some(rt) = GradientRelativeTo::from_kdl(s) {
                            relative_to = rt;
                        }
                    }
                }
                "in" => {
                    if let Some(s) = entry.value().as_string() {
                        if let Some(cs) = ColorSpace::from_kdl(s) {
                            color_space = cs;
                        }
                        // Check for hue interpolation (only for oklch)
                        if s.contains("hue") {
                            hue_interpolation = HueInterpolation::from_kdl(s);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Both from and to are required
    let from = from?;
    let to = to?;

    Some(Gradient {
        from,
        to,
        angle,
        relative_to,
        color_space,
        hue_interpolation,
    })
}

/// Try to load a gradient from a KDL node.
///
/// Looks for `{variant}-gradient` node (e.g., "active-gradient") in the given children.
pub fn load_gradient(children: &kdl::KdlDocument, variant: &str) -> Option<Gradient> {
    let gradient_key = format!("{}-gradient", variant);
    if let Some(node) = children.get(&gradient_key) {
        return parse_gradient_from_entries(node.entries().iter());
    }
    None
}

/// Load either a color or gradient from KDL.
///
/// First tries to find `{variant}-gradient`, then falls back to `{variant}-color`.
pub fn load_color_or_gradient(
    children: &kdl::KdlDocument,
    variant: &str,
) -> Option<ColorOrGradient> {
    // Try gradient first
    if let Some(gradient) = load_gradient(children, variant) {
        return Some(ColorOrGradient::Gradient(gradient));
    }

    // Fall back to solid color
    let color_key = format!("{}-color", variant);
    if let Some(hex) = get_string(children, &[&color_key]) {
        if let Some(color) = parse_color(&hex) {
            return Some(ColorOrGradient::Color(color));
        }
    }

    None
}
