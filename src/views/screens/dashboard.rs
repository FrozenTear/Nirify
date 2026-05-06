//! Dashboard screen — compositor status, workspace overview, session actions
//!
//! Designed to match the Tokyo Neon "Command Center" aesthetic.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::config::Settings;
use crate::ipc::WorkspaceInfo;
use crate::messages::Message;
use crate::theme::{fonts, neon, PillVariant};
use crate::version::NiriVersion;
use crate::views::status_bar::NiriStatus;
use crate::views::tools::ToolsState;

use neon::{
    ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, SECONDARY, SURFACE_CONTAINER,
    SURFACE_CONTAINER_HIGH, SURFACE_CONTAINER_HIGHEST,
};

/// Dashboard screen
pub fn view<'a>(
    niri_status: NiriStatus,
    niri_version: Option<NiriVersion>,
    tools_state: &'a ToolsState,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let content = column![
        // ── Hero: Compositor Status ─────────────────────────────────────
        compositor_status_card(niri_status, niri_version),
        Space::new().height(8),
        // ── Main grid: two columns ──────────────────────────────────────
        row![
            // Left column
            column![
                label_text("ACTIVE FEATURES"),
                feature_toggle_card("Window Gaps", settings.appearance.gaps > 0.0),
                feature_toggle_card("Animations", settings.animations.enabled),
                feature_toggle_card("Focus Ring", settings.appearance.focus_ring_enabled),
                Space::new().height(8),
                label_text("SESSION ACTIONS"),
                session_action_card(
                    "Reload Config",
                    Message::Tools(crate::messages::ToolsMessage::ReloadConfig),
                ),
            ]
            .spacing(8)
            .width(Length::FillPortion(4)),
            Space::new().width(16),
            // Right column
            column![
                workspace_status_section(&tools_state.workspaces),
                Space::new().height(16),
                column_preview_placeholder(),
            ]
            .spacing(8)
            .width(Length::FillPortion(8)),
        ],
        Space::new().height(16),
        // ── Bottom stats row ────────────────────────────────────────────
        stats_row(tools_state.windows.len(), tools_state.workspaces.len(),),
    ]
    .spacing(8)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

// ── Compositor Status Card ──────────────────────────────────────────────────

fn compositor_status_card<'a>(
    status: NiriStatus,
    version: Option<NiriVersion>,
) -> Element<'a, Message> {
    let (pill_label, pill_variant) = match status {
        NiriStatus::Connected => ("RUNNING", PillVariant::Active),
        NiriStatus::Disconnected => ("DISCONNECTED", PillVariant::Error),
        NiriStatus::Unknown => ("CHECKING", PillVariant::Muted),
    };

    let version_text = version
        .map(|v| format!("niri v{}.{:02}", v.major, v.minor))
        .unwrap_or_else(|| "niri".to_string());

    let status_headline = match status {
        NiriStatus::Connected => "Compositor Running",
        NiriStatus::Disconnected => "Compositor Offline",
        NiriStatus::Unknown => "Checking Status...",
    };

    let config_path = "~/.config/niri/config.kdl";

    neon_card(
        column![row![
            column![
                text(status_headline).size(28).font(fonts::UI_FONT_SEMIBOLD),
                row![
                    text(version_text)
                        .size(13)
                        .font(fonts::MONO_FONT)
                        .color(ON_SURFACE_VARIANT),
                    text(" | ").size(13).color(OUTLINE_VARIANT),
                    text(config_path)
                        .size(13)
                        .font(fonts::MONO_FONT)
                        .color(OUTLINE_VARIANT),
                ]
                .spacing(4),
            ]
            .spacing(6)
            .width(Length::Fill),
            status_pill(pill_label, pill_variant),
        ]
        .align_y(Alignment::Start),]
        .spacing(8),
    )
}

// ── Workspace Status ────────────────────────────────────────────────────────

fn workspace_status_section<'a>(workspaces: &'a [WorkspaceInfo]) -> Element<'a, Message> {
    let mut ws_row = row![].spacing(8);

    if workspaces.is_empty() {
        ws_row = ws_row.push(
            container(
                text("No workspaces detected")
                    .size(13)
                    .color(OUTLINE_VARIANT),
            )
            .padding(24)
            .center(Length::Fill),
        );
    } else {
        for ws in workspaces.iter().take(6) {
            ws_row = ws_row.push(workspace_card(ws));
        }
        if workspaces.len() > 6 {
            ws_row = ws_row.push(
                container(
                    text(format!("+{}", workspaces.len() - 6))
                        .size(14)
                        .color(OUTLINE_VARIANT),
                )
                .padding([16, 12])
                .center(Length::Fixed(60.0)),
            );
        }
    }

    column![
        row![
            label_text("WORKSPACE STATUS"),
            Space::new().width(Length::Fill),
            text(format!("{} Active", workspaces.len()))
                .size(11)
                .font(fonts::MONO_FONT)
                .color(SECONDARY),
        ]
        .align_y(Alignment::Center),
        scrollable(ws_row).direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default().width(0).scroller_width(0),
        )),
    ]
    .spacing(8)
    .into()
}

