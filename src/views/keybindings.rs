//! Keybindings settings view - list-detail implementation with key capture
//!
//! Provides a visual editor for keyboard shortcuts with:
//! - List of all keybindings
//! - Key capture widget for setting key combinations
//! - Action type selection (spawn command, niri action)
//! - Advanced options (cooldown, repeat, allow when locked)

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space,
};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{KeybindAction, Keybinding, KeybindingsSettings};
use crate::messages::{KeybindingsMessage, Message};
use crate::theme::{fonts, neon};
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
    "focus-column-first",
    "focus-column-last",
    "focus-window-up",
    "focus-window-down",
    "focus-window-or-workspace-up",
    "focus-window-or-workspace-down",
    "move-column-left",
    "move-column-right",
    "move-column-to-first",
    "move-column-to-last",
    "move-window-up",
    "move-window-down",
    "move-window-to-workspace-up",
    "move-window-to-workspace-down",
    "focus-workspace-down",
    "focus-workspace-up",
    "focus-workspace-previous",
    "move-workspace-down",
    "move-workspace-up",
    "move-column-to-workspace-down",
    "move-column-to-workspace-up",
    "consume-window-into-column",
    "expel-window-from-column",
    "center-column",
    "maximize-column",
    "fullscreen-window",
    "switch-preset-column-width",
    "switch-preset-window-height",
    "reset-window-height",
    "set-column-width",
    "set-window-height",
    "power-off-monitors",
    "suspend",
    "toggle-window-floating",
    "switch-focus-between-floating-and-tiling",
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
    let mut list = column![row![
        text("Keybindings").size(18),
        add_button(Message::Keybindings(KeybindingsMessage::AddKeybinding)),
    ]
    .spacing(10)
    .padding([12, 20])
    .align_y(Alignment::Center),]
    .spacing(0);

    // Show error if loading failed
    if let Some(error) = &settings.error {
        list = list.push(
            container(
                text(format!("Error loading keybindings:\n{}", error))
                    .size(12)
                    .color([0.9, 0.4, 0.4]),
            )
            .padding(12),
        );
    }

    if settings.bindings.is_empty() {
        list = list.push(empty_list_placeholder(
            "No keybindings configured\nClick + to add one",
        ));
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
                            text(key_display).size(14).color(if is_selected {
                                [1.0, 1.0, 1.0]
                            } else {
                                [0.9, 0.9, 0.9]
                            }),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(action_preview).size(11).color([0.75, 0.75, 0.75]),
                    ]
                    .spacing(2),
                )
                .on_press(Message::Keybindings(KeybindingsMessage::SelectKeybinding(
                    idx,
                )))
                .padding([8, 12])
                .width(Length::Fill)
                .style(list_item_style(is_selected)),
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty detail view shown when no keybinding is selected
fn empty_detail_view() -> Element<'static, Message> {
    empty_detail_placeholder("Select a keybinding to edit", "Or click + to add a new one")
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
            delete_button(Message::Keybindings(KeybindingsMessage::RemoveKeybinding(
                idx
            ))),
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
    content =
        content.push(column![section_header("Action"), action_editor(binding, idx),].spacing(8));

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
                text("Optional title shown in niri's hotkey overlay")
                    .size(11)
                    .color([0.75, 0.75, 0.75]),
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
                    text("Binding works even when screen is locked")
                        .size(11)
                        .color([0.75, 0.75, 0.75]),
                ]
                .width(Length::Fill),
                toggler(binding.allow_when_locked).on_toggle(move |value| Message::Keybindings(
                    KeybindingsMessage::SetAllowWhenLocked(idx, value)
                )),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            spacer(8.0),
            // Repeat toggle
            row![
                column![
                    text("Repeat when held").size(14),
                    text("Action repeats while key is held down")
                        .size(11)
                        .color([0.75, 0.75, 0.75]),
                ]
                .width(Length::Fill),
                toggler(binding.repeat).on_toggle(move |value| Message::Keybindings(
                    KeybindingsMessage::SetRepeat(idx, value)
                )),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            spacer(8.0),
            // Cooldown
            if let Some(cooldown) = binding.cooldown_ms {
                column![
                    text("Cooldown").size(14),
                    text(format!("{} ms between activations", cooldown))
                        .size(11)
                        .color([0.75, 0.75, 0.75]),
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
                    .color([0.0, 0.0, 0.0]),
            )
            .on_press(Message::Keybindings(KeybindingsMessage::CancelKeyCapture))
            .padding([12, 20])
            .width(Length::Fill)
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.9, 0.7, 0.2,
                ))),
                text_color: iced::Color::BLACK,
                border: iced::Border {
                    color: iced::Color::from_rgb(1.0, 0.8, 0.3),
                    width: 2.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .into()
    } else if binding.key_combo.is_empty() {
        container(
            button(text("Click to set key combination").size(16))
                .on_press(Message::Keybindings(KeybindingsMessage::StartKeyCapture(
                    idx,
                )))
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
                }),
        )
        .width(Length::Fill)
        .into()
    } else {
        // Borrow from binding.key_combo which has lifetime 'a
        container(
            button(text(&binding.key_combo).size(16))
                .on_press(Message::Keybindings(KeybindingsMessage::StartKeyCapture(
                    idx,
                )))
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
                }),
        )
        .width(Length::Fill)
        .into()
    }
}

