//! Keybindings settings view - list-detail implementation with key capture
//!
//! Provides a visual editor for keyboard shortcuts with:
//! - List of all keybindings (with duplicate badges)
//! - Key capture widget for setting key combinations
//! - Two-level action picker (category → action) over the full niri catalog
//! - Typed argument widgets per action, with inline validation
//! - Advanced options (cooldown, repeat, allow-inhibiting, allow-when-locked,
//!   overlay title)

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space,
};
use iced::{Alignment, Element, Length};
use std::collections::{HashMap, HashSet};

use super::widgets::*;
use crate::config::models::{
    actions_in_category, lookup_action, normalized_key_combo, ActionCategory, ActionNode,
    ActionSpec, ActionValue, ArgKind, HotkeyOverlayTitle, KeybindAction, Keybinding,
    KeybindingsSettings, ScreenshotKind,
};
use crate::config::parser::parse_document;
use crate::messages::{KeybindingsMessage, Message};
use crate::theme::{fonts, neon};
use crate::types::ModKey;
use crate::version::NiriVersion;

/// Wrapper so an action can be shown in a pick_list by its humanized label.
#[derive(Clone, Copy)]
struct ActionOption(&'static ActionSpec);

impl PartialEq for ActionOption {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl std::fmt::Display for ActionOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.label())
    }
}

/// 3-way choice for the hotkey overlay title.
#[derive(Clone, Copy, PartialEq)]
enum OverlayChoice {
    Auto,
    Hidden,
    Custom,
}

impl std::fmt::Display for OverlayChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OverlayChoice::Auto => "Automatic",
            OverlayChoice::Hidden => "Hidden from overlay",
            OverlayChoice::Custom => "Custom…",
        };
        write!(f, "{}", s)
    }
}

// ── Small text helpers ─────────────────────────────────────────────────────

fn error_text<'a>(msg: &'a str) -> Element<'a, Message> {
    text(msg).size(11).color([0.95, 0.45, 0.45]).into()
}

fn labeled_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: String,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(13).color([0.85, 0.85, 0.85]),
        text_input(placeholder, &value)
            .on_input(on_input)
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .into()
}

// ── Category / action introspection ────────────────────────────────────────

fn current_category(action: &KeybindAction) -> ActionCategory {
    match action {
        KeybindAction::Spawn(_) | KeybindAction::SpawnSh(_) => ActionCategory::Run,
        KeybindAction::Custom(_) => ActionCategory::Custom,
        KeybindAction::NiriAction(node) => lookup_action(&node.name)
            .map(|s| s.category)
            .unwrap_or(ActionCategory::System),
    }
}

/// The ArgKind for the current action (Spawn/SpawnSh handled specially).
fn arg_kind_of(action: &KeybindAction) -> ArgKind {
    match action {
        KeybindAction::Spawn(_) => ArgKind::SpawnCmd,
        KeybindAction::SpawnSh(_) => ArgKind::SpawnShCmd,
        KeybindAction::Custom(_) => ArgKind::None,
        KeybindAction::NiriAction(node) => lookup_action(&node.name)
            .map(|s| s.args)
            .unwrap_or(ArgKind::None),
    }
}

/// Set of binding indices that are duplicates of an earlier binding.
fn duplicate_indices(settings: &KeybindingsSettings) -> HashSet<usize> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut dups = HashSet::new();
    for (idx, b) in settings.bindings.iter().enumerate() {
        if b.key_combo.trim().is_empty() {
            continue;
        }
        let norm = normalized_key_combo(&b.key_combo);
        if !seen.insert(norm) {
            dups.insert(idx);
        }
    }
    dups
}

// ── Top-level view ─────────────────────────────────────────────────────────

/// Creates the keybindings settings view with list-detail pattern
#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    settings: &'a KeybindingsSettings,
    selected_index: Option<usize>,
    sections_expanded: &'a HashMap<String, bool>,
    key_capture_active: Option<usize>,
    niri_version: Option<NiriVersion>,
    capture_conflict: Option<&'a (usize, String, String)>,
) -> Element<'a, Message> {
    let dups = duplicate_indices(settings);
    let list_panel = keybinding_list(settings, selected_index, &dups);

    let detail_panel = if let Some(idx) = selected_index {
        if let Some(binding) = settings.bindings.get(idx) {
            keybinding_detail_view(
                binding,
                idx,
                sections_expanded,
                key_capture_active,
                niri_version,
                capture_conflict,
            )
        } else {
            empty_detail_view()
        }
    } else {
        empty_detail_view()
    };

    list_detail_layout(list_panel, detail_panel)
}

