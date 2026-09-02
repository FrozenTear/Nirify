//! Outputs (displays) settings message handler

use crate::config::SettingsCategory;
use crate::messages::{Message, OutputsMessage as M, ToolsMessage};
use crate::views::display_layout::{
    collect_monitors, compute_preview_layout, hit_test, output_from_ipc, snap_position,
    unconfigured_outputs, PREVIEW_MAX_HEIGHT, PREVIEW_MAX_WIDTH, SNAP_THRESHOLD,
};
use iced::Task;

/// Canvas pixels that still count as a click rather than a drag.
const CLICK_SLOP: f32 = 4.0;

impl super::super::App {
    /// Updates outputs (displays) settings
    pub(in crate::app) fn update_outputs(&mut self, msg: M) -> Task<Message> {
        match msg {
            M::AddOutput => {
                let unused = unconfigured_outputs(
                    &self.settings.outputs.outputs,
                    &self.ui.tools_state.outputs,
                );
                let Some(info) = unused.into_iter().next() else {
                    self.ui.toast = Some(
                        "No unused connected displays. Connect niri or adopt a connector from an existing output."
                            .to_string(),
                    );
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                    return Task::none();
                };
                let config = output_from_ipc(info);
                log::info!("Added output {}", config.name);
                self.settings.outputs.outputs.push(config);
                let idx = self.settings.outputs.outputs.len() - 1;
                self.ui.selected_output_index = Some(idx);
                self.ui.editing_output_index = Some(idx);
            }

            M::AdoptConnected => {
                let unused = unconfigured_outputs(
                    &self.settings.outputs.outputs,
                    &self.ui.tools_state.outputs,
                );
                if unused.is_empty() {
                    self.ui.toast =
                        Some("All connected displays are already configured.".to_string());
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                    return Task::none();
                }
                let added = unused.len();
                let configs: Vec<_> = unused.into_iter().map(output_from_ipc).collect();
                self.settings.outputs.outputs.extend(configs);
                log::info!("Adopted {added} connected output(s)");
                self.ui.toast = Some(format!("Adopted {added} connected display(s)"));
                self.ui.toast_shown_at = Some(std::time::Instant::now());
            }

            M::RemoveOutput(idx) => {
                if idx < self.settings.outputs.outputs.len() {
                    self.settings.outputs.outputs.remove(idx);
                    self.ui.selected_output_index =
                        adjust_index_after_remove(self.ui.selected_output_index, idx);
                    self.ui.editing_output_index =
                        adjust_index_after_remove(self.ui.editing_output_index, idx);
                    if self
                        .ui
                        .monitor_drag
                        .as_ref()
                        .is_some_and(|d| d.index == idx)
                    {
                        self.ui.monitor_drag = None;
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

            M::CanvasMove(x, y) => {
                self.ui.canvas_pointer = Some((x, y));
                if self.ui.monitor_drag.is_none() {
                    return Task::none();
                }
                if !self.apply_monitor_drag(x, y) {
                    return Task::none();
                }
            }

            M::CanvasPress => {
                if !self.begin_monitor_drag() {
                    return Task::none();
                }
            }

            M::CanvasRelease => {
                let drag = self.ui.monitor_drag.take();
                if let Some(drag) = drag {
                    if !drag.moved {
                        self.ui.editing_output_index = Some(drag.index);
                        self.ui.selected_output_index = Some(drag.index);
                    }
                }
                return Task::none();
            }

            M::SetOutputName(idx, name) => {
                if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
                    output.name = name.trim().to_string();
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

    /// Start a canvas drag (or adopt the clicked unconfigured output).
    ///
    /// Returns `true` when a new output was adopted and the save path should run.
    fn begin_monitor_drag(&mut self) -> bool {
        let Some((cx, cy)) = self.ui.canvas_pointer else {
            return false;
        };
        let monitors = collect_monitors(&self.settings.outputs, &self.ui.tools_state.outputs);
        let Some(layout) = compute_preview_layout(&monitors, PREVIEW_MAX_HEIGHT, PREVIEW_MAX_WIDTH)
        else {
            return false;
        };
        let Some(hit) = hit_test(&layout, cx, cy) else {
            return false;
        };

        let adopted = if hit.rect.config_index.is_none() {
            let name = hit.rect.name.clone();
            if self
                .settings
                .outputs
                .outputs
                .iter()
                .any(|output| output.name == name)
            {
                false
            } else if let Some(info) = self
                .ui
                .tools_state
                .outputs
                .iter()
                .find(|info| info.name == name)
                .cloned()
            {
                self.settings.outputs.outputs.push(output_from_ipc(&info));
                true
            } else {
                false
            }
        } else {
            false
        };

        let idx = hit
            .rect
            .config_index
            .unwrap_or_else(|| self.settings.outputs.outputs.len().saturating_sub(1));
        self.ui.selected_output_index = Some(idx);
        self.ui.monitor_drag = Some(crate::app::ui_state::MonitorDrag {
            index: idx,
            last_canvas_x: cx,
            last_canvas_y: cy,
            press_canvas_x: cx,
            press_canvas_y: cy,
            scale: layout.scale,
            origin_logical: (hit.rect.x, hit.rect.y),
            moved: false,
        });
        adopted
    }

    /// Apply an in-progress drag. Returns `true` when position changed.
    fn apply_monitor_drag(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.ui.monitor_drag.as_ref() else {
            return false;
        };
        let scale = drag.scale.max(0.001);
        let canvas_dx = x - drag.press_canvas_x;
        let canvas_dy = y - drag.press_canvas_y;
        if !drag.moved && canvas_dx.abs() < CLICK_SLOP && canvas_dy.abs() < CLICK_SLOP {
            if let Some(drag) = self.ui.monitor_drag.as_mut() {
                drag.last_canvas_x = x;
                drag.last_canvas_y = y;
            }
            return false;
        }

        let idx = drag.index;
        let first_move = !drag.moved;
        let press_x = drag.press_canvas_x;
        let press_y = drag.press_canvas_y;
        let mut origin = drag.origin_logical;
        if first_move {
            origin = crate::config::seed_manual_position(
                idx,
                &self.settings.outputs.outputs,
                &self.ui.tools_state.outputs,
            );
        }

        let total_dx = ((x - press_x) / scale).round() as i32;
        let total_dy = ((y - press_y) / scale).round() as i32;
        if let Some(drag) = self.ui.monitor_drag.as_mut() {
            drag.last_canvas_x = x;
            drag.last_canvas_y = y;
            drag.origin_logical = origin;
            drag.moved = true;
        }

        let live = &self.ui.tools_state.outputs;
        let (width, height) = self
            .settings
            .outputs
            .outputs
            .get(idx)
            .map(|output| crate::config::estimated_logical_size(output, live))
            .unwrap_or((1920, 1080));
        let others: Vec<(i32, i32, i32, i32)> = self
            .settings
            .outputs
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(i, output)| {
                if i == idx {
                    return None;
                }
                let (ox, oy) = output.position?;
                let (ow, oh) = crate::config::estimated_logical_size(output, live);
                Some((ox, oy, ow as i32, oh as i32))
            })
            .collect();

        let (nx, ny) = snap_position(
            origin.0 + total_dx,
            origin.1 + total_dy,
            width as i32,
            height as i32,
            &others,
            SNAP_THRESHOLD,
        );
        if let Some(output) = self.settings.outputs.outputs.get_mut(idx) {
            if output.position == Some((nx, ny)) {
                return false;
            }
            output.position = Some((nx, ny));
            true
        } else {
            false
        }
    }
}

fn adjust_index_after_remove(current: Option<usize>, removed: usize) -> Option<usize> {
    match current {
        None => None,
        Some(idx) if idx == removed => None,
        Some(idx) if idx > removed => Some(idx - 1),
        Some(idx) => Some(idx),
    }
}

#[cfg(test)]
mod tests {
    use super::adjust_index_after_remove;

    #[test]
    fn remove_clears_matching_selection_and_shifts_later() {
        assert_eq!(adjust_index_after_remove(Some(2), 2), None);
        assert_eq!(adjust_index_after_remove(Some(3), 1), Some(2));
        assert_eq!(adjust_index_after_remove(Some(0), 1), Some(0));
        assert_eq!(adjust_index_after_remove(None, 0), None);
    }
}
