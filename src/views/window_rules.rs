//! Window rules settings view - list-detail implementation
//!
//! Provides a comprehensive interface for creating and editing window rules
//! that control per-application behavior in niri.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{OpenBehavior, WindowRule, WindowRulesSettings};
use crate::messages::{Message, WindowRulesMessage};

/// Creates the window rules settings view with list-detail pattern
pub fn view<'a>(
    settings: &'a WindowRulesSettings,
    selected_rule_id: Option<u32>,
    sections_expanded: &'a HashMap<(u32, String), bool>,
    regex_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    // Left panel: List of rules
    let list_panel = rule_list(settings, selected_rule_id);

    // Right panel: Detail view for selected rule
    let detail_panel = if let Some(id) = selected_rule_id {
        if let Some(rule) = settings.rules.iter().find(|r| r.id == id) {
            rule_detail_view(rule, sections_expanded, regex_errors)
        } else {
            empty_detail_view()
        }
    } else {
        empty_detail_view()
    };

    // Horizontal split layout
    row![
        container(list_panel)
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
                    ..Default::default()
                }
            }),
        container(detail_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20),
    ]
    .spacing(0)
    .into()
}

/// List panel showing all window rules
fn rule_list(settings: &WindowRulesSettings, selected_id: Option<u32>) -> Element<'static, Message> {
    let mut list = column![
        row![
            text("Window Rules").size(18),
            button(text("+").size(18))
                .on_press(Message::WindowRules(WindowRulesMessage::AddRule))
                .padding([4, 12])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
                        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
                        _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        ]
        .spacing(10)
        .padding([12, 20])
        .align_y(Alignment::Center),
    ]
    .spacing(0);

    if settings.rules.is_empty() {
        list = list.push(
            container(
                text("No window rules defined\nClick + to add one")
                    .size(13)
                    .color([0.6, 0.6, 0.6])
                    .center()
            )
            .padding(20)
            .center(Length::Fill)
        );
    } else {
        for rule in &settings.rules {
            let rule_id = rule.id;
            let is_selected = selected_id == Some(rule_id);
            let rule_name = rule.name.clone();

            // Create a summary of the rule's match criteria
            let match_summary = if !rule.matches.is_empty() {
                let first_match = &rule.matches[0];
                if let Some(ref app_id) = first_match.app_id {
                    format!("app: {}", truncate_str(app_id, 20))
                } else if let Some(ref title) = first_match.title {
                    format!("title: {}", truncate_str(title, 18))
                } else {
                    "any window".to_string()
                }
            } else {
                "no match".to_string()
            };

            let behavior_badge = match rule.open_behavior {
                OpenBehavior::Normal => None,
                OpenBehavior::Maximized => Some("max"),
                OpenBehavior::Fullscreen => Some("full"),
                OpenBehavior::Floating => Some("float"),
            };

            list = list.push(
                button(
                    column![
                        row![
                            text(if is_selected { "●" } else { "○" })
                                .size(12)
                                .width(Length::Fixed(20.0))
                                .color(if is_selected { [0.5, 0.7, 1.0] } else { [0.5, 0.5, 0.5] }),
                            text(rule_name)
                                .size(14)
                                .color(if is_selected { [1.0, 1.0, 1.0] } else { [0.9, 0.9, 0.9] }),
                            if let Some(badge) = behavior_badge {
                                container(
                                    text(badge)
                                        .size(10)
                                        .color([0.9, 0.9, 0.9])
                                )
                                .padding([2, 6])
                                .style(|_theme| {
                                    container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.4, 0.3, 0.6, 0.4)
                                        )),
                                        border: iced::Border {
                                            radius: 3.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                })
                            } else {
                                container(text(""))
                            },
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(match_summary)
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                    ]
                    .spacing(2)
                )
                .on_press(Message::WindowRules(WindowRulesMessage::SelectRule(rule_id)))
                .padding([10, 12])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    let background = match (is_selected, status) {
                        (true, button::Status::Hovered) => iced::Color::from_rgba(0.3, 0.4, 0.6, 0.5),
                        (true, button::Status::Pressed) => iced::Color::from_rgba(0.4, 0.5, 0.7, 0.5),
                        (true, _) => iced::Color::from_rgba(0.2, 0.3, 0.5, 0.4),
                        (false, button::Status::Hovered) => iced::Color::from_rgba(0.25, 0.25, 0.25, 0.5),
                        (false, button::Status::Pressed) => iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                        (false, _) => iced::Color::TRANSPARENT,
                    };

                    button::Style {
                        background: Some(iced::Background::Color(background)),
                        border: iced::Border::default(),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    }
                })
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty detail view shown when no rule is selected
fn empty_detail_view() -> Element<'static, Message> {
    container(
        column![
            text("Select a window rule to edit")
                .size(16)
                .color([0.6, 0.6, 0.6]),
            spacer(8.0),
            text("Window rules let you configure per-application behavior")
                .size(13)
                .color([0.5, 0.5, 0.5]),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
    )
    .center(Length::Fill)
    .into()
}

