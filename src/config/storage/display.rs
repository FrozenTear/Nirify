//! Display-related KDL generation
//!
//! Generates KDL configuration for animations, cursor, overview, and outputs.

use super::helpers::escape_kdl_string;
use crate::config::models::{
    AnimationSettings, AnimationType, CursorSettings, LayoutOverride, OutputSettings,
    OverviewSettings, SingleAnimationConfig,
};

/// Generate KDL for a single animation config
fn generate_single_animation_kdl(
    name: &str,
    config: &SingleAnimationConfig,
    indent: &str,
) -> Option<String> {
    match config.animation_type {
        AnimationType::Default => None, // Don't output, use niri defaults
        AnimationType::Off => Some(format!(
            "{}{} {{\n{}    off\n{}}}\n",
            indent, name, indent, indent
        )),
        AnimationType::Spring => {
            let spring = &config.spring;
            Some(format!(
                "{}{} {{\n{}    spring damping-ratio={:.4} stiffness={} epsilon={:.6}\n{}}}\n",
                indent,
                name,
                indent,
                spring.damping_ratio,
                spring.stiffness,
                spring.epsilon,
                indent
            ))
        }
        AnimationType::Easing => {
            let easing = &config.easing;
            let curve_str = if let Some((x1, y1, x2, y2)) = easing.curve.bezier_points() {
                // Cubic-bezier format: curve "cubic-bezier" x1 y1 x2 y2
                format!("curve \"cubic-bezier\" {} {} {} {}", x1, y1, x2, y2)
            } else {
                // Preset curve format: curve "ease-out-quad"
                format!(
                    "curve \"{}\"",
                    easing.curve.to_kdl().unwrap_or("ease-out-quad")
                )
            };
            Some(format!(
                "{}{} {{\n{}    duration-ms {}\n{}    {}\n{}}}\n",
                indent, name, indent, easing.duration_ms, indent, curve_str, indent
            ))
        }
        AnimationType::CustomShader => {
            // Custom GLSL shader - only valid for window-open, window-close, window-resize
            config.custom_shader.as_ref().map(|code| {
                format!(
                    "{}{} {{\n{}    custom-shader r\"\n{}\n\"\n{}}}\n",
                    indent, name, indent, code, indent
                )
            })
        }
    }
}

/// Generate animations.kdl content
pub fn generate_animations_kdl(settings: &AnimationSettings) -> String {
    // Pre-allocate for animation config with per-animation settings
    let mut content = String::with_capacity(2048);
    content.push_str("// Animation settings - managed by Nirify\n\nanimations {\n");

    if !settings.enabled {
        content.push_str("    off\n");
    }

    if (settings.slowdown - 1.0).abs() > 0.01 {
        content.push_str(&format!("    slowdown {:.2}\n", settings.slowdown));
    }

    // Per-animation configurations
    let per = &settings.per_animation;
    let animations: [(&str, &SingleAnimationConfig); 11] = [
        ("workspace-switch", &per.workspace_switch),
        ("window-open", &per.window_open),
        ("window-close", &per.window_close),
        ("horizontal-view-movement", &per.horizontal_view_movement),
        ("window-movement", &per.window_movement),
        ("window-resize", &per.window_resize),
        (
            "config-notification-open-close",
            &per.config_notification_open_close,
        ),
        (
            "exit-confirmation-open-close",
            &per.exit_confirmation_open_close,
        ),
        ("screenshot-ui-open", &per.screenshot_ui_open),
        ("overview-open-close", &per.overview_open_close),
        ("recent-windows-close", &per.recent_windows_close),
    ];

    for (name, config) in animations {
        if let Some(anim_kdl) = generate_single_animation_kdl(name, config, "    ") {
            content.push('\n');
            content.push_str(&anim_kdl);
        }
    }

    content.push_str("}\n");
    content
}

/// Generate cursor.kdl content
pub fn generate_cursor_kdl(settings: &CursorSettings) -> String {
    // Pre-allocate ~256 bytes for cursor config
    let mut content = String::with_capacity(256);
    content.push_str("// Cursor settings - managed by Nirify\n\ncursor {\n");

    if !settings.theme.is_empty() {
        content.push_str(&format!(
            "    xcursor-theme \"{}\"\n",
            escape_kdl_string(&settings.theme)
        ));
    }
    content.push_str(&format!("    xcursor-size {}\n", settings.size));

    if settings.hide_when_typing {
        content.push_str("    hide-when-typing\n");
    }

    if let Some(ms) = settings.hide_after_inactive_ms {
        content.push_str(&format!("    hide-after-inactive-ms {}\n", ms));
    }

    content.push_str("}\n");
    content
}

