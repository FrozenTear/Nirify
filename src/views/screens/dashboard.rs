//! Dashboard screen — command center chrome matching Layout / Visuals
//!
//! Same page shell as the redesigned screens: category label, large title,
//! descriptive subtitle, hero visual band, then summary cards.

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::config::Settings;
use crate::ipc::WorkspaceInfo;
use crate::messages::{EditableSection, Message, ToolsMessage};
use crate::theme::{fonts, neon};
use crate::version::NiriVersion;
use crate::views::status_bar::NiriStatus;
use crate::views::tools::ToolsState;

use neon::{
    ERROR, ON_SURFACE_VARIANT, OUTLINE_VARIANT, PRIMARY, SECONDARY, SURFACE_CONTAINER,
    SURFACE_CONTAINER_HIGHEST, TERTIARY,
};

const CATEGORY: &str = "COMMAND CENTER";
const TITLE: &str = "Live Session";
const DESCRIPTION: &str = "Session health, workspace occupancy, and the features currently shaping your ribbon. Status updates live from niri; changes save automatically.";
const CONFIG_PATH: &str = "~/.config/niri/config.kdl";

/// Dashboard screen
pub fn view<'a>(
    niri_status: NiriStatus,
    niri_version: Option<NiriVersion>,
    tools_state: &'a ToolsState,
    settings: &'a Settings,
) -> Element<'a, Message> {
    let status = session_status(niri_status, niri_version);
    let reload = reload_action(niri_status, tools_state.reloading);

    let content = column![
        super::hero_header(CATEGORY, TITLE, DESCRIPTION, SECONDARY),
        Space::new().height(16),
        session_preview(&status, &tools_state.workspaces),
        Space::new().height(24),
        row![
            super::summary_card(
                "◈",
                "Session",
                SECONDARY,
                vec![
                    ("Status", status.headline.to_string()),
                    ("Version", status.version.clone()),
                    (
                        "Occupancy",
                        occupancy_pair_label(
                            tools_state.windows.len(),
                            tools_state.workspaces.len(),
                            niri_status,
                        ),
                    ),
                ],
                reload.label,
                reload.message,
            ),
            super::section_summary_card(
                EditableSection::SpatialGaps,
                vec![
                    ("Gaps", format!("{:.0}px", settings.appearance.gaps)),
                    (
                        "Radius",
                        format!("{:.0}px", settings.appearance.corner_radius)
                    ),
                ],
            ),
            super::section_summary_card(
                EditableSection::Animations,
                vec![
                    (
                        "Enabled",
                        if settings.animations.enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string()
                    ),
                    ("Slowdown", format!("{:.1}x", settings.animations.slowdown)),
                ],
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Start),
        Space::new().height(12),
        row![
            super::section_summary_card(
                EditableSection::FocusRing,
                vec![
                    (
                        "Enabled",
                        if settings.appearance.focus_ring_enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string()
                    ),
                    (
                        "Width",
                        format!("{}px", settings.appearance.focus_ring_width)
                    ),
                ],
            ),
            super::section_summary_card(
                EditableSection::NamedWorkspaces,
                vec![
                    (
                        "Active",
                        occupancy_count_label(tools_state.workspaces.len(), niri_status)
                    ),
                    ("Named", format!("{}", settings.workspaces.workspaces.len())),
                ],
            ),
            super::section_summary_card(
                EditableSection::Overview,
                vec![
                    ("Zoom", format!("{:.2}x", settings.overview.zoom)),
                    (
                        "Shadow",
                        if settings
                            .overview
                            .workspace_shadow
                            .as_ref()
                            .is_some_and(|s| s.enabled)
                        {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                    ),
                ],
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Start),
    ]
    .spacing(0)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

// ── Status copy ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct SessionStatus {
    pill: &'static str,
    headline: &'static str,
    version: String,
    accent: iced::Color,
}

#[derive(Debug, Clone)]
struct ReloadAction {
    label: &'static str,
    message: Option<Message>,
}

fn session_status(status: NiriStatus, version: Option<NiriVersion>) -> SessionStatus {
    let (pill, headline, accent) = match status {
        NiriStatus::Connected => ("RUNNING", "Running", SECONDARY),
        NiriStatus::Disconnected => ("OFFLINE", "Offline", ERROR),
        NiriStatus::Unknown => ("CHECKING", "Checking", OUTLINE_VARIANT),
    };

    SessionStatus {
        pill,
        headline,
        version: version_label(version),
        accent,
    }
}

fn version_label(version: Option<NiriVersion>) -> String {
    version
        .map(|v| format!("niri v{v}"))
        .unwrap_or_else(|| "niri".to_string())
}

fn reload_action(status: NiriStatus, reloading: bool) -> ReloadAction {
    let can_reload = matches!(status, NiriStatus::Connected) && !reloading;
    ReloadAction {
        label: if reloading {
            "RELOADING..."
        } else {
            "RELOAD CONFIG"
        },
        message: can_reload.then_some(Message::Tools(ToolsMessage::ReloadConfig)),
    }
}

fn occupancy_count_label(count: usize, status: NiriStatus) -> String {
    if matches!(status, NiriStatus::Connected) {
        count.to_string()
    } else {
        "—".to_string()
    }
}

fn occupancy_pair_label(windows: usize, workspaces: usize, status: NiriStatus) -> String {
    if matches!(status, NiriStatus::Connected) {
        format!("{windows} win / {workspaces} ws")
    } else {
        "—".to_string()
    }
}

// ── Hero visual band ────────────────────────────────────────────────────────

fn session_preview<'a>(
    status: &SessionStatus,
    workspaces: &'a [WorkspaceInfo],
) -> Element<'a, Message> {
    let (left, right) = flanking_workspaces(workspaces);

    super::hero_visual_band(
        row![
            workspace_column(left),
            focused_session_column(status, focused_workspace(workspaces)),
            workspace_column(right),
        ]
        .spacing(12)
        .padding(24)
        .height(Length::Fixed(220.0)),
    )
}

