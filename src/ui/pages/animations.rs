//! Animation settings page

use freya::prelude::*;

use crate::config::models::AnimationType;
use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the animations settings page
pub fn animations_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let animations = settings.animations;

    // Per-animation enabled states
    let workspace_switch_enabled =
        animations.per_animation.workspace_switch.animation_type != AnimationType::Off;
    let window_open_enabled =
        animations.per_animation.window_open.animation_type != AnimationType::Off;
    let window_close_enabled =
        animations.per_animation.window_close.animation_type != AnimationType::Off;
    let window_movement_enabled =
        animations.per_animation.window_movement.animation_type != AnimationType::Off;
    let window_resize_enabled =
        animations.per_animation.window_resize.animation_type != AnimationType::Off;

    let state_enabled = state.clone();
    let state_slowdown = state.clone();
    let state_ws_switch = state.clone();
    let state_win_open = state.clone();
    let state_win_close = state.clone();
    let state_win_move = state.clone();
    let state_win_resize = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Global section
        .child(section(
            "Global",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable animations",
                    "Turn on all window animations",
                    animations.enabled,
                    move |val| {
                        state_enabled.update_settings(|s| s.animations.enabled = val);
                        state_enabled.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                ))
                .child(slider_row(
                    "Animation speed",
                    "Global speed multiplier (1.0 = normal)",
                    animations.slowdown,
                    0.1,
                    10.0,
                    "x",
                    move |val| {
                        state_slowdown.update_settings(|s| s.animations.slowdown = val);
                        state_slowdown.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                )),
        ))
        // Animation Types section
        .child(section(
            "Animation Types",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Workspace switch",
                    "Animate when switching workspaces",
                    workspace_switch_enabled,
                    move |val| {
                        state_ws_switch.update_settings(|s| {
                            s.animations.per_animation.workspace_switch.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        state_ws_switch.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                ))
                .child(toggle_row(
                    "Window open",
                    "Animate window opening",
                    window_open_enabled,
                    move |val| {
                        state_win_open.update_settings(|s| {
                            s.animations.per_animation.window_open.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        state_win_open.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                ))
                .child(toggle_row(
                    "Window close",
                    "Animate window closing",
                    window_close_enabled,
                    move |val| {
                        state_win_close.update_settings(|s| {
                            s.animations.per_animation.window_close.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        state_win_close.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                ))
                .child(toggle_row(
                    "Window movement",
                    "Animate window dragging",
                    window_movement_enabled,
                    move |val| {
                        state_win_move.update_settings(|s| {
                            s.animations.per_animation.window_movement.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        state_win_move.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                ))
                .child(toggle_row(
                    "Window resize",
                    "Animate window resizing",
                    window_resize_enabled,
                    move |val| {
                        state_win_resize.update_settings(|s| {
                            s.animations.per_animation.window_resize.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        state_win_resize.mark_dirty_and_save(SettingsCategory::Animations);
                    },
                )),
        ))
}
