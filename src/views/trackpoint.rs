//! Trackpoint settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the trackpoint settings view
/// Note: This is a simplified version
/// Full trackpoint configuration UI will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Trackpoint Settings"),
        info_text(
            "Configure trackpoint (pointing stick) behavior. \
             Edit the trackpoint.kdl file directly."
        ),
        text("Trackpoint settings are defined in ~/.config/niri/niri-settings/input/trackpoint.kdl").size(14),
        spacer(16.0),
        section_header("Example Configuration"),
        text("input {").size(13),
        text("    trackpoint {").size(13),
        text("        accel-profile \"flat\"").size(13),
        text("        accel-speed 0.5").size(13),
        text("    }").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the trackpoint.kdl file directly for now. Full UI coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