/// Detail view for a selected window rule
fn rule_detail_view<'a>(
    rule: &'a WindowRule,
    sections_expanded: &'a HashMap<(u32, String), bool>,
    _regex_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    let id = rule.id;

    // Check expanded state for each section
    let matching_expanded = sections_expanded.get(&(id, "matching".to_string())).copied().unwrap_or(true);
    let behavior_expanded = sections_expanded.get(&(id, "behavior".to_string())).copied().unwrap_or(true);
    let sizing_expanded = sections_expanded.get(&(id, "sizing".to_string())).copied().unwrap_or(false);
    let styling_expanded = sections_expanded.get(&(id, "styling".to_string())).copied().unwrap_or(false);
    let advanced_expanded = sections_expanded.get(&(id, "advanced".to_string())).copied().unwrap_or(false);

    let mut content = column![
        // Header with rule name and actions
        row![
            column![
                text("Rule Name").size(14).color([0.7, 0.7, 0.7]),
                text_input("Rule name", &rule.name)
                    .on_input(move |name| Message::WindowRules(WindowRulesMessage::SetRuleName(id, name)))
                    .padding(10)
                    .size(16),
            ]
            .spacing(4)
            .width(Length::Fill),
            row![
                button(text("Duplicate").size(13))
                    .on_press(Message::WindowRules(WindowRulesMessage::DuplicateRule(id)))
                    .padding([8, 12])
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.5),
                            button::Status::Pressed => iced::Color::from_rgba(0.4, 0.5, 0.6, 0.5),
                            _ => iced::Color::from_rgba(0.25, 0.3, 0.35, 0.4),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: iced::Color::WHITE,
                            border: iced::Border { radius: 4.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
                button(text("Delete").size(13))
                    .on_press(Message::WindowRules(WindowRulesMessage::DeleteRule(id)))
                    .padding([8, 12])
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.7, 0.2, 0.2, 0.6),
                            button::Status::Pressed => iced::Color::from_rgba(0.8, 0.3, 0.3, 0.7),
                            _ => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: iced::Color::WHITE,
                            border: iced::Border { radius: 4.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(8),
        ]
        .spacing(16)
        .align_y(Alignment::End),
        spacer(16.0),
    ];

    // Matching Section
    content = content.push(expandable_section(
        "Match Criteria",
        matching_expanded,
        Message::WindowRules(WindowRulesMessage::ToggleSection(id, "matching".to_string())),
        {
            let mut match_content = column![
                info_text("Rules apply to windows that match ANY of the criteria below"),
            ]
            .spacing(8);

            for (match_idx, rule_match) in rule.matches.iter().enumerate() {
                let app_id_value = rule_match.app_id.clone().unwrap_or_default();
                let title_value = rule_match.title.clone().unwrap_or_default();

                match_content = match_content.push(
                    container(
                        column![
                            row![
                                text(format!("Match {}", match_idx + 1)).size(13).color([0.8, 0.8, 0.8]),
                                if rule.matches.len() > 1 {
                                    button(text("×").size(14))
                                        .on_press(Message::WindowRules(WindowRulesMessage::RemoveMatch(id, match_idx)))
                                        .padding([2, 8])
                                        .style(|_theme, status| {
                                            let bg = match status {
                                                button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
                                                _ => iced::Color::TRANSPARENT,
                                            };
                                            button::Style {
                                                background: Some(iced::Background::Color(bg)),
                                                text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
                                                ..Default::default()
                                            }
                                        })
                                } else {
                                    button(text("")).width(Length::Shrink)
                                },
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                            column![
                                text("App ID (regex)").size(14),
                                text_input("e.g., firefox", &app_id_value)
                                    .on_input(move |value| Message::WindowRules(WindowRulesMessage::SetMatchAppId(id, match_idx, if value.is_empty() { None } else { Some(value) })))
                                    .padding(8),
                            ].spacing(4),
                            column![
                                text("Title (regex)").size(14),
                                text_input("e.g., .*YouTube.*", &title_value)
                                    .on_input(move |value| Message::WindowRules(WindowRulesMessage::SetMatchTitle(id, match_idx, if value.is_empty() { None } else { Some(value) })))
                                    .padding(8),
                            ].spacing(4),
                            optional_bool_picker(
                                "Is floating",
                                "Match only floating/tiled windows",
                                rule_match.is_floating,
                                move |value| Message::WindowRules(WindowRulesMessage::SetMatchIsFloating(id, match_idx, value)),
                            ),
                        ]
                        .spacing(8)
                    )
                    .padding(12)
                    .style(|_theme| {
                        container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.15, 0.4))),
                            border: iced::Border {
                                color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                );
            }

            match_content = match_content.push(
                button(
                    row![
                        text("+").size(14),
                        text("Add Match Criteria").size(13),
                    ]
                    .spacing(6)
                )
                .on_press(Message::WindowRules(WindowRulesMessage::AddMatch(id)))
                .padding([8, 16])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.4),
                        _ => iced::Color::from_rgba(0.2, 0.25, 0.3, 0.3),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: iced::Color::from_rgb(0.7, 0.8, 0.9),
                        border: iced::Border {
                            color: iced::Color::from_rgba(0.4, 0.5, 0.6, 0.3),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
            );

            match_content
        },
    ));

    // Behavior Section
    content = content.push(expandable_section(
        "Opening Behavior",
        behavior_expanded,
        Message::WindowRules(WindowRulesMessage::ToggleSection(id, "behavior".to_string())),
        column![
            picker_row(
                "Open as",
                "How the window should open",
                OpenBehavior::all(),
                Some(rule.open_behavior),
                move |value| Message::WindowRules(WindowRulesMessage::SetOpenBehavior(id, value)),
            ),
            optional_bool_picker(
                "Open focused",
                "Whether to focus the window when it opens",
                rule.open_focused,
                move |value| Message::WindowRules(WindowRulesMessage::SetOpenFocused(id, value)),
            ),
            text_input_row(
                "Open on output",
                "Output name (e.g., HDMI-1)",
                rule.open_on_output.as_deref().unwrap_or(""),
                move |value| Message::WindowRules(WindowRulesMessage::SetOpenOnOutput(id, if value.is_empty() { None } else { Some(value) })),
            ),
            text_input_row(
                "Open on workspace",
                "Workspace name",
                rule.open_on_workspace.as_deref().unwrap_or(""),
                move |value| Message::WindowRules(WindowRulesMessage::SetOpenOnWorkspace(id, if value.is_empty() { None } else { Some(value) })),
            ),
            toggle_row(
                "Block from screencast",
                "Hide this window in screen recordings",
                rule.block_out_from_screencast,
                move |value| Message::WindowRules(WindowRulesMessage::SetBlockScreencast(id, value)),
            ),
        ]
        .spacing(8),
    ));

    // Sizing Section
    content = content.push(expandable_section(
        "Size & Position",
        sizing_expanded,
        Message::WindowRules(WindowRulesMessage::ToggleSection(id, "sizing".to_string())),
        column![
            info_text("Configure default window dimensions and constraints"),
            optional_slider_row(
                "Default column width",
                "Proportion of screen width (0.0-1.0)",
                rule.default_column_width,
                0.1,
                1.0,
                "",
                move |value| Message::WindowRules(WindowRulesMessage::SetDefaultColumnWidth(id, value)),
            ),
            optional_slider_row(
                "Default window height",
                "Proportion of screen height (0.0-1.0)",
                rule.default_window_height,
                0.1,
                1.0,
                "",
                move |value| Message::WindowRules(WindowRulesMessage::SetDefaultWindowHeight(id, value)),
            ),
        ]
        .spacing(8),
    ));

    // Styling Section
    content = content.push(expandable_section(
        "Visual Styling",
        styling_expanded,
        Message::WindowRules(WindowRulesMessage::ToggleSection(id, "styling".to_string())),
        column![
            info_text("Override global appearance settings for this window"),
            optional_slider_row(
                "Opacity",
                "Window transparency (0.0-1.0)",
                rule.opacity,
                0.0,
                1.0,
                "",
                move |value| Message::WindowRules(WindowRulesMessage::SetOpacity(id, value)),
            ),
            optional_slider_row_int(
                "Corner radius",
                "Window corner radius in pixels",
                rule.corner_radius,
                0,
                32,
                "px",
                move |value| Message::WindowRules(WindowRulesMessage::SetCornerRadius(id, value)),
            ),
            optional_bool_picker(
                "Clip to geometry",
                "Clip window rendering to visual geometry",
                rule.clip_to_geometry,
                move |value| Message::WindowRules(WindowRulesMessage::SetClipToGeometry(id, value)),
            ),
        ]
        .spacing(8),
    ));

    // Advanced Section
    content = content.push(expandable_section(
        "Advanced",
        advanced_expanded,
        Message::WindowRules(WindowRulesMessage::ToggleSection(id, "advanced".to_string())),
        column![
            optional_bool_picker(
                "Variable refresh rate",
                "Enable VRR/FreeSync for this window",
                rule.variable_refresh_rate,
                move |value| Message::WindowRules(WindowRulesMessage::SetVariableRefreshRate(id, value)),
            ),
            optional_bool_picker(
                "Floating animation",
                "Apply floating animation effect (baba-is-float)",
                rule.baba_is_float,
                move |value| Message::WindowRules(WindowRulesMessage::SetBabaIsFloat(id, value)),
            ),
            info_text("More settings available in future updates"),
        ]
        .spacing(8),
    ));

    scrollable(content.spacing(12)).height(Length::Fill).into()
}

/// Helper to truncate strings for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

/// Optional slider for f32 values (None = disabled)
fn optional_slider_row<'a, M: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: Option<f32>,
    min: f32,
    max: f32,
    unit: &'a str,
    on_change: impl Fn(Option<f32>) -> M + 'a + Copy,
) -> Element<'a, M> {
    let is_enabled = value.is_some();
    let current_value = value.unwrap_or((min + max) / 2.0);

    column![
        row![
            column![
                text(label).size(14),
                text(description).size(12).color([0.6, 0.6, 0.6]),
            ]
            .width(Length::Fill),
            iced::widget::toggler(is_enabled)
                .on_toggle(move |enabled| {
                    if enabled {
                        on_change(Some((min + max) / 2.0))
                    } else {
                        on_change(None)
                    }
                }),
        ]
        .align_y(Alignment::Center),
        if is_enabled {
            row![
                iced::widget::slider(min..=max, current_value, move |v| on_change(Some(v)))
                    .width(Length::Fill),
                text(format!("{:.2}{}", current_value, unit))
                    .size(13)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        } else {
            row![text("").size(1)]
        },
    ]
    .spacing(8)
    .into()
}

