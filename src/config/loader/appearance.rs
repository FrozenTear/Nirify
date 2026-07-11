//! Appearance settings loader
//!
//! Loads visual appearance settings from KDL configuration.

use super::super::parser::{get_f64, get_i64, get_string, has_flag};
use super::gradient::load_color_or_gradient;
use super::helpers::{parse_color, read_kdl_file};
use crate::config::models::{ColumnWidthType, Settings};
use crate::types::{CenterFocusedColumn, ColorOrGradient};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Parsed data for styled features like focus-ring and border.
///
/// These features share a common structure: they can be enabled/disabled,
/// have a width, active/inactive/urgent colors or gradients.
struct StyledFeatureData {
    /// Whether the feature is enabled (false if "off" is present)
    enabled: bool,
    /// Width of the feature (e.g., ring width or border thickness)
    width: Option<f32>,
    /// Active color or gradient
    active: Option<ColorOrGradient>,
    /// Inactive color or gradient
    inactive: Option<ColorOrGradient>,
    /// Urgent color or gradient
    urgent: Option<ColorOrGradient>,
}

/// Parse a styled feature (focus-ring, border) from a layout's children.
///
/// Both focus-ring and border share the same structure:
/// - Enabled state is controlled by explicit `on`/`off` flags (default on)
/// - Have a "width" property
/// - Have "active-color" or "active-gradient" for active state
/// - Have "inactive-color" or "inactive-gradient" for inactive state
/// - Have "urgent-color" or "urgent-gradient" for urgent windows
///
/// Styling is parsed unconditionally so that width/colors round-trip even when
/// the feature is disabled.
///
/// Returns `None` if the feature node is not present in the document.
fn parse_styled_feature(children: &KdlDocument, feature_name: &str) -> Option<StyledFeatureData> {
    let node = children.get(feature_name)?;
    let feature_children = node.children()?;

    // Enabled by default; an explicit `off` flag disables it. An `on` flag is
    // simply the default and requires no special handling.
    let enabled = !has_flag(feature_children, &["off"]);

    // Parse all styling properties regardless of enabled state.
    let width = get_i64(feature_children, &["width"])
        .map(|w| w as f32)
        .or_else(|| get_f64(feature_children, &["width"]).map(|w| w as f32));
    let active = load_color_or_gradient(feature_children, "active");
    let inactive = load_color_or_gradient(feature_children, "inactive");
    let urgent = load_color_or_gradient(feature_children, "urgent");

    Some(StyledFeatureData {
        enabled,
        width,
        active,
        inactive,
        urgent,
    })
}

/// Load appearance settings from KDL file
pub fn load_appearance(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_appearance_from_doc(&doc, settings);

    // Window rule for corner radius (global) - only first one in managed file
    if let Some(wr) = doc.get("window-rule") {
        if let Some(wr_children) = wr.children() {
            if let Some(cr) = get_i64(wr_children, &["geometry-corner-radius"]) {
                settings.appearance.corner_radius = cr as f32;
            }
        }
    }

    debug!("Loaded appearance settings from {:?}", path);
}

/// Parse appearance settings from a KDL document
///
/// This is the shared parsing logic used by both `load_appearance()` (for managed files)
/// and `import_appearance_from_doc()` (for importing from user's config).
pub fn parse_appearance_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // Get layout node
    if let Some(layout) = doc.get("layout") {
        if let Some(layout_children) = layout.children() {
            parse_layout_children(layout_children, settings);
        }
    }
}

