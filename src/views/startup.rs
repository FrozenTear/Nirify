//! Startup commands settings view — neon modal style

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::info_text;
use crate::config::models::StartupSettings;
use crate::messages::{Message, StartupMessage};
use crate::theme::{fonts, neon};

/// Creates the startup commands settings view (with scrollable wrapper)
pub fn view(settings: &StartupSettings) -> Element<'static, Message> {
    let content = column![view_section(settings),]
        .spacing(0)
        .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Inner content without scrollable wrapper
pub fn view_section(settings: &StartupSettings) -> Element<'static, Message> {
    let commands = settings.commands.clone();

    let mut left_col = column![
        modal_section("\u{25B6}", "STARTUP COMMANDS", neon::PRIMARY),
        info_text("Commands run automatically when niri starts."),
        Space::new().height(4),
    ]
    .spacing(6);

    if commands.is_empty() {
        left_col = left_col.push(
            container(
                column![
                    text("No startup commands configured")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                    Space::new().height(4),
                    text("Click the button below to add your first command")
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
        for cmd in &commands {
            let cmd_id = cmd.id;
            let cmd_display = cmd.display();

            left_col = left_col.push(
                container(
                    column![
                        row![
                            text(format!("#{}", cmd_id))
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().width(Length::Fill),
                            button(text("\u{00D7}").size(14).color(neon::ERROR))
                                .on_press(Message::Startup(StartupMessage::RemoveCommand(cmd_id)))
                                .padding([2, 8])
                                .style(delete_button_style),
                        ]
                        .align_y(Alignment::Center),
                        text_input("e.g., waybar", &cmd_display)
                            .on_input(move |s| Message::Startup(StartupMessage::SetCommand(
                                cmd_id, s
                            )))
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

    left_col = left_col.push(Space::new().height(8));
    left_col = left_col.push(
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
        .on_press(Message::Startup(StartupMessage::AddCommand))
        .padding([10, 18])
        .style(add_button_style),
    );

    let right_col = column![
        modal_section("\u{2139}", "TIPS", neon::SECONDARY),
        Space::new().height(4),
        container(
            column![
                info_text(
                    "Commands are split by whitespace. For complex commands, create a script."
                ),
                Space::new().height(8),
                text("EXAMPLES")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().height(4),
                text("waybar")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("mako")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
                text("/home/user/scripts/startup.sh")
                    .size(12)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .spacing(4),
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
