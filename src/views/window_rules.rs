//! Window rules settings view - card grid + modal editor
//!
//! Displays window rules as a card grid with matcher/effect pills.
//! Editing is done through a modal overlay.

use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{
    DefaultColumnDisplay, FloatingPosition, OpenBehavior, PositionRelativeTo, WindowRule,
    WindowRulesSettings,
};
use crate::messages::{Message, RulesFilter, WindowRulesMessage};
use crate::theme::{fonts, neon};
use crate::types::{Color as NiriColor, ColorOrGradient};

const RULE_CARD_HEIGHT: f32 = 320.0;
const RULE_CARD_SECTION_HEIGHT: f32 = 68.0;
const MAX_VISIBLE_SUMMARY_PILLS: usize = 2;

// ── Card Grid View ─────────────────────────────────────────────────────────

/// Creates the window rules card grid view
pub fn view<'a>(
    settings: &'a WindowRulesSettings,
    search: &'a str,
    filter: RulesFilter,
    _sections_expanded: &'a HashMap<(u32, String), bool>,
    _regex_errors: &'a HashMap<(u32, String), String>,
    _available_workspaces: &'a [String],
) -> Element<'a, Message> {
    let search_owned = search.to_string();

    // Filter rules
    let filtered_rules: Vec<&WindowRule> = settings
        .rules
        .iter()
        .filter(|rule| match filter {
            RulesFilter::All => true,
            RulesFilter::Active => rule.enabled,
            RulesFilter::Disabled => !rule.enabled,
        })
        .filter(|rule| {
            if search.is_empty() {
                return true;
            }
            let search_lower = search.to_lowercase();
            if rule.name.to_lowercase().contains(&search_lower) {
                return true;
            }
            rule.matches.iter().any(|m| {
                m.app_id
                    .as_ref()
                    .is_some_and(|id| id.to_lowercase().contains(&search_lower))
                    || m.title
                        .as_ref()
                        .is_some_and(|t| t.to_lowercase().contains(&search_lower))
            })
        })
        .collect();

    let active_count = settings.rules.iter().filter(|r| r.enabled).count();

    let mut content = column![
        // ── Search bar + filter tabs ────────────────────────────────────
        row![
            container(
                row![
                    text("⌕").size(16).color(neon::OUTLINE_VARIANT),
                    text_input("Search by App ID or Title...", &search_owned)
                        .on_input(|s| Message::WindowRules(WindowRulesMessage::SetSearch(s)))
                        .padding([8, 4])
                        .size(14)
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding([8, 16])
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
                border: iced::Border {
                    radius: 12.0.into(),
                    color: iced::Color {
                        a: 0.15,
                        ..neon::OUTLINE_VARIANT
                    },
                    width: 1.0,
                },
                ..Default::default()
            }),
            Space::new().width(12),
            filter_tabs(filter),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
    ]
    .spacing(0);

    if filtered_rules.is_empty() {
        content = content.push(empty_state(search, filter));
    } else {
        // Distribute cards across 3 columns (round-robin)
        let mut col1_items: Vec<Element<'a, Message>> = Vec::new();
        let mut col2_items: Vec<Element<'a, Message>> = Vec::new();
        let mut col3_items: Vec<Element<'a, Message>> = Vec::new();

        for (i, rule) in filtered_rules.iter().enumerate() {
            let card = rule_card(rule);
            match i % 3 {
                0 => col1_items.push(card),
                1 => col2_items.push(card),
                _ => col3_items.push(card),
            }
        }

        let grid = row![
            column(col1_items).spacing(12).width(Length::FillPortion(1)),
            column(col2_items).spacing(12).width(Length::FillPortion(1)),
            column(col3_items).spacing(12).width(Length::FillPortion(1)),
        ]
        .spacing(12)
        .align_y(Alignment::Start);

        content = content.push(grid);
    }

    // Stats bar
    content = content.push(Space::new().height(16));
    content = content.push(stats_bar(active_count, settings.rules.len()));

    content.into()
}

// ── Rule Card ──────────────────────────────────────────────────────────────

