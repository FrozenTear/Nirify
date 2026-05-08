//! Recent windows settings view — neon modal style

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space,
};
use iced::{Alignment, Element, Length};

use crate::config::models::{RecentWindowsScope, RecentWindowsSettings};
use crate::messages::{Message, RecentWindowsMessage};
use crate::theme::{fonts, neon};

pub fn view(settings: &RecentWindowsSettings) -> Element<'static, Message> {
    let off = settings.off;
    let debounce_ms = settings.debounce_ms;
    let open_delay_ms = settings.open_delay_ms;
    let active_color = settings.highlight.active_color.to_hex();
    let urgent_color = settings.highlight.urgent_color.to_hex();
    let padding = settings.highlight.padding;
    let corner_radius = settings.highlight.corner_radius;
    let max_height = settings.previews.max_height;
    let max_scale = settings.previews.max_scale;
    let binds = settings.binds.clone();

    let mut content = column![
        // ── 2-COLUMN: STATUS + TIMING | HIGHLIGHT + PREVIEW ──
        row![
            // Left: Status + Timing
            column![
                modal_section("⏻", "STATUS", neon::SECONDARY),
                container(column![
                    toggle_row("Disable Switcher", "Completely disable the window switcher",
                        off, |v| Message::RecentWindows(RecentWindowsMessage::SetOff(v))),
                ].spacing(0)).padding(8).style(crate::theme::card_style),
                Space::new().height(12),
                modal_section("⏱", "TIMING", neon::PRIMARY),
                Space::new().height(4),
                styled_slider_int("DEBOUNCE DELAY", &format!("{}ms", debounce_ms),
                    0..=2000, debounce_ms,
                    |v| Message::RecentWindows(RecentWindowsMessage::SetDebounceMs(v))),
                styled_slider_int("OPEN DELAY", &format!("{}ms", open_delay_ms),
                    0..=2000, open_delay_ms,
                    |v| Message::RecentWindows(RecentWindowsMessage::SetOpenDelayMs(v))),
            ].spacing(4).width(Length::FillPortion(1)),

            // Right: Highlight + Preview
            column![
                modal_section("◉", "HIGHLIGHT STYLE", neon::TERTIARY),
                Space::new().height(4),
                color_input("ACTIVE COLOR", &active_color,
                    |s| Message::RecentWindows(RecentWindowsMessage::SetActiveColor(s))),
                color_input("URGENT COLOR", &urgent_color,
                    |s| Message::RecentWindows(RecentWindowsMessage::SetUrgentColor(s))),
                row![
                    styled_slider_int("PADDING", &format!("{}px", padding),
                        0..=50, padding,
                        |v| Message::RecentWindows(RecentWindowsMessage::SetHighlightPadding(v))),
                    styled_slider_int("CORNER RADIUS", &format!("{}px", corner_radius),
                        0..=50, corner_radius,
                        |v| Message::RecentWindows(RecentWindowsMessage::SetHighlightCornerRadius(v))),
                ].spacing(8),
                Space::new().height(12),
                modal_section("▭", "PREVIEW", neon::SECONDARY),
                Space::new().height(4),
                row![
                    styled_slider_int("MAX HEIGHT", &format!("{}px", max_height),
                        50..=500, max_height,
                        |v| Message::RecentWindows(RecentWindowsMessage::SetPreviewMaxHeight(v))),
                    {
                        let pct = (max_scale * 100.0) as i32;
                        styled_slider_int("MAX SCALE", &format!("{}%", pct),
                            10..=100, pct,
                            |v| Message::RecentWindows(RecentWindowsMessage::SetPreviewMaxScale(v as f64 / 100.0)))
                    },
                ].spacing(8),
            ].spacing(4).width(Length::FillPortion(1)),
        ].spacing(32).align_y(Alignment::Start),

        Space::new().height(20),

        // ── CUSTOM KEYBINDINGS ──
        modal_section("⌨", "CUSTOM KEYBINDINGS", neon::PRIMARY),
        super::widgets::info_text("Add custom keybindings for the window switcher. Leave empty to use defaults (Alt+Tab)."),
    ]
    .spacing(4);

    // Keybindings list
    if binds.is_empty() {
        content = content.push(
            text("No custom keybindings — using defaults")
                .size(12)
                .color(neon::ON_SURFACE_VARIANT),
        );
    } else {
        for (idx, bind) in binds.iter().enumerate() {
            let key_combo = bind.key_combo.clone();
            let is_next = bind.is_next;
            let filter_app = bind.filter_app_id;
            let scope = bind.scope;
            let cooldown = bind.cooldown_ms;

            content = content.push(
                container(
                    row![
                        // Left: key combo + direction
                        column![
                            text(format!("Binding #{}", idx + 1))
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().height(4),
                            styled_text_input("KEY COMBO", "e.g., Alt+Tab", &key_combo, move |s| {
                                Message::RecentWindows(RecentWindowsMessage::SetBindKeyCombo(
                                    idx, s,
                                ))
                            }),
                            row![pick_list(
                                vec!["Next Window", "Previous Window"],
                                Some(if is_next {
                                    "Next Window"
                                } else {
                                    "Previous Window"
                                }),
                                move |s| Message::RecentWindows(
                                    RecentWindowsMessage::SetBindIsNext(idx, s == "Next Window")
                                ),
                            )
                            .width(Length::Fill)
                            .padding(8),],
                        ]
                        .spacing(4)
                        .width(Length::FillPortion(1)),
                        // Right: scope + filter + cooldown + delete
                        column![
                            row![
                                text("Filter to app")
                                    .size(12)
                                    .color(neon::ON_SURFACE_VARIANT),
                                Space::new().width(Length::Fill),
                                toggler(filter_app).on_toggle(move |v| Message::RecentWindows(
                                    RecentWindowsMessage::SetBindFilterAppId(idx, v)
                                )),
                            ]
                            .align_y(Alignment::Center)
                            .padding([4, 0]),
                            pick_list(
                                vec![
                                    ScopeOption::All,
                                    ScopeOption::Output,
                                    ScopeOption::Workspace
                                ],
                                Some(scope_to_option(&scope)),
                                move |opt| Message::RecentWindows(
                                    RecentWindowsMessage::SetBindScope(idx, option_to_scope(opt))
                                ),
                            )
                            .width(Length::Fill)
                            .padding(8),
                            row![
                                text_input(
                                    "Cooldown (ms)",
                                    &cooldown.map(|c| c.to_string()).unwrap_or_default()
                                )
                                .on_input(move |s| {
                                    let cd = if s.is_empty() { None } else { s.parse().ok() };
                                    Message::RecentWindows(RecentWindowsMessage::SetBindCooldown(
                                        idx, cd,
                                    ))
                                })
                                .padding(8)
                                .size(12)
                                .width(Length::Fill),
                                button(text("✕").size(12).color(neon::ERROR))
                                    .on_press(Message::RecentWindows(
                                        RecentWindowsMessage::RemoveBind(idx)
                                    ))
                                    .padding([6, 10])
                                    .style(ghost_btn_style),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(4)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(16)
                    .align_y(Alignment::Start),
                )
                .padding(12)
                .style(crate::theme::card_style),
            );
            content = content.push(Space::new().height(4));
        }
    }

    content = content.push(Space::new().height(8));
    content = content.push(
        button(
            row![
                text("+").size(14),
                text("Add Keybinding").size(12).font(fonts::UI_FONT_MEDIUM)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::RecentWindows(RecentWindowsMessage::AddBind))
        .padding([8, 16])
        .style(|_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => iced::Color {
                    a: 0.15,
                    ..neon::PRIMARY
                },
                _ => iced::Color {
                    a: 0.08,
                    ..neon::PRIMARY
                },
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: neon::PRIMARY,
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color {
                        a: 0.2,
                        ..neon::PRIMARY
                    },
                    width: 1.0,
                },
                ..Default::default()
            }
        }),
    );

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

// ── Scope helpers ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeOption {
    All,
    Output,
    Workspace,
}

