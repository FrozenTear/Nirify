//! Window and layer rules KDL generation
//!
//! Generates KDL for window rules and layer rules.
//!
//! Uses `build_match_line` helper to reduce duplication in match criteria generation.

use super::helpers::escape_kdl_string;
use crate::config::models::{BlockOutFrom, LayerRulesSettings, OpenBehavior, WindowRulesSettings};

/// Helper to build a match line from criteria.
///
/// Takes a closure that populates match parts, then formats and writes
/// the match line if any criteria were added.
///
/// # Arguments
/// * `content` - String to append the match line to
/// * `builder` - Closure that adds criteria to the match_parts Vec
fn build_match_line<F>(content: &mut String, builder: F)
where
    F: FnOnce(&mut Vec<String>),
{
    build_match_or_exclude_line(content, "match", builder);
}

/// Helper to build an exclude line with criteria
fn build_exclude_line<F>(content: &mut String, builder: F)
where
    F: FnOnce(&mut Vec<String>),
{
    build_match_or_exclude_line(content, "exclude", builder);
}

/// Helper to build a match or exclude line with criteria
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

/// Helper to add a string criterion to match parts
fn add_string_criterion(parts: &mut Vec<String>, name: &str, value: &Option<String>) {
    if let Some(ref v) = value {
        parts.push(format!("{}=\"{}\"", name, escape_kdl_string(v)));
    }
}

/// Helper to add a boolean criterion to match parts
fn add_bool_criterion(parts: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(v) = value {
        parts.push(format!("{}={}", name, v));
    }
}

/// Generate layer-rules.kdl from layer rules settings
///
/// Creates KDL configuration for layer rules including:
/// - Match criteria (namespace, at-startup)
/// - Opacity, block-out-from, corner radius
/// - Shadow settings
/// - Special flags (place-within-backdrop, baba-is-float)
pub fn generate_layer_rules_kdl(settings: &LayerRulesSettings) -> String {
    // Pre-allocate ~2KB for layer rules
    let mut content = String::with_capacity(2048);
    content.push_str("// Layer rules - managed by niri-settings-rust\n");
    content.push_str("// Rules for layer-shell surfaces (panels, notifications, etc.)\n\n");

    if settings.rules.is_empty() {
        content.push_str("// No layer rules configured yet.\n");
        content.push_str("// Add rules through the UI or manually here.\n");
        content.push_str("// Example:\n");
        content.push_str("// layer-rule {\n");
        content.push_str("//     match namespace=\"^waybar$\"\n");
        content.push_str("//     opacity 0.95\n");
        content.push_str("// }\n");
    } else {
        for rule in &settings.rules {
            content.push_str(&format!("// {}\n", rule.name));
            content.push_str("layer-rule {\n");

            // Match criteria
            for m in &rule.matches {
                build_match_line(&mut content, |parts| {
                    add_string_criterion(parts, "namespace", &m.namespace);
                    add_bool_criterion(parts, "at-startup", m.at_startup);
                });
            }

            // block-out-from
            if let Some(ref bof) = rule.block_out_from {
                let bof_str = match bof {
                    BlockOutFrom::Screencast => "screencast",
                    BlockOutFrom::ScreenCapture => "screen-capture",
                };
                content.push_str(&format!("    block-out-from \"{}\"\n", bof_str));
            }

            // opacity
            if let Some(opacity) = rule.opacity {
                content.push_str(&format!("    opacity {:.2}\n", opacity));
            }

            // geometry-corner-radius
            if let Some(radius) = rule.geometry_corner_radius {
                content.push_str(&format!("    geometry-corner-radius {}\n", radius));
            }

            // place-within-backdrop
            if rule.place_within_backdrop {
                content.push_str("    place-within-backdrop\n");
            }

            // baba-is-float
            if rule.baba_is_float {
                content.push_str("    baba-is-float\n");
            }

            // shadow
            if let Some(ref shadow) = rule.shadow {
                if shadow.enabled {
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
                        content.push_str("        draw-behind-window\n");
                    }
                    content.push_str("    }\n");
                }
            }

            content.push_str("}\n\n");
        }
    }

    content
}

