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

    // Use shared list-detail layout
    list_detail_layout(list_panel, detail_panel)
}

/// List panel showing all layer rules
fn rule_list(settings: &LayerRulesSettings, selected_id: Option<u32>) -> Element<'static, Message> {
    let mut list = column![
        row![
            text("Layer Rules").size(18),
            add_button(Message::LayerRules(LayerRulesMessage::AddRule)),
        ]
        .spacing(10)
        .padding([12, 20])
        .align_y(Alignment::Center),
    ]
    .spacing(0);

    if settings.rules.is_empty() {
        list = list.push(empty_list_placeholder("No layer rules defined\nClick + to add one"));
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
                            selection_indicator(is_selected),
                            text(rule_name)
                                .size(14)
                                .color(if is_selected {
                                    [1.0, 1.0, 1.0]
                                } else {
                                    [0.9, 0.9, 0.9]
                                }),
                            if let Some(badge_text) = block_badge {
                                badge(badge_text, BADGE_VISIBILITY)
                            } else {
                                container(text("")).into()
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
                .style(list_item_style(is_selected)),
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty state when no rule is selected
fn empty_detail_view() -> Element<'static, Message> {
    empty_detail_placeholder(
        "Select a Layer Rule",
        "Layer rules control behavior of panels, docks, and notifications",
    )
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
                action_button("▲", Message::LayerRules(LayerRulesMessage::ReorderRule(id, true))),
                action_button("▼", Message::LayerRules(LayerRulesMessage::ReorderRule(id, false))),
                action_button("Duplicate", Message::LayerRules(LayerRulesMessage::DuplicateRule(id))),
                delete_button(Message::LayerRules(LayerRulesMessage::DeleteRule(id))),
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
                                    remove_button(Message::LayerRules(LayerRulesMessage::RemoveMatch(id, match_idx)))
                                } else {
                                    button(text("")).width(Length::Shrink).into()
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
                    .style(match_container_style),
                );
            }

            match_content = match_content.push(
                add_item_button("Add Match Criteria", Message::LayerRules(LayerRulesMessage::AddMatch(id)))
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
