//! Tools page message handler (IPC operations)

use crate::messages::{ToolsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle tools page messages (IPC operations)
    pub(in crate::app) fn update_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            // Query triggers - spawn async tasks
            ToolsMessage::RefreshWindows => {
                self.tools_state.loading_windows = true;
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_windows().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::WindowsLoaded(result)),
                )
            }
            ToolsMessage::RefreshWorkspaces => {
                self.tools_state.loading_workspaces = true;
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_workspaces().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::WorkspacesLoaded(result)),
                )
            }
            ToolsMessage::RefreshOutputs => {
                self.tools_state.loading_outputs = true;
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_full_outputs().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::OutputsLoaded(result)),
                )
            }
            ToolsMessage::RefreshFocusedWindow => {
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_focused_window().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::FocusedWindowLoaded(result)),
                )
            }
            ToolsMessage::RefreshVersion => {
                self.tools_state.loading_version = true;
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_version().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::VersionLoaded(result)),
                )
            }

            // Query results
            ToolsMessage::WindowsLoaded(result) => {
                self.tools_state.loading_windows = false;
                match result {
                    Ok(windows) => {
                        self.tools_state.windows = windows;
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::WorkspacesLoaded(result) => {
                self.tools_state.loading_workspaces = false;
                match result {
                    Ok(workspaces) => {
                        self.tools_state.workspaces = workspaces;
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::OutputsLoaded(result) => {
                self.tools_state.loading_outputs = false;
                match result {
                    Ok(outputs) => {
                        self.tools_state.outputs = outputs;
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::FocusedWindowLoaded(result) => {
                match result {
                    Ok(window) => {
                        self.tools_state.focused_window = window;
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::VersionLoaded(result) => {
                self.tools_state.loading_version = false;
                match result {
                    Ok(version) => {
                        self.tools_state.version = Some(version);
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }

            // Actions
            ToolsMessage::ReloadConfig => {
                self.tools_state.reloading = true;
                self.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::reload_config().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::ReloadCompleted(result)),
                )
            }
            ToolsMessage::ValidateConfig => {
                self.tools_state.validating = true;
                self.tools_state.last_error = None;
                self.tools_state.validation_result = None;
                Task::perform(
                    async { crate::ipc::validate_config().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::ValidateCompleted(result)),
                )
            }

            // Action results
            ToolsMessage::ReloadCompleted(result) => {
                self.tools_state.reloading = false;
                match result {
                    Ok(()) => {
                        self.toast = Some("Config reloaded successfully".to_string());
                        self.toast_shown_at = Some(std::time::Instant::now());
                    }
                    Err(e) => {
                        self.tools_state.last_error = Some(format!("Reload failed: {}", e));
                    }
                }
                Task::none()
            }
            ToolsMessage::ValidateCompleted(result) => {
                self.tools_state.validating = false;
                self.tools_state.validation_result = Some(result);
                Task::none()
            }
        }
    }
}
