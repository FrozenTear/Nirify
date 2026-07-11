//! Window and layer rules KDL generation
//!
//! Generates KDL for window rules and layer rules.
//!
//! Everything emitted here is verified against niri's config grammar. Nodes or
//! properties niri (knuffel) does not accept are never written, because a single
//! unknown node makes niri reject the user's whole config.

use super::helpers::escape_kdl_string;
use crate::config::models::{
    BackgroundEffectSettings, BlockOutFrom, CornerRadiusValue, LayerRuleMatch, LayerRulesSettings,
    PopupsSettings, RuleDefaultSize, ShadowSettings, TabIndicatorOverride, WindowRuleMatch,
    WindowRulesSettings,
};
use crate::version::FeatureCompat;

const GATE_COMMENT: &str = "    // (preserved via /- for older niri; applies on niri 26.04+)\n";

/// Prefix the first non-whitespace token of a rendered node block with `/-`
/// (KDL slashdash) so niri ignores it while the raw text (and thus the data)
/// is preserved on disk for re-loading.
fn slashdash_block(block: &str) -> String {
    let idx = block
        .find(|c: char| !c.is_whitespace())
        .unwrap_or(block.len());
    let mut s = String::with_capacity(block.len() + 2);
    s.push_str(&block[..idx]);
    s.push_str("/-");
    s.push_str(&block[idx..]);
    s
}

/// Format a float trimming a redundant `.0` (niri accepts both int and float).
fn fmt_num(v: f32) -> String {
    if v.is_finite() && v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

/// Sanitize a rule name for use in a `// name` comment line (no newlines).
fn sanitize_comment(name: &str) -> String {
    name.replace(['\n', '\r'], " ")
}

// ── Match / exclude line builders ───────────────────────────────────────────

fn build_match_or_exclude_line<F>(content: &mut String, directive: &str, builder: F)
where
    F: FnOnce(&mut Vec<String>),
{
    let mut parts = Vec::new();
    builder(&mut parts);
    if !parts.is_empty() {
        content.push_str(&format!("    {} {}\n", directive, parts.join(" ")));
    }
}

fn add_string_criterion(parts: &mut Vec<String>, name: &str, value: &Option<String>) {
    if let Some(ref v) = value {
        parts.push(format!("{}=\"{}\"", name, escape_kdl_string(v)));
    }
}

fn add_bool_criterion(parts: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(v) = value {
        parts.push(format!("{}={}", name, v));
    }
}

// ── Shared value emitters ───────────────────────────────────────────────────

/// Emit `default-column-width`/`default-window-height` blocks.
fn emit_default_size(content: &mut String, prefix: &str, size: &RuleDefaultSize) {
    match size {
        RuleDefaultSize::Natural => content.push_str(&format!("    {} {{}}\n", prefix)),
        RuleDefaultSize::Proportion(p) => content.push_str(&format!(
            "    {} {{ proportion {}; }}\n",
            prefix,
            fmt_num(*p)
        )),
        RuleDefaultSize::Fixed(n) => {
            content.push_str(&format!("    {} {{ fixed {}; }}\n", prefix, n))
        }
    }
}

/// Emit a `geometry-corner-radius` line at the given indent.
fn emit_corner_radius(content: &mut String, indent: &str, cr: &CornerRadiusValue) {
    if cr.is_uniform() {
        content.push_str(&format!(
            "{}geometry-corner-radius {}\n",
            indent,
            fmt_num(cr.top_left)
        ));
    } else {
        content.push_str(&format!(
            "{}geometry-corner-radius {} {} {} {}\n",
            indent,
            fmt_num(cr.top_left),
            fmt_num(cr.top_right),
            fmt_num(cr.bottom_right),
            fmt_num(cr.bottom_left)
        ));
    }
}

/// Emit a `shadow { ... }` block. Shared by window and layer rules.
fn emit_shadow(content: &mut String, shadow: &ShadowSettings) {
    if !shadow.enabled {
        content.push_str("    shadow {\n        off\n    }\n");
        return;
    }
    content.push_str("    shadow {\n");
    content.push_str("        on\n");
    content.push_str(&format!("        softness {}\n", shadow.softness));
    content.push_str(&format!("        spread {}\n", shadow.spread));
    content.push_str(&format!(
        "        offset x={} y={}\n",
        shadow.offset_x, shadow.offset_y
    ));
    content.push_str(&format!("        color \"{}\"\n", shadow.color.to_hex()));
    content.push_str(&format!(
        "        inactive-color \"{}\"\n",
        shadow.inactive_color.to_hex()
    ));
    if shadow.draw_behind_window {
        content.push_str("        draw-behind-window true\n");
    }
    content.push_str("    }\n");
}

/// Emit a `tab-indicator { ... }` block (colours only — niri rejects anything else).
fn emit_tab_indicator(content: &mut String, ti: &TabIndicatorOverride) {
    if ti.is_empty() {
        return;
    }
    content.push_str("    tab-indicator {\n");
    if let Some(ref c) = ti.active {
        content.push_str(&format!("        active-color \"{}\"\n", c.to_hex()));
    }
    if let Some(ref c) = ti.inactive {
        content.push_str(&format!("        inactive-color \"{}\"\n", c.to_hex()));
    }
    if let Some(ref c) = ti.urgent {
        content.push_str(&format!("        urgent-color \"{}\"\n", c.to_hex()));
    }
    content.push_str("    }\n");
}

/// Emit a `background-effect { ... }` block at the given indent (Since 26.04).
fn emit_background_effect(content: &mut String, indent: &str, be: &BackgroundEffectSettings) {
    if be.is_empty() {
        return;
    }
    let inner = format!("{}    ", indent);
    content.push_str(&format!("{}background-effect {{\n", indent));
    if let Some(x) = be.xray {
        content.push_str(&format!("{}xray {}\n", inner, x));
    }
    if let Some(b) = be.blur {
        content.push_str(&format!("{}blur {}\n", inner, b));
    }
    if let Some(n) = be.noise {
        content.push_str(&format!("{}noise {}\n", inner, fmt_num(n)));
    }
    if let Some(s) = be.saturation {
        content.push_str(&format!("{}saturation {}\n", inner, fmt_num(s)));
    }
    content.push_str(&format!("{}}}\n", indent));
}

/// Emit a `popups { ... }` block (Since 26.04).
fn emit_popups(content: &mut String, popups: &PopupsSettings) {
    if popups.is_empty() {
        return;
    }
    content.push_str("    popups {\n");
    if let Some(o) = popups.opacity {
        content.push_str(&format!("        opacity {:.2}\n", o));
    }
    if let Some(ref cr) = popups.geometry_corner_radius {
        emit_corner_radius(content, "        ", cr);
    }
    if let Some(ref be) = popups.background_effect {
        emit_background_effect(content, "        ", be);
    }
    content.push_str("    }\n");
}

fn block_out_str(bof: BlockOutFrom) -> &'static str {
    match bof {
        BlockOutFrom::Screencast => "screencast",
        BlockOutFrom::ScreenCapture => "screen-capture",
    }
}

