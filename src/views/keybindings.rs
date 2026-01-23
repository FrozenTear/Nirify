//! Keybindings settings view - list-detail implementation with key capture
//!
//! Provides a visual editor for keyboard shortcuts with:
//! - List of all keybindings
//! - Key capture widget for setting key combinations
//! - Action type selection (spawn command, niri action)
//! - Advanced options (cooldown, repeat, allow when locked)

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{KeybindAction, Keybinding, KeybindingsSettings};
use crate::messages::{KeybindingsMessage, Message};
use crate::types::ModKey;

/// Common niri actions for quick selection
const COMMON_ACTIONS: &[&str] = &[
    "close-window",
    "quit",
    "toggle-overview",
    "screenshot",
    "screenshot-screen",
    "screenshot-window",
    "focus-column-left",
    "focus-column-right",
    "focus-window-up",
    "focus-window-down",
    "move-column-left",
    "move-column-right",
    "move-window-up",
    "move-window-down",
    "focus-workspace-down",
    "focus-workspace-up",
    "move-column-to-workspace-down",
    "move-column-to-workspace-up",
    "consume-window-into-column",
    "expel-window-from-column",
    "maximize-column",
    "fullscreen-window",
    "switch-preset-column-width",
    "reset-window-height",
    "power-off-monitors",
    "spawn",
];

/// Creates the keybindings settings view with list-detail pattern
pub fn view<'a>(
    settings: &'a KeybindingsSettings,
    selected_index: Option<usize>,
    sections_expanded: &'a HashMap<String, bool>,
    key_capture_active: Option<usize>,
) -> Element<'a, Message> {
    // Left panel: List of keybindings
    let list_panel = keybinding_list(settings, selected_index);

    // Right panel: Detail view for selected keybinding
    let detail_panel = if let Some(idx) = selected_index {
        if let Some(binding) = settings.bindings.get(idx) {
            keybinding_detail_view(binding, idx, sections_expanded, key_capture_active)
        } else {
            empty_detail_view()
        }
    } else {
        empty_detail_view()
    };

    // Use shared list-detail layout
    list_detail_layout(list_panel, detail_panel)
}

