//! Appearance KDL generation
//!
//! Generates KDL configuration for visual appearance settings.

use super::builder::KdlBuilder;
use crate::config::models::{
    AppearanceSettings, BehaviorSettings, ColumnWidthType, WindowRulesSettings,
};

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
///
/// A radius-only catch-all is emitted only when no managed window-rule
/// already acts as a catch-all (see [`generate_appearance_kdl_for_settings`]).
///
/// # Omit-default policy
///
/// Keywords Nirify owns are **always emitted**, including niri defaults
/// (`center-focused-column "never"`). We do **not** wait for a
/// `ConflictingInclude`: if this category file exists, Nirify claims the
/// property and must last-wins against earlier user includes. `scale` stays
/// `Option<f64>` (`None` = omit / niri auto) and is not affected here.
pub fn generate_appearance_kdl(
    settings: &AppearanceSettings,
    behavior: &BehaviorSettings,
) -> String {
    generate_appearance_kdl_ex(settings, behavior, true)
}

/// Generate appearance.kdl, skipping the radius-only catch-all when
/// `window_rules` already contains a no-match rule.
pub fn generate_appearance_kdl_for_settings(
    settings: &AppearanceSettings,
    behavior: &BehaviorSettings,
    window_rules: &WindowRulesSettings,
) -> String {
    generate_appearance_kdl_ex(settings, behavior, !window_rules.has_catch_all())
}