fn focused_workspace(workspaces: &[WorkspaceInfo]) -> Option<&WorkspaceInfo> {
    workspaces
        .iter()
        .find(|ws| ws.is_focused || ws.is_active)
        .or_else(|| workspaces.first())
}

fn flanking_workspaces(
    workspaces: &[WorkspaceInfo],
) -> (Option<&WorkspaceInfo>, Option<&WorkspaceInfo>) {
    let focused_id = focused_workspace(workspaces).map(|ws| ws.id);
    let mut others = workspaces.iter().filter(|ws| Some(ws.id) != focused_id);
    (others.next(), others.next())
}

fn workspace_column<'a>(ws: Option<&'a WorkspaceInfo>) -> Element<'a, Message> {
    match ws {
        Some(ws) => {
            let default_name = format!("{:02}", ws.idx);
            let name = ws.name.as_deref().unwrap_or(&default_name);
            container(
                column![
                    text(format!("{:02}", ws.idx))
                        .size(18)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(ON_SURFACE_VARIANT),
                    text(name.to_uppercase())
                        .size(9)
                        .font(fonts::MONO_FONT)
                        .color(OUTLINE_VARIANT),
                    Space::new().height(Length::Fill),
                    container(Space::new().width(Length::Fill).height(6)).style(
                        |_: &iced::Theme| {
                            container::Style {
                                background: Some(iced::Background::Color(OUTLINE_VARIANT)),
                                border: iced::Border {
                                    radius: 2.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }
                    ),
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
            .into()
        }
        None => inactive_skeleton(),
    }
}

fn inactive_skeleton<'a>() -> Element<'a, Message> {
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
    .into()
}

fn focused_session_column<'a>(
    status: &SessionStatus,
    workspace: Option<&'a WorkspaceInfo>,
) -> Element<'a, Message> {
    let pill = status.pill;
    let accent = status.accent;
    let version = status.version.clone();
    let workspace_tag = workspace
        .map(|ws| {
            ws.name
                .as_deref()
                .map(|n| n.to_uppercase())
                .unwrap_or_else(|| format!("WS {:02}", ws.idx))
        })
        .unwrap_or_else(|| "LIVE_SESSION".to_string());

    container(
        column![
            row![
                traffic_light(ERROR),
                traffic_light(TERTIARY),
                traffic_light(SECONDARY),
                Space::new().width(Length::Fill),
                status_tag(pill, accent),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            Space::new().height(8),
            row![
                text(workspace_tag)
                    .size(13)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(PRIMARY),
                Space::new().width(Length::Fill),
                text(version)
                    .size(10)
                    .font(fonts::MONO_FONT)
                    .color(ON_SURFACE_VARIANT),
            ]
            .align_y(Alignment::Center),
            text(CONFIG_PATH)
                .size(10)
                .font(fonts::MONO_FONT)
                .color(OUTLINE_VARIANT),
            Space::new().height(4),
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
                |_: &iced::Theme| {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color {
                            a: 0.06,
                            ..PRIMARY
                        })),
                        border: iced::Border {
                            radius: 3.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }
            ),
            Space::new().height(4),
            container(Space::new().width(Length::Fill).height(Length::Fill)).style(
                |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.04, ..PRIMARY })),
                    border: iced::Border {
                        color: iced::Color { a: 0.10, ..PRIMARY },
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                },
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
    })
    .into()
}

