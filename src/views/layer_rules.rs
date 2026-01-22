//! Layer rules settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the layer rules settings view
/// Note: This is a simplified version
/// Full layer rules editor will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Layer Rules"),
        info_text(
            "Configure behavior for layer-shell surfaces (panels, docks, notifications). \
             Edit the layer-rules.kdl file directly."
        ),
        text("Layer rules are defined in ~/.config/niri/niri-settings/advanced/layer-rules.kdl").size(14),
        spacer(16.0),
        section_header("Example Rule"),
        text("layer-rule {").size(13),
        text("    match namespace=\"waybar\"").size(13),
        text("    block-out-from \"screencast\"").size(13),
        text("    block-out-from \"screen-capture\"").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the layer-rules.kdl file directly for now. Visual rule editor coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
