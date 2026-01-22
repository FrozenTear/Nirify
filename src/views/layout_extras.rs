//! Layout extras settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;

/// Creates the layout extras settings view
/// Note: This is a simplified version
/// Full layout extras UI will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Layout Extras"),
        info_text(
            "Advanced layout configuration including struts, focus settings, and window positioning. \
             Edit the layout-extras.kdl file directly."
        ),
        text("Layout extras are defined in ~/.config/niri/niri-settings/layout-extras.kdl").size(14),
        spacer(16.0),
        section_header("Example Configuration"),
        text("layout {").size(13),
        text("    struts {").size(13),
        text("        top 32").size(13),
        text("        bottom 32").size(13),
        text("    }").size(13),
        text("    focus-ring { enable false; }").size(13),
        text("}").size(13),
        spacer(16.0),
        info_text("Edit the layout-extras.kdl file directly for now. Full UI coming in a future update!"),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