fn traffic_light<'a>(color: iced::Color) -> Element<'a, Message> {
    container(Space::new().width(8).height(8))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: 999.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn status_tag<'a>(label: &'static str, accent: iced::Color) -> Element<'a, Message> {
    container(text(label).size(9).font(fonts::MONO_FONT).color(accent))
        .padding([4, 8])
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color { a: 0.10, ..accent })),
            border: iced::Border {
                color: iced::Color { a: 0.20, ..accent },
                width: 1.0,
                radius: 999.0.into(),
            },
            ..Default::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_status_uses_running_copy() {
        let status = session_status(
            NiriStatus::Connected,
            Some(NiriVersion {
                major: 26,
                minor: 4,
            }),
        );
        assert_eq!(status.pill, "RUNNING");
        assert_eq!(status.headline, "Running");
        assert_eq!(status.version, "niri v26.04");
        assert_eq!(status.accent, SECONDARY);
    }

    #[test]
    fn disconnected_status_uses_offline_copy() {
        let status = session_status(NiriStatus::Disconnected, None);
        assert_eq!(status.pill, "OFFLINE");
        assert_eq!(status.headline, "Offline");
        assert_eq!(status.version, "niri");
        assert_eq!(status.accent, ERROR);
    }

    #[test]
    fn unknown_status_uses_checking_copy() {
        let status = session_status(NiriStatus::Unknown, None);
        assert_eq!(status.pill, "CHECKING");
        assert_eq!(status.headline, "Checking");
    }

    #[test]
    fn reload_enabled_only_when_connected_and_idle() {
        assert!(reload_action(NiriStatus::Connected, false)
            .message
            .is_some());
        assert!(reload_action(NiriStatus::Connected, true).message.is_none());
        assert!(reload_action(NiriStatus::Disconnected, false)
            .message
            .is_none());
        assert_eq!(
            reload_action(NiriStatus::Connected, true).label,
            "RELOADING..."
        );
        assert_eq!(
            reload_action(NiriStatus::Connected, false).label,
            "RELOAD CONFIG"
        );
    }

    #[test]
    fn occupancy_hides_counts_when_offline() {
        assert_eq!(occupancy_count_label(13, NiriStatus::Connected), "13");
        assert_eq!(occupancy_count_label(13, NiriStatus::Disconnected), "—");
        assert_eq!(occupancy_count_label(13, NiriStatus::Unknown), "—");
        assert_eq!(
            occupancy_pair_label(13, 9, NiriStatus::Connected),
            "13 win / 9 ws"
        );
        assert_eq!(occupancy_pair_label(13, 9, NiriStatus::Disconnected), "—");
    }

    #[test]
    fn flanking_workspaces_skip_focused() {
        let workspaces = vec![
            WorkspaceInfo {
                id: 1,
                idx: 1,
                name: Some("one".into()),
                output: None,
                is_active: false,
                is_focused: false,
                active_window_id: None,
            },
            WorkspaceInfo {
                id: 2,
                idx: 2,
                name: Some("two".into()),
                output: None,
                is_active: true,
                is_focused: true,
                active_window_id: None,
            },
            WorkspaceInfo {
                id: 3,
                idx: 3,
                name: Some("three".into()),
                output: None,
                is_active: false,
                is_focused: false,
                active_window_id: None,
            },
        ];
        let focused = focused_workspace(&workspaces).unwrap();
        assert_eq!(focused.id, 2);
        let (left, right) = flanking_workspaces(&workspaces);
        assert_eq!(left.map(|ws| ws.id), Some(1));
        assert_eq!(right.map(|ws| ws.id), Some(3));
    }
}
