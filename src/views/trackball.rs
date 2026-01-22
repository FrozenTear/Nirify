//! Trackball settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the trackball settings view
/// Note: This is a simplified version
/// Full trackball configuration UI will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Trackball Settings"),
        info_text(
            "Configure trackball behavior and acceleration. \
             Edit the trackball.kdl file directly."
        ),
        text("Trackball settings are defined in ~/.config/niri/niri-settings/input/trackball.kdl").size(14),
        spacer(16.0),
        section_header("Example Configuration"),
        text("input {").size(13),
        text("    trackball {").size(13),
        text("        accel-profile \"adaptive\"").size(13),
        text("        accel-speed 0.3").size(13),
        text("        scroll-method \"on-button-down\"").size(13),
        text("    }").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the trackball.kdl file directly for now. Full UI coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
