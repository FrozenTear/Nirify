//! Switch events settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::info_text;
use crate::config::models::SwitchEventsSettings;
use crate::messages::{Message, SwitchEventsMessage};
use crate::theme::{fonts, neon};

/// Creates the switch events settings view (with scrollable wrapper)
pub fn view(settings: &SwitchEventsSettings) -> Element<'static, Message> {
    let content = column![view_section(settings),]
        .spacing(0)
        .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Inner content without scrollable wrapper
pub fn view_section(settings: &SwitchEventsSettings) -> Element<'static, Message> {
    let lid_close = settings.lid_close.display();
    let lid_open = settings.lid_open.display();
    let tablet_on = settings.tablet_mode_on.display();
    let tablet_off = settings.tablet_mode_off.display();

    let content = column![row![
        // Left: Lid Events
        column![
            modal_section("\u{25A3}", "LID EVENTS", neon::PRIMARY),
            info_text("Commands to run when the laptop lid opens or closes."),
            Space::new().height(4),
            styled_text_input("LID CLOSE", "e.g., systemctl suspend", &lid_close, |s| {
                Message::SwitchEvents(SwitchEventsMessage::SetLidCloseCommand(s))
            },),
            styled_text_input(
                "LID OPEN",
                "e.g., notify-send \"Lid opened\"",
                &lid_open,
                |s| Message::SwitchEvents(SwitchEventsMessage::SetLidOpenCommand(s)),
            ),
        ]
        .spacing(6)
        .width(Length::FillPortion(1)),
        // Right: Tablet Mode Events
        column![
            modal_section("\u{25CE}", "TABLET MODE", neon::SECONDARY),
            info_text("Commands to run when entering or exiting tablet mode."),
            Space::new().height(4),
            styled_text_input(
                "TABLET MODE ON",
                "e.g., notify-send \"Tablet mode\"",
                &tablet_on,
                |s| Message::SwitchEvents(SwitchEventsMessage::SetTabletModeOnCommand(s)),
            ),
            styled_text_input(
                "TABLET MODE OFF",
                "e.g., notify-send \"Laptop mode\"",
                &tablet_off,
                |s| Message::SwitchEvents(SwitchEventsMessage::SetTabletModeOffCommand(s)),
            ),
            Space::new().height(12),
            modal_section("\u{2139}", "TIPS", neon::TERTIARY),
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
                    text("systemctl suspend")
                        .size(12)
                        .font(fonts::MONO_FONT)
                        .color(neon::SECONDARY),
                    text("notify-send \"Message here\"")
                        .size(12)
                        .font(fonts::MONO_FONT)
                        .color(neon::SECONDARY),
                    text("/home/user/scripts/on-lid-close.sh")
                        .size(12)
                        .font(fonts::MONO_FONT)
                        .color(neon::SECONDARY),
                ]
                .spacing(2),
            )
            .padding(12)
            .style(crate::theme::card_style),
        ]
        .spacing(6)
        .width(Length::FillPortion(1)),
    ]
    .spacing(32)
    .align_y(Alignment::Start),]
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