/// List panel showing all keybindings
fn keybinding_list<'a>(
    settings: &'a KeybindingsSettings,
    selected_index: Option<usize>,
    dups: &HashSet<usize>,
) -> Element<'a, Message> {
    let mut list = column![row![
        text("Keybindings").size(18),
        add_button(Message::Keybindings(KeybindingsMessage::AddKeybinding)),
    ]
    .spacing(10)
    .padding([12, 20])
    .align_y(Alignment::Center),]
    .spacing(0);

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

            let key_display = if binding.key_combo.is_empty() {
                "(no key set)".to_string()
            } else {
                binding.key_combo.clone()
            };

            let action_preview = binding.action.description();

            let mut inner = column![
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
            .spacing(2);

            if dups.contains(&idx) {
                inner = inner.push(
                    text("Duplicate shortcut — only the first will be saved")
                        .size(10)
                        .color([0.95, 0.6, 0.3]),
                );
            }

            list = list.push(
                button(inner)
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

fn empty_detail_view() -> Element<'static, Message> {
    empty_detail_placeholder("Select a keybinding to edit", "Or click + to add a new one")
}

/// Detail view for a selected keybinding
fn keybinding_detail_view<'a>(
    binding: &'a Keybinding,
    idx: usize,
    sections_expanded: &HashMap<String, bool>,
    key_capture_active: Option<usize>,
    niri_version: Option<NiriVersion>,
    capture_conflict: Option<&'a (usize, String, String)>,
) -> Element<'a, Message> {
    let basic_expanded = sections_expanded.get("basic").copied().unwrap_or(true);
    let advanced_expanded = sections_expanded.get("advanced").copied().unwrap_or(false);

    let is_capturing = key_capture_active == Some(idx);

    let mut content = column![
        row![
            text("Edit Keybinding").size(20),
            delete_button(Message::ShowDialog(crate::messages::DialogState::Confirm {
                title: "Delete keybinding?".to_string(),
                message: format!(
                    "Delete the keybinding \"{}\"? This cannot be undone.",
                    binding.display_name()
                ),
                confirm_label: "Delete".to_string(),
                on_confirm: crate::messages::ConfirmAction::DeleteKeybinding(idx),
            })),
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
            key_capture_display(binding, idx, is_capturing, capture_conflict),
            spacer(8.0),
            info_text("Click the button above to capture a new key combination"),
            spacer(12.0),
            text("Modifiers").size(14),
            modifier_toggles(binding, idx),
        ]
        .spacing(8),
    ));

    // Action Section
    content = content.push(spacer(12.0));
    content = content.push(
        column![
            section_header("Action"),
            action_editor(binding, idx, niri_version),
        ]
        .spacing(8),
    );

    // Advanced Options Section
    content = content.push(spacer(12.0));
    content = content.push(expandable_section(
        "Advanced Options",
        advanced_expanded,
        Message::Keybindings(KeybindingsMessage::ToggleSection("advanced".to_string())),
        advanced_options(binding, idx),
    ));

    scrollable(content).height(Length::Fill).into()
}

// ── Key capture ────────────────────────────────────────────────────────────

fn key_capture_display<'a>(
    binding: &'a Keybinding,
    idx: usize,
    is_capturing: bool,
    capture_conflict: Option<&'a (usize, String, String)>,
) -> Element<'a, Message> {
    let button_el: Element<'a, Message> = if is_capturing {
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
        })
        .into()
    } else {
        let label = if binding.key_combo.is_empty() {
            "Click to set key combination".to_string()
        } else {
            binding.key_combo.clone()
        };
        button(text(label).size(16))
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
            })
            .into()
    };

    let mut col = column![button_el].spacing(6);

    // Inline duplicate-capture warning
    if let Some((cidx, combo, name)) = capture_conflict {
        if *cidx == idx {
            col = col.push(error_text_owned(format!(
                "‘{}’ is already used by ‘{}’ — press another key or Esc",
                combo, name
            )));
        }
    }

    container(col).width(Length::Fill).into()
}

fn error_text_owned<'a>(msg: String) -> Element<'a, Message> {
    text(msg).size(11).color([0.95, 0.45, 0.45]).into()
}

// ── Action editor (category + action picker + typed args) ──────────────────

