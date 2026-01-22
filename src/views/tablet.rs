//! Tablet settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the tablet settings view
/// Note: This is a simplified version
/// Full tablet configuration UI will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Tablet Settings"),
        info_text(
            "Configure graphics tablet and pen input behavior. \
             Edit the tablet.kdl file directly."
        ),
        text("Tablet settings are defined in ~/.config/niri/niri-settings/input/tablet.kdl").size(14),
        spacer(16.0),
        section_header("Example Configuration"),
        text("input {").size(13),
        text("    tablet {").size(13),
        text("        map-to-output \"eDP-1\"").size(13),
        text("        left-handed false").size(13),
        text("    }").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the tablet.kdl file directly for now. Full UI coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
