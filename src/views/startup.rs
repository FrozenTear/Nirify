//! Startup commands settings view
//!
//! Configure commands that run when niri starts.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::StartupSettings;
use crate::messages::{Message, StartupMessage};
use crate::theme::fonts;

/// Creates the startup commands settings view
pub fn view(settings: &StartupSettings) -> Element<'static, Message> {
    let commands = settings.commands.clone();

    let mut content = column![
        page_title("Startup Commands"),
        info_text(
            "Commands listed here will run automatically when niri starts. \
             Useful for launching background services, setting up your environment, etc."
        ),
    ]
    .spacing(4);

    if commands.is_empty() {
        content = content.push(
            container(
                column![
                    text("No startup commands configured")
                        .size(14)
                        .color([0.75, 0.75, 0.75]),
                    spacer(8.0),
                    text("Click the button below to add your first command")
                        .size(13)
                        .color([0.5, 0.5, 0.5]),
                ]
                .align_x(Alignment::Center)
            )
            .padding(24)
            .center(Length::Fill)
        );
    } else {
        content = content.push(subsection_header("Configured Commands"));

        for cmd in &commands {
            let cmd_id = cmd.id;
            let cmd_display = cmd.display();

            content = content.push(
                container(
                    column![
                        row![
                            text(format!("Command #{}", cmd_id)).size(12).color([0.5, 0.5, 0.5]),
                            button(text("×").size(14))
                                .on_press(Message::Startup(StartupMessage::RemoveCommand(cmd_id)))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        row![
                            text("Command").size(12).color([0.75, 0.75, 0.75]),
                            text_input("e.g., waybar", &cmd_display)
                                .on_input(move |s| Message::Startup(StartupMessage::SetCommand(cmd_id, s)))
                                .padding(8)
                                .font(fonts::MONO_FONT)
                                .width(Length::Fill),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(8)
                )
                .padding(12)
                .style(card_style)
            );
            content = content.push(spacer(4.0));
        }
    }

    // Add command button
    content = content.push(spacer(8.0));
    content = content.push(
        button(
            row![
                text("+").size(16),
                text("Add Command").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .on_press(Message::Startup(StartupMessage::AddCommand))
        .padding([12, 20])
        .style(add_button_style)
    );

    content = content.push(subsection_header("Tips"));
    content = content.push(info_text("Commands are split by whitespace. For complex commands, create a script and call it here."));
    content = content.push(spacer(4.0));
    content = content.push(text("Example commands:").size(13).color([0.7, 0.7, 0.7]));
    content = content.push(text("  waybar").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  mako").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  /home/user/scripts/startup.sh").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(spacer(32.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Style for delete buttons
fn delete_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
        ..Default::default()
    }
}

/// Style for add buttons
fn add_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
        _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
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

/// Style for card containers
fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.15, 0.4))),
        border: iced::Border {
            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