fn action_editor<'a>(
    binding: &'a Keybinding,
    idx: usize,
    niri_version: Option<NiriVersion>,
) -> Element<'a, Message> {
    let category = current_category(&binding.action);

    let category_picker = row![
        text("Category:").size(14).width(Length::Fixed(90.0)),
        pick_list(ActionCategory::ALL.to_vec(), Some(category), move |cat| {
            Message::Keybindings(KeybindingsMessage::SelectActionCategory(idx, cat))
        })
        .width(Length::Fixed(240.0))
        .padding(8),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let mut content = column![category_picker].spacing(8);

    if category != ActionCategory::Custom {
        let options: Vec<ActionOption> = actions_in_category(category)
            .into_iter()
            .map(ActionOption)
            .collect();
        let current = lookup_action(&binding.action.name()).map(ActionOption);
        content = content.push(
            row![
                text("Action:").size(14).width(Length::Fixed(90.0)),
                pick_list(options, current, move |opt: ActionOption| {
                    Message::Keybindings(KeybindingsMessage::UpdateAction(
                        idx,
                        opt.0.name.to_string(),
                    ))
                })
                .width(Length::Fixed(240.0))
                .padding(8),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        );
    }

    content = content.push(action_arg_editor(binding, idx, niri_version));
    content.into()
}

/// Typed argument widgets for the current action.
fn action_arg_editor<'a>(
    binding: &'a Keybinding,
    idx: usize,
    niri_version: Option<NiriVersion>,
) -> Element<'a, Message> {
    match &binding.action {
        KeybindAction::Spawn(args) => labeled_input(
            "Command",
            "e.g. alacritty --new-window",
            args.join(" "),
            move |v| Message::Keybindings(KeybindingsMessage::SetCommand(idx, v)),
        ),
        KeybindAction::SpawnSh(cmd) => labeled_input(
            "Shell command",
            "e.g. pkill orca || exec orca",
            cmd.clone(),
            move |v| Message::Keybindings(KeybindingsMessage::SetSpawnShCommand(idx, v)),
        ),
        KeybindAction::Custom(raw) => {
            let mut col = column![labeled_input(
                "Custom action (KDL)",
                "e.g. focus-workspace \"web\"",
                raw.clone(),
                move |v| Message::Keybindings(KeybindingsMessage::SetCustomActionText(idx, v)),
            )]
            .spacing(4);
            col = col.push(info_text("Advanced: written to the config as-is."));
            if !raw.trim().is_empty() {
                let ok = parse_document(raw)
                    .map(|d| d.nodes().len() == 1)
                    .unwrap_or(false);
                if !ok {
                    col = col.push(error_text(
                        "Must be exactly one valid KDL action node, or it won't be saved.",
                    ));
                }
            }
            col.into()
        }
        KeybindAction::NiriAction(node) => {
            niri_arg_widget(node, idx, arg_kind_of(&binding.action), niri_version)
        }
    }
}

fn primary_input<'a>(
    node: &ActionNode,
    idx: usize,
    label: &'a str,
    placeholder: &'a str,
) -> Element<'a, Message> {
    labeled_input(label, placeholder, node.primary_arg_display(), move |v| {
        Message::Keybindings(KeybindingsMessage::SetActionArgText(idx, v))
    })
}

fn required_error<'a>() -> Element<'a, Message> {
    error_text("This action requires a value — the shortcut won't be saved until it's set.")
}

fn flag_toggle<'a>(
    label: &'a str,
    desc: &'a str,
    value: bool,
    msg: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    toggle_row(label, desc, value, msg)
}

