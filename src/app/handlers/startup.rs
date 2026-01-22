//! Startup commands message handler

use crate::config::SettingsCategory;
use crate::messages::{StartupMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle startup commands messages
    pub(in crate::app) fn update_startup(&mut self, msg: StartupMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let startup = &mut settings.startup;

        match msg {
            StartupMessage::AddCommand => {
                let id = startup.next_id;
                startup.next_id += 1;
                startup.commands.push(crate::config::models::StartupCommand {
                    id,
                    command: vec![String::new()],
                });
            }
            StartupMessage::RemoveCommand(id) => {
                startup.commands.retain(|c| c.id != id);
            }
            StartupMessage::SetCommand(id, cmd) => {
                if let Some(command) = startup.commands.iter_mut().find(|c| c.id == id) {
                    // Split by whitespace for the command vector
                    command.command = cmd.split_whitespace().map(String::from).collect();
                    if command.command.is_empty() {
                        command.command.push(String::new());
                    }
                }
            }
        }

        drop(settings);
        self.dirty_tracker.mark(SettingsCategory::Startup);
        self.save_manager.mark_changed();
        Task::none()
    }
}
