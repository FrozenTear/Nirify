//! Startup commands settings view
//!
//! Configure commands that run when niri starts.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::StartupSettings;
use crate::messages::{Message, StartupMessage};
use crate::theme::{fonts, muted_text_container, code_text_container};

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
            card(column![
                container(
                    column![
                        container(text("No startup commands configured").size(14)).style(muted_text_container),
                        spacer(8.0),
                        container(text("Click the button below to add your first command").size(13)).style(muted_text_container),
                    ]
                    .align_x(Alignment::Center)
                )
                .padding(24)
                .center(Length::Fill)
            ].width(Length::Fill))
        );
    } else {
        content = content.push(subsection_header("Configured Commands"));

        for cmd in &commands {
            let cmd_id = cmd.id;
            let cmd_display = cmd.display();

            content = content.push(
                card(column![
                    row![
                        container(text(format!("Command #{}", cmd_id)).size(12)).style(muted_text_container),
                        button(text("×").size(14))
                            .on_press(Message::Startup(StartupMessage::RemoveCommand(cmd_id)))
                            .padding([2, 8])
                            .style(delete_button_style),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    row![
                        container(text("Command").size(12)).style(muted_text_container),
                        text_input("e.g., waybar", &cmd_display)
                            .on_input(move |s| Message::Startup(StartupMessage::SetCommand(cmd_id, s)))
                            .padding(8)
                            .font(fonts::MONO_FONT)
                            .width(Length::Fill),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ].spacing(8).padding(12).width(Length::Fill))
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
    content = content.push(card(column![
        info_text("Commands are split by whitespace. For complex commands, create a script and call it here."),
        spacer(4.0),
        container(text("Example commands:").size(13)).style(muted_text_container),
        container(text("  waybar").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  mako").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  /home/user/scripts/startup.sh").size(13).font(fonts::MONO_FONT)).style(code_text_container),
    ].spacing(4).padding(12).width(Length::Fill)));
    content = content.push(spacer(32.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Style for delete buttons - uses theme danger color
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

/// Style for add buttons - uses theme primary color
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
