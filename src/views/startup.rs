//! Startup commands settings view
//!
//! Display commands that run when niri starts.

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::config::models::StartupSettings;
use crate::messages::Message;

/// Creates the startup commands settings view
pub fn view(settings: &StartupSettings) -> Element<'static, Message> {
    let mut content = column![
        section_header("Startup Commands"),
        info_text(
            "Commands listed here will run automatically when niri starts. \
             Useful for launching background services, setting up your environment, etc."
        ),
        spacer(16.0),
    ]
    .spacing(8);

    if settings.commands.is_empty() {
        content = content.push(
            text("No startup commands configured")
                .size(14)
                .color([0.6, 0.6, 0.6])
        );
    } else {
        for (idx, cmd) in settings.commands.iter().enumerate() {
            let cmd_display = cmd.display();
            content = content.push(
                container(
                    column![
                        text(format!("Command {}", idx + 1)).size(13).color([0.7, 0.7, 0.7]),
                        text(if cmd_display.is_empty() { "(empty)".to_string() } else { cmd_display })
                            .size(14)
                            .color([0.9, 0.9, 0.9]),
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
        }
    }

    content = content.push(spacer(16.0));
    content = content.push(info_text(
        "Edit startup.kdl directly to add or modify startup commands. Full UI coming in a future update."
    ));

    scrollable(container(content).padding(20)).into()
}
