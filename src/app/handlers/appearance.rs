//! Appearance settings message handler

use crate::app::helpers::apply_gradient_message;
use crate::config::SettingsCategory;
use crate::messages::{AppearanceMessage, Message};
use iced::Task;

impl super::super::App {
    /// Updates appearance settings
    pub(in crate::app) fn update_appearance(&mut self, msg: AppearanceMessage) -> Task<Message> {
        match msg {
            // Focus ring
            AppearanceMessage::ToggleFocusRing(value) => {
                self.settings.appearance.focus_ring_enabled = value;
            }
            AppearanceMessage::SetFocusRingWidth(value) => {
                self.settings.appearance.focus_ring_width = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::FocusRingActive(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.focus_ring_active, gradient_msg);
            }
            AppearanceMessage::FocusRingInactive(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.focus_ring_inactive, gradient_msg);
            }
            AppearanceMessage::FocusRingUrgent(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.focus_ring_urgent, gradient_msg);
            }

            // Border
            AppearanceMessage::ToggleBorder(value) => {
                self.settings.appearance.border_enabled = value;
            }
            AppearanceMessage::SetBorderThickness(value) => {
                self.settings.appearance.border_thickness = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::BorderActive(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.border_active, gradient_msg);
            }
            AppearanceMessage::BorderInactive(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.border_inactive, gradient_msg);
            }
            AppearanceMessage::BorderUrgent(gradient_msg) => {
                apply_gradient_message(&mut self.settings.appearance.border_urgent, gradient_msg);
            }

            // Layout
            AppearanceMessage::SetGaps(value) => {
                self.settings.appearance.gaps = value.clamp(0.0, 64.0);
            }
            AppearanceMessage::SetCornerRadius(value) => {
                self.settings.appearance.corner_radius = value.clamp(0.0, 32.0);
            }

            // Background
            AppearanceMessage::SetBackgroundColor(hex_opt) => {
                use crate::types::Color;
                self.settings.appearance.background_color = hex_opt.and_then(|hex| Color::from_hex(&hex));
            }
        }

        // Mark as dirty for auto-save
        self.dirty_tracker.mark(SettingsCategory::Appearance);
        self.mark_changed();

        Task::none()
    }
}
