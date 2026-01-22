//! Animations settings message handler

use crate::config::SettingsCategory;
use crate::config::models::{AnimationType, AnimationId, EasingCurve};
use crate::messages::{AnimationsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates animation settings
    pub(in crate::app) fn update_animations(&mut self, msg: AnimationsMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            AnimationsMessage::ToggleSlowdown(enabled) => {
                // Toggle between slowdown factor and normal speed (1.0)
                if enabled {
                    // Enable slowdown (if it's at 1.0, set to default 3.0)
                    if (settings.animations.slowdown - 1.0).abs() < 0.01 {
                        settings.animations.slowdown = 3.0;
                    }
                } else {
                    // Disable slowdown (set to 1.0 = normal speed)
                    settings.animations.slowdown = 1.0;
                }
            }
            AnimationsMessage::SetSlowdownFactor(value) => {
                settings.animations.slowdown = value.clamp(0.1, 10.0) as f64;
            }
            AnimationsMessage::SetAnimationEnabled(name, enabled) => {
                // Parse animation name to AnimationId
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.animation_type = if enabled {
                        AnimationType::Spring  // Default to spring when enabled
                    } else {
                        AnimationType::Off
                    };
                }
            }
            AnimationsMessage::SetAnimationDuration(name, duration) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.easing.duration_ms = duration.clamp(50, 5000);
                }
            }
            AnimationsMessage::SetAnimationCurve(name, curve_name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.easing.curve = EasingCurve::from_kdl(&curve_name);
                }
            }
            AnimationsMessage::SetAnimationSpringDampingRatio(name, ratio) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.spring.damping_ratio = ratio.clamp(0.1, 2.0) as f64;
                }
            }
            AnimationsMessage::SetAnimationSpringEpsilon(name, epsilon) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.spring.epsilon = epsilon.clamp(0.0001, 1.0) as f64;
                }
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Animations);
        self.save_manager.mark_changed();

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
