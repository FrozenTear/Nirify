//! Recent windows settings view
//!
//! Shows settings for the window switcher overlay (Alt-Tab).

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::{RecentWindowsSettings, RecentWindowsScope};
use crate::messages::Message;
use crate::theme::fonts;

/// Creates the recent windows settings view
pub fn view(settings: &RecentWindowsSettings) -> Element<'static, Message> {
    let mut content = column![
        section_header("Recent Windows Switcher"),
        info_text(
            "Configure the window switcher overlay for switching between recently used windows (Alt-Tab)."
        ),
        spacer(16.0),

        // Status
        subsection_header("Status"),
        display_toggle("Switcher Enabled", !settings.off),
        spacer(16.0),

        // Timing
        subsection_header("Timing"),
        display_value("Debounce Delay", &format!("{} ms", settings.debounce_ms)),
        display_value("Open Delay", &format!("{} ms", settings.open_delay_ms)),
        info_text("Debounce: delay before window is added to recent list. Open: delay before switcher UI appears."),
        spacer(16.0),

        // Highlight
        subsection_header("Highlight Style"),
        display_color("Active Color", &settings.highlight.active_color.to_hex()),
        display_color("Urgent Color", &settings.highlight.urgent_color.to_hex()),
        display_value("Padding", &format!("{} px", settings.highlight.padding)),
        display_value("Corner Radius", &format!("{} px", settings.highlight.corner_radius)),
        spacer(16.0),

        // Previews
        subsection_header("Preview Settings"),
        display_value("Max Height", &format!("{} px", settings.previews.max_height)),
        display_value("Max Scale", &format!("{:.0}%", settings.previews.max_scale * 100.0)),
        spacer(16.0),
    ]
    .spacing(4);

    // Keybindings
    content = content.push(subsection_header("Custom Keybindings"));
    content = content.push(spacer(8.0));

    if settings.binds.is_empty() {
        content = content.push(
            text("Using default keybindings (Alt+Tab)")
                .size(14)
                .color([0.6, 0.6, 0.6])
        );
    } else {
        for bind in &settings.binds {
            let direction = if bind.is_next { "Next" } else { "Previous" };
            let scope = bind.scope.as_ref().map_or("All".to_string(), |s| match s {
                RecentWindowsScope::All => "All".to_string(),
                RecentWindowsScope::Output => "Output".to_string(),
                RecentWindowsScope::Workspace => "Workspace".to_string(),
            });
            let filter = if bind.filter_app_id { " (same app)" } else { "" };

            content = content.push(
                container(
                    column![
                        row![
                            text(bind.key_combo.clone())
                                .size(14)
                                .font(fonts::MONO_FONT)
                                .color([0.7, 0.85, 0.7]),
                            text(format!("→ {} window{}", direction, filter))
                                .size(14)
                                .color([0.8, 0.8, 0.8]),
                        ]
                        .spacing(12),
                        row![
                            text(format!("Scope: {}", scope))
                                .size(13)
                                .color([0.6, 0.6, 0.6]),
                            text(bind.cooldown_ms.map_or("No cooldown".to_string(), |ms| format!("Cooldown: {} ms", ms)))
                                .size(13)
                                .color([0.6, 0.6, 0.6]),
                        ]
                        .spacing(16),
                    ]
                    .spacing(4)
                )
                .padding(12)
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.15, 0.4))),
                        border: iced::Border {
                            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    }
                })
            );
            content = content.push(spacer(4.0));
        }
    }

    content = content.push(spacer(16.0));
    content = content.push(info_text(
        "Edit recent-windows.kdl directly to configure these settings. Full editing UI coming in a future update."
    ));

    scrollable(container(content).padding(20)).into()
}

/// Display a toggle value (read-only)
fn display_toggle(label: &str, value: bool) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        text(if value { "Yes" } else { "No" })
            .size(14)
            .color(if value { [0.5, 0.8, 0.5] } else { [0.6, 0.6, 0.6] }),
    ]
    .spacing(16)
    .into()
}

/// Display a value (read-only)
fn display_value(label: &str, value: &str) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        text(value.to_string()).size(14).color([0.8, 0.8, 0.8]),
    ]
    .spacing(16)
    .into()
}

/// Display a color value (read-only)
fn display_color(label: &str, hex: &str) -> Element<'static, Message> {
    let hex_owned = hex.to_string();
    let hex_display = hex.to_string();

    // Parse color once for the swatch
    let parsed_color = crate::types::Color::from_hex(&hex_owned)
        .map(|c| iced::Color::from_rgba(c.r as f32, c.g as f32, c.b as f32, c.a as f32))
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        container(text("").size(14))
            .width(20)
            .height(20)
            .style(move |_theme| {
                container::Style {
                    background: Some(iced::Background::Color(parsed_color)),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            }),
        text(hex_display).size(14).font(fonts::MONO_FONT).color([0.8, 0.8, 0.8]),
    ]
    .spacing(8)
    .into()
}
