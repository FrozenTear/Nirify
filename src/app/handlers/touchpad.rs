//! Touchpad settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TouchpadMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates touchpad settings
    pub(in crate::app) fn update_touchpad(&mut self, msg: TouchpadMessage) -> Task<Message> {
        

        match msg {
            TouchpadMessage::ToggleTapToClick(value) => {
                self.settings.touchpad.tap = value;
            }
            TouchpadMessage::ToggleDwt(value) => {
                self.settings.touchpad.dwt = value;
            }
            TouchpadMessage::ToggleDwtp(value) => {
                self.settings.touchpad.dwtp = value;
            }
            TouchpadMessage::ToggleNaturalScroll(value) => {
                self.settings.touchpad.natural_scroll = value;
            }
            TouchpadMessage::SetAccelSpeed(value) => {
                self.settings.touchpad.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            TouchpadMessage::SetAccelProfile(profile) => {
                self.settings.touchpad.accel_profile = profile;
            }
            TouchpadMessage::SetScrollFactor(value) => {
                self.settings.touchpad.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            TouchpadMessage::SetScrollMethod(method) => {
                self.settings.touchpad.scroll_method = method;
            }
            TouchpadMessage::SetClickMethod(method) => {
                self.settings.touchpad.click_method = method;
            }
            TouchpadMessage::SetTapButtonMap(map) => {
                self.settings.touchpad.tap_button_map = map;
            }
            TouchpadMessage::ToggleLeftHanded(value) => {
                self.settings.touchpad.left_handed = value;
            }
            TouchpadMessage::ToggleDrag(value) => {
                self.settings.touchpad.drag = value;
            }
            TouchpadMessage::ToggleDragLock(value) => {
                self.settings.touchpad.drag_lock = value;
            }
            TouchpadMessage::ToggleMiddleEmulation(value) => {
                self.settings.touchpad.middle_emulation = value;
            }
            TouchpadMessage::ToggleDisabledOnExternalMouse(value) => {
                self.settings.touchpad.disabled_on_external_mouse = value;
            }
        }


        self.dirty_tracker.mark(SettingsCategory::Touchpad);
        self.mark_changed();

        Task::none()
    }
}
