//! Layer rules settings view — card grid + modal editor
//!
//! Displays layer rules as a card grid with matcher/effect pills.
//! Editing is done through a modal overlay.

use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{BlockOutFrom, LayerRule, LayerRulesSettings};
use crate::messages::{LayerRulesMessage, Message, RulesFilter};
use crate::theme::{fonts, neon};

const RULE_CARD_HEIGHT: f32 = 320.0;
const RULE_CARD_SECTION_HEIGHT: f32 = 68.0;
const MAX_VISIBLE_SUMMARY_PILLS: usize = 2;

// ── Card Grid View ─────────────────────────────────────────────────────────

/// Creates the layer rules card grid view
pub fn view<'a>(
    settings: &'a LayerRulesSettings,
    search: &'a str,
    filter: RulesFilter,
    _sections_expanded: &'a HashMap<(u32, String), bool>,
    _regex_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    let search_owned = search.to_string();

    let filtered_rules: Vec<&LayerRule> = settings
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
                m.namespace
                    .as_ref()
                    .is_some_and(|ns| ns.to_lowercase().contains(&search_lower))
            })
        })
        .collect();

    let active_count = settings.rules.iter().filter(|r| r.enabled).count();

    let mut content = column![
        // Search bar + filter tabs
        row![
            container(
                row![
                    text("⌕").size(16).color(neon::OUTLINE_VARIANT),
                    text_input("Search by name or namespace...", &search_owned)
                        .on_input(|s| Message::LayerRules(LayerRulesMessage::SetSearch(s)))
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
        let mut col1: Vec<Element<'a, Message>> = Vec::new();
        let mut col2: Vec<Element<'a, Message>> = Vec::new();
        let mut col3: Vec<Element<'a, Message>> = Vec::new();

        for (i, rule) in filtered_rules.iter().enumerate() {
            let card = rule_card(rule);
            match i % 3 {
                0 => col1.push(card),
                1 => col2.push(card),
                _ => col3.push(card),
            }
        }

        content = content.push(
            row![
                column(col1).spacing(12).width(Length::FillPortion(1)),
                column(col2).spacing(12).width(Length::FillPortion(1)),
                column(col3).spacing(12).width(Length::FillPortion(1)),
            ]
            .spacing(12)
            .align_y(Alignment::Start),
        );
    }

    content = content.push(Space::new().height(16));
    content = content.push(stats_bar(active_count, settings.rules.len()));

    content.into()
}

// ── Rule Card ──────────────────────────────────────────────────────────────

fn rule_card(rule: &LayerRule) -> Element<'_, Message> {
    let id = rule.id;
    let enabled = rule.enabled;

    let accent = match id % 3 {
        0 => neon::PRIMARY,
        1 => neon::SECONDARY,
        _ => neon::TERTIARY,
    };

    let avatar_char = rule
        .matches
        .first()
        .and_then(|m| m.namespace.as_ref())
        .and_then(|ns| ns.chars().next())
        .unwrap_or('L')
        .to_uppercase()
        .next()
        .unwrap_or('L');

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

    let ns_display: String = rule
        .matches
        .first()
        .and_then(|m| m.namespace.as_ref())
        .map(|ns| format!("namespace: {}", ns))
        .unwrap_or_else(|| "any layer surface".to_string());

    // Matcher pills
    let mut matcher_pills: Vec<(String, iced::Color)> = Vec::new();
    for m in &rule.matches {
        if let Some(ref ns) = m.namespace {
            matcher_pills.push((format!("NS: {}", truncate_str(ns, 18)), neon::SECONDARY));
        }
    }

    // Effect pills
    let mut effect_pills: Vec<(String, iced::Color)> = Vec::new();
    if let Some(block_out) = rule.block_out_from {
        let label = match block_out {
            BlockOutFrom::Screencast => "⊘ Hide: Cast",
            BlockOutFrom::ScreenCapture => "⊘ Hide: All",
        };
        effect_pills.push((label.to_string(), neon::ERROR));
    }
    if let Some(opacity) = rule.opacity {
        effect_pills.push((format!("◉ Opacity {:.2}", opacity), neon::PRIMARY));
    }
    if rule.place_within_backdrop {
        effect_pills.push(("▤ Backdrop".to_string(), neon::TERTIARY));
    }
    if rule.baba_is_float {
        effect_pills.push(("◇ Float Anim".to_string(), neon::TERTIARY));
    }

    let card_content = column![
        row![
            avatar,
            column![
                text(&rule.name).size(15).font(fonts::UI_FONT_SEMIBOLD),
                text(ns_display).size(11).color(neon::ON_SURFACE_VARIANT),
            ]
            .spacing(2)
            .width(Length::Fill),
            toggler(enabled)
                .on_toggle(move |v| Message::LayerRules(LayerRulesMessage::SetRuleEnabled(id, v)))
                .width(Length::Shrink),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        Space::new().height(12),
        rule_summary_section("MATCHERS", &matcher_pills, "Matches any layer"),
        rule_summary_section("EFFECTS", &effect_pills, "No rule effects"),
        Space::new().height(Length::Fill),
        // Divider
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
        row![
            button(row![text("✎").size(12), text("Edit Rule").size(12)].spacing(4))
                .on_press(Message::LayerRules(LayerRulesMessage::OpenEditor(id)))
                .padding([6, 12])
                .style(ghost_button_style),
            Space::new().width(Length::Fill),
            button(
                row![
                    text("🗑").size(12),
                    text("Remove").size(12).color(neon::ERROR)
                ]
                .spacing(4)
            )
            .on_press(Message::LayerRules(LayerRulesMessage::DeleteRule(id)))
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

pub fn editor_modal<'a>(
    rule: &'a LayerRule,
    _sections_expanded: &'a HashMap<(u32, String), bool>,
    regex_errors: &'a HashMap<(u32, String), String>,
) -> Element<'a, Message> {
    let id = rule.id;

    let mut editor = column![
        // Header
        row![
            container(text("⊡").size(24).color(neon::SECONDARY))
                .width(48)
                .height(48)
                .center(Length::Shrink)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        a: 0.15,
                        ..neon::SECONDARY
                    })),
                    border: iced::Border {
                        radius: 14.0.into(),
                        color: iced::Color {
                            a: 0.25,
                            ..neon::SECONDARY
                        },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("LAYER RULE EDITOR")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::SECONDARY),
                row![
                    text("Modify Rule: ").size(22).font(fonts::UI_FONT_SEMIBOLD),
                    text(&rule.name)
                        .size(22)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::SECONDARY),
                ],
            ]
            .spacing(4)
            .width(Length::Fill),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::LayerRules(LayerRulesMessage::CloseEditor))
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
        // Rule name
        row![
            text("Rule Name")
                .size(12)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::ON_SURFACE_VARIANT),
            Space::new().width(12),
            text_input("Rule name", &rule.name)
                .on_input(move |name| Message::LayerRules(LayerRulesMessage::SetRuleName(id, name)))
                .padding(10)
                .size(14)
                .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
    ]
    .spacing(0);

    // ── NAMESPACE MATCHERS ──
    editor = editor.push(modal_section_header(
        "▼",
        "NAMESPACE MATCHERS",
        neon::SECONDARY,
    ));
    for (idx, rule_match) in rule.matches.iter().enumerate() {
        let ns_value = rule_match.namespace.clone().unwrap_or_default();
        let ns_error_key = (id, format!("namespace_{}", idx));
        let ns_error = regex_errors.get(&ns_error_key);

        let mut ns_col =
            column![
                text("NAMESPACE (REGEX)")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                text_input("e.g., waybar", &ns_value)
                    .on_input(move |value| Message::LayerRules(
                        LayerRulesMessage::SetMatchNamespace(id, idx, value)
                    ))
                    .padding(12),
            ]
            .spacing(6);
        if let Some(error) = ns_error {
            ns_col = ns_col.push(text(error).size(11).color(neon::ERROR));
        }

        let mut match_row = column![ns_col];
        match_row = match_row.push(optional_bool_picker(
            "At startup only",
            "Match only during first 60 seconds",
            rule_match.at_startup,
            move |value| Message::LayerRules(LayerRulesMessage::SetMatchAtStartup(id, idx, value)),
        ));

        if rule.matches.len() > 1 {
            match_row = match_row.push(
                button(
                    text(format!("Remove Match {}", idx + 1))
                        .size(11)
                        .color(neon::ERROR),
                )
                .on_press(Message::LayerRules(LayerRulesMessage::RemoveMatch(id, idx)))
                .padding([4, 8])
                .style(ghost_button_style),
            );
        }
        editor = editor.push(match_row);
    }
    editor = editor.push(
        button(text("+ Add Match").size(12).color(neon::SECONDARY))
            .on_press(Message::LayerRules(LayerRulesMessage::AddMatch(id)))
            .padding([6, 12])
            .style(ghost_button_style),
    );

    editor = editor.push(Space::new().height(20));

    // ── 2-COLUMN: VISIBILITY | STYLING ──
    editor = editor.push(
        row![
            column![
                modal_section_header("⊘", "VISIBILITY", neon::ERROR),
                block_out_picker(
                    "Block out from",
                    "Hide from recordings/captures",
                    rule.block_out_from,
                    move |value| Message::LayerRules(LayerRulesMessage::SetBlockOutFrom(id, value)),
                ),
            ]
            .spacing(8)
            .width(Length::FillPortion(1)),
            column![
                modal_section_header("◉", "STYLING", neon::TERTIARY),
                Space::new().height(4),
                styled_slider(
                    "OPACITY",
                    &format!("{:.2}", rule.opacity.unwrap_or(1.0)),
                    move |s| s.parse::<f32>().ok().map(|v| Message::LayerRules(
                        LayerRulesMessage::SetOpacity(id, Some(v.clamp(0.0, 1.0)))
                    )),
                    0.0..=1.0,
                    rule.opacity.unwrap_or(1.0),
                    0.01,
                    move |v| Message::LayerRules(LayerRulesMessage::SetOpacity(id, Some(v))),
                ),
                styled_slider_int(
                    "CORNER RADIUS",
                    &format!("{}px", rule.geometry_corner_radius.unwrap_or(0)),
                    move |s| s
                        .replace("px", "")
                        .parse::<i32>()
                        .ok()
                        .map(|v| Message::LayerRules(LayerRulesMessage::SetCornerRadius(
                            id,
                            Some(v.clamp(0, 32))
                        ))),
                    0..=32,
                    rule.geometry_corner_radius.unwrap_or(0),
                    move |v| Message::LayerRules(LayerRulesMessage::SetCornerRadius(id, Some(v))),
                ),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    );

    editor = editor.push(Space::new().height(20));

    // ── ADVANCED ──
    editor = editor.push(modal_section_header("⬡", "ADVANCED", neon::OUTLINE));
    editor = editor.push(
        container(
            column![
                toggle_row(
                    "Place within backdrop",
                    "Place this layer within the desktop backdrop (v25.05+)",
                    rule.place_within_backdrop,
                    move |value| Message::LayerRules(LayerRulesMessage::SetPlaceWithinBackdrop(
                        id, value
                    )),
                ),
                toggle_row(
                    "Treat as floating",
                    "Use floating window animations (v25.05+)",
                    rule.baba_is_float,
                    move |value| Message::LayerRules(LayerRulesMessage::SetBabaIsFloat(id, value)),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
    );

    editor = editor.push(Space::new().height(8));
    editor = editor.push(info_text(
        "Per-layer shadow overrides can be configured via KDL.",
    ));

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
                    .on_press(Message::LayerRules(LayerRulesMessage::CloseEditor))
                    .padding([10, 20])
                    .style(ghost_button_style),
                Space::new().width(8),
                button(text("Save Changes").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::LayerRules(LayerRulesMessage::CloseEditor))
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

    // Wrap in scrollable modal
    let modal_content = scrollable(editor.spacing(12).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(800.0))
        .max_height(700.0)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color {
                    a: 0.3,
                    ..neon::SECONDARY
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

// ── Helpers ────────────────────────────────────────────────────────────────

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

fn filter_tabs(active_filter: RulesFilter) -> Element<'static, Message> {
    let tab = |label: &'static str, filter: RulesFilter| {
        let is_active = active_filter == filter;
        button(text(label).size(12).font(if is_active {
            fonts::UI_FONT_MEDIUM
        } else {
            fonts::UI_FONT
        }))
        .on_press(Message::LayerRules(LayerRulesMessage::SetFilter(filter)))
        .padding([6, 14])
        .style(move |_: &iced::Theme, _| {
            let (bg, tc) = if is_active {
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
                text_color: tc,
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
            tab("All", RulesFilter::All)
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

fn empty_state<'a>(search: &str, filter: RulesFilter) -> Element<'a, Message> {
    let has_filters = !search.is_empty() || filter != RulesFilter::All;
    let (title, message) = if has_filters {
        (
            "No Layer Rules Matching",
            "Try adjusting your search or filter.",
        )
    } else {
        (
            "No Layer Rules Yet",
            "Layer rules control behavior of panels, docks, and notification surfaces.",
        )
    };
    container(
        column![
            container(text("⊡").size(32).color(neon::SECONDARY))
                .width(72)
                .height(72)
                .center(Length::Shrink)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        a: 0.12,
                        ..neon::SECONDARY
                    })),
                    border: iced::Border {
                        radius: 999.0.into(),
                        color: iced::Color {
                            a: 0.2,
                            ..neon::SECONDARY
                        },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().height(16),
            text(title).size(22).font(fonts::UI_FONT_SEMIBOLD),
            text(message).size(13).color(neon::ON_SURFACE_VARIANT),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(60)
    .center(Length::Fill)
    .into()
}

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

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

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
        text(label).size(14).font(fonts::UI_FONT_MEDIUM),
        text(description).size(11).color(neon::ON_SURFACE_VARIANT),
        iced::widget::pick_list(
            options.clone(),
            Some(options[selected_idx]),
            move |selected| {
                let value = match selected {
                    "Screencast only" => Some(BlockOutFrom::Screencast),
                    "All screen captures" => Some(BlockOutFrom::ScreenCapture),
                    _ => None,
                };
                on_change(value)
            }
        )
        .padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

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
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text_input("", &display_owned)
                    .on_input(move |s| on_text(s).unwrap_or(Message::NoOp))
                    .padding([4, 8])
                    .size(11)
                    .width(Length::Fixed(55.0)),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4)
        .padding(12),
    )
    .style(crate::theme::card_style)
    .into()
}

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
                    .on_input(move |s| on_text(s).unwrap_or(Message::NoOp))
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
