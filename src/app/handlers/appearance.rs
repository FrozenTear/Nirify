//! Appearance settings message handler

use crate::config::SettingsCategory;
use crate::messages::{AppearanceMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates appearance settings
    pub(in crate::app) fn update_appearance(&mut self, msg: AppearanceMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");

        match msg {
            // Focus ring
            AppearanceMessage::ToggleFocusRing(value) => {
                settings.appearance.focus_ring_enabled = value;
            }
            AppearanceMessage::SetFocusRingWidth(value) => {
                settings.appearance.focus_ring_width = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::FocusRingActive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_active, gradient_msg);
            }
            AppearanceMessage::FocusRingInactive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_inactive, gradient_msg);
            }
            AppearanceMessage::FocusRingUrgent(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_urgent, gradient_msg);
            }

            // Border
            AppearanceMessage::ToggleBorder(value) => {
                settings.appearance.border_enabled = value;
            }
            AppearanceMessage::SetBorderThickness(value) => {
                settings.appearance.border_thickness = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::BorderActive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_active, gradient_msg);
            }
            AppearanceMessage::BorderInactive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_inactive, gradient_msg);
            }
            AppearanceMessage::BorderUrgent(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_urgent, gradient_msg);
            }

            // Layout
            AppearanceMessage::SetGaps(value) => {
                settings.appearance.gaps = value.clamp(0.0, 64.0);
            }
            AppearanceMessage::SetCornerRadius(value) => {
                settings.appearance.corner_radius = value.clamp(0.0, 32.0);
            }

            // Background
            AppearanceMessage::SetBackgroundColor(hex_opt) => {
                use crate::types::Color;
                settings.appearance.background_color = hex_opt.and_then(|hex| Color::from_hex(&hex));
            }
        }

        drop(settings); // Release lock

        // Mark as dirty for auto-save
        self.dirty_tracker.mark(SettingsCategory::Appearance);
        self.save_manager.mark_changed();

        Task::none()
    }
}