/// A single rule card for the grid
fn rule_card(rule: &WindowRule) -> Element<'_, Message> {
    let id = rule.id;
    let enabled = rule.enabled;

    // Rotate accent color per card for visual variety
    let accent = match id % 3 {
        0 => neon::PRIMARY,
        1 => neon::SECONDARY,
        _ => neon::TERTIARY,
    };

    // Letter avatar from first match app_id
    let avatar_char = rule
        .matches
        .first()
        .and_then(|m| m.app_id.as_ref())
        .and_then(|id| id.chars().next())
        .unwrap_or('R')
        .to_uppercase()
        .next()
        .unwrap_or('R');

    let avatar = container(
        text(avatar_char.to_string())
            .size(18)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(accent),
    )
    .width(44)
    .height(44)
    .center(Length::Shrink)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color { a: 0.15, ..accent })),
        border: iced::Border {
            radius: 12.0.into(),
            color: iced::Color { a: 0.2, ..accent },
            width: 1.0,
        },
        ..Default::default()
    });

    // App ID subtitle (owned string for text widget)
    let app_id_display: String = rule
        .matches
        .first()
        .and_then(|m| m.app_id.as_ref())
        .map(|id| format!("app-id: {}", id))
        .unwrap_or_else(|| "any window".to_string());

    // Matcher pills
    let mut matcher_pills: Vec<(String, iced::Color)> = Vec::new();
    for m in &rule.matches {
        if let Some(ref app_id) = m.app_id {
            matcher_pills.push((format!("ID: {}", truncate_str(app_id, 18)), neon::SECONDARY));
        }
        if let Some(ref title) = m.title {
            matcher_pills.push((
                format!("Title: \"{}\"", truncate_str(title, 15)),
                neon::PRIMARY,
            ));
        }
    }

    // Effect pills with icons
    let mut effect_pills: Vec<(String, iced::Color)> = Vec::new();
    match rule.open_behavior {
        OpenBehavior::Floating => {
            effect_pills.push(("◇ Always Float".to_string(), neon::TERTIARY));
        }
        OpenBehavior::Maximized => {
            effect_pills.push(("⊞ Maximize".to_string(), neon::TERTIARY));
        }
        OpenBehavior::Fullscreen => {
            effect_pills.push(("⊡ Fullscreen".to_string(), neon::TERTIARY));
        }
        OpenBehavior::Normal => {}
    }
    if let Some(ref ws) = rule.open_on_workspace {
        effect_pills.push((format!("▤ WS {}", truncate_str(ws, 12)), neon::SECONDARY));
    }
    if let Some(opacity) = rule.opacity {
        effect_pills.push((format!("◉ Opacity {:.2}", opacity), neon::PRIMARY));
    }
    if rule.block_out_from_screencast {
        effect_pills.push(("⊘ Block Capture".to_string(), neon::ERROR));
    }

    let card_content = column![
        // Header: avatar + name + toggle
        row![
            avatar,
            column![
                text(&rule.name).size(15).font(fonts::UI_FONT_SEMIBOLD),
                text(app_id_display)
                    .size(11)
                    .color(neon::ON_SURFACE_VARIANT),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(enabled)
                .on_toggle(move |v| Message::WindowRules(WindowRulesMessage::SetRuleEnabled(id, v)))
                .width(Length::Shrink),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        Space::new().height(12),
        rule_summary_section("MATCHERS", &matcher_pills, "Matches any window"),
        rule_summary_section("EFFECTS", &effect_pills, "No rule effects"),
        Space::new().height(Length::Fill),
        // Divider line
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.2,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        Space::new().height(8),
        // Footer: Edit + Remove
        row![
            button(row![text("✎").size(12), text("Edit Rule").size(12),].spacing(4))
                .on_press(Message::WindowRules(WindowRulesMessage::OpenEditor(id)))
                .padding([6, 12])
                .style(ghost_button_style),
            Space::new().width(Length::Fill),
            button(
                row![
                    text("🗑").size(12),
                    text("Remove").size(12).color(neon::ERROR),
                ]
                .spacing(4)
            )
            .on_press(Message::WindowRules(WindowRulesMessage::DeleteRule(id)))
            .padding([6, 12])
            .style(ghost_button_style),
        ],
    ]
    .spacing(4)
    .height(Length::Fill);

    container(card_content)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fixed(RULE_CARD_HEIGHT))
        .style(move |_: &iced::Theme| {
            let (border_color, shadow_color) = if enabled {
                (
                    iced::Color { a: 0.2, ..accent },
                    iced::Color { a: 0.12, ..accent },
                )
            } else {
                (
                    iced::Color {
                        a: 0.08,
                        ..neon::OUTLINE_VARIANT
                    },
                    iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                )
            };
            container::Style {
                background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 16.0.into(),
                },
                shadow: iced::Shadow {
                    color: shadow_color,
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        })
        .into()
}

// ── Modal Editor ───────────────────────────────────────────────────────────

/// Creates the modal editor overlay for a window rule
pub fn editor_modal<'a>(
    rule: &'a WindowRule,
    _sections_expanded: &'a HashMap<(u32, String), bool>,
    regex_errors: &'a HashMap<(u32, String), String>,
    available_workspaces: &'a [String],
) -> Element<'a, Message> {
    let id = rule.id;

    let mut editor = column![
        // Header with icon
        row![
            // Icon badge
            container(text("⊞").size(24).color(neon::PRIMARY),)
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
                        width: 1.0,
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("CONFIGURATION EDITOR")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::SECONDARY),
                row![
                    text("Modify Rule: ").size(22).font(fonts::UI_FONT_SEMIBOLD),
                    text(&rule.name)
                        .size(22)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::PRIMARY),
                ],
            ]
            .spacing(4)
            .width(Length::Fill),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::WindowRules(WindowRulesMessage::CloseEditor))
                .padding([8, 12])
                .style(|_: &iced::Theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => iced::Color {
                            a: 0.15,
                            ..neon::ON_SURFACE
                        },
                        _ => iced::Color {
                            a: 0.08,
                            ..neon::ON_SURFACE
                        },
                    };
                    iced::widget::button::Style {
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
        .spacing(0)
        .align_y(Alignment::Center),
        Space::new().height(12),
        // Rule name input
        row![
            text("Rule Name")
                .size(12)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::ON_SURFACE_VARIANT),
            Space::new().width(12),
            text_input("Rule name", &rule.name)
                .on_input(
                    move |name| Message::WindowRules(WindowRulesMessage::SetRuleName(id, name))
                )
                .padding(10)
                .size(14)
                .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
    ]
    .spacing(0);

    // ── APPLICATION MATCHERS ──
    editor = editor.push(modal_section_header(
        "▼",
        "APPLICATION MATCHERS",
        neon::SECONDARY,
    ));
    {
        let mut match_content = column![].spacing(8);
        for (match_idx, rule_match) in rule.matches.iter().enumerate() {
            let app_id_value = rule_match.app_id.clone().unwrap_or_default();
            let title_value = rule_match.title.clone().unwrap_or_default();

            let app_id_error_key = (id, format!("app_id_{}", match_idx));
            let app_id_error = regex_errors.get(&app_id_error_key);
            let title_error_key = (id, format!("title_{}", match_idx));
            let title_error = regex_errors.get(&title_error_key);

            let mut app_col = column![
                text("APP ID (REGEX)")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                text_input("e.g., ^firefox$", &app_id_value)
                    .on_input(
                        move |value| Message::WindowRules(WindowRulesMessage::SetMatchAppId(
                            id,
                            match_idx,
                            if value.is_empty() { None } else { Some(value) }
                        ))
                    )
                    .padding(12),
            ]
            .spacing(6)
            .width(Length::FillPortion(1));
            if let Some(error) = app_id_error {
                app_col = app_col.push(text(error).size(11).color(neon::ERROR));
            }

            let mut title_col = column![
                text("WINDOW TITLE (REGEX)")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                text_input("e.g., .*", &title_value)
                    .on_input(
                        move |value| Message::WindowRules(WindowRulesMessage::SetMatchTitle(
                            id,
                            match_idx,
                            if value.is_empty() { None } else { Some(value) }
                        ))
                    )
                    .padding(12),
            ]
            .spacing(6)
            .width(Length::FillPortion(1));
            if let Some(error) = title_error {
                title_col = title_col.push(text(error).size(11).color(neon::ERROR));
            }

            match_content = match_content.push(row![app_col, title_col].spacing(12));

            // Bool matchers in a compact card
            match_content = match_content.push(
                container(
                    column![
                        row![
                            compact_bool("Floating", rule_match.is_floating, move |v| {
                                Message::WindowRules(WindowRulesMessage::SetMatchIsFloating(
                                    id, match_idx, v,
                                ))
                            }),
                            compact_bool("Focused", rule_match.is_focused, move |v| {
                                Message::WindowRules(WindowRulesMessage::SetMatchIsFocused(
                                    id, match_idx, v,
                                ))
                            }),
                            compact_bool("Active", rule_match.is_active, move |v| {
                                Message::WindowRules(WindowRulesMessage::SetMatchIsActive(
                                    id, match_idx, v,
                                ))
                            }),
                            compact_bool("Urgent", rule_match.is_urgent, move |v| {
                                Message::WindowRules(WindowRulesMessage::SetMatchIsUrgent(
                                    id, match_idx, v,
                                ))
                            }),
                        ]
                        .spacing(8)
                        .wrap(),
                        row![
                            compact_bool(
                                "Active in column",
                                rule_match.is_active_in_column,
                                move |v| Message::WindowRules(
                                    WindowRulesMessage::SetMatchIsActiveInColumn(id, match_idx, v)
                                )
                            ),
                            compact_bool(
                                "Cast target",
                                rule_match.is_window_cast_target,
                                move |v| Message::WindowRules(
                                    WindowRulesMessage::SetMatchIsWindowCastTarget(
                                        id, match_idx, v
                                    )
                                )
                            ),
                            compact_bool("At startup", rule_match.at_startup, move |v| {
                                Message::WindowRules(WindowRulesMessage::SetMatchAtStartup(
                                    id, match_idx, v,
                                ))
                            }),
                        ]
                        .spacing(8)
                        .wrap(),
                    ]
                    .spacing(4),
                )
                .padding(8),
            );

            if rule.matches.len() > 1 {
                match_content = match_content.push(
                    button(
                        text(format!("Remove Match {}", match_idx + 1))
                            .size(11)
                            .color(neon::ERROR),
                    )
                    .on_press(Message::WindowRules(WindowRulesMessage::RemoveMatch(
                        id, match_idx,
                    )))
                    .padding([4, 8])
                    .style(ghost_button_style),
                );
            }
        }
        match_content = match_content.push(
            button(text("+ Add Match Criteria").size(12).color(neon::SECONDARY))
                .on_press(Message::WindowRules(WindowRulesMessage::AddMatch(id)))
                .padding([6, 12])
                .style(ghost_button_style),
        );
        editor = editor.push(match_content);
    }

    // ── EXCLUDE CRITERIA ──
    editor = editor.push(modal_section_header("✕", "EXCLUDE CRITERIA", neon::ERROR));
    {
        let mut exclude_content = column![].spacing(8);
        if rule.excludes.is_empty() {
            exclude_content = exclude_content.push(
                text("No exclude criteria — rule applies to all matching windows")
                    .size(12)
                    .color(neon::ON_SURFACE_VARIANT),
            );
        }
        for (idx, exclude_match) in rule.excludes.iter().enumerate() {
            let app_id_value = exclude_match.app_id.clone().unwrap_or_default();
            let title_value = exclude_match.title.clone().unwrap_or_default();

            exclude_content = exclude_content.push(
                row![
                    column![
                        text("APP ID")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        text_input("e.g., firefox", &app_id_value)
                            .on_input(move |value| Message::WindowRules(
                                WindowRulesMessage::SetExcludeAppId(
                                    id,
                                    idx,
                                    if value.is_empty() { None } else { Some(value) }
                                )
                            ))
                            .padding(10),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    column![
                        text("TITLE")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        text_input("e.g., .*YouTube.*", &title_value)
                            .on_input(move |value| Message::WindowRules(
                                WindowRulesMessage::SetExcludeTitle(
                                    id,
                                    idx,
                                    if value.is_empty() { None } else { Some(value) }
                                )
                            ))
                            .padding(10),
                    ]
                    .spacing(4)
                    .width(Length::FillPortion(1)),
                    button(text("✕").size(12).color(neon::ERROR))
                        .on_press(Message::WindowRules(WindowRulesMessage::RemoveExclude(
                            id, idx
                        )))
                        .padding([8, 10])
                        .style(ghost_button_style),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            );
        }
        exclude_content = exclude_content.push(
            button(text("+ Add Exclude").size(12).color(neon::ERROR))
                .on_press(Message::WindowRules(WindowRulesMessage::AddExclude(id)))
                .padding([6, 12])
                .style(ghost_button_style),
        );
        editor = editor.push(exclude_content);
    }

    editor = editor.push(Space::new().height(20));

    // ── ROW 1: OPENING BEHAVIOR | PLACEMENT ──
    editor = editor.push(
        row![
            column![
                modal_section_header("⚙", "OPENING BEHAVIOR", neon::TERTIARY),
                container(
                    column![
                        picker_row(
                            "Open as",
                            "How the window should open",
                            OpenBehavior::all(),
                            Some(rule.open_behavior),
                            move |value| Message::WindowRules(WindowRulesMessage::SetOpenBehavior(
                                id, value
                            )),
                        ),
                        optional_bool_picker(
                            "Open focused",
                            "Focus window when it opens",
                            rule.open_focused,
                            move |value| Message::WindowRules(WindowRulesMessage::SetOpenFocused(
                                id, value
                            )),
                        ),
                        optional_bool_picker(
                            "Maximize to edges",
                            "Maximize to screen edges (v25.11+)",
                            rule.open_maximized_to_edges,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetOpenMaximizedToEdges(id, value)
                            ),
                        ),
                        toggle_row(
                            "Block from screencast",
                            "Hide in screen recordings",
                            rule.block_out_from_screencast,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetBlockScreencast(id, value)
                            ),
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(8)
            .width(Length::FillPortion(1)),
            column![
                modal_section_header("▦", "PLACEMENT", neon::PRIMARY),
                text_input_row(
                    "Open on output",
                    "Output name (e.g., HDMI-1)",
                    rule.open_on_output.as_deref().unwrap_or(""),
                    move |value| Message::WindowRules(WindowRulesMessage::SetOpenOnOutput(
                        id,
                        if value.is_empty() { None } else { Some(value) }
                    )),
                ),
                text_input_with_suggestions(
                    "Open on workspace",
                    "Workspace name",
                    rule.open_on_workspace.as_deref().unwrap_or(""),
                    available_workspaces,
                    move |value| Message::WindowRules(WindowRulesMessage::SetOpenOnWorkspace(
                        id,
                        if value.is_empty() { None } else { Some(value) }
                    )),
                ),
            ]
            .spacing(8)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    );

    editor = editor.push(Space::new().height(20));

    // ── ROW 2: SIZE & DIMENSIONS | VISUAL STYLING ──
    editor = editor.push(
        row![
            column![
                modal_section_header("⊞", "SIZE & DIMENSIONS", neon::SECONDARY),
                Space::new().height(4),
                styled_slider(
                    "OPACITY",
                    &format!("{:.2}", rule.opacity.unwrap_or(1.0)),
                    move |s| s.parse::<f32>().ok().map(|v| Message::WindowRules(
                        WindowRulesMessage::SetOpacity(id, Some(v.clamp(0.0, 1.0)))
                    )),
                    0.0..=1.0,
                    rule.opacity.unwrap_or(1.0),
                    0.01,
                    move |v| Message::WindowRules(WindowRulesMessage::SetOpacity(id, Some(v))),
                ),
                row![
                    styled_slider(
                        "COLUMN WIDTH",
                        &format!("{:.0}%", rule.default_column_width.unwrap_or(0.5) * 100.0),
                        move |s| s.replace('%', "").parse::<f32>().ok().map(|v| {
                            Message::WindowRules(WindowRulesMessage::SetDefaultColumnWidth(
                                id,
                                Some((v / 100.0).clamp(0.1, 1.0)),
                            ))
                        }),
                        0.1..=1.0,
                        rule.default_column_width.unwrap_or(0.5),
                        0.01,
                        move |v| Message::WindowRules(WindowRulesMessage::SetDefaultColumnWidth(
                            id,
                            Some(v)
                        )),
                    ),
                    styled_slider(
                        "WINDOW HEIGHT",
                        &format!("{:.0}%", rule.default_window_height.unwrap_or(0.5) * 100.0),
                        move |s| s.replace('%', "").parse::<f32>().ok().map(|v| {
                            Message::WindowRules(WindowRulesMessage::SetDefaultWindowHeight(
                                id,
                                Some((v / 100.0).clamp(0.1, 1.0)),
                            ))
                        }),
                        0.1..=1.0,
                        rule.default_window_height.unwrap_or(0.5),
                        0.01,
                        move |v| Message::WindowRules(WindowRulesMessage::SetDefaultWindowHeight(
                            id,
                            Some(v)
                        )),
                    ),
                ]
                .spacing(8),
                row![
                    styled_slider_int(
                        "MIN WIDTH",
                        &format!("{}", rule.min_width.unwrap_or(0)),
                        move |s| s.parse::<i32>().ok().map(|v| Message::WindowRules(
                            WindowRulesMessage::SetMinWidth(id, Some(v.clamp(0, 9999)))
                        )),
                        0..=4000,
                        rule.min_width.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetMinWidth(id, Some(v))),
                    ),
                    styled_slider_int(
                        "MAX WIDTH",
                        &format!("{}", rule.max_width.unwrap_or(0)),
                        move |s| s.parse::<i32>().ok().map(|v| Message::WindowRules(
                            WindowRulesMessage::SetMaxWidth(id, Some(v.clamp(0, 9999)))
                        )),
                        0..=4000,
                        rule.max_width.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetMaxWidth(id, Some(v))),
                    ),
                ]
                .spacing(8),
                row![
                    styled_slider_int(
                        "MIN HEIGHT",
                        &format!("{}", rule.min_height.unwrap_or(0)),
                        move |s| s.parse::<i32>().ok().map(|v| Message::WindowRules(
                            WindowRulesMessage::SetMinHeight(id, Some(v.clamp(0, 9999)))
                        )),
                        0..=4000,
                        rule.min_height.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetMinHeight(
                            id,
                            Some(v)
                        )),
                    ),
                    styled_slider_int(
                        "MAX HEIGHT",
                        &format!("{}", rule.max_height.unwrap_or(0)),
                        move |s| s.parse::<i32>().ok().map(|v| Message::WindowRules(
                            WindowRulesMessage::SetMaxHeight(id, Some(v.clamp(0, 9999)))
                        )),
                        0..=4000,
                        rule.max_height.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetMaxHeight(
                            id,
                            Some(v)
                        )),
                    ),
                ]
                .spacing(8),
                styled_slider(
                    "SCROLL FACTOR",
                    &format!("{:.1}", rule.scroll_factor.unwrap_or(1.0)),
                    move |s| s.parse::<f64>().ok().map(|v| Message::WindowRules(
                        WindowRulesMessage::SetScrollFactor(id, Some(v.clamp(0.1, 10.0)))
                    )),
                    0.1..=5.0,
                    rule.scroll_factor.unwrap_or(1.0) as f32,
                    0.1,
                    move |v| Message::WindowRules(WindowRulesMessage::SetScrollFactor(
                        id,
                        Some(v as f64)
                    )),
                ),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                modal_section_header("◉", "VISUAL STYLING", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        optional_bool_picker(
                            "Focus ring",
                            "Override focus ring on/off",
                            rule.focus_ring_enabled,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetFocusRingEnabled(id, value)
                            ),
                        ),
                        optional_bool_picker(
                            "Border",
                            "Override border on/off",
                            rule.border_enabled,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetBorderEnabled(id, value)
                            ),
                        ),
                        optional_bool_picker(
                            "Clip to geometry",
                            "Clip rendering to visual bounds",
                            rule.clip_to_geometry,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetClipToGeometry(id, value)
                            ),
                        ),
                        optional_bool_picker(
                            "Draw border with bg",
                            "Draw border with background",
                            rule.draw_border_with_background,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetDrawBorderWithBackground(id, value)
                            ),
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                row![
                    styled_slider_int(
                        "CORNER RADIUS",
                        &format!("{}px", rule.corner_radius.unwrap_or(0)),
                        move |s| s.replace("px", "").parse::<i32>().ok().map(|v| {
                            Message::WindowRules(WindowRulesMessage::SetCornerRadius(
                                id,
                                Some(v.clamp(0, 32)),
                            ))
                        }),
                        0..=32,
                        rule.corner_radius.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetCornerRadius(
                            id,
                            Some(v)
                        )),
                    ),
                    styled_slider_int(
                        "FOCUS RING W",
                        &format!("{}px", rule.focus_ring_width.unwrap_or(0)),
                        move |s| s.replace("px", "").parse::<i32>().ok().map(|v| {
                            Message::WindowRules(WindowRulesMessage::SetFocusRingWidth(
                                id,
                                Some(v.clamp(0, 20)),
                            ))
                        }),
                        0..=20,
                        rule.focus_ring_width.unwrap_or(0),
                        move |v| Message::WindowRules(WindowRulesMessage::SetFocusRingWidth(
                            id,
                            Some(v)
                        )),
                    ),
                ]
                .spacing(8),
                styled_slider_int(
                    "BORDER WIDTH",
                    &format!("{}px", rule.border_width.unwrap_or(0)),
                    move |s| s
                        .replace("px", "")
                        .parse::<i32>()
                        .ok()
                        .map(|v| Message::WindowRules(WindowRulesMessage::SetBorderWidth(
                            id,
                            Some(v.clamp(0, 20))
                        ))),
                    0..=20,
                    rule.border_width.unwrap_or(0),
                    move |v| Message::WindowRules(WindowRulesMessage::SetBorderWidth(id, Some(v))),
                ),
                // Focus ring colors
                color_picker_row(
                    "Focus ring active",
                    "Active color override",
                    &color_or_gradient_to_niri(rule.focus_ring_active.as_ref()),
                    move |hex| Message::WindowRules(WindowRulesMessage::SetFocusRingActive(
                        id,
                        Some(ColorOrGradient::Color(hex_to_niri_color(&hex)))
                    )),
                ),
                color_picker_row(
                    "Focus ring inactive",
                    "Inactive color override",
                    &color_or_gradient_to_niri(rule.focus_ring_inactive.as_ref()),
                    move |hex| Message::WindowRules(WindowRulesMessage::SetFocusRingInactive(
                        id,
                        Some(ColorOrGradient::Color(hex_to_niri_color(&hex)))
                    )),
                ),
                // Border colors
                color_picker_row(
                    "Border active",
                    "Active color override",
                    &color_or_gradient_to_niri(rule.border_active.as_ref()),
                    move |hex| Message::WindowRules(WindowRulesMessage::SetBorderActive(
                        id,
                        Some(ColorOrGradient::Color(hex_to_niri_color(&hex)))
                    )),
                ),
                color_picker_row(
                    "Border inactive",
                    "Inactive color override",
                    &color_or_gradient_to_niri(rule.border_inactive.as_ref()),
                    move |hex| Message::WindowRules(WindowRulesMessage::SetBorderInactive(
                        id,
                        Some(ColorOrGradient::Color(hex_to_niri_color(&hex)))
                    )),
                ),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    );

    editor = editor.push(Space::new().height(20));

    // ── ROW 3: ADVANCED | KDL PREVIEW ──
    let kdl_preview = rule_to_kdl_preview(rule);
    editor = editor.push(row![
        column![
            modal_section_header("⬡", "ADVANCED", neon::OUTLINE),
            container(column![
                optional_bool_picker("Variable refresh rate", "Enable VRR/FreeSync",
                    rule.variable_refresh_rate,
                    move |value| Message::WindowRules(WindowRulesMessage::SetVariableRefreshRate(id, value)),
                ),
                optional_bool_picker("Floating animation", "baba-is-float effect",
                    rule.baba_is_float,
                    move |value| Message::WindowRules(WindowRulesMessage::SetBabaIsFloat(id, value)),
                ),
                optional_bool_picker("Tiled state", "Mark as tiled (X11 compat)",
                    rule.tiled_state,
                    move |value| Message::WindowRules(WindowRulesMessage::SetTiledState(id, value)),
                ),
                picker_row("Column display", "Default display mode",
                    &[DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
                    rule.default_column_display,
                    move |value| Message::WindowRules(WindowRulesMessage::SetDefaultColumnDisplay(id, Some(value))),
                ),
            ].spacing(0)).padding(8).style(crate::theme::card_style),
            Space::new().height(8),
            // Floating position
            modal_section_header("◇", "FLOATING POSITION", neon::TERTIARY),
            {
                let pos = rule.default_floating_position.clone().unwrap_or(FloatingPosition { x: 0, y: 0, relative_to: PositionRelativeTo::TopLeft });
                container(column![
                    row![
                        column![
                            text("X").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT),
                            text_input("0", &format!("{}", pos.x))
                                .on_input(move |s| {
                                    if let Ok(x) = s.parse::<i32>() {
                                        let mut p = rule.default_floating_position.clone().unwrap_or(FloatingPosition { x: 0, y: 0, relative_to: PositionRelativeTo::TopLeft });
                                        p.x = x;
                                        Message::WindowRules(WindowRulesMessage::SetDefaultFloatingPosition(id, Some(p)))
                                    } else { Message::NoOp }
                                })
                                .padding(8).size(12),
                        ].spacing(4).width(Length::FillPortion(1)),
                        column![
                            text("Y").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT),
                            text_input("0", &format!("{}", pos.y))
                                .on_input(move |s| {
                                    if let Ok(y) = s.parse::<i32>() {
                                        let mut p = rule.default_floating_position.clone().unwrap_or(FloatingPosition { x: 0, y: 0, relative_to: PositionRelativeTo::TopLeft });
                                        p.y = y;
                                        Message::WindowRules(WindowRulesMessage::SetDefaultFloatingPosition(id, Some(p)))
                                    } else { Message::NoOp }
                                })
                                .padding(8).size(12),
                        ].spacing(4).width(Length::FillPortion(1)),
                    ].spacing(8),
                    picker_row("Relative to", "Anchor point",
                        PositionRelativeTo::all(),
                        Some(pos.relative_to),
                        move |value| {
                            let mut p = rule.default_floating_position.clone().unwrap_or(FloatingPosition { x: 0, y: 0, relative_to: PositionRelativeTo::TopLeft });
                            p.relative_to = value;
                            Message::WindowRules(WindowRulesMessage::SetDefaultFloatingPosition(id, Some(p)))
                        },
                    ),
                ].spacing(8)).padding(8).style(crate::theme::card_style)
            },
            Space::new().height(8),
            info_text("Per-window shadow, tab indicator, and urgent color overrides can be configured via KDL. See the live preview for the full config block."),
        ].spacing(8).width(Length::FillPortion(1)),

        column![
            row![
                text("CUSTOM KDL BLOCK").size(10).font(fonts::UI_FONT_SEMIBOLD).color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text("Live Preview").size(10).color(neon::SECONDARY),
            ].padding([10, 0]),
            container(
                scrollable(
                    text(kdl_preview).size(12).font(fonts::MONO_FONT).color(neon::ON_SURFACE_VARIANT),
                ).height(Length::Fixed(160.0)),
            )
            .padding(16).width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(neon::SURFACE_LOW)),
                border: iced::Border {
                    color: iced::Color { a: 0.15, ..neon::OUTLINE_VARIANT },
                    width: 1.0, radius: 12.0.into(),
                },
                ..Default::default()
            }),
        ].spacing(0).width(Length::FillPortion(1)),
    ].spacing(32).align_y(Alignment::Start));

    // ── Footer ──
    editor = editor.push(Space::new().height(20));
    editor = editor.push(
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
    );
    editor = editor.push(
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
                    .on_press(Message::WindowRules(WindowRulesMessage::CloseEditor))
                    .padding([10, 20])
                    .style(ghost_button_style),
                Space::new().width(8),
                button(text("Save Changes").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::WindowRules(WindowRulesMessage::CloseEditor))
                    .padding([10, 24])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => neon::PRIMARY,
                            _ => iced::Color {
                                a: 0.85,
                                ..neon::PRIMARY
                            },
                        };
                        iced::widget::button::Style {
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
            .align_y(Alignment::Center),
        )
        .padding([16, 0]),
    );

    // Wrap in scrollable modal container
    let modal_content = scrollable(editor.spacing(12).width(Length::Fill)).height(Length::Fill);

    // Modal dialog
    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(960.0))
        .max_height(750.0)
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

    // Backdrop
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

