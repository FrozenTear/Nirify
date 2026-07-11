//! Tools page message handler (IPC operations)

use crate::messages::{Message, ToolsMessage};
use iced::Task;

impl super::super::App {
    /// Handle tools page messages (IPC operations)
    pub(in crate::app) fn update_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            // Query triggers - spawn async tasks
            ToolsMessage::RefreshWindows => {
                self.ui.tools_state.loading_windows = true;
                self.ui.tools_state.last_error = None;
                crate::ipc::tasks::get_windows_async(|r| {
                    Message::Tools(ToolsMessage::WindowsLoaded(r.map_err(|e| e.to_string())))
                })
            }
            ToolsMessage::RefreshWorkspaces => {
                self.ui.tools_state.loading_workspaces = true;
                self.ui.tools_state.last_error = None;
                crate::ipc::tasks::get_workspaces_async(|r| {
                    Message::Tools(ToolsMessage::WorkspacesLoaded(r.map_err(|e| e.to_string())))
                })
            }
            ToolsMessage::RefreshOutputs => {
                self.ui.tools_state.loading_outputs = true;
                self.ui.tools_state.last_error = None;
                crate::ipc::tasks::get_full_outputs_async(|r| {
                    Message::Tools(ToolsMessage::OutputsLoaded(r.map_err(|e| e.to_string())))
                })
            }
            ToolsMessage::RefreshFocusedWindow => {
                self.ui.tools_state.last_error = None;
                crate::ipc::tasks::get_focused_window_async(|r| {
                    Message::Tools(ToolsMessage::FocusedWindowLoaded(
                        r.map_err(|e| e.to_string()),
                    ))
                })
            }
            ToolsMessage::RefreshVersion => {
                self.ui.tools_state.loading_version = true;
                self.ui.tools_state.last_error = None;
                crate::ipc::tasks::get_version_async(|r| {
                    Message::Tools(ToolsMessage::VersionLoaded(r.map_err(|e| e.to_string())))
                })
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
                        // Extract named workspaces for use in dropdowns (e.g., window rules)
                        self.ui.available_workspaces =
                            workspaces.iter().filter_map(|w| w.name.clone()).collect();
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
                        self.ui.tools_state.version = Some(version.clone());
                        // Refresh version-gated feature compatibility on (re)connect.
                        if let Some(parsed) = crate::version::NiriVersion::parse(&version) {
                            let new_compat =
                                crate::version::FeatureCompat::from_version(Some(parsed));
                            let compat_changed = new_compat != self.ui.feature_compat;
                            self.ui.niri_version = Some(parsed);
                            self.ui.feature_compat = new_compat;
                            if compat_changed {
                                // main.kdl's include list is version-gated; regenerate now.
                                let content = crate::config::storage::generate_main_kdl(new_compat);
                                if let Err(e) =
                                    crate::config::atomic_write(&self.paths.main_kdl, &content)
                                {
                                    log::warn!(
                                        "failed to refresh main.kdl after version change: {e}"
                                    );
                                }
                                // Create any files newly allowed by the version.
                                if let Err(e) = crate::config::ensure_required_files_exist(
                                    &self.paths,
                                    &self.settings,
                                    new_compat,
                                ) {
                                    log::warn!(
                                        "failed to ensure config files after version change: {e}"
                                    );
                                }
                            }
                        }
                        // If parse fails, leave niri_version/feature_compat untouched.
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
                crate::ipc::tasks::reload_config_async(|r| {
                    Message::Tools(ToolsMessage::ReloadCompleted(r.map_err(|e| e.to_string())))
                })
            }
            ToolsMessage::ValidateConfig => {
                self.ui.tools_state.validating = true;
                self.ui.tools_state.last_error = None;
                self.ui.tools_state.validation_result = None;
                crate::ipc::tasks::validate_config_async(|r| {
                    Message::Tools(ToolsMessage::ValidateCompleted(
                        r.map_err(|e| e.to_string()),
                    ))
                })
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
