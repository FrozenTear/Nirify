//! Startup commands message handler

use crate::app::helpers::parse_spawn_command;
use crate::config::SettingsCategory;
use crate::messages::{StartupMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle startup commands messages
    pub(in crate::app) fn update_startup(&mut self, msg: StartupMessage) -> Task<Message> {
        
        let startup = &mut self.settings.startup;

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
                    // Use proper command parsing with quote handling and validation
                    match parse_spawn_command(&cmd) {
                        Ok(parsed) => {
                            if let Some(warning) = &parsed.warning {
                                log::warn!("Startup command {}: {}", id, warning);
                            }
                            command.command = if parsed.args.is_empty() {
                                vec![String::new()]
                            } else {
                                parsed.args
                            };
                        }
                        Err(e) => {
                            log::error!("Failed to parse startup command {}: {}", id, e);
                            return Task::none();
                        }
                    }
                }
            }
        }

        self.dirty_tracker.mark(SettingsCategory::Startup);
        self.mark_changed();
        Task::none()
    }
}
