//! Gestures settings view
//!
//! Configure hot corners and drag-and-drop edge triggers.

use iced::widget::{column, container, scrollable, text};
use iced::Element;

use super::widgets::*;
use crate::config::models::GestureSettings;
use crate::messages::Message;

/// Creates the gestures settings view
pub fn view(settings: &GestureSettings) -> Element<'static, Message> {
    let content = column![
        section_header("Gestures"),
        info_text(
            "Configure hot corners and drag-and-drop edge triggers for workspace and view navigation."
        ),
        spacer(16.0),

        // Hot Corners Section
        subsection_header("Hot Corners"),
        info_text("Trigger overview mode by moving the cursor to screen corners."),
        spacer(8.0),
        display_toggle("Enabled", settings.hot_corners.enabled),
        display_toggle("Top Left", settings.hot_corners.top_left),
        display_toggle("Top Right", settings.hot_corners.top_right),
        display_toggle("Bottom Left", settings.hot_corners.bottom_left),
        display_toggle("Bottom Right", settings.hot_corners.bottom_right),
        spacer(16.0),

        // DnD Edge View Scroll Section
        subsection_header("DnD Edge View Scroll"),
        info_text("Scroll the view when dragging items to screen edges."),
        spacer(8.0),
        display_toggle("Enabled", settings.dnd_edge_view_scroll.enabled),
        display_value("Trigger Width", &format!("{} px", settings.dnd_edge_view_scroll.trigger_size)),
        display_value("Delay", &format!("{} ms", settings.dnd_edge_view_scroll.delay_ms)),
        display_value("Max Speed", &format!("{} px/s", settings.dnd_edge_view_scroll.max_speed)),
        spacer(16.0),

        // DnD Edge Workspace Switch Section
        subsection_header("DnD Edge Workspace Switch"),
        info_text("Switch workspaces when dragging items to screen top/bottom edges."),
        spacer(8.0),
        display_toggle("Enabled", settings.dnd_edge_workspace_switch.enabled),
        display_value("Trigger Height", &format!("{} px", settings.dnd_edge_workspace_switch.trigger_size)),
        display_value("Delay", &format!("{} ms", settings.dnd_edge_workspace_switch.delay_ms)),
        display_value("Max Speed", &format!("{} px/s", settings.dnd_edge_workspace_switch.max_speed)),
        spacer(16.0),

        info_text("Full gesture editing UI coming in a future update. Edit gestures.kdl directly for now."),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}

/// Display a toggle value (read-only for now)
fn display_toggle(label: &str, value: bool) -> Element<'static, Message> {
    use iced::widget::row;
    use iced::Length;

    row![
        text(label.to_string()).size(14).width(Length::Fixed(150.0)),
        text(if value { "Yes" } else { "No" })
            .size(14)
            .color(if value { [0.5, 0.8, 0.5] } else { [0.6, 0.6, 0.6] }),
    ]
    .spacing(16)
    .into()
}

/// Display a value (read-only)
fn display_value(label: &str, value: &str) -> Element<'static, Message> {
    use iced::widget::row;
    use iced::Length;

    row![
        text(label.to_string()).size(14).width(Length::Fixed(150.0)),
        text(value.to_string()).size(14).color([0.8, 0.8, 0.8]),
    ]
    .spacing(16)
    .into()
}
