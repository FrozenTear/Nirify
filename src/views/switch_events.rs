//! Switch events settings view
//!
//! Shows actions configured for laptop lid and tablet mode switches.

use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::SwitchEventsSettings;
use crate::messages::{Message, SwitchEventsMessage};
use crate::theme::{fonts, muted_text_container, code_text_container};

/// Creates the switch events settings view
pub fn view(settings: &SwitchEventsSettings) -> Element<'static, Message> {
    let lid_close = settings.lid_close.display();
    let lid_open = settings.lid_open.display();
    let tablet_on = settings.tablet_mode_on.display();
    let tablet_off = settings.tablet_mode_off.display();

    let content = column![
        page_title("Switch Events"),
        info_text(
            "Configure actions for laptop lid and tablet mode switches. \
             Enter the command to run when each event occurs."
        ),

        // Lid Events
        subsection_header("Lid Events"),
        card(column![
            command_input("Lid Close", &lid_close, "e.g., systemctl suspend", SwitchEventsMessage::SetLidCloseCommand),
            info_text("Command to run when the laptop lid is closed."),
            spacer(8.0),
            command_input("Lid Open", &lid_open, "e.g., notify-send \"Lid opened\"", SwitchEventsMessage::SetLidOpenCommand),
            info_text("Command to run when the laptop lid is opened."),
        ].spacing(4).width(Length::Fill).padding(12)),

        // Tablet Mode Events
        subsection_header("Tablet Mode Events"),
        card(column![
            command_input("Tablet Mode On", &tablet_on, "e.g., notify-send \"Tablet mode\"", SwitchEventsMessage::SetTabletModeOnCommand),
            info_text("Command to run when entering tablet mode."),
            spacer(8.0),
            command_input("Tablet Mode Off", &tablet_off, "e.g., notify-send \"Laptop mode\"", SwitchEventsMessage::SetTabletModeOffCommand),
            info_text("Command to run when exiting tablet mode."),
        ].spacing(4).width(Length::Fill).padding(12)),

        // Tips
        subsection_header("Tips"),
        card(column![
            info_text("Commands are split by whitespace. For complex commands, create a script and call it here."),
            spacer(4.0),
            container(text("Example commands:").size(13)).style(muted_text_container),
            container(text("  systemctl suspend").size(13).font(fonts::MONO_FONT)).style(code_text_container),
            container(text("  notify-send \"Message here\"").size(13).font(fonts::MONO_FONT)).style(code_text_container),
            container(text("  /home/user/scripts/on-lid-close.sh").size(13).font(fonts::MONO_FONT)).style(code_text_container),
        ].spacing(4).width(Length::Fill).padding(12)),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Create a command input row
fn command_input<F>(label: &str, value: &str, placeholder: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(String) -> SwitchEventsMessage + 'static,
{
    let value_owned = value.to_string();
    let placeholder_owned = placeholder.to_string();

    row![
        text(label.to_string()).size(14).width(Length::Fixed(150.0)),
        text_input(&placeholder_owned, &value_owned)
            .on_input(move |s| Message::SwitchEvents(msg_fn(s)))
            .padding(8)
            .font(fonts::MONO_FONT)
            .width(Length::Fill),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}
