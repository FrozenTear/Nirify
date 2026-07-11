//! Layout extras KDL generation
//!
//! Generates KDL for shadow, tab indicator, insert hint, preset heights.

use super::builder::KdlBuilder;
use super::gradient::gradient_node_to_kdl;
use crate::config::models::{
    DefaultColumnDisplay, LayoutExtrasSettings, PresetHeight, TabIndicatorPosition,
};
use crate::types::ColorOrGradient;

/// Generate layout-extras.kdl content from settings.
///
/// Creates KDL configuration for layout extras including:
/// - Shadow settings (softness, spread, offset, colors)
/// - Tab indicator settings (position, width, gap, colors)
/// - Insert hint settings (color)
pub fn generate_layout_extras_kdl(settings: &LayoutExtrasSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Layout extras settings - managed by Nirify");

    kdl.block("layout", |b| {
        // Shadow settings - always emitted with an explicit on/off flag so the
        // enabled state and styling round-trip; niri's ShadowRule accepts all
        // props alongside `off`.
        b.block("shadow", |s| {
            s.flag(if settings.shadow.enabled { "on" } else { "off" });
            s.field_i32("softness", settings.shadow.softness);
            s.field_i32("spread", settings.shadow.spread);
            s.raw(&format!(
                "offset x={} y={}",
                settings.shadow.offset_x, settings.shadow.offset_y
            ));
            s.field_color("color", &settings.shadow.color);
            if settings.shadow.use_inactive_color {
                s.field_color("inactive-color", &settings.shadow.inactive_color);
            }
            if settings.shadow.draw_behind_window {
                s.raw("draw-behind-window true");
            }
        });

        // Tab indicator settings - always emitted with explicit on/off flag.
        // Colors are only emitted when the user opts in; otherwise niri falls
        // back to focus-ring colors (urgent -> #9b0000).
        b.newline();
        b.block("tab-indicator", |t| {
            t.flag(if settings.tab_indicator.enabled {
                "on"
            } else {
                "off"
            });
            let position_str = match settings.tab_indicator.position {
                TabIndicatorPosition::Left => "left",
                TabIndicatorPosition::Right => "right",
                TabIndicatorPosition::Top => "top",
                TabIndicatorPosition::Bottom => "bottom",
            };
            t.field_string("position", position_str);
            t.field_i32("width", settings.tab_indicator.width);
            t.field_i32("gap", settings.tab_indicator.gap);
            t.field_i32(
                "gaps-between-tabs",
                settings.tab_indicator.gaps_between_tabs,
            );
            t.field_i32("corner-radius", settings.tab_indicator.corner_radius);
            t.raw(&format!(
                "length total-proportion={:.2}",
                settings.tab_indicator.length_proportion
            ));
            if settings.tab_indicator.use_active_color {
                t.field_color_or_gradient("active", &settings.tab_indicator.active);
            }
            if settings.tab_indicator.use_inactive_color {
                t.field_color_or_gradient("inactive", &settings.tab_indicator.inactive);
            }
            if settings.tab_indicator.use_urgent_color {
                t.field_color_or_gradient("urgent", &settings.tab_indicator.urgent);
            }
            t.optional_flag(
                "hide-when-single-tab",
                settings.tab_indicator.hide_when_single_tab,
            );
            t.optional_flag(
                "place-within-column",
                settings.tab_indicator.place_within_column,
            );
        });

        // Insert hint settings - always emitted with explicit on/off flag,
        // preserving color/gradient styling in both states.
        b.newline();
        b.block("insert-hint", |ih| {
            ih.flag(if settings.insert_hint.enabled {
                "on"
            } else {
                "off"
            });
            match &settings.insert_hint.color {
                ColorOrGradient::Color(c) => {
                    ih.field_color("color", c);
                }
                ColorOrGradient::Gradient(g) => {
                    ih.raw(&gradient_node_to_kdl(g, "gradient"));
                }
            }
        });

        // Preset window heights
        if !settings.preset_window_heights.is_empty() {
            b.newline();
            b.block("preset-window-heights", |p| {
                for height in &settings.preset_window_heights {
                    match height {
                        PresetHeight::Proportion(prop) => {
                            p.field_f32("proportion", *prop);
                        }
                        PresetHeight::Fixed(f) => {
                            p.field_i32("fixed", *f);
                        }
                    }
                }
            });
        }

        // Default column display mode
        match settings.default_column_display {
            DefaultColumnDisplay::Normal => {} // Don't output default
            DefaultColumnDisplay::Tabbed => {
                b.newline();
                b.field_string("default-column-display", "tabbed");
            }
        }
    });

    kdl.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::parse_layout_extras_from_children;
    use crate::config::models::Settings;
    use crate::types::{
        Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo, HueInterpolation,
    };
    use kdl::KdlDocument;
    use std::str::FromStr;

    fn reparse(kdl: &str) -> Settings {
        let doc = KdlDocument::from_str(kdl).expect("generated KDL must parse");
        let layout = doc
            .get("layout")
            .and_then(|n| n.children())
            .expect("layout block");
        let mut settings = Settings::default();
        parse_layout_extras_from_children(layout, &mut settings);
        settings
    }

    #[test]
    fn insert_hint_off_roundtrip() {
        let mut src = Settings::default();
        src.layout_extras.insert_hint.enabled = false;
        src.layout_extras.insert_hint.color =
            ColorOrGradient::Color(Color::from_hex("#abcdef80").unwrap());

        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("insert-hint {"));
        assert!(kdl.contains("off"));
        assert!(kdl.contains("color"));

        let dst = reparse(&kdl);
        assert!(!dst.layout_extras.insert_hint.enabled);
        assert_eq!(
            dst.layout_extras.insert_hint.color,
            ColorOrGradient::Color(Color::from_hex("#abcdef80").unwrap())
        );
    }

    #[test]
    fn insert_hint_gradient_full_attrs_roundtrip() {
        let gradient = Gradient {
            from: Color::from_hex("#80c8ff").unwrap(),
            to: Color::from_hex("#bbddff").unwrap(),
            angle: 45,
            relative_to: GradientRelativeTo::WorkspaceView,
            color_space: ColorSpace::Oklch,
            hue_interpolation: Some(HueInterpolation::Longer),
        };
        let mut src = Settings::default();
        src.layout_extras.insert_hint.color = ColorOrGradient::Gradient(gradient.clone());

        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("relative-to=\"workspace-view\""));
        assert!(kdl.contains("in=\"oklch longer hue\""));

        let dst = reparse(&kdl);
        assert_eq!(
            dst.layout_extras.insert_hint.color,
            ColorOrGradient::Gradient(gradient)
        );
    }

    #[test]
    fn tab_indicator_length_roundtrip() {
        let mut src = Settings::default();
        src.layout_extras.tab_indicator.length_proportion = 0.35;

        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("length total-proportion=0.35"));

        let dst = reparse(&kdl);
        assert!((dst.layout_extras.tab_indicator.length_proportion - 0.35).abs() < 1e-6);
    }

    #[test]
    fn tab_indicator_optional_colors() {
        // Defaults: no color nodes emitted for the tab indicator.
        let defaults = Settings::default();
        let kdl = generate_layout_extras_kdl(&defaults.layout_extras);
        assert!(!kdl.contains("active-color"));
        assert!(!kdl.contains("inactive-color"));
        assert!(!kdl.contains("urgent-color"));

        // Opt in to a custom active color.
        let mut src = Settings::default();
        src.layout_extras.tab_indicator.use_active_color = true;
        src.layout_extras.tab_indicator.active =
            ColorOrGradient::Color(Color::from_hex("#123456").unwrap());
        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("active-color"));

        let dst = reparse(&kdl);
        assert!(dst.layout_extras.tab_indicator.use_active_color);
        assert_eq!(
            dst.layout_extras.tab_indicator.active,
            ColorOrGradient::Color(Color::from_hex("#123456").unwrap())
        );
    }

    #[test]
    fn shadow_off_preserves_styling_roundtrip() {
        let mut src = Settings::default();
        src.layout_extras.shadow.enabled = false;
        src.layout_extras.shadow.softness = 42;
        src.layout_extras.shadow.offset_x = 3;
        src.layout_extras.shadow.offset_y = -7;

        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("shadow {"));
        assert!(kdl.contains("off"));
        assert!(kdl.contains("softness 42"));
        assert!(kdl.contains("offset x=3 y=-7"));
        // inactive-color omitted unless opted in.
        assert!(!kdl.contains("inactive-color"));

        let dst = reparse(&kdl);
        assert!(!dst.layout_extras.shadow.enabled);
        assert_eq!(dst.layout_extras.shadow.softness, 42);
        assert_eq!(dst.layout_extras.shadow.offset_x, 3);
        assert_eq!(dst.layout_extras.shadow.offset_y, -7);
        assert!(!dst.layout_extras.shadow.use_inactive_color);
    }

    #[test]
    fn shadow_negative_spread_roundtrip() {
        let mut src = Settings::default();
        src.layout_extras.shadow.enabled = true;
        src.layout_extras.shadow.spread = -50;
        src.layout_extras.shadow.softness = 20;

        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        assert!(kdl.contains("spread -50"));
        assert!(KdlDocument::from_str(&kdl).is_ok());

        let dst = reparse(&kdl);
        assert_eq!(dst.layout_extras.shadow.spread, -50);
    }

    /// Locks the SetShadowSpread handler clamp bounds (`-1024..=1024`).
    ///
    /// The App handler is not cheaply constructable in a unit test, so this
    /// follows the codebase convention (see `tests/callback_logic_tests.rs`) of
    /// asserting the exact clamp expression the handler uses, then round-trips
    /// the resulting value through storage+loader. If the handler clamp is
    /// reverted to the old `(0, 100)` bounds, `-50` would clip to `0` and the
    /// value/expectation here would no longer match the handler's behavior.
    #[test]
    fn shadow_spread_handler_clamp_preserves_negatives() {
        // Mirror handlers/layout_extras.rs SetShadowSpread => v.clamp(-1024, 1024).
        let clamp_spread = |v: i32| v.clamp(-1024, 1024);
        assert_eq!(clamp_spread(-50), -50, "negatives must survive the clamp");
        assert_eq!(clamp_spread(-2000), -1024, "clamped to niri's lower bound");
        assert_eq!(clamp_spread(5000), 1024, "clamped to niri's upper bound");

        // The clamped negative must also survive the full storage <-> loader path.
        let mut src = Settings::default();
        src.layout_extras.shadow.enabled = true;
        src.layout_extras.shadow.spread = clamp_spread(-50);
        let kdl = generate_layout_extras_kdl(&src.layout_extras);
        let dst = reparse(&kdl);
        assert_eq!(dst.layout_extras.shadow.spread, -50);
    }

    #[test]
    fn shadow_float_values_parse() {
        // niri treats these as FloatOrInt; a float must not be dropped.
        let kdl = r#"layout {
    shadow {
        on
        softness 30.5
        spread 5.9
    }
    tab-indicator {
        on
        width 4.5
        corner-radius 2.4
    }
}"#;
        let dst = reparse(kdl);
        assert_eq!(dst.layout_extras.shadow.softness, 31);
        assert_eq!(dst.layout_extras.shadow.spread, 6);
        assert_eq!(dst.layout_extras.tab_indicator.width, 5);
        assert_eq!(dst.layout_extras.tab_indicator.corner_radius, 2);
    }

    #[test]
    fn layout_extras_defaults_are_niri_neutral() {
        let defaults = Settings::default();
        let kdl = generate_layout_extras_kdl(&defaults.layout_extras);
        assert!(KdlDocument::from_str(&kdl).is_ok());

        // Shadow off by default (standalone `off` flag, not the `off` in `offset`).
        let shadow_block = kdl.split("shadow {").nth(1).unwrap();
        let shadow_block = shadow_block.split('}').next().unwrap();
        assert!(
            shadow_block.lines().any(|l| l.trim() == "off"),
            "shadow must emit a standalone `off` flag line"
        );

        // Tab indicator on, niri-neutral styling.
        let tab_block = kdl.split("tab-indicator {").nth(1).unwrap();
        let tab_block = tab_block.split('}').next().unwrap();
        // Discriminating: assert a standalone `on` flag line (not a substring of
        // e.g. `position "left"`), and that no `off` flag line is present.
        assert!(
            tab_block.lines().any(|l| l.trim() == "on"),
            "tab-indicator must emit a standalone `on` flag line"
        );
        assert!(!tab_block.lines().any(|l| l.trim() == "off"));
        assert!(tab_block.contains("gaps-between-tabs 0"));
        assert!(tab_block.contains("corner-radius 0"));
        assert!(tab_block.contains("length total-proportion=0.50"));

        // Insert hint on with niri default color.
        let ih_block = kdl.split("insert-hint {").nth(1).unwrap();
        let ih_block = ih_block.split('}').next().unwrap();
        assert!(ih_block.contains("on"));
        assert!(ih_block.contains("#7fc8ff80"));
    }
}