fn niri_arg_widget<'a>(
    node: &'a ActionNode,
    idx: usize,
    kind: ArgKind,
    niri_version: Option<NiriVersion>,
) -> Element<'a, Message> {
    let primary_missing = node
        .primary_arg()
        .map(|v| v.as_display().trim().is_empty())
        .unwrap_or(true);

    match kind {
        ArgKind::None | ArgKind::SpawnCmd | ArgKind::SpawnShCmd => Space::new().into(),

        ArgKind::SizeChange => {
            let mut col =
                column![primary_input(node, idx, "Size", "e.g. +10%, -50, 500, 50%")].spacing(4);
            let val = node.primary_arg_display();
            if primary_missing {
                col = col.push(required_error());
            } else if !crate::config::models::is_valid_size_change(&val) {
                col = col.push(error_text("Enter a size like +10%, -50, 500 or 50%."));
            }
            col.into()
        }

        ArgKind::IndexInt => {
            let mut col = column![primary_input(node, idx, "Index", "1 or greater")].spacing(4);
            if primary_missing {
                col = col.push(required_error());
            } else if let Some(ActionValue::Int(n)) = node.primary_arg() {
                if *n < 1 {
                    col = col.push(error_text("Index must be 1 or greater."));
                }
            }
            col.into()
        }

        ArgKind::WorkspaceRef => {
            let mut col = column![primary_input(
                node,
                idx,
                "Workspace",
                "index (1) or name (browser)"
            )]
            .spacing(4);
            if primary_missing {
                col = col.push(required_error());
            }
            col.into()
        }

        ArgKind::WorkspaceRefFocus => {
            let focus = node.get_prop("focus") != Some(&ActionValue::Bool(false));
            let mut col = column![
                primary_input(node, idx, "Workspace", "index (1) or name (browser)"),
                flag_toggle(
                    "Focus after moving",
                    "Move focus to the target workspace",
                    focus,
                    move |v| Message::Keybindings(KeybindingsMessage::SetActionFocusFlag(idx, v))
                ),
            ]
            .spacing(8);
            if primary_missing {
                col = col.push(required_error());
            }
            col.into()
        }

        ArgKind::FocusFlag => {
            let focus = node.get_prop("focus") != Some(&ActionValue::Bool(false));
            flag_toggle(
                "Focus after moving",
                "Move focus to the target workspace",
                focus,
                move |v| Message::Keybindings(KeybindingsMessage::SetActionFocusFlag(idx, v)),
            )
        }

        ArgKind::OutputName => {
            let mut col = column![primary_input(node, idx, "Monitor", "e.g. eDP-1")].spacing(4);
            if primary_missing {
                col = col.push(required_error());
            }
            col.into()
        }

        ArgKind::OptionalOutputName => {
            primary_input(node, idx, "Monitor (optional)", "e.g. HDMI-A-1")
        }

        ArgKind::NameString => {
            let mut col =
                column![primary_input(node, idx, "Workspace name", "e.g. browser")].spacing(4);
            if primary_missing {
                col = col.push(required_error());
            }
            col.into()
        }

        ArgKind::ColumnDisplay => {
            let current = node.primary_arg_display();
            let current = if current.is_empty() {
                "normal".to_string()
            } else {
                current
            };
            column![
                text("Display").size(13).color([0.85, 0.85, 0.85]),
                pick_list(
                    vec!["normal".to_string(), "tabbed".to_string()],
                    Some(current),
                    move |v: String| Message::Keybindings(KeybindingsMessage::SetActionArgText(
                        idx, v
                    ))
                )
                .width(Length::Fixed(200.0))
                .padding(8),
            ]
            .spacing(4)
            .into()
        }

        ArgKind::LayoutTarget => column![
            primary_input(node, idx, "Target", "next, prev, or an index"),
            info_text("Enter 'next', 'prev', or a keyboard layout index."),
        ]
        .spacing(4)
        .into(),

        ArgKind::DelayMs => {
            let val = match node.get_prop("delay-ms") {
                Some(v) => v.as_display(),
                None => String::new(),
            };
            labeled_input("Delay (ms)", "optional, e.g. 100", val, move |v| {
                let parsed = if v.trim().is_empty() {
                    None
                } else {
                    v.trim().parse::<u16>().ok()
                };
                Message::Keybindings(KeybindingsMessage::SetActionDelayMs(idx, parsed))
            })
        }

        ArgKind::QuitFlags => {
            let skip = node.get_prop("skip-confirmation") == Some(&ActionValue::Bool(true));
            flag_toggle(
                "Skip confirmation",
                "Quit immediately without the confirmation prompt",
                skip,
                move |v| {
                    Message::Keybindings(KeybindingsMessage::SetActionSkipConfirmation(idx, v))
                },
            )
        }

        ArgKind::ScreenshotFlags(skind) => screenshot_flags(node, idx, skind, niri_version),
    }
}

