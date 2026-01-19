//! Outputs (displays) settings page
//!
//! Two-panel layout with list on left and editor on right.

use freya::prelude::*;

use crate::config::{OutputConfig, SettingsCategory};
use crate::types::{Transform, VrrMode};
use crate::ui::app::{ReactiveState, DROPDOWN_OUTPUTS_TRANSFORM, DROPDOWN_OUTPUTS_VRR};
use crate::ui::components::{section, select_row_with_state, slider_row, text_row, toggle_row};
use crate::ui::theme::*;

/// Transform option labels
const TRANSFORM_OPTIONS: &[&str] = &[
    "Normal",
    "Rotate 90°",
    "Rotate 180°",
    "Rotate 270°",
    "Flipped",
    "Flipped 90°",
    "Flipped 180°",
    "Flipped 270°",
];

/// VRR mode option labels
const VRR_OPTIONS: &[&str] = &["Off", "On", "On Demand"];

/// Convert Transform enum to index
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

/// Convert index to Transform enum
fn index_to_transform(i: usize) -> Transform {
    match i {
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

/// Convert VrrMode enum to index
fn vrr_to_index(v: VrrMode) -> usize {
    match v {
        VrrMode::Off => 0,
        VrrMode::On => 1,
        VrrMode::OnDemand => 2,
    }
}

/// Convert index to VrrMode enum
fn index_to_vrr(i: usize) -> VrrMode {
    match i {
        0 => VrrMode::Off,
        1 => VrrMode::On,
        2 => VrrMode::OnDemand,
        _ => VrrMode::Off,
    }
}

/// Create the outputs settings page
pub fn outputs_page(state: ReactiveState) -> impl IntoElement {
    // Get UI state from ReactiveState (hooks called in app_view)
    let selected_index = state.outputs_selected;
    let new_output_name = state.outputs_new_name;

    let settings = state.get_settings();
    let outputs = settings.outputs.outputs.clone();
    let sel_idx = *selected_index.read();

    // Get selected output if valid
    let selected_output = if sel_idx >= 0 && (sel_idx as usize) < outputs.len() {
        Some(outputs[sel_idx as usize].clone())
    } else {
        None
    };

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(SPACING_LG)
        // Left panel - output list
        .child(output_list_panel(
            state.clone(),
            outputs.clone(),
            selected_index,
            new_output_name,
        ))
        // Right panel - output editor
        .child(output_editor_panel(state, selected_output, selected_index))
}

/// Left panel with list of outputs
fn output_list_panel(
    state: ReactiveState,
    outputs: Vec<OutputConfig>,
    selected_index: State<i32>,
    new_output_name: State<String>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read();
    let new_name = new_output_name.read().clone();

    rect()
        .content(Content::flex())
        .width(Size::px(280.0))
        .height(Size::fill())
        .spacing(SPACING_MD)
        .child(section(
            "Configured Outputs",
            rect()
                .width(Size::fill())
                .spacing(SPACING_MD)
                // Add new output input
                .child(add_output_row(
                    state.clone(),
                    new_output_name.clone(),
                    new_name,
                    selected_index.clone(),
                ))
                // Output list
                .child(output_list(outputs.clone(), selected_index.clone(), sel_idx))
                // Remove button
                .child(remove_output_button(
                    state,
                    outputs,
                    selected_index,
                    sel_idx,
                )),
        ))
}

/// Add output row with input and button
fn add_output_row(
    state: ReactiveState,
    mut new_output_name: State<String>,
    new_name: String,
    mut selected_index: State<i32>,
) -> impl IntoElement {
    let state_clone = state.clone();
    let mut new_output_name_clone = new_output_name.clone();
    let mut refresh = state.refresh.clone();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .spacing(SPACING_SM)
        .child(
            Input::new()
                .value(new_name.clone())
                .placeholder("Output name (e.g. eDP-1)")
                .width(Size::flex(1.0))
                .on_change(move |v: String| {
                    *new_output_name.write() = v;
                }),
        )
        .child(
            rect()
                .content(Content::flex())
                .cross_align(Alignment::Center)
                .main_align(Alignment::Center)
                .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                .corner_radius(RADIUS_MD)
                .background(ACCENT_VIVID)
                .on_pointer_down(move |_| {
                    let name = new_output_name_clone.read().trim().to_string();
                    if !name.is_empty() {
                        let mut new_output = OutputConfig::default();
                        new_output.name = name;
                        state_clone.update_and_save(SettingsCategory::Outputs, |s| {
                            let new_idx = s.outputs.outputs.len();
                            s.outputs.outputs.push(new_output);
                            *selected_index.write() = new_idx as i32;
                        });
                        *new_output_name_clone.write() = String::new();
                        *refresh.write() += 1;
                    }
                })
                .child(
                    label()
                        .text("+")
                        .color(BG_DEEP)
                        .font_size(FONT_SIZE_LG)
                        .font_weight(FontWeight::BOLD),
                ),
        )
}

/// List of outputs
fn output_list(
    outputs: Vec<OutputConfig>,
    selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let mut container = rect().width(Size::fill()).spacing(SPACING_XS);

    for (idx, output) in outputs.iter().enumerate() {
        let is_selected = idx as i32 == sel_idx;
        let name = output.name.clone();
        let enabled = output.enabled;
        let mut selected_index = selected_index.clone();

        let bg_color = if is_selected {
            SELECTED_BG
        } else {
            (0, 0, 0, 0)
        };

        let text_color = if is_selected { ACCENT_VIVID } else { TEXT_BRIGHT };
        let status_color = if enabled { SUCCESS } else { TEXT_DIM };
        let status_text = if enabled { "●" } else { "○" };

        container = container.child(
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                .corner_radius(RADIUS_MD)
                .background(bg_color)
                .spacing(SPACING_SM)
                .cross_align(Alignment::Center)
                .on_pointer_down(move |_| {
                    *selected_index.write() = idx as i32;
                })
                .child(
                    label()
                        .text(status_text)
                        .color(status_color)
                        .font_size(FONT_SIZE_SM),
                )
                .child(
                    label()
                        .width(Size::flex(1.0))
                        .text(name)
                        .color(text_color)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                ),
        );
    }

    if outputs.is_empty() {
        container = container.child(
            label()
                .text("No outputs configured")
                .color(TEXT_DIM)
                .font_size(FONT_SIZE_SM),
        );
    }

    container
}

/// Remove output button
fn remove_output_button(
    state: ReactiveState,
    outputs: Vec<OutputConfig>,
    mut selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let can_remove = sel_idx >= 0 && (sel_idx as usize) < outputs.len();
    let mut refresh = state.refresh.clone();

    if can_remove {
        rect()
            .content(Content::flex())
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
            .corner_radius(RADIUS_MD)
            .background(ERROR)
            .on_pointer_down(move |_| {
                state.update_and_save(SettingsCategory::Outputs, |s| {
                    if sel_idx >= 0 && (sel_idx as usize) < s.outputs.outputs.len() {
                        s.outputs.outputs.remove(sel_idx as usize);
                        // Update selection
                        let new_len = s.outputs.outputs.len() as i32;
                        if new_len == 0 {
                            *selected_index.write() = -1;
                        } else if sel_idx >= new_len {
                            *selected_index.write() = new_len - 1;
                        }
                    }
                });
                *refresh.write() += 1;
            })
            .child(
                label()
                    .text("Remove Output")
                    .color(TEXT_BRIGHT)
                    .font_size(FONT_SIZE_SM)
                    .font_weight(FontWeight::MEDIUM),
            )
            .into_element()
    } else {
        rect().into_element()
    }
}

/// Right panel with output editor
fn output_editor_panel(
    state: ReactiveState,
    selected_output: Option<OutputConfig>,
    selected_index: State<i32>,
) -> impl IntoElement {
    match selected_output {
        Some(output) => output_editor(state, output, selected_index).into_element(),
        None => no_selection_panel().into_element(),
    }
}

/// Panel shown when no output is selected
fn no_selection_panel() -> impl IntoElement {
    rect()
        .content(Content::flex())
        .width(Size::flex(1.0))
        .height(Size::fill())
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .spacing(SPACING_MD)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text("No Output Selected")
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_LG)
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    label()
                        .text("Select an output from the list or add a new one")
                        .color(TEXT_GHOST)
                        .font_size(FONT_SIZE_SM),
                ),
        )
}

