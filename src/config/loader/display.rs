//! Display-related settings loaders
//!
//! Loads settings for animations, cursor, overview, and outputs.

use super::super::parser::{get_f64, get_i64, get_string, has_flag};
use super::helpers::{parse_color, read_kdl_file};
use crate::config::models::{
    AnimationType, EasingCurve, LayoutOverride, OutputConfig, OutputHotCorners, Settings,
    SingleAnimationConfig, SpringParams, WorkspaceShadow,
};
use crate::constants::{
    DAMPING_RATIO_MAX, DAMPING_RATIO_MIN, EASING_DURATION_MAX, EASING_DURATION_MIN, EPSILON_MAX,
    EPSILON_MIN, STIFFNESS_MAX, STIFFNESS_MIN,
};
use crate::types::{CenterFocusedColumn, Transform, VrrMode};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Type alias for animation accessor functions to reduce type complexity
type AnimationAccessor = (
    &'static str,
    fn(&mut Settings) -> &mut SingleAnimationConfig,
);

/// Parse spring parameters from a KDL node's children
fn parse_spring_params(children: &KdlDocument) -> Option<SpringParams> {
    let spring_node = children.get("spring")?;
    let mut params = SpringParams::default();

    // Parse "spring damping-ratio=1.0 stiffness=1000 epsilon=0.0001"
    for entry in spring_node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "damping-ratio" => {
                    if let Some(v) = entry.value().as_float() {
                        params.damping_ratio = v.clamp(DAMPING_RATIO_MIN, DAMPING_RATIO_MAX);
                    }
                }
                "stiffness" => {
                    if let Some(v) = entry.value().as_integer() {
                        params.stiffness = (v as i32).clamp(STIFFNESS_MIN, STIFFNESS_MAX);
                    }
                }
                "epsilon" => {
                    if let Some(v) = entry.value().as_float() {
                        params.epsilon = v.clamp(EPSILON_MIN, EPSILON_MAX);
                    }
                }
                _ => {}
            }
        }
    }
    Some(params)
}

/// Parse a single animation configuration from KDL
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_single_animation(children: &KdlDocument) -> SingleAnimationConfig {
    let mut config = SingleAnimationConfig::default();

    // Check for "off"
    if has_flag(children, &["off"]) {
        config.animation_type = AnimationType::Off;
        return config;
    }

    // Check for custom-shader (raw GLSL code)
    // Only supported for window-open, window-close, window-resize
    if let Some(shader_node) = children.get("custom-shader") {
        if let Some(first_entry) = shader_node.entries().first() {
            if let Some(code) = first_entry.value().as_string() {
                config.animation_type = AnimationType::CustomShader;
                config.custom_shader = Some(code.to_string());
                return config;
            }
        }
    }

    // Check for spring
    if let Some(spring) = parse_spring_params(children) {
        config.animation_type = AnimationType::Spring;
        config.spring = spring;
        return config;
    }

    // Check for easing (duration-ms + curve)
    if let Some(duration) = get_i64(children, &["duration-ms"]) {
        config.animation_type = AnimationType::Easing;
        config.easing.duration_ms =
            (duration as i32).clamp(EASING_DURATION_MIN, EASING_DURATION_MAX);

        // Parse curve - can be preset (e.g., "ease-out-quad") or cubic-bezier
        if let Some(curve_node) = children.get("curve") {
            let entries = curve_node.entries();
            if let Some(first) = entries.first() {
                if let Some(curve_str) = first.value().as_string() {
                    if curve_str == "cubic-bezier" && entries.len() >= 5 {
                        // Parse cubic-bezier control points: curve "cubic-bezier" x1 y1 x2 y2
                        let x1 = entries
                            .get(1)
                            .and_then(|e| e.value().as_float())
                            .unwrap_or(0.25);
                        let y1 = entries
                            .get(2)
                            .and_then(|e| e.value().as_float())
                            .unwrap_or(0.1);
                        let x2 = entries
                            .get(3)
                            .and_then(|e| e.value().as_float())
                            .unwrap_or(0.25);
                        let y2 = entries
                            .get(4)
                            .and_then(|e| e.value().as_float())
                            .unwrap_or(1.0);
                        config.easing.curve = EasingCurve::CubicBezier { x1, y1, x2, y2 };
                    } else {
                        // Preset curve
                        config.easing.curve = EasingCurve::from_kdl(curve_str);
                    }
                }
            }
        }
        return config;
    }

    config // AnimationType::Default
}

