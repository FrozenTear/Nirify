//! Workspaces settings message handler

use crate::config::SettingsCategory;
use crate::config::models::NamedWorkspace;
use crate::messages::{WorkspacesMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates workspaces settings
    pub(in crate::app) fn update_workspaces(&mut self, msg: WorkspacesMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            WorkspacesMessage::AddWorkspace => {
                let id = settings.workspaces.next_id;
                settings.workspaces.next_id += 1;

                let new_workspace = NamedWorkspace {
                    id,
                    name: format!("Workspace {}", settings.workspaces.workspaces.len() + 1),
                    open_on_output: None,
                    layout_override: None,
                };

                settings.workspaces.workspaces.push(new_workspace);
            }
            WorkspacesMessage::RemoveWorkspace(index) => {
                if index < settings.workspaces.workspaces.len() {
                    settings.workspaces.workspaces.remove(index);
                }
            }
            WorkspacesMessage::UpdateWorkspaceName(index, name) => {
                if let Some(workspace) = settings.workspaces.workspaces.get_mut(index) {
                    workspace.name = name;
                }
            }
            WorkspacesMessage::UpdateWorkspaceOutput(index, output) => {
                if let Some(workspace) = settings.workspaces.workspaces.get_mut(index) {
                    workspace.open_on_output = output;
                }
            }
            WorkspacesMessage::MoveWorkspaceUp(index) => {
                if index > 0 && index < settings.workspaces.workspaces.len() {
                    settings.workspaces.workspaces.swap(index - 1, index);
                }
            }
            WorkspacesMessage::MoveWorkspaceDown(index) => {
                if index < settings.workspaces.workspaces.len().saturating_sub(1) {
                    settings.workspaces.workspaces.swap(index, index + 1);
                }
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Workspaces);
        self.save_manager.mark_changed();

        Task::none()
    }
}
