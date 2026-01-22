//! Switch events settings message handler

use crate::config::SettingsCategory;
use crate::messages::{SwitchEventsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle switch events settings messages
    pub(in crate::app) fn update_switch_events(&mut self, msg: SwitchEventsMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let switch = &mut settings.switch_events;

        // Helper to parse command string into Vec<String>
        fn parse_command(cmd: &str) -> Vec<String> {
            if cmd.trim().is_empty() {
                Vec::new()
            } else {
                // Simple split by whitespace - could be enhanced with proper shell parsing
                cmd.split_whitespace().map(String::from).collect()
            }
        }

        match msg {
            SwitchEventsMessage::SetLidCloseCommand(cmd) => {
                switch.lid_close.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetLidOpenCommand(cmd) => {
                switch.lid_open.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetTabletModeOnCommand(cmd) => {
                switch.tablet_mode_on.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetTabletModeOffCommand(cmd) => {
                switch.tablet_mode_off.spawn = parse_command(&cmd);
            }
        }

        drop(settings);
        self.dirty_tracker.mark(SettingsCategory::SwitchEvents);
        self.save_manager.mark_changed();
        Task::none()
    }
}
