//! Config Editor view — neon modal style
//!
//! Viewer and editor for generated KDL config files.
//! Read-only by default, with optional edit mode for advanced users.

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_editor, Space,
};
use iced::{Alignment, Element, Length};

use super::widgets::toggle_row;
use crate::messages::{ConfigEditorMessage, Message};
use crate::theme::{fonts, neon};

/// List of config files that can be viewed (relative paths from managed_dir)
pub const CONFIG_FILES: &[&str] = &[
    "main.kdl",
    "appearance.kdl",
    "behavior.kdl",
    "animations.kdl",
    "cursor.kdl",
    "overview.kdl",
    "outputs.kdl",
    "workspaces.kdl",
    "keybindings.kdl",
    // Input subdirectory
    "input/keyboard.kdl",
    "input/mouse.kdl",
    "input/touchpad.kdl",
    "input/trackpoint.kdl",
    "input/trackball.kdl",
    "input/tablet.kdl",
    "input/touch.kdl",
    // Advanced subdirectory
    "advanced/layout-extras.kdl",
    "advanced/gestures.kdl",
    "advanced/misc.kdl",
    "advanced/startup.kdl",
    "advanced/environment.kdl",
    "advanced/debug.kdl",
    "advanced/switch-events.kdl",
    "advanced/window-rules.kdl",
    "advanced/layer-rules.kdl",
    "advanced/recent-windows.kdl",
];

/// State for the config editor page
#[derive(Debug, Clone, Default)]
pub struct ConfigEditorState {
    /// Currently selected file index
    pub selected_file: Option<usize>,
    /// Content of the selected file (or error message)
    pub file_content: Option<Result<String, String>>,
    /// Whether content is loading
    pub loading: bool,
    /// Whether edit mode is enabled
    pub edit_mode: bool,
    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
}

