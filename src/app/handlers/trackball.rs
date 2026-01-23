//! Trackball settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TrackballMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle trackball settings messages
    pub(in crate::app) fn update_trackball(&mut self, msg: TrackballMessage) -> Task<Message> {
        
        let trackball = &mut self.settings.trackball;

        match msg {
            TrackballMessage::SetOff(v) => trackball.off = v,
            TrackballMessage::SetNaturalScroll(v) => trackball.natural_scroll = v,
            TrackballMessage::SetAccelSpeed(v) => trackball.accel_speed = v.clamp(-1.0, 1.0) as f64,
            TrackballMessage::SetAccelProfile(v) => trackball.accel_profile = v,
            TrackballMessage::SetScrollMethod(v) => trackball.scroll_method = v,
            TrackballMessage::SetLeftHanded(v) => trackball.left_handed = v,
            TrackballMessage::SetMiddleEmulation(v) => trackball.middle_emulation = v,
            TrackballMessage::SetScrollButtonLock(v) => trackball.scroll_button_lock = v,
            TrackballMessage::SetScrollButton(v) => trackball.scroll_button = v,
        }

        self.dirty_tracker.mark(SettingsCategory::Trackball);
        self.mark_changed();
        Task::none()
    }
}