/// Generate overview.kdl content
pub fn generate_overview_kdl(settings: &OverviewSettings) -> String {
    // Pre-allocate ~512 bytes for overview config (with workspace-shadow)
    let mut content = String::with_capacity(512);
    content.push_str("// Overview settings - managed by Nirify\n\noverview {\n");

    content.push_str(&format!("    zoom {:.2}\n", settings.zoom));

    if let Some(ref color) = settings.backdrop_color {
        content.push_str(&format!("    backdrop-color \"{}\"\n", color.to_hex()));
    }

    // Workspace shadow (v25.05+)
    if let Some(ref shadow) = settings.workspace_shadow {
        content.push_str("    workspace-shadow {\n");
        if !shadow.enabled {
            content.push_str("        off\n");
        } else {
            content.push_str(&format!("        softness {}\n", shadow.softness));
            content.push_str(&format!("        spread {}\n", shadow.spread));
            content.push_str(&format!(
                "        offset x={} y={}\n",
                shadow.offset_x, shadow.offset_y
            ));
            content.push_str(&format!("        color \"{}\"\n", shadow.color.to_hex()));
        }
        content.push_str("    }\n");
    }

    content.push_str("}\n");
    content
}

/// Generate KDL for a layout override block
pub fn generate_layout_override_kdl(layout: &LayoutOverride, indent: &str) -> String {
    use crate::config::models::{
        DefaultColumnDisplay, PresetHeight, PresetWidth, TabIndicatorPosition,
    };
    use crate::config::storage::gradient::{color_or_gradient_to_kdl, gradient_node_to_kdl};
    use crate::types::ColorOrGradient;

    let mut content = String::with_capacity(512);
    let inner_indent = format!("{}    ", indent);
    let deep_indent = format!("{}        ", indent);

    content.push_str(&format!("{}layout {{\n", indent));

    // Gaps - niri `FloatOrInt` (do not coerce 0.5 to 0/1)
    if let Some(gaps) = layout.gaps {
        content.push_str(&format!(
            "{}gaps {}\n",
            inner_indent,
            crate::config::storage::builder::KdlBuilder::format_f32(gaps)
        ));
    }

    // Struts block
    let has_struts = layout.strut_left.is_some()
        || layout.strut_right.is_some()
        || layout.strut_top.is_some()
        || layout.strut_bottom.is_some();
    if has_struts {
        content.push_str(&format!("{}struts {{\n", inner_indent));
        if let Some(left) = layout.strut_left {
            content.push_str(&format!(
                "{}left {}\n",
                deep_indent,
                crate::config::storage::builder::KdlBuilder::format_f32(left)
            ));
        }
        if let Some(right) = layout.strut_right {
            content.push_str(&format!(
                "{}right {}\n",
                deep_indent,
                crate::config::storage::builder::KdlBuilder::format_f32(right)
            ));
        }
        if let Some(top) = layout.strut_top {
            content.push_str(&format!(
                "{}top {}\n",
                deep_indent,
                crate::config::storage::builder::KdlBuilder::format_f32(top)
            ));
        }
        if let Some(bottom) = layout.strut_bottom {
            content.push_str(&format!(
                "{}bottom {}\n",
                deep_indent,
                crate::config::storage::builder::KdlBuilder::format_f32(bottom)
            ));
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // center-focused-column
    if let Some(ref cfc) = layout.center_focused_column {
        content.push_str(&format!(
            "{}center-focused-column \"{}\"\n",
            inner_indent,
            cfc.to_kdl()
        ));
    }

    // always-center-single-column (emit explicit bool so `false` overrides round-trip)
    match layout.always_center_single_column {
        Some(true) => {
            content.push_str(&format!("{}always-center-single-column\n", inner_indent));
        }
        Some(false) => {
            content.push_str(&format!(
                "{}always-center-single-column false\n",
                inner_indent
            ));
        }
        None => {}
    }

    // empty-workspace-above-first
    match layout.empty_workspace_above_first {
        Some(true) => {
            content.push_str(&format!("{}empty-workspace-above-first\n", inner_indent));
        }
        Some(false) => {
            content.push_str(&format!(
                "{}empty-workspace-above-first false\n",
                inner_indent
            ));
        }
        None => {}
    }

    // background-color
    if let Some(ref c) = layout.background_color {
        content.push_str(&format!(
            "{}background-color \"{}\"\n",
            inner_indent,
            c.to_hex()
        ));
    }

    // default-column-display
    if let Some(ref dcd) = layout.default_column_display {
        let mode = match dcd {
            DefaultColumnDisplay::Normal => "normal",
            DefaultColumnDisplay::Tabbed => "tabbed",
        };
        content.push_str(&format!(
            "{}default-column-display \"{}\"\n",
            inner_indent, mode
        ));
    }

    // default-column-width
    // An empty block (`default-column-width {}`) is the niri "auto" form: windows
    // pick their own width. Emit it when marked auto so it round-trips.
    if layout.default_column_width_proportion.is_some()
        || layout.default_column_width_fixed.is_some()
        || layout.default_column_width_auto == Some(true)
    {
        content.push_str(&format!("{}default-column-width {{\n", inner_indent));
        if let Some(p) = layout.default_column_width_proportion {
            content.push_str(&format!("{}proportion {:.5}\n", deep_indent, p));
        }
        if let Some(f) = layout.default_column_width_fixed {
            content.push_str(&format!(
                "{}fixed {}\n",
                deep_indent,
                crate::config::storage::builder::KdlBuilder::format_f32(f)
            ));
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // preset-column-widths
    if let Some(ref presets) = layout.preset_column_widths {
        content.push_str(&format!("{}preset-column-widths {{\n", inner_indent));
        for preset in presets {
            match preset {
                PresetWidth::Proportion(p) => {
                    content.push_str(&format!("{}proportion {:.5}\n", deep_indent, p));
                }
                PresetWidth::Fixed(f) => {
                    content.push_str(&format!("{}fixed {}\n", deep_indent, f));
                }
            }
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // preset-window-heights
    if let Some(ref presets) = layout.preset_window_heights {
        content.push_str(&format!("{}preset-window-heights {{\n", inner_indent));
        for preset in presets {
            match preset {
                PresetHeight::Proportion(p) => {
                    content.push_str(&format!("{}proportion {:.5}\n", deep_indent, p));
                }
                PresetHeight::Fixed(f) => {
                    content.push_str(&format!("{}fixed {}\n", deep_indent, f));
                }
            }
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // focus-ring
    let has_focus_ring = layout.focus_ring_enabled.is_some()
        || layout.focus_ring_width.is_some()
        || layout.focus_ring_active.is_some()
        || layout.focus_ring_inactive.is_some()
        || layout.focus_ring_urgent.is_some();
    if has_focus_ring {
        content.push_str(&format!("{}focus-ring {{\n", inner_indent));
        if layout.focus_ring_enabled == Some(false) {
            content.push_str(&format!("{}off\n", deep_indent));
        }
        if let Some(w) = layout.focus_ring_width {
            content.push_str(&format!("{}width {}\n", deep_indent, w));
        }
        if let Some(ref cog) = layout.focus_ring_active {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "active")
            ));
        }
        if let Some(ref cog) = layout.focus_ring_inactive {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "inactive")
            ));
        }
        if let Some(ref cog) = layout.focus_ring_urgent {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "urgent")
            ));
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // border
    let has_border = layout.border_enabled.is_some()
        || layout.border_width.is_some()
        || layout.border_active.is_some()
        || layout.border_inactive.is_some()
        || layout.border_urgent.is_some();
    if has_border {
        content.push_str(&format!("{}border {{\n", inner_indent));
        if layout.border_enabled == Some(false) {
            content.push_str(&format!("{}off\n", deep_indent));
        }
        if let Some(w) = layout.border_width {
            content.push_str(&format!("{}width {}\n", deep_indent, w));
        }
        if let Some(ref cog) = layout.border_active {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "active")
            ));
        }
        if let Some(ref cog) = layout.border_inactive {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "inactive")
            ));
        }
        if let Some(ref cog) = layout.border_urgent {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(cog, "urgent")
            ));
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // shadow
    let has_shadow = layout.shadow_enabled.is_some()
        || layout.shadow_softness.is_some()
        || layout.shadow_spread.is_some()
        || layout.shadow_offset_x.is_some()
        || layout.shadow_offset_y.is_some()
        || layout.shadow_color.is_some()
        || layout.shadow_inactive_color.is_some()
        || layout.shadow_draw_behind_window.is_some();
    if has_shadow {
        content.push_str(&format!("{}shadow {{\n", inner_indent));
        if layout.shadow_enabled == Some(false) {
            content.push_str(&format!("{}off\n", deep_indent));
        } else {
            if let Some(s) = layout.shadow_softness {
                content.push_str(&format!("{}softness {}\n", deep_indent, s));
            }
            if let Some(s) = layout.shadow_spread {
                content.push_str(&format!("{}spread {}\n", deep_indent, s));
            }
            if layout.shadow_offset_x.is_some() || layout.shadow_offset_y.is_some() {
                let x = layout.shadow_offset_x.unwrap_or(0);
                let y = layout.shadow_offset_y.unwrap_or(0);
                content.push_str(&format!("{}offset x={} y={}\n", deep_indent, x, y));
            }
            if let Some(ref c) = layout.shadow_color {
                content.push_str(&format!("{}color \"{}\"\n", deep_indent, c.to_hex()));
            }
            if let Some(ref c) = layout.shadow_inactive_color {
                content.push_str(&format!(
                    "{}inactive-color \"{}\"\n",
                    deep_indent,
                    c.to_hex()
                ));
            }
            if let Some(draw) = layout.shadow_draw_behind_window {
                content.push_str(&format!("{}draw-behind-window {}\n", deep_indent, draw));
            }
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // tab-indicator sub-block (mirrors the global layout-extras generator so it
    // round-trips through parse_layout_extras_from_children).
    if let Some(ref ti) = layout.tab_indicator {
        content.push_str(&format!("{}tab-indicator {{\n", inner_indent));
        content.push_str(&format!(
            "{}{}\n",
            deep_indent,
            if ti.enabled { "on" } else { "off" }
        ));
        let position_str = match ti.position {
            TabIndicatorPosition::Left => "left",
            TabIndicatorPosition::Right => "right",
            TabIndicatorPosition::Top => "top",
            TabIndicatorPosition::Bottom => "bottom",
        };
        content.push_str(&format!("{}position \"{}\"\n", deep_indent, position_str));
        content.push_str(&format!("{}width {}\n", deep_indent, ti.width));
        content.push_str(&format!("{}gap {}\n", deep_indent, ti.gap));
        content.push_str(&format!(
            "{}gaps-between-tabs {}\n",
            deep_indent, ti.gaps_between_tabs
        ));
        content.push_str(&format!(
            "{}corner-radius {}\n",
            deep_indent, ti.corner_radius
        ));
        content.push_str(&format!(
            "{}length total-proportion={:.2}\n",
            deep_indent, ti.length_proportion
        ));
        if ti.use_active_color {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(&ti.active, "active")
            ));
        }
        if ti.use_inactive_color {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(&ti.inactive, "inactive")
            ));
        }
        if ti.use_urgent_color {
            content.push_str(&format!(
                "{}{}\n",
                deep_indent,
                color_or_gradient_to_kdl(&ti.urgent, "urgent")
            ));
        }
        if ti.hide_when_single_tab {
            content.push_str(&format!("{}hide-when-single-tab\n", deep_indent));
        }
        if ti.place_within_column {
            content.push_str(&format!("{}place-within-column\n", deep_indent));
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    // insert-hint sub-block.
    if let Some(ref ih) = layout.insert_hint {
        content.push_str(&format!("{}insert-hint {{\n", inner_indent));
        content.push_str(&format!(
            "{}{}\n",
            deep_indent,
            if ih.enabled { "on" } else { "off" }
        ));
        match &ih.color {
            ColorOrGradient::Color(c) => {
                content.push_str(&format!("{}color \"{}\"\n", deep_indent, c.to_hex()));
            }
            ColorOrGradient::Gradient(g) => {
                content.push_str(&format!(
                    "{}{}\n",
                    deep_indent,
                    gradient_node_to_kdl(g, "gradient")
                ));
            }
            ColorOrGradient::Raw(raw) => {
                content.push_str(&format!("{}{}\n", deep_indent, raw));
            }
        }
        content.push_str(&format!("{}}}\n", inner_indent));
    }

    content.push_str(&format!("{}}}\n", indent));
    content
}

