//! Behavior settings message handler

use crate::config::SettingsCategory;
use crate::messages::{BehaviorMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates behavior settings
    pub(in crate::app) fn update_behavior(&mut self, msg: BehaviorMessage) -> Task<Message> {
        

        match msg {
            BehaviorMessage::ToggleFocusFollowsMouse(value) => {
                self.settings.behavior.focus_follows_mouse = value;
            }
            BehaviorMessage::SetFocusFollowsMouseMaxScroll(value) => {
                self.settings.behavior.focus_follows_mouse_max_scroll_amount = value.map(|v| v.clamp(0.0, 100.0));
            }
            BehaviorMessage::SetWarpMouseToFocus(mode) => {
                self.settings.behavior.warp_mouse_to_focus = mode;
            }
            BehaviorMessage::ToggleWorkspaceAutoBackAndForth(value) => {
                self.settings.behavior.workspace_auto_back_and_forth = value;
            }
            BehaviorMessage::ToggleAlwaysCenterSingleColumn(value) => {
                self.settings.behavior.always_center_single_column = value;
            }
            BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(value) => {
                self.settings.behavior.empty_workspace_above_first = value;
            }
            BehaviorMessage::SetCenterFocusedColumn(mode) => {
                self.settings.behavior.center_focused_column = mode;
            }
            BehaviorMessage::SetDefaultColumnWidthType(width_type) => {
                self.settings.behavior.default_column_width_type = width_type;
            }
            BehaviorMessage::SetStrutLeft(value) => {
                self.settings.behavior.strut_left = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutRight(value) => {
                self.settings.behavior.strut_right = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutTop(value) => {
                self.settings.behavior.strut_top = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutBottom(value) => {
                self.settings.behavior.strut_bottom = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetModKey(key) => {
                self.settings.behavior.mod_key = key;
            }
            BehaviorMessage::SetModKeyNested(key) => {
                self.settings.behavior.mod_key_nested = key;
            }
            BehaviorMessage::ToggleDisablePowerKeyHandling(value) => {
                self.settings.behavior.disable_power_key_handling = value;
            }
        }


        self.dirty_tracker.mark(SettingsCategory::Behavior);
        self.mark_changed();

        Task::none()
    }
}
