//! Trackpoint settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TrackpointMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle trackpoint settings messages
    pub(in crate::app) fn update_trackpoint(&mut self, msg: TrackpointMessage) -> Task<Message> {
        
        let trackpoint = &mut self.settings.trackpoint;

        match msg {
            TrackpointMessage::SetOff(v) => trackpoint.off = v,
            TrackpointMessage::SetNaturalScroll(v) => trackpoint.natural_scroll = v,
            TrackpointMessage::SetAccelSpeed(v) => trackpoint.accel_speed = v.clamp(-1.0, 1.0) as f64,
            TrackpointMessage::SetAccelProfile(v) => trackpoint.accel_profile = v,
            TrackpointMessage::SetScrollMethod(v) => trackpoint.scroll_method = v,
            TrackpointMessage::SetLeftHanded(v) => trackpoint.left_handed = v,
            TrackpointMessage::SetMiddleEmulation(v) => trackpoint.middle_emulation = v,
            TrackpointMessage::SetScrollButtonLock(v) => trackpoint.scroll_button_lock = v,
            TrackpointMessage::SetScrollButton(v) => trackpoint.scroll_button = v,
        }

        self.save.dirty_tracker.mark(SettingsCategory::Trackpoint);
        self.mark_changed();
        Task::none()
    }
}