fn generate_appearance_kdl_ex(
    settings: &AppearanceSettings,
    behavior: &BehaviorSettings,
    emit_corner_radius_rule: bool,
) -> String {
    let mut kdl = KdlBuilder::with_header("Appearance settings - managed by Nirify");

    kdl.block("layout", |b| {
        // Gaps - single value (niri only supports one gaps value)
        let gaps = settings.gaps.round() as i32;
        b.raw(&format!("gaps {}", gaps));

        // Focus ring - always emitted with an explicit on/off flag so the
        // enabled state (and styling) round-trips and overrides the user's
        // main config via niri's merge_on_off.
        b.newline();
        b.block("focus-ring", |fr| {
            fr.flag(if settings.focus_ring_enabled {
                "on"
            } else {
                "off"
            });
            fr.field_f32_as_int("width", settings.focus_ring_width);
            fr.field_color_or_gradient("active", &settings.focus_ring_active);
            fr.field_color_or_gradient("inactive", &settings.focus_ring_inactive);
            fr.field_color_or_gradient("urgent", &settings.focus_ring_urgent);
        });

        // Border - same BorderRule grammar, always emitted with explicit on/off.
        b.newline();
        b.block("border", |br| {
            br.flag(if settings.border_enabled { "on" } else { "off" });
            br.field_f32_as_int("width", settings.border_thickness);
            br.field_color_or_gradient("active", &settings.border_active);
            br.field_color_or_gradient("inactive", &settings.border_inactive);
            br.field_color_or_gradient("urgent", &settings.border_urgent);
        });

        // Background color
        if let Some(ref bg) = settings.background_color {
            b.newline();
            b.field_color("background-color", bg);
        }

        // Struts (from behavior settings)
        let has_struts = behavior.strut_left != 0.0
            || behavior.strut_right != 0.0
            || behavior.strut_top != 0.0
            || behavior.strut_bottom != 0.0;
        if has_struts {
            b.newline();
            b.block("struts", |s| {
                s.field_f32_as_int("left", behavior.strut_left);
                s.field_f32_as_int("right", behavior.strut_right);
                s.field_f32_as_int("top", behavior.strut_top);
                s.field_f32_as_int("bottom", behavior.strut_bottom);
            });
        }

        // Always emit center-focused-column, including niri's default "never".
        // Nirify's include is last, but omitted properties do not override an
        // earlier user include (niri merges only written keys). See the
        // omit-default policy on [`generate_appearance_kdl`].
        b.newline();
        b.field_string(
            "center-focused-column",
            behavior.center_focused_column.to_kdl(),
        );

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
            // Empty block: niri lets each new window choose its own initial width.
            ColumnWidthType::Auto => {
                b.raw("default-column-width {}");
            }
        }
    });

    // Window corner radius — only when no managed catch-all already carries
    // (or will carry) that radius. Dual-emitting a radius-only catch-all from
    // appearance.kdl would overwrite imported global opacity/clip/open-*.
    if emit_corner_radius_rule && settings.corner_radius > 0.0 {
        kdl.newline();
        kdl.block("window-rule", |wr| {
            wr.field_f32_as_int("geometry-corner-radius", settings.corner_radius);
        });
    }

    kdl.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::parse_layout_children;
    use crate::config::models::Settings;
    use crate::types::{Color, ColorOrGradient};
    use kdl::KdlDocument;
    use std::str::FromStr;

    fn reparse_layout(kdl: &str) -> Settings {
        let doc = KdlDocument::from_str(kdl).expect("generated KDL must parse");
        let layout = doc
            .get("layout")
            .and_then(|n| n.children())
            .expect("layout block");
        let mut settings = Settings::default();
        parse_layout_children(layout, &mut settings);
        settings
    }

    #[test]
    fn focus_ring_off_roundtrip() {
        let mut src = Settings::default();
        src.appearance.focus_ring_enabled = false;
        src.appearance.focus_ring_width = 6.0;
        src.appearance.focus_ring_active =
            ColorOrGradient::Color(Color::from_hex("#112233").unwrap());

        let kdl = generate_appearance_kdl(&src.appearance, &src.behavior);
        assert!(kdl.contains("focus-ring {"));
        assert!(kdl.contains("off"));

        let dst = reparse_layout(&kdl);
        assert!(!dst.appearance.focus_ring_enabled);
        assert_eq!(dst.appearance.focus_ring_width, 6.0);
        assert_eq!(
            dst.appearance.focus_ring_active,
            ColorOrGradient::Color(Color::from_hex("#112233").unwrap())
        );
    }

    #[test]
    fn border_off_block_emitted() {
        let mut src = Settings::default();
        src.appearance.border_enabled = false;
        src.appearance.border_thickness = 3.0;
        src.appearance.border_active = ColorOrGradient::Color(Color::from_hex("#445566").unwrap());

        let kdl = generate_appearance_kdl(&src.appearance, &src.behavior);
        assert!(kdl.contains("border {"));
        assert!(kdl.contains("off"));

        let dst = reparse_layout(&kdl);
        assert!(!dst.appearance.border_enabled);
        assert_eq!(dst.appearance.border_thickness, 3.0);
        assert_eq!(
            dst.appearance.border_active,
            ColorOrGradient::Color(Color::from_hex("#445566").unwrap())
        );
    }

    #[test]
    fn negative_struts_roundtrip() {
        let mut src = Settings::default();
        src.behavior.strut_left = -64.0;
        src.behavior.strut_top = 12.0;

        let kdl = generate_appearance_kdl(&src.appearance, &src.behavior);
        assert!(kdl.contains("struts {"));
        assert!(kdl.contains("left -64"));
        assert!(kdl.contains("top 12"));

        let dst = reparse_layout(&kdl);
        assert_eq!(dst.behavior.strut_left, -64.0);
        assert_eq!(dst.behavior.strut_top, 12.0);

        // Absent when all four struts are zero (niri default).
        let zero = Settings::default();
        let kdl0 = generate_appearance_kdl(&zero.appearance, &zero.behavior);
        assert!(!kdl0.contains("struts {"));
    }

    #[test]
    fn default_column_width_auto_roundtrip() {
        let mut src = Settings::default();
        src.behavior.default_column_width_type = ColumnWidthType::Auto;

        let kdl = generate_appearance_kdl(&src.appearance, &src.behavior);
        assert!(kdl.contains("default-column-width {}"));
        assert!(!kdl.contains("proportion"));
        assert!(KdlDocument::from_str(&kdl).is_ok());

        let dst = reparse_layout(&kdl);
        assert_eq!(
            dst.behavior.default_column_width_type,
            ColumnWidthType::Auto
        );
    }

    #[test]
    fn always_emits_center_focused_column_never() {
        let defaults = Settings::default();
        let kdl = generate_appearance_kdl(&defaults.appearance, &defaults.behavior);
        assert!(
            kdl.contains("center-focused-column \"never\""),
            "owned default must last-wins against earlier includes:\n{kdl}"
        );
        let dst = reparse_layout(&kdl);
        assert_eq!(
            dst.behavior.center_focused_column,
            crate::types::CenterFocusedColumn::Never
        );
    }

    #[test]
    fn appearance_kdl_reparses() {
        let defaults = Settings::default();
        assert!(KdlDocument::from_str(&generate_appearance_kdl(
            &defaults.appearance,
            &defaults.behavior
        ))
        .is_ok());

        let mut variant = Settings::default();
        variant.appearance.focus_ring_enabled = false;
        variant.appearance.border_enabled = false;
        variant.behavior.strut_left = -64.0;
        variant.behavior.strut_bottom = -10.0;
        assert!(KdlDocument::from_str(&generate_appearance_kdl(
            &variant.appearance,
            &variant.behavior
        ))
        .is_ok());
    }

    #[test]
    fn appearance_skips_radius_rule_when_catch_all_exists() {
        use crate::config::models::{CornerRadiusValue, WindowRule, WindowRulesSettings};

        let appearance = AppearanceSettings {
            corner_radius: 12.0,
            ..Default::default()
        };
        let behavior = BehaviorSettings::default();

        let emitted = generate_appearance_kdl(&appearance, &behavior);
        assert!(emitted.contains("window-rule {"));
        assert!(emitted.contains("geometry-corner-radius 12"));

        let window_rules = WindowRulesSettings {
            rules: vec![WindowRule {
                opacity: Some(0.9),
                clip_to_geometry: Some(true),
                corner_radius: Some(CornerRadiusValue::uniform(12.0)),
                ..Default::default()
            }],
            next_id: 1,
        };
        let skipped = generate_appearance_kdl_for_settings(&appearance, &behavior, &window_rules);
        assert!(
            !skipped.contains("window-rule"),
            "must not dual-emit a radius-only catch-all:\n{skipped}"
        );
    }
}