// ============================================================================
// LAYER RULES
// ============================================================================

/// Generate layer-rules.kdl from layer rules settings.
pub fn generate_layer_rules_kdl(settings: &LayerRulesSettings, compat: FeatureCompat) -> String {
    let mut content = String::with_capacity(2048);
    content.push_str("// Layer rules - managed by Nirify\n");
    content.push_str("// Rules for layer-shell surfaces (panels, notifications, etc.)\n");

    if settings.rules.iter().any(|r| !r.enabled) {
        content.push_str("// Note: rules with the /- prefix are disabled via Nirify\n");
    }
    content.push('\n');

    if settings.rules.is_empty() {
        content.push_str("// No layer rules configured yet.\n");
        content.push_str("// Add rules through the UI or manually here.\n");
        return content;
    }

    for rule in &settings.rules {
        content.push_str(&format!("// {}\n", sanitize_comment(&rule.name)));
        if !rule.enabled {
            content.push_str("/-");
        }
        content.push_str("layer-rule {\n");

        for m in &rule.matches {
            build_layer_match_line(&mut content, "match", m, compat);
        }
        for m in &rule.excludes {
            build_layer_match_line(&mut content, "exclude", m, compat);
        }

        if let Some(bof) = rule.block_out_from {
            content.push_str(&format!("    block-out-from \"{}\"\n", block_out_str(bof)));
        }
        if let Some(opacity) = rule.opacity {
            content.push_str(&format!("    opacity {:.2}\n", opacity));
        }
        if let Some(ref cr) = rule.geometry_corner_radius {
            emit_corner_radius(&mut content, "    ", cr);
        }
        if rule.place_within_backdrop {
            content.push_str("    place-within-backdrop true\n");
        }
        if rule.baba_is_float {
            content.push_str("    baba-is-float true\n");
        }
        if let Some(ref shadow) = rule.shadow {
            emit_shadow(&mut content, shadow);
        }

        emit_gated_effects(
            &mut content,
            compat,
            rule.background_effect.as_ref(),
            rule.popups.as_ref(),
        );

        content.push_str("}\n\n");
    }

    content
}

fn build_layer_match_line(
    content: &mut String,
    directive: &str,
    m: &LayerRuleMatch,
    compat: FeatureCompat,
) {
    build_match_or_exclude_line(content, directive, |parts| {
        add_string_criterion(parts, "namespace", &m.namespace);
        add_bool_criterion(parts, "at-startup", m.at_startup);
        if let Some(layer) = m.layer {
            // `layer=` requires niri 26.04; on older niri preserve it slashdashed
            // (policy P1) so it survives a save→reload without silently broadening
            // what the rule matches.
            if compat.background_effects {
                parts.push(format!("layer=\"{}\"", layer.to_kdl()));
            } else {
                parts.push(format!("/-layer=\"{}\"", layer.to_kdl()));
            }
        }
    });
}

