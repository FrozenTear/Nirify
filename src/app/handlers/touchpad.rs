//! Touchpad settings message handler

use crate::config::SettingsCategory;
use crate::messages::{Message, TouchpadMessage};
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
                self.settings.touchpad.scroll_factor = (value as f64).clamp(
                    crate::constants::SCROLL_FACTOR_MIN,
                    crate::constants::SCROLL_FACTOR_MAX,
                );
                // Keep the exact-entry buffer in sync when the slider drives it.
                self.ui.touchpad_scroll_factor_text =
                    format!("{}", self.settings.touchpad.scroll_factor);
            }
            TouchpadMessage::SetScrollFactorText(text) => {
                // Commit on successful parse (clamped so out-of-range never
                // persists) but keep the raw text so intermediate strings like
                // "-", "1." and "" survive re-render.
                if let Ok(v) = text.parse::<f64>() {
                    self.settings.touchpad.scroll_factor = v.clamp(
                        crate::constants::SCROLL_FACTOR_MIN,
                        crate::constants::SCROLL_FACTOR_MAX,
                    );
                }
                self.ui.touchpad_scroll_factor_text = text;
            }
            TouchpadMessage::CommitScrollFactor => {
                // On submit/blur, snap the buffer back to the clamped model value.
                self.ui.touchpad_scroll_factor_text =
                    format!("{}", self.settings.touchpad.scroll_factor);
            }
            TouchpadMessage::SetScrollFactorHorizontal(value) => {
                self.settings.touchpad.scroll_factor_horizontal = value.map(|v| {
                    (v as f64).clamp(
                        crate::constants::SCROLL_FACTOR_MIN,
                        crate::constants::SCROLL_FACTOR_MAX,
                    )
                });
            }
            TouchpadMessage::SetScrollMethod(method) => {
                self.settings.touchpad.scroll_method = method;
            }
            TouchpadMessage::SetScrollButton(value) => {
                self.settings.touchpad.scroll_button = value;
            }
            TouchpadMessage::ToggleScrollButtonLock(value) => {
                self.settings.touchpad.scroll_button_lock = value;
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
                self.settings.touchpad.drag = Some(value);
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

        self.save.dirty_tracker.mark(SettingsCategory::Touchpad);
        self.mark_changed();

        Task::none()
    }
}
