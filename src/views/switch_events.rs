//! Switch events settings view
//!
//! Shows actions configured for laptop lid and tablet mode switches.

use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::SwitchEventsSettings;
use crate::messages::{Message, SwitchEventsMessage};
use crate::theme::fonts;

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
        spacer(16.0),

        // Lid Events
        subsection_header("Lid Events"),
        command_input("Lid Close", &lid_close, "e.g., systemctl suspend", SwitchEventsMessage::SetLidCloseCommand),
        info_text("Command to run when the laptop lid is closed."),
        spacer(8.0),
        command_input("Lid Open", &lid_open, "e.g., notify-send \"Lid opened\"", SwitchEventsMessage::SetLidOpenCommand),
        info_text("Command to run when the laptop lid is opened."),
        spacer(16.0),

        // Tablet Mode Events
        subsection_header("Tablet Mode Events"),
        command_input("Tablet Mode On", &tablet_on, "e.g., notify-send \"Tablet mode\"", SwitchEventsMessage::SetTabletModeOnCommand),
        info_text("Command to run when entering tablet mode."),
        spacer(8.0),
        command_input("Tablet Mode Off", &tablet_off, "e.g., notify-send \"Laptop mode\"", SwitchEventsMessage::SetTabletModeOffCommand),
        info_text("Command to run when exiting tablet mode."),
        spacer(16.0),

        // Tips
        subsection_header("Tips"),
        spacer(8.0),
        info_text("Commands are split by whitespace. For complex commands, create a script and call it here."),
        spacer(4.0),
        text("Example commands:").size(13).color([0.7, 0.7, 0.7]),
        text("  systemctl suspend").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]),
        text("  notify-send \"Message here\"").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]),
        text("  /home/user/scripts/on-lid-close.sh").size(13).font(fonts::MONO_FONT).color([0.7, 0.85, 0.7]),
        spacer(16.0),
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