/// Parse the children of a layout node into settings
///
/// Shared between loader and importer for DRY parsing of layout settings.
pub fn parse_layout_children(layout_children: &KdlDocument, settings: &mut Settings) {
    // Gaps - niri uses a single value: `gaps 16`
    // For backwards compatibility, we also handle `gaps inner=X outer=Y` by taking the inner value
    if let Some(gaps_node) = layout_children.get("gaps") {
        let mut found_value = false;

        // Check for named entries (backwards compatibility with old configs)
        for entry in gaps_node.entries() {
            if let Some(name) = entry.name() {
                if let Some(val) = entry.value().as_integer() {
                    // Use inner value if present (for backwards compatibility)
                    if name.value() == "inner" {
                        settings.appearance.gaps = val as f32;
                        found_value = true;
                        break;
                    }
                }
            }
        }

        // Check for a single positional argument (the standard format)
        if !found_value {
            if let Some(first_entry) = gaps_node.entries().iter().next() {
                if first_entry.name().is_none() {
                    if let Some(val) = first_entry.value().as_integer() {
                        settings.appearance.gaps = val as f32;
                    }
                }
            }
        }
    }

    // Focus ring
    if let Some(data) = parse_styled_feature(layout_children, "focus-ring") {
        settings.appearance.focus_ring_enabled = data.enabled;
        if let Some(width) = data.width {
            settings.appearance.focus_ring_width = width;
        }
        if let Some(active) = data.active {
            settings.appearance.focus_ring_active = active;
        }
        if let Some(inactive) = data.inactive {
            settings.appearance.focus_ring_inactive = inactive;
        }
        if let Some(urgent) = data.urgent {
            settings.appearance.focus_ring_urgent = urgent;
        }
    }

    // Border
    if let Some(data) = parse_styled_feature(layout_children, "border") {
        settings.appearance.border_enabled = data.enabled;
        if let Some(width) = data.width {
            settings.appearance.border_thickness = width;
        }
        if let Some(active) = data.active {
            settings.appearance.border_active = active;
        }
        if let Some(inactive) = data.inactive {
            settings.appearance.border_inactive = inactive;
        }
        if let Some(urgent) = data.urgent {
            settings.appearance.border_urgent = urgent;
        }
    }

    // Struts
    if let Some(struts) = layout_children.get("struts") {
        if let Some(s_children) = struts.children() {
            if let Some(v) = get_i64(s_children, &["left"])
                .map(|v| v as f32)
                .or_else(|| get_f64(s_children, &["left"]).map(|v| v as f32))
            {
                settings.behavior.strut_left = v;
            }
            if let Some(v) = get_i64(s_children, &["right"])
                .map(|v| v as f32)
                .or_else(|| get_f64(s_children, &["right"]).map(|v| v as f32))
            {
                settings.behavior.strut_right = v;
            }
            if let Some(v) = get_i64(s_children, &["top"])
                .map(|v| v as f32)
                .or_else(|| get_f64(s_children, &["top"]).map(|v| v as f32))
            {
                settings.behavior.strut_top = v;
            }
            if let Some(v) = get_i64(s_children, &["bottom"])
                .map(|v| v as f32)
                .or_else(|| get_f64(s_children, &["bottom"]).map(|v| v as f32))
            {
                settings.behavior.strut_bottom = v;
            }
        }
    }

    // Center focused column
    if let Some(cfc) = get_string(layout_children, &["center-focused-column"]) {
        settings.behavior.center_focused_column = match cfc.as_str() {
            "always" => CenterFocusedColumn::Always,
            "on-overflow" => CenterFocusedColumn::OnOverflow,
            _ => CenterFocusedColumn::Never,
        };
    }

    // Always center single column
    if has_flag(layout_children, &["always-center-single-column"]) {
        settings.behavior.always_center_single_column = true;
    }

    // Empty workspace above first
    if has_flag(layout_children, &["empty-workspace-above-first"]) {
        settings.behavior.empty_workspace_above_first = true;
    }

    // Default column width. An empty `default-column-width {}` (or a bare node
    // with no proportion/fixed child) is niri's "windows pick their own width"
    // semantic and must be preserved as Auto rather than rewritten.
    if let Some(dcw) = layout_children.get("default-column-width") {
        match dcw.children() {
            Some(dcw_children) => {
                if let Some(prop) = get_f64(dcw_children, &["proportion"]) {
                    settings.behavior.default_column_width_type = ColumnWidthType::Proportion;
                    settings.behavior.default_column_width_proportion = prop as f32;
                } else if let Some(fixed) = get_f64(dcw_children, &["fixed"]) {
                    // `fixed` is FloatOrInt in niri, so accept a float such as
                    // `fixed 500.5` (get_f64 handles both int and float). Reading
                    // it with get_i64 only would drop the float and misclassify it
                    // as Auto, rewriting the user's fixed width to an empty block.
                    settings.behavior.default_column_width_type = ColumnWidthType::Fixed;
                    settings.behavior.default_column_width_fixed = fixed as f32;
                } else {
                    // `default-column-width {}` — empty block.
                    settings.behavior.default_column_width_type = ColumnWidthType::Auto;
                }
            }
            None => {
                // `default-column-width` with no block at all — treat as Auto.
                settings.behavior.default_column_width_type = ColumnWidthType::Auto;
            }
        }
    }

    // Background color
    if let Some(bg) = get_string(layout_children, &["background-color"]) {
        settings.appearance.background_color = parse_color(&bg);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_layout_children;
    use crate::config::models::{ColumnWidthType, Settings};
    use kdl::KdlDocument;
    use std::str::FromStr;

    fn parse(kdl: &str) -> Settings {
        let doc = KdlDocument::from_str(kdl).expect("KDL must parse");
        let layout = doc
            .get("layout")
            .and_then(|n| n.children())
            .expect("layout block");
        let mut settings = Settings::default();
        parse_layout_children(layout, &mut settings);
        settings
    }

    #[test]
    fn default_column_width_float_fixed_is_fixed_not_auto() {
        // `fixed 500.5` is a valid niri FloatOrInt; it must be read as Fixed rather
        // than dropped and misclassified as Auto (which would rewrite it on save).
        let s = parse("layout {\n    default-column-width {\n        fixed 500.5\n    }\n}");
        assert_eq!(s.behavior.default_column_width_type, ColumnWidthType::Fixed);
        assert!((s.behavior.default_column_width_fixed - 500.5).abs() < 1e-3);
    }

    #[test]
    fn default_column_width_integer_fixed_still_fixed() {
        let s = parse("layout {\n    default-column-width {\n        fixed 640\n    }\n}");
        assert_eq!(s.behavior.default_column_width_type, ColumnWidthType::Fixed);
        assert!((s.behavior.default_column_width_fixed - 640.0).abs() < 1e-3);
    }

    #[test]
    fn default_column_width_empty_is_auto() {
        let s = parse("layout {\n    default-column-width {\n    }\n}");
        assert_eq!(s.behavior.default_column_width_type, ColumnWidthType::Auto);
    }
}
