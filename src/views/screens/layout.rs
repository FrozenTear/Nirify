//! Layout screen — "Infinite Ribbon" with summary cards + modal editors

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::{
    AppearanceSettings, BehaviorSettings, LayoutExtrasSettings, WorkspacesSettings,
};
use crate::messages::{EditableSection, Message};
use crate::theme::{fonts, neon};

use neon::{
    OUTLINE_VARIANT, PRIMARY, SECONDARY, SURFACE_CONTAINER, SURFACE_CONTAINER_HIGHEST, SURFACE_LOW,
};

/// Layout screen
pub fn view<'a>(
    layout_extras: &'a LayoutExtrasSettings,
    workspaces: &'a WorkspacesSettings,
    behavior: &'a BehaviorSettings,
    appearance: &'a AppearanceSettings,
) -> Element<'a, Message> {
    let content = column![
        // ── Hero header ─────────────────────────────────────────────
        super::hero_header(
            "WORKSPACE ENGINE",
            "Infinite Ribbon",
            "Spatial layout defines how windows occupy your virtual canvas. The ribbon extends infinitely across the horizontal axis, prioritizing focus through precision column management.",
            SECONDARY,
        ),
        Space::new().height(16),
        // ── Ribbon visualizer ───────────────────────────────────────
        ribbon_preview(),
        Space::new().height(24),
        // ── Summary Cards Row 1 ─────────────────────────────────────
        row![
            summary_card(EditableSection::SpatialGaps, vec![
                ("Gaps", format!("{:.0}px", appearance.gaps)),
                ("Radius", format!("{:.0}px", appearance.corner_radius)),
            ]),
            summary_card(EditableSection::CenteringDynamics, vec![
                ("Focus Mouse", if behavior.focus_follows_mouse { "On" } else { "Off" }.to_string()),
                ("Center Col", format!("{}", behavior.center_focused_column)),
            ]),
            summary_card(EditableSection::ColumnManager, vec![
                ("Width", format!("{}", behavior.default_column_width_type)),
                ("Display", format!("{}", layout_extras.default_column_display)),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        // ── Summary Cards Row 2 ─────────────────────────────────────
        row![
            summary_card(EditableSection::ScreenEdgeStruts, vec![
                ("L/R", format!("{:.0}/{:.0}", behavior.strut_left, behavior.strut_right)),
                ("T/B", format!("{:.0}/{:.0}", behavior.strut_top, behavior.strut_bottom)),
            ]),
            summary_card(EditableSection::TabIndicator, vec![
                ("Enabled", if layout_extras.tab_indicator.enabled { "On" } else { "Off" }.to_string()),
                ("Position", format!("{}", layout_extras.tab_indicator.position)),
            ]),
            summary_card(EditableSection::InsertHint, vec![
                ("Enabled", if layout_extras.insert_hint.enabled { "On" } else { "Off" }.to_string()),
                ("Color", layout_extras.insert_hint.color.to_hex()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        // ── Row 3 ───────────────────────────────────────────────────
        row![
            summary_card(EditableSection::NamedWorkspaces, vec![
                ("Count", format!("{}", workspaces.workspaces.len())),
                ("Status", if workspaces.workspaces.is_empty() { "None" } else { "Active" }.to_string()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
    ]
    .spacing(0)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

// ── Summary Card ───────────────────────────────────────────────────────────

fn summary_card<'a>(
    section: EditableSection,
    summary: Vec<(&'static str, String)>,
) -> Element<'a, Message> {
    let accent = section.accent();
    let icon = section.icon();
    let name = section.name();

    let mut summary_items = column![].spacing(4);
    for (label, value) in summary {
        summary_items = summary_items.push(
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(value).size(11).font(fonts::MONO_FONT).color(accent),
            ]
            .align_y(Alignment::Center),
        );
    }

    let card = column![
        row![
            container(text(icon).size(16).color(accent))
                .width(36)
                .height(36)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.12, ..accent })),
                    border: iced::Border {
                        radius: 10.0.into(),
                        color: iced::Color { a: 0.2, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(10),
            text(name).size(14).font(fonts::UI_FONT_SEMIBOLD),
        ]
        .align_y(Alignment::Center),
        Space::new().height(10),
        summary_items,
        Space::new().height(10),
        iced::widget::button(
            text("CONFIGURE")
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(accent),
        )
        .on_press(Message::OpenSectionEditor(section))
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => iced::Color { a: 0.15, ..accent },
                _ => iced::Color { a: 0.08, ..accent },
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: accent,
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color { a: 0.2, ..accent },
                    width: 1.0,
                },
                ..Default::default()
            }
        }),
    ];

    container(card)
        .padding(16)
        .width(Length::FillPortion(1))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(SURFACE_CONTAINER)),
            border: iced::Border {
                color: iced::Color { a: 0.12, ..accent },
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.08, ..accent },
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 20.0,
            },
            ..Default::default()
        })
        .into()
}

// ── Ribbon Preview ──────────────────────────────────────────────────────────

fn ribbon_preview<'a>() -> Element<'a, Message> {
    let inactive_win = || {
        container(
            column![
                container(Space::new().width(40).height(3)).style(|_: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Background::Color(OUTLINE_VARIANT)),
                        border: iced::Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
                Space::new().height(Length::Fill),
            ]
            .spacing(8)
            .padding(12),
        )
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(SURFACE_CONTAINER)),
            border: iced::Border {
                color: OUTLINE_VARIANT,
                width: 0.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        })
    };

    let focused_window = container(
        column![
            row![
                container(Space::new().width(8).height(8)).style(|_: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Background::Color(neon::ERROR)),
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
                container(Space::new().width(8).height(8)).style(|_: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Background::Color(neon::TERTIARY)),
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
                container(Space::new().width(8).height(8)).style(|_: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Background::Color(SECONDARY)),
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
                Space::new().width(Length::Fill),
                container(
                    text("FOCUSED_COLUMN")
                        .size(9)
                        .font(fonts::MONO_FONT)
                        .color(PRIMARY),
                )
                .padding([4, 8])
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.10, ..PRIMARY })),
                    border: iced::Border {
                        color: iced::Color { a: 0.20, ..PRIMARY },
                        width: 1.0,
                        radius: 999.0.into()
                    },
                    ..Default::default()
                }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            Space::new().height(8),
            container(Space::new().width(Length::Fill).height(6)).style(|_: &iced::Theme| {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.10, ..PRIMARY })),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
            container(Space::new().width(Length::Fixed(180.0)).height(6)).style(
                |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.06, ..PRIMARY })),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            ),
            Space::new().height(4),
            container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.04, ..PRIMARY })),
                    border: iced::Border {
                        color: iced::Color { a: 0.10, ..PRIMARY },
                        width: 1.0,
                        radius: 12.0.into()
                    },
                    ..Default::default()
                }
            ),
        ]
        .spacing(4)
        .padding(16),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_CONTAINER_HIGHEST)),
        border: iced::Border {
            color: iced::Color { a: 0.4, ..PRIMARY },
            width: 2.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color { a: 0.15, ..PRIMARY },
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 40.0,
        },
        ..Default::default()
    });

    container(
        row![inactive_win(), focused_window, inactive_win()]
            .spacing(12)
            .padding(24)
            .height(Length::Fixed(220.0)),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_LOW)),
        border: iced::Border {
            radius: 24.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