/// Generate window-rules.kdl from window rules settings
///
/// # Arguments
/// * `settings` - Window rules settings
/// * `float_settings_app` - Whether to add a rule to float the settings app
pub fn generate_window_rules_kdl(settings: &WindowRulesSettings, float_settings_app: bool) -> String {
    // Pre-allocate ~2KB for window rules (can be complex)
    let mut content = String::with_capacity(2048);
    content.push_str("// Window rules - managed by niri-settings-rust\n\n");

    // Auto-generated rule to float the settings app
    if float_settings_app {
        content.push_str("// Auto-generated: Float niri-settings app\n");
        content.push_str("window-rule {\n");
        content.push_str("    match app-id=\"^niri-settings$\"\n");
        content.push_str("    open-floating true\n");
        content.push_str("}\n\n");
    }

    if settings.rules.is_empty() && !float_settings_app {
        content.push_str("// No window rules configured yet.\n");
        content.push_str("// Add rules through the UI or manually here.\n");
        content.push_str("// Example:\n");
        content.push_str("// window-rule {\n");
        content.push_str("//     match app-id=\"firefox\"\n");
        content.push_str("//     open-maximized true\n");
        content.push_str("// }\n");
    } else if !settings.rules.is_empty() {
        for rule in &settings.rules {
            content.push_str(&format!("// {}\n", rule.name));
            content.push_str("window-rule {\n");

            // Match criteria (multiple matches supported - rule applies if ANY match)
            for m in &rule.matches {
                build_match_line(&mut content, |parts| {
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

            // Exclude criteria (multiple excludes supported - rule doesn't apply if ANY exclude matches)
            for m in &rule.excludes {
                build_exclude_line(&mut content, |parts| {
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

            // Opening behavior (these are flags in KDL, not boolean properties)
            match rule.open_behavior {
                OpenBehavior::Maximized => {
                    content.push_str("    open-maximized true\n");
                }
                OpenBehavior::Fullscreen => {
                    content.push_str("    open-fullscreen true\n");
                }
                OpenBehavior::Floating => {
                    content.push_str("    open-floating true\n");
                }
                OpenBehavior::Normal => {}
            }

            // Default floating position (for floating windows)
            if let Some(ref pos) = rule.default_floating_position {
                content.push_str(&format!(
                    "    default-floating-position x={} y={} relative-to=\"{}\"\n",
                    pos.x,
                    pos.y,
                    pos.relative_to.to_kdl()
                ));
            }

            // Open focused
            if let Some(focused) = rule.open_focused {
                if focused {
                    content.push_str("    open-focused true\n");
                } else {
                    content.push_str("    open-focused false\n");
                }
            }

            // Open on specific output
            if let Some(ref output) = rule.open_on_output {
                content.push_str(&format!(
                    "    open-on-output \"{}\"\n",
                    escape_kdl_string(output)
                ));
            }

            // Open on specific workspace
            if let Some(ref workspace) = rule.open_on_workspace {
                content.push_str(&format!(
                    "    open-on-workspace \"{}\"\n",
                    escape_kdl_string(workspace)
                ));
            }

            // Opacity
            if let Some(opacity) = rule.opacity {
                content.push_str(&format!("    opacity {:.2}\n", opacity));
            }

            // Corner radius
            if let Some(radius) = rule.corner_radius {
                content.push_str(&format!("    geometry-corner-radius {}\n", radius));
            }

            // Clip to geometry
            if let Some(clip) = rule.clip_to_geometry {
                content.push_str(&format!("    clip-to-geometry {}\n", clip));
            }

            // Block from screencast
            if rule.block_out_from_screencast {
                content.push_str("    block-out-from \"screencast\"\n");
            }

            // Default column width
            if let Some(width) = rule.default_column_width {
                content.push_str(&format!(
                    "    default-column-width {{ proportion {:.2}; }}\n",
                    width
                ));
            }

            // Default window height
            if let Some(height) = rule.default_window_height {
                content.push_str(&format!(
                    "    default-window-height {{ proportion {:.2}; }}\n",
                    height
                ));
            }

            // Open maximized to edges
            if let Some(true) = rule.open_maximized_to_edges {
                content.push_str("    open-maximized-to-edges\n");
            }

            // Scroll factor
            if let Some(factor) = rule.scroll_factor {
                content.push_str(&format!("    scroll-factor {:.2}\n", factor));
            }

            // Draw border with background
            if let Some(true) = rule.draw_border_with_background {
                content.push_str("    draw-border-with-background\n");
            }

            // Size constraints
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

            // Focus ring overrides
            if rule.focus_ring_width.is_some()
                || rule.focus_ring_active.is_some()
                || rule.focus_ring_inactive.is_some()
                || rule.focus_ring_urgent.is_some()
            {
                content.push_str("    focus-ring {\n");
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

            // Border overrides
            if rule.border_width.is_some()
                || rule.border_active.is_some()
                || rule.border_inactive.is_some()
                || rule.border_urgent.is_some()
            {
                content.push_str("    border {\n");
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

            // Variable refresh rate (boolean, not string)
            if let Some(vrr) = rule.variable_refresh_rate {
                content.push_str(&format!("    variable-refresh-rate {}\n", vrr));
            }

            // Default column display
            if let Some(ref display) = rule.default_column_display {
                use crate::config::models::DefaultColumnDisplay;
                match display {
                    DefaultColumnDisplay::Tabbed => {
                        content.push_str("    default-column-display \"tabbed\"\n");
                    }
                    DefaultColumnDisplay::Normal => {} // Don't output default
                }
            }

            // Tiled state
            if let Some(tiled) = rule.tiled_state {
                let state_str = if tiled { "tiled" } else { "floating" };
                content.push_str(&format!("    tiled-state \"{}\"\n", state_str));
            }

            // Baba is float
            if rule.baba_is_float == Some(true) {
                content.push_str("    baba-is-float\n");
            }

            // Per-window shadow settings
            if let Some(ref shadow) = rule.shadow {
                if !shadow.enabled {
                    content.push_str("    shadow {\n        off\n    }\n");
                } else {
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
                        content.push_str("        draw-behind-window\n");
                    }
                    content.push_str("    }\n");
                }
            }

            // Per-window tab-indicator settings
            if let Some(ref ti) = rule.tab_indicator {
                use crate::config::models::TabIndicatorPosition;
                if !ti.enabled {
                    content.push_str("    tab-indicator {\n        off\n    }\n");
                } else {
                    content.push_str("    tab-indicator {\n");
                    content.push_str("        on\n");
                    if ti.hide_when_single_tab {
                        content.push_str("        hide-when-single-tab\n");
                    }
                    if ti.place_within_column {
                        content.push_str("        place-within-column\n");
                    }
                    content.push_str(&format!("        gap {}\n", ti.gap));
                    content.push_str(&format!("        width {}\n", ti.width));
                    content.push_str(&format!(
                        "        length {{ proportion {:.2}; }}\n",
                        ti.length_proportion
                    ));
                    let pos_str = match ti.position {
                        TabIndicatorPosition::Left => "left",
                        TabIndicatorPosition::Right => "right",
                        TabIndicatorPosition::Top => "top",
                        TabIndicatorPosition::Bottom => "bottom",
                    };
                    content.push_str(&format!("        position \"{}\"\n", pos_str));
                    content.push_str(&format!(
                        "        gaps-between-tabs {}\n",
                        ti.gaps_between_tabs
                    ));
                    content.push_str(&format!("        corner-radius {}\n", ti.corner_radius));
                    content.push_str(&format!(
                        "        active-color \"{}\"\n",
                        ti.active.to_hex()
                    ));
                    content.push_str(&format!(
                        "        inactive-color \"{}\"\n",
                        ti.inactive.to_hex()
                    ));
                    content.push_str(&format!(
                        "        urgent-color \"{}\"\n",
                        ti.urgent.to_hex()
                    ));
                    content.push_str("    }\n");
                }
            }

            content.push_str("}\n\n");
        }
    }

    content
}
