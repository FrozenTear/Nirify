//! Workspaces settings message handler

use crate::config::SettingsCategory;
use crate::config::models::NamedWorkspace;
use crate::messages::{WorkspacesMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates workspaces settings
    pub(in crate::app) fn update_workspaces(&mut self, msg: WorkspacesMessage) -> Task<Message> {
        

        match msg {
            WorkspacesMessage::AddWorkspace => {
                let id = self.settings.workspaces.next_id;
                self.settings.workspaces.next_id += 1;

                let new_workspace = NamedWorkspace {
                    id,
                    name: format!("Workspace {}", self.settings.workspaces.workspaces.len() + 1),
                    open_on_output: None,
                    layout_override: None,
                };

                self.settings.workspaces.workspaces.push(new_workspace);
            }
            WorkspacesMessage::RemoveWorkspace(index) => {
                if index < self.settings.workspaces.workspaces.len() {
                    self.settings.workspaces.workspaces.remove(index);
                }
            }
            WorkspacesMessage::UpdateWorkspaceName(index, name) => {
                if let Some(workspace) = self.settings.workspaces.workspaces.get_mut(index) {
                    workspace.name = name;
                }
            }
            WorkspacesMessage::UpdateWorkspaceOutput(index, output) => {
                if let Some(workspace) = self.settings.workspaces.workspaces.get_mut(index) {
                    workspace.open_on_output = output;
                }
            }
            WorkspacesMessage::MoveWorkspaceUp(index) => {
                if index > 0 && index < self.settings.workspaces.workspaces.len() {
                    self.settings.workspaces.workspaces.swap(index - 1, index);
                }
            }
            WorkspacesMessage::MoveWorkspaceDown(index) => {
                if index < self.settings.workspaces.workspaces.len().saturating_sub(1) {
                    self.settings.workspaces.workspaces.swap(index, index + 1);
                }
            }
        }


        self.save.dirty_tracker.mark(SettingsCategory::Workspaces);
        self.mark_changed();

        Task::none()
    }
}
