//! Behavior settings view

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::BehaviorSettings;
use crate::config::ColumnWidthType;
use crate::messages::{BehaviorMessage, Message};
use crate::types::{CenterFocusedColumn, ModKey, WarpMouseMode};

/// Creates the behavior settings view
pub fn view(settings: &BehaviorSettings) -> Element<'_, Message> {
    let focus_follows_mouse = settings.focus_follows_mouse;
    let focus_max_scroll = settings.focus_follows_mouse_max_scroll_amount;
    let warp_mouse_to_focus = settings.warp_mouse_to_focus;
    let workspace_auto_back_and_forth = settings.workspace_auto_back_and_forth;
    let always_center_single_column = settings.always_center_single_column;
    let empty_workspace_above_first = settings.empty_workspace_above_first;
    let center_focused_column = settings.center_focused_column;
    let default_column_width_type = settings.default_column_width_type;
    let disable_power_key_handling = settings.disable_power_key_handling;
    let strut_left = settings.strut_left;
    let strut_right = settings.strut_right;
    let strut_top = settings.strut_top;
    let strut_bottom = settings.strut_bottom;
    let mod_key = settings.mod_key;
    let mod_key_nested = settings.mod_key_nested;

    let content = column![
        section_header("Focus Behavior"),
        info_text(
            "Control how window focus behaves when moving your mouse."
        ),
        toggle_row(
            "Focus follows mouse",
            "Automatically focus windows when hovering over them",
            focus_follows_mouse,
            |value| Message::Behavior(BehaviorMessage::ToggleFocusFollowsMouse(value)),
        ),
        optional_slider_row(
            "Max scroll amount",
            "Limit viewport scrolling when focus-follows-mouse triggers (% of window size)",
            focus_max_scroll,
            0.0,
            100.0,
            "%",
            |value| Message::Behavior(BehaviorMessage::SetFocusFollowsMouseMaxScroll(value)),
        ),
        picker_row(
            "Warp mouse to focused window",
            "Automatically move mouse pointer to newly focused windows",
            WarpMouseMode::all(),
            Some(warp_mouse_to_focus),
            |value| Message::Behavior(BehaviorMessage::SetWarpMouseToFocus(value)),
        ),
        spacer(16.0),

        section_header("Workspace Behavior"),
        toggle_row(
            "Auto back-and-forth",
            "Switching to the current workspace switches to the previous one",
            workspace_auto_back_and_forth,
            |value| Message::Behavior(BehaviorMessage::ToggleWorkspaceAutoBackAndForth(value)),
        ),
        toggle_row(
            "Always center single column",
            "Center windows when only one column is present",
            always_center_single_column,
            |value| Message::Behavior(BehaviorMessage::ToggleAlwaysCenterSingleColumn(value)),
        ),
        toggle_row(
            "Empty workspace above first",
            "Add an empty workspace above workspace 1",
            empty_workspace_above_first,
            |value| Message::Behavior(BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(value)),
        ),
        picker_row(
            "Center focused column",
            "When to automatically center the focused column in viewport",
            CenterFocusedColumn::all(),
            Some(center_focused_column),
            |value| Message::Behavior(BehaviorMessage::SetCenterFocusedColumn(value)),
        ),
        spacer(16.0),

        section_header("Default Column Width"),
        info_text(
            "Choose how new window columns are sized by default."
        ),
        picker_row(
            "Width type",
            "Proportion (relative to screen) or Fixed (absolute pixels)",
            ColumnWidthType::all(),
            Some(default_column_width_type),
            |value| Message::Behavior(BehaviorMessage::SetDefaultColumnWidthType(value)),
        ),
        spacer(16.0),

        section_header("Screen Edge Struts"),
        info_text(
            "Reserve space at screen edges (useful for panels/docks). Values in pixels."
        ),
        slider_row(
            "Left strut",
            "Reserved space on left edge",
            strut_left,
            0.0,
            200.0,
            " px",
            |value| Message::Behavior(BehaviorMessage::SetStrutLeft(value)),
        ),
        slider_row(
            "Right strut",
            "Reserved space on right edge",
            strut_right,
            0.0,
            200.0,
            " px",
            |value| Message::Behavior(BehaviorMessage::SetStrutRight(value)),
        ),
        slider_row(
            "Top strut",
            "Reserved space on top edge",
            strut_top,
            0.0,
            200.0,
            " px",
            |value| Message::Behavior(BehaviorMessage::SetStrutTop(value)),
        ),
        slider_row(
            "Bottom strut",
            "Reserved space on bottom edge",
            strut_bottom,
            0.0,
            200.0,
            " px",
            |value| Message::Behavior(BehaviorMessage::SetStrutBottom(value)),
        ),
        spacer(16.0),

        section_header("Modifier Keys"),
        info_text(
            "Choose the primary modifier key for niri window management shortcuts."
        ),
        picker_row(
            "Modifier key",
            "Primary modifier key for compositor shortcuts (usually Super/Win key)",
            ModKey::all(),
            Some(mod_key),
            |value| Message::Behavior(BehaviorMessage::SetModKey(value)),
        ),
        optional_picker_row(
            "Nested modifier key",
            "Override modifier key when niri runs nested inside another compositor",
            ModKey::all(),
            mod_key_nested,
            |value| Message::Behavior(BehaviorMessage::SetModKeyNested(value)),
        ),
        spacer(16.0),

        section_header("Power Button"),
        toggle_row(
            "Disable power key handling",
            "Let the system handle power button instead of niri",
            disable_power_key_handling,
            |value| Message::Behavior(BehaviorMessage::ToggleDisablePowerKeyHandling(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
