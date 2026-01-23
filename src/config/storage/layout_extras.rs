//! Layout extras KDL generation
//!
//! Generates KDL for shadow, tab indicator, insert hint, preset heights.

use super::builder::KdlBuilder;
use crate::config::models::{
    DefaultColumnDisplay, LayoutExtrasSettings, PresetHeight, TabIndicatorPosition,
};
use crate::types::ColorOrGradient;
use std::fmt::Write;

/// Generate layout-extras.kdl content from settings.
///
/// Creates KDL configuration for layout extras including:
/// - Shadow settings (softness, spread, offset, colors)
/// - Tab indicator settings (position, width, gap, colors)
/// - Insert hint settings (color)
pub fn generate_layout_extras_kdl(settings: &LayoutExtrasSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Layout extras settings - managed by Nirify");

    kdl.block("layout", |b| {
        // Shadow settings
        if settings.shadow.enabled {
            b.block("shadow", |s| {
                s.field_i32("softness", settings.shadow.softness);
                s.field_i32("spread", settings.shadow.spread);
                s.raw(&format!(
                    "offset x={} y={}",
                    settings.shadow.offset_x, settings.shadow.offset_y
                ));
                s.field_color("color", &settings.shadow.color);
                s.field_color("inactive-color", &settings.shadow.inactive_color);
                s.optional_flag("draw-behind-window", settings.shadow.draw_behind_window);
            });
        } else {
            b.block("shadow", |s| {
                s.flag("off");
            });
        }

        // Tab indicator settings
        b.newline();
        if settings.tab_indicator.enabled {
            b.block("tab-indicator", |t| {
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
                t.field_color_or_gradient("active", &settings.tab_indicator.active);
                t.field_color_or_gradient("inactive", &settings.tab_indicator.inactive);
                t.field_color_or_gradient("urgent", &settings.tab_indicator.urgent);
                t.optional_flag(
                    "hide-when-single-tab",
                    settings.tab_indicator.hide_when_single_tab,
                );
                t.optional_flag(
                    "place-within-column",
                    settings.tab_indicator.place_within_column,
                );
            });
        } else {
            b.block("tab-indicator", |t| {
                t.flag("off");
            });
        }

        // Insert hint settings
        if settings.insert_hint.enabled {
            b.newline();
            // Insert hint can be color or gradient
            let color_kdl = match &settings.insert_hint.color {
                ColorOrGradient::Color(c) => format!("color \"{}\"", c.to_hex()),
                ColorOrGradient::Gradient(g) => {
                    let mut output = String::with_capacity(64);
                    // SAFETY: write! to String is infallible
                    let _ = write!(
                        output,
                        "gradient from=\"{}\" to=\"{}\"",
                        g.from.to_hex(),
                        g.to.to_hex()
                    );
                    if g.angle != 180 {
                        let _ = write!(output, " angle={}", g.angle);
                    }
                    output
                }
            };
            // Semicolon required for inline KDL node
            b.raw(&format!("insert-hint {{ {}; }}", color_kdl));
        }
        // When insert-hint is disabled, don't output the block (niri's default)

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