/// Validate and normalize a modeline into a niri-accepted `modeline` node line.
///
/// Expects 11 whitespace-separated tokens: a clock (float MHz), 8 integer timings,
/// and hsync/vsync polarity flags. Returns the normalized node line (clock as
/// `{:.2}`, integers verbatim, polarity lowercased and quoted) without indentation,
/// or `None` if the input is not a valid modeline.
fn format_modeline(modeline: &str) -> Option<String> {
    let tokens: Vec<&str> = modeline.split_whitespace().collect();
    if tokens.len() != 11 {
        return None;
    }

    // Clock: float or int
    let clock: f64 = tokens[0].parse().ok()?;
    // Reject non-finite / negative clocks
    if !clock.is_finite() || clock < 0.0 {
        return None;
    }

    // 8 integer timings
    let mut ints: [u32; 8] = [0; 8];
    for (i, tok) in tokens[1..=8].iter().enumerate() {
        ints[i] = tok.parse::<u32>().ok()?;
    }

    // Polarity flags
    let hsync = tokens[9].to_ascii_lowercase();
    let vsync = tokens[10].to_ascii_lowercase();
    if hsync != "+hsync" && hsync != "-hsync" {
        return None;
    }
    if vsync != "+vsync" && vsync != "-vsync" {
        return None;
    }

    Some(format!(
        "modeline {:.2} {} {} {} {} {} {} {} {} \"{}\" \"{}\"",
        clock, ints[0], ints[1], ints[2], ints[3], ints[4], ints[5], ints[6], ints[7], hsync, vsync
    ))
}

