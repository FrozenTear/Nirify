//! Animations settings message handler

use crate::config::SettingsCategory;
use crate::config::models::{AnimationType, AnimationId, EasingCurve};
use crate::messages::{AnimationsMessage, Message};
use iced::Task;

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
                        AnimationType::Spring  // Default to spring when enabled
                    } else {
                        AnimationType::Off
                    };
                }
            }
            AnimationsMessage::SetAnimationDuration(name, duration) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.easing.duration_ms = duration.clamp(50, 5000);
                }
            }
            AnimationsMessage::SetAnimationCurve(name, curve_name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.easing.curve = EasingCurve::from_kdl(&curve_name);
                }
            }
            AnimationsMessage::SetAnimationSpringDampingRatio(name, ratio) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.spring.damping_ratio = ratio.clamp(0.1, 2.0) as f64;
                }
            }
            AnimationsMessage::SetAnimationSpringEpsilon(name, epsilon) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                    anim_config.spring.epsilon = epsilon.clamp(0.0001, 1.0) as f64;
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
                        let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
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
                        let anim_config = anim_id.get_mut(&mut self.settings.animations.per_animation);
                        anim_config.custom_shader = Some(template);
                        anim_config.animation_type = AnimationType::CustomShader;
                    }
                }
            }
        }


        self.dirty_tracker.mark(SettingsCategory::Animations);
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
            "horizontal_view" | "horizontal-view" | "horizontal_view_movement" | "horizontal-view-movement" => Some(AnimationId::HorizontalViewMovement),
            "config_notification" | "config-notification" => Some(AnimationId::ConfigNotification),
            "exit_confirmation" | "exit-confirmation" => Some(AnimationId::ExitConfirmation),
            "screenshot_ui" | "screenshot-ui" => Some(AnimationId::ScreenshotUi),
            "recent_windows" | "recent-windows" => Some(AnimationId::RecentWindows),
            _ => None,
        }
    }
}
