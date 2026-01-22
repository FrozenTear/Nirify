//! Switch events settings view
//!
//! Shows actions configured for laptop lid and tablet mode switches.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::SwitchEventsSettings;
use crate::messages::Message;
use crate::theme::fonts;

/// Creates the switch events settings view
pub fn view(settings: &SwitchEventsSettings) -> Element<'static, Message> {
    let content = column![
        section_header("Switch Events"),
        info_text(
            "Configure actions for laptop lid and tablet mode switches."
        ),
        spacer(16.0),

        // Lid Events
        subsection_header("Lid Events"),
        display_event("Lid Close", &settings.lid_close.display(), settings.lid_close.has_action()),
        display_event("Lid Open", &settings.lid_open.display(), settings.lid_open.has_action()),
        spacer(16.0),

        // Tablet Mode Events
        subsection_header("Tablet Mode Events"),
        display_event("Tablet Mode On", &settings.tablet_mode_on.display(), settings.tablet_mode_on.has_action()),
        display_event("Tablet Mode Off", &settings.tablet_mode_off.display(), settings.tablet_mode_off.has_action()),
        spacer(16.0),

        // Example Configuration
        subsection_header("Example Configuration"),
        spacer(8.0),
        text("binds {").size(13).font(fonts::MONO_FONT),
        text("    Switch-Lid-Close { spawn \"systemctl\" \"suspend\"; }").size(13).font(fonts::MONO_FONT),
        text("    Switch-Lid-Open { spawn \"notify-send\" \"Lid opened\"; }").size(13).font(fonts::MONO_FONT),
        text("    Switch-Tablet-Mode-On { spawn \"notify-send\" \"Tablet mode\"; }").size(13).font(fonts::MONO_FONT),
        text("    Switch-Tablet-Mode-Off { spawn \"notify-send\" \"Laptop mode\"; }").size(13).font(fonts::MONO_FONT),
        text("}").size(13).font(fonts::MONO_FONT),
        spacer(16.0),

        info_text("Edit switch-events.kdl directly to configure these actions. Full editing UI coming in a future update."),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}

/// Display a switch event action (read-only)
fn display_event(label: &str, command: &str, has_action: bool) -> Element<'static, Message> {
    let display_text = if has_action {
        command.to_string()
    } else {
        "No action configured".to_string()
    };

    row![
        text(label.to_string()).size(14).width(Length::Fixed(150.0)),
        text(display_text)
            .size(14)
            .font(fonts::MONO_FONT)
            .color(if has_action { [0.7, 0.85, 0.7] } else { [0.5, 0.5, 0.5] }),
    ]
    .spacing(16)
    .into()
}
