//! Miscellaneous settings view — neon modal style

use iced::widget::{column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::toggle_row;
use crate::config::models::{MiscSettings, XWaylandSatelliteConfig};
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

    let content = column![
        // ── 2-COLUMN LAYOUT ──
        row![
            // Left column: Window Decorations, Clipboard, Hotkey Overlay, Notifications
            column![
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
                        |v| Message::Miscellaneous(
                            MiscellaneousMessage::SetDisablePrimaryClipboard(v)
                        ),
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
                            |v| Message::Miscellaneous(
                                MiscellaneousMessage::SetHotkeyOverlayHideNotBound(v)
                            ),
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
            .width(Length::FillPortion(1)),
            // Right column: Screenshots, Startup Shell, XWayland
            column![
                modal_section("\u{25A6}", "SCREENSHOTS", neon::SECONDARY),
                Space::new().height(4),
                styled_text_input(
                    "SCREENSHOT PATH",
                    "~/Pictures/Screenshots/%Y-%m-%d_%H-%M-%S.png",
                    &screenshot_path,
                    |s| Message::Miscellaneous(MiscellaneousMessage::SetScreenshotPath(s)),
                ),
                Space::new().height(12),
                modal_section("\u{25B6}", "STARTUP BEHAVIOR", neon::TERTIARY),
                Space::new().height(4),
                styled_text_input(
                    "SHELL COMMAND AT STARTUP",
                    "e.g., ~/.config/niri/startup.sh",
                    &spawn_sh,
                    |s| Message::Miscellaneous(MiscellaneousMessage::SetSpawnShAtStartup(s)),
                ),
                Space::new().height(12),
                modal_section("\u{2B1C}", "XWAYLAND", neon::PRIMARY),
                Space::new().height(4),
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
                            |selected| Message::Miscellaneous(
                                MiscellaneousMessage::SetXWaylandSatellite(selected)
                            ),
                        )
                        .width(Length::Fixed(120.0)),
                    ]
                    .align_y(Alignment::Center),]
                    .spacing(4),
                )
                .padding(12)
                .style(crate::theme::card_style),
                if is_custom {
                    styled_text_input(
                        "CUSTOM PATH",
                        "Path to xwayland-satellite",
                        &custom_path,
                        |s| {
                            Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(
                                XWaylandSatelliteConfig::CustomPath(s),
                            ))
                        },
                    )
                } else {
                    column![].into()
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0);

    content.into()
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
