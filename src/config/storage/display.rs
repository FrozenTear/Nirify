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
        AnimationType::Off => Some(format!("{}{} {{ off }}\n", indent, name)),
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
                format!("curve \"{}\"", easing.curve.to_kdl().unwrap_or("ease-out-quad"))
            };
            Some(format!(
                "{}{} {{\n{}    duration-ms {}\n{}    {}\n{}}}\n",
                indent, name, indent, easing.duration_ms, indent, curve_str, indent
            ))
        }
    }
}

/// Generate animations.kdl content
pub fn generate_animations_kdl(settings: &AnimationSettings) -> String {
    // Pre-allocate for animation config with per-animation settings
    let mut content = String::with_capacity(2048);
    content.push_str("// Animation settings - managed by niri-settings-rust\n\nanimations {\n");

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
    content.push_str("// Cursor settings - managed by niri-settings-rust\n\ncursor {\n");

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
    content.push_str("// Overview settings - managed by niri-settings-rust\n\noverview {\n");

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
    let mut content = String::with_capacity(256);
    let inner_indent = format!("{}    ", indent);

    content.push_str(&format!("{}layout {{\n", indent));

    // Gaps - niri uses a single value
    if let Some(gaps) = layout.gaps_inner.or(layout.gaps_outer) {
        content.push_str(&format!("{}gaps {}\n", inner_indent, gaps as i32));
    }

    // Struts - use block format like global layout
    let has_struts = layout.strut_left.is_some()
        || layout.strut_right.is_some()
        || layout.strut_top.is_some()
        || layout.strut_bottom.is_some();
    if has_struts {
        content.push_str(&format!("{}struts {{\n", inner_indent));
        let strut_indent = format!("{}    ", inner_indent);
        if let Some(left) = layout.strut_left {
            content.push_str(&format!("{}left {}\n", strut_indent, left as i32));
        }
        if let Some(right) = layout.strut_right {
            content.push_str(&format!("{}right {}\n", strut_indent, right as i32));
        }
        if let Some(top) = layout.strut_top {
            content.push_str(&format!("{}top {}\n", strut_indent, top as i32));
        }
        if let Some(bottom) = layout.strut_bottom {
            content.push_str(&format!("{}bottom {}\n", strut_indent, bottom as i32));
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

    // always-center-single-column
    if layout.always_center_single_column == Some(true) {
        content.push_str(&format!("{}always-center-single-column\n", inner_indent));
    }

    content.push_str(&format!("{}}}\n", indent));
    content
}

/// Generate outputs.kdl from output settings
pub fn generate_outputs_kdl(settings: &OutputSettings) -> String {
    // Pre-allocate ~1.5KB for outputs (multiple displays with new features)
    let mut content = String::with_capacity(1536);
    content.push_str("// Output/Display settings - managed by niri-settings-rust\n\n");

    if settings.outputs.is_empty() {
        content.push_str("// No outputs configured yet.\n");
        content.push_str("// Add outputs through the UI or manually here.\n");
        content.push_str("// Example:\n");
        content.push_str("// output \"eDP-1\" {\n");
        content.push_str("//     scale 1.0\n");
        content.push_str("// }\n");
    } else {
        for output in &settings.outputs {
            content.push_str(&format!(
                "output \"{}\" {{\n",
                escape_kdl_string(&output.name)
            ));

            if !output.enabled {
                content.push_str("    off\n");
            } else {
                if (output.scale - 1.0).abs() > 0.001 {
                    content.push_str(&format!("    scale {:.2}\n", output.scale));
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
                    content.push_str(&format!("    modeline {}\n", modeline));
                }

                if output.position_x != 0 || output.position_y != 0 {
                    content.push_str(&format!(
                        "    position x={} y={}\n",
                        output.position_x, output.position_y
                    ));
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
