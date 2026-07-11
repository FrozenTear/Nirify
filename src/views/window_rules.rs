//! Window rules settings view - card grid + modal editor
//!
//! Displays window rules as a card grid with matcher/effect pills.
//! Editing is done through a modal overlay.

use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{
    BackgroundEffectSettings, BlockOutFrom, CornerRadiusValue, DefaultColumnDisplay,
    FloatingPosition, PopupsSettings, PositionRelativeTo, RuleDefaultSize, ShadowSettings,
    WindowRule, WindowRulesSettings,
};
use crate::messages::{Message, RulesFilter, WindowRulesMessage};
use crate::theme::{fonts, neon};
use crate::types::{Color as NiriColor, ColorOrGradient};

/// Display wrapper for a tri-state (Default / Force on / Force off) picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriState {
    Default,
    On,
    Off,
}

impl TriState {
    const ALL: [TriState; 3] = [TriState::Default, TriState::On, TriState::Off];
    fn from_opt(v: Option<bool>) -> Self {
        match v {
            None => TriState::Default,
            Some(true) => TriState::On,
            Some(false) => TriState::Off,
        }
    }
    fn to_opt(self) -> Option<bool> {
        match self {
            TriState::Default => None,
            TriState::On => Some(true),
            TriState::Off => Some(false),
        }
    }
}

impl std::fmt::Display for TriState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriState::Default => write!(f, "Default"),
            TriState::On => write!(f, "Force on"),
            TriState::Off => write!(f, "Force off"),
        }
    }
}

/// Display wrapper for the block-out-from picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockOutChoice {
    None,
    Screencast,
    ScreenCapture,
}

impl BlockOutChoice {
    const ALL: [BlockOutChoice; 3] = [
        BlockOutChoice::None,
        BlockOutChoice::Screencast,
        BlockOutChoice::ScreenCapture,
    ];
    fn from_opt(v: Option<BlockOutFrom>) -> Self {
        match v {
            None => BlockOutChoice::None,
            Some(BlockOutFrom::Screencast) => BlockOutChoice::Screencast,
            Some(BlockOutFrom::ScreenCapture) => BlockOutChoice::ScreenCapture,
        }
    }
    fn to_opt(self) -> Option<BlockOutFrom> {
        match self {
            BlockOutChoice::None => None,
            BlockOutChoice::Screencast => Some(BlockOutFrom::Screencast),
            BlockOutChoice::ScreenCapture => Some(BlockOutFrom::ScreenCapture),
        }
    }
}

impl std::fmt::Display for BlockOutChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockOutChoice::None => write!(f, "Off"),
            BlockOutChoice::Screencast => write!(f, "Screencast"),
            BlockOutChoice::ScreenCapture => write!(f, "Screen capture"),
        }
    }
}

/// Display wrapper for the default-size mode picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeMode {
    Unset,
    Natural,
    Proportion,
    Fixed,
}

impl SizeMode {
    const ALL: [SizeMode; 4] = [
        SizeMode::Unset,
        SizeMode::Natural,
        SizeMode::Proportion,
        SizeMode::Fixed,
    ];
    fn of(v: &Option<RuleDefaultSize>) -> Self {
        match v {
            None => SizeMode::Unset,
            Some(RuleDefaultSize::Natural) => SizeMode::Natural,
            Some(RuleDefaultSize::Proportion(_)) => SizeMode::Proportion,
            Some(RuleDefaultSize::Fixed(_)) => SizeMode::Fixed,
        }
    }
}

