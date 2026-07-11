//! Gestures settings message handler

use crate::config::SettingsCategory;
use crate::messages::{GesturesMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle gestures settings messages
    pub(in crate::app) fn update_gestures(&mut self, msg: GesturesMessage) -> Task<Message> {
        let gestures = &mut self.settings.gestures;

        match msg {
            // Hot corners
            GesturesMessage::SetHotCornersEnabled(v) => {
                gestures.hot_corners.enabled = v;
                // niri needs at least one corner when enabled; default to top-left.
                if v && !hot_corners_any_selected(&gestures.hot_corners) {
                    gestures.hot_corners.top_left = true;
                }
            }
            GesturesMessage::SetHotCornerTopLeft(v) => {
                gestures.hot_corners.top_left = v;
                sync_hot_corners_enabled(&mut gestures.hot_corners);
            }
            GesturesMessage::SetHotCornerTopRight(v) => {
                gestures.hot_corners.top_right = v;
                sync_hot_corners_enabled(&mut gestures.hot_corners);
            }
            GesturesMessage::SetHotCornerBottomLeft(v) => {
                gestures.hot_corners.bottom_left = v;
                sync_hot_corners_enabled(&mut gestures.hot_corners);
            }
            GesturesMessage::SetHotCornerBottomRight(v) => {
                gestures.hot_corners.bottom_right = v;
                sync_hot_corners_enabled(&mut gestures.hot_corners);
            }

            // DnD edge view scroll
            GesturesMessage::SetDndScrollEnabled(v) => gestures.dnd_edge_view_scroll.enabled = v,
            GesturesMessage::SetDndScrollTriggerWidth(v) => {
                gestures.dnd_edge_view_scroll.trigger_size = v.clamp(10, 200)
            }
            GesturesMessage::SetDndScrollDelayMs(v) => {
                gestures.dnd_edge_view_scroll.delay_ms = v.clamp(0, 2000)
            }
            GesturesMessage::SetDndScrollMaxSpeed(v) => {
                gestures.dnd_edge_view_scroll.max_speed = v.clamp(100, 5000)
            }

            // DnD edge workspace switch
            GesturesMessage::SetDndWorkspaceEnabled(v) => {
                gestures.dnd_edge_workspace_switch.enabled = v
            }
            GesturesMessage::SetDndWorkspaceTriggerHeight(v) => {
                gestures.dnd_edge_workspace_switch.trigger_size = v.clamp(10, 200)
            }
            GesturesMessage::SetDndWorkspaceDelayMs(v) => {
                gestures.dnd_edge_workspace_switch.delay_ms = v.clamp(0, 2000)
            }
            GesturesMessage::SetDndWorkspaceMaxSpeed(v) => {
                gestures.dnd_edge_workspace_switch.max_speed = v.clamp(100, 5000)
            }
        }

        self.save.dirty_tracker.mark(SettingsCategory::Gestures);
        self.mark_changed();
        Task::none()
    }
}

/// Returns true if any hot corner is selected.
fn hot_corners_any_selected(hc: &crate::config::models::HotCorners) -> bool {
    hc.top_left || hc.top_right || hc.bottom_left || hc.bottom_right
}

/// Keeps "enabled with zero corners" unrepresentable: unchecking the last
/// selected corner disables hot corners entirely.
fn sync_hot_corners_enabled(hc: &mut crate::config::models::HotCorners) {
    hc.enabled = hot_corners_any_selected(hc);
}