// ============================================================================
// WINDOW RULES
// ============================================================================

/// Generate window-rules.kdl from window rules settings.
pub fn generate_window_rules_kdl(
    settings: &WindowRulesSettings,
    float_settings_app: bool,
    compat: FeatureCompat,
) -> String {
    let mut content = String::with_capacity(2048);
    content.push_str("// Window rules - managed by Nirify\n\n");

    let has_nirify_rule = settings.rules.iter().any(|rule| {
        rule.matches
            .iter()
            .any(|m| m.app_id.as_ref().is_some_and(|id| id.contains("nirify")))
    });

    if float_settings_app && !has_nirify_rule {
        content.push_str("// Auto-generated: Float Nirify app\n");
        content.push_str("window-rule {\n");
        content.push_str("    match app-id=\"^nirify$\"\n");
        content.push_str("    open-floating true\n");
        content.push_str("}\n\n");
    }

    if settings.rules.iter().any(|r| !r.enabled) {
        content.push_str("// Note: rules with the /- prefix are disabled via Nirify\n");
    }
    content.push('\n');

    if settings.rules.is_empty() && !float_settings_app {
        content.push_str("// No window rules configured yet.\n");
        content.push_str("// Add rules through the UI or manually here.\n");
        return content;
    }

    for rule in &settings.rules {
        content.push_str(&format!("// {}\n", sanitize_comment(&rule.name)));
        if !rule.enabled {
            content.push_str("/-");
        }
        content.push_str("window-rule {\n");

        for m in &rule.matches {
            build_window_match_line(&mut content, "match", m);
        }
        for m in &rule.excludes {
            build_window_match_line(&mut content, "exclude", m);
        }

        // Opening behaviour — each independent; false is meaningful.
        if let Some(v) = rule.open_maximized {
            content.push_str(&format!("    open-maximized {}\n", v));
        }
        if let Some(v) = rule.open_maximized_to_edges {
            content.push_str(&format!("    open-maximized-to-edges {}\n", v));
        }
        if let Some(v) = rule.open_fullscreen {
            content.push_str(&format!("    open-fullscreen {}\n", v));
        }
        if let Some(v) = rule.open_floating {
            content.push_str(&format!("    open-floating {}\n", v));
        }
        if let Some(v) = rule.open_focused {
            content.push_str(&format!("    open-focused {}\n", v));
        }

        if let Some(ref pos) = rule.default_floating_position {
            content.push_str(&format!(
                "    default-floating-position x={} y={} relative-to=\"{}\"\n",
                pos.x,
                pos.y,
                pos.relative_to.to_kdl()
            ));
        }
        if let Some(ref output) = rule.open_on_output {
            content.push_str(&format!(
                "    open-on-output \"{}\"\n",
                escape_kdl_string(output)
            ));
        }
        if let Some(ref workspace) = rule.open_on_workspace {
            content.push_str(&format!(
                "    open-on-workspace \"{}\"\n",
                escape_kdl_string(workspace)
            ));
        }
        if let Some(opacity) = rule.opacity {
            content.push_str(&format!("    opacity {:.2}\n", opacity));
        }
        if let Some(ref cr) = rule.corner_radius {
            emit_corner_radius(&mut content, "    ", cr);
        }
        if let Some(clip) = rule.clip_to_geometry {
            content.push_str(&format!("    clip-to-geometry {}\n", clip));
        }
        if let Some(bof) = rule.block_out_from {
            content.push_str(&format!("    block-out-from \"{}\"\n", block_out_str(bof)));
        }
        if let Some(ref size) = rule.default_column_width {
            emit_default_size(&mut content, "default-column-width", size);
        }
        if let Some(ref size) = rule.default_window_height {
            emit_default_size(&mut content, "default-window-height", size);
        }
        if let Some(factor) = rule.scroll_factor {
            content.push_str(&format!("    scroll-factor {:.2}\n", factor));
        }
        if let Some(v) = rule.draw_border_with_background {
            content.push_str(&format!("    draw-border-with-background {}\n", v));
        }
        if let Some(min) = rule.min_width {
            content.push_str(&format!("    min-width {}\n", min));
        }
        if let Some(max) = rule.max_width {
            content.push_str(&format!("    max-width {}\n", max));
        }
        if let Some(min) = rule.min_height {
            content.push_str(&format!("    min-height {}\n", min));
        }
        if let Some(max) = rule.max_height {
            content.push_str(&format!("    max-height {}\n", max));
        }

        // Focus ring override
        if rule.focus_ring_enabled.is_some()
            || rule.focus_ring_width.is_some()
            || rule.focus_ring_active.is_some()
            || rule.focus_ring_inactive.is_some()
            || rule.focus_ring_urgent.is_some()
        {
            content.push_str("    focus-ring {\n");
            match rule.focus_ring_enabled {
                Some(false) => content.push_str("        off\n"),
                Some(true) => content.push_str("        on\n"),
                None => {}
            }
            if let Some(width) = rule.focus_ring_width {
                content.push_str(&format!("        width {}\n", width));
            }
            if let Some(ref color) = rule.focus_ring_active {
                content.push_str(&format!("        active-color \"{}\"\n", color.to_hex()));
            }
            if let Some(ref color) = rule.focus_ring_inactive {
                content.push_str(&format!("        inactive-color \"{}\"\n", color.to_hex()));
            }
            if let Some(ref color) = rule.focus_ring_urgent {
                content.push_str(&format!("        urgent-color \"{}\"\n", color.to_hex()));
            }
            content.push_str("    }\n");
        }

        // Border override
        if rule.border_enabled.is_some()
            || rule.border_width.is_some()
            || rule.border_active.is_some()
            || rule.border_inactive.is_some()
            || rule.border_urgent.is_some()
        {
            content.push_str("    border {\n");
            match rule.border_enabled {
                Some(false) => content.push_str("        off\n"),
                Some(true) => content.push_str("        on\n"),
                None => {}
            }
            if let Some(width) = rule.border_width {
                content.push_str(&format!("        width {}\n", width));
            }
            if let Some(ref color) = rule.border_active {
                content.push_str(&format!("        active-color \"{}\"\n", color.to_hex()));
            }
            if let Some(ref color) = rule.border_inactive {
                content.push_str(&format!("        inactive-color \"{}\"\n", color.to_hex()));
            }
            if let Some(ref color) = rule.border_urgent {
                content.push_str(&format!("        urgent-color \"{}\"\n", color.to_hex()));
            }
            content.push_str("    }\n");
        }

        if let Some(vrr) = rule.variable_refresh_rate {
            content.push_str(&format!("    variable-refresh-rate {}\n", vrr));
        }
        if let Some(ref display) = rule.default_column_display {
            use crate::config::models::DefaultColumnDisplay;
            if matches!(display, DefaultColumnDisplay::Tabbed) {
                content.push_str("    default-column-display \"tabbed\"\n");
            }
        }
        if let Some(tiled) = rule.tiled_state {
            content.push_str(&format!("    tiled-state {}\n", tiled));
        }
        if rule.baba_is_float == Some(true) {
            content.push_str("    baba-is-float true\n");
        }

        if let Some(ref shadow) = rule.shadow {
            emit_shadow(&mut content, shadow);
        }
        if let Some(ref ti) = rule.tab_indicator {
            emit_tab_indicator(&mut content, ti);
        }

        emit_gated_effects(
            &mut content,
            compat,
            rule.background_effect.as_ref(),
            rule.popups.as_ref(),
        );

        content.push_str("}\n\n");
    }

    content
}

