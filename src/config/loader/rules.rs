//! Window and layer rules loader
//!
//! Handles window rules and layer rules parsing, including persistent disabled
//! rules (`/-` slashdash) via the string-aware `preprocess_disabled_rules` pass:
//! disabled nodes are renamed so the kdl crate parses them as real nodes, which
//! preserves document order and is immune to braces inside strings/comments.

use super::gradient::load_color_or_gradient;
use super::helpers::{
    parse_color, preprocess_disabled_rules, read_raw_file, unslashdash_gated_content,
};
use crate::config::models::{
    BackgroundEffectSettings, BlockOutFrom, CornerRadiusValue, FloatingPosition, LayerKind,
    LayerRule, LayerRuleMatch, PopupsSettings, PositionRelativeTo, RuleDefaultSize, Settings,
    ShadowSettings, TabIndicatorOverride, WindowRule, WindowRuleMatch,
};
use crate::config::parser::{get_f64, get_i64, get_string, has_flag, parse_document};
use crate::config::validation::validate_regex_pattern;
use kdl::{KdlDocument, KdlNode};
use log::{debug, warn};
use std::path::Path;

/// Safely convert i64 to i32 with bounds checking.
fn safe_i64_to_i32(value: i64, context: &str) -> Option<i32> {
    if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
        Some(value as i32)
    } else {
        warn!("Value {} out of i32 range for {}, ignoring", value, context);
        None
    }
}

/// Clamp opacity into 0.0..=1.0.
fn safe_opacity_to_f32(value: f64, context: &str) -> f32 {
    if !(0.0..=1.0).contains(&value) {
        warn!(
            "Opacity {} out of range (0.0-1.0) for {}, clamping",
            value, context
        );
    }
    (value.clamp(0.0, 1.0)) as f32
}

