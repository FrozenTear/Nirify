//! Miscellaneous settings view — neon modal style

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use super::widgets::toggle_row;
use crate::config::models::{MiscSettings, ScreenshotPathConfig, XWaylandSatelliteConfig};
use crate::messages::{Message, MiscellaneousMessage};
use crate::theme::{fonts, neon};

/// Creates the miscellaneous settings view (with scrollable wrapper)
pub fn view(settings: &MiscSettings) -> Element<'static, Message> {
    let content = column![view_section(settings),]
        .spacing(0)
        .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Inner content without scrollable wrapper
pub fn view_section(settings: &MiscSettings) -> Element<'static, Message> {
    let screenshot_path = settings.screenshot_path.clone();
    let spawn_sh = settings.spawn_sh_at_startup.clone();
    let xwayland = settings.xwayland_satellite.clone();

    let xwayland_options = vec![
        XWaylandSatelliteConfig::Default,
        XWaylandSatelliteConfig::Off,
    ];
    let is_custom = matches!(&xwayland, XWaylandSatelliteConfig::CustomPath(_));
    let custom_path = match &xwayland {
        XWaylandSatelliteConfig::CustomPath(p) => p.clone(),
        _ => String::new(),
    };

    // ── LEFT COLUMN ──
    let left_col = column![
        modal_section("\u{25A3}", "WINDOW DECORATIONS", neon::PRIMARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Prefer No Client-Side Decorations",
                "Ask apps to use server-side decorations",
                settings.prefer_no_csd,
                |v| Message::Miscellaneous(MiscellaneousMessage::SetPreferNoCsd(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(12),
        modal_section("\u{25CE}", "CLIPBOARD", neon::SECONDARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Disable Primary Clipboard",
                "Disable middle-click paste (primary selection)",
                settings.disable_primary_clipboard,
                |v| Message::Miscellaneous(MiscellaneousMessage::SetDisablePrimaryClipboard(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(12),
        modal_section("\u{2328}", "HOTKEY OVERLAY", neon::TERTIARY),
        Space::new().height(4),
        container(
            column![
                toggle_row(
                    "Skip at Startup",
                    "Don't show hotkey overlay when niri starts",
                    settings.hotkey_overlay_skip_at_startup,
                    |v| Message::Miscellaneous(
                        MiscellaneousMessage::SetHotkeyOverlaySkipAtStartup(v)
                    ),
                ),
                toggle_row(
                    "Hide Unbound Actions",
                    "Hide actions without keybindings",
                    settings.hotkey_overlay_hide_not_bound,
                    |v| Message::Miscellaneous(MiscellaneousMessage::SetHotkeyOverlayHideNotBound(
                        v
                    )),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(12),
        modal_section("\u{26A0}", "NOTIFICATIONS", neon::PRIMARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Disable Config Failed Notification",
                "Don't show notification on config parse failure",
                settings.config_notification_disable_failed,
                |v| Message::Miscellaneous(
                    MiscellaneousMessage::SetConfigNotificationDisableFailed(v)
                ),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
    ]
    .spacing(6)
    .width(Length::FillPortion(1));

    // ── RIGHT COLUMN (built dynamically for list editors) ──
    let mut right_col = column![
        modal_section("\u{25A6}", "SCREENSHOTS", neon::SECONDARY),
        Space::new().height(4),
    ]
    .spacing(6);

    let screenshot_mode = match &screenshot_path {
        ScreenshotPathConfig::Default => ScreenshotMode::Default,
        ScreenshotPathConfig::Disabled => ScreenshotMode::Disabled,
        ScreenshotPathConfig::Custom(_) => ScreenshotMode::Custom,
    };
    let screenshot_custom = match &screenshot_path {
        ScreenshotPathConfig::Custom(p) => p.clone(),
        _ => String::new(),
    };

    right_col = right_col.push(
        container(
            row![
                text("SCREENSHOT PATH")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                pick_list(
                    vec![
                        ScreenshotMode::Default,
                        ScreenshotMode::Disabled,
                        ScreenshotMode::Custom,
                    ],
                    Some(screenshot_mode),
                    {
                        // Preserve the already-typed custom path when re-selecting "Custom"
                        // (pick_list fires on_select even for the currently-selected option).
                        let existing_custom = screenshot_custom.clone();
                        move |m| {
                            let cfg = match m {
                                ScreenshotMode::Default => ScreenshotPathConfig::Default,
                                ScreenshotMode::Disabled => ScreenshotPathConfig::Disabled,
                                ScreenshotMode::Custom => {
                                    ScreenshotPathConfig::Custom(existing_custom.clone())
                                }
                            };
                            Message::Miscellaneous(MiscellaneousMessage::SetScreenshotPath(cfg))
                        }
                    },
                )
                .width(Length::Fixed(170.0)),
            ]
            .align_y(Alignment::Center),
        )
        .padding(12)
        .style(crate::theme::card_style),
    );

    if screenshot_mode == ScreenshotMode::Custom {
        right_col = right_col.push(styled_text_input(
            "CUSTOM PATH",
            "~/Pictures/Screenshots/%Y-%m-%d_%H-%M-%S.png",
            &screenshot_custom,
            |s| {
                Message::Miscellaneous(MiscellaneousMessage::SetScreenshotPath(
                    ScreenshotPathConfig::Custom(s),
                ))
            },
        ));
    }

    right_col = right_col.push(Space::new().height(12));
    right_col = right_col.push(modal_section(
        "\u{25B6}",
        "SHELL COMMANDS AT STARTUP",
        neon::TERTIARY,
    ));
    right_col = right_col.push(Space::new().height(4));

    if spawn_sh.is_empty() {
        right_col = right_col.push(
            container(
                text("No shell commands configured")
                    .size(11)
                    .color(neon::OUTLINE_VARIANT),
            )
            .padding(12)
            .width(Length::Fill)
            .style(crate::theme::card_style),
        );
    } else {
        for cmd in &spawn_sh {
            let id = cmd.id;
            let command = cmd.command.clone();
            right_col = right_col.push(
                container(
                    column![
                        row![
                            text(format!("#{}", id))
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().width(Length::Fill),
                            button(text("\u{00D7}").size(14).color(neon::ERROR))
                                .on_press(Message::Miscellaneous(
                                    MiscellaneousMessage::RemoveSpawnShAtStartup(id),
                                ))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .align_y(Alignment::Center),
                        text_input("e.g., qs -c ~/.config/quickshell", &command)
                            .on_input(move |s| Message::Miscellaneous(
                                MiscellaneousMessage::SetSpawnShAtStartup(id, s)
                            ))
                            .padding(8)
                            .font(fonts::MONO_FONT)
                            .size(12)
                            .width(Length::Fill),
                    ]
                    .spacing(6),
                )
                .padding(12)
                .style(crate::theme::card_style),
            );
        }
    }

    right_col = right_col.push(Space::new().height(8));
    right_col = right_col.push(
        button(
            row![
                text("+").size(14).color(iced::Color::WHITE),
                text("Add Command")
                    .size(12)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(iced::Color::WHITE),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::Miscellaneous(
            MiscellaneousMessage::AddSpawnShAtStartup,
        ))
        .padding([10, 18])
        .style(add_button_style),
    );

    right_col = right_col.push(Space::new().height(12));
    right_col = right_col.push(modal_section("\u{2B1C}", "XWAYLAND", neon::PRIMARY));
    right_col = right_col.push(Space::new().height(4));
    right_col = right_col.push(
        container(
            column![row![
                text("XWAYLAND SATELLITE")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                pick_list(
                    xwayland_options,
                    Some(if is_custom {
                        XWaylandSatelliteConfig::Default
                    } else {
                        xwayland.clone()
                    }),
                    |selected| Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(
                        selected
                    )),
                )
                .width(Length::Fixed(120.0)),
            ]
            .align_y(Alignment::Center),]
            .spacing(4),
        )
        .padding(12)
        .style(crate::theme::card_style),
    );

    if is_custom {
        right_col = right_col.push(styled_text_input(
            "CUSTOM PATH",
            "Path to xwayland-satellite",
            &custom_path,
            |s| {
                Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(
                    XWaylandSatelliteConfig::CustomPath(s),
                ))
            },
        ));
    }

    let right_col = right_col.width(Length::FillPortion(1));

    row![left_col, right_col]
        .spacing(32)
        .align_y(Alignment::Start)
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

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

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let v = value.to_string();
    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text_input(placeholder, &v)
                .on_input(on_change)
                .padding(10)
                .size(13)
                .font(fonts::MONO_FONT),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

/// Local pick_list mode for screenshot path (maps to `ScreenshotPathConfig`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenshotMode {
    Default,
    Disabled,
    Custom,
}

impl std::fmt::Display for ScreenshotMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenshotMode::Default => write!(f, "Default location"),
            ScreenshotMode::Disabled => write!(f, "Don't save to disk"),
            ScreenshotMode::Custom => write!(f, "Custom path"),
        }
    }
}

fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.3, ..danger },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: danger,
        ..Default::default()
    }
}

fn add_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let primary = theme.palette().primary;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.5, ..primary },
        button::Status::Pressed => iced::Color { a: 0.6, ..primary },
        _ => iced::Color { a: 0.4, ..primary },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Implement Display for XWaylandSatelliteConfig for pick_list
impl std::fmt::Display for XWaylandSatelliteConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XWaylandSatelliteConfig::Default => write!(f, "Default"),
            XWaylandSatelliteConfig::Off => write!(f, "Disabled"),
            XWaylandSatelliteConfig::CustomPath(p) => write!(f, "Custom: {}", p),
        }
    }
}
