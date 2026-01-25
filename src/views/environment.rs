//! Environment settings view
//!
//! Shows environment variables configured for niri with add/edit/remove functionality.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::EnvironmentSettings;
use crate::messages::{EnvironmentMessage, Message};
use crate::theme::{fonts, muted_text_container, code_text_container};

/// Creates the environment settings view
pub fn view(settings: &EnvironmentSettings) -> Element<'static, Message> {
    let variables = settings.variables.clone();

    let mut content = column![
        page_title("Environment Variables"),
        info_text(
            "Environment variables set for programs launched by niri."
        ),
    ]
    .spacing(4);

    if variables.is_empty() {
        content = content.push(
            card(column![
                container(
                    column![
                        container(text("No environment variables configured").size(14)).style(muted_text_container),
                        spacer(8.0),
                        container(text("Click the button below to add your first variable").size(13)).style(muted_text_container),
                    ]
                    .align_x(Alignment::Center)
                )
                .padding(24)
                .center(Length::Fill)
            ].width(Length::Fill))
        );
    } else {
        content = content.push(subsection_header("Configured Variables"));

        for var in &variables {
            let var_id = var.id;
            let var_name = var.name.clone();
            let var_value = var.value.clone();

            content = content.push(
                card(column![
                    row![
                        container(text(format!("Variable #{}", var_id)).size(12)).style(muted_text_container).width(Length::Fill),
                        button(text("Delete").size(12))
                            .on_press(Message::Environment(EnvironmentMessage::RemoveVariable(var_id)))
                            .padding([4, 12])
                            .style(delete_button_style),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    row![
                        column![
                            container(text("Name").size(12)).style(muted_text_container),
                            text_input("VARIABLE_NAME", &var_name)
                                .on_input(move |s| Message::Environment(EnvironmentMessage::SetVariableName(var_id, s)))
                                .padding(8)
                                .font(fonts::MONO_FONT)
                                .width(Length::Fixed(200.0)),
                        ]
                        .spacing(4),
                        column![
                            container(text("Value").size(12)).style(muted_text_container),
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
                ].spacing(8).padding(12).width(Length::Fill))
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

    content = content.push(subsection_header("Common Variables"));
    content = content.push(card(column![
        info_text("Examples of commonly used environment variables:"),
        spacer(4.0),
        container(text("DISPLAY").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  X11 display (e.g., \":0\")").size(12)).style(muted_text_container),
        container(text("WAYLAND_DISPLAY").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  Wayland display socket (e.g., \"wayland-1\")").size(12)).style(muted_text_container),
        container(text("XDG_CURRENT_DESKTOP").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  Desktop environment name (e.g., \"niri\")").size(12)).style(muted_text_container),
        container(text("GTK_THEME").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        container(text("  GTK theme name (e.g., \"Adwaita:dark\")").size(12)).style(muted_text_container),
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