// ── Helper Components ──────────────────────────────────────────────────────

/// Generate a simplified KDL preview for a window rule
fn rule_to_kdl_preview(rule: &WindowRule) -> String {
    let mut lines = Vec::new();
    let name = if rule.name.is_empty() {
        "unnamed"
    } else {
        &rule.name
    };
    lines.push(format!("window-rule {{"));

    for m in &rule.matches {
        let mut parts = Vec::new();
        if let Some(ref app_id) = m.app_id {
            parts.push(format!("app-id=\"{}\"", app_id));
        }
        if let Some(ref title) = m.title {
            parts.push(format!("title=\"{}\"", title));
        }
        if !parts.is_empty() {
            lines.push(format!("    match {}", parts.join(" ")));
        }
    }

    for e in &rule.excludes {
        let mut parts = Vec::new();
        if let Some(ref app_id) = e.app_id {
            parts.push(format!("app-id=\"{}\"", app_id));
        }
        if let Some(ref title) = e.title {
            parts.push(format!("title=\"{}\"", title));
        }
        if !parts.is_empty() {
            lines.push(format!("    exclude {}", parts.join(" ")));
        }
    }

    match rule.open_behavior {
        OpenBehavior::Floating => lines.push("    open-floating true".to_string()),
        OpenBehavior::Maximized => lines.push("    open-maximized true".to_string()),
        OpenBehavior::Fullscreen => lines.push("    open-fullscreen true".to_string()),
        OpenBehavior::Normal => {}
    }
    if let Some(focused) = rule.open_focused {
        lines.push(format!("    open-focused {}", focused));
    }
    if let Some(ref output) = rule.open_on_output {
        lines.push(format!("    open-on-output \"{}\"", output));
    }
    if let Some(ref ws) = rule.open_on_workspace {
        lines.push(format!("    open-on-workspace \"{}\"", ws));
    }
    if rule.block_out_from_screencast {
        lines.push("    block-out-from \"screencast\"".to_string());
    }
    if let Some(width) = rule.default_column_width {
        lines.push(format!("    default-column-width {:.2}", width));
    }
    if let Some(height) = rule.default_window_height {
        lines.push(format!("    default-window-height {:.2}", height));
    }
    if let Some(opacity) = rule.opacity {
        lines.push(format!("    opacity {:.2}", opacity));
    }
    if let Some(radius) = rule.corner_radius {
        lines.push(format!("    geometry-corner-radius {}", radius));
    }
    if let Some(clip) = rule.clip_to_geometry {
        lines.push(format!("    clip-to-geometry {}", clip));
    }
    if let Some(focus_ring) = rule.focus_ring_enabled {
        lines.push(format!("    focus-ring {{ off {}; }}", !focus_ring));
    }
    if let Some(border) = rule.border_enabled {
        lines.push(format!("    border {{ off {}; }}", !border));
    }
    if let Some(min_w) = rule.min_width {
        lines.push(format!("    min-width {}", min_w));
    }
    if let Some(max_w) = rule.max_width {
        lines.push(format!("    max-width {}", max_w));
    }
    if let Some(min_h) = rule.min_height {
        lines.push(format!("    min-height {}", min_h));
    }
    if let Some(max_h) = rule.max_height {
        lines.push(format!("    max-height {}", max_h));
    }
    if let Some(scroll) = rule.scroll_factor {
        lines.push(format!("    scroll-factor {:.1}", scroll));
    }
    if let Some(fw) = rule.focus_ring_width {
        lines.push(format!("    focus-ring {{ width {}; }}", fw));
    }
    if let Some(bw) = rule.border_width {
        lines.push(format!("    border {{ width {}; }}", bw));
    }
    if let Some(edges) = rule.open_maximized_to_edges {
        lines.push(format!("    open-maximized-to-edges {}", edges));
    }
    if let Some(vrr) = rule.variable_refresh_rate {
        lines.push(format!("    variable-refresh-rate {}", vrr));
    }
    if let Some(tiled) = rule.tiled_state {
        lines.push(format!("    tiled-state {}", tiled));
    }
    if let Some(baba) = rule.baba_is_float {
        lines.push(format!("    baba-is-float {}", baba));
    }

    lines.push("}".to_string());

    // Add comment with rule name
    format!("// {}\n{}", name, lines.join("\n"))
}

