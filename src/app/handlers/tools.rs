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
                        // Older builds stored refresh rounded to 2 decimals
                        // (143.98), which niri cannot match against 143.981.
                        // Snap near matches to the exact live mode in memory
                        // only. Mark Outputs dirty so unsaved state shows, but
                        // do not call mark_changed(): loading outputs (page
                        // open, startup) must never start the debounced save.
                        // niri reloads outputs.kdl on write and can blank the
                        // screen. The exact mode is written on the next save.
                        let snapped = crate::config::snap_output_modes_to_live(
                            &mut self.settings.outputs,
                            &self.ui.tools_state.outputs,
                        );
                        if snapped > 0 {
                            self.save
                                .dirty_tracker
                                .mark(crate::config::SettingsCategory::Outputs);
                        }
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

#[cfg(test)]
mod tests {
    use crate::config::models::OutputConfig;
    use crate::config::SettingsCategory;
    use crate::ipc::{FullOutputInfo, OutputMode};
    use crate::messages::ToolsMessage;

    fn dp3_live() -> FullOutputInfo {
        FullOutputInfo {
            name: "DP-3".to_string(),
            current_mode: Some(1),
            modes: [60_000, 143_981]
                .iter()
                .map(|&refresh_rate| OutputMode {
                    width: 1920,
                    height: 1080,
                    refresh_rate,
                    is_preferred: false,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn outputs_loaded_snaps_in_memory_without_starting_save() {
        let (mut app, _) = crate::app::App::new_with_error(String::new(), None);
        app.settings.outputs.outputs = vec![OutputConfig {
            name: "DP-3".into(),
            mode: "1920x1080@143.98".into(),
            ..Default::default()
        }];
        assert!(app.save.last_change_time.is_none());
        assert!(!app.save.dirty_tracker.is_dirty());

        let _ = app.update_tools(ToolsMessage::OutputsLoaded(Ok(vec![dp3_live()])));

        // In-memory config shows the exact live mode.
        assert_eq!(app.settings.outputs.outputs[0].mode, "1920x1080@143.981");
        // Outputs shows as unsaved...
        assert!(app
            .save
            .dirty_tracker
            .peek()
            .contains(&SettingsCategory::Outputs));
        // ...but the debounced auto-save is not armed, so nothing is written.
        assert!(app.save.last_change_time.is_none());
        assert!(!app.should_save());
        assert!(!app.save.in_progress);
    }

    #[test]
    fn outputs_loaded_without_snap_stays_clean() {
        let (mut app, _) = crate::app::App::new_with_error(String::new(), None);
        app.settings.outputs.outputs = vec![OutputConfig {
            name: "DP-3".into(),
            mode: "1920x1080@143.981".into(),
            ..Default::default()
        }];

        let _ = app.update_tools(ToolsMessage::OutputsLoaded(Ok(vec![dp3_live()])));

        assert!(!app.save.dirty_tracker.is_dirty());
        assert!(app.save.last_change_time.is_none());
    }
}
