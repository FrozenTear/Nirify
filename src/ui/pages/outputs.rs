//! Outputs (displays) settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::OutputConfig;
use crate::config::SettingsCategory;
use crate::types::{Transform, VrrMode};
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, ACCENT, BG_ELEVATED, BG_SURFACE,
    BORDER_SUBTLE, ERROR, FONT_SIZE_SM, RADIUS_MD, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XS, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
};

/// Create the outputs settings page
pub fn outputs_page(state: AppState) -> impl IntoView {
    // Create a signal for outputs list
    let outputs = RwSignal::new(state.get_settings().outputs.outputs.clone());

    Stack::vertical((
        section(
            "Configured Displays",
            Stack::vertical((
                // List of existing outputs
                output_list(state.clone(), outputs),
                // Add button
                add_output_button(state.clone(), outputs),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "About Display Configuration",
            Stack::vertical((Label::derived(|| {
                "Display settings control resolution, refresh rate, scaling, \
                 rotation, and position for each monitor. Use the output name \
                 from niri (e.g., 'eDP-1', 'HDMI-A-1') to configure each display."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// List of output configuration cards
fn output_list(state: AppState, outputs: RwSignal<Vec<OutputConfig>>) -> impl IntoView {
    floem::views::dyn_container(
        move || outputs.get(),
        move |out_list| {
            if out_list.is_empty() {
                Label::derived(|| {
                    "No displays configured. Add a display to get started.".to_string()
                })
                .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                .into_any()
            } else {
                Stack::vertical(
                    out_list
                        .into_iter()
                        .enumerate()
                        .map(|(idx, output)| output_card(state.clone(), idx, output, outputs))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_MD))
                .into_any()
            }
        },
    )
}

/// Single output configuration card
fn output_card(
    state: AppState,
    index: usize,
    output: OutputConfig,
    outputs: RwSignal<Vec<OutputConfig>>,
) -> impl IntoView {
    let name_signal = RwSignal::new(output.name.clone());
    let mode_signal = RwSignal::new(output.mode.clone());
    let scale_signal = RwSignal::new(output.scale);
    let enabled_signal = RwSignal::new(output.enabled);
    let pos_x_signal = RwSignal::new(output.position_x as f64);
    let pos_y_signal = RwSignal::new(output.position_y as f64);
    let transform_idx = RwSignal::new(transform_to_index(output.transform));
    let vrr_idx = RwSignal::new(vrr_to_index(output.vrr));

    // Expanded state for this card
    let expanded = RwSignal::new(false);

    // Save helper
    let save = {
        let state = state.clone();
        Rc::new(move || {
            state.update_settings(|s| {
                s.outputs.outputs = outputs.get();
            });
            state.mark_dirty_and_save(SettingsCategory::Outputs);
        })
    };

    // Name change
    let save_name = save.clone();
    let on_name_change = move || {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.name = name_signal.get();
            }
        });
        save_name();
    };

    // Mode change
    let save_mode = save.clone();
    let on_mode_change = Rc::new(move || {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.mode = mode_signal.get();
            }
        });
        save_mode();
    });

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        outputs.update(|out| {
            out.remove(index);
        });
        state_delete.update_settings(|s| {
            s.outputs.outputs = outputs.get();
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Outputs);
    };

    // Enabled toggle callback
    let save_enabled = save.clone();
    let on_enabled = Rc::new(move |val: bool| {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.enabled = val;
            }
        });
        save_enabled();
    });

    // Scale callback
    let save_scale = save.clone();
    let on_scale = Rc::new(move |val: f64| {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.scale = val;
            }
        });
        save_scale();
    });

    // Position callbacks
    let save_pos_x = save.clone();
    let on_pos_x = Rc::new(move |val: f64| {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.position_x = val as i32;
            }
        });
        save_pos_x();
    });

    let save_pos_y = save.clone();
    let on_pos_y = Rc::new(move |val: f64| {
        outputs.update(|out| {
            if let Some(o) = out.get_mut(index) {
                o.position_y = val as i32;
            }
        });
        save_pos_y();
    });

    Stack::vertical((
        // Header row with name, summary, expand/collapse, delete
        Stack::horizontal((
            // Output name input
            text_input(name_signal)
                .placeholder("eDP-1")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_name_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .width(120.0)
                        .font_family("monospace".to_string())
                }),
            // Summary text
            Label::derived(move || {
                let mode = mode_signal.get();
                let scale = scale_signal.get();
                let mode_str = if mode.is_empty() {
                    "auto".to_string()
                } else {
                    mode
                };
                format!("{} @ {:.1}x", mode_str, scale)
            })
            .style(|s| {
                s.color(TEXT_SECONDARY)
                    .font_size(FONT_SIZE_SM)
                    .flex_grow(1.0)
            }),
            // Expand/collapse button
            Container::new(
                Label::derived(move || if expanded.get() { "▼" } else { "▶" }.to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(ACCENT)))
            .on_click_stop(move |_| expanded.set(!expanded.get())),
            // Delete button
            Container::new(
                Label::derived(|| "✕".to_string())
                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
            )
            .style(|s| {
                icon_button_style(s).hover(|s| s.background(ERROR.with_alpha(0.2)).color(ERROR))
            })
            .on_click_stop(move |_| on_delete()),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Expanded settings (conditionally shown)
        {
            // Clone callbacks for use in the inner closure
            let on_mode_change = on_mode_change.clone();
            let on_scale = on_scale.clone();
            let on_enabled = on_enabled.clone();
            let on_pos_x = on_pos_x.clone();
            let on_pos_y = on_pos_y.clone();
            let state = state.clone();

            floem::views::dyn_container(
                move || expanded.get(),
                move |is_expanded| {
                    let on_mode_change = on_mode_change.clone();
                    let on_scale = on_scale.clone();
                    let on_enabled = on_enabled.clone();
                    let on_pos_x = on_pos_x.clone();
                    let on_pos_y = on_pos_y.clone();
                    let state = state.clone();

                    if is_expanded {
                        Stack::vertical((
                            // Mode input
                            Stack::horizontal((
                                Label::derived(|| "Mode".to_string()).style(|s| {
                                    s.color(TEXT_PRIMARY)
                                        .font_size(FONT_SIZE_SM)
                                        .min_width(80.0)
                                }),
                                text_input(mode_signal)
                                    .placeholder("1920x1080@60.000 (leave empty for auto)")
                                    .on_event_stop(EventListener::FocusLost, move |_| {
                                        on_mode_change();
                                    })
                                    .style(|s| {
                                        text_input_style(s)
                                            .flex_grow(1.0)
                                            .font_family("monospace".to_string())
                                    }),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                            // Scale slider
                            slider_row_with_callback(
                                "Scale",
                                Some("Display scaling factor"),
                                scale_signal,
                                0.5,
                                4.0,
                                0.25,
                                "x",
                                Some(on_scale),
                            ),
                            // Enabled toggle
                            toggle_row_with_callback(
                                "Enabled",
                                Some("Turn this display on or off"),
                                enabled_signal,
                                Some(on_enabled),
                            ),
                            // Position
                            Stack::horizontal((
                                Label::derived(|| "Position".to_string()).style(|s| {
                                    s.color(TEXT_PRIMARY)
                                        .font_size(FONT_SIZE_SM)
                                        .min_width(80.0)
                                }),
                                Label::derived(|| "X:".to_string())
                                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
                                slider_row_with_callback(
                                    "",
                                    None,
                                    pos_x_signal,
                                    -10000.0,
                                    10000.0,
                                    1.0,
                                    "",
                                    Some(on_pos_x),
                                ),
                                Label::derived(|| "Y:".to_string())
                                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
                                slider_row_with_callback(
                                    "",
                                    None,
                                    pos_y_signal,
                                    -10000.0,
                                    10000.0,
                                    1.0,
                                    "",
                                    Some(on_pos_y),
                                ),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                            // Transform dropdown (simplified as label for now)
                            Stack::horizontal((
                                Label::derived(|| "Transform".to_string()).style(|s| {
                                    s.color(TEXT_PRIMARY)
                                        .font_size(FONT_SIZE_SM)
                                        .min_width(80.0)
                                }),
                                transform_selector(transform_idx, outputs, index, state.clone()),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                            // VRR dropdown
                            Stack::horizontal((
                                Label::derived(|| "VRR".to_string()).style(|s| {
                                    s.color(TEXT_PRIMARY)
                                        .font_size(FONT_SIZE_SM)
                                        .min_width(80.0)
                                }),
                                vrr_selector(vrr_idx, outputs, index, state.clone()),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                        ))
                        .style(|s| {
                            s.width_full()
                                .gap(SPACING_SM)
                                .padding_top(SPACING_MD)
                                .border_top(1.0)
                                .border_color(BORDER_SUBTLE)
                                .margin_top(SPACING_SM)
                        })
                        .into_any()
                    } else {
                        floem::views::Empty::new().into_any()
                    }
                },
            )
        },
    ))
    .style(|s| {
        s.width_full()
            .padding(SPACING_MD)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_MD)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// Static array of transform names
static TRANSFORMS: &[&str] = &[
    "Normal", "90°", "180°", "270°", "Flip", "Flip90", "Flip180", "Flip270",
];

/// Transform selector (simple text buttons for now)
fn transform_selector(
    transform_idx: RwSignal<usize>,
    outputs: RwSignal<Vec<OutputConfig>>,
    index: usize,
    state: AppState,
) -> impl IntoView {
    Stack::horizontal(
        TRANSFORMS
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let state = state.clone();
                let name = *name;
                let is_selected = move || transform_idx.get() == idx;

                Container::new(Label::derived(move || name.to_string()).style(move |s| {
                    let base = s
                        .font_size(FONT_SIZE_SM)
                        .padding_horiz(SPACING_XS)
                        .padding_vert(2.0);
                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_MUTED)
                    }
                }))
                .style(move |s| {
                    let base = s.border_radius(RADIUS_SM);
                    if is_selected() {
                        base.background(ACCENT.with_alpha(0.15))
                    } else {
                        base.hover(|s| s.background(BG_SURFACE))
                    }
                })
                .on_click_stop(move |_| {
                    transform_idx.set(idx);
                    outputs.update(|out| {
                        if let Some(o) = out.get_mut(index) {
                            o.transform = index_to_transform(idx);
                        }
                    });
                    state.update_settings(|s| {
                        s.outputs.outputs = outputs.get();
                    });
                    state.mark_dirty_and_save(SettingsCategory::Outputs);
                })
            })
            .collect::<Vec<_>>(),
    )
    .style(|s| s.gap(SPACING_XS).flex_wrap(floem::style::FlexWrap::Wrap))
}

/// Static array of VRR mode names
static VRR_MODES: &[&str] = &["Off", "On", "On Demand"];

/// VRR selector
fn vrr_selector(
    vrr_idx: RwSignal<usize>,
    outputs: RwSignal<Vec<OutputConfig>>,
    index: usize,
    state: AppState,
) -> impl IntoView {
    Stack::horizontal(
        VRR_MODES
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let state = state.clone();
                let name = *name;
                let is_selected = move || vrr_idx.get() == idx;

                Container::new(Label::derived(move || name.to_string()).style(move |s| {
                    let base = s
                        .font_size(FONT_SIZE_SM)
                        .padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS);
                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_MUTED)
                    }
                }))
                .style(move |s| {
                    let base = s.border_radius(RADIUS_SM);
                    if is_selected() {
                        base.background(ACCENT.with_alpha(0.15))
                    } else {
                        base.hover(|s| s.background(BG_SURFACE))
                    }
                })
                .on_click_stop(move |_| {
                    vrr_idx.set(idx);
                    outputs.update(|out| {
                        if let Some(o) = out.get_mut(index) {
                            o.vrr = index_to_vrr(idx);
                        }
                    });
                    state.update_settings(|s| {
                        s.outputs.outputs = outputs.get();
                    });
                    state.mark_dirty_and_save(SettingsCategory::Outputs);
                })
            })
            .collect::<Vec<_>>(),
    )
    .style(|s| s.gap(SPACING_XS))
}