/// Format an explicit output scale for KDL (`1.0`, `1.5`, `1.333333`).
fn format_output_scale(scale: f64) -> String {
    let formatted = format!("{:.6}", scale);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    if trimmed.contains('.') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.0")
    }
}

/// Generate outputs.kdl from output settings
pub fn generate_outputs_kdl(settings: &OutputSettings) -> String {
    // Pre-allocate ~1.5KB for outputs (multiple displays with new features)
    let mut content = String::with_capacity(1536);
    content.push_str("// Output/Display settings - managed by Nirify\n\n");

    let named_outputs: Vec<&crate::config::models::OutputConfig> = settings
        .outputs
        .iter()
        .filter(|output| !output.name.trim().is_empty())
        .collect();

    if named_outputs.is_empty() {
        content.push_str("// No outputs configured yet.\n");
        content.push_str("// Add outputs through the UI or manually here.\n");
        content.push_str("// Example:\n");
        content.push_str("// output \"eDP-1\" {\n");
        content.push_str("//     scale 1.0\n");
        content.push_str("// }\n");
    } else {
        for output in named_outputs {
            content.push_str(&format!(
                "output \"{}\" {{\n",
                escape_kdl_string(&output.name)
            ));

            // niri's Output accepts `off` alongside every other property (off is a
            // standalone bool flag in the grammar), so we still emit the full
            // configuration for a disabled output. This preserves scale/mode/
            // position/colors/etc. across a save+reload of a disabled output
            // instead of wiping them.
            if !output.enabled {
                content.push_str("    off\n");
            }
            {
                // `None` = omit (niri auto-guess). `Some(1.0)` must be written:
                // niri treats an unset scale as "guess from the monitor", not 1×.
                if let Some(scale) = output.scale {
                    content.push_str(&format!("    scale {}\n", format_output_scale(scale)));
                }

                // Mode with optional custom flag (v25.11+)
                if !output.mode.is_empty() {
                    if output.mode_custom {
                        content.push_str(&format!(
                            "    mode custom=true \"{}\"\n",
                            escape_kdl_string(&output.mode)
                        ));
                    } else {
                        content.push_str(&format!(
                            "    mode \"{}\"\n",
                            escape_kdl_string(&output.mode)
                        ));
                    }
                }

                // Custom modeline (v25.11+) - WARNING: can damage monitors
                if let Some(ref modeline) = output.modeline {
                    if let Some(line) = format_modeline(modeline) {
                        content.push_str(&format!("    {}\n", line));
                    } else {
                        log::warn!("invalid modeline, not writing: {:?}", modeline);
                    }
                }

                if let Some((x, y)) = output.position {
                    content.push_str(&format!("    position x={} y={}\n", x, y));
                }
                let transform_str = match output.transform {
                    crate::types::Transform::Normal => "",
                    crate::types::Transform::Rotate90 => "90",
                    crate::types::Transform::Rotate180 => "180",
                    crate::types::Transform::Rotate270 => "270",
                    crate::types::Transform::Flipped => "flipped",
                    crate::types::Transform::Flipped90 => "flipped-90",
                    crate::types::Transform::Flipped180 => "flipped-180",
                    crate::types::Transform::Flipped270 => "flipped-270",
                };
                if !transform_str.is_empty() {
                    content.push_str(&format!("    transform \"{}\"\n", transform_str));
                }
                // VRR: flag only for "on", attribute syntax for "on-demand"
                match output.vrr {
                    crate::types::VrrMode::Off => {}
                    crate::types::VrrMode::On => {
                        content.push_str("    variable-refresh-rate\n");
                    }
                    crate::types::VrrMode::OnDemand => {
                        content.push_str("    variable-refresh-rate on-demand=true\n");
                    }
                }
                if output.focus_at_startup {
                    content.push_str("    focus-at-startup\n");
                }
                if let Some(ref color) = output.background_color {
                    content.push_str(&format!("    background-color \"{}\"\n", color.to_hex()));
                }
                if let Some(ref color) = output.backdrop_color {
                    content.push_str(&format!("    backdrop-color \"{}\"\n", color.to_hex()));
                }

                // Per-output hot corners (v25.11+)
                if let Some(ref hc) = output.hot_corners {
                    content.push_str("    hot-corners {\n");
                    if hc.is_off() {
                        content.push_str("        off\n");
                    } else {
                        if hc.top_left {
                            content.push_str("        top-left\n");
                        }
                        if hc.top_right {
                            content.push_str("        top-right\n");
                        }
                        if hc.bottom_left {
                            content.push_str("        bottom-left\n");
                        }
                        if hc.bottom_right {
                            content.push_str("        bottom-right\n");
                        }
                    }
                    content.push_str("    }\n");
                }

                // Per-output layout override (v25.11+)
                if let Some(ref layout) = output.layout_override {
                    content.push_str(&generate_layout_override_kdl(layout, "    "));
                }
            }
            content.push_str("}\n\n");
        }
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::OutputConfig;

    /// Round-trip a single output through generate + parse.
    fn roundtrip_output(output: &OutputConfig) -> (String, OutputConfig) {
        let settings = OutputSettings {
            outputs: vec![output.clone()],
        };
        let kdl = generate_outputs_kdl(&settings);
        let doc: kdl::KdlDocument = kdl.parse().expect("generated outputs KDL must re-parse");
        let node = doc.get("output").expect("output node present");
        let children = node.children().expect("output has children");
        let mut loaded = OutputConfig {
            name: output.name.clone(),
            ..Default::default()
        };
        crate::config::loader::parse_output_node_children(children, &mut loaded);
        (kdl, loaded)
    }

    #[test]
    fn test_layout_override_extended_fields_round_trip() {
        use crate::types::{Color, ColorOrGradient};
        let lo = LayoutOverride {
            focus_ring_urgent: Some(ColorOrGradient::Color(Color::from_hex("#ff0000").unwrap())),
            border_urgent: Some(ColorOrGradient::Color(Color::from_hex("#00ff00").unwrap())),
            shadow_enabled: Some(true),
            shadow_inactive_color: Some(Color::from_hex("#00000050").unwrap()),
            shadow_draw_behind_window: Some(true),
            background_color: Some(Color::from_hex("#123456").unwrap()),
            empty_workspace_above_first: Some(true),
            always_center_single_column: Some(false),
            ..Default::default()
        };
        let output = OutputConfig {
            name: "DP-9".into(),
            layout_override: Some(lo.clone()),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        let got = loaded
            .layout_override
            .expect("override survives round-trip");
        assert_eq!(got.focus_ring_urgent, lo.focus_ring_urgent, "{kdl}");
        assert_eq!(got.border_urgent, lo.border_urgent, "{kdl}");
        assert_eq!(got.shadow_inactive_color, lo.shadow_inactive_color, "{kdl}");
        assert_eq!(
            got.shadow_draw_behind_window, lo.shadow_draw_behind_window,
            "{kdl}"
        );
        assert_eq!(got.background_color, lo.background_color, "{kdl}");
        assert_eq!(
            got.empty_workspace_above_first, lo.empty_workspace_above_first,
            "{kdl}"
        );
        assert_eq!(
            got.always_center_single_column, lo.always_center_single_column,
            "{kdl}"
        );
    }

    #[test]
    fn test_layout_override_gradients_round_trip() {
        use crate::types::{Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo};
        let gradient = ColorOrGradient::Gradient(Gradient {
            from: Color::from_hex("#80c8ff").unwrap(),
            to: Color::from_hex("#bbddff").unwrap(),
            angle: 90,
            relative_to: GradientRelativeTo::Window,
            color_space: ColorSpace::Srgb,
            hue_interpolation: None,
        });
        let lo = LayoutOverride {
            focus_ring_active: Some(gradient.clone()),
            focus_ring_inactive: Some(ColorOrGradient::Color(Color::from_hex("#333333").unwrap())),
            focus_ring_urgent: Some(gradient.clone()),
            border_active: Some(gradient.clone()),
            border_urgent: Some(gradient.clone()),
            ..Default::default()
        };
        let output = OutputConfig {
            name: "DP-grad".into(),
            layout_override: Some(lo.clone()),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("active-gradient"), "{kdl}");
        assert!(kdl.contains("urgent-gradient"), "{kdl}");
        let got = loaded
            .layout_override
            .expect("override survives round-trip");
        assert_eq!(got.focus_ring_active, lo.focus_ring_active, "{kdl}");
        assert_eq!(got.focus_ring_inactive, lo.focus_ring_inactive, "{kdl}");
        assert_eq!(got.focus_ring_urgent, lo.focus_ring_urgent, "{kdl}");
        assert_eq!(got.border_active, lo.border_active, "{kdl}");
        assert_eq!(got.border_urgent, lo.border_urgent, "{kdl}");
    }

    #[test]
    fn test_custom_mode_round_trip() {
        let output = OutputConfig {
            name: "eDP-2".into(),
            mode: "1920x1080@144".into(),
            mode_custom: true,
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("mode custom=true \"1920x1080@144\""), "{kdl}");
        assert_eq!(loaded.mode, "1920x1080@144");
        assert!(loaded.mode_custom);
    }

    #[test]
    fn test_plain_mode_round_trip() {
        let output = OutputConfig {
            name: "eDP-1".into(),
            mode: "1920x1080@60".into(),
            mode_custom: false,
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("mode \"1920x1080@60\""), "{kdl}");
        assert!(!kdl.contains("custom=true"));
        assert_eq!(loaded.mode, "1920x1080@60");
        assert!(!loaded.mode_custom);
    }

    #[test]
    fn test_modeline_round_trip() {
        let output = OutputConfig {
            name: "DP-1".into(),
            modeline: Some("173.00 1920 2048 2248 2576 1080 1083 1088 1120 -hsync +vsync".into()),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(
            kdl.contains(
                "modeline 173.00 1920 2048 2248 2576 1080 1083 1088 1120 \"-hsync\" \"+vsync\""
            ),
            "{kdl}"
        );
        assert_eq!(
            loaded.modeline.as_deref(),
            Some("173.00 1920 2048 2248 2576 1080 1083 1088 1120 -hsync +vsync")
        );
    }

    #[test]
    fn test_invalid_modeline_not_written() {
        let output = OutputConfig {
            name: "DP-2".into(),
            modeline: Some("garbage } 1 2".into()),
            ..Default::default()
        };
        let settings = OutputSettings {
            outputs: vec![output],
        };
        let kdl = generate_outputs_kdl(&settings);
        assert!(!kdl.contains("modeline"), "{kdl}");
        let _doc: kdl::KdlDocument = kdl.parse().expect("document must still parse");
    }

    #[test]
    fn test_position_zero_round_trip() {
        let output = OutputConfig {
            name: "DP-3".into(),
            position: Some((0, 0)),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("position x=0 y=0"), "{kdl}");
        assert_eq!(loaded.position, Some((0, 0)));
    }

    #[test]
    fn test_background_and_backdrop_color_round_trip() {
        use crate::types::Color;
        let output = OutputConfig {
            name: "DP-5".into(),
            background_color: Some(Color {
                r: 25,
                g: 25,
                b: 102,
                a: 255,
            }),
            backdrop_color: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("background-color \""), "{kdl}");
        assert!(kdl.contains("backdrop-color \""), "{kdl}");
        assert_eq!(loaded.background_color, output.background_color);
        assert_eq!(loaded.backdrop_color, output.backdrop_color);
    }

    #[test]
    fn empty_connector_name_is_not_written() {
        let settings = OutputSettings {
            outputs: vec![
                OutputConfig {
                    name: String::new(),
                    mode: "1920x1080@60".into(),
                    ..Default::default()
                },
                OutputConfig {
                    name: "   ".into(),
                    ..Default::default()
                },
                OutputConfig {
                    name: "DP-1".into(),
                    mode: "1920x1080@60".into(),
                    ..Default::default()
                },
            ],
        };
        let kdl = generate_outputs_kdl(&settings);
        assert!(
            !kdl.contains("output \"\""),
            "blank output blocks must not be saved: {kdl}"
        );
        assert!(kdl.contains("output \"DP-1\""), "{kdl}");
        assert_eq!(kdl.matches("output \"").count(), 1, "{kdl}");
    }

    #[test]
    fn test_position_auto_round_trip() {
        let output = OutputConfig {
            name: "DP-4".into(),
            position: None,
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(!kdl.contains("position "), "{kdl}");
        assert_eq!(loaded.position, None);
    }

    /// Build a LayoutOverride carrying customized tab-indicator and insert-hint
    /// sub-blocks (the fields niri exposes under `layout {}` that were previously
    /// dropped on save).
    fn override_with_tab_and_insert() -> LayoutOverride {
        use crate::config::models::{
            InsertHintSettings, TabIndicatorPosition, TabIndicatorSettings,
        };
        use crate::types::{Color, ColorOrGradient};

        let mut ti = TabIndicatorSettings {
            enabled: false,
            position: TabIndicatorPosition::Bottom,
            width: 7,
            gap: 9,
            gaps_between_tabs: 3,
            corner_radius: 4,
            length_proportion: 0.35,
            hide_when_single_tab: true,
            place_within_column: true,
            ..Default::default()
        };
        ti.use_active_color = true;
        ti.active = ColorOrGradient::Color(Color::from_hex("#112233").unwrap());

        let ih = InsertHintSettings {
            enabled: false,
            color: ColorOrGradient::Color(Color::from_hex("#abcdef80").unwrap()),
        };

        LayoutOverride {
            gaps: Some(12.0),
            tab_indicator: Some(ti),
            insert_hint: Some(ih),
            ..Default::default()
        }
    }

    fn assert_tab_and_insert_preserved(got: &LayoutOverride, kdl: &str) {
        let ti = got.tab_indicator.as_ref().expect("tab-indicator survives");
        assert!(!ti.enabled, "{kdl}");
        assert_eq!(
            ti.position,
            crate::config::models::TabIndicatorPosition::Bottom,
            "{kdl}"
        );
        assert_eq!(ti.width, 7, "{kdl}");
        assert_eq!(ti.gap, 9, "{kdl}");
        assert_eq!(ti.gaps_between_tabs, 3, "{kdl}");
        assert_eq!(ti.corner_radius, 4, "{kdl}");
        assert!((ti.length_proportion - 0.35).abs() < 1e-6, "{kdl}");
        assert!(ti.hide_when_single_tab, "{kdl}");
        assert!(ti.place_within_column, "{kdl}");
        assert!(ti.use_active_color, "{kdl}");

        let ih = got.insert_hint.as_ref().expect("insert-hint survives");
        assert!(!ih.enabled, "{kdl}");
        assert_eq!(
            ih.color,
            crate::types::ColorOrGradient::Color(
                crate::types::Color::from_hex("#abcdef80").unwrap()
            ),
            "{kdl}"
        );
    }

    #[test]
    fn layout_override_tab_indicator_and_insert_hint_per_output_round_trip() {
        let lo = override_with_tab_and_insert();
        let output = OutputConfig {
            name: "DP-7".into(),
            layout_override: Some(lo),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("tab-indicator {"), "{kdl}");
        assert!(kdl.contains("insert-hint {"), "{kdl}");
        let got = loaded.layout_override.expect("override survives");
        assert_tab_and_insert_preserved(&got, &kdl);
    }

    #[test]
    fn layout_override_tab_indicator_and_insert_hint_per_workspace_round_trip() {
        // Per-workspace overrides emit/parse via generate_layout_override_kdl and
        // parse_layout_override directly (no enclosing output block).
        let lo = override_with_tab_and_insert();
        let kdl = generate_layout_override_kdl(&lo, "");
        let doc: kdl::KdlDocument = kdl.parse().expect("override KDL must re-parse");
        let children = doc
            .get("layout")
            .and_then(|n| n.children())
            .expect("layout block");
        let got = crate::config::loader::parse_layout_override(children)
            .expect("override survives round-trip");
        assert_tab_and_insert_preserved(&got, &kdl);
    }

    #[test]
    fn layout_override_default_column_width_auto_round_trip() {
        // Empty `default-column-width {}` (niri "auto") must round-trip, not drop.
        let lo = LayoutOverride {
            default_column_width_auto: Some(true),
            ..Default::default()
        };
        let kdl = generate_layout_override_kdl(&lo, "");
        assert!(kdl.contains("default-column-width {"), "{kdl}");
        let doc: kdl::KdlDocument = kdl.parse().expect("re-parse");
        let children = doc.get("layout").and_then(|n| n.children()).unwrap();
        let got = crate::config::loader::parse_layout_override(children).expect("survives");
        assert_eq!(got.default_column_width_auto, Some(true), "{kdl}");
        assert_eq!(got.default_column_width_proportion, None);
        assert_eq!(got.default_column_width_fixed, None);
    }

    #[test]
    fn layout_override_default_column_width_float_fixed_parses() {
        // `fixed 500.5` (niri FloatOrInt) must parse as Fixed, not fall through to auto.
        let kdl = "layout {\n    default-column-width {\n        fixed 500.5\n    }\n}";
        let doc: kdl::KdlDocument = kdl.parse().unwrap();
        let children = doc.get("layout").and_then(|n| n.children()).unwrap();
        let got = crate::config::loader::parse_layout_override(children).expect("survives");
        assert_eq!(got.default_column_width_fixed, Some(500.5));
        assert_eq!(got.default_column_width_auto, None);

        // Writer must keep the fraction too.
        let written = generate_layout_override_kdl(
            &LayoutOverride {
                gaps: Some(0.5),
                strut_left: Some(1.25),
                default_column_width_fixed: Some(500.5),
                ..Default::default()
            },
            "",
        );
        assert!(written.contains("gaps 0.5"), "{written}");
        assert!(written.contains("left 1.25"), "{written}");
        assert!(written.contains("fixed 500.5"), "{written}");
    }

    #[test]
    fn disabled_output_preserves_properties_round_trip() {
        // niri accepts `off` alongside every other output property, so disabling an
        // output must not wipe its saved scale/mode/position/etc.
        use crate::types::Color;
        let output = OutputConfig {
            name: "DP-8".into(),
            enabled: false,
            scale: Some(2.0),
            mode: "1920x1080@60".into(),
            position: Some((10, 20)),
            background_color: Some(Color::from_hex("#123456").unwrap()),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(kdl.contains("off"), "{kdl}");
        assert!(kdl.contains("scale 2"), "{kdl}");
        assert!(kdl.contains("mode \"1920x1080@60\""), "{kdl}");
        assert!(kdl.contains("position x=10 y=20"), "{kdl}");
        assert!(!loaded.enabled, "{kdl}");
        assert_eq!(loaded.scale, Some(2.0), "{kdl}");
        assert_eq!(loaded.mode, "1920x1080@60", "{kdl}");
        assert_eq!(loaded.position, Some((10, 20)), "{kdl}");
        assert_eq!(loaded.background_color, output.background_color, "{kdl}");
    }

    #[test]
    fn explicit_scale_1_round_trips_and_is_written() {
        // niri unset scale = auto-guess, not 1.0. An explicit 1× must survive.
        let output = OutputConfig {
            name: "eDP-1".into(),
            scale: Some(1.0),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(
            kdl.contains("scale 1.0") || kdl.contains("scale 1\n") || kdl.contains("scale 1 "),
            "explicit 1.0 must be written, got:\n{kdl}"
        );
        assert_eq!(loaded.scale, Some(1.0), "{kdl}");
    }

    #[test]
    fn unset_scale_is_omitted_and_stays_none() {
        let output = OutputConfig {
            name: "eDP-1".into(),
            scale: None,
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_output(&output);
        assert!(
            !kdl.lines()
                .any(|line| line.trim_start().starts_with("scale ")),
            "unset scale must be omitted so niri can auto-guess, got:\n{kdl}"
        );
        assert_eq!(loaded.scale, None, "{kdl}");
    }

    #[test]
    fn cubic_bezier_whole_number_points_round_trip() {
        // Storage writes points with `{}` formatting, so 0.0/1.0 serialize as the
        // KDL integers `0`/`1`. The loader must read them back via int-or-float
        // instead of resetting whole-number points to the defaults.
        use crate::config::models::{AnimationType, EasingCurve, SingleAnimationConfig};
        let mut config = SingleAnimationConfig {
            animation_type: AnimationType::Easing,
            ..Default::default()
        };
        config.easing.duration_ms = 300;
        config.easing.curve = EasingCurve::CubicBezier {
            x1: 0.0,
            y1: 1.0,
            x2: 1.0,
            y2: 0.0,
        };
        let kdl = generate_single_animation_kdl("window-open", &config, "").expect("emits KDL");
        // Whole-number points serialize as bare KDL integers.
        assert!(kdl.contains("cubic-bezier\" 0 1 1 0"), "{kdl}");
        let doc: kdl::KdlDocument = kdl.parse().expect("re-parse");
        let children = doc.get("window-open").and_then(|n| n.children()).unwrap();
        let parsed = crate::config::loader::parse_single_animation(children);
        match parsed.easing.curve {
            EasingCurve::CubicBezier { x1, y1, x2, y2 } => {
                assert_eq!((x1, y1, x2, y2), (0.0, 1.0, 1.0, 0.0), "{kdl}");
            }
            other => panic!("expected cubic-bezier, got {other:?} ({kdl})"),
        }
    }
}