/// Parse a layout override block from KDL children
pub fn parse_layout_override(layout_children: &KdlDocument) -> Option<LayoutOverride> {
    use crate::config::models::{DefaultColumnDisplay, PresetHeight, PresetWidth};
    use crate::types::{Color, ColorOrGradient};

    let mut layout = LayoutOverride::default();

    // Parse gaps (single value in niri)
    if let Some(gaps_node) = layout_children.get("gaps") {
        if let Some(entry) = gaps_node.entries().first() {
            if entry.name().is_none() {
                // Positional argument: gaps 16
                if let Some(v) = entry.value().as_float() {
                    layout.gaps = Some(v as f32);
                } else if let Some(v) = entry.value().as_integer() {
                    layout.gaps = Some(v as f32);
                }
            }
        }
    }

    // Parse struts block
    if let Some(struts_node) = layout_children.get("struts") {
        if let Some(struts_children) = struts_node.children() {
            if let Some(v) = get_i64(struts_children, &["left"]) {
                layout.strut_left = Some(v as f32);
            }
            if let Some(v) = get_i64(struts_children, &["right"]) {
                layout.strut_right = Some(v as f32);
            }
            if let Some(v) = get_i64(struts_children, &["top"]) {
                layout.strut_top = Some(v as f32);
            }
            if let Some(v) = get_i64(struts_children, &["bottom"]) {
                layout.strut_bottom = Some(v as f32);
            }
        }
    }

    // center-focused-column
    if let Some(v) = get_string(layout_children, &["center-focused-column"]) {
        layout.center_focused_column = CenterFocusedColumn::parse_kdl(&v);
    }

    // always-center-single-column
    if has_flag(layout_children, &["always-center-single-column"]) {
        layout.always_center_single_column = Some(true);
    }

    // default-column-display
    if let Some(v) = get_string(layout_children, &["default-column-display"]) {
        layout.default_column_display = match v.as_str() {
            "tabbed" => Some(DefaultColumnDisplay::Tabbed),
            "normal" => Some(DefaultColumnDisplay::Normal),
            _ => None,
        };
    }

    // default-column-width
    if let Some(dcw_node) = layout_children.get("default-column-width") {
        if let Some(dcw_children) = dcw_node.children() {
            if let Some(v) = get_f64(dcw_children, &["proportion"]) {
                layout.default_column_width_proportion = Some(v as f32);
            }
            if let Some(v) = get_i64(dcw_children, &["fixed"]) {
                layout.default_column_width_fixed = Some(v as i32);
            }
        }
    }

    // preset-column-widths
    if let Some(pcw_node) = layout_children.get("preset-column-widths") {
        if let Some(pcw_children) = pcw_node.children() {
            let mut presets = Vec::new();
            for node in pcw_children.nodes() {
                match node.name().value() {
                    "proportion" => {
                        if let Some(v) = node.entries().first().and_then(|e| e.value().as_float()) {
                            presets.push(PresetWidth::Proportion(v as f32));
                        }
                    }
                    "fixed" => {
                        if let Some(v) = node.entries().first().and_then(|e| e.value().as_integer())
                        {
                            presets.push(PresetWidth::Fixed(v as i32));
                        }
                    }
                    _ => {}
                }
            }
            if !presets.is_empty() {
                layout.preset_column_widths = Some(presets);
            }
        }
    }

    // preset-window-heights
    if let Some(pwh_node) = layout_children.get("preset-window-heights") {
        if let Some(pwh_children) = pwh_node.children() {
            let mut presets = Vec::new();
            for node in pwh_children.nodes() {
                match node.name().value() {
                    "proportion" => {
                        if let Some(v) = node.entries().first().and_then(|e| e.value().as_float()) {
                            presets.push(PresetHeight::Proportion(v as f32));
                        }
                    }
                    "fixed" => {
                        if let Some(v) = node.entries().first().and_then(|e| e.value().as_integer())
                        {
                            presets.push(PresetHeight::Fixed(v as i32));
                        }
                    }
                    _ => {}
                }
            }
            if !presets.is_empty() {
                layout.preset_window_heights = Some(presets);
            }
        }
    }

    // focus-ring
    if let Some(fr_node) = layout_children.get("focus-ring") {
        if let Some(fr_children) = fr_node.children() {
            if has_flag(fr_children, &["off"]) {
                layout.focus_ring_enabled = Some(false);
            } else {
                layout.focus_ring_enabled = Some(true);
            }
            if let Some(v) = get_i64(fr_children, &["width"]) {
                layout.focus_ring_width = Some(v as i32);
            }
            if let Some(s) = get_string(fr_children, &["active-color"]) {
                if let Some(c) = Color::from_hex(&s) {
                    layout.focus_ring_active = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(s) = get_string(fr_children, &["inactive-color"]) {
                if let Some(c) = Color::from_hex(&s) {
                    layout.focus_ring_inactive = Some(ColorOrGradient::Color(c));
                }
            }
        }
    }

    // border
    if let Some(b_node) = layout_children.get("border") {
        if let Some(b_children) = b_node.children() {
            if has_flag(b_children, &["off"]) {
                layout.border_enabled = Some(false);
            } else {
                layout.border_enabled = Some(true);
            }
            if let Some(v) = get_i64(b_children, &["width"]) {
                layout.border_width = Some(v as i32);
            }
            if let Some(s) = get_string(b_children, &["active-color"]) {
                if let Some(c) = Color::from_hex(&s) {
                    layout.border_active = Some(ColorOrGradient::Color(c));
                }
            }
            if let Some(s) = get_string(b_children, &["inactive-color"]) {
                if let Some(c) = Color::from_hex(&s) {
                    layout.border_inactive = Some(ColorOrGradient::Color(c));
                }
            }
        }
    }

    // shadow
    if let Some(s_node) = layout_children.get("shadow") {
        if let Some(s_children) = s_node.children() {
            if has_flag(s_children, &["off"]) {
                layout.shadow_enabled = Some(false);
            } else {
                layout.shadow_enabled = Some(true);
            }
            if let Some(v) = get_i64(s_children, &["softness"]) {
                layout.shadow_softness = Some(v as i32);
            }
            if let Some(v) = get_i64(s_children, &["spread"]) {
                layout.shadow_spread = Some(v as i32);
            }
            // offset can be: offset x=0 y=5
            if let Some(offset_node) = s_children.get("offset") {
                for entry in offset_node.entries() {
                    if let Some(name) = entry.name() {
                        match name.value() {
                            "x" => {
                                if let Some(v) = entry.value().as_integer() {
                                    layout.shadow_offset_x = Some(v as i32);
                                }
                            }
                            "y" => {
                                if let Some(v) = entry.value().as_integer() {
                                    layout.shadow_offset_y = Some(v as i32);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            if let Some(s) = get_string(s_children, &["color"]) {
                if let Some(c) = Color::from_hex(&s) {
                    layout.shadow_color = Some(c);
                }
            }
        }
    }

    // Return Some only if any field was set
    if layout.has_any() {
        Some(layout)
    } else {
        None
    }
}

/// Parse output hot corners from KDL children (v25.11+)
fn parse_output_hot_corners(hc_children: &KdlDocument) -> OutputHotCorners {
    let mut hc = OutputHotCorners::default();

    // Check for "off" flag
    if has_flag(hc_children, &["off"]) {
        hc.enabled = Some(false);
        return hc;
    }

    // Check individual corners
    if has_flag(hc_children, &["top-left"]) {
        hc.top_left = true;
    }
    if has_flag(hc_children, &["top-right"]) {
        hc.top_right = true;
    }
    if has_flag(hc_children, &["bottom-left"]) {
        hc.bottom_left = true;
    }
    if has_flag(hc_children, &["bottom-right"]) {
        hc.bottom_right = true;
    }

    hc
}

/// Parse workspace shadow from overview children (v25.05+)
fn parse_workspace_shadow(shadow_children: &KdlDocument) -> WorkspaceShadow {
    let mut shadow = WorkspaceShadow::default();

    // Check for "off" flag
    if has_flag(shadow_children, &["off"]) {
        shadow.enabled = false;
        return shadow;
    }

    if let Some(v) = get_i64(shadow_children, &["softness"]) {
        shadow.softness = v as i32;
    }
    if let Some(v) = get_i64(shadow_children, &["spread"]) {
        shadow.spread = v as i32;
    }

    // Parse offset x=N y=N
    if let Some(offset_node) = shadow_children.get("offset") {
        for entry in offset_node.entries() {
            if let Some(name) = entry.name() {
                if let Some(val) = entry.value().as_integer() {
                    match name.value() {
                        "x" => shadow.offset_x = val as i32,
                        "y" => shadow.offset_y = val as i32,
                        _ => {}
                    }
                }
            }
        }
    }

    if let Some(v) = get_string(shadow_children, &["color"]) {
        if let Some(color) = parse_color(&v) {
            shadow.color = color;
        }
    }

    shadow
}

/// Parse animations from animations node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_animations_from_children(a_children: &KdlDocument, settings: &mut Settings) {
    // Check if animations are off globally
    if has_flag(a_children, &["off"]) {
        settings.animations.enabled = false;
    }

    if let Some(v) = get_f64(a_children, &["slowdown"]) {
        settings.animations.slowdown = v;
    }

    // Parse per-animation settings
    let animation_names: [AnimationAccessor; 11] = [
        ("workspace-switch", |s| {
            &mut s.animations.per_animation.workspace_switch
        }),
        ("window-open", |s| {
            &mut s.animations.per_animation.window_open
        }),
        ("window-close", |s| {
            &mut s.animations.per_animation.window_close
        }),
        ("horizontal-view-movement", |s| {
            &mut s.animations.per_animation.horizontal_view_movement
        }),
        ("window-movement", |s| {
            &mut s.animations.per_animation.window_movement
        }),
        ("window-resize", |s| {
            &mut s.animations.per_animation.window_resize
        }),
        ("config-notification-open-close", |s| {
            &mut s.animations.per_animation.config_notification_open_close
        }),
        ("exit-confirmation-open-close", |s| {
            &mut s.animations.per_animation.exit_confirmation_open_close
        }),
        ("screenshot-ui-open", |s| {
            &mut s.animations.per_animation.screenshot_ui_open
        }),
        ("overview-open-close", |s| {
            &mut s.animations.per_animation.overview_open_close
        }),
        ("recent-windows-close", |s| {
            &mut s.animations.per_animation.recent_windows_close
        }),
    ];

    for (name, getter) in animation_names {
        if let Some(anim_node) = a_children.get(name) {
            if let Some(anim_children) = anim_node.children() {
                *getter(settings) = parse_single_animation(anim_children);
            }
        }
    }
}

/// Load animation settings from KDL file including per-animation configurations
pub fn load_animations(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    if let Some(anim) = doc.get("animations") {
        if let Some(a_children) = anim.children() {
            parse_animations_from_children(a_children, settings);
        }
    }

    debug!("Loaded animation settings from {:?}", path);
}

/// Parse cursor settings from cursor node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_cursor_from_children(c_children: &KdlDocument, settings: &mut Settings) {
    if let Some(v) = get_string(c_children, &["xcursor-theme"]) {
        settings.cursor.theme = v;
    }
    if let Some(v) = get_i64(c_children, &["xcursor-size"]) {
        settings.cursor.size = v as i32;
    }
    if has_flag(c_children, &["hide-when-typing"]) {
        settings.cursor.hide_when_typing = true;
    }
    if let Some(v) = get_i64(c_children, &["hide-after-inactive-ms"]) {
        settings.cursor.hide_after_inactive_ms = Some(v as i32);
    }
}

/// Load cursor settings from KDL file
pub fn load_cursor(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    if let Some(cursor) = doc.get("cursor") {
        if let Some(c_children) = cursor.children() {
            parse_cursor_from_children(c_children, settings);
        }
    }

    debug!("Loaded cursor settings from {:?}", path);
}

/// Parse overview settings from overview node children
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_overview_from_children(o_children: &KdlDocument, settings: &mut Settings) {
    if let Some(v) = get_f64(o_children, &["zoom"]) {
        settings.overview.zoom = v;
    }
    if let Some(v) = get_string(o_children, &["backdrop-color"]) {
        settings.overview.backdrop_color = parse_color(&v);
    }

    // Workspace shadow
    if let Some(ws_shadow_node) = o_children.get("workspace-shadow") {
        if let Some(ws_children) = ws_shadow_node.children() {
            settings.overview.workspace_shadow = Some(parse_workspace_shadow(ws_children));
        }
    }
}

/// Load overview settings from KDL file
pub fn load_overview(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    if let Some(overview) = doc.get("overview") {
        if let Some(o_children) = overview.children() {
            parse_overview_from_children(o_children, settings);
        }
    }

    debug!("Loaded overview settings from {:?}", path);
}

/// Parse output settings into an OutputConfig
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_output_node_children(o_children: &KdlDocument, output: &mut OutputConfig) {
    // Check if off
    if has_flag(o_children, &["off"]) {
        output.enabled = false;
    }

    if let Some(v) = get_f64(o_children, &["scale"]) {
        output.scale = v;
    }

    // Mode - check for custom flag
    if let Some(mode_node) = o_children.get("mode") {
        // Get the mode string from first argument
        if let Some(first_entry) = mode_node.entries().first() {
            if first_entry.name().is_none() {
                if let Some(mode_str) = first_entry.value().as_string() {
                    output.mode = mode_str.to_string();
                }
            }
        }
        // Check for custom=true flag
        for entry in mode_node.entries() {
            if let Some(name) = entry.name() {
                if name.value() == "custom" {
                    if let Some(v) = entry.value().as_bool() {
                        output.mode_custom = v;
                    }
                }
            }
        }
    }

    // Modeline
    if let Some(v) = get_string(o_children, &["modeline"]) {
        output.modeline = Some(v);
    }

    // Position
    if let Some(pos) = o_children.get("position") {
        for entry in pos.entries() {
            if let Some(name) = entry.name() {
                if let Some(val) = entry.value().as_integer() {
                    match name.value() {
                        "x" => output.position_x = val as i32,
                        "y" => output.position_y = val as i32,
                        _ => {}
                    }
                }
            }
        }
    }

    if let Some(v) = get_string(o_children, &["transform"]) {
        output.transform = match v.as_str() {
            "90" => Transform::Rotate90,
            "180" => Transform::Rotate180,
            "270" => Transform::Rotate270,
            "flipped" => Transform::Flipped,
            "flipped-90" => Transform::Flipped90,
            "flipped-180" => Transform::Flipped180,
            "flipped-270" => Transform::Flipped270,
            _ => Transform::Normal,
        };
    }

    // VRR: can be a flag (variable-refresh-rate) or have on-demand=true
    if has_flag(o_children, &["variable-refresh-rate"]) {
        output.vrr = VrrMode::On;
    } else if let Some(v) = get_string(o_children, &["variable-refresh-rate"]) {
        output.vrr = match v.as_str() {
            "on" => VrrMode::On,
            "on-demand" => VrrMode::OnDemand,
            _ => VrrMode::Off,
        };
    } else if let Some(vrr_node) = o_children.get("variable-refresh-rate") {
        // Check for on-demand=true attribute syntax
        for entry in vrr_node.entries() {
            if let Some(name) = entry.name() {
                if name.value() == "on-demand" {
                    if let Some(val) = entry.value().as_bool() {
                        if val {
                            output.vrr = VrrMode::OnDemand;
                        }
                    }
                }
            }
        }
        // If no on-demand attribute but node exists, it's On
        if output.vrr == VrrMode::Off {
            output.vrr = VrrMode::On;
        }
    }

    if has_flag(o_children, &["focus-at-startup"]) {
        output.focus_at_startup = true;
    }

    if let Some(v) = get_string(o_children, &["backdrop-color"]) {
        output.backdrop_color = parse_color(&v);
    }

    // Hot corners (per-output)
    if let Some(hc_node) = o_children.get("hot-corners") {
        if let Some(hc_children) = hc_node.children() {
            output.hot_corners = Some(parse_output_hot_corners(hc_children));
        }
    }

    // Layout override (per-output)
    if let Some(layout_node) = o_children.get("layout") {
        if let Some(layout_children) = layout_node.children() {
            output.layout_override = parse_layout_override(layout_children);
        }
    }
}

/// Load output settings from KDL file
pub fn load_outputs(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    settings.outputs.outputs.clear();

    // Find all output nodes
    for node in doc.nodes() {
        if node.name().value() == "output" {
            // Get output name from first argument
            let name = node
                .entries()
                .first()
                .and_then(|e| e.value().as_string())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let mut output = OutputConfig {
                name,
                ..Default::default()
            };

            if let Some(o_children) = node.children() {
                parse_output_node_children(o_children, &mut output);
            }

            settings.outputs.outputs.push(output);
        }
    }

    debug!(
        "Loaded {} outputs from {:?}",
        settings.outputs.outputs.len(),
        path
    );
}