/// Output editor with all fields
fn output_editor(
    state: ReactiveState,
    output: OutputConfig,
    selected_index: State<i32>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read() as usize;
    let mut refresh = state.refresh.clone();

    rect()
        .content(Content::flex())
        .width(Size::flex(1.0))
        .height(Size::fill())
        .spacing(SPACING_LG)
        .child(section(
            &format!("Output: {}", output.name),
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                // Enabled toggle
                .child({
                    let state = state.clone();
                    let enabled = output.enabled;
                    let mut refresh = refresh.clone();
                    toggle_row("Enabled", "Enable or disable this output", enabled, move |v| {
                        state.update_and_save(SettingsCategory::Outputs, |s| {
                            if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                o.enabled = v;
                            }
                        });
                        *refresh.write() += 1;
                    })
                })
                // Scale slider
                .child({
                    let state = state.clone();
                    let scale = output.scale;
                    let mut refresh = refresh.clone();
                    slider_row(
                        "Scale",
                        "Display scaling factor",
                        scale,
                        0.25,
                        4.0,
                        "x",
                        move |v| {
                            state.update_and_save(SettingsCategory::Outputs, |s| {
                                if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                    o.scale = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Mode text input
                .child({
                    let state = state.clone();
                    let mode = output.mode.clone();
                    let mut refresh = refresh.clone();
                    text_row(
                        "Mode",
                        "Resolution and refresh rate",
                        &mode,
                        "1920x1080@60",
                        move |v| {
                            state.update_and_save(SettingsCategory::Outputs, |s| {
                                if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                    o.mode = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Position X
                .child({
                    let state = state.clone();
                    let x = output.position_x;
                    let mut refresh = refresh.clone();
                    text_row(
                        "Position X",
                        "Horizontal position in pixels",
                        &x.to_string(),
                        "0",
                        move |v| {
                            if let Ok(x) = v.parse::<i32>() {
                                state.update_and_save(SettingsCategory::Outputs, |s| {
                                    if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                        o.position_x = x;
                                    }
                                });
                                *refresh.write() += 1;
                            }
                        },
                    )
                })
                // Position Y
                .child({
                    let state = state.clone();
                    let y = output.position_y;
                    let mut refresh = refresh.clone();
                    text_row(
                        "Position Y",
                        "Vertical position in pixels",
                        &y.to_string(),
                        "0",
                        move |v| {
                            if let Ok(y) = v.parse::<i32>() {
                                state.update_and_save(SettingsCategory::Outputs, |s| {
                                    if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                        o.position_y = y;
                                    }
                                });
                                *refresh.write() += 1;
                            }
                        },
                    )
                })
                // Transform select
                .child({
                    let state_clone = state.clone();
                    let transform = output.transform;
                    let mut refresh = refresh.clone();
                    select_row_with_state(
                        "Transform",
                        "Rotation and flip settings",
                        TRANSFORM_OPTIONS,
                        transform_to_index(transform),
                        DROPDOWN_OUTPUTS_TRANSFORM,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Outputs, |s| {
                                if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                    o.transform = index_to_transform(i);
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // VRR select
                .child({
                    let state_clone = state.clone();
                    let vrr = output.vrr;
                    let mut refresh = refresh.clone();
                    select_row_with_state(
                        "Variable Refresh Rate",
                        "FreeSync/G-Sync mode",
                        VRR_OPTIONS,
                        vrr_to_index(vrr),
                        DROPDOWN_OUTPUTS_VRR,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Outputs, |s| {
                                if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                    o.vrr = index_to_vrr(i);
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Focus at startup toggle
                .child({
                    let state = state.clone();
                    let focus = output.focus_at_startup;
                    toggle_row(
                        "Focus at Startup",
                        "Focus this output when niri starts",
                        focus,
                        move |v| {
                            state.update_and_save(SettingsCategory::Outputs, |s| {
                                if let Some(o) = s.outputs.outputs.get_mut(sel_idx) {
                                    o.focus_at_startup = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                }),
        ))
}
