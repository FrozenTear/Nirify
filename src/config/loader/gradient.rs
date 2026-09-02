//! Gradient and color parsing
//!
//! # Round-trip policy
//!
//! niri accepts a **2-stop linear gradient** on `*-gradient` (and insert-hint
//! `gradient`) nodes:
//!
//! ```kdl
//! active-gradient from="#80c8ff" to="#bbddff" angle=45 relative-to="workspace-view" in="oklch shorter hue"
//! ```
//!
//! Supported properties: `from`, `to` (required), optional `angle`,
//! `relative-to` (`window` | `workspace-view`), and `in` (`srgb` |
//! `srgb-linear` | `oklab` | `oklch` plus optional oklch hue interpolation).
//! Colors accept the same CSS strings as niri (`#hex`, `rgb()`, named, …).
//!
//! That form is modeled as [`Gradient`] and written back in niri-valid KDL.
//!
//! ## Remaining unsupported shapes (kept raw, never stripped)
//!
//! If a gradient node is present but is **not** the form above, the whole
//! node is stored as [`ColorOrGradient::Raw`] and re-emitted verbatim:
//!
//! - Extra / unknown properties (`stop=`, `via=`, …)
//! - Unexpected positional arguments or child nodes (extra color stops)
//! - Missing or unparseable `from` / `to`
//! - Unknown `relative-to` or `in` color-space tokens
//! - CSS `linear-gradient(...)` functions (niri does not use this syntax)
//!
//! Raw is a Config-path guarantee so first-run import, absorb, and re-save
//! cannot drop a gradient the UI does not yet model. Editing the field in
//! the UI may convert it to a solid or a standard 2-stop gradient.

use super::super::parser::get_string;
use super::helpers::parse_color;
use crate::types::{
    Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo, HueInterpolation,
};
use kdl::KdlNode;
use log::debug;

/// Why a gradient node could not be modeled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedGradient {
    MissingFromTo,
    UnknownProperty,
    UnexpectedPositional,
    HasChildren,
    UnknownRelativeTo,
    UnknownColorSpace,
}

/// Render a KDL node as a single-line raw string (name + entries [+ children]).
pub fn raw_kdl_node(node: &KdlNode) -> String {
    let mut out = node.name().value().to_string();
    for entry in node.entries() {
        out.push(' ');
        out.push_str(&entry.to_string());
    }
    if let Some(children) = node.children() {
        if !children.nodes().is_empty() {
            out.push_str(" {");
            for child in children.nodes() {
                out.push(' ');
                out.push_str(&raw_kdl_node(child));
            }
            out.push_str(" }");
        }
    }
    out
}

/// Parse a supported niri 2-stop gradient from a node's entries.
///
/// Returns `Err` when the node is not the modeled form so callers can keep
/// the raw node instead of dropping it.
pub fn parse_supported_gradient(node: &KdlNode) -> Result<Gradient, ()> {
    parse_supported_gradient_inner(node).map_err(|reason| {
        debug!(
            "Keeping gradient node {:?} as raw ({:?})",
            node.name().value(),
            reason
        );
    })
}

fn parse_supported_gradient_inner(node: &KdlNode) -> Result<Gradient, UnsupportedGradient> {
    if node.children().is_some_and(|ch| !ch.nodes().is_empty()) {
        return Err(UnsupportedGradient::HasChildren);
    }

    let mut from: Option<Color> = None;
    let mut to: Option<Color> = None;
    let mut angle = 180;
    let mut relative_to = GradientRelativeTo::Window;
    let mut color_space = ColorSpace::Srgb;
    let mut hue_interpolation: Option<HueInterpolation> = None;

    for entry in node.entries() {
        let Some(name) = entry.name() else {
            return Err(UnsupportedGradient::UnexpectedPositional);
        };
        match name.value() {
            "from" => {
                let Some(s) = entry.value().as_string() else {
                    return Err(UnsupportedGradient::MissingFromTo);
                };
                from = Some(parse_color(s).ok_or(UnsupportedGradient::MissingFromTo)?);
            }
            "to" => {
                let Some(s) = entry.value().as_string() else {
                    return Err(UnsupportedGradient::MissingFromTo);
                };
                to = Some(parse_color(s).ok_or(UnsupportedGradient::MissingFromTo)?);
            }
            "angle" => {
                if let Some(v) = entry.value().as_integer() {
                    angle = ((v % 360 + 360) % 360) as i32;
                } else if let Some(v) = entry.value().as_float() {
                    let rounded = v.round() as i64;
                    angle = ((rounded % 360 + 360) % 360) as i32;
                } else {
                    return Err(UnsupportedGradient::UnknownProperty);
                }
            }
            "relative-to" => {
                let Some(s) = entry.value().as_string() else {
                    return Err(UnsupportedGradient::UnknownRelativeTo);
                };
                relative_to = GradientRelativeTo::from_kdl(s)
                    .ok_or(UnsupportedGradient::UnknownRelativeTo)?;
            }
            "in" => {
                let Some(s) = entry.value().as_string() else {
                    return Err(UnsupportedGradient::UnknownColorSpace);
                };
                color_space =
                    ColorSpace::from_kdl(s).ok_or(UnsupportedGradient::UnknownColorSpace)?;
                if s.contains("hue") {
                    hue_interpolation = HueInterpolation::from_kdl(s);
                }
            }
            _ => return Err(UnsupportedGradient::UnknownProperty),
        }
    }

    let from = from.ok_or(UnsupportedGradient::MissingFromTo)?;
    let to = to.ok_or(UnsupportedGradient::MissingFromTo)?;

    Ok(Gradient {
        from,
        to,
        angle,
        relative_to,
        color_space,
        hue_interpolation,
    })
}

