//! Tablet settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TabletMessage, Message};
use crate::views::widgets::format_matrix_values;
use iced::Task;

impl super::super::App {
    /// Handle tablet settings messages
    pub(in crate::app) fn update_tablet(&mut self, msg: TabletMessage) -> Task<Message> {
        
        let tablet = &mut self.settings.tablet;

        match msg {
            TabletMessage::SetOff(v) => tablet.off = v,
            TabletMessage::SetLeftHanded(v) => tablet.left_handed = v,
            TabletMessage::SetMapToOutput(v) => tablet.map_to_output = v,
            TabletMessage::SetCalibrationMatrix(v) => {
                tablet.calibration_matrix = v;
                // Update cache
                self.tablet_calibration_cache = format_matrix_values(v);
            }
            TabletMessage::SetCalibrationValue(idx, value) => {
                if idx < 6 {
                    // Update cache immediately for responsive UI
                    self.tablet_calibration_cache[idx] = value.clone();
                    // Parse and update actual matrix
                    if let Ok(val) = value.parse::<f64>() {
                        let matrix = tablet.calibration_matrix
                            .get_or_insert([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                        matrix[idx] = val;
                    }
                }
            }
            TabletMessage::ClearCalibration => {
                tablet.calibration_matrix = None;
                // Update cache to show identity (default display)
                self.tablet_calibration_cache = format_matrix_values(None);
            }
            TabletMessage::ResetCalibration => {
                tablet.calibration_matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                self.tablet_calibration_cache = format_matrix_values(
                    Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0])
                );
            }
        }

        self.dirty_tracker.mark(SettingsCategory::Tablet);
        self.mark_changed();
        Task::none()
    }
}