impl std::fmt::Display for SizeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeMode::Unset => write!(f, "Unset"),
            SizeMode::Natural => write!(f, "Natural"),
            SizeMode::Proportion => write!(f, "Proportion"),
            SizeMode::Fixed => write!(f, "Fixed px"),
        }
    }
}

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
    if rule.open_floating == Some(true) {
        effect_pills.push(("◇ Always Float".to_string(), neon::TERTIARY));
    }
    if rule.open_maximized == Some(true) {
        effect_pills.push(("⊞ Maximize".to_string(), neon::TERTIARY));
    }
    if rule.open_fullscreen == Some(true) {
        effect_pills.push(("⊡ Fullscreen".to_string(), neon::TERTIARY));
    }
    if let Some(ref ws) = rule.open_on_workspace {
        effect_pills.push((format!("▤ WS {}", truncate_str(ws, 12)), neon::SECONDARY));
    }
    if let Some(opacity) = rule.opacity {
        effect_pills.push((format!("◉ Opacity {:.2}", opacity), neon::PRIMARY));
    }
    if rule.block_out_from.is_some() {
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
            .on_press(Message::ShowDialog(crate::messages::DialogState::Confirm {
                title: "Delete window rule?".to_string(),
                message: format!(
                    "Delete the rule \"{}\"? This cannot be undone.",
                    if rule.name.is_empty() {
                        "Untitled rule"
                    } else {
                        rule.name.as_str()
                    }
                ),
                confirm_label: "Delete".to_string(),
                on_confirm: crate::messages::ConfirmAction::DeleteWindowRule(id),
            }))
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
    supports_background_effects: bool,
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
    editor =
        editor.push(
            row![
                column![
                    modal_section_header("⚙", "OPENING BEHAVIOR", neon::TERTIARY),
                    container(
                        column![
                            picker_row(
                                "Maximized",
                                "Open maximized within the column",
                                &TriState::ALL,
                                Some(TriState::from_opt(rule.open_maximized)),
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetOpenMaximized(id, value.to_opt())
                                ),
                            ),
                            picker_row(
                                "Fullscreen",
                                "Open fullscreen",
                                &TriState::ALL,
                                Some(TriState::from_opt(rule.open_fullscreen)),
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetOpenFullscreen(id, value.to_opt())
                                ),
                            ),
                            picker_row(
                                "Floating",
                                "Open as a floating window",
                                &TriState::ALL,
                                Some(TriState::from_opt(rule.open_floating)),
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetOpenFloating(id, value.to_opt())
                                ),
                            ),
                            picker_row(
                                "Maximize to edges",
                                "Maximize to screen edges (v25.11+)",
                                &TriState::ALL,
                                Some(TriState::from_opt(rule.open_maximized_to_edges)),
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetOpenMaximizedToEdges(id, value.to_opt())
                                ),
                            ),
                            optional_bool_picker(
                                "Open focused",
                                "Focus window when it opens",
                                rule.open_focused,
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetOpenFocused(id, value)
                                ),
                            ),
                            picker_row(
                                "Block out from",
                                "Hide in screencasts / captures",
                                &BlockOutChoice::ALL,
                                Some(BlockOutChoice::from_opt(rule.block_out_from)),
                                move |value| Message::WindowRules(
                                    WindowRulesMessage::SetBlockOutFrom(id, value.to_opt())
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
                default_size_editor("COLUMN WIDTH", id, &rule.default_column_width, |id, v| {
                    Message::WindowRules(WindowRulesMessage::SetDefaultColumnWidth(id, v))
                }),
                default_size_editor("WINDOW HEIGHT", id, &rule.default_window_height, |id, v| {
                    Message::WindowRules(WindowRulesMessage::SetDefaultWindowHeight(id, v))
                }),
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
                    corner_radius_editor("CORNER RADIUS", id, &rule.corner_radius, |id, v| {
                        Message::WindowRules(WindowRulesMessage::SetCornerRadius(id, v))
                    }),
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
    editor = editor.push(
        row![
            column![
                modal_section_header("⬡", "ADVANCED", neon::OUTLINE),
                container(
                    column![
                        optional_bool_picker(
                            "Variable refresh rate",
                            "Enable VRR/FreeSync",
                            rule.variable_refresh_rate,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetVariableRefreshRate(id, value)
                            ),
                        ),
                        optional_bool_picker(
                            "Floating animation",
                            "baba-is-float effect",
                            rule.baba_is_float,
                            move |value| Message::WindowRules(WindowRulesMessage::SetBabaIsFloat(
                                id, value
                            )),
                        ),
                        optional_bool_picker(
                            "Tiled state",
                            "Mark as tiled (X11 compat)",
                            rule.tiled_state,
                            move |value| Message::WindowRules(WindowRulesMessage::SetTiledState(
                                id, value
                            )),
                        ),
                        picker_row(
                            "Column display",
                            "Default display mode",
                            &[DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
                            rule.default_column_display,
                            move |value| Message::WindowRules(
                                WindowRulesMessage::SetDefaultColumnDisplay(id, Some(value))
                            ),
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                // Floating position
                modal_section_header("◇", "FLOATING POSITION", neon::TERTIARY),
                {
                    let pos = rule
                        .default_floating_position
                        .clone()
                        .unwrap_or(FloatingPosition {
                            x: 0,
                            y: 0,
                            relative_to: PositionRelativeTo::TopLeft,
                        });
                    container(
                        column![
                            row![
                                column![
                                    text("X")
                                        .size(10)
                                        .font(fonts::UI_FONT_SEMIBOLD)
                                        .color(neon::OUTLINE_VARIANT),
                                    text_input("0", &format!("{}", pos.x))
                                        .on_input(move |s| {
                                            if let Ok(x) = s.parse::<i32>() {
                                                let mut p = rule
                                                    .default_floating_position
                                                    .clone()
                                                    .unwrap_or(FloatingPosition {
                                                        x: 0,
                                                        y: 0,
                                                        relative_to: PositionRelativeTo::TopLeft,
                                                    });
                                                p.x = x;
                                                Message::WindowRules(
                                                    WindowRulesMessage::SetDefaultFloatingPosition(
                                                        id,
                                                        Some(p),
                                                    ),
                                                )
                                            } else {
                                                Message::NoOp
                                            }
                                        })
                                        .padding(8)
                                        .size(12),
                                ]
                                .spacing(4)
                                .width(Length::FillPortion(1)),
                                column![
                                    text("Y")
                                        .size(10)
                                        .font(fonts::UI_FONT_SEMIBOLD)
                                        .color(neon::OUTLINE_VARIANT),
                                    text_input("0", &format!("{}", pos.y))
                                        .on_input(move |s| {
                                            if let Ok(y) = s.parse::<i32>() {
                                                let mut p = rule
                                                    .default_floating_position
                                                    .clone()
                                                    .unwrap_or(FloatingPosition {
                                                        x: 0,
                                                        y: 0,
                                                        relative_to: PositionRelativeTo::TopLeft,
                                                    });
                                                p.y = y;
                                                Message::WindowRules(
                                                    WindowRulesMessage::SetDefaultFloatingPosition(
                                                        id,
                                                        Some(p),
                                                    ),
                                                )
                                            } else {
                                                Message::NoOp
                                            }
                                        })
                                        .padding(8)
                                        .size(12),
                                ]
                                .spacing(4)
                                .width(Length::FillPortion(1)),
                            ]
                            .spacing(8),
                            picker_row(
                                "Relative to",
                                "Anchor point",
                                PositionRelativeTo::all(),
                                Some(pos.relative_to),
                                move |value| {
                                    let mut p = rule.default_floating_position.clone().unwrap_or(
                                        FloatingPosition {
                                            x: 0,
                                            y: 0,
                                            relative_to: PositionRelativeTo::TopLeft,
                                        },
                                    );
                                    p.relative_to = value;
                                    Message::WindowRules(
                                        WindowRulesMessage::SetDefaultFloatingPosition(id, Some(p)),
                                    )
                                },
                            ),
                        ]
                        .spacing(8),
                    )
                    .padding(8)
                    .style(crate::theme::card_style)
                },
                Space::new().height(8),
                modal_section_header("◌", "SHADOW", neon::TERTIARY),
                shadow_editor(id, &rule.shadow, |id, v| Message::WindowRules(
                    WindowRulesMessage::SetShadow(id, v)
                ),),
            ]
            .spacing(8)
            .width(Length::FillPortion(1)),
            column![
                row![
                    text("CUSTOM KDL BLOCK")
                        .size(10)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::OUTLINE_VARIANT),
                    Space::new().width(Length::Fill),
                    text("Live Preview").size(10).color(neon::SECONDARY),
                ]
                .padding([10, 0]),
                container(
                    scrollable(
                        text(kdl_preview)
                            .size(12)
                            .font(fonts::MONO_FONT)
                            .color(neon::ON_SURFACE_VARIANT),
                    )
                    .height(Length::Fixed(160.0)),
                )
                .padding(16)
                .width(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(neon::SURFACE_LOW)),
                    border: iced::Border {
                        color: iced::Color {
                            a: 0.15,
                            ..neon::OUTLINE_VARIANT
                        },
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }),
            ]
            .spacing(0)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    );

    // ── TAB INDICATOR COLOURS ──
    editor = editor.push(Space::new().height(20));
    editor = editor.push(modal_section_header(
        "▤",
        "TAB INDICATOR COLOURS",
        neon::SECONDARY,
    ));
    {
        let ti = rule.tab_indicator.clone().unwrap_or_default();
        let ti_a = ti.clone();
        let ti_i = ti.clone();
        let ti_u = ti.clone();
        editor = editor.push(
            container(
                column![
                    color_picker_row(
                        "Active",
                        "Active tab colour",
                        &color_or_gradient_to_niri(ti.active.as_ref()),
                        move |hex| {
                            let mut new = ti_a.clone();
                            new.active = Some(ColorOrGradient::Color(hex_to_niri_color(&hex)));
                            Message::WindowRules(WindowRulesMessage::SetTabIndicator(id, Some(new)))
                        },
                    ),
                    color_picker_row(
                        "Inactive",
                        "Inactive tab colour",
                        &color_or_gradient_to_niri(ti.inactive.as_ref()),
                        move |hex| {
                            let mut new = ti_i.clone();
                            new.inactive = Some(ColorOrGradient::Color(hex_to_niri_color(&hex)));
                            Message::WindowRules(WindowRulesMessage::SetTabIndicator(id, Some(new)))
                        },
                    ),
                    color_picker_row(
                        "Urgent",
                        "Urgent tab colour",
                        &color_or_gradient_to_niri(ti.urgent.as_ref()),
                        move |hex| {
                            let mut new = ti_u.clone();
                            new.urgent = Some(ColorOrGradient::Color(hex_to_niri_color(&hex)));
                            Message::WindowRules(WindowRulesMessage::SetTabIndicator(id, Some(new)))
                        },
                    ),
                ]
                .spacing(0),
            )
            .padding(8)
            .style(crate::theme::card_style),
        );
    }

    // ── BACKGROUND EFFECT & POPUPS (niri 26.04) ──
    editor = editor.push(Space::new().height(20));
    editor = editor.push(modal_section_header(
        "✦",
        "BACKGROUND EFFECT & POPUPS",
        neon::TERTIARY,
    ));
    editor = editor.push(background_effect_editor(
        id,
        &rule.background_effect,
        &rule.popups,
        supports_background_effects,
        |id, v| Message::WindowRules(WindowRulesMessage::SetBackgroundEffect(id, v)),
        |id, v| Message::WindowRules(WindowRulesMessage::SetPopups(id, v)),
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
    lines.push("window-rule {".to_string());

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

    if let Some(v) = rule.open_maximized {
        lines.push(format!("    open-maximized {}", v));
    }
    if let Some(v) = rule.open_fullscreen {
        lines.push(format!("    open-fullscreen {}", v));
    }
    if let Some(v) = rule.open_floating {
        lines.push(format!("    open-floating {}", v));
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
    if let Some(bof) = rule.block_out_from {
        let s = match bof {
            BlockOutFrom::Screencast => "screencast",
            BlockOutFrom::ScreenCapture => "screen-capture",
        };
        lines.push(format!("    block-out-from \"{}\"", s));
    }
    if let Some(ref width) = rule.default_column_width {
        lines.push(format!("    default-column-width {}", size_preview(width)));
    }
    if let Some(ref height) = rule.default_window_height {
        lines.push(format!(
            "    default-window-height {}",
            size_preview(height)
        ));
    }
    if let Some(opacity) = rule.opacity {
        lines.push(format!("    opacity {:.2}", opacity));
    }
    if let Some(ref cr) = rule.corner_radius {
        lines.push(format!("    geometry-corner-radius {}", corner_preview(cr)));
    }
    if let Some(clip) = rule.clip_to_geometry {
        lines.push(format!("    clip-to-geometry {}", clip));
    }
    // One focus-ring block combining presence + width, mirroring the generator.
    {
        let mut parts = Vec::new();
        if let Some(focus_ring) = rule.focus_ring_enabled {
            parts.push(if focus_ring {
                "on".to_string()
            } else {
                "off".to_string()
            });
        }
        if let Some(fw) = rule.focus_ring_width {
            parts.push(format!("width {}", fw));
        }
        if !parts.is_empty() {
            lines.push(format!("    focus-ring {{ {}; }}", parts.join("; ")));
        }
    }
    // One border block combining presence + width, mirroring the generator.
    {
        let mut parts = Vec::new();
        if let Some(border) = rule.border_enabled {
            parts.push(if border {
                "on".to_string()
            } else {
                "off".to_string()
            });
        }
        if let Some(bw) = rule.border_width {
            parts.push(format!("width {}", bw));
        }
        if !parts.is_empty() {
            lines.push(format!("    border {{ {}; }}", parts.join("; ")));
        }
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
    if let Some(ref ti) = rule.tab_indicator {
        if !ti.is_empty() {
            lines.push("    tab-indicator {".to_string());
            if ti.active.is_some() {
                lines.push("        active-color \"…\"".to_string());
            }
            if ti.inactive.is_some() {
                lines.push("        inactive-color \"…\"".to_string());
            }
            if ti.urgent.is_some() {
                lines.push("        urgent-color \"…\"".to_string());
            }
            lines.push("    }".to_string());
        }
    }
    if let Some(ref sh) = rule.shadow {
        if sh.enabled {
            lines.push("    shadow { on … }".to_string());
        } else {
            lines.push("    shadow { off }".to_string());
        }
    }
    if let Some(ref be) = rule.background_effect {
        if !be.is_empty() {
            lines.push("    background-effect { … }".to_string());
        }
    }
    if let Some(ref p) = rule.popups {
        if !p.is_empty() {
            lines.push("    popups { … }".to_string());
        }
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
        Some(ColorOrGradient::Color(c)) => *c,
        Some(ColorOrGradient::Gradient(g)) => g.from,
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

/// Preview string for a RuleDefaultSize.
fn size_preview(size: &RuleDefaultSize) -> String {
    match size {
        RuleDefaultSize::Natural => "{}".to_string(),
        RuleDefaultSize::Proportion(p) => format!("{{ proportion {}; }}", p),
        RuleDefaultSize::Fixed(n) => format!("{{ fixed {}; }}", n),
    }
}

/// Preview string for a CornerRadiusValue.
fn corner_preview(cr: &CornerRadiusValue) -> String {
    if cr.is_uniform() {
        format!("{}", cr.top_left)
    } else {
        format!(
            "{} {} {} {}",
            cr.top_left, cr.top_right, cr.bottom_right, cr.bottom_left
        )
    }
}

/// Editor card for a `RuleDefaultSize` (Unset / Natural / Proportion / Fixed px).
fn default_size_editor<'a>(
    label: &'a str,
    id: u32,
    value: &Option<RuleDefaultSize>,
    on_change: impl Fn(u32, Option<RuleDefaultSize>) -> Message + Copy + 'a,
) -> Element<'a, Message> {
    let mode = SizeMode::of(value);
    let mut col = column![picker_row(
        label,
        "Default size mode",
        &SizeMode::ALL,
        Some(mode),
        move |m| {
            let new = match m {
                SizeMode::Unset => None,
                SizeMode::Natural => Some(RuleDefaultSize::Natural),
                SizeMode::Proportion => Some(RuleDefaultSize::Proportion(0.5)),
                SizeMode::Fixed => Some(RuleDefaultSize::Fixed(800)),
            };
            on_change(id, new)
        },
    )]
    .spacing(4);

    match value {
        Some(RuleDefaultSize::Proportion(p)) => {
            let p = *p;
            col = col.push(styled_slider(
                "PROPORTION",
                &format!("{:.2}", p),
                move |s| {
                    s.parse::<f32>().ok().map(|v| {
                        on_change(id, Some(RuleDefaultSize::Proportion(v.clamp(0.1, 1.0))))
                    })
                },
                0.1..=1.0,
                p,
                0.01,
                move |v| on_change(id, Some(RuleDefaultSize::Proportion(v))),
            ));
        }
        Some(RuleDefaultSize::Fixed(n)) => {
            let n = *n;
            col = col.push(styled_slider_int(
                "FIXED PX",
                &format!("{}", n),
                move |s| {
                    s.parse::<i32>()
                        .ok()
                        .map(|v| on_change(id, Some(RuleDefaultSize::Fixed(v.clamp(1, 10000)))))
                },
                1..=10000,
                n,
                move |v| on_change(id, Some(RuleDefaultSize::Fixed(v))),
            ));
        }
        _ => {}
    }
    container(col).padding(4).width(Length::Fill).into()
}

/// Editor card for a `CornerRadiusValue`: uniform slider + per-corner inputs.
pub(crate) fn corner_radius_editor<'a>(
    label: &'a str,
    id: u32,
    value: &Option<CornerRadiusValue>,
    on_change: impl Fn(u32, Option<CornerRadiusValue>) -> Message + Copy + 'a,
) -> Element<'a, Message> {
    let cr = value.unwrap_or_default();
    let uniform = cr.top_left;
    let corner_input =
        move |field_label: &'static str, cur: f32, set: fn(&mut CornerRadiusValue, f32)| {
            let base = cr;
            column![
                text(field_label)
                    .size(9)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                text_input("0", &format!("{}", cur))
                    .on_input(move |s| {
                        if let Ok(v) = s.parse::<f32>() {
                            let mut new = base;
                            set(&mut new, v.max(0.0));
                            on_change(id, Some(new))
                        } else {
                            Message::NoOp
                        }
                    })
                    .padding(6)
                    .size(11),
            ]
            .spacing(2)
            .width(Length::FillPortion(1))
        };
    container(
        column![
            styled_slider_int(
                label,
                &format!("{}px", uniform as i32),
                move |s| {
                    s.replace("px", "").parse::<i32>().ok().map(|v| {
                        on_change(id, Some(CornerRadiusValue::uniform(v.clamp(0, 64) as f32)))
                    })
                },
                0..=64,
                uniform as i32,
                move |v| on_change(id, Some(CornerRadiusValue::uniform(v as f32))),
            ),
            row![
                corner_input("TL", cr.top_left, |c, v| c.top_left = v),
                corner_input("TR", cr.top_right, |c, v| c.top_right = v),
                corner_input("BR", cr.bottom_right, |c, v| c.bottom_right = v),
                corner_input("BL", cr.bottom_left, |c, v| c.bottom_left = v),
            ]
            .spacing(4),
        ]
        .spacing(4),
    )
    .width(Length::FillPortion(1))
    .into()
}

/// Editor card for an optional `ShadowSettings` override (window & layer rules).
///
/// Tri-state presence: Default = no override (None), Force on = `Some { enabled:
/// true }`, Force off = `Some { enabled: false }` (emits `shadow { off }`).
/// When forced on, the softness/spread/offset/colour/draw-behind controls show.
pub(crate) fn shadow_editor<'a>(
    id: u32,
    shadow: &Option<ShadowSettings>,
    on_change: impl Fn(u32, Option<ShadowSettings>) -> Message + Copy + 'a,
) -> Element<'a, Message> {
    let state = match shadow {
        None => TriState::Default,
        Some(s) if s.enabled => TriState::On,
        Some(_) => TriState::Off,
    };
    let base = shadow.clone().unwrap_or_default();
    let base_p = base.clone();
    let mut col = column![picker_row(
        "Shadow",
        "Drop-shadow override for matching windows",
        &TriState::ALL,
        Some(state),
        move |v| {
            let new = match v {
                TriState::Default => None,
                TriState::On => {
                    let mut s = base_p.clone();
                    s.enabled = true;
                    Some(s)
                }
                TriState::Off => {
                    let mut s = base_p.clone();
                    s.enabled = false;
                    Some(s)
                }
            };
            on_change(id, new)
        },
    )]
    .spacing(0);

    if state == TriState::On {
        let int_slider = move |label: &'a str,
                               cur: i32,
                               range: std::ops::RangeInclusive<i32>,
                               set: fn(&mut ShadowSettings, i32),
                               base: ShadowSettings| {
            let b_t = base.clone();
            let b_s = base;
            styled_slider_int(
                label,
                &format!("{}", cur),
                move |txt| {
                    txt.parse::<i32>().ok().map(|v| {
                        let mut s = b_t.clone();
                        set(&mut s, v);
                        on_change(id, Some(s))
                    })
                },
                range,
                cur,
                move |v| {
                    let mut s = b_s.clone();
                    set(&mut s, v);
                    on_change(id, Some(s))
                },
            )
        };
        let s = base.clone();
        let b_col = base.clone();
        let b_ic = base.clone();
        let b_dbw = base.clone();
        col = col.push(
            container(
                column![
                    int_slider(
                        "SOFTNESS",
                        s.softness,
                        0..=100,
                        |s, v| s.softness = v.clamp(0, 1024),
                        base.clone()
                    ),
                    int_slider(
                        "SPREAD",
                        s.spread,
                        -64..=64,
                        |s, v| s.spread = v.clamp(-1024, 1024),
                        base.clone()
                    ),
                    row![
                        int_slider(
                            "OFFSET X",
                            s.offset_x,
                            -128..=128,
                            |s, v| s.offset_x = v,
                            base.clone()
                        ),
                        int_slider(
                            "OFFSET Y",
                            s.offset_y,
                            -128..=128,
                            |s, v| s.offset_y = v,
                            base.clone()
                        ),
                    ]
                    .spacing(8),
                    color_picker_row(
                        "Shadow colour",
                        "Active-window shadow",
                        &s.color,
                        move |hex| {
                            let mut ns = b_col.clone();
                            ns.color = hex_to_niri_color(&hex);
                            on_change(id, Some(ns))
                        }
                    ),
                    color_picker_row(
                        "Inactive colour",
                        "Inactive-window shadow",
                        &s.inactive_color,
                        move |hex| {
                            let mut ns = b_ic.clone();
                            ns.inactive_color = hex_to_niri_color(&hex);
                            on_change(id, Some(ns))
                        },
                    ),
                    toggle_row(
                        "Draw behind window",
                        "Render the shadow behind the window surface",
                        s.draw_behind_window,
                        move |on| {
                            let mut ns = b_dbw.clone();
                            ns.draw_behind_window = on;
                            on_change(id, Some(ns))
                        },
                    ),
                ]
                .spacing(4),
            )
            .padding(8)
            .style(crate::theme::card_style),
        );
    }

    container(col)
        .padding(8)
        .style(crate::theme::card_style)
        .into()
}

/// Render the xray / blur / noise / saturation controls for a
/// `BackgroundEffectSettings`. Shared between the top-level rule effect and the
/// nested popups effect.
pub(crate) fn bg_effect_controls<'a>(
    be: BackgroundEffectSettings,
    on_change: impl Fn(BackgroundEffectSettings) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let oc_x = on_change.clone();
    let oc_b = on_change.clone();
    let oc_nt = on_change.clone();
    let oc_ns = on_change.clone();
    let oc_st = on_change.clone();
    let oc_ss = on_change;
    column![
        picker_row(
            "Background xray",
            "See through to the wallpaper",
            &TriState::ALL,
            Some(TriState::from_opt(be.xray)),
            move |v| {
                let mut new = be;
                new.xray = v.to_opt();
                oc_x(new)
            },
        ),
        picker_row(
            "Background blur",
            "Blur behind the surface",
            &TriState::ALL,
            Some(TriState::from_opt(be.blur)),
            move |v| {
                let mut new = be;
                new.blur = v.to_opt();
                oc_b(new)
            },
        ),
        styled_slider(
            "NOISE",
            &be.noise.map(|n| format!("{:.2}", n)).unwrap_or_default(),
            move |s| {
                if s.trim().is_empty() {
                    let mut new = be;
                    new.noise = None;
                    Some(oc_nt(new))
                } else {
                    s.parse::<f32>().ok().map(|v| {
                        let mut new = be;
                        new.noise = Some(v.max(0.0));
                        oc_nt(new)
                    })
                }
            },
            0.0..=1.0,
            be.noise.unwrap_or(0.0),
            0.01,
            move |v| {
                let mut new = be;
                new.noise = Some(v);
                oc_ns(new)
            },
        ),
        styled_slider(
            "SATURATION",
            &be.saturation
                .map(|n| format!("{:.2}", n))
                .unwrap_or_default(),
            move |s| {
                if s.trim().is_empty() {
                    let mut new = be;
                    new.saturation = None;
                    Some(oc_st(new))
                } else {
                    s.parse::<f32>().ok().map(|v| {
                        let mut new = be;
                        new.saturation = Some(v.max(0.0));
                        oc_st(new)
                    })
                }
            },
            0.0..=2.0,
            be.saturation.unwrap_or(1.0),
            0.05,
            move |v| {
                let mut new = be;
                new.saturation = Some(v);
                oc_ss(new)
            },
        ),
    ]
    .spacing(0)
    .into()
}

/// Editor for the niri 26.04 background-effect + popups blocks (gated).
#[allow(clippy::too_many_arguments)]
pub(crate) fn background_effect_editor<'a>(
    id: u32,
    background_effect: &Option<BackgroundEffectSettings>,
    popups: &Option<PopupsSettings>,
    supported: bool,
    on_bg: impl Fn(u32, Option<BackgroundEffectSettings>) -> Message + Copy + 'a,
    on_popups: impl Fn(u32, Option<PopupsSettings>) -> Message + Copy + 'a,
) -> Element<'a, Message> {
    if !supported {
        return container(info_text("Requires niri 26.04"))
            .padding(8)
            .style(crate::theme::card_style)
            .into();
    }
    let be = (*background_effect).unwrap_or_default();
    let popups_present = popups.as_ref().map(|p| !p.is_empty()).unwrap_or(false);

    let mut col = column![
        bg_effect_controls(be, move |new| on_bg(
            id,
            if new.is_empty() { None } else { Some(new) }
        )),
        toggle_row(
            "Popup overrides",
            "Override opacity / corners / effects for popups",
            popups_present,
            move |on| {
                let new = if on {
                    Some(PopupsSettings {
                        opacity: Some(1.0),
                        ..Default::default()
                    })
                } else {
                    None
                };
                on_popups(id, new)
            },
        ),
    ]
    .spacing(0);

    if popups_present {
        let p = popups.clone().unwrap_or_default();
        let p_op = p.clone();
        let p_op2 = p.clone();
        let p_cr = p.clone();
        let p_cr2 = p.clone();
        let p_be = p.clone();
        let cur_op = p.opacity.unwrap_or(1.0);
        let cur_cr = p
            .geometry_corner_radius
            .map(|c| c.top_left as i32)
            .unwrap_or(0);
        col = col.push(
            container(
                column![
                    styled_slider(
                        "POPUP OPACITY",
                        &format!("{:.2}", cur_op),
                        move |s| s.parse::<f32>().ok().map(|v| {
                            let mut np = p_op.clone();
                            np.opacity = Some(v.clamp(0.0, 1.0));
                            on_popups(id, Some(np))
                        }),
                        0.0..=1.0,
                        cur_op,
                        0.01,
                        move |v| {
                            let mut np = p_op2.clone();
                            np.opacity = Some(v);
                            on_popups(id, Some(np))
                        },
                    ),
                    styled_slider_int(
                        "POPUP CORNER RADIUS",
                        &format!("{}px", cur_cr),
                        move |s| {
                            s.replace("px", "").parse::<i32>().ok().map(|v| {
                                let mut np = p_cr.clone();
                                np.geometry_corner_radius =
                                    Some(CornerRadiusValue::uniform(v.clamp(0, 64) as f32));
                                on_popups(id, Some(np))
                            })
                        },
                        0..=64,
                        cur_cr,
                        move |v| {
                            let mut np = p_cr2.clone();
                            np.geometry_corner_radius = Some(CornerRadiusValue::uniform(v as f32));
                            on_popups(id, Some(np))
                        },
                    ),
                    bg_effect_controls(p.background_effect.unwrap_or_default(), move |new| {
                        let mut np = p_be.clone();
                        np.background_effect = if new.is_empty() { None } else { Some(new) };
                        on_popups(id, Some(np))
                    }),
                ]
                .spacing(4),
            )
            .padding(8)
            .style(crate::theme::card_style),
        );
    }

    container(col)
        .padding(8)
        .style(crate::theme::card_style)
        .into()
}
