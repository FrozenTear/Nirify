//! Appearance KDL generation
//!
//! Generates KDL configuration for visual appearance settings.

use super::builder::KdlBuilder;
use crate::config::models::{AppearanceSettings, BehaviorSettings, ColumnWidthType};

/// Generate appearance.kdl content from settings.
///
/// Creates KDL configuration for visual appearance settings including:
/// - Window gaps (inner and outer)
/// - Focus ring (width, colors, enabled state)
/// - Window borders (width, colors, enabled state)
/// - Screen edge struts
/// - Column centering behavior
/// - Default window width
/// - Corner radius (via window rules)
///
/// # Arguments
/// * `settings` - The appearance settings to convert
/// * `behavior` - Behavior settings (contains struts and column width)
///
/// # Returns
/// A string containing valid KDL configuration for niri.
pub fn generate_appearance_kdl(
    settings: &AppearanceSettings,
    behavior: &BehaviorSettings,
) -> String {
    let mut kdl = KdlBuilder::with_header("Appearance settings - managed by niri-settings-rust");

    kdl.block("layout", |b| {
        // Gaps - niri uses a single value
        b.raw(&format!("gaps {}", settings.gaps_inner.round() as i32));

        // Focus ring
        if settings.focus_ring_enabled {
            b.newline();
            b.block("focus-ring", |fr| {
                fr.field_f32_as_int("width", settings.focus_ring_width);
                fr.field_color_or_gradient("active", &settings.focus_ring_active);
                fr.field_color_or_gradient("inactive", &settings.focus_ring_inactive);
                fr.field_color_or_gradient("urgent", &settings.focus_ring_urgent);
            });
        }

        // Border
        if settings.border_enabled {
            b.newline();
            b.block("border", |br| {
                br.field_f32_as_int("width", settings.border_thickness);
                br.field_color_or_gradient("active", &settings.border_active);
                br.field_color_or_gradient("inactive", &settings.border_inactive);
                br.field_color_or_gradient("urgent", &settings.border_urgent);
            });
        }

        // Background color
        if let Some(ref bg) = settings.background_color {
            b.newline();
            b.field_color("background-color", bg);
        }

        // Struts (from behavior settings)
        let has_struts = behavior.strut_left > 0.0
            || behavior.strut_right > 0.0
            || behavior.strut_top > 0.0
            || behavior.strut_bottom > 0.0;
        if has_struts {
            b.newline();
            b.block("struts", |s| {
                s.field_f32_as_int("left", behavior.strut_left);
                s.field_f32_as_int("right", behavior.strut_right);
                s.field_f32_as_int("top", behavior.strut_top);
                s.field_f32_as_int("bottom", behavior.strut_bottom);
            });
        }

        // Center focused column
        match behavior.center_focused_column {
            crate::types::CenterFocusedColumn::Never => {}
            crate::types::CenterFocusedColumn::Always => {
                b.newline();
                b.field_string("center-focused-column", "always");
            }
            crate::types::CenterFocusedColumn::OnOverflow => {
                b.newline();
                b.field_string("center-focused-column", "on-overflow");
            }
        }

        // Always center single column
        b.optional_flag(
            "always-center-single-column",
            behavior.always_center_single_column,
        );

        // Empty workspace above first
        b.optional_flag(
            "empty-workspace-above-first",
            behavior.empty_workspace_above_first,
        );

        // Default column width
        b.newline();
        match behavior.default_column_width_type {
            ColumnWidthType::Proportion => {
                b.raw(&format!(
                    "default-column-width {{ proportion {:.2}; }}",
                    behavior.default_column_width_proportion
                ));
            }
            ColumnWidthType::Fixed => {
                b.raw(&format!(
                    "default-column-width {{ fixed {}; }}",
                    behavior.default_column_width_fixed.round() as i32
                ));
            }
        }
    });

    // Window corner radius
    if settings.corner_radius > 0.0 {
        kdl.newline();
        kdl.block("window-rule", |wr| {
            wr.field_f32_as_int("geometry-corner-radius", settings.corner_radius);
        });
    }

    kdl.build()
}