/// Creates the config editor view
pub fn view<'a>(
    state: &'a ConfigEditorState,
    editor_content: &'a text_editor::Content,
) -> Element<'a, Message> {
    let content = column![
        // ── 2-COLUMN: CONTROLS | EDITOR ──
        row![
            // Left: File selector, mode toggle, action buttons
            column![
                modal_section("\u{1F4C4}", "FILE SELECTOR", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        text("CONFIG FILE")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        {
                            let file_names: Vec<&str> = CONFIG_FILES.to_vec();
                            let selected_name = state.selected_file.map(|i| CONFIG_FILES[i]);
                            pick_list(file_names, selected_name, |name| {
                                let idx = CONFIG_FILES.iter().position(|&f| f == name).unwrap_or(0);
                                Message::ConfigEditor(ConfigEditorMessage::SelectFile(idx))
                            })
                            .placeholder("Select a file...")
                            .width(Length::Fill)
                        },
                    ]
                    .spacing(4),
                )
                .padding(12)
                .style(crate::theme::card_style),
                Space::new().height(8),
                container(
                    column![
                        styled_button(
                            if state.loading { "Loading..." } else { "Refresh" },
                            state.selected_file.is_some() && !state.loading,
                            Message::ConfigEditor(ConfigEditorMessage::Refresh),
                            neon::SECONDARY,
                        ),
                    ]
                    .spacing(6),
                )
                .padding(12)
                .style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("\u{270F}", "EDIT MODE", neon::PRIMARY),
                Space::new().height(4),
                container(
                    toggle_row(
                        "Enable Editing",
                        "Allow direct editing of KDL config files",
                        state.edit_mode,
                        |v| Message::ConfigEditor(ConfigEditorMessage::ToggleEditMode(v)),
                    ),
                )
                .padding(8)
                .style(crate::theme::card_style),
                // Warning / info banner
                Space::new().height(8),
                if state.edit_mode {
                    container(
                        column![
                            text("EDIT MODE ACTIVE")
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::TERTIARY),
                            Space::new().height(2),
                            text("Manual edits will be OVERWRITTEN when you change settings in the app.")
                                .size(11)
                                .color(neon::ON_SURFACE_VARIANT),
                        ]
                        .spacing(2)
                        .padding(4),
                    )
                    .padding(8)
                    .style(crate::theme::card_style)
                } else {
                    container(
                        text("Read-only mode. Enable editing above to make changes.")
                            .size(11)
                            .color(neon::OUTLINE),
                    )
                    .padding([8, 12])
                },
                // Save / Discard buttons when in edit mode with changes
                if state.edit_mode && state.has_unsaved_changes {
                    container(
                        column![
                            text("UNSAVED CHANGES")
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::TERTIARY),
                            Space::new().height(4),
                            styled_button("Save", true,
                                Message::ConfigEditor(ConfigEditorMessage::SaveEdits),
                                neon::SECONDARY),
                            styled_button("Discard", true,
                                Message::ConfigEditor(ConfigEditorMessage::DiscardEdits),
                                neon::TERTIARY),
                        ]
                        .spacing(6),
                    )
                    .padding(12)
                    .style(crate::theme::card_style)
                } else {
                    container(Space::new().height(0))
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),

            // Right: Editor / content display (takes more space)
            column![
                modal_section("\u{2728}", "CONTENTS", neon::TERTIARY),
                Space::new().height(4),
                if let Some(selected_idx) = state.selected_file {
                    let filename = CONFIG_FILES[selected_idx];
                    container(
                        column![
                            text(format!("{}", filename))
                                .size(11)
                                .font(fonts::MONO_FONT)
                                .color(neon::SECONDARY),
                            Space::new().height(6),
                            if state.edit_mode {
                                container(
                                    text_editor(editor_content)
                                        .on_action(|action| {
                                            Message::ConfigEditor(ConfigEditorMessage::EditorAction(action))
                                        })
                                        .font(fonts::MONO_FONT)
                                        .size(12)
                                        .padding(12)
                                        .height(Length::Fixed(550.0)),
                                )
                                .style(|_theme| container::Style {
                                    background: Some(iced::Background::Color(neon::SURFACE_LOW)),
                                    border: iced::Border {
                                        radius: 6.0.into(),
                                        width: 1.0,
                                        color: iced::Color { a: 0.3, ..neon::PRIMARY },
                                    },
                                    ..Default::default()
                                })
                            } else {
                                let content_display = match &state.file_content {
                                    Some(Ok(file_text)) => {
                                        if file_text.is_empty() {
                                            text("(empty file)")
                                                .size(12)
                                                .color(neon::OUTLINE)
                                                .font(fonts::MONO_FONT)
                                        } else {
                                            text(file_text)
                                                .size(12)
                                                .font(fonts::MONO_FONT)
                                                .color(neon::ON_SURFACE)
                                        }
                                    }
                                    Some(Err(error)) => text(format!("Error: {}", error))
                                        .size(12)
                                        .color(neon::ERROR),
                                    None => {
                                        if state.loading {
                                            text("Loading...")
                                                .size(12)
                                                .color(neon::OUTLINE)
                                        } else {
                                            text("Click Refresh to load file contents")
                                                .size(12)
                                                .color(neon::OUTLINE)
                                        }
                                    }
                                };
                                container(
                                    scrollable(
                                        container(content_display)
                                            .padding(12)
                                            .width(Length::Fill),
                                    )
                                    .height(Length::Fixed(550.0)),
                                )
                                .style(|_theme| container::Style {
                                    background: Some(iced::Background::Color(neon::SURFACE_LOW)),
                                    border: iced::Border {
                                        radius: 6.0.into(),
                                        width: 1.0,
                                        color: iced::Color { a: 0.2, ..neon::OUTLINE },
                                    },
                                    ..Default::default()
                                })
                            },
                        ]
                        .spacing(0),
                    )
                    .padding(12)
                    .style(crate::theme::card_style)
                } else {
                    container(
                        column![
                            Space::new().height(40),
                            text("Select a file from the dropdown to view its contents")
                                .size(13)
                                .color(neon::OUTLINE),
                            Space::new().height(40),
                        ]
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                    )
                    .padding(12)
                    .style(crate::theme::card_style)
                    .width(Length::Fill)
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(2)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0)
    .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

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

fn styled_button<'a>(
    label: &'a str,
    enabled: bool,
    message: Message,
    accent: iced::Color,
) -> Element<'a, Message> {
    let mut btn = button(text(label).size(12).font(fonts::UI_FONT_SEMIBOLD))
        .padding([6, 16])
        .width(Length::Fill)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => iced::Color { a: 0.35, ..accent },
                button::Status::Pressed => iced::Color { a: 0.45, ..accent },
                _ => iced::Color { a: 0.2, ..accent },
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: accent,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    if enabled {
        btn = btn.on_press(message);
    }

    btn.into()
}