fn build_window_match_line(content: &mut String, directive: &str, m: &WindowRuleMatch) {
    build_match_or_exclude_line(content, directive, |parts| {
        add_string_criterion(parts, "app-id", &m.app_id);
        add_string_criterion(parts, "title", &m.title);
        add_bool_criterion(parts, "is-floating", m.is_floating);
        add_bool_criterion(parts, "is-active", m.is_active);
        add_bool_criterion(parts, "is-focused", m.is_focused);
        add_bool_criterion(parts, "is-active-in-column", m.is_active_in_column);
        add_bool_criterion(parts, "is-window-cast-target", m.is_window_cast_target);
        add_bool_criterion(parts, "is-urgent", m.is_urgent);
        add_bool_criterion(parts, "at-startup", m.at_startup);
    });
}

/// Emit background-effect and popups blocks (Since niri 26.04).
///
/// When the detected niri version doesn't support them (`!compat.background_effects`),
/// the blocks are still written but slashdashed (`/-background-effect { ... }`) so
/// older niri ignores them while the data survives a save→reload round-trip
/// (policy P1: gate without data loss).
fn emit_gated_effects(
    content: &mut String,
    compat: FeatureCompat,
    background_effect: Option<&BackgroundEffectSettings>,
    popups: Option<&PopupsSettings>,
) {
    let has_be = background_effect.map(|b| !b.is_empty()).unwrap_or(false);
    let has_popups = popups.map(|p| !p.is_empty()).unwrap_or(false);
    if !has_be && !has_popups {
        return;
    }
    if !compat.background_effects {
        content.push_str(GATE_COMMENT);
        if let Some(be) = background_effect.filter(|b| !b.is_empty()) {
            let mut tmp = String::new();
            emit_background_effect(&mut tmp, "    ", be);
            content.push_str(&slashdash_block(&tmp));
        }
        if let Some(p) = popups.filter(|p| !p.is_empty()) {
            let mut tmp = String::new();
            emit_popups(&mut tmp, p);
            content.push_str(&slashdash_block(&tmp));
        }
        return;
    }
    if let Some(be) = background_effect {
        emit_background_effect(content, "    ", be);
    }
    if let Some(p) = popups {
        emit_popups(content, p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::{load_layer_rules, load_window_rules};
    use crate::config::models::*;
    use crate::config::Settings;
    use crate::types::{Color, ColorOrGradient};
    use std::io::Write;

    fn parse_ok(content: &str) {
        content
            .parse::<kdl::KdlDocument>()
            .expect("generated KDL should parse");
    }

    fn write_temp(content: &str, name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("nirify_test_{}_{}.kdl", name, std::process::id()));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    fn color(hex: &str) -> ColorOrGradient {
        ColorOrGradient::Color(Color::from_hex(hex).unwrap())
    }

    #[test]
    fn window_rule_full_roundtrip() {
        let rule = WindowRule {
            id: 0,
            enabled: true,
            name: "Full Rule".to_string(),
            matches: vec![WindowRuleMatch {
                app_id: Some("^firefox$".to_string()),
                title: Some("^Fig".to_string()),
                is_floating: Some(true),
                is_active: Some(false),
                is_urgent: Some(false),
                ..Default::default()
            }],
            excludes: vec![WindowRuleMatch {
                app_id: Some("^Steam$".to_string()),
                ..Default::default()
            }],
            open_maximized: Some(true),
            open_fullscreen: Some(false),
            open_floating: Some(true),
            open_maximized_to_edges: Some(false),
            open_focused: Some(true),
            opacity: Some(0.90),
            block_out_from: Some(BlockOutFrom::ScreenCapture),
            corner_radius: Some(CornerRadiusValue {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            }),
            clip_to_geometry: Some(true),
            open_on_output: Some("DP-1".to_string()),
            open_on_workspace: Some("chat".to_string()),
            default_floating_position: Some(FloatingPosition {
                x: 32,
                y: -16,
                relative_to: PositionRelativeTo::BottomRight,
            }),
            default_column_width: Some(RuleDefaultSize::Fixed(1200)),
            default_window_height: Some(RuleDefaultSize::Natural),
            scroll_factor: Some(1.5),
            draw_border_with_background: Some(true),
            min_width: Some(100),
            max_width: Some(800),
            min_height: Some(200),
            max_height: Some(600),
            focus_ring_enabled: Some(false),
            focus_ring_width: Some(2),
            focus_ring_active: Some(color("#7fc8ff")),
            border_enabled: Some(true),
            border_width: Some(3),
            border_active: Some(color("#ff0000")),
            variable_refresh_rate: Some(true),
            default_column_display: Some(DefaultColumnDisplay::Tabbed),
            shadow: Some(ShadowSettings {
                enabled: true,
                softness: 30,
                spread: 5,
                offset_x: 11,
                offset_y: -9,
                draw_behind_window: true,
                color: Color::from_hex("#00000070").unwrap(),
                inactive_color: Color::from_hex("#00000040").unwrap(),
                use_inactive_color: false,
            }),
            tab_indicator: Some(TabIndicatorOverride {
                active: Some(color("#ff0000")),
                inactive: Some(color("#808080")),
                urgent: Some(color("#0000ff")),
            }),
            background_effect: Some(BackgroundEffectSettings {
                xray: Some(true),
                blur: Some(true),
                noise: Some(0.05),
                saturation: Some(3.0),
            }),
            popups: Some(PopupsSettings {
                opacity: Some(0.85),
                geometry_corner_radius: Some(CornerRadiusValue::uniform(6.0)),
                background_effect: Some(BackgroundEffectSettings {
                    blur: Some(true),
                    ..Default::default()
                }),
            }),
            tiled_state: Some(true),
            baba_is_float: Some(true),
            focus_ring_inactive: None,
            focus_ring_urgent: None,
            border_inactive: None,
            border_urgent: None,
        };
        let settings = WindowRulesSettings {
            rules: vec![rule.clone()],
            next_id: 1,
        };

        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        parse_ok(&content);

        let path = write_temp(&content, "window_full");
        let mut loaded = Settings::default();
        load_window_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.window_rules.rules.len(), 1);
        let mut expected = rule;
        expected.id = 0;
        let got = &loaded.window_rules.rules[0];
        assert_eq!(got.name, expected.name);
        assert!(got.enabled);
        assert_eq!(got.matches, expected.matches);
        assert_eq!(got.excludes, expected.excludes);
        assert_eq!(got.open_maximized, expected.open_maximized);
        assert_eq!(got.open_fullscreen, expected.open_fullscreen);
        assert_eq!(got.open_floating, expected.open_floating);
        assert_eq!(
            got.open_maximized_to_edges,
            expected.open_maximized_to_edges
        );
        assert_eq!(got.open_focused, expected.open_focused);
        assert_eq!(got.block_out_from, expected.block_out_from);
        assert_eq!(got.corner_radius, expected.corner_radius);
        assert_eq!(got.default_column_width, expected.default_column_width);
        assert_eq!(got.default_window_height, expected.default_window_height);
        assert_eq!(
            got.default_floating_position,
            expected.default_floating_position
        );
        assert_eq!(got.scroll_factor, expected.scroll_factor);
        assert_eq!(got.shadow, expected.shadow);
        assert_eq!(got.tab_indicator, expected.tab_indicator);
        assert_eq!(got.background_effect, expected.background_effect);
        assert_eq!(got.popups, expected.popups);
        assert_eq!(got.min_width, expected.min_width);
        assert_eq!(got.max_height, expected.max_height);
        assert_eq!(got.variable_refresh_rate, expected.variable_refresh_rate);
        assert_eq!(got.default_column_display, expected.default_column_display);
        assert_eq!(got.tiled_state, expected.tiled_state);
        assert_eq!(got.baba_is_float, expected.baba_is_float);
    }

    #[test]
    fn layer_rule_full_roundtrip() {
        let rule = LayerRule {
            id: 0,
            enabled: true,
            name: "Full Layer".to_string(),
            matches: vec![LayerRuleMatch {
                namespace: Some("^waybar$".to_string()),
                at_startup: Some(true),
                layer: Some(LayerKind::Overlay),
            }],
            excludes: vec![LayerRuleMatch {
                namespace: Some("^notifications$".to_string()),
                at_startup: None,
                layer: Some(LayerKind::Top),
            }],
            block_out_from: Some(BlockOutFrom::ScreenCapture),
            opacity: Some(0.95),
            shadow: Some(ShadowSettings {
                enabled: true,
                offset_x: 3,
                offset_y: 4,
                ..Default::default()
            }),
            geometry_corner_radius: Some(CornerRadiusValue {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            }),
            place_within_backdrop: true,
            baba_is_float: true,
            background_effect: Some(BackgroundEffectSettings {
                xray: Some(false),
                ..Default::default()
            }),
            popups: Some(PopupsSettings {
                opacity: Some(0.85),
                ..Default::default()
            }),
        };
        let disabled_shadow = LayerRule {
            id: 1,
            name: "Shadow Off".to_string(),
            matches: vec![LayerRuleMatch {
                namespace: Some("^dock$".to_string()),
                ..Default::default()
            }],
            shadow: Some(ShadowSettings {
                enabled: false,
                ..Default::default()
            }),
            ..Default::default()
        };
        let settings = LayerRulesSettings {
            rules: vec![rule.clone(), disabled_shadow.clone()],
            next_id: 2,
        };

        let content = generate_layer_rules_kdl(&settings, FeatureCompat::all_enabled());
        parse_ok(&content);

        let path = write_temp(&content, "layer_full");
        let mut loaded = Settings::default();
        load_layer_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.layer_rules.rules.len(), 2);
        let got = &loaded.layer_rules.rules[0];
        assert_eq!(got.matches, rule.matches);
        assert_eq!(got.excludes, rule.excludes);
        assert_eq!(got.block_out_from, rule.block_out_from);
        assert_eq!(got.geometry_corner_radius, rule.geometry_corner_radius);
        assert_eq!(got.shadow, rule.shadow);
        assert_eq!(got.background_effect, rule.background_effect);
        assert_eq!(got.popups, rule.popups);
        assert!(got.place_within_backdrop);
        assert!(got.baba_is_float);

        let got2 = &loaded.layer_rules.rules[1];
        assert_eq!(got2.shadow.as_ref().map(|s| s.enabled), Some(false));
    }

    #[test]
    fn disabled_rules_roundtrip_preserves_order_and_content() {
        let a = WindowRule {
            name: "A".to_string(),
            matches: vec![WindowRuleMatch {
                app_id: Some("^a$".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let b = WindowRule {
            enabled: false,
            name: "B".to_string(),
            matches: vec![WindowRuleMatch {
                title: Some("^b".to_string()),
                ..Default::default()
            }],
            shadow: Some(ShadowSettings {
                enabled: true,
                offset_x: 7,
                ..Default::default()
            }),
            ..Default::default()
        };
        let c = WindowRule {
            name: "C".to_string(),
            matches: vec![WindowRuleMatch {
                app_id: Some("^c$".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![a, b.clone(), c],
            next_id: 3,
        };

        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        parse_ok(&content);
        let path = write_temp(&content, "disabled_order");
        let mut loaded = Settings::default();
        load_window_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        let rules = &loaded.window_rules.rules;
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].name, "A");
        assert!(rules[0].enabled);
        assert_eq!(rules[1].name, "B");
        assert!(!rules[1].enabled);
        assert_eq!(rules[1].matches[0].title.as_deref(), Some("^b"));
        assert_eq!(rules[1].shadow.as_ref().map(|s| s.offset_x), Some(7));
        assert_eq!(rules[2].name, "C");
        assert!(rules[2].enabled);
    }

    #[test]
    fn disabled_rule_with_brace_in_regex_survives() {
        let disabled = WindowRule {
            enabled: false,
            name: "Brace".to_string(),
            matches: vec![WindowRuleMatch {
                title: Some("^\\{".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let enabled = WindowRule {
            name: "After".to_string(),
            matches: vec![WindowRuleMatch {
                app_id: Some("^after$".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![disabled, enabled],
            next_id: 2,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        parse_ok(&content);
        let path = write_temp(&content, "brace_regex");
        let mut loaded = Settings::default();
        load_window_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.window_rules.rules.len(), 2);
        assert!(!loaded.window_rules.rules[0].enabled);
        assert_eq!(
            loaded.window_rules.rules[0].matches[0].title.as_deref(),
            Some("^\\{")
        );
        assert_eq!(loaded.window_rules.rules[1].name, "After");
    }

    #[test]
    fn gated_nodes_slashdashed_for_old_niri() {
        // Policy P1: on old/unknown niri, gated content is preserved slashdashed
        // (niri ignores it, Nirify reloads it) rather than dropped.
        let rule = WindowRule {
            name: "Effects".to_string(),
            background_effect: Some(BackgroundEffectSettings {
                blur: Some(true),
                ..Default::default()
            }),
            popups: Some(PopupsSettings {
                opacity: Some(0.5),
                ..Default::default()
            }),
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![rule],
            next_id: 1,
        };
        let compat = FeatureCompat {
            recent_windows: true,
            background_effects: false,
            blur: false,
            map_to_focused_output: false,
        };
        let content = generate_window_rules_kdl(&settings, false, compat);
        // Present but slashdashed (no enabled/live background-effect node).
        assert!(content.contains("/-background-effect"));
        assert!(content.contains("/-popups"));
        assert!(!content.contains("\n    background-effect"));
        assert!(!content.contains("\n    popups"));
        parse_ok(&content);

        let layer = LayerRule {
            name: "L".to_string(),
            matches: vec![LayerRuleMatch {
                namespace: Some("^x$".to_string()),
                layer: Some(LayerKind::Top),
                ..Default::default()
            }],
            ..Default::default()
        };
        let lsettings = LayerRulesSettings {
            rules: vec![layer],
            next_id: 1,
        };
        let lcontent = generate_layer_rules_kdl(&lsettings, compat);
        // The `layer="…"` criterion is preserved slashdashed, not omitted.
        assert!(lcontent.contains("/-layer=\"top\""));
        assert!(!lcontent.contains(" layer=\"top\""));
        parse_ok(&lcontent);
    }

    #[test]
    fn window_tab_indicator_emits_only_colors() {
        let rule = WindowRule {
            name: "TI".to_string(),
            tab_indicator: Some(TabIndicatorOverride {
                active: Some(color("#ff0000")),
                inactive: None,
                urgent: None,
            }),
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![rule],
            next_id: 1,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        parse_ok(&content);
        // Isolate the tab-indicator block.
        let start = content.find("tab-indicator {").unwrap();
        let block = &content[start..];
        let block = &block[..block.find('}').unwrap()];
        assert!(block.contains("active-color"));
        for forbidden in [
            " on\n",
            " off\n",
            "gap",
            "width",
            "length",
            "position",
            "corner-radius",
        ] {
            assert!(
                !block.contains(forbidden),
                "tab-indicator block leaked {:?}",
                forbidden
            );
        }
    }

    #[test]
    fn generate_window_rules_kdl_emits_disabled_rules_with_slashdash() {
        let settings = WindowRulesSettings {
            rules: vec![WindowRule {
                enabled: false,
                name: "Disabled Window Rule".to_string(),
                ..Default::default()
            }],
            next_id: 1,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        assert!(content.contains("// Disabled Window Rule"));
        assert!(content.contains("/-window-rule {"));
        parse_ok(&content);
    }

    #[test]
    fn generate_layer_rules_kdl_emits_disabled_rules_with_slashdash() {
        let settings = LayerRulesSettings {
            rules: vec![LayerRule {
                enabled: false,
                name: "Disabled Layer Rule".to_string(),
                ..Default::default()
            }],
            next_id: 1,
        };
        let content = generate_layer_rules_kdl(&settings, FeatureCompat::all_enabled());
        assert!(content.contains("// Disabled Layer Rule"));
        assert!(content.contains("/-layer-rule {"));
        parse_ok(&content);
    }

    #[test]
    fn generate_rules_kdl_marks_enabled_and_disabled() {
        let settings = WindowRulesSettings {
            rules: vec![
                WindowRule {
                    enabled: true,
                    name: "Active Window".to_string(),
                    ..Default::default()
                },
                WindowRule {
                    enabled: false,
                    name: "Hidden Window".to_string(),
                    ..Default::default()
                },
            ],
            next_id: 2,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        // Enabled rule has no slashdash on its own line.
        assert!(content.contains("// Active Window\nwindow-rule {"));
        // Disabled rule is slashdashed.
        assert!(content.contains("// Hidden Window\n/-window-rule {"));
        parse_ok(&content);
    }

    #[test]
    fn boolean_window_rule_properties_are_written_with_arguments() {
        let settings = WindowRulesSettings {
            rules: vec![WindowRule {
                open_maximized: Some(true),
                open_maximized_to_edges: Some(true),
                draw_border_with_background: Some(true),
                tiled_state: Some(true),
                baba_is_float: Some(true),
                ..Default::default()
            }],
            next_id: 1,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        assert!(content.contains("open-maximized true"));
        assert!(content.contains("open-maximized-to-edges true"));
        assert!(content.contains("draw-border-with-background true"));
        assert!(content.contains("tiled-state true"));
        assert!(content.contains("baba-is-float true"));
        parse_ok(&content);
    }

    #[test]
    fn focus_ring_and_border_force_on_roundtrip() {
        // Explicit "force on" (Some(true)) must emit `on` and survive reload.
        let rule = WindowRule {
            focus_ring_enabled: Some(true),
            border_enabled: Some(true),
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![rule],
            next_id: 1,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::all_enabled());
        assert!(content.contains("focus-ring {\n        on\n"));
        assert!(content.contains("border {\n        on\n"));
        assert!(!content.contains("off"));
        parse_ok(&content);

        let path = write_temp(&content, "force_on");
        let mut loaded = Settings::default();
        load_window_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();
        let got = &loaded.window_rules.rules[0];
        assert_eq!(got.focus_ring_enabled, Some(true));
        assert_eq!(got.border_enabled, Some(true));
    }

    #[test]
    fn gated_content_preserved_when_compat_none() {
        // Policy P1: when niri version is unknown/old, gated content
        // (background-effect, popups, layer= matcher) must be written slashdashed
        // and round-trip losslessly instead of being dropped.
        let rule = LayerRule {
            id: 0,
            enabled: true,
            name: "Gated".to_string(),
            matches: vec![LayerRuleMatch {
                namespace: Some("^waybar$".to_string()),
                at_startup: None,
                layer: Some(LayerKind::Overlay),
            }],
            background_effect: Some(BackgroundEffectSettings {
                xray: Some(true),
                blur: Some(true),
                noise: Some(0.05),
                saturation: Some(3.0),
            }),
            popups: Some(PopupsSettings {
                opacity: Some(0.85),
                geometry_corner_radius: Some(CornerRadiusValue::uniform(6.0)),
                background_effect: Some(BackgroundEffectSettings {
                    blur: Some(true),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };
        let settings = LayerRulesSettings {
            rules: vec![rule.clone()],
            next_id: 1,
        };

        // compat=default => background_effects unsupported (unknown/old niri).
        let content = generate_layer_rules_kdl(&settings, FeatureCompat::default());
        // niri sees valid KDL; gated pieces are slashdashed, not dropped.
        parse_ok(&content);
        assert!(content.contains("/-background-effect"));
        assert!(content.contains("/-popups"));
        assert!(content.contains("/-layer=\"overlay\""));

        let path = write_temp(&content, "gated_none");
        let mut loaded = Settings::default();
        load_layer_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.layer_rules.rules.len(), 1);
        let got = &loaded.layer_rules.rules[0];
        assert_eq!(got.matches, rule.matches);
        assert_eq!(got.background_effect, rule.background_effect);
        assert_eq!(got.popups, rule.popups);
    }

    #[test]
    fn gated_window_rule_content_preserved_when_compat_none() {
        let rule = WindowRule {
            background_effect: Some(BackgroundEffectSettings {
                xray: Some(true),
                blur: Some(false),
                noise: Some(0.1),
                saturation: Some(2.0),
            }),
            popups: Some(PopupsSettings {
                opacity: Some(0.7),
                ..Default::default()
            }),
            ..Default::default()
        };
        let settings = WindowRulesSettings {
            rules: vec![rule.clone()],
            next_id: 1,
        };
        let content = generate_window_rules_kdl(&settings, false, FeatureCompat::default());
        parse_ok(&content);
        assert!(content.contains("/-background-effect"));
        assert!(content.contains("/-popups"));

        let path = write_temp(&content, "gated_win_none");
        let mut loaded = Settings::default();
        load_window_rules(&path, &mut loaded);
        std::fs::remove_file(&path).ok();

        let got = &loaded.window_rules.rules[0];
        assert_eq!(got.background_effect, rule.background_effect);
        assert_eq!(got.popups, rule.popups);
    }
}
