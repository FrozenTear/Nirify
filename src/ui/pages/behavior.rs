//! Behavior settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the behavior settings page
pub fn behavior_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let behavior = &settings.behavior;

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
    let state8 = state.clone();
    let mut refresh8 = state.refresh.clone();
    let state9 = state.clone();
    let mut refresh9 = state.refresh.clone();
    let state10 = state.clone();
    let mut refresh10 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Focus section
        .child(section(
            "Focus",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Focus follows mouse",
                    "Automatically focus window under cursor",
                    behavior.focus_follows_mouse,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.focus_follows_mouse = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Workspace back and forth",
                    "Switch to previous workspace with same key",
                    behavior.workspace_auto_back_and_forth,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.workspace_auto_back_and_forth = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Layout section
        .child(section(
            "Layout",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Center single column",
                    "Always center when only one column exists",
                    behavior.always_center_single_column,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.always_center_single_column = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Empty workspace above first",
                    "Add empty workspace above the first one",
                    behavior.empty_workspace_above_first,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.empty_workspace_above_first = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Default column width",
                    "Width proportion for new columns (0.5 = half)",
                    behavior.default_column_width_proportion as f64,
                    0.1,
                    2.0,
                    "",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.default_column_width_proportion = val as f32
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Screen Margins section
        .child(section(
            "Screen Margins (Struts)",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Left margin",
                    "Reserved space on left edge",
                    behavior.strut_left as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state6.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_left = val as f32
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Right margin",
                    "Reserved space on right edge",
                    behavior.strut_right as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state7.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_right = val as f32
                        });
                        refresh7.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Top margin",
                    "Reserved space on top edge",
                    behavior.strut_top as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state8.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_top = val as f32
                        });
                        refresh8.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Bottom margin",
                    "Reserved space on bottom edge",
                    behavior.strut_bottom as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state9.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_bottom = val as f32
                        });
                        refresh9.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // System section
        .child(section(
            "System",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable power key handling",
                    "Let the system handle the power button",
                    behavior.disable_power_key_handling,
                    move |val| {
                        state10.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.disable_power_key_handling = val
                        });
                        refresh10.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
