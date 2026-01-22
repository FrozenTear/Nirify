//! Touchpad settings message handler

use crate::config::SettingsCategory;
use crate::messages::{TouchpadMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates touchpad settings
    pub(in crate::app) fn update_touchpad(&mut self, msg: TouchpadMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            TouchpadMessage::ToggleTapToClick(value) => {
                settings.touchpad.tap = value;
            }
            TouchpadMessage::ToggleDwt(value) => {
                settings.touchpad.dwt = value;
            }
            TouchpadMessage::ToggleDwtp(value) => {
                settings.touchpad.dwtp = value;
            }
            TouchpadMessage::ToggleNaturalScroll(value) => {
                settings.touchpad.natural_scroll = value;
            }
            TouchpadMessage::SetAccelSpeed(value) => {
                settings.touchpad.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            TouchpadMessage::SetAccelProfile(profile) => {
                settings.touchpad.accel_profile = profile;
            }
            TouchpadMessage::SetScrollFactor(value) => {
                settings.touchpad.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            TouchpadMessage::SetScrollMethod(method) => {
                settings.touchpad.scroll_method = method;
            }
            TouchpadMessage::SetClickMethod(method) => {
                settings.touchpad.click_method = method;
            }
            TouchpadMessage::SetTapButtonMap(map) => {
                settings.touchpad.tap_button_map = map;
            }
            TouchpadMessage::ToggleLeftHanded(value) => {
                settings.touchpad.left_handed = value;
            }
            TouchpadMessage::ToggleDrag(value) => {
                settings.touchpad.drag = value;
            }
            TouchpadMessage::ToggleDragLock(value) => {
                settings.touchpad.drag_lock = value;
            }
            TouchpadMessage::ToggleMiddleEmulation(value) => {
                settings.touchpad.middle_emulation = value;
            }
            TouchpadMessage::ToggleDisabledOnExternalMouse(value) => {
                settings.touchpad.disabled_on_external_mouse = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Touchpad);
        self.save_manager.mark_changed();

        Task::none()
    }
}