fn screenshot_flags<'a>(
    node: &'a ActionNode,
    idx: usize,
    skind: ScreenshotKind,
    niri_version: Option<NiriVersion>,
) -> Element<'a, Message> {
    let mut col = column![].spacing(8);

    // write-to-disk (default true) for screen/window
    if matches!(skind, ScreenshotKind::Screen | ScreenshotKind::Window) {
        let wtd = node.get_prop("write-to-disk") != Some(&ActionValue::Bool(false));
        col = col.push(flag_toggle(
            "Write to disk",
            "Also save the screenshot to a file",
            wtd,
            move |v| Message::Keybindings(KeybindingsMessage::SetActionWriteToDisk(idx, v)),
        ));
    }

    // show-pointer
    match skind {
        ScreenshotKind::Region | ScreenshotKind::Screen => {
            let sp = node.get_prop("show-pointer") != Some(&ActionValue::Bool(false));
            col = col.push(flag_toggle(
                "Show pointer",
                "Include the mouse cursor in the screenshot",
                sp,
                move |v| Message::Keybindings(KeybindingsMessage::SetActionShowPointer(idx, v)),
            ));
        }
        ScreenshotKind::Window => {
            // 26.04+ only
            let supported = niri_version.is_some_and(|v| v.at_least(26, 4));
            if supported {
                let sp = node.get_prop("show-pointer") == Some(&ActionValue::Bool(true));
                col = col.push(flag_toggle(
                    "Show pointer",
                    "Include the mouse cursor in the screenshot",
                    sp,
                    move |v| Message::Keybindings(KeybindingsMessage::SetActionShowPointer(idx, v)),
                ));
            } else {
                col = col.push(info_text("Show pointer requires niri 26.04"));
            }
        }
    }

    col.into()
}

// ── Advanced options ───────────────────────────────────────────────────────

fn advanced_options<'a>(binding: &'a Keybinding, idx: usize) -> Element<'a, Message> {
    let is_spawn = matches!(
        &binding.action,
        KeybindAction::Spawn(_) | KeybindAction::SpawnSh(_)
    );
    let is_inhibit_toggle = matches!(
        &binding.action,
        KeybindAction::NiriAction(n) if n.name == "toggle-keyboard-shortcuts-inhibit"
    );

    let mut col = column![].spacing(8);

    // Allow when locked (spawn-only)
    if is_spawn {
        col = col.push(toggle_row(
            "Allow when locked",
            "Binding works even when the screen is locked",
            binding.allow_when_locked,
            move |v| Message::Keybindings(KeybindingsMessage::SetAllowWhenLocked(idx, v)),
        ));
    } else {
        col = col.push(disabled_toggle_row(
            "Allow when locked",
            "Only available for Spawn actions",
            false,
        ));
    }

    // Allow inhibiting (default true)
    if is_inhibit_toggle {
        col = col.push(disabled_toggle_row(
            "Allow apps to inhibit this shortcut",
            "Always allowed to bypass inhibition",
            false,
        ));
    } else {
        col = col.push(toggle_row(
            "Allow apps to inhibit this shortcut",
            "Apps like remote-desktop clients may grab this shortcut",
            binding.allow_inhibiting,
            move |v| Message::Keybindings(KeybindingsMessage::SetAllowInhibiting(idx, v)),
        ));
    }

    // Repeat when held (default true)
    col = col.push(toggle_row(
        "Repeat when held",
        "Action repeats while the key is held down",
        binding.repeat,
        move |v| Message::Keybindings(KeybindingsMessage::SetRepeat(idx, v)),
    ));

    // Hotkey overlay title (3-way)
    col = col.push(spacer(4.0));
    col = col.push(overlay_title_editor(binding, idx));

    // Cooldown
    col = col.push(spacer(4.0));
    let cooldown = binding
        .cooldown_ms
        .map(|c| c.to_string())
        .unwrap_or_default();
    col = col.push(labeled_input(
        "Cooldown (ms)",
        "optional, e.g. 150",
        cooldown,
        move |v| {
            let cd = if v.trim().is_empty() {
                None
            } else {
                v.trim().parse::<i32>().ok()
            };
            Message::Keybindings(KeybindingsMessage::SetCooldown(idx, cd))
        },
    ));

    col.into()
}

