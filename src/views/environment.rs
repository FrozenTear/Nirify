//! Environment settings view
//!
//! Shows environment variables configured for niri.

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::config::models::EnvironmentSettings;
use crate::messages::Message;
use crate::theme::fonts;

/// Creates the environment settings view
pub fn view(settings: &EnvironmentSettings) -> Element<'static, Message> {
    let mut content = column![
        section_header("Environment Variables"),
        info_text(
            "Environment variables set for programs launched by niri."
        ),
        spacer(16.0),
    ]
    .spacing(4);

    if settings.variables.is_empty() {
        content = content.push(
            text("No environment variables configured")
                .size(14)
                .color([0.6, 0.6, 0.6])
        );
        content = content.push(spacer(8.0));
        content = content.push(
            text("Environment variables can be set in environment.kdl")
                .size(13)
                .color([0.5, 0.5, 0.5])
        );
    } else {
        content = content.push(subsection_header("Configured Variables"));
        content = content.push(spacer(8.0));

        for var in &settings.variables {
            content = content.push(
                container(
                    column![
                        text(var.name.clone())
                            .size(14)
                            .font(fonts::MONO_FONT)
                            .color([0.7, 0.85, 0.7]),
                        text(if var.value.is_empty() { "(empty)".to_string() } else { var.value.clone() })
                            .size(14)
                            .font(fonts::MONO_FONT)
                            .color([0.8, 0.8, 0.8]),
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
    content = content.push(subsection_header("Example Configuration"));
    content = content.push(spacer(8.0));
    content = content.push(text("environment {").size(13).font(fonts::MONO_FONT));
    content = content.push(text("    DISPLAY \":0\"").size(13).font(fonts::MONO_FONT));
    content = content.push(text("    WAYLAND_DISPLAY \"wayland-1\"").size(13).font(fonts::MONO_FONT));
    content = content.push(text("    XDG_CURRENT_DESKTOP \"niri\"").size(13).font(fonts::MONO_FONT));
    content = content.push(text("}").size(13).font(fonts::MONO_FONT));
    content = content.push(spacer(16.0));
    content = content.push(info_text(
        "Edit environment.kdl directly to add or modify variables. Full editing UI coming in a future update."
    ));

    scrollable(container(content).padding(20)).into()
}