/// Add new output button
fn add_output_button(state: AppState, outputs: RwSignal<Vec<OutputConfig>>) -> impl IntoView {
    let on_add = move || {
        let new_output = OutputConfig::default();

        outputs.update(|out| {
            out.push(new_output);
        });

        state.update_settings(|s| {
            s.outputs.outputs = outputs.get();
        });
        state.mark_dirty_and_save(SettingsCategory::Outputs);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Display".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}

// Transform conversion helpers
fn transform_to_index(t: Transform) -> usize {
    match t {
        Transform::Normal => 0,
        Transform::Rotate90 => 1,
        Transform::Rotate180 => 2,
        Transform::Rotate270 => 3,
        Transform::Flipped => 4,
        Transform::Flipped90 => 5,
        Transform::Flipped180 => 6,
        Transform::Flipped270 => 7,
    }
}

fn index_to_transform(idx: usize) -> Transform {
    match idx {
        0 => Transform::Normal,
        1 => Transform::Rotate90,
        2 => Transform::Rotate180,
        3 => Transform::Rotate270,
        4 => Transform::Flipped,
        5 => Transform::Flipped90,
        6 => Transform::Flipped180,
        7 => Transform::Flipped270,
        _ => Transform::Normal,
    }
}

fn vrr_to_index(v: VrrMode) -> usize {
    match v {
        VrrMode::Off => 0,
        VrrMode::On => 1,
        VrrMode::OnDemand => 2,
    }
}

fn index_to_vrr(idx: usize) -> VrrMode {
    match idx {
        0 => VrrMode::Off,
        1 => VrrMode::On,
        2 => VrrMode::OnDemand,
        _ => VrrMode::Off,
    }
}
