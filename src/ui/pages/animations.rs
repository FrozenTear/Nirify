//! Animation settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::models::AnimationType;
use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the animations settings page
pub fn animations_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let animations = settings.animations;

    let enabled = RwSignal::new(animations.enabled);
    let slowdown = RwSignal::new(animations.slowdown);

    // Per-animation enabled states
    let workspace_switch =
        RwSignal::new(animations.per_animation.workspace_switch.animation_type != AnimationType::Off);
    let window_open =
        RwSignal::new(animations.per_animation.window_open.animation_type != AnimationType::Off);
    let window_close =
        RwSignal::new(animations.per_animation.window_close.animation_type != AnimationType::Off);
    let window_movement =
        RwSignal::new(animations.per_animation.window_movement.animation_type != AnimationType::Off);
    let window_resize =
        RwSignal::new(animations.per_animation.window_resize.animation_type != AnimationType::Off);

    // Callbacks for auto-save
    let on_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.animations.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_slowdown = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.animations.slowdown = val);
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_workspace_switch = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| {
                s.animations.per_animation.workspace_switch.animation_type = if val {
                    AnimationType::Easing
                } else {
                    AnimationType::Off
                };
            });
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_window_open = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| {
                s.animations.per_animation.window_open.animation_type = if val {
                    AnimationType::Easing
                } else {
                    AnimationType::Off
                };
            });
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_window_close = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| {
                s.animations.per_animation.window_close.animation_type = if val {
                    AnimationType::Easing
                } else {
                    AnimationType::Off
                };
            });
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_window_movement = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| {
                s.animations.per_animation.window_movement.animation_type = if val {
                    AnimationType::Easing
                } else {
                    AnimationType::Off
                };
            });
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    let on_window_resize = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| {
                s.animations.per_animation.window_resize.animation_type = if val {
                    AnimationType::Easing
                } else {
                    AnimationType::Off
                };
            });
            state.mark_dirty_and_save(SettingsCategory::Animations);
        })
    };

    Stack::vertical((
        section(
            "Global",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable animations",
                    Some("Turn on all window animations"),
                    enabled,
                    Some(on_enabled),
                ),
                slider_row_with_callback(
                    "Animation speed",
                    Some("Global speed multiplier (1.0 = normal)"),
                    slowdown,
                    0.1,
                    10.0,
                    0.1,
                    "x",
                    Some(on_slowdown),
                ),
            )),
        ),
        section(
            "Animation Types",
            Stack::vertical((
                toggle_row_with_callback(
                    "Workspace switch",
                    Some("Animate when switching workspaces"),
                    workspace_switch,
                    Some(on_workspace_switch),
                ),
                toggle_row_with_callback(
                    "Window open",
                    Some("Animate window opening"),
                    window_open,
                    Some(on_window_open),
                ),
                toggle_row_with_callback(
                    "Window close",
                    Some("Animate window closing"),
                    window_close,
                    Some(on_window_close),
                ),
                toggle_row_with_callback(
                    "Window movement",
                    Some("Animate window dragging"),
                    window_movement,
                    Some(on_window_movement),
                ),
                toggle_row_with_callback(
                    "Window resize",
                    Some("Animate window resizing"),
                    window_resize,
                    Some(on_window_resize),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