/// Compact inline bool picker for match criteria (shown as small pill-style toggle)
fn compact_bool<'a>(
    label: &'a str,
    value: Option<bool>,
    on_change: impl Fn(Option<bool>) -> Message + 'a,
) -> Element<'a, Message> {
    let display = match value {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "Any",
    };
    let color = match value {
        Some(true) => neon::SECONDARY,
        Some(false) => neon::TERTIARY,
        None => neon::OUTLINE_VARIANT,
    };
    button(
        row![
            text(label).size(10).color(neon::ON_SURFACE_VARIANT),
            text(display)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(color),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .on_press({
        // Cycle: None -> Some(true) -> Some(false) -> None
        let next = match value {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };
        on_change(next)
    })
    .padding([4, 10])
    .style(move |_: &iced::Theme, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => iced::Color { a: 0.12, ..color },
            _ => iced::Color { a: 0.06, ..color },
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: neon::ON_SURFACE,
            border: iced::Border {
                radius: 6.0.into(),
                color: iced::Color { a: 0.15, ..color },
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Styled slider with uppercase label and editable value box
fn styled_slider<'a>(
    label: &'a str,
    display_value: &str,
    on_text: impl Fn(String) -> Option<Message> + 'a,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_slide: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    let display_owned = display_value.to_string();
    let current_val = value;
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text_input("", &display_owned)
                    .on_input(move |s| { on_text(s).unwrap_or(Message::NoOp) })
                    .padding([4, 8])
                    .size(11)
                    .width(Length::Fixed(55.0)),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, current_val, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4)
        .padding(12),
    )
    .style(crate::theme::card_style)
    .into()
}

/// Styled slider for integer values
fn styled_slider_int<'a>(
    label: &'a str,
    display_value: &str,
    on_text: impl Fn(String) -> Option<Message> + 'a,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let display_owned = display_value.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text_input("", &display_owned)
                    .on_input(move |s| { on_text(s).unwrap_or(Message::NoOp) })
                    .padding([4, 8])
                    .size(11)
                    .width(Length::Fixed(50.0)),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide).width(Length::Fill),
        ]
        .spacing(4)
        .padding(12),
    )
    .style(crate::theme::card_style)
    .into()
}

