//! Gradient KDL generation helpers
//!
//! Functions for converting gradient and color types to KDL string representations.

use crate::types::{ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo};

/// Generate KDL for a gradient with a given variant name (e.g., "active", "inactive").
///
/// Produces output like:
/// `active-gradient from="#80c8ff" to="#bbddff" angle=45 relative-to="workspace-view" in="oklch shorter hue"`
pub fn gradient_to_kdl(gradient: &Gradient, variant: &str) -> String {
    use std::fmt::Write;

    // Pre-allocate buffer - typical gradient is ~80-120 chars
    let mut output = String::with_capacity(128);

    // Write required parts directly to buffer (no intermediate allocations)
    // Note: write! to String is infallible - String implements fmt::Write
    // and always succeeds (may allocate but never returns Err)
    let _ = write!(
        output,
        "{}-gradient from=\"{}\" to=\"{}\"",
        variant,
        gradient.from.to_hex(),
        gradient.to.to_hex()
    );

    // Only output non-default angle
    if gradient.angle != 180 {
        let _ = write!(output, " angle={}", gradient.angle);
    }

    // Only output non-default relative-to
    if gradient.relative_to != GradientRelativeTo::Window {
        let _ = write!(output, " relative-to=\"{}\"", gradient.relative_to.to_kdl());
    }

    // Only output non-default color space
    if gradient.color_space != ColorSpace::Srgb {
        if gradient.color_space == ColorSpace::Oklch {
            // Include hue interpolation for oklch
            if let Some(ref hue) = gradient.hue_interpolation {
                let _ = write!(
                    output,
                    " in=\"{} {}\"",
                    gradient.color_space.to_kdl(),
                    hue.to_kdl()
                );
            } else {
                let _ = write!(output, " in=\"{}\"", gradient.color_space.to_kdl());
            }
        } else {
            let _ = write!(output, " in=\"{}\"", gradient.color_space.to_kdl());
        }
    }

    output
}

/// Generate KDL for a ColorOrGradient with a given variant name.
///
/// For solid colors, outputs: `active-color "#80c8ff"`
/// For gradients, outputs: `active-gradient from="#80c8ff" to="#bbddff" ...`
pub fn color_or_gradient_to_kdl(cog: &ColorOrGradient, variant: &str) -> String {
    use std::fmt::Write;
    match cog {
        ColorOrGradient::Color(c) => {
            // Pre-allocate for typical color output: "active-color \"#rrggbb\"" ~24 chars
            let mut output = String::with_capacity(32);
            // SAFETY: write! to String is infallible
            let _ = write!(output, "{}-color \"{}\"", variant, c.to_hex());
            output
        }
        ColorOrGradient::Gradient(g) => gradient_to_kdl(g, variant),
    }
}