/// List panel showing all keybindings
fn keybinding_list<'a>(
    settings: &'a KeybindingsSettings,
    selected_index: Option<usize>,
) -> Element<'a, Message> {
    let mut list = column![
        row![
            text("Keybindings").size(18),
            add_button(Message::Keybindings(KeybindingsMessage::AddKeybinding)),
        ]
        .spacing(10)
        .padding([12, 20])
        .align_y(Alignment::Center),
    ]
    .spacing(0);

    // Show error if loading failed
    if let Some(error) = &settings.error {
        list = list.push(
            container(
                text(format!("Error loading keybindings:\n{}", error))
                    .size(12)
                    .color([0.9, 0.4, 0.4])
            )
            .padding(12)
        );
    }

    if settings.bindings.is_empty() {
        list = list.push(empty_list_placeholder("No keybindings configured\nClick + to add one"));
    } else {
        for (idx, binding) in settings.bindings.iter().enumerate() {
            let is_selected = selected_index == Some(idx);

            // Format the display: key combo + action preview
            let key_display = if binding.key_combo.is_empty() {
                "(no key set)".to_string()
            } else {
                binding.key_combo.clone()
            };

            let action_preview = match &binding.action {
                KeybindAction::Spawn(args) => {
                    if args.is_empty() {
                        "spawn ...".to_string()
                    } else {
                        format!("spawn {}", args.first().unwrap_or(&String::new()))
                    }
                }
                KeybindAction::NiriAction(action) => action.clone(),
                KeybindAction::NiriActionWithArgs(action, _) => action.clone(),
            };

            list = list.push(
                button(
                    column![
                        row![
                            selection_indicator(is_selected),
                            text(key_display)
                                .size(14)
                                .color(if is_selected { [1.0, 1.0, 1.0] } else { [0.9, 0.9, 0.9] }),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(action_preview)
                            .size(11)
                            .color([0.75, 0.75, 0.75]),
                    ]
                    .spacing(2)
                )
                .on_press(Message::Keybindings(KeybindingsMessage::SelectKeybinding(idx)))
                .padding([8, 12])
                .width(Length::Fill)
                .style(list_item_style(is_selected))
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty detail view shown when no keybinding is selected
fn empty_detail_view() -> Element<'static, Message> {
    empty_detail_placeholder(
        "Select a keybinding to edit",
        "Or click + to add a new one",
    )
}

/// Detail view for a selected keybinding
fn keybinding_detail_view<'a>(
    binding: &'a Keybinding,
    idx: usize,
    sections_expanded: &HashMap<String, bool>,
    key_capture_active: Option<usize>,
) -> Element<'a, Message> {
    let basic_expanded = sections_expanded.get("basic").copied().unwrap_or(true);
    let advanced_expanded = sections_expanded.get("advanced").copied().unwrap_or(false);

    let is_capturing = key_capture_active == Some(idx);

    let mut content = column![
        // Header with delete button
        row![
            text("Edit Keybinding").size(20),
            delete_button(Message::Keybindings(KeybindingsMessage::RemoveKeybinding(idx))),
        ]
        .spacing(20)
        .align_y(Alignment::Center),
        spacer(16.0),
    ];

    // Key Combination Section
    content = content.push(expandable_section(
        "Key Combination",
        basic_expanded,
        Message::Keybindings(KeybindingsMessage::ToggleSection("basic".to_string())),
        column![
            // Key capture area
            key_capture_display(binding, idx, is_capturing),
            spacer(8.0),
            info_text("Click the button above to capture a new key combination"),
            spacer(12.0),
            // Modifier toggles
            text("Modifiers").size(14),
            modifier_toggles(binding, idx),
            info_text("Toggle modifiers to quickly adjust the key combination"),
        ]
        .spacing(8),
    ));

    // Action Section
    content = content.push(spacer(12.0));
    content = content.push(
        column![
            section_header("Action"),
            action_editor(binding, idx),
        ]
        .spacing(8)
    );

    // Advanced Options Section
    content = content.push(spacer(12.0));
    content = content.push(expandable_section(
        "Advanced Options",
        advanced_expanded,
        Message::Keybindings(KeybindingsMessage::ToggleSection("advanced".to_string())),
        column![
            // Overlay title
            column![
                text("Hotkey Overlay Title").size(14),
                text("Optional title shown in niri's hotkey overlay").size(11).color([0.75, 0.75, 0.75]),
                text_input(
                    "Leave empty for auto-generated",
                    binding.hotkey_overlay_title.as_deref().unwrap_or("")
                )
                .on_input(move |value| {
                    let title = if value.is_empty() { None } else { Some(value) };
                    Message::Keybindings(KeybindingsMessage::SetHotkeyOverlayTitle(idx, title))
                })
                .padding(8)
                .width(Length::Fill),
            ]
            .spacing(4),
            spacer(8.0),
            // Allow when locked toggle
            row![
                column![
                    text("Allow when locked").size(14),
                    text("Binding works even when screen is locked").size(11).color([0.75, 0.75, 0.75]),
                ]
                .width(Length::Fill),
                toggler(binding.allow_when_locked)
                    .on_toggle(move |value| Message::Keybindings(KeybindingsMessage::SetAllowWhenLocked(idx, value))),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            spacer(8.0),
            // Repeat toggle
            row![
                column![
                    text("Repeat when held").size(14),
                    text("Action repeats while key is held down").size(11).color([0.75, 0.75, 0.75]),
                ]
                .width(Length::Fill),
                toggler(binding.repeat)
                    .on_toggle(move |value| Message::Keybindings(KeybindingsMessage::SetRepeat(idx, value))),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            spacer(8.0),
            // Cooldown
            if let Some(cooldown) = binding.cooldown_ms {
                column![
                    text("Cooldown").size(14),
                    text(format!("{} ms between activations", cooldown)).size(11).color([0.75, 0.75, 0.75]),
                ]
            } else {
                column![
                    text("Cooldown").size(14),
                    text("No cooldown set").size(11).color([0.75, 0.75, 0.75]),
                ]
            },
        ]
        .spacing(8),
    ));

    scrollable(content).height(Length::Fill).into()
}

/// Key capture display and button
fn key_capture_display<'a>(
    binding: &'a Keybinding,
    idx: usize,
    is_capturing: bool,
) -> Element<'a, Message> {
    if is_capturing {
        container(
            button(
                text("Press any key combination... (ESC to cancel)")
                    .size(16)
                    .color([0.0, 0.0, 0.0])
            )
            .on_press(Message::Keybindings(KeybindingsMessage::CancelKeyCapture))
            .padding([12, 20])
            .width(Length::Fill)
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.9, 0.7, 0.2))),
                text_color: iced::Color::BLACK,
                border: iced::Border {
                    color: iced::Color::from_rgb(1.0, 0.8, 0.3),
                    width: 2.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
        )
        .width(Length::Fill)
        .into()
    } else if binding.key_combo.is_empty() {
        container(
            button(
                text("Click to set key combination")
                    .size(16)
            )
            .on_press(Message::Keybindings(KeybindingsMessage::StartKeyCapture(idx)))
            .padding([12, 20])
            .width(Length::Fill)
            .style(|_theme, status| {
                let bg = match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.3, 0.35, 0.4, 0.8),
                    button::Status::Pressed => iced::Color::from_rgba(0.35, 0.4, 0.45, 0.8),
                    _ => iced::Color::from_rgba(0.2, 0.25, 0.3, 0.8),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.4, 0.45, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .width(Length::Fill)
        .into()
    } else {
        // Borrow from binding.key_combo which has lifetime 'a
        container(
            button(
                text(&binding.key_combo)
                    .size(16)
            )
            .on_press(Message::Keybindings(KeybindingsMessage::StartKeyCapture(idx)))
            .padding([12, 20])
            .width(Length::Fill)
            .style(|_theme, status| {
                let bg = match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.3, 0.35, 0.4, 0.8),
                    button::Status::Pressed => iced::Color::from_rgba(0.35, 0.4, 0.45, 0.8),
                    _ => iced::Color::from_rgba(0.2, 0.25, 0.3, 0.8),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.4, 0.45, 0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        )
        .width(Length::Fill)
        .into()
    }
}

/// Action type editor
fn action_editor<'a>(binding: &'a Keybinding, idx: usize) -> Element<'a, Message> {
    let action_options: Vec<&str> = COMMON_ACTIONS.to_vec();

    // Determine current action for pick_list selection
    let current_action: &str = match &binding.action {
        KeybindAction::Spawn(_) => "spawn",
        KeybindAction::NiriAction(action) => {
            // Find if action is in our list
            COMMON_ACTIONS.iter()
                .find(|&&a| a == action.as_str())
                .copied()
                .unwrap_or("close-window")
        }
        KeybindAction::NiriActionWithArgs(action, _) => {
            COMMON_ACTIONS.iter()
                .find(|&&a| a == action.as_str())
                .copied()
                .unwrap_or("close-window")
        }
    };

    let is_spawn = matches!(&binding.action, KeybindAction::Spawn(_));

    let mut content = column![
        // Action type selector using pick_list
        row![
            text("Action:").size(14).width(Length::Fixed(80.0)),
            pick_list(
                action_options,
                Some(current_action),
                move |selected: &str| {
                    Message::Keybindings(KeybindingsMessage::UpdateAction(idx, selected.to_string()))
                }
            )
            .width(Length::Fixed(200.0))
            .padding(8),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    // Command input for spawn actions
    if is_spawn {
        if let KeybindAction::Spawn(args) = &binding.action {
            let cmd_display = args.join(" ");
            content = content.push(spacer(8.0));
            content = content.push(
                row![
                    text("Command:").size(14).width(Length::Fixed(80.0)),
                    text_input("Enter command...", &cmd_display)
                        .on_input(move |value| {
                            Message::Keybindings(KeybindingsMessage::SetCommand(idx, value))
                        })
                        .padding(8)
                        .width(Length::Fill),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            );
            content = content.push(
                info_text("Enter the command to run (e.g., 'alacritty' or 'firefox --new-window')")
            );
        }
    }

    content.into()
}

/// Parse modifiers from a key combo string
fn parse_modifiers_from_combo(key_combo: &str) -> Vec<ModKey> {
    let mut modifiers = Vec::new();

    for part in key_combo.split('+') {
        let trimmed = part.trim();
        match trimmed.to_lowercase().as_str() {
            "mod" | "super" => {
                if !modifiers.contains(&ModKey::Super) {
                    modifiers.push(ModKey::Super);
                }
            }
            "ctrl" | "control" => {
                if !modifiers.contains(&ModKey::Ctrl) {
                    modifiers.push(ModKey::Ctrl);
                }
            }
            "shift" => {
                if !modifiers.contains(&ModKey::Shift) {
                    modifiers.push(ModKey::Shift);
                }
            }
            "alt" => {
                if !modifiers.contains(&ModKey::Alt) {
                    modifiers.push(ModKey::Alt);
                }
            }
            _ => {} // Not a modifier we handle here, skip
        }
    }

    modifiers
}

/// Modifier toggle buttons for quick editing
fn modifier_toggles<'a>(binding: &'a Keybinding, idx: usize) -> Element<'a, Message> {
    let current_mods = parse_modifiers_from_combo(&binding.key_combo);

    let has_mod = current_mods.contains(&ModKey::Super);
    let has_ctrl = current_mods.contains(&ModKey::Ctrl);
    let has_shift = current_mods.contains(&ModKey::Shift);
    let has_alt = current_mods.contains(&ModKey::Alt);

    row![
        modifier_toggle_button("Mod", has_mod, idx, ModKey::Super, &current_mods),
        modifier_toggle_button("Ctrl", has_ctrl, idx, ModKey::Ctrl, &current_mods),
        modifier_toggle_button("Shift", has_shift, idx, ModKey::Shift, &current_mods),
        modifier_toggle_button("Alt", has_alt, idx, ModKey::Alt, &current_mods),
    ]
    .spacing(8)
    .into()
}

/// Single modifier toggle button
fn modifier_toggle_button<'a>(
    label: &'a str,
    is_active: bool,
    idx: usize,
    modifier: ModKey,
    current_mods: &[ModKey],
) -> Element<'a, Message> {
    // Build new modifiers list by toggling this modifier
    let new_mods: Vec<ModKey> = if is_active {
        current_mods.iter().filter(|m| **m != modifier).cloned().collect()
    } else {
        let mut mods = current_mods.to_vec();
        mods.push(modifier);
        mods
    };

    button(
        text(label)
            .size(13)
            .color(if is_active { [1.0, 1.0, 1.0] } else { [0.6, 0.6, 0.6] })
    )
    .on_press(Message::Keybindings(KeybindingsMessage::UpdateModifiers(idx, new_mods)))
    .padding([6, 12])
    .style(move |_theme, status| {
        let bg = if is_active {
            match status {
                button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.7),
                button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.8),
                _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.6),
            }
        } else {
            match status {
                button::Status::Hovered => iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                button::Status::Pressed => iced::Color::from_rgba(0.35, 0.35, 0.35, 0.6),
                _ => iced::Color::from_rgba(0.2, 0.2, 0.2, 0.4),
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: if is_active { iced::Color::WHITE } else { iced::Color::from_rgb(0.6, 0.6, 0.6) },
            border: iced::Border {
                color: if is_active {
                    iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5)
                } else {
                    iced::Color::from_rgba(0.3, 0.3, 0.3, 0.3)
                },
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
