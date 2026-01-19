//! Animation settings page

use freya::prelude::*;

use crate::config::models::AnimationType;
use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the animations settings page
pub fn animations_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let animations = &settings.animations;

    // Per-animation enabled states
    let ws_switch =
        animations.per_animation.workspace_switch.animation_type != AnimationType::Off;
    let win_open = animations.per_animation.window_open.animation_type != AnimationType::Off;
    let win_close = animations.per_animation.window_close.animation_type != AnimationType::Off;
    let win_move = animations.per_animation.window_movement.animation_type != AnimationType::Off;
    let win_resize = animations.per_animation.window_resize.animation_type != AnimationType::Off;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();
    let state6 = state.clone();
    let mut refresh6 = state.refresh.clone();
    let state7 = state.clone();
    let mut refresh7 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.enabled = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.slowdown = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
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
                    ws_switch,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.per_animation.workspace_switch.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Window open",
                    "Animate window opening",
                    win_open,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.per_animation.window_open.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Window close",
                    "Animate window closing",
                    win_close,
                    move |val| {
                        state5.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.per_animation.window_close.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Window movement",
                    "Animate window dragging",
                    win_move,
                    move |val| {
                        state6.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.per_animation.window_movement.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Window resize",
                    "Animate window resizing",
                    win_resize,
                    move |val| {
                        state7.update_and_save(SettingsCategory::Animations, |s| {
                            s.animations.per_animation.window_resize.animation_type = if val {
                                AnimationType::Easing
                            } else {
                                AnimationType::Off
                            };
                        });
                        refresh7.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