/// Extract rule name from a leading `// Rule Name` comment.
pub fn extract_name_from_leading_comment(node: &KdlNode) -> Option<String> {
    let format = node.format()?;
    let leading = &format.leading;
    if let Some(start) = leading.rfind("// ") {
        let after_comment = &leading[start + 3..];
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

// ── small entry helpers ─────────────────────────────────────────────────────

/// Read the first positional entry of a node as a bool; a bare node → true.
fn node_bool(node: &KdlNode) -> bool {
    for entry in node.entries() {
        if entry.name().is_none() {
            if let Some(b) = entry.value().as_bool() {
                return b;
            }
        }
    }
    true
}

/// Check if a node has a flag as a positional string entry (e.g. `shadow { on }`).
///
/// Retained as a shared helper for other loaders (e.g. system.rs); the rules
/// loader itself now uses child-node flag checks.
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

/// Read a `key value` numeric child as i32 (accepts int or float, rounds).
fn get_rounded_i32(children: &KdlDocument, key: &str) -> Option<i32> {
    get_f64(children, &[key]).map(|v| v.round() as i32)
}

/// Convert a KDL entry value to f32 (int or float).
fn entry_to_f32(entry: &kdl::KdlEntry) -> Option<f32> {
    entry
        .value()
        .as_float()
        .map(|f| f as f32)
        .or_else(|| entry.value().as_integer().map(|i| i as f32))
}

/// Convert a KDL entry value to i32 (int or rounded float).
fn entry_to_i32(entry: &kdl::KdlEntry) -> Option<i32> {
    if let Some(i) = entry.value().as_integer() {
        safe_i64_to_i32(i as i64, "entry")
    } else {
        entry.value().as_float().map(|f| f.round() as i32)
    }
}

// ── shared block parsers ────────────────────────────────────────────────────

/// Parse a `geometry-corner-radius` node (1 uniform value, or 4 per-corner).
fn parse_corner_radius(node: &KdlNode) -> Option<CornerRadiusValue> {
    let vals: Vec<f32> = node
        .entries()
        .iter()
        .filter(|e| e.name().is_none())
        .filter_map(entry_to_f32)
        .collect();
    match vals.len() {
        1 => Some(CornerRadiusValue::uniform(vals[0])),
        4 => Some(CornerRadiusValue {
            top_left: vals[0],
            top_right: vals[1],
            bottom_right: vals[2],
            bottom_left: vals[3],
        }),
        other => {
            warn!(
                "geometry-corner-radius expects 1 or 4 values, got {}; ignoring",
                other
            );
            None
        }
    }
}

/// Parse a `default-column-width` / `default-window-height` node.
fn parse_default_size(node: &KdlNode) -> Option<RuleDefaultSize> {
    match node.children() {
        None => Some(RuleDefaultSize::Natural),
        Some(ch) => {
            if let Some(v) = get_i64(ch, &["fixed"]) {
                return safe_i64_to_i32(v, "default size fixed").map(RuleDefaultSize::Fixed);
            }
            if let Some(v) = get_f64(ch, &["proportion"]) {
                return Some(RuleDefaultSize::Proportion(v as f32));
            }
            // Empty `{}` block → natural sizing.
            Some(RuleDefaultSize::Natural)
        }
    }
}

/// Parse a `shadow { ... }` node. Shared by window and layer rules.
fn parse_shadow_rule(shadow_node: &KdlNode) -> Option<ShadowSettings> {
    let mut shadow = ShadowSettings::default();
    let mut enabled = true;

    if let Some(ch) = shadow_node.children() {
        // `off` child disables; `on` (or any config) enables.
        if ch.get("off").is_some() {
            enabled = false;
        } else if ch.get("on").is_some() {
            enabled = true;
        }
        if let Some(v) = get_rounded_i32(ch, "softness") {
            shadow.softness = v;
        }
        if let Some(v) = get_rounded_i32(ch, "spread") {
            shadow.spread = v;
        }
        if let Some(offset_node) = ch.get("offset") {
            for entry in offset_node.entries() {
                if let Some(name) = entry.name() {
                    if let Some(v) = entry_to_i32(entry) {
                        match name.value() {
                            "x" => shadow.offset_x = v,
                            "y" => shadow.offset_y = v,
                            _ => {}
                        }
                    }
                }
            }
        }
        if let Some(s) = get_string(ch, &["color"]) {
            if let Some(c) = parse_color(&s) {
                shadow.color = c;
            }
        }
        if let Some(s) = get_string(ch, &["inactive-color"]) {
            if let Some(c) = parse_color(&s) {
                shadow.inactive_color = c;
            }
        }
        if has_flag(ch, &["draw-behind-window"]) {
            shadow.draw_behind_window = true;
        }
    }

    shadow.enabled = enabled;
    Some(shadow)
}

/// Parse a `background-effect { ... }` node (Since 26.04).
fn parse_background_effect(node: &KdlNode) -> Option<BackgroundEffectSettings> {
    let ch = node.children()?;
    let mut be = BackgroundEffectSettings::default();
    if let Some(n) = ch.get("xray") {
        be.xray = Some(node_bool(n));
    }
    if let Some(n) = ch.get("blur") {
        be.blur = Some(node_bool(n));
    }
    if let Some(v) = get_f64(ch, &["noise"]) {
        be.noise = Some(v as f32);
    }
    if let Some(v) = get_f64(ch, &["saturation"]) {
        be.saturation = Some(v as f32);
    }
    if be.is_empty() {
        None
    } else {
        Some(be)
    }
}

/// Parse a `popups { ... }` node (Since 26.04).
fn parse_popups(node: &KdlNode) -> Option<PopupsSettings> {
    let ch = node.children()?;
    let mut p = PopupsSettings::default();
    if let Some(v) = get_f64(ch, &["opacity"]) {
        p.opacity = Some(safe_opacity_to_f32(v, "popups opacity"));
    }
    if let Some(gcr) = ch.get("geometry-corner-radius") {
        p.geometry_corner_radius = parse_corner_radius(gcr);
    }
    if let Some(be) = ch.get("background-effect") {
        p.background_effect = parse_background_effect(be);
    }
    if p.is_empty() {
        None
    } else {
        Some(p)
    }
}

// ── match parsers ───────────────────────────────────────────────────────────

fn parse_layer_match(node: &KdlNode, context: &str) -> LayerRuleMatch {
    let mut m = LayerRuleMatch::default();
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "namespace" => {
                    if let Some(s) = entry.value().as_string() {
                        m.namespace = validate_regex_pattern(s, context);
                    }
                }
                "at-startup" => {
                    if let Some(b) = entry.value().as_bool() {
                        m.at_startup = Some(b);
                    }
                }
                "layer" => {
                    if let Some(s) = entry.value().as_string() {
                        m.layer = LayerKind::from_kdl(s);
                    }
                }
                _ => {}
            }
        }
    }
    m
}

fn parse_window_match(node: &KdlNode, context: &str) -> WindowRuleMatch {
    let mut m = WindowRuleMatch::default();
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "app-id" => {
                    if let Some(v) = entry.value().as_string() {
                        m.app_id = validate_regex_pattern(v, context);
                    }
                }
                "title" => {
                    if let Some(v) = entry.value().as_string() {
                        m.title = validate_regex_pattern(v, context);
                    }
                }
                "is-floating" => m.is_floating = entry.value().as_bool(),
                "is-active" => m.is_active = entry.value().as_bool(),
                "is-focused" => m.is_focused = entry.value().as_bool(),
                "is-active-in-column" => m.is_active_in_column = entry.value().as_bool(),
                "is-window-cast-target" => m.is_window_cast_target = entry.value().as_bool(),
                "is-urgent" => m.is_urgent = entry.value().as_bool(),
                "at-startup" => m.at_startup = entry.value().as_bool(),
                _ => {}
            }
        }
    }
    m
}