/// Load a gradient node as modeled [`Gradient`] or lossless [`ColorOrGradient::Raw`].
pub fn load_gradient_node(node: &KdlNode) -> ColorOrGradient {
    match parse_supported_gradient(node) {
        Ok(gradient) => ColorOrGradient::Gradient(gradient),
        Err(()) => ColorOrGradient::Raw(raw_kdl_node(node)),
    }
}

/// Parse a gradient from a KDL node's entries.
///
/// Expected format:
/// `active-gradient from="#80c8ff" to="#bbddff" angle=45 relative-to="workspace-view" in="oklch shorter hue"`
///
/// Prefer [`load_gradient_node`] when a missing/unknown form must be kept raw.
pub fn parse_gradient_from_entries<'a>(
    entries: impl Iterator<Item = &'a kdl::KdlEntry>,
) -> Option<Gradient> {
    let entries: Vec<_> = entries.collect();
    // Reconstruct a throwaway node so the strict parser can run. Callers that
    // still use this helper treat failure as "no gradient" (legacy).
    let mut node = KdlNode::new("gradient");
    for entry in entries {
        node.entries_mut().push(entry.clone());
    }
    parse_supported_gradient(&node).ok()
}

/// Try to load a gradient from a KDL node.
///
/// Looks for `{variant}-gradient` node (e.g., "active-gradient") in the given children.
/// Unmodeled forms are **not** returned here — use [`load_color_or_gradient`].
pub fn load_gradient(children: &kdl::KdlDocument, variant: &str) -> Option<Gradient> {
    let gradient_key = format!("{}-gradient", variant);
    children
        .get(&gradient_key)
        .and_then(|node| parse_supported_gradient(node).ok())
}

/// Load either a color or gradient from KDL.
///
/// First tries `{variant}-gradient` (modeled or raw), then `{variant}-color`.
pub fn load_color_or_gradient(
    children: &kdl::KdlDocument,
    variant: &str,
) -> Option<ColorOrGradient> {
    let gradient_key = format!("{}-gradient", variant);
    if let Some(node) = children.get(&gradient_key) {
        return Some(load_gradient_node(node));
    }

    let color_key = format!("{}-color", variant);
    if let Some(hex) = get_string(children, &[&color_key]) {
        if let Some(color) = parse_color(&hex) {
            return Some(ColorOrGradient::Color(color));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parser::parse_document;

    fn first_child<'a>(kdl: &'a str, parent: &str) -> kdl::KdlDocument {
        let doc = parse_document(kdl).unwrap();
        doc.get(parent).unwrap().children().unwrap().clone()
    }

    #[test]
    fn load_modeled_gradient_with_all_attrs() {
        let ch = first_child(
            r##"
focus-ring {
    active-gradient from="#80c8ff" to="#bbddff" angle=45 relative-to="workspace-view" in="oklch shorter hue"
}
"##,
            "focus-ring",
        );
        let cog = load_color_or_gradient(&ch, "active").unwrap();
        match cog {
            ColorOrGradient::Gradient(g) => {
                assert_eq!(g.from.to_hex(), "#80c8ff");
                assert_eq!(g.to.to_hex(), "#bbddff");
                assert_eq!(g.angle, 45);
                assert_eq!(g.relative_to, GradientRelativeTo::WorkspaceView);
                assert_eq!(g.color_space, ColorSpace::Oklch);
                assert_eq!(g.hue_interpolation, Some(HueInterpolation::Shorter));
            }
            other => panic!("expected modeled gradient, got {other:?}"),
        }
    }

    #[test]
    fn unknown_property_is_kept_raw() {
        let ch = first_child(
            r##"
focus-ring {
    active-gradient from="#80c8ff" to="#bbddff" via="#ffffff"
}
"##,
            "focus-ring",
        );
        let cog = load_color_or_gradient(&ch, "active").unwrap();
        match cog {
            ColorOrGradient::Raw(raw) => {
                assert!(raw.contains("active-gradient"), "{raw}");
                assert!(raw.contains("via="), "{raw}");
            }
            other => panic!("expected raw, got {other:?}"),
        }
    }

    #[test]
    fn missing_from_is_kept_raw_not_dropped() {
        let ch = first_child(
            r##"
focus-ring {
    urgent-gradient to="#ff0000"
}
"##,
            "focus-ring",
        );
        let cog = load_color_or_gradient(&ch, "urgent").unwrap();
        assert!(cog.is_raw(), "{cog:?}");
    }

    #[test]
    fn solid_color_still_loads() {
        let ch = first_child(
            r##"
border {
    inactive-color "#333333"
}
"##,
            "border",
        );
        match load_color_or_gradient(&ch, "inactive") {
            Some(ColorOrGradient::Color(c)) => assert_eq!(c.to_hex(), "#333333"),
            other => panic!("expected color, got {other:?}"),
        }
    }
}
