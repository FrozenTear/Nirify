//! Animations settings view (simplified for Phase 6)

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;
use crate::theme::fonts;

/// Creates the animations settings view
/// Note: This is a simplified version deferred from Phase 5
/// Full animation configuration UI with 20+ animation types will be added in a future phase
pub fn view() -> Element<'static, Message> {
    let content = column![
        section_header("Animations"),
        info_text(
            "Configure niri's animations including workspace switching, window opening/closing, and more. \
             This is a complex feature with 20+ animation types that will receive a comprehensive UI in a future update."
        ),
        text("Animations are defined in ~/.config/niri/niri-settings/animations.kdl").size(14).font(fonts::MONO_FONT),
        spacer(16.0),
        section_header("Example Configuration"),
        text("animations {").size(13).font(fonts::MONO_FONT),
        text("    workspace-switch {").size(13).font(fonts::MONO_FONT),
        text("        spring damping-ratio=1.0 stiffness=800 epsilon=0.0001").size(13).font(fonts::MONO_FONT),
        text("    }").size(13).font(fonts::MONO_FONT),
        text("    window-open {").size(13).font(fonts::MONO_FONT),
        text("        duration-ms 150").size(13).font(fonts::MONO_FONT),
        text("        curve \"ease-out-cubic\"").size(13).font(fonts::MONO_FONT),
        text("    }").size(13).font(fonts::MONO_FONT),
        text("}").size(13).font(fonts::MONO_FONT),
        spacer(16.0),
        info_text(
            "Note: Animations support is extensive with many configuration options. \
             Edit the animations.kdl file directly for now. A comprehensive model-driven UI \
             is planned for a future update!"
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