// ── generic load pipeline ───────────────────────────────────────────────────

trait RuleWithId {
    fn set_id(&mut self, id: u32);
    fn set_name(&mut self, name: String);
    fn set_enabled(&mut self, enabled: bool);
}

impl RuleWithId for LayerRule {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl RuleWithId for WindowRule {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Load rules (enabled and disabled) from a KDL file, preserving document order.
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
    let Some(raw) = read_raw_file(path) else {
        return (Vec::new(), 0);
    };

    // Legacy detection: old bare `off` at rule level (Option 1 era).
    let legacy_a = format!("{} {{\n    off", rule_node_name);
    let legacy_b = format!("{}{{off", rule_node_name);
    if raw.contains(&legacy_a) || raw.contains(&legacy_b) {
        warn!(
            "Legacy disabled {} syntax detected in {:?}. The old bare 'off' format \
             is no longer supported; re-save from Nirify to migrate to /- syntax.",
            rule_node_name, path
        );
    }

    // First un-slashdash any version-gated content the generator preserved via `/-`
    // (P1), then rewrite disabled top-level rules so the kdl crate parses both back.
    let processed = preprocess_disabled_rules(&unslashdash_gated_content(&raw), &[rule_node_name]);
    let doc = match parse_document(&processed) {
        Ok(doc) => doc,
        Err(e) => {
            warn!(
                "Failed to parse {:?} ({}): loading no {}s",
                path, e, rule_node_name
            );
            return (Vec::new(), 0);
        }
    };

    let disabled_name = format!("nirify-disabled-{}", rule_node_name);
    let mut rules = Vec::new();
    let mut next_id = 0u32;

    for node in doc.nodes() {
        let nm = node.name().value();
        let enabled = if nm == rule_node_name {
            true
        } else if nm == disabled_name {
            false
        } else {
            continue;
        };

        let mut rule = R::default();
        rule.set_id(next_id);
        let name = extract_name_from_leading_comment(node)
            .unwrap_or_else(|| format!("{} {}", name_prefix, next_id + 1));
        rule.set_name(name);

        if let Some(children) = node.children() {
            parser(children, &mut rule);
        }
        if !enabled {
            rule.set_enabled(false);
        }

        rules.push(rule);
        next_id += 1;
    }

