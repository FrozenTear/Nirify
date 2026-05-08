//! Environment settings view — neon modal style

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::info_text;
use crate::config::models::EnvironmentSettings;
use crate::messages::{EnvironmentMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the environment settings view (with scrollable wrapper)
pub fn view(settings: &EnvironmentSettings) -> Element<'static, Message> {
    let content = column![view_section(settings),]
        .spacing(0)
        .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Inner content without scrollable wrapper
pub fn view_section(settings: &EnvironmentSettings) -> Element<'static, Message> {
    let variables = settings.variables.clone();

    let mut left_col = column![
        modal_section("\u{2699}", "ENVIRONMENT VARIABLES", neon::PRIMARY),
        info_text("Environment variables set for programs launched by niri."),
        Space::new().height(4),
    ]
    .spacing(6);

    if variables.is_empty() {
        left_col = left_col.push(
            container(
                column![
                    text("No environment variables configured")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                    Space::new().height(4),
                    text("Click the button below to add your first variable")
                        .size(11)
                        .color(neon::OUTLINE_VARIANT),
                ]
                .align_x(Alignment::Center),
            )
            .padding(24)
            .center(Length::Fill)
            .style(crate::theme::card_style),
        );
    } else {
        for var in &variables {
            let var_id = var.id;
            let var_name = var.name.clone();
            let var_value = var.value.clone();

            left_col = left_col.push(
                container(
                    column![
                        row![
                            text(format!("#{}", var_id))
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().width(Length::Fill),
                            button(text("Delete").size(10).color(neon::ERROR))
                                .on_press(Message::Environment(EnvironmentMessage::RemoveVariable(
                                    var_id
                                )))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .align_y(Alignment::Center),
                        row![
                            column![
                                text("NAME")
                                    .size(10)
                                    .font(fonts::UI_FONT_SEMIBOLD)
                                    .color(neon::OUTLINE_VARIANT),
                                text_input("VARIABLE_NAME", &var_name)
                                    .on_input(move |s| Message::Environment(
                                        EnvironmentMessage::SetVariableName(var_id, s)
                                    ))
                                    .padding(8)
                                    .font(fonts::MONO_FONT)
                                    .size(12)
                                    .width(Length::Fixed(180.0)),
                            ]
                            .spacing(4),
                            column![
                                text("VALUE")
                                    .size(10)
                                    .font(fonts::UI_FONT_SEMIBOLD)
                                    .color(neon::OUTLINE_VARIANT),
                                text_input("value", &var_value)
                                    .on_input(move |s| Message::Environment(
                                        EnvironmentMessage::SetVariableValue(var_id, s)
                                    ))
                                    .padding(8)
                                    .font(fonts::MONO_FONT)
                                    .size(12)
                                    .width(Length::Fill),
                            ]
                            .spacing(4)
                            .width(Length::Fill),
                        ]
                        .spacing(12),
                    ]
                    .spacing(6),
                )
                .padding(12)
                .style(crate::theme::card_style),
            );
        }
    }

    left_col = left_col.push(Space::new().height(8));
    left_col = left_col.push(
        button(
            row![
                text("+").size(14).color(iced::Color::WHITE),
                text("Add Variable")
                    .size(12)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(iced::Color::WHITE),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::Environment(EnvironmentMessage::AddVariable))
        .padding([10, 18])
        .style(add_button_style),
    );

    let right_col = column![
        modal_section("\u{2139}", "COMMON VARIABLES", neon::SECONDARY),
        Space::new().height(4),
        container(
            column![
                info_text("Examples of commonly used environment variables:"),
                Space::new().height(8),
                text("DISPLAY")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("  X11 display (e.g., \":0\")")
                    .size(11)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().height(4),
                text("WAYLAND_DISPLAY")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("  Wayland display socket")
                    .size(11)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().height(4),
                text("XDG_CURRENT_DESKTOP")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("  Desktop environment name")
                    .size(11)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().height(4),
                text("GTK_THEME")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("  GTK theme (e.g., \"Adwaita:dark\")")
                    .size(11)
                    .color(neon::OUTLINE_VARIANT),
            ]
            .spacing(2),
        )
        .padding(12)
        .style(crate::theme::card_style),
    ]
    .spacing(6);

    row![
        left_col.width(Length::FillPortion(1)),
        right_col.width(Length::FillPortion(1)),
    ]
    .spacing(32)
    .align_y(Alignment::Start)
    .into()
}

// ── Styles ────────────────────────────────────────────────────────────────

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
