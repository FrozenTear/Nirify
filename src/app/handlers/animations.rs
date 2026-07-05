//! Animations settings message handler

use crate::config::models::{AnimationId, AnimationType, EasingCurve};
use crate::config::SettingsCategory;
use crate::constants::{
    DAMPING_RATIO_MAX, DAMPING_RATIO_MIN, EASING_DURATION_MAX, EASING_DURATION_MIN, EPSILON_MAX,
    EPSILON_MIN, STIFFNESS_MAX, STIFFNESS_MIN,
};
use crate::messages::{AnimationsMessage, Message};
use iced::Task;

/// Clamp a cubic-bezier X control point to niri's valid range.
///
/// niri decodes bezier X coordinates as `FloatOrInt<0, 1>` (the X axis is a
/// time frame and cannot be negative or larger than 1) and hard-errors on
/// out-of-range values, so an unclamped X would make Nirify emit a config niri
/// rejects. Y coordinates are unbounded in niri and are not clamped.
fn clamp_bezier_x(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

impl super::super::App {
    /// Updates animation settings
    pub(in crate::app) fn update_animations(&mut self, msg: AnimationsMessage) -> Task<Message> {
        match msg {
            AnimationsMessage::ToggleSlowdown(enabled) => {
                // Toggle between slowdown factor and normal speed (1.0)
                if enabled {
                    // Enable slowdown (if it's at 1.0, set to default 3.0)
                    if (self.settings.animations.slowdown - 1.0).abs() < 0.01 {
                        self.settings.animations.slowdown = 3.0;
                    }
                } else {
                    // Disable slowdown (set to 1.0 = normal speed)
                    self.settings.animations.slowdown = 1.0;
                }
            }
            AnimationsMessage::SetSlowdownFactor(value) => {
                self.settings.animations.slowdown = value.clamp(0.1, 10.0) as f64;
            }
            AnimationsMessage::SetAnimationEnabled(name, enabled) => {
                // Parse animation name to AnimationId
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.animation_type = if enabled {
                        AnimationType::Default // niri's per-animation default when enabled
                    } else {
                        AnimationType::Off
                    };
                }
            }
            AnimationsMessage::SetAnimationDuration(name, duration) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.easing.duration_ms =
                        duration.clamp(EASING_DURATION_MIN, EASING_DURATION_MAX);
                }
            }
            AnimationsMessage::SetAnimationCurve(name, curve_name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    // `cubic-bezier` isn't a named preset; seed default control
                    // points so the per-point editor can refine them.
                    anim_config.easing.curve = if curve_name == "cubic-bezier" {
                        EasingCurve::from_index(4)
                    } else {
                        EasingCurve::from_kdl(&curve_name)
                    };
                }
            }
            AnimationsMessage::SetAnimationBezier(name, x1, y1, x2, y2) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    // niri decodes the X control points as FloatOrInt<0, 1> (the X
                    // axis is a time frame, so it cannot be negative or exceed 1) and
                    // hard-errors otherwise. Clamp X here so we never emit a config
                    // niri rejects. Y is unbounded in niri (FloatOrInt<i32::MIN, i32::MAX>).
                    anim_config.easing.curve = EasingCurve::CubicBezier {
                        x1: clamp_bezier_x(x1),
                        y1,
                        x2: clamp_bezier_x(x2),
                        y2,
                    };
                }
            }
            AnimationsMessage::SetAnimationSpringDampingRatio(name, ratio) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.spring.damping_ratio =
                        (ratio as f64).clamp(DAMPING_RATIO_MIN, DAMPING_RATIO_MAX);
                }
            }
            AnimationsMessage::SetAnimationSpringStiffness(name, stiffness) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.spring.stiffness = stiffness.clamp(STIFFNESS_MIN, STIFFNESS_MAX);
                }
            }
            AnimationsMessage::SetAnimationSpringEpsilon(name, epsilon) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.spring.epsilon = (epsilon as f64).clamp(EPSILON_MIN, EPSILON_MAX);
                }
            }
            AnimationsMessage::SetAnimationType(name, type_index) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.animation_type = match type_index {
                        0 => AnimationType::Default,
                        1 => AnimationType::Off,
                        2 => AnimationType::Spring,
                        3 => AnimationType::Easing,
                        4 if anim_id.supports_custom_shader() => AnimationType::CustomShader,
                        _ => AnimationType::Default,
                    };
                }
            }
            AnimationsMessage::SetCustomShader(name, code) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    if anim_id.supports_custom_shader() {
                        let anim_config =
                            anim_id.get_mut(&mut self.settings.animations.per_animation);
                        anim_config.custom_shader = Some(code);
                        anim_config.animation_type = AnimationType::CustomShader;
                    }
                }
            }
            AnimationsMessage::ClearCustomShader(name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.custom_shader = None;
                    // Revert to default if clearing shader
                    if anim_config.animation_type == AnimationType::CustomShader {
                        anim_config.animation_type = AnimationType::Default;
                    }
                }
            }
            AnimationsMessage::InsertShaderTemplate(name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    if let Some(func_name) = anim_id.shader_function_name() {
                        let template = format!(
                            r#"vec4 {}(vec3 coords_geo, vec3 size_geo) {{
    float progress = niri_clamped_progress;
    // Your GLSL code here
    return vec4(1.0);
}}"#,
                            func_name
                        );
                        let anim_config =
                            anim_id.get_mut(&mut self.settings.animations.per_animation);
                        anim_config.custom_shader = Some(template);
                        anim_config.animation_type = AnimationType::CustomShader;
                    }
                }
            }
        }

        self.save.dirty_tracker.mark(SettingsCategory::Animations);
        self.mark_changed();

        Task::none()
    }

    /// Helper to parse animation name string to AnimationId
    pub(in crate::app) fn parse_animation_name(name: &str) -> Option<AnimationId> {
        match name.to_lowercase().as_str() {
            "workspace_switch" | "workspace-switch" => Some(AnimationId::WorkspaceSwitch),
            "overview" => Some(AnimationId::Overview),
            "window_open" | "window-open" => Some(AnimationId::WindowOpen),
            "window_close" | "window-close" => Some(AnimationId::WindowClose),
            "window_movement" | "window-movement" => Some(AnimationId::WindowMovement),
            "window_resize" | "window-resize" => Some(AnimationId::WindowResize),
            "horizontal_view"
            | "horizontal-view"
            | "horizontal_view_movement"
            | "horizontal-view-movement" => Some(AnimationId::HorizontalViewMovement),
            "config_notification" | "config-notification" => Some(AnimationId::ConfigNotification),
            "exit_confirmation" | "exit-confirmation" => Some(AnimationId::ExitConfirmation),
            "screenshot_ui" | "screenshot-ui" => Some(AnimationId::ScreenshotUi),
            "recent_windows" | "recent-windows" => Some(AnimationId::RecentWindows),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clamp_bezier_x;

    #[test]
    fn bezier_x_clamped_to_niri_range() {
        // Out-of-range X control points are clamped into niri's FloatOrInt<0, 1>
        // range so we never emit a cubic-bezier niri would reject.
        assert_eq!(clamp_bezier_x(2.0), 1.0);
        assert_eq!(clamp_bezier_x(-0.5), 0.0);
        // In-range values pass through unchanged.
        assert_eq!(clamp_bezier_x(0.0), 0.0);
        assert_eq!(clamp_bezier_x(1.0), 1.0);
        assert!((clamp_bezier_x(0.42) - 0.42).abs() < f64::EPSILON);
    }
}