fn workspace_card<'a>(ws: &'a WorkspaceInfo) -> Element<'a, Message> {
    let default_name = format!("{:02}", ws.idx);
    let name = ws.name.as_deref().unwrap_or(&default_name);
    let border_color = if ws.is_active {
        PRIMARY
    } else {
        OUTLINE_VARIANT
    };

    container(
        column![
            text(format!("{:02}", ws.idx))
                .size(22)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(if ws.is_active {
                    PRIMARY
                } else {
                    ON_SURFACE_VARIANT
                }),
            text(name.to_uppercase())
                .size(9)
                .font(fonts::UI_FONT_MEDIUM)
                .color(OUTLINE_VARIANT),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .padding([16, 20])
    .width(Length::Fixed(90.0))
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_CONTAINER_HIGH)),
        border: iced::Border {
            color: border_color,
            width: if ws.is_active { 1.5 } else { 0.0 },
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Column Preview Placeholder ──────────────────────────────────────────────

fn column_preview_placeholder<'a>() -> Element<'a, Message> {
    container(
        column![
            label_text("DYNAMIC COLUMN PREVIEW"),
            Space::new().height(8),
            container(
                text("Canvas visualizer coming soon")
                    .size(13)
                    .color(OUTLINE_VARIANT),
            )
            .padding(40)
            .center(Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(SURFACE_CONTAINER_HIGHEST)),
                border: iced::Border {
                    color: OUTLINE_VARIANT,
                    width: 0.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(4),
    )
    .padding(20)
    .width(Length::Fill)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_CONTAINER)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 16.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Feature Toggle Cards ────────────────────────────────────────────────────

fn feature_toggle_card<'a>(name: &'a str, enabled: bool) -> Element<'a, Message> {
    let (pill_text, pill_color) = if enabled {
        ("ON", SECONDARY)
    } else {
        ("OFF", OUTLINE_VARIANT)
    };

    container(
        row![
            text(name)
                .size(14)
                .font(fonts::UI_FONT_MEDIUM)
                .width(Length::Fill),
            container(
                text(pill_text)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(pill_color),
            )
            .padding([4, 10])
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: if enabled { 0.15 } else { 0.08 },
                    ..pill_color
                })),
                border: iced::Border {
                    radius: 999.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .align_y(Alignment::Center),
    )
    .padding([14, 16])
    .width(Length::Fill)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_CONTAINER)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Session Action Card ─────────────────────────────────────────────────────

fn session_action_card<'a>(label: &'a str, on_press: Message) -> Element<'a, Message> {
    button(
        container(text(label).size(14).font(fonts::UI_FONT_MEDIUM))
            .padding([14, 20])
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .on_press(on_press)
    .width(Length::Fill)
    .style(|_theme: &iced::Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => SURFACE_CONTAINER_HIGHEST,
            button::Status::Pressed => SURFACE_CONTAINER_HIGH,
            _ => SURFACE_CONTAINER,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: PRIMARY,
            border: iced::Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

// ── Bottom Stats Row ────────────────────────────────────────────────────────

fn stats_row<'a>(window_count: usize, workspace_count: usize) -> Element<'a, Message> {
    row![
        stat_card("WINDOWS", &window_count.to_string(), PRIMARY),
        stat_card("WORKSPACES", &workspace_count.to_string(), SECONDARY),
    ]
    .spacing(12)
    .into()
}

fn stat_card<'a>(label: &'a str, value: &str, accent: iced::Color) -> Element<'a, Message> {
    let value_owned = value.to_string();

    container(
        row![column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_MEDIUM)
                .color(OUTLINE_VARIANT),
            text(value_owned).size(24).font(fonts::UI_FONT_SEMIBOLD),
        ]
        .spacing(4)
        .width(Length::Fill),]
        .align_y(Alignment::Center),
    )
    .padding([16, 20])
    .width(Length::FillPortion(1))
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(SURFACE_CONTAINER)),
        border: iced::Border {
            color: iced::Color { a: 0.3, ..accent },
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn label_text<'a>(s: &'a str) -> Element<'a, Message> {
    text(s)
        .size(10)
        .font(fonts::UI_FONT_SEMIBOLD)
        .color(OUTLINE_VARIANT)
        .into()
}

fn status_pill<'a>(label: &'a str, variant: PillVariant) -> Element<'a, Message> {
    let (bg, fg) = match variant {
        PillVariant::Active => (
            iced::Color {
                a: 0.12,
                ..SECONDARY
            },
            SECONDARY,
        ),
        PillVariant::Error => (
            iced::Color {
                a: 0.12,
                ..iced::Color::from_rgb(1.0, 0.44, 0.42)
            },
            iced::Color::from_rgb(1.0, 0.44, 0.42),
        ),
        PillVariant::Warning => (
            iced::Color {
                a: 0.12,
                ..iced::Color::from_rgb(0.96, 0.62, 0.04)
            },
            iced::Color::from_rgb(0.96, 0.62, 0.04),
        ),
        PillVariant::Muted => (SURFACE_CONTAINER_HIGHEST, OUTLINE_VARIANT),
    };

    container(
        row![
            container(Space::new().width(6).height(6)).style(move |_theme: &iced::Theme| {
                container::Style {
                    background: Some(iced::Background::Color(fg)),
                    border: iced::Border {
                        radius: 999.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
            text(label).size(10).font(fonts::UI_FONT_SEMIBOLD).color(fg),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            radius: 999.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn neon_card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(24)
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(SURFACE_CONTAINER)),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}
