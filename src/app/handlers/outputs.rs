//! Outputs (displays) settings message handler

use crate::config::SettingsCategory;
use crate::messages::{OutputsMessage as M, ToolsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates outputs (displays) settings
    pub(in crate::app) fn update_outputs(&mut self, msg: M) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            M::AddOutput => {
                settings.outputs.outputs.push(crate::config::models::OutputConfig::default());
                self.selected_output_index = Some(settings.outputs.outputs.len() - 1);
                log::info!("Added new output");
            }

            M::RemoveOutput(idx) => {
                if idx < settings.outputs.outputs.len() {
                    settings.outputs.outputs.remove(idx);
                    if self.selected_output_index == Some(idx) {
                        self.selected_output_index = if settings.outputs.outputs.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                    }
                    log::info!("Removed output at index {}", idx);
                }
            }

            M::SelectOutput(idx) => {
                self.selected_output_index = Some(idx);
                // Auto-refresh IPC outputs to get available modes for dropdown
                // Only refresh if connected to niri
                let is_connected = matches!(
                    self.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                if is_connected {
                    return Task::perform(
                        async { crate::ipc::get_full_outputs().map_err(|e| e.to_string()) },
                        |result| Message::Tools(ToolsMessage::OutputsLoaded(result)),
                    );
                }
                return Task::none();
            }

            M::SetOutputName(idx, name) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.name = name;
                }
            }

            M::SetEnabled(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.enabled = value;
                }
            }

            M::SetScale(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.scale = value;
                }
            }

            M::SetMode(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.mode = value;
                }
            }

            M::SetModeCustom(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.mode_custom = value;
                }
            }

            M::SetModeline(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.modeline = value;
                }
            }

            M::SetPositionX(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.position_x = value;
                }
            }

            M::SetPositionY(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.position_y = value;
                }
            }

            M::SetTransform(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.transform = value;
                }
            }

            M::SetVrr(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.vrr = value;
                }
            }

            M::SetFocusAtStartup(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.focus_at_startup = value;
                }
            }

            M::SetBackdropColor(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.backdrop_color = value;
                }
            }

            M::SetHotCornersEnabled(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
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
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
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
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
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
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
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
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
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
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.layout_override = value;
                }
            }

            M::ToggleSection(section_name) => {
                let expanded = self.output_sections_expanded.get(&section_name).copied().unwrap_or(true);
                self.output_sections_expanded.insert(section_name, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }
        }

        // Update the cache for view borrowing
        self.outputs_cache = settings.outputs.clone();
        drop(settings); // Explicitly drop the lock before other operations

        self.dirty_tracker.mark(SettingsCategory::Outputs);
        self.save_manager.mark_changed();
        Task::none()
    }
}