/// Optional slider for i32 values (None = disabled)
fn optional_slider_row_int<'a, M: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: Option<i32>,
    min: i32,
    max: i32,
    unit: &'a str,
    on_change: impl Fn(Option<i32>) -> M + 'a + Copy,
) -> Element<'a, M> {
    let is_enabled = value.is_some();
    let current_value = value.unwrap_or((min + max) / 2);

    column![
        row![
            column![
                text(label).size(14),
                text(description).size(12).color([0.6, 0.6, 0.6]),
            ]
            .width(Length::Fill),
            iced::widget::toggler(is_enabled)
                .on_toggle(move |enabled| {
                    if enabled {
                        on_change(Some((min + max) / 2))
                    } else {
                        on_change(None)
                    }
                }),
        ]
        .align_y(Alignment::Center),
        if is_enabled {
            row![
                iced::widget::slider(min..=max, current_value, move |v| on_change(Some(v)))
                    .width(Length::Fill),
                text(format!("{}{}", current_value, unit))
                    .size(13)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        } else {
            row![text("").size(1)]
        },
    ]
    .spacing(8)
    .into()
}

// Implement Display for OpenBehavior to use with picker_row
impl std::fmt::Display for OpenBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenBehavior::Normal => write!(f, "Normal"),
            OpenBehavior::Maximized => write!(f, "Maximized"),
            OpenBehavior::Fullscreen => write!(f, "Fullscreen"),
            OpenBehavior::Floating => write!(f, "Floating"),
        }
    }
}

impl OpenBehavior {
    pub fn all() -> &'static [OpenBehavior] {
        &[
            OpenBehavior::Normal,
            OpenBehavior::Maximized,
            OpenBehavior::Fullscreen,
            OpenBehavior::Floating,
        ]
    }
}
