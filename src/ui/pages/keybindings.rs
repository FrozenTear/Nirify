//! Keybindings settings page
//!
//! Two-panel layout with list on left and editor on right.

use freya::prelude::*;

use crate::config::models::{KeybindAction, Keybinding};
use crate::config::SettingsCategory;
use crate::ui::app::{ReactiveState, DROPDOWN_KEYBINDINGS_ACTION};
use crate::ui::components::{section, select_row_with_state, text_row, toggle_row};
use crate::ui::theme::*;

/// Action type options for the select row
const ACTION_TYPE_OPTIONS: &[&str] = &["Spawn Command", "Niri Action", "Action with Args"];

/// Convert KeybindAction to action type index
fn action_to_type_index(action: &KeybindAction) -> usize {
    match action {
        KeybindAction::Spawn(_) => 0,
        KeybindAction::NiriAction(_) => 1,
        KeybindAction::NiriActionWithArgs(_, _) => 2,
    }
}

/// Create the keybindings settings page
pub fn keybindings_page(state: ReactiveState) -> impl IntoElement {
    // Get UI state from ReactiveState (hooks called in app_view)
    let selected_index = state.keybindings_selected;

    let settings = state.get_settings();
    let bindings = settings.keybindings.bindings.clone();
    let sel_idx = *selected_index.read();

    // Get selected keybinding if valid
    let selected_binding = if sel_idx >= 0 && (sel_idx as usize) < bindings.len() {
        Some(bindings[sel_idx as usize].clone())
    } else {
        None
    };

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(SPACING_LG)
        // Left panel - keybinding list
        .child(keybinding_list_panel(
            state.clone(),
            bindings.clone(),
            selected_index,
        ))
        // Right panel - keybinding editor
        .child(keybinding_editor_panel(state, selected_binding, selected_index))
}

/// Left panel with list of keybindings
fn keybinding_list_panel(
    state: ReactiveState,
    bindings: Vec<Keybinding>,
    selected_index: State<i32>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read();

    rect()
        .content(Content::flex())
        .width(Size::px(300.0))
        .height(Size::fill())
        .spacing(SPACING_MD)
        .child(section(
            "Keybindings",
            rect()
                .width(Size::fill())
                .spacing(SPACING_MD)
                // Add new keybinding button
                .child(add_keybinding_button(state.clone(), selected_index.clone()))
                // Keybinding list
                .child(keybinding_list(bindings.clone(), selected_index.clone(), sel_idx))
                // Remove button
                .child(remove_keybinding_button(
                    state,
                    bindings,
                    selected_index,
                    sel_idx,
                )),
        ))
}

/// Add keybinding button
fn add_keybinding_button(state: ReactiveState, mut selected_index: State<i32>) -> impl IntoElement {
    let mut refresh = state.refresh.clone();

    rect()
        .content(Content::flex())
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .main_align(Alignment::Center)
        .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
        .corner_radius(RADIUS_MD)
        .background(ACCENT_VIVID)
        .on_pointer_down(move |_| {
            state.update_and_save(SettingsCategory::Keybindings, |s| {
                // Generate a new unique ID
                let new_id = s
                    .keybindings
                    .bindings
                    .iter()
                    .map(|b| b.id)
                    .max()
                    .unwrap_or(0)
                    + 1;

                let new_binding = Keybinding {
                    id: new_id,
                    key_combo: "Mod+".to_string(),
                    hotkey_overlay_title: None,
                    allow_when_locked: false,
                    cooldown_ms: None,
                    repeat: false,
                    action: KeybindAction::NiriAction(String::new()),
                };
                let new_idx = s.keybindings.bindings.len();
                s.keybindings.bindings.push(new_binding);
                *selected_index.write() = new_idx as i32;
            });
            *refresh.write() += 1;
        })
        .child(
            label()
                .text("+ Add Keybinding")
                .color(BG_DEEP)
                .font_size(FONT_SIZE_SM)
                .font_weight(FontWeight::MEDIUM),
        )
}

