//! Tools page message handler (IPC operations)

use crate::messages::{ToolsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle tools page messages (IPC operations)
    pub(in crate::app) fn update_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            // Query triggers - spawn async tasks
            ToolsMessage::RefreshWindows => {
                self.ui.tools_state.loading_windows = true;
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_windows().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::WindowsLoaded(result)),
                )
            }
            ToolsMessage::RefreshWorkspaces => {
                self.ui.tools_state.loading_workspaces = true;
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_workspaces().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::WorkspacesLoaded(result)),
                )
            }
            ToolsMessage::RefreshOutputs => {
                self.ui.tools_state.loading_outputs = true;
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_full_outputs().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::OutputsLoaded(result)),
                )
            }
            ToolsMessage::RefreshFocusedWindow => {
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_focused_window().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::FocusedWindowLoaded(result)),
                )
            }
            ToolsMessage::RefreshVersion => {
                self.ui.tools_state.loading_version = true;
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::get_version().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::VersionLoaded(result)),
                )
            }

            // Query results
            ToolsMessage::WindowsLoaded(result) => {
                self.ui.tools_state.loading_windows = false;
                match result {
                    Ok(windows) => {
                        self.ui.tools_state.windows = windows;
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::WorkspacesLoaded(result) => {
                self.ui.tools_state.loading_workspaces = false;
                match result {
                    Ok(workspaces) => {
                        self.ui.tools_state.workspaces = workspaces;
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::OutputsLoaded(result) => {
                self.ui.tools_state.loading_outputs = false;
                match result {
                    Ok(outputs) => {
                        self.ui.tools_state.outputs = outputs;
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::FocusedWindowLoaded(result) => {
                match result {
                    Ok(window) => {
                        self.ui.tools_state.focused_window = window;
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }
            ToolsMessage::VersionLoaded(result) => {
                self.ui.tools_state.loading_version = false;
                match result {
                    Ok(version) => {
                        self.ui.tools_state.version = Some(version);
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(e);
                    }
                }
                Task::none()
            }

            // Actions
            ToolsMessage::ReloadConfig => {
                self.ui.tools_state.reloading = true;
                self.ui.tools_state.last_error = None;
                Task::perform(
                    async { crate::ipc::reload_config().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::ReloadCompleted(result)),
                )
            }
            ToolsMessage::ValidateConfig => {
                self.ui.tools_state.validating = true;
                self.ui.tools_state.last_error = None;
                self.ui.tools_state.validation_result = None;
                Task::perform(
                    async { crate::ipc::validate_config().map_err(|e| e.to_string()) },
                    |result| Message::Tools(ToolsMessage::ValidateCompleted(result)),
                )
            }

            // Action results
            ToolsMessage::ReloadCompleted(result) => {
                self.ui.tools_state.reloading = false;
                match result {
                    Ok(()) => {
                        self.ui.toast = Some("Config reloaded successfully".to_string());
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                    }
                    Err(e) => {
                        self.ui.tools_state.last_error = Some(format!("Reload failed: {}", e));
                    }
                }
                Task::none()
            }
            ToolsMessage::ValidateCompleted(result) => {
                self.ui.tools_state.validating = false;
                self.ui.tools_state.validation_result = Some(result);
                Task::none()
            }
        }
    }
}
