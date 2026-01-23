//! Gestures settings view
//!
//! Configure hot corners and drag-and-drop edge triggers.

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::GestureSettings;
use crate::messages::{GesturesMessage, Message};

/// Creates the gestures settings view
pub fn view(settings: &GestureSettings) -> Element<'static, Message> {
    // Clone values for closures
    let hot_corners_enabled = settings.hot_corners.enabled;
    let hot_corner_tl = settings.hot_corners.top_left;
    let hot_corner_tr = settings.hot_corners.top_right;
    let hot_corner_bl = settings.hot_corners.bottom_left;
    let hot_corner_br = settings.hot_corners.bottom_right;

    let dnd_scroll_enabled = settings.dnd_edge_view_scroll.enabled;
    let dnd_scroll_trigger = settings.dnd_edge_view_scroll.trigger_size;
    let dnd_scroll_delay = settings.dnd_edge_view_scroll.delay_ms;
    let dnd_scroll_speed = settings.dnd_edge_view_scroll.max_speed;

    let dnd_workspace_enabled = settings.dnd_edge_workspace_switch.enabled;
    let dnd_workspace_trigger = settings.dnd_edge_workspace_switch.trigger_size;
    let dnd_workspace_delay = settings.dnd_edge_workspace_switch.delay_ms;
    let dnd_workspace_speed = settings.dnd_edge_workspace_switch.max_speed;

    let content = column![
        page_title("Gestures"),
        info_text(
            "Configure hot corners and drag-and-drop edge triggers for workspace and view navigation."
        ),
        subsection_header("Hot Corners"),
        info_text("Trigger overview mode by moving the cursor to screen corners."),
        toggle_row(
            "Enable hot corners",
            "Allow triggering overview from screen corners",
            hot_corners_enabled,
            |value| Message::Gestures(GesturesMessage::SetHotCornersEnabled(value)),
        ),
        toggle_row(
            "Top Left",
            "Trigger from top-left corner",
            hot_corner_tl,
            |value| Message::Gestures(GesturesMessage::SetHotCornerTopLeft(value)),
        ),
        toggle_row(
            "Top Right",
            "Trigger from top-right corner",
            hot_corner_tr,
            |value| Message::Gestures(GesturesMessage::SetHotCornerTopRight(value)),
        ),
        toggle_row(
            "Bottom Left",
            "Trigger from bottom-left corner",
            hot_corner_bl,
            |value| Message::Gestures(GesturesMessage::SetHotCornerBottomLeft(value)),
        ),
        toggle_row(
            "Bottom Right",
            "Trigger from bottom-right corner",
            hot_corner_br,
            |value| Message::Gestures(GesturesMessage::SetHotCornerBottomRight(value)),
        ),
        subsection_header("DnD Edge View Scroll"),
        info_text("Scroll the view when dragging items to screen edges."),
        toggle_row(
            "Enable edge scroll",
            "Scroll view when dragging to left/right edges",
            dnd_scroll_enabled,
            |value| Message::Gestures(GesturesMessage::SetDndScrollEnabled(value)),
        ),
        slider_row_int(
            "Trigger width",
            "Edge zone width in pixels",
            dnd_scroll_trigger,
            10,
            200,
            " px",
            |value| Message::Gestures(GesturesMessage::SetDndScrollTriggerWidth(value)),
        ),
        slider_row_int(
            "Delay",
            "Delay before scroll starts",
            dnd_scroll_delay,
            0,
            2000,
            " ms",
            |value| Message::Gestures(GesturesMessage::SetDndScrollDelayMs(value)),
        ),
        slider_row_int(
            "Max speed",
            "Maximum scroll speed",
            dnd_scroll_speed,
            100,
            5000,
            " px/s",
            |value| Message::Gestures(GesturesMessage::SetDndScrollMaxSpeed(value)),
        ),
        subsection_header("DnD Edge Workspace Switch"),
        info_text("Switch workspaces when dragging items to screen top/bottom edges."),
        toggle_row(
            "Enable workspace switch",
            "Switch workspace when dragging to top/bottom edges",
            dnd_workspace_enabled,
            |value| Message::Gestures(GesturesMessage::SetDndWorkspaceEnabled(value)),
        ),
        slider_row_int(
            "Trigger height",
            "Edge zone height in pixels",
            dnd_workspace_trigger,
            10,
            200,
            " px",
            |value| Message::Gestures(GesturesMessage::SetDndWorkspaceTriggerHeight(value)),
        ),
        slider_row_int(
            "Delay",
            "Delay before workspace switch",
            dnd_workspace_delay,
            0,
            2000,
            " ms",
            |value| Message::Gestures(GesturesMessage::SetDndWorkspaceDelayMs(value)),
        ),
        slider_row_int(
            "Max speed",
            "Maximum switch speed",
            dnd_workspace_speed,
            100,
            5000,
            " px/s",
            |value| Message::Gestures(GesturesMessage::SetDndWorkspaceMaxSpeed(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}
