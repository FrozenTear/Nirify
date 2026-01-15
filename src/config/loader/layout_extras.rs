//! Layout extras settings loader
//!
//! Handles shadow, tab indicator, insert hint, preset heights, and default column display.

use super::gradient::{load_color_or_gradient, parse_gradient_from_entries};
use super::helpers::{load_color, parse_color, read_kdl_file};
use crate::config::models::{DefaultColumnDisplay, PresetHeight, Settings, TabIndicatorPosition};
use crate::config::parser::{get_i64, get_string, has_flag};
use crate::types::ColorOrGradient;
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Parse layout extras from layout node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_layout_extras_from_children(l_children: &KdlDocument, settings: &mut Settings) {
    // Shadow
    if let Some(shadow) = l_children.get("shadow") {
        if let Some(s_children) = shadow.children() {
            if has_flag(s_children, &["off"]) {
                settings.layout_extras.shadow.enabled = false;
            } else {
                settings.layout_extras.shadow.enabled = true;

                if let Some(v) = get_i64(s_children, &["softness"]) {
                    settings.layout_extras.shadow.softness = v as i32;
                }
                if let Some(v) = get_i64(s_children, &["spread"]) {
                    settings.layout_extras.shadow.spread = v as i32;
                }

                // Offset
                if let Some(offset) = s_children.get("offset") {
                    for entry in offset.entries() {
                        if let Some(name) = entry.name() {
                            if let Some(val) = entry.value().as_integer() {
                                match name.value() {
                                    "x" => settings.layout_extras.shadow.offset_x = val as i32,
                                    "y" => settings.layout_extras.shadow.offset_y = val as i32,
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                load_color(
                    s_children,
                    &["color"],
                    &mut settings.layout_extras.shadow.color,
                );
                load_color(
                    s_children,
                    &["inactive-color"],
                    &mut settings.layout_extras.shadow.inactive_color,
                );
                if has_flag(s_children, &["draw-behind-window"]) {
                    settings.layout_extras.shadow.draw_behind_window = true;
                }
            }
        }
    }

    // Tab indicator
    if let Some(ti) = l_children.get("tab-indicator") {
        if let Some(ti_children) = ti.children() {
            if has_flag(ti_children, &["off"]) {
                settings.layout_extras.tab_indicator.enabled = false;
            } else {
                settings.layout_extras.tab_indicator.enabled = true;

                if let Some(v) = get_string(ti_children, &["position"]) {
                    settings.layout_extras.tab_indicator.position = match v.as_str() {
                        "right" => TabIndicatorPosition::Right,
                        "top" => TabIndicatorPosition::Top,
                        "bottom" => TabIndicatorPosition::Bottom,
                        _ => TabIndicatorPosition::Left,
                    };
                }
                if let Some(v) = get_i64(ti_children, &["width"]) {
                    settings.layout_extras.tab_indicator.width = v as i32;
                }
                if let Some(v) = get_i64(ti_children, &["gap"]) {
                    settings.layout_extras.tab_indicator.gap = v as i32;
                }
                if let Some(v) = get_i64(ti_children, &["gaps-between-tabs"]) {
                    settings.layout_extras.tab_indicator.gaps_between_tabs = v as i32;
                }
                if let Some(v) = get_i64(ti_children, &["corner-radius"]) {
                    settings.layout_extras.tab_indicator.corner_radius = v as i32;
                }
                // Load color or gradient for active/inactive/urgent
                if let Some(cog) = load_color_or_gradient(ti_children, "active") {
                    settings.layout_extras.tab_indicator.active = cog;
                }
                if let Some(cog) = load_color_or_gradient(ti_children, "inactive") {
                    settings.layout_extras.tab_indicator.inactive = cog;
                }
                if let Some(cog) = load_color_or_gradient(ti_children, "urgent") {
                    settings.layout_extras.tab_indicator.urgent = cog;
                }
                if has_flag(ti_children, &["hide-when-single-tab"]) {
                    settings.layout_extras.tab_indicator.hide_when_single_tab = true;
                }
                if has_flag(ti_children, &["place-within-column"]) {
                    settings.layout_extras.tab_indicator.place_within_column = true;
                }
            }
        }
    }

    // Insert hint (supports gradient in niri)
    if let Some(ih) = l_children.get("insert-hint") {
        if let Some(ih_children) = ih.children() {
            if has_flag(ih_children, &["off"]) {
                settings.layout_extras.insert_hint.enabled = false;
            } else {
                // Try gradient first, then fall back to color
                if let Some(gradient_node) = ih_children.get("gradient") {
                    if let Some(gradient) =
                        parse_gradient_from_entries(gradient_node.entries().iter())
                    {
                        settings.layout_extras.insert_hint.color =
                            ColorOrGradient::Gradient(gradient);
                    }
                } else if let Some(hex) = get_string(ih_children, &["color"]) {
                    if let Some(color) = parse_color(&hex) {
                        settings.layout_extras.insert_hint.color = ColorOrGradient::Color(color);
                    }
                }
            }
        }
    }

    // Preset window heights
    if let Some(pwh) = l_children.get("preset-window-heights") {
        if let Some(pwh_children) = pwh.children() {
            settings.layout_extras.preset_window_heights.clear();
            for node in pwh_children.nodes() {
                let name = node.name().value();
                if name == "proportion" {
                    if let Some(entry) = node.entries().first() {
                        if let Some(v) = entry.value().as_float() {
                            settings
                                .layout_extras
                                .preset_window_heights
                                .push(PresetHeight::Proportion(v as f32));
                        }
                    }
                } else if name == "fixed" {
                    if let Some(entry) = node.entries().first() {
                        if let Some(v) = entry.value().as_integer() {
                            settings
                                .layout_extras
                                .preset_window_heights
                                .push(PresetHeight::Fixed(v as i32));
                        }
                    }
                }
            }
        }
    }

    // Default column display mode
    if let Some(v) = get_string(l_children, &["default-column-display"]) {
        settings.layout_extras.default_column_display = match v.as_str() {
            "tabbed" => DefaultColumnDisplay::Tabbed,
            _ => DefaultColumnDisplay::Normal,
        };
    }
}

/// Load layout extras settings from KDL file
pub fn load_layout_extras(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    if let Some(layout) = doc.get("layout") {
        if let Some(l_children) = layout.children() {
            parse_layout_extras_from_children(l_children, settings);
        }
    }

    debug!("Loaded layout extras settings from {:?}", path);
}
