//! Layer rules settings view - list-detail implementation
//!
//! Provides an interface for creating and editing layer rules
//! that control behavior of layer-shell surfaces (panels, docks, notifications).

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{BlockOutFrom, LayerRule, LayerRulesSettings};
use crate::messages::{LayerRulesMessage, Message};

/// Creates the layer rules settings view with list-detail pattern
pub fn view<'a>(
    settings: &'a LayerRulesSettings,
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

    // Horizontal split layout (responsive 1:2 ratio)
    row![
        container(list_panel)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.1, 0.1, 0.1, 0.5
                    ))),
                    ..Default::default()
                }
            }),
        container(detail_panel)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .padding(20),
    ]
    .spacing(0)
    .into()
}

/// List panel showing all layer rules
fn rule_list(settings: &LayerRulesSettings, selected_id: Option<u32>) -> Element<'static, Message> {
    let mut list = column![row![
        text("Layer Rules").size(18),
        button(text("+").size(18))
            .on_press(Message::LayerRules(LayerRulesMessage::AddRule))
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
    .align_y(Alignment::Center),]
    .spacing(0);

    if settings.rules.is_empty() {
        list = list.push(
            container(
                text("No layer rules defined\nClick + to add one")
                    .size(13)
                    .color([0.75, 0.75, 0.75])
                    .center(),
            )
            .padding(20)
            .center(Length::Fill),
        );
    } else {
        for rule in &settings.rules {
            let rule_id = rule.id;
            let is_selected = selected_id == Some(rule_id);
            let rule_name = rule.name.clone();

            // Create a summary of the rule's match criteria
            let match_summary = if !rule.matches.is_empty() {
                let first_match = &rule.matches[0];
                if let Some(ref namespace) = first_match.namespace {
                    format!("ns: {}", truncate_str(namespace, 20))
                } else {
                    "any layer".to_string()
                }
            } else {
                "no match".to_string()
            };

            // Badge for block-out setting
            let block_badge = rule.block_out_from.map(|b| match b {
                BlockOutFrom::Screencast => "hide:cast",
                BlockOutFrom::ScreenCapture => "hide:all",
            });

            list = list.push(
                button(
                    column![
                        row![
                            text(if is_selected { "●" } else { "○" })
                                .size(12)
                                .width(Length::Fixed(20.0))
                                .color(if is_selected {
                                    [0.5, 0.7, 1.0]
                                } else {
                                    [0.5, 0.5, 0.5]
                                }),
                            text(rule_name)
                                .size(14)
                                .color(if is_selected {
                                    [1.0, 1.0, 1.0]
                                } else {
                                    [0.9, 0.9, 0.9]
                                }),
                            if let Some(badge) = block_badge {
                                container(text(badge).size(10).color([0.9, 0.9, 0.9]))
                                    .padding([2, 6])
                                    .style(|_theme| container::Style {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgba(0.6, 0.3, 0.3, 0.4),
                                        )),
                                        border: iced::Border {
                                            radius: 3.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    })
                            } else {
                                container(text(""))
                            },
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(match_summary)
                            .size(12)
                            .color([0.5, 0.55, 0.6]),
                    ]
                    .spacing(4),
                )
                .on_press(Message::LayerRules(LayerRulesMessage::SelectRule(rule_id)))
                .padding([10, 20])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    let bg = if is_selected {
                        iced::Color::from_rgba(0.25, 0.35, 0.45, 0.6)
                    } else {
                        match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.2, 0.25, 0.3, 0.4),
                            _ => iced::Color::TRANSPARENT,
                        }
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: if is_selected {
                                iced::Color::from_rgba(0.4, 0.5, 0.6, 0.5)
                            } else {
                                iced::Color::TRANSPARENT
                            },
                            width: if is_selected { 1.0 } else { 0.0 },
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                }),
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty state when no rule is selected
fn empty_detail_view() -> Element<'static, Message> {
    container(
        column![
            text("Select a Layer Rule").size(18).color([0.6, 0.6, 0.6]),
            spacer(8.0),
            text("Layer rules control behavior of panels, docks, and notifications")
                .size(13)
                .color([0.5, 0.5, 0.5]),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

/// Detail view for a selected layer rule
fn rule_detail_view<'a>(
    rule: &'a LayerRule,
    sections_expanded: &'a HashMap<(u32, String), bool>,
    regex_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    let id = rule.id;

    // Check expanded state for each section
    let matching_expanded = sections_expanded
        .get(&(id, "matching".to_string()))
        .copied()
        .unwrap_or(true);
    let visibility_expanded = sections_expanded
        .get(&(id, "visibility".to_string()))
        .copied()
        .unwrap_or(true);
    let styling_expanded = sections_expanded
        .get(&(id, "styling".to_string()))
        .copied()
        .unwrap_or(false);
    let advanced_expanded = sections_expanded
        .get(&(id, "advanced".to_string()))
        .copied()
        .unwrap_or(false);

    let mut content = column![
        // Header with rule name and actions
        row![
            column![
                text("Rule Name").size(14).color([0.7, 0.7, 0.7]),
                text_input("Rule name", &rule.name)
                    .on_input(move |name| Message::LayerRules(LayerRulesMessage::SetRuleName(
                        id, name
                    )))
                    .padding(10)
                    .size(16),
            ]
            .spacing(4)
            .width(Length::Fill),
            row![
                button(text("▲").size(13))
                    .on_press(Message::LayerRules(LayerRulesMessage::ReorderRule(id, true)))
                    .padding([8, 12])
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.5),
                            _ => iced::Color::from_rgba(0.25, 0.3, 0.35, 0.4),
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
                button(text("▼").size(13))
                    .on_press(Message::LayerRules(LayerRulesMessage::ReorderRule(
                        id, false
                    )))
                    .padding([8, 12])
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.5),
                            _ => iced::Color::from_rgba(0.25, 0.3, 0.35, 0.4),
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
                button(text("Duplicate").size(13))
                    .on_press(Message::LayerRules(LayerRulesMessage::DuplicateRule(id)))
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
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                button(text("Delete").size(13))
                    .on_press(Message::LayerRules(LayerRulesMessage::DeleteRule(id)))
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
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
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
        Message::LayerRules(LayerRulesMessage::ToggleSection(id, "matching".to_string())),
        {
            let mut match_content = column![info_text(
                "Rules apply to layer surfaces that match ANY of the criteria below"
            ),]
            .spacing(8);

            for (match_idx, rule_match) in rule.matches.iter().enumerate() {
                let namespace_value = rule_match.namespace.clone().unwrap_or_default();

                match_content = match_content.push(
                    container(
                        column![
                            row![
                                text(format!("Match {}", match_idx + 1))
                                    .size(13)
                                    .color([0.8, 0.8, 0.8]),
                                if rule.matches.len() > 1 {
                                    button(text("×").size(14))
                                        .on_press(Message::LayerRules(
                                            LayerRulesMessage::RemoveMatch(id, match_idx)
                                        ))
                                        .padding([2, 8])
                                        .style(|_theme, status| {
                                            let bg = match status {
                                                button::Status::Hovered => {
                                                    iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5)
                                                }
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
                            {
                                let namespace_error_key = (id, format!("namespace_{}", match_idx));
                                let namespace_error = regex_errors.get(&namespace_error_key);
                                let mut namespace_col = column![
                                    text("Namespace (regex)").size(14),
                                    text_input("e.g., waybar", &namespace_value)
                                        .on_input(move |value| Message::LayerRules(
                                            LayerRulesMessage::SetMatchNamespace(
                                                id, match_idx, value
                                            )
                                        ))
                                        .padding(8),
                                ]
                                .spacing(4);
                                if let Some(error) = namespace_error {
                                    namespace_col = namespace_col
                                        .push(text(error).size(12).color([0.9, 0.4, 0.4]));
                                }
                                namespace_col
                            },
                            optional_bool_picker(
                                "At startup only",
                                "Match only during first 60 seconds after niri launch",
                                rule_match.at_startup,
                                move |value| {
                                    Message::LayerRules(LayerRulesMessage::SetMatchAtStartup(
                                        id, match_idx, value,
                                    ))
                                },
                            ),
                        ]
                        .spacing(8),
                    )
                    .padding(12)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.15, 0.15, 0.15, 0.4,
                        ))),
                        border: iced::Border {
                            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    }),
                );
            }

            match_content = match_content.push(
                button(row![text("+").size(14), text("Add Match Criteria").size(13),].spacing(6))
                    .on_press(Message::LayerRules(LayerRulesMessage::AddMatch(id)))
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
                    }),
            );

            match_content
        },
    ));

    // Visibility Section
    content = content.push(expandable_section(
        "Visibility",
        visibility_expanded,
        Message::LayerRules(LayerRulesMessage::ToggleSection(
            id,
            "visibility".to_string(),
        )),
        column![
            info_text("Control whether this layer surface is hidden from screen captures"),
            block_out_picker(
                "Block out from",
                "Hide this surface from recordings/captures",
                rule.block_out_from,
                move |value| Message::LayerRules(LayerRulesMessage::SetBlockOutFrom(id, value)),
            ),
        ]
        .spacing(8),
    ));

    // Styling Section
    content = content.push(expandable_section(
        "Styling",
        styling_expanded,
        Message::LayerRules(LayerRulesMessage::ToggleSection(id, "styling".to_string())),
        column![
            optional_slider_row(
                "Opacity",
                "Layer surface opacity (0.0 = transparent, 1.0 = opaque)",
                rule.opacity,
                0.0,
                1.0,
                "",
                move |value| Message::LayerRules(LayerRulesMessage::SetOpacity(id, value)),
            ),
            optional_int_slider_row(
                "Corner radius",
                "Rounded corners in pixels (v25.02+)",
                rule.geometry_corner_radius,
                0,
                32,
                " px",
                move |value| Message::LayerRules(LayerRulesMessage::SetCornerRadius(id, value)),
            ),
        ]
        .spacing(8),
    ));

    // Advanced Section
    content = content.push(expandable_section(
        "Advanced",
        advanced_expanded,
        Message::LayerRules(LayerRulesMessage::ToggleSection(id, "advanced".to_string())),
        column![
            toggle_row(
                "Place within backdrop",
                "Place this layer within the desktop backdrop (v25.05+)",
                rule.place_within_backdrop,
                move |value| {
                    Message::LayerRules(LayerRulesMessage::SetPlaceWithinBackdrop(id, value))
                },
            ),
            toggle_row(
                "Treat as floating",
                "Use floating window animations for this layer (v25.05+)",
                rule.baba_is_float,
                move |value| Message::LayerRules(LayerRulesMessage::SetBabaIsFloat(id, value)),
            ),
        ]
        .spacing(8),
    ));

    scrollable(content.spacing(12))
        .height(Length::Fill)
        .into()
}

