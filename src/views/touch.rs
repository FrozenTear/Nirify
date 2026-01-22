//! Touch settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the touch settings view
/// Note: This is a simplified version
/// Full touchscreen configuration UI will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Touch Settings"),
        info_text(
            "Configure touchscreen behavior and mapping. \
             Edit the touch.kdl file directly."
        ),
        text("Touch settings are defined in ~/.config/niri/niri-settings/input/touch.kdl").size(14),
        spacer(16.0),
        section_header("Example Configuration"),
        text("input {").size(13),
        text("    touch {").size(13),
        text("        map-to-output \"eDP-1\"").size(13),
        text("        tap enabled").size(13),
        text("    }").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the touch.kdl file directly for now. Full UI coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
