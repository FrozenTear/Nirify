//! Outputs (displays) settings message handler

use crate::config::SettingsCategory;
use crate::messages::{Message, OutputsMessage as M, ToolsMessage};
use iced::Task;

impl super::super::App {
    /// Updates outputs (displays) settings
    pub(in crate::app) fn update_outputs(&mut self, msg: M) -> Task<Message> {
        match msg {
            M::AddOutput => {
                self.settings
                    .outputs
                    .outputs
                    .push(crate::config::models::OutputConfig::default());
                self.ui.selected_output_index = Some(self.settings.outputs.outputs.len() - 1);
                log::info!("Added new output");
            }

            M::RemoveOutput(idx) => {
                if idx < self.settings.outputs.outputs.len() {
                    self.settings.outputs.outputs.remove(idx);
                    if self.ui.selected_output_index == Some(idx) {
                        self.ui.selected_output_index = if self.settings.outputs.outputs.is_empty()
                        {
                            None
                        } else {
                            Some(0)
                        };
                    }
                    log::info!("Removed output at index {}", idx);
                }
            }

            M::SelectOutput(idx) => {
                self.ui.selected_output_index = Some(idx);
                // Auto-refresh IPC outputs to get available modes for dropdown
                // Only refresh if connected to niri
                let is_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                if is_connected {
                    return crate::ipc::tasks::get_full_outputs_async(|r| {
                        Message::Tools(ToolsMessage::OutputsLoaded(r.map_err(|e| e.to_string())))
                    });
                }
                return Task::none();
            }

            M::SetOutputName(idx, name) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.name = name;
                }
            }

            M::SetEnabled(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.enabled = value;
                }
            }

            M::SetScale(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.scale = value;
                }
            }

            M::SetMode(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.mode = value;
                }
            }

            M::SetModeCustom(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.mode_custom = value;
                }
            }

            M::SetModeline(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.modeline = value;
                }
            }

            M::SetPositionX(idx, value) => {
                let mut position = crate::config::seed_manual_position(
                    idx,
                    &self.settings.outputs.outputs,
                    &self.ui.tools_state.outputs,
                );
                position.0 = value;
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.position = Some(position);
                }
            }

            M::SetPositionY(idx, value) => {
                let mut position = crate::config::seed_manual_position(
                    idx,
                    &self.settings.outputs.outputs,
                    &self.ui.tools_state.outputs,
                );
                position.1 = value;
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.position = Some(position);
                }
            }

            M::SetPositionAuto(idx, auto) => {
                if auto {
                    if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                        output.position = None;
                    }
                } else {
                    let position = crate::config::seed_manual_position(
                        idx,
                        &self.settings.outputs.outputs,
                        &self.ui.tools_state.outputs,
                    );
                    if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                        output.position = Some(position);
                    }
                }
            }

            M::ImportConnectedLayout => {
                let is_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                if !is_connected {
                    self.ui.toast =
                        Some("Connect to niri to import the live display layout".to_string());
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                    return Task::none();
                }
                return crate::ipc::tasks::get_full_outputs_async(|result| {
                    Message::Outputs(M::LiveOutputsSnapshotLoaded(
                        result.map_err(|e| e.to_string()),
                    ))
                });
            }

            M::LiveOutputsSnapshotLoaded(result) => match result {
                Ok(live) => {
                    self.ui.tools_state.outputs = live;
                    let applied = crate::config::apply_live_outputs_to_settings(
                        &mut self.settings.outputs,
                        &self.ui.tools_state.outputs,
                    );
                    log::info!("{}", applied.summary());
                    self.ui.toast = Some(applied.summary());
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                    if self.ui.selected_output_index.is_none()
                        && !self.settings.outputs.outputs.is_empty()
                    {
                        self.ui.selected_output_index = Some(0);
                    }
                }
                Err(error) => {
                    self.ui.toast = Some(format!("Could not read live outputs: {error}"));
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                    return Task::none();
                }
            },

            M::SetTransform(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.transform = value;
                }
            }

            M::SetVrr(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.vrr = value;
                }
            }

            M::SetFocusAtStartup(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.focus_at_startup = value;
                }
            }

            M::SetBackgroundColor(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.background_color = value;
                }
            }

            M::SetBackdropColor(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.backdrop_color = value;
                }
            }

            M::SetHotCornersEnabled(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.enabled = value;
                    } else if value.is_some() {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            enabled: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerTopLeft(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.top_left = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            top_left: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerTopRight(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.top_right = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            top_right: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerBottomLeft(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.bottom_left = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            bottom_left: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerBottomRight(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.bottom_right = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            bottom_right: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetLayoutOverride(idx, value) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.layout_override = value;
                }
            }

            M::ToggleSection(section_name) => {
                let expanded = self
                    .ui
                    .output_sections_expanded
                    .get(&section_name)
                    .copied()
                    .unwrap_or(true);
                self.ui
                    .output_sections_expanded
                    .insert(section_name, !expanded);
                return Task::none();
            }

            M::OpenEditor(idx) => {
                self.ui.editing_output_index = Some(idx);
                self.ui.selected_output_index = Some(idx);
                return Task::none();
            }

            M::CloseEditor => {
                self.ui.editing_output_index = None;
                return Task::none();
            }
        }

        // Update the cache for view borrowing

        self.save.dirty_tracker.mark(SettingsCategory::Outputs);
        self.mark_changed();
        Task::none()
    }
}
