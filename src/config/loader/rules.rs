//! Window and layer rules loader
//!
//! Handles window rules and layer rules parsing.
//!
//! Uses generic `load_rules` helper to eliminate boilerplate between
//! window rules and layer rules loaders.

use super::helpers::{parse_color, read_kdl_file};
use crate::config::models::{
    BlockOutFrom, FloatingPosition, LayerRule, LayerRuleMatch, OpenBehavior, PositionRelativeTo,
    Settings, ShadowSettings, TabIndicatorSettings, WindowRule, WindowRuleMatch,
};
use crate::config::parser::{get_f64, get_i64, get_string, has_flag};
use crate::config::validation::validate_regex_pattern;
use crate::types::{Color, ColorOrGradient};
use kdl::{KdlDocument, KdlNode};
use log::{debug, warn};
use std::path::Path;

/// Safely convert i64 to i32 with bounds checking
///
/// Returns None and logs a warning if the value is out of i32 range.
fn safe_i64_to_i32(value: i64, context: &str) -> Option<i32> {
    if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
        Some(value as i32)
    } else {
        warn!("Value {} out of i32 range for {}, ignoring", value, context);
        None
    }
}

/// Safely convert f64 opacity to f32 with range validation
///
/// Clamps to valid opacity range (0.0-1.0) and warns if out of range.
fn safe_opacity_to_f32(value: f64, context: &str) -> f32 {
    if !(0.0..=1.0).contains(&value) {
        warn!(
            "Opacity {} out of range (0.0-1.0) for {}, clamping",
            value, context
        );
    }
    (value.clamp(0.0, 1.0)) as f32
}