/// List of keybindings
fn keybinding_list(
    bindings: Vec<Keybinding>,
    selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let mut container = rect()
        .width(Size::fill())
        .spacing(SPACING_XS)
        .max_height(Size::px(400.0));

    for (idx, binding) in bindings.iter().enumerate() {
        let is_selected = idx as i32 == sel_idx;
        let key_combo = binding.key_combo.clone();
        let action_desc = binding.action.description();
        let mut selected_index = selected_index.clone();

        let bg_color = if is_selected {
            SELECTED_BG
        } else {
            (0, 0, 0, 0)
        };

        let text_color = if is_selected { ACCENT_VIVID } else { TEXT_BRIGHT };

        container = container.child(
            rect()
                .content(Content::flex())
                .direction(Direction::Vertical)
                .width(Size::fill())
                .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                .corner_radius(RADIUS_MD)
                .background(bg_color)
                .spacing(SPACING_2XS)
                .on_pointer_down(move |_| {
                    *selected_index.write() = idx as i32;
                })
                .child(
                    label()
                        .text(key_combo)
                        .color(text_color)
                        .font_size(FONT_SIZE_BASE)
                        .font_weight(FontWeight::MEDIUM)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(action_desc)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        );
    }

    if bindings.is_empty() {
        container = container.child(
            label()
                .text("No keybindings configured")
                .color(TEXT_DIM)
                .font_size(FONT_SIZE_SM),
        );
    }

    container
}

/// Remove keybinding button
fn remove_keybinding_button(
    state: ReactiveState,
    bindings: Vec<Keybinding>,
    mut selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let can_remove = sel_idx >= 0 && (sel_idx as usize) < bindings.len();
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
                state.update_and_save(SettingsCategory::Keybindings, |s| {
                    if sel_idx >= 0 && (sel_idx as usize) < s.keybindings.bindings.len() {
                        s.keybindings.bindings.remove(sel_idx as usize);
                        let new_len = s.keybindings.bindings.len() as i32;
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
                    .text("Remove Keybinding")
                    .color(TEXT_BRIGHT)
                    .font_size(FONT_SIZE_SM)
                    .font_weight(FontWeight::MEDIUM),
            )
            .into_element()
    } else {
        rect().into_element()
    }
}

/// Right panel with keybinding editor
fn keybinding_editor_panel(
    state: ReactiveState,
    selected_binding: Option<Keybinding>,
    selected_index: State<i32>,
) -> impl IntoElement {
    match selected_binding {
        Some(binding) => keybinding_editor(state, binding, selected_index).into_element(),
        None => no_selection_panel().into_element(),
    }
}

/// Panel shown when no keybinding is selected
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
                        .text("No Keybinding Selected")
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_LG)
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    label()
                        .text("Select a keybinding from the list or add a new one")
                        .color(TEXT_GHOST)
                        .font_size(FONT_SIZE_SM),
                ),
        )
}