/// Helper to truncate long strings
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

/// Picker for BlockOutFrom enum (None, Screencast, ScreenCapture)
fn block_out_picker<'a>(
    label: &'static str,
    description: &'static str,
    value: Option<BlockOutFrom>,
    on_change: impl Fn(Option<BlockOutFrom>) -> Message + 'a,
) -> Element<'a, Message> {
    let options = vec!["None", "Screencast only", "All screen captures"];
    let selected_idx = match value {
        None => 0,
        Some(BlockOutFrom::Screencast) => 1,
        Some(BlockOutFrom::ScreenCapture) => 2,
    };

    column![
        text(label).size(14),
        text(description).size(12).color([0.7, 0.7, 0.7]),
        iced::widget::pick_list(options.clone(), Some(options[selected_idx]), move |selected| {
            let value = match selected {
                "Screencast only" => Some(BlockOutFrom::Screencast),
                "All screen captures" => Some(BlockOutFrom::ScreenCapture),
                _ => None,
            };
            on_change(value)
        })
        .padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Optional slider that can be None or Some(f32)
fn optional_slider_row<'a>(
    label: &'static str,
    description: &'static str,
    value: Option<f32>,
    min: f32,
    max: f32,
    suffix: &'static str,
    on_change: impl Fn(Option<f32>) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_enabled = value.is_some();
    let current_value = value.unwrap_or((min + max) / 2.0);

    column![
        row![
            column![
                text(label).size(14),
                text(description).size(12).color([0.7, 0.7, 0.7]),
            ]
            .width(Length::Fill),
            iced::widget::toggler(is_enabled)
                .on_toggle({
                    let on_change = on_change.clone();
                    move |enabled| {
                        if enabled {
                            on_change(Some((min + max) / 2.0))
                        } else {
                            on_change(None)
                        }
                    }
                })
                .size(20),
        ]
        .align_y(Alignment::Center),
        if is_enabled {
            row![
                iced::widget::slider(min..=max, current_value, {
                    let on_change = on_change.clone();
                    move |v| on_change(Some(v))
                })
                .step(0.01)
                .width(Length::Fill),
                text(format!("{:.2}{}", current_value, suffix))
                    .size(13)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        } else {
            row![text("Disabled").size(12).color([0.5, 0.5, 0.5])]
        },
    ]
    .spacing(8)
    .padding(12)
    .into()
}

/// Optional int slider that can be None or Some(i32)
fn optional_int_slider_row<'a>(
    label: &'static str,
    description: &'static str,
    value: Option<i32>,
    min: i32,
    max: i32,
    suffix: &'static str,
    on_change: impl Fn(Option<i32>) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_enabled = value.is_some();
    let current_value = value.unwrap_or((min + max) / 2);

    column![
        row![
            column![
                text(label).size(14),
                text(description).size(12).color([0.7, 0.7, 0.7]),
            ]
            .width(Length::Fill),
            iced::widget::toggler(is_enabled)
                .on_toggle({
                    let on_change = on_change.clone();
                    move |enabled| {
                        if enabled {
                            on_change(Some((min + max) / 2))
                        } else {
                            on_change(None)
                        }
                    }
                })
                .size(20),
        ]
        .align_y(Alignment::Center),
        if is_enabled {
            row![
                iced::widget::slider(min..=max, current_value, {
                    let on_change = on_change.clone();
                    move |v| on_change(Some(v))
                })
                .width(Length::Fill),
                text(format!("{}{}", current_value, suffix))
                    .size(13)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        } else {
            row![text("Disabled").size(12).color([0.5, 0.5, 0.5])]
        },
    ]
    .spacing(8)
    .padding(12)
    .into()
}
