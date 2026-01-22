//! Behavior settings message handler

use crate::config::SettingsCategory;
use crate::messages::{BehaviorMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates behavior settings
    pub(in crate::app) fn update_behavior(&mut self, msg: BehaviorMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            BehaviorMessage::ToggleFocusFollowsMouse(value) => {
                settings.behavior.focus_follows_mouse = value;
            }
            BehaviorMessage::SetWarpMouseToFocus(mode) => {
                settings.behavior.warp_mouse_to_focus = mode;
            }
            BehaviorMessage::ToggleWorkspaceAutoBackAndForth(value) => {
                settings.behavior.workspace_auto_back_and_forth = value;
            }
            BehaviorMessage::ToggleAlwaysCenterSingleColumn(value) => {
                settings.behavior.always_center_single_column = value;
            }
            BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(value) => {
                settings.behavior.empty_workspace_above_first = value;
            }
            BehaviorMessage::SetCenterFocusedColumn(mode) => {
                settings.behavior.center_focused_column = mode;
            }
            BehaviorMessage::SetDefaultColumnWidthType(width_type) => {
                settings.behavior.default_column_width_type = width_type;
            }
            BehaviorMessage::SetStrutLeft(value) => {
                settings.behavior.strut_left = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutRight(value) => {
                settings.behavior.strut_right = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutTop(value) => {
                settings.behavior.strut_top = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutBottom(value) => {
                settings.behavior.strut_bottom = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetModKey(key) => {
                settings.behavior.mod_key = key;
            }
            BehaviorMessage::SetModKeyNested(key) => {
                settings.behavior.mod_key_nested = key;
            }
            BehaviorMessage::ToggleDisablePowerKeyHandling(value) => {
                settings.behavior.disable_power_key_handling = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Behavior);
        self.save_manager.mark_changed();

        Task::none()
    }
}