/// Extract rule name from leading comment
///
/// Looks for a comment like "// Rule Name\n" before the node.
/// Returns None if no valid name comment found.
fn extract_name_from_leading_comment(node: &KdlNode) -> Option<String> {
    let format = node.format()?;
    let leading = &format.leading;
    // Look for "// " pattern in the leading content
    if let Some(start) = leading.rfind("// ") {
        let after_comment = &leading[start + 3..];
        // Find end of line or end of string
        let name = if let Some(newline) = after_comment.find('\n') {
            after_comment[..newline].trim()
        } else {
            after_comment.trim()
        };
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Check if a node has a flag as entry (e.g., `shadow { on }`)
pub fn has_flag_in_node(node: &kdl::KdlNode, flag: &str) -> bool {
    for entry in node.entries() {
        if entry.name().is_none() {
            if let Some(s) = entry.value().as_string() {
                if s == flag {
                    return true;
                }
            }
        }
    }
    false
}

/// Trait for rule types that have an ID and name.
///
/// Both LayerRule and WindowRule implement this trait to enable
/// generic rule loading.
trait RuleWithId {
    fn set_id(&mut self, id: u32);
    fn set_name(&mut self, name: String);
}

impl RuleWithId for LayerRule {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl RuleWithId for WindowRule {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// Generic helper for loading rules from a KDL file.
///
/// Handles the common pattern shared by layer rules and window rules:
/// 1. Reading the KDL file
/// 2. Iterating over nodes matching `rule_node_name`
/// 3. Creating rule instances with sequential IDs
/// 4. Calling the rule-specific parser
/// 5. Collecting into the rules vector
///
/// Returns the loaded rules and the next available ID.
fn load_rules<R, F>(
    path: &Path,
    rule_node_name: &str,
    name_prefix: &str,
    parser: F,
) -> (Vec<R>, u32)
where
    R: Default + RuleWithId,
    F: Fn(&KdlDocument, &mut R),
{
    let Some(doc) = read_kdl_file(path) else {
        return (Vec::new(), 0);
    };

    let mut rules = Vec::new();
    let mut next_id = 0u32;

    for node in doc.nodes() {
        if node.name().value() == rule_node_name {
            let mut rule = R::default();
            rule.set_id(next_id);

            // Try to extract name from leading comment (format: "// Rule Name\n")
            let name = extract_name_from_leading_comment(node)
                .unwrap_or_else(|| format!("{} {}", name_prefix, next_id + 1));
            rule.set_name(name);

            if let Some(children) = node.children() {
                parser(children, &mut rule);
            }

            rules.push(rule);
            next_id += 1;
        }
    }

    debug!(
        "Loaded {} {} from {:?}",
        rules.len(),
        rule_node_name.replace('-', " ") + "s",
        path
    );

    (rules, next_id)
}

// ============================================================================
// LAYER RULES
// ============================================================================

/// Parse layer rule node children into a LayerRule
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_layer_rule_node_children(children: &KdlDocument, rule: &mut LayerRule) {
    // Parse matches
    rule.matches.clear();
    for match_node in children.nodes() {
        if match_node.name().value() == "match" {
            let mut m = LayerRuleMatch::default();

            // Check entries for named arguments
            for entry in match_node.entries() {
                if let Some(name) = entry.name() {
                    match name.value() {
                        "namespace" => {
                            if let Some(s) = entry.value().as_string() {
                                m.namespace = validate_regex_pattern(s, "layer rule namespace");
                            }
                        }
                        "at-startup" => {
                            if let Some(b) = entry.value().as_bool() {
                                m.at_startup = Some(b);
                            }
                        }
                        _ => {}
                    }
                }
            }

            rule.matches.push(m);
        }
    }

    // Ensure at least one match exists
    if rule.matches.is_empty() {
        rule.matches.push(LayerRuleMatch::default());
    }

    // block-out-from
    if let Some(bof) = get_string(children, &["block-out-from"]) {
        rule.block_out_from = match bof.as_str() {
            "screencast" => Some(BlockOutFrom::Screencast),
            "screen-capture" => Some(BlockOutFrom::ScreenCapture),
            _ => None,
        };
    }

    // opacity
    if let Some(v) = get_f64(children, &["opacity"]) {
        rule.opacity = Some(safe_opacity_to_f32(v, "layer rule opacity"));
    }

    // geometry-corner-radius
    if let Some(v) = get_i64(children, &["geometry-corner-radius"]) {
        rule.geometry_corner_radius = safe_i64_to_i32(v, "geometry-corner-radius");
    }

    // place-within-backdrop
    if has_flag(children, &["place-within-backdrop"]) {
        rule.place_within_backdrop = true;
    }

    // baba-is-float
    if has_flag(children, &["baba-is-float"]) {
        rule.baba_is_float = true;
    }

    // shadow (complex nested block)
    if let Some(shadow_node) = children.get("shadow") {
        if has_flag_in_node(shadow_node, "on") {
            let mut shadow = ShadowSettings {
                enabled: true,
                ..Default::default()
            };

            if let Some(shadow_children) = shadow_node.children() {
                if let Some(v) = get_i64(shadow_children, &["softness"]) {
                    if let Some(safe_v) = safe_i64_to_i32(v, "shadow softness") {
                        shadow.softness = safe_v;
                    }
                }
                if let Some(v) = get_i64(shadow_children, &["spread"]) {
                    if let Some(safe_v) = safe_i64_to_i32(v, "shadow spread") {
                        shadow.spread = safe_v;
                    }
                }
                // Parse offset x=... y=... as named entries
                if let Some(offset_node) = shadow_children.get("offset") {
                    for entry in offset_node.entries() {
                        if let Some(name) = entry.name() {
                            if let Some(val) = entry.value().as_integer() {
                                let val_i64 = val as i64;
                                match name.value() {
                                    "x" => {
                                        if let Some(safe_v) =
                                            safe_i64_to_i32(val_i64, "shadow offset x")
                                        {
                                            shadow.offset_x = safe_v;
                                        }
                                    }
                                    "y" => {
                                        if let Some(safe_v) =
                                            safe_i64_to_i32(val_i64, "shadow offset y")
                                        {
                                            shadow.offset_y = safe_v;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                if let Some(s) = get_string(shadow_children, &["color"]) {
                    if let Some(c) = Color::from_hex(&s) {
                        shadow.color = c;
                    }
                }
                if let Some(s) = get_string(shadow_children, &["inactive-color"]) {
                    if let Some(c) = Color::from_hex(&s) {
                        shadow.inactive_color = c;
                    }
                }
                if has_flag(shadow_children, &["draw-behind-window"]) {
                    shadow.draw_behind_window = true;
                }
            }

            rule.shadow = Some(shadow);
        }
    }
}

/// Load layer rules from KDL file
pub fn load_layer_rules(path: &Path, settings: &mut Settings) {
    let (rules, next_id) = load_rules(
        path,
        "layer-rule",
        "Layer Rule",
        parse_layer_rule_node_children,
    );
    settings.layer_rules.rules = rules;
    settings.layer_rules.next_id = next_id;
}

// ============================================================================
// WINDOW RULES
// ============================================================================

/// Parse window rule node children into a WindowRule
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_window_rule_node_children(wr_children: &KdlDocument, rule: &mut WindowRule) {
    rule.matches.clear();
    rule.excludes.clear();

    // Parse match criteria
    for child in wr_children.nodes() {
        if child.name().value() == "match" {
            let mut m = WindowRuleMatch::default();
            for entry in child.entries() {
                if let Some(name) = entry.name() {
                    match name.value() {
                        "app-id" => {
                            if let Some(v) = entry.value().as_string() {
                                m.app_id = validate_regex_pattern(v, "window rule app-id");
                            }
                        }
                        "title" => {
                            if let Some(v) = entry.value().as_string() {
                                m.title = validate_regex_pattern(v, "window rule title");
                            }
                        }
                        "is-floating" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_floating = Some(v);
                            }
                        }
                        "is-active" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_active = Some(v);
                            }
                        }
                        "is-focused" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_focused = Some(v);
                            }
                        }
                        "is-active-in-column" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_active_in_column = Some(v);
                            }
                        }
                        "is-window-cast-target" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_window_cast_target = Some(v);
                            }
                        }
                        "is-urgent" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_urgent = Some(v);
                            }
                        }
                        "at-startup" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.at_startup = Some(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
            rule.matches.push(m);
        } else if child.name().value() == "exclude" {
            // Parse exclude criteria (same structure as match)
            let mut m = WindowRuleMatch::default();
            for entry in child.entries() {
                if let Some(name) = entry.name() {
                    match name.value() {
                        "app-id" => {
                            if let Some(v) = entry.value().as_string() {
                                m.app_id = validate_regex_pattern(v, "window rule exclude app-id");
                            }
                        }
                        "title" => {
                            if let Some(v) = entry.value().as_string() {
                                m.title = validate_regex_pattern(v, "window rule exclude title");
                            }
                        }
                        "is-floating" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_floating = Some(v);
                            }
                        }
                        "is-active" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_active = Some(v);
                            }
                        }
                        "is-focused" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_focused = Some(v);
                            }
                        }
                        "is-active-in-column" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_active_in_column = Some(v);
                            }
                        }
                        "is-window-cast-target" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_window_cast_target = Some(v);
                            }
                        }
                        "is-urgent" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.is_urgent = Some(v);
                            }
                        }
                        "at-startup" => {
                            if let Some(v) = entry.value().as_bool() {
                                m.at_startup = Some(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
            rule.excludes.push(m);
        }
    }

    // If no matches were parsed, add a default empty match
    if rule.matches.is_empty() {
        rule.matches.push(WindowRuleMatch::default());
    }

    // Open behavior
    if has_flag(wr_children, &["open-maximized"]) {
        rule.open_behavior = OpenBehavior::Maximized;
    } else if has_flag(wr_children, &["open-fullscreen"]) {
        rule.open_behavior = OpenBehavior::Fullscreen;
    } else if has_flag(wr_children, &["open-floating"]) {
        rule.open_behavior = OpenBehavior::Floating;
    }

    // Default floating position
    if let Some(dfp) = wr_children.get("default-floating-position") {
        let mut x = 0i32;
        let mut y = 0i32;
        let mut relative_to = PositionRelativeTo::TopLeft;

        for entry in dfp.entries() {
            if let Some(name) = entry.name() {
                match name.value() {
                    "x" => {
                        if let Some(v) = entry.value().as_integer() {
                            if let Some(safe_v) = safe_i64_to_i32(v as i64, "floating position x") {
                                x = safe_v;
                            }
                        }
                    }
                    "y" => {
                        if let Some(v) = entry.value().as_integer() {
                            if let Some(safe_v) = safe_i64_to_i32(v as i64, "floating position y") {
                                y = safe_v;
                            }
                        }
                    }
                    "relative-to" => {
                        if let Some(s) = entry.value().as_string() {
                            relative_to = PositionRelativeTo::from_kdl(s);
                        }
                    }
                    _ => {}
                }
            }
        }

        rule.default_floating_position = Some(FloatingPosition { x, y, relative_to });
    }

    // Opacity
    if let Some(v) = get_f64(wr_children, &["opacity"]) {
        rule.opacity = Some(safe_opacity_to_f32(v, "window rule opacity"));
    }

    // Corner radius
    if let Some(v) = get_i64(wr_children, &["geometry-corner-radius"]) {
        rule.corner_radius = safe_i64_to_i32(v, "window geometry-corner-radius");
    }

    // Clip to geometry
    if let Some(ctg) = wr_children.get("clip-to-geometry") {
        if let Some(entry) = ctg.entries().first() {
            if let Some(b) = entry.value().as_bool() {
                rule.clip_to_geometry = Some(b);
            }
        }
    }

    // Block screencast
    if let Some(bof) = wr_children.get("block-out-from") {
        for entry in bof.entries() {
            if entry.value().as_string() == Some("screencast") {
                rule.block_out_from_screencast = true;
            }
        }
    }

    // Open on output
    if let Some(v) = get_string(wr_children, &["open-on-output"]) {
        rule.open_on_output = Some(v);
    }

    // Open on workspace
    if let Some(v) = get_string(wr_children, &["open-on-workspace"]) {
        rule.open_on_workspace = Some(v);
    }

    // Open focused
    if let Some(node) = wr_children.get("open-focused") {
        if let Some(entry) = node.entries().first() {
            if let Some(val) = entry.value().as_bool() {
                rule.open_focused = Some(val);
            }
        }
    }

    // Default column width
    if let Some(dcw) = wr_children.get("default-column-width") {
        if let Some(dcw_children) = dcw.children() {
            if let Some(v) = get_f64(dcw_children, &["proportion"]) {
                rule.default_column_width = Some(v as f32);
            }
        }
    }

    // Default window height
    if let Some(dwh) = wr_children.get("default-window-height") {
        if let Some(dwh_children) = dwh.children() {
            if let Some(v) = get_f64(dwh_children, &["proportion"]) {
                rule.default_window_height = Some(v as f32);
            }
        }
    }

    // Open maximized to edges
    if has_flag(wr_children, &["open-maximized-to-edges"]) {
        rule.open_maximized_to_edges = Some(true);
    }

    // Scroll factor
    if let Some(v) = get_f64(wr_children, &["scroll-factor"]) {
        rule.scroll_factor = Some(v);
    }

    // Draw border with background
    if has_flag(wr_children, &["draw-border-with-background"]) {
        rule.draw_border_with_background = Some(true);
    }

    // Size constraints
    if let Some(v) = get_i64(wr_children, &["min-width"]) {
        rule.min_width = safe_i64_to_i32(v, "min-width");
    }
    if let Some(v) = get_i64(wr_children, &["max-width"]) {
        rule.max_width = safe_i64_to_i32(v, "max-width");
    }
    if let Some(v) = get_i64(wr_children, &["min-height"]) {
        rule.min_height = safe_i64_to_i32(v, "min-height");
    }
    if let Some(v) = get_i64(wr_children, &["max-height"]) {
        rule.max_height = safe_i64_to_i32(v, "max-height");
    }

    // Focus ring overrides
    if let Some(fr) = wr_children.get("focus-ring") {
        if let Some(fr_children) = fr.children() {
            if let Some(v) = get_i64(fr_children, &["width"]) {
                rule.focus_ring_width = safe_i64_to_i32(v, "focus-ring width");
            }
            if let Some(hex) = get_string(fr_children, &["active-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.focus_ring_active = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(hex) = get_string(fr_children, &["inactive-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.focus_ring_inactive = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(hex) = get_string(fr_children, &["urgent-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.focus_ring_urgent = Some(ColorOrGradient::Color(c));
                }
            }
        }
    }

    // Border overrides
    if let Some(border) = wr_children.get("border") {
        if let Some(border_children) = border.children() {
            if let Some(v) = get_i64(border_children, &["width"]) {
                rule.border_width = safe_i64_to_i32(v, "border width");
            }
            if let Some(hex) = get_string(border_children, &["active-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.border_active = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(hex) = get_string(border_children, &["inactive-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.border_inactive = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(hex) = get_string(border_children, &["urgent-color"]) {
                if let Some(c) = parse_color(&hex) {
                    rule.border_urgent = Some(ColorOrGradient::Color(c));
                }
            }
        }
    }

    // Variable refresh rate (per-window VRR)
    if let Some(vrr) = wr_children.get("variable-refresh-rate") {
        if has_flag_in_node(vrr, "on") {
            rule.variable_refresh_rate = Some(true);
        } else if has_flag_in_node(vrr, "off") {
            rule.variable_refresh_rate = Some(false);
        } else {
            // Check for boolean value
            if let Some(entry) = vrr.entries().first() {
                if let Some(b) = entry.value().as_bool() {
                    rule.variable_refresh_rate = Some(b);
                }
            }
        }
    }

    // Default column display (Normal/Tabbed)
    if let Some(v) = get_string(wr_children, &["default-column-display"]) {
        use crate::config::models::DefaultColumnDisplay;
        rule.default_column_display = Some(match v.as_str() {
            "tabbed" => DefaultColumnDisplay::Tabbed,
            _ => DefaultColumnDisplay::Normal,
        });
    }

    // Tiled state (for X11 compatibility)
    if let Some(ts) = wr_children.get("tiled-state") {
        if has_flag_in_node(ts, "tiled") {
            rule.tiled_state = Some(true);
        } else if has_flag_in_node(ts, "floating") {
            rule.tiled_state = Some(false);
        }
    }

    // Baba is float (animated floating effect)
    if has_flag(wr_children, &["baba-is-float"]) {
        rule.baba_is_float = Some(true);
    }

    // Per-window shadow settings
    if let Some(shadow_node) = wr_children.get("shadow") {
        // Check for simple on/off
        if has_flag_in_node(shadow_node, "off") {
            rule.shadow = Some(ShadowSettings {
                enabled: false,
                ..Default::default()
            });
        } else if has_flag_in_node(shadow_node, "on") {
            rule.shadow = Some(ShadowSettings::default());
        } else if let Some(shadow_children) = shadow_node.children() {
            // Full shadow configuration
            let mut shadow = ShadowSettings::default();
            if has_flag(shadow_children, &["off"]) {
                shadow.enabled = false;
            }
            if let Some(v) = get_i64(shadow_children, &["softness"]) {
                shadow.softness = v as i32;
            }
            if let Some(v) = get_i64(shadow_children, &["spread"]) {
                shadow.spread = v as i32;
            }
            if let Some(v) = get_i64(shadow_children, &["offset-x"]) {
                shadow.offset_x = v as i32;
            }
            if let Some(v) = get_i64(shadow_children, &["offset-y"]) {
                shadow.offset_y = v as i32;
            }
            if has_flag(shadow_children, &["draw-behind-window"]) {
                shadow.draw_behind_window = true;
            }
            if let Some(hex) = get_string(shadow_children, &["color"]) {
                if let Some(c) = parse_color(&hex) {
                    shadow.color = c;
                }
            }
            if let Some(hex) = get_string(shadow_children, &["inactive-color"]) {
                if let Some(c) = parse_color(&hex) {
                    shadow.inactive_color = c;
                }
            }
            rule.shadow = Some(shadow);
        }
    }

    // Per-window tab-indicator settings
    if let Some(ti_node) = wr_children.get("tab-indicator") {
        use crate::config::models::TabIndicatorPosition;
        // Check for simple on/off
        if has_flag_in_node(ti_node, "off") {
            rule.tab_indicator = Some(TabIndicatorSettings {
                enabled: false,
                ..Default::default()
            });
        } else if has_flag_in_node(ti_node, "on") {
            rule.tab_indicator = Some(TabIndicatorSettings::default());
        } else if let Some(ti_children) = ti_node.children() {
            // Full tab-indicator configuration
            let mut ti = TabIndicatorSettings::default();
            if has_flag(ti_children, &["off"]) {
                ti.enabled = false;
            }
            if has_flag(ti_children, &["hide-when-single-tab"]) {
                ti.hide_when_single_tab = true;
            }
            if has_flag(ti_children, &["place-within-column"]) {
                ti.place_within_column = true;
            }
            if let Some(v) = get_i64(ti_children, &["gap"]) {
                ti.gap = v as i32;
            }
            if let Some(v) = get_i64(ti_children, &["width"]) {
                ti.width = v as i32;
            }
            if let Some(v) = get_f64(ti_children, &["length"]) {
                ti.length_proportion = v as f32;
            }
            if let Some(pos) = get_string(ti_children, &["position"]) {
                ti.position = match pos.as_str() {
                    "right" => TabIndicatorPosition::Right,
                    "top" => TabIndicatorPosition::Top,
                    "bottom" => TabIndicatorPosition::Bottom,
                    _ => TabIndicatorPosition::Left,
                };
            }
            if let Some(v) = get_i64(ti_children, &["gaps-between-tabs"]) {
                ti.gaps_between_tabs = v as i32;
            }
            if let Some(v) = get_i64(ti_children, &["corner-radius"]) {
                ti.corner_radius = v as i32;
            }
            if let Some(hex) = get_string(ti_children, &["active-color"]) {
                if let Some(c) = parse_color(&hex) {
                    ti.active = ColorOrGradient::Color(c);
                }
            }
            if let Some(hex) = get_string(ti_children, &["inactive-color"]) {
                if let Some(c) = parse_color(&hex) {
                    ti.inactive = ColorOrGradient::Color(c);
                }
            }
            if let Some(hex) = get_string(ti_children, &["urgent-color"]) {
                if let Some(c) = parse_color(&hex) {
                    ti.urgent = ColorOrGradient::Color(c);
                }
            }
            rule.tab_indicator = Some(ti);
        }
    }
}

/// Load window rules from KDL file
pub fn load_window_rules(path: &Path, settings: &mut Settings) {
    let (rules, next_id) = load_rules(path, "window-rule", "Rule", parse_window_rule_node_children);
    settings.window_rules.rules = rules;
    settings.window_rules.next_id = next_id;
}
