//! Touch settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TouchMessage, Message};
use crate::views::widgets::format_matrix_values;
use iced::Task;

impl super::super::App {
    /// Handle touch settings messages
    pub(in crate::app) fn update_touch(&mut self, msg: TouchMessage) -> Task<Message> {
        
        let touch = &mut self.settings.touch;

        match msg {
            TouchMessage::SetOff(v) => touch.off = v,
            TouchMessage::SetMapToOutput(v) => touch.map_to_output = v,
            TouchMessage::SetCalibrationMatrix(v) => {
                touch.calibration_matrix = v;
                // Update cache
                self.touch_calibration_cache = format_matrix_values(v);
            }
            TouchMessage::SetCalibrationValue(idx, value) => {
                if idx < 6 {
                    // Update cache immediately for responsive UI
                    self.touch_calibration_cache[idx] = value.clone();
                    // Parse and update actual matrix
                    if let Ok(val) = value.parse::<f64>() {
                        let matrix = touch.calibration_matrix
                            .get_or_insert([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                        matrix[idx] = val;
                    }
                }
            }
            TouchMessage::ClearCalibration => {
                touch.calibration_matrix = None;
                // Update cache to show identity (default display)
                self.touch_calibration_cache = format_matrix_values(None);
            }
            TouchMessage::ResetCalibration => {
                touch.calibration_matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                self.touch_calibration_cache = format_matrix_values(
                    Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0])
                );
            }
        }

        self.dirty_tracker.mark(SettingsCategory::Touch);
        self.mark_changed();
        Task::none()
    }
}