/// Keybinding editor with all fields
fn keybinding_editor(
    state: ReactiveState,
    binding: Keybinding,
    selected_index: State<i32>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read() as usize;
    let mut refresh = state.refresh.clone();

    // Get action details
    let action_type_idx = action_to_type_index(&binding.action);
    let (action_value, action_args): (String, String) = match &binding.action {
        KeybindAction::Spawn(args) => (
            args.first().cloned().unwrap_or_default(),
            args.get(1..).map(|a: &[String]| a.join(" ")).unwrap_or_default(),
        ),
        KeybindAction::NiriAction(action) => (action.clone(), String::new()),
        KeybindAction::NiriActionWithArgs(action, args) => (action.clone(), args.join(" ")),
    };

    rect()
        .content(Content::flex())
        .width(Size::flex(1.0))
        .height(Size::fill())
        .spacing(SPACING_LG)
        // Key combination section
        .child(section(
            "Key Combination",
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                .child({
                    let state = state.clone();
                    let key_combo = binding.key_combo.clone();
                    let mut refresh = refresh.clone();
                    text_row(
                        "Key",
                        "Key combination (e.g., Mod+Space)",
                        &key_combo,
                        "Mod+Space",
                        move |v| {
                            state.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.key_combo = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                }),
        ))
        // Action type section
        .child(section(
            "Action",
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                // Action type selector
                .child({
                    let state_clone = state.clone();
                    let mut refresh = refresh.clone();
                    select_row_with_state(
                        "Action Type",
                        "Type of action to perform",
                        ACTION_TYPE_OPTIONS,
                        action_type_idx,
                        DROPDOWN_KEYBINDINGS_ACTION,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.action = match i {
                                        0 => KeybindAction::Spawn(vec![]),
                                        1 => KeybindAction::NiriAction(String::new()),
                                        2 => KeybindAction::NiriActionWithArgs(String::new(), vec![]),
                                        _ => KeybindAction::NiriAction(String::new()),
                                    };
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Command/Action input
                .child({
                    let state = state.clone();
                    let action_value = action_value.clone();
                    let mut refresh = refresh.clone();
                    let action_type_idx = action_type_idx;
                    let (label, desc, placeholder) = match action_type_idx {
                        0 => ("Command", "Command to execute", "alacritty"),
                        1 => ("Action", "Niri action name", "close-window"),
                        2 => ("Action", "Niri action name", "focus-workspace"),
                        _ => ("Action", "Action to perform", ""),
                    };
                    text_row(label, desc, &action_value, placeholder, move |v| {
                        state.update_and_save(SettingsCategory::Keybindings, |s| {
                            if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                match &mut b.action {
                                    KeybindAction::Spawn(args) => {
                                        if args.is_empty() {
                                            args.push(v);
                                        } else {
                                            args[0] = v;
                                        }
                                    }
                                    KeybindAction::NiriAction(action) => {
                                        *action = v;
                                    }
                                    KeybindAction::NiriActionWithArgs(action, _) => {
                                        *action = v;
                                    }
                                }
                            }
                        });
                        *refresh.write() += 1;
                    })
                })
                // Args input (shown for Spawn and ActionWithArgs)
                .child({
                    let state = state.clone();
                    let action_args = action_args.clone();
                    let mut refresh = refresh.clone();
                    let show_args = action_type_idx == 0 || action_type_idx == 2;
                    if show_args {
                        text_row(
                            "Arguments",
                            "Additional arguments",
                            &action_args,
                            "",
                            move |v| {
                                state.update_and_save(SettingsCategory::Keybindings, |s| {
                                    if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                        let args: Vec<String> = v.split_whitespace().map(|s| s.to_string()).collect();
                                        match &mut b.action {
                                            KeybindAction::Spawn(existing_args) => {
                                                let cmd = existing_args.first().cloned().unwrap_or_default();
                                                *existing_args = std::iter::once(cmd).chain(args).collect();
                                            }
                                            KeybindAction::NiriActionWithArgs(_, existing_args) => {
                                                *existing_args = args;
                                            }
                                            _ => {}
                                        }
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        ).into_element()
                    } else {
                        rect().into_element()
                    }
                }),
        ))
        // Options section
        .child(section(
            "Options",
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                // Allow when locked
                .child({
                    let state = state.clone();
                    let allow = binding.allow_when_locked;
                    let mut refresh = refresh.clone();
                    toggle_row(
                        "Allow When Locked",
                        "Execute even when screen is locked",
                        allow,
                        move |v| {
                            state.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.allow_when_locked = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Repeat when held
                .child({
                    let state = state.clone();
                    let repeat = binding.repeat;
                    let mut refresh = refresh.clone();
                    toggle_row(
                        "Repeat When Held",
                        "Repeat action while key is held",
                        repeat,
                        move |v| {
                            state.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.repeat = v;
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Cooldown
                .child({
                    let state = state.clone();
                    let cooldown: String = binding.cooldown_ms.map(|c: i32| c.to_string()).unwrap_or_default();
                    let mut refresh = refresh.clone();
                    text_row(
                        "Cooldown",
                        "Minimum time between activations (ms)",
                        &cooldown,
                        "0",
                        move |v| {
                            state.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.cooldown_ms = v.parse::<i32>().ok().filter(|&c| c > 0);
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Hotkey overlay title label
                .child({
                    let title = binding.hotkey_overlay_title.clone().unwrap_or_default();
                    toggle_row(
                        "Overlay Title",
                        "Custom title for hotkey overlay",
                        !title.is_empty(),
                        |_| {},
                    )
                })
                .child({
                    let state = state.clone();
                    let title = binding.hotkey_overlay_title.clone().unwrap_or_default();
                    text_row(
                        "",
                        "",
                        &title,
                        "Optional title",
                        move |v| {
                            state.update_and_save(SettingsCategory::Keybindings, |s| {
                                if let Some(b) = s.keybindings.bindings.get_mut(sel_idx) {
                                    b.hotkey_overlay_title = if v.is_empty() { None } else { Some(v) };
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                }),
        ))
}