/// Action type editor
fn action_editor<'a>(binding: &'a Keybinding, idx: usize) -> Element<'a, Message> {
    // Get the actual action name
    let actual_action: &str = match &binding.action {
        KeybindAction::Spawn(_) => "spawn",
        KeybindAction::NiriAction(action) => action.as_str(),
        KeybindAction::NiriActionWithArgs(action, _) => action.as_str(),
    };

    // Build action options, including the actual action if not in common list
    let mut action_options: Vec<&str> = COMMON_ACTIONS.to_vec();
    let is_custom_action = !action_options.contains(&actual_action);
    if is_custom_action {
        // Insert the custom action at the beginning so it's visible
        action_options.insert(0, actual_action);
    }

    // Current action is always the actual action (never default to close-window)
    let current_action: &str = actual_action;

    let is_spawn = matches!(&binding.action, KeybindAction::Spawn(_));

    let mut content = column![
        // Action type selector using pick_list
        row![
            text("Action:").size(14).width(Length::Fixed(80.0)),
            pick_list(
                action_options,
                Some(current_action),
                move |selected: &str| {
                    Message::Keybindings(KeybindingsMessage::UpdateAction(
                        idx,
                        selected.to_string(),
                    ))
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
                .align_y(Alignment::Center),
            );
            content = content.push(info_text(
                "Enter the command to run (e.g., 'alacritty' or 'firefox --new-window')",
            ));
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
        current_mods
            .iter()
            .filter(|m| **m != modifier)
            .cloned()
            .collect()
    } else {
        let mut mods = current_mods.to_vec();
        mods.push(modifier);
        mods
    };

    let color = if is_active {
        neon::SECONDARY
    } else {
        neon::OUTLINE_VARIANT
    };
    button(
        text(label)
            .size(12)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(color),
    )
    .on_press(Message::Keybindings(KeybindingsMessage::UpdateModifiers(
        idx, new_mods,
    )))
    .padding([6, 14])
    .style(move |_theme: &iced::Theme, status| {
        let bg = match status {
            button::Status::Hovered => iced::Color { a: 0.15, ..color },
            _ => iced::Color {
                a: if is_active { 0.10 } else { 0.05 },
                ..color
            },
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: color,
            border: iced::Border {
                color: iced::Color {
                    a: if is_active { 0.3 } else { 0.15 },
                    ..color
                },
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

// ── Keybinding Editor Modal ────────────────────────────────────────────────

/// Creates a modal overlay for editing a keybinding
pub fn editor_modal<'a>(
    binding: &'a Keybinding,
    idx: usize,
    sections_expanded: &'a HashMap<String, bool>,
    key_capture_active: Option<usize>,
) -> Element<'a, Message> {
    let is_capturing = key_capture_active == Some(idx);

    let actual_action: &str = match &binding.action {
        KeybindAction::Spawn(_) => "spawn",
        KeybindAction::NiriAction(action) => action.as_str(),
        KeybindAction::NiriActionWithArgs(action, _) => action.as_str(),
    };

    let mut action_options: Vec<&str> = COMMON_ACTIONS.to_vec();
    if !action_options.contains(&actual_action) {
        action_options.insert(0, actual_action);
    }

    let editor = column![
        // Header
        row![
            container(text("⌘").size(24).color(neon::PRIMARY))
                .width(48)
                .height(48)
                .center(Length::Shrink)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        a: 0.15,
                        ..neon::PRIMARY
                    })),
                    border: iced::Border {
                        radius: 14.0.into(),
                        color: iced::Color {
                            a: 0.25,
                            ..neon::PRIMARY
                        },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("KEYBINDING EDITOR")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::SECONDARY),
                text(format!("Modify: {}", binding.display_name()))
                    .size(22)
                    .font(fonts::UI_FONT_SEMIBOLD),
            ]
            .spacing(4)
            .width(Length::Fill),
            row![
                button(text("Delete").size(12).color(neon::ERROR))
                    .on_press(Message::Keybindings(KeybindingsMessage::RemoveKeybinding(
                        idx
                    )))
                    .padding([6, 12])
                    .style(ghost_button_style),
                button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                    .on_press(Message::CloseKeybindingEditor)
                    .padding([8, 12])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color {
                                a: 0.15,
                                ..neon::ON_SURFACE
                            },
                            _ => iced::Color {
                                a: 0.08,
                                ..neon::ON_SURFACE
                            },
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::ON_SURFACE,
                            border: iced::Border {
                                radius: 999.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .align_y(Alignment::Center),
        Space::new().height(20),
        // ── 2-COLUMN: KEY COMBO | ACTION ──
        row![
            // Left: Key Combination
            column![
                modal_section("⌨", "KEY COMBINATION", neon::SECONDARY),
                Space::new().height(8),
                container(
                    column![
                        text("CURRENT BINDING")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().height(6),
                        key_capture_display(binding, idx, is_capturing),
                        Space::new().height(14),
                        text("MODIFIERS")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().height(6),
                        modifier_toggles(binding, idx),
                    ]
                    .spacing(0)
                )
                .padding(12)
                .style(crate::theme::card_style),
            ]
            .spacing(0)
            .width(Length::FillPortion(1)),
            // Right: Action
            column![
                modal_section("⚡", "ACTION", neon::PRIMARY),
                Space::new().height(8),
                container(
                    column![
                        text("ACTION TYPE")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().height(6),
                        pick_list(
                            action_options,
                            Some(actual_action),
                            move |selected: &str| {
                                Message::Keybindings(KeybindingsMessage::UpdateAction(
                                    idx,
                                    selected.to_string(),
                                ))
                            }
                        )
                        .width(Length::Fill)
                        .padding(10),
                        {
                            let is_spawn = matches!(&binding.action, KeybindAction::Spawn(_));
                            if is_spawn {
                                let cmd = match &binding.action {
                                    KeybindAction::Spawn(args) => args.join(" "),
                                    _ => String::new(),
                                };
                                Element::from(
                                    column![
                                        Space::new().height(12),
                                        text("COMMAND")
                                            .size(10)
                                            .font(fonts::UI_FONT_SEMIBOLD)
                                            .color(neon::OUTLINE_VARIANT),
                                        Space::new().height(6),
                                        text_input("e.g., alacritty --new-window", &cmd)
                                            .on_input(move |v| Message::Keybindings(
                                                KeybindingsMessage::SetCommand(idx, v)
                                            ))
                                            .padding(10)
                                            .size(13),
                                    ]
                                    .spacing(0),
                                )
                            } else {
                                Space::new().into()
                            }
                        },
                    ]
                    .spacing(0)
                )
                .padding(12)
                .style(crate::theme::card_style),
            ]
            .spacing(0)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ADVANCED OPTIONS ──
        modal_section("⬡", "ADVANCED OPTIONS", neon::OUTLINE),
        Space::new().height(4),
        row![
            column![container(
                column![
                    toggle_row(
                        "Allow when locked",
                        "Works even when screen is locked",
                        binding.allow_when_locked,
                        move |v| Message::Keybindings(KeybindingsMessage::SetAllowWhenLocked(
                            idx, v
                        ))
                    ),
                    toggle_row(
                        "Repeat when held",
                        "Action repeats while key held",
                        binding.repeat,
                        move |v| Message::Keybindings(KeybindingsMessage::SetRepeat(idx, v))
                    ),
                ]
                .spacing(0)
            )
            .padding(8)
            .style(crate::theme::card_style),]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![container(
                column![
                    text("HOTKEY OVERLAY TITLE")
                        .size(10)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::OUTLINE_VARIANT),
                    Space::new().height(4),
                    text_input(
                        "Auto-generated if empty",
                        binding.hotkey_overlay_title.as_deref().unwrap_or("")
                    )
                    .on_input(move |v| {
                        let title = if v.is_empty() { None } else { Some(v) };
                        Message::Keybindings(KeybindingsMessage::SetHotkeyOverlayTitle(idx, title))
                    })
                    .padding(10)
                    .size(13),
                    Space::new().height(8),
                    text("COOLDOWN")
                        .size(10)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::OUTLINE_VARIANT),
                    Space::new().height(4),
                    text_input(
                        "ms between activations",
                        &binding
                            .cooldown_ms
                            .map(|c| c.to_string())
                            .unwrap_or_default()
                    )
                    .on_input(move |v| {
                        let cd = if v.is_empty() {
                            None
                        } else {
                            v.parse::<i32>().ok()
                        };
                        Message::Keybindings(KeybindingsMessage::SetCooldown(idx, cd))
                    })
                    .padding(10)
                    .size(13),
                ]
                .spacing(0)
                .padding(12)
            )
            .style(crate::theme::card_style),]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        // ── Footer ──
        Space::new().height(20),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        container(
            row![
                row![
                    text("●").size(10).color(neon::SECONDARY),
                    text("Live Configuration Sync Active")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                button(text("Discard").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::CloseKeybindingEditor)
                    .padding([10, 20])
                    .style(ghost_button_style),
                Space::new().width(8),
                button(text("Save Changes").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::CloseKeybindingEditor)
                    .padding([10, 24])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            button::Status::Hovered => neon::PRIMARY,
                            _ => iced::Color {
                                a: 0.85,
                                ..neon::PRIMARY
                            },
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::SURFACE_LOW,
                            border: iced::Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .align_y(Alignment::Center)
        )
        .padding([16, 0]),
    ];

    let modal_content = scrollable(editor.spacing(0).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(900.0))
        .max_height(700.0)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color {
                    a: 0.3,
                    ..neon::PRIMARY
                },
                width: 2.0,
                radius: 20.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}

fn modal_section<'a>(icon: &'a str, label: &'a str, accent: iced::Color) -> Element<'a, Message> {
    row![
        text(icon).size(14).color(accent),
        Space::new().width(6),
        text(label)
            .size(11)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(accent),
        Space::new().width(12),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.25, ..accent })),
                ..Default::default()
            }),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding([14, 0])
    .into()
}

fn ghost_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color {
            a: 0.08,
            ..neon::ON_SURFACE
        },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: neon::ON_SURFACE,
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
