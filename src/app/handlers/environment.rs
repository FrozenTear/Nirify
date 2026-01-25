//! Environment settings message handler

use crate::config::SettingsCategory;
use crate::messages::{EnvironmentMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle environment settings messages
    pub(in crate::app) fn update_environment(&mut self, msg: EnvironmentMessage) -> Task<Message> {
        
        let env = &mut self.settings.environment;

        match msg {
            EnvironmentMessage::AddVariable => {
                let id = env.next_id;
                env.next_id += 1;
                env.variables.push(crate::config::models::EnvironmentVariable {
                    id,
                    name: String::new(),
                    value: String::new(),
                });
            }
            EnvironmentMessage::RemoveVariable(id) => {
                env.variables.retain(|v| v.id != id);
            }
            EnvironmentMessage::SetVariableName(id, name) => {
                if let Some(var) = env.variables.iter_mut().find(|v| v.id == id) {
                    var.name = name;
                }
            }
            EnvironmentMessage::SetVariableValue(id, value) => {
                if let Some(var) = env.variables.iter_mut().find(|v| v.id == id) {
                    var.value = value;
                }
            }
        }

        self.save.dirty_tracker.mark(SettingsCategory::Environment);
        self.mark_changed();
        Task::none()
    }
}