    debug!("Loaded {} {}s from {:?}", rules.len(), rule_node_name, path);
    (rules, next_id)
}

// ============================================================================
// LAYER RULES
// ============================================================================

/// Parse layer rule node children into a LayerRule.
pub fn parse_layer_rule_node_children(children: &KdlDocument, rule: &mut LayerRule) {
    rule.enabled = !has_flag(children, &["off"]);

    rule.matches.clear();
    rule.excludes.clear();
    for node in children.nodes() {
        match node.name().value() {
            "match" => rule
                .matches
                .push(parse_layer_match(node, "layer rule namespace")),
            "exclude" => rule
                .excludes
                .push(parse_layer_match(node, "layer rule exclude namespace")),
            _ => {}
        }
    }
    if rule.matches.is_empty() {
        rule.matches.push(LayerRuleMatch::default());
    }

    if let Some(bof) = get_string(children, &["block-out-from"]) {
        rule.block_out_from = match bof.as_str() {
            "screencast" => Some(BlockOutFrom::Screencast),
            "screen-capture" => Some(BlockOutFrom::ScreenCapture),
            _ => None,
        };
    }

    if let Some(v) = get_f64(children, &["opacity"]) {
        rule.opacity = Some(safe_opacity_to_f32(v, "layer rule opacity"));
    }

    if let Some(gcr) = children.get("geometry-corner-radius") {
        rule.geometry_corner_radius = parse_corner_radius(gcr);
    }

    if has_flag(children, &["place-within-backdrop"]) {
        rule.place_within_backdrop = true;
    }
    if has_flag(children, &["baba-is-float"]) {
        rule.baba_is_float = true;
    }

    if let Some(shadow_node) = children.get("shadow") {
        rule.shadow = parse_shadow_rule(shadow_node);
    }
    if let Some(be) = children.get("background-effect") {
        rule.background_effect = parse_background_effect(be);
    }
    if let Some(popups) = children.get("popups") {
        rule.popups = parse_popups(popups);
    }
}

/// Load layer rules from KDL file.
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

/// Parse window rule node children into a WindowRule.
pub fn parse_window_rule_node_children(children: &KdlDocument, rule: &mut WindowRule) {
    rule.enabled = !has_flag(children, &["off"]);
    rule.matches.clear();
    rule.excludes.clear();

    for node in children.nodes() {
        match node.name().value() {
            "match" => rule
                .matches
                .push(parse_window_match(node, "window rule app-id/title")),
            "exclude" => rule
                .excludes
                .push(parse_window_match(node, "window rule exclude app-id/title")),
            _ => {}
        }
    }
    if rule.matches.is_empty() {
        rule.matches.push(WindowRuleMatch::default());
    }

    // Opening behaviour — each independent; a bool arg is expected, bare → true.
    rule.open_maximized = children.get("open-maximized").map(node_bool);
    rule.open_maximized_to_edges = children.get("open-maximized-to-edges").map(node_bool);
    rule.open_fullscreen = children.get("open-fullscreen").map(node_bool);
    rule.open_floating = children.get("open-floating").map(node_bool);
    rule.open_focused = children.get("open-focused").map(node_bool);

    if let Some(dfp) = children.get("default-floating-position") {
        let mut pos = FloatingPosition::default();
        for entry in dfp.entries() {
            if let Some(name) = entry.name() {
                match name.value() {
                    "x" => {
                        if let Some(v) = entry_to_i32(entry) {
                            pos.x = v;
                        }
                    }
                    "y" => {
                        if let Some(v) = entry_to_i32(entry) {
                            pos.y = v;
                        }
                    }
                    "relative-to" => {
                        if let Some(s) = entry.value().as_string() {
                            pos.relative_to = PositionRelativeTo::from_kdl(s);
                        }
                    }
                    _ => {}
                }
            }
        }
        rule.default_floating_position = Some(pos);
    }

    if let Some(v) = get_f64(children, &["opacity"]) {
        rule.opacity = Some(safe_opacity_to_f32(v, "window rule opacity"));
    }

    if let Some(gcr) = children.get("geometry-corner-radius") {
        rule.corner_radius = parse_corner_radius(gcr);
    }

    if let Some(ctg) = children.get("clip-to-geometry") {
        if let Some(entry) = ctg.entries().first() {
            if let Some(b) = entry.value().as_bool() {
                rule.clip_to_geometry = Some(b);
            }
        }
    }

    if let Some(bof) = get_string(children, &["block-out-from"]) {
        rule.block_out_from = match bof.as_str() {
            "screencast" => Some(BlockOutFrom::Screencast),
            "screen-capture" => Some(BlockOutFrom::ScreenCapture),
            _ => None,
        };
    }

    if let Some(v) = get_string(children, &["open-on-output"]) {
        rule.open_on_output = Some(v);
    }
    if let Some(v) = get_string(children, &["open-on-workspace"]) {
        rule.open_on_workspace = Some(v);
    }

    if let Some(dcw) = children.get("default-column-width") {
        rule.default_column_width = parse_default_size(dcw);
    }
    if let Some(dwh) = children.get("default-window-height") {
        rule.default_window_height = parse_default_size(dwh);
    }

    if let Some(v) = get_f64(children, &["scroll-factor"]) {
        rule.scroll_factor = Some(v);
    }

    if let Some(node) = children.get("draw-border-with-background") {
        rule.draw_border_with_background = Some(node_bool(node));
    }

    if let Some(v) = get_i64(children, &["min-width"]) {
        rule.min_width = safe_i64_to_i32(v, "min-width");
    }
    if let Some(v) = get_i64(children, &["max-width"]) {
        rule.max_width = safe_i64_to_i32(v, "max-width");
    }
    if let Some(v) = get_i64(children, &["min-height"]) {
        rule.min_height = safe_i64_to_i32(v, "min-height");
    }
    if let Some(v) = get_i64(children, &["max-height"]) {
        rule.max_height = safe_i64_to_i32(v, "max-height");
    }

    // Focus ring overrides.
    if let Some(fr) = children.get("focus-ring") {
        if let Some(ch) = fr.children() {
            if has_flag(ch, &["off"]) {
                rule.focus_ring_enabled = Some(false);
            } else if has_flag(ch, &["on"]) {
                rule.focus_ring_enabled = Some(true);
            }
            if let Some(v) = get_i64(ch, &["width"]) {
                rule.focus_ring_width = safe_i64_to_i32(v, "focus-ring width");
            }
            rule.focus_ring_active = load_color_or_gradient(ch, "active");
            rule.focus_ring_inactive = load_color_or_gradient(ch, "inactive");
            rule.focus_ring_urgent = load_color_or_gradient(ch, "urgent");
        }
    }

    // Border overrides.
    if let Some(border) = children.get("border") {
        if let Some(ch) = border.children() {
            if has_flag(ch, &["off"]) {
                rule.border_enabled = Some(false);
            } else if has_flag(ch, &["on"]) {
                rule.border_enabled = Some(true);
            }
            if let Some(v) = get_i64(ch, &["width"]) {
                rule.border_width = safe_i64_to_i32(v, "border width");
            }
            rule.border_active = load_color_or_gradient(ch, "active");
            rule.border_inactive = load_color_or_gradient(ch, "inactive");
            rule.border_urgent = load_color_or_gradient(ch, "urgent");
        }
    }

    if let Some(vrr) = children.get("variable-refresh-rate") {
        rule.variable_refresh_rate = Some(node_bool(vrr));
    }

    if let Some(v) = get_string(children, &["default-column-display"]) {
        use crate::config::models::DefaultColumnDisplay;
        rule.default_column_display = Some(match v.as_str() {
            "tabbed" => DefaultColumnDisplay::Tabbed,
            _ => DefaultColumnDisplay::Normal,
        });
    }

    if let Some(ts) = children.get("tiled-state") {
        if let Some(entry) = ts.entries().first() {
            if let Some(b) = entry.value().as_bool() {
                rule.tiled_state = Some(b);
            } else if entry.value().as_string() == Some("tiled") {
                rule.tiled_state = Some(true);
            } else if entry.value().as_string() == Some("floating") {
                rule.tiled_state = Some(false);
            }
        }
    }

    if has_flag(children, &["baba-is-float"]) {
        rule.baba_is_float = Some(true);
    }

    if let Some(shadow_node) = children.get("shadow") {
        rule.shadow = parse_shadow_rule(shadow_node);
    }

    // Tab indicator: colours only (niri's TabIndicatorRule has no on/off/etc.).
    if let Some(ti_node) = children.get("tab-indicator") {
        if let Some(ch) = ti_node.children() {
            let mut ti = TabIndicatorOverride::default();
            ti.active = load_color_or_gradient(ch, "active");
            ti.inactive = load_color_or_gradient(ch, "inactive");
            ti.urgent = load_color_or_gradient(ch, "urgent");
            if !ti.is_empty() {
                rule.tab_indicator = Some(ti);
            }
        }
    }

    if let Some(be) = children.get("background-effect") {
        rule.background_effect = parse_background_effect(be);
    }
    if let Some(popups) = children.get("popups") {
        rule.popups = parse_popups(popups);
    }
}

/// Load window rules from KDL file.
pub fn load_window_rules(path: &Path, settings: &mut Settings) {
    let (rules, next_id) = load_rules(path, "window-rule", "Rule", parse_window_rule_node_children);
    settings.window_rules.rules = rules;
    settings.window_rules.next_id = next_id;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_layer_rule_node_children_reads_off_flag() {
        let document = parse_document("layer-rule {\n    off\n}\n").unwrap();
        let children = document.get("layer-rule").unwrap().children().unwrap();
        let mut rule = LayerRule::default();
        parse_layer_rule_node_children(children, &mut rule);
        assert!(!rule.enabled);
    }

    #[test]
    fn parse_window_rule_node_children_reads_off_flag() {
        let document = parse_document("window-rule {\n    off\n}\n").unwrap();
        let children = document.get("window-rule").unwrap().children().unwrap();
        let mut rule = WindowRule::default();
        parse_window_rule_node_children(children, &mut rule);
        assert!(!rule.enabled);
    }

    #[test]
    fn lone_brace_regex_preserved_on_load() {
        let document = parse_document("window-rule {\n    match title=\"foo}bar\"\n}\n").unwrap();
        let children = document.get("window-rule").unwrap().children().unwrap();
        let mut rule = WindowRule::default();
        parse_window_rule_node_children(children, &mut rule);
        assert_eq!(rule.matches[0].title.as_deref(), Some("foo}bar"));
    }

    #[test]
    fn open_behavior_reads_false() {
        let document =
            parse_document("window-rule {\n    open-fullscreen false\n    open-floating true\n}\n")
                .unwrap();
        let children = document.get("window-rule").unwrap().children().unwrap();
        let mut rule = WindowRule::default();
        parse_window_rule_node_children(children, &mut rule);
        assert_eq!(rule.open_fullscreen, Some(false));
        assert_eq!(rule.open_floating, Some(true));
        assert_eq!(rule.open_maximized, None);
    }
}
