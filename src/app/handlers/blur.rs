//! Top-level background blur settings message handler (niri 26.04+)

use crate::config::SettingsCategory;
use crate::messages::{BlurMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle background blur settings messages
    pub(in crate::app) fn update_blur(&mut self, msg: BlurMessage) -> Task<Message> {
        let blur = &mut self.settings.blur;
        match msg {
            BlurMessage::SetEnabled(v) => blur.enabled = v,
            BlurMessage::SetPasses(v) => blur.passes = v.clamp(0, 255),
            BlurMessage::SetOffset(v) => blur.offset = v.clamp(0.0, 100.0),
            BlurMessage::SetNoise(v) => blur.noise = v.clamp(0.0, 1000.0),
            BlurMessage::SetSaturation(v) => blur.saturation = v.clamp(0.0, 1000.0),
        }
        self.save.dirty_tracker.mark(SettingsCategory::Blur);
        self.mark_changed();
        Task::none()
    }
}