/// Modal section header: icon + uppercase label + accent line
fn modal_section_header<'a>(
    icon: &'a str,
    label: &'a str,
    accent: iced::Color,
) -> Element<'a, Message> {
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

/// Colored pill/tag with neon glow styling
fn pill<'a>(label: &str, color: iced::Color) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(11)
            .font(fonts::UI_FONT_MEDIUM)
            .color(color),
    )
    .padding([5, 12])
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color { a: 0.10, ..color })),
        border: iced::Border {
            color: iced::Color { a: 0.30, ..color },
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color { a: 0.08, ..color },
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    })
    .into()
}

fn rule_summary_section<'a>(
    label: &'static str,
    pills: &[(String, iced::Color)],
    empty_label: &'static str,
) -> Element<'a, Message> {
    let mut visible_pills: Vec<Element<'a, Message>> = pills
        .iter()
        .take(MAX_VISIBLE_SUMMARY_PILLS)
        .map(|(pill_label, color)| pill(pill_label, *color))
        .collect();

    if pills.len() > MAX_VISIBLE_SUMMARY_PILLS {
        visible_pills.push(pill(
            &format!("+{} more", pills.len() - MAX_VISIBLE_SUMMARY_PILLS),
            neon::ON_SURFACE_VARIANT,
        ));
    }

    let summary_content: Element<'a, Message> = if visible_pills.is_empty() {
        text(empty_label)
            .size(11)
            .color(neon::ON_SURFACE_VARIANT)
            .into()
    } else {
        row(visible_pills).spacing(6).wrap().into()
    };

    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            summary_content,
        ]
        .spacing(8),
    )
    .width(Length::Fill)
    .height(Length::Fixed(RULE_CARD_SECTION_HEIGHT))
    .into()
}