fn disabled_toggle_row<'a>(label: &'a str, desc: &'a str, value: bool) -> Element<'a, Message> {
    row![
        column![
            text(label).size(15),
            text(desc).size(11).color([0.6, 0.6, 0.6]),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value), // no on_toggle => disabled
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn overlay_title_editor<'a>(binding: &'a Keybinding, idx: usize) -> Element<'a, Message> {
    let choice = match &binding.hotkey_overlay_title {
        HotkeyOverlayTitle::Auto => OverlayChoice::Auto,
        HotkeyOverlayTitle::Hidden => OverlayChoice::Hidden,
        HotkeyOverlayTitle::Custom(_) => OverlayChoice::Custom,
    };
    let custom_val = match &binding.hotkey_overlay_title {
        HotkeyOverlayTitle::Custom(s) => s.clone(),
        _ => String::new(),
    };
    let existing_custom = custom_val.clone();

    let mut col = column![
        text("Overlay label").size(14),
        pick_list(
            vec![
                OverlayChoice::Auto,
                OverlayChoice::Hidden,
                OverlayChoice::Custom
            ],
            Some(choice),
            move |c| {
                let title = match c {
                    OverlayChoice::Auto => HotkeyOverlayTitle::Auto,
                    OverlayChoice::Hidden => HotkeyOverlayTitle::Hidden,
                    OverlayChoice::Custom => HotkeyOverlayTitle::Custom(existing_custom.clone()),
                };
                Message::Keybindings(KeybindingsMessage::SetHotkeyOverlayTitle(idx, title))
            }
        )
        .width(Length::Fixed(240.0))
        .padding(8),
    ]
    .spacing(4);

    if choice == OverlayChoice::Custom {
        col = col.push(
            text_input("Title shown in the hotkey overlay", &custom_val)
                .on_input(move |v| {
                    Message::Keybindings(KeybindingsMessage::SetHotkeyOverlayTitle(
                        idx,
                        HotkeyOverlayTitle::Custom(v),
                    ))
                })
                .padding(8)
                .width(Length::Fill),
        );
    }

    col.into()
}

// ── Modifier toggles (unchanged behavior) ──────────────────────────────────

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
            "alt" if !modifiers.contains(&ModKey::Alt) => {
                modifiers.push(ModKey::Alt);
            }
            _ => {}
        }
    }
    modifiers
}

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

fn modifier_toggle_button<'a>(
    label: &'a str,
    is_active: bool,
    idx: usize,
    modifier: ModKey,
    current_mods: &[ModKey],
) -> Element<'a, Message> {
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

/// Creates a modal overlay for editing a keybinding.
#[allow(clippy::too_many_arguments)]
pub fn editor_modal<'a>(
    binding: &'a Keybinding,
    idx: usize,
    _sections_expanded: &'a HashMap<String, bool>,
    key_capture_active: Option<usize>,
    niri_version: Option<NiriVersion>,
    capture_conflict: Option<&'a (usize, String, String)>,
) -> Element<'a, Message> {
    let is_capturing = key_capture_active == Some(idx);

    let editor = column![
        // Header
        row![
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
            button(text("Delete").size(12).color(neon::ERROR))
                .on_press(Message::ShowDialog(crate::messages::DialogState::Confirm {
                    title: "Delete keybinding?".to_string(),
                    message: format!(
                        "Delete the keybinding \"{}\"? This cannot be undone.",
                        binding.display_name()
                    ),
                    confirm_label: "Delete".to_string(),
                    on_confirm: crate::messages::ConfirmAction::DeleteKeybinding(idx),
                }))
                .padding([6, 12])
                .style(ghost_button_style),
            Space::new().width(8),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::CloseKeybindingEditor)
                .padding([8, 12])
                .style(ghost_button_style),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        Space::new().height(20),
        // Key combination
        modal_section("⌨", "KEY COMBINATION", neon::SECONDARY),
        Space::new().height(6),
        container(
            column![
                key_capture_display(binding, idx, is_capturing, capture_conflict),
                Space::new().height(10),
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
        Space::new().height(16),
        // Action
        modal_section("⚡", "ACTION", neon::PRIMARY),
        Space::new().height(6),
        container(action_editor(binding, idx, niri_version))
            .padding(12)
            .style(crate::theme::card_style),
        Space::new().height(16),
        // Advanced options
        modal_section("⬡", "ADVANCED OPTIONS", neon::OUTLINE),
        Space::new().height(6),
        container(advanced_options(binding, idx))
            .padding(12)
            .style(crate::theme::card_style),
        Space::new().height(20),
        // Footer
        container(
            row![
                Space::new().width(Length::Fill),
                button(text("Done").size(13).font(fonts::UI_FONT_MEDIUM))
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
        .padding([8, 0]),
    ];

    let modal_content = scrollable(editor.spacing(0).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(760.0))
        .max_height(720.0)
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
