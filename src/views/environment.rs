//! Environment settings view
//!
//! Shows environment variables configured for niri with add/edit/remove functionality.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::EnvironmentSettings;
use crate::messages::{EnvironmentMessage, Message};
use crate::theme::fonts;

/// Creates the environment settings view
pub fn view(settings: &EnvironmentSettings) -> Element<'static, Message> {
    let variables = settings.variables.clone();

    let mut content = column![
        section_header("Environment Variables"),
        info_text(
            "Environment variables set for programs launched by niri."
        ),
        spacer(16.0),
    ]
    .spacing(4);

    if variables.is_empty() {
        content = content.push(
            container(
                column![
                    text("No environment variables configured")
                        .size(14)
                        .color([0.6, 0.6, 0.6]),
                    spacer(8.0),
                    text("Click the button below to add your first variable")
                        .size(13)
                        .color([0.5, 0.5, 0.5]),
                ]
                .align_x(Alignment::Center)
            )
            .padding(24)
            .center(Length::Fill)
        );
    } else {
        content = content.push(subsection_header("Configured Variables"));
        content = content.push(spacer(8.0));

        for var in &variables {
            let var_id = var.id;
            let var_name = var.name.clone();
            let var_value = var.value.clone();

            content = content.push(
                container(
                    column![
                        row![
                            text(format!("Variable #{}", var_id)).size(12).color([0.5, 0.5, 0.5]),
                            button(text("×").size(14))
                                .on_press(Message::Environment(EnvironmentMessage::RemoveVariable(var_id)))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        row![
                            column![
                                text("Name").size(12).color([0.6, 0.6, 0.6]),
                                text_input("VARIABLE_NAME", &var_name)
                                    .on_input(move |s| Message::Environment(EnvironmentMessage::SetVariableName(var_id, s)))
                                    .padding(8)
                                    .font(fonts::MONO_FONT)
                                    .width(Length::Fixed(200.0)),
                            ]
                            .spacing(4),
                            column![
                                text("Value").size(12).color([0.6, 0.6, 0.6]),
                                text_input("value", &var_value)
                                    .on_input(move |s| Message::Environment(EnvironmentMessage::SetVariableValue(var_id, s)))
                                    .padding(8)
                                    .font(fonts::MONO_FONT)
                                    .width(Length::Fill),
                            ]
                            .spacing(4)
                            .width(Length::Fill),
                        ]
                        .spacing(16),
                    ]
                    .spacing(8)
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

    // Add variable button
    content = content.push(spacer(8.0));
    content = content.push(
        button(
            row![
                text("+").size(16),
                text("Add Variable").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .on_press(Message::Environment(EnvironmentMessage::AddVariable))
        .padding([12, 20])
        .style(add_button_style)
    );

    content = content.push(spacer(16.0));
    content = content.push(subsection_header("Common Variables"));
    content = content.push(spacer(8.0));
    content = content.push(info_text("Examples of commonly used environment variables:"));
    content = content.push(spacer(4.0));
    content = content.push(text("DISPLAY").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  X11 display (e.g., \":0\")").size(12).color([0.6, 0.6, 0.6]));
    content = content.push(text("WAYLAND_DISPLAY").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  Wayland display socket (e.g., \"wayland-1\")").size(12).color([0.6, 0.6, 0.6]));
    content = content.push(text("XDG_CURRENT_DESKTOP").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  Desktop environment name (e.g., \"niri\")").size(12).color([0.6, 0.6, 0.6]));
    content = content.push(text("GTK_THEME").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]));
    content = content.push(text("  GTK theme name (e.g., \"Adwaita:dark\")").size(12).color([0.6, 0.6, 0.6]));

    scrollable(container(content).padding(20)).into()
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