/// Filter tabs (Active / Disabled / All)
fn filter_tabs(active_filter: RulesFilter) -> Element<'static, Message> {
    let tab = |label: &'static str, filter: RulesFilter| {
        let is_active = active_filter == filter;
        button(text(label).size(12).font(if is_active {
            fonts::UI_FONT_MEDIUM
        } else {
            fonts::UI_FONT
        }))
        .on_press(Message::WindowRules(WindowRulesMessage::SetFilter(filter)))
        .padding([6, 14])
        .style(move |_: &iced::Theme, _| {
            let (bg, text_color) = if is_active {
                (
                    iced::Color {
                        a: 0.15,
                        ..neon::PRIMARY
                    },
                    neon::PRIMARY,
                )
            } else {
                (iced::Color::TRANSPARENT, neon::ON_SURFACE_VARIANT)
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    };

    container(
        row![
            tab("Active", RulesFilter::Active),
            tab("Disabled", RulesFilter::Disabled),
            tab("All", RulesFilter::All),
        ]
        .spacing(2),
    )
    .padding(4)
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
        border: iced::Border {
            radius: 10.0.into(),
            color: iced::Color {
                a: 0.2,
                ..neon::OUTLINE_VARIANT
            },
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}

/// Empty state shown when no rules match
fn empty_state<'a>(search: &str, filter: RulesFilter) -> Element<'a, Message> {
    let has_filters = !search.is_empty() || filter != RulesFilter::All;
    let (title, message) = if has_filters {
        (
            "No Active Rules Matching",
            "Manage how your windows behave automatically. Adjust opacity, workspace assignments, and floating states with technical precision.",
        )
    } else {
        (
            "No Window Rules Yet",
            "Window rules let you configure per-application behavior — floating mode, workspace placement, opacity, and more.",
        )
    };

    container(
        column![
            // Decorative icon in a circle
            container(text("⊞").size(32).color(neon::PRIMARY),)
                .width(72)
                .height(72)
                .center(Length::Shrink)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        a: 0.12,
                        ..neon::PRIMARY
                    })),
                    border: iced::Border {
                        radius: 999.0.into(),
                        color: iced::Color {
                            a: 0.2,
                            ..neon::PRIMARY
                        },
                        width: 1.0,
                    },
                    ..Default::default()
                }),
            Space::new().height(16),
            text(title).size(22).font(fonts::UI_FONT_SEMIBOLD),
            text(message)
                .size(13)
                .color(neon::ON_SURFACE_VARIANT)
                .width(Length::Fixed(400.0))
                .center(),
            Space::new().height(16),
            row![if has_filters {
                Element::from(
                    button(text("Clear Filters").size(13))
                        .on_press(Message::WindowRules(WindowRulesMessage::SetFilter(
                            RulesFilter::All,
                        )))
                        .padding([10, 20])
                        .style(ghost_button_style),
                )
            } else {
                Element::from(
                    button(text("+ New Rule").size(13).font(fonts::UI_FONT_MEDIUM))
                        .on_press(Message::WindowRules(WindowRulesMessage::AddRule))
                        .padding([10, 20])
                        .style(|_: &iced::Theme, status| {
                            let bg = match status {
                                iced::widget::button::Status::Hovered => neon::PRIMARY,
                                _ => iced::Color {
                                    a: 0.8,
                                    ..neon::PRIMARY
                                },
                            };
                            iced::widget::button::Style {
                                background: Some(iced::Background::Color(bg)),
                                text_color: neon::ON_SURFACE,
                                border: iced::Border {
                                    radius: 10.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                )
            },]
            .spacing(12),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(60)
    .center(Length::Fill)
    .into()
}

/// Stats bar at the bottom
fn stats_bar<'a>(active: usize, total: usize) -> Element<'a, Message> {
    container(
        row![
            text(format!("{} Active", active))
                .size(12)
                .font(fonts::UI_FONT_MEDIUM)
                .color(neon::SECONDARY),
            text("·").size(12).color(neon::OUTLINE_VARIANT),
            text(format!("{} Total", total))
                .size(12)
                .color(neon::ON_SURFACE_VARIANT),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([8, 16])
    .width(Length::Fill)
    .center_x(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
        border: iced::Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Ghost button style (transparent bg, subtle hover)
fn ghost_button_style(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let bg = match status {
        iced::widget::button::Status::Hovered => iced::Color {
            a: 0.08,
            ..neon::ON_SURFACE
        },
        _ => iced::Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: neon::ON_SURFACE,
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Helper to truncate strings for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

/// Extract NiriColor from ColorOrGradient (uses first color of gradient)
fn color_or_gradient_to_niri(cog: Option<&ColorOrGradient>) -> NiriColor {
    match cog {
        Some(ColorOrGradient::Color(c)) => c.clone(),
        Some(ColorOrGradient::Gradient(g)) => g.from.clone(),
        None => NiriColor {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        },
    }
}

/// Parse hex string to NiriColor
fn hex_to_niri_color(hex: &str) -> NiriColor {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(255),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(255),
            255,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(255),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(255),
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255),
        ),
        _ => (255, 255, 255, 255),
    };
    NiriColor { r, g, b, a }
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