impl std::fmt::Display for ScopeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All Windows"),
            Self::Output => write!(f, "Current Output"),
            Self::Workspace => write!(f, "Current Workspace"),
        }
    }
}

fn scope_to_option(scope: &Option<RecentWindowsScope>) -> ScopeOption {
    match scope {
        None | Some(RecentWindowsScope::All) => ScopeOption::All,
        Some(RecentWindowsScope::Output) => ScopeOption::Output,
        Some(RecentWindowsScope::Workspace) => ScopeOption::Workspace,
    }
}

fn option_to_scope(opt: ScopeOption) -> Option<RecentWindowsScope> {
    match opt {
        ScopeOption::All => None,
        ScopeOption::Output => Some(RecentWindowsScope::Output),
        ScopeOption::Workspace => Some(RecentWindowsScope::Workspace),
    }
}

// ── Helpers ──

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

fn toggle_row<'a>(
    label: &'a str,
    desc: &'a str,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        column![
            text(label).size(14).font(fonts::UI_FONT_MEDIUM),
            text(desc).size(11).color(neon::ON_SURFACE_VARIANT),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value).on_toggle(on_toggle),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}

fn styled_slider_int<'a>(
    label: &'a str,
    display: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(d)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide).width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let v = value.to_string();
    column![
        text(label)
            .size(10)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::OUTLINE_VARIANT),
        text_input(placeholder, &v)
            .on_input(on_change)
            .padding(8)
            .size(12)
            .font(fonts::MONO_FONT),
    ]
    .spacing(4)
    .into()
}

fn color_input<'a>(
    label: &'a str,
    hex: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let hex_owned = hex.to_string();
    let parsed = crate::types::Color::from_hex(&hex_owned)
        .map(|c| {
            iced::Color::from_rgba(
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            )
        })
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    container(
        row![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT)
                .width(Length::Fill),
            container(Space::new().width(20).height(20)).style(move |_: &iced::Theme| {
                container::Style {
                    background: Some(iced::Background::Color(parsed)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        color: iced::Color {
                            a: 0.3,
                            ..neon::OUTLINE_VARIANT
                        },
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
            Space::new().width(8),
            text_input("#RRGGBB", &hex_owned)
                .on_input(on_change)
                .padding(6)
                .size(12)
                .font(fonts::MONO_FONT)
                .width(Length::Fixed(100.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn ghost_btn_style(
    _: &iced::Theme,
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
