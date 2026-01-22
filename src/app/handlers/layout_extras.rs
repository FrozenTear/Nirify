//! Layout extras settings message handler

use crate::config::SettingsCategory;
use crate::messages::{LayoutExtrasMessage, Message};
use crate::types::ColorOrGradient;
use iced::Task;

impl super::super::App {
    /// Handle layout extras settings messages
    pub(in crate::app) fn update_layout_extras(&mut self, msg: LayoutExtrasMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let layout = &mut settings.layout_extras;

        match msg {
            // Shadow settings
            LayoutExtrasMessage::SetShadowEnabled(v) => layout.shadow.enabled = v,
            LayoutExtrasMessage::SetShadowSoftness(v) => layout.shadow.softness = v.clamp(0, 100),
            LayoutExtrasMessage::SetShadowSpread(v) => layout.shadow.spread = v.clamp(0, 100),
            LayoutExtrasMessage::SetShadowOffsetX(v) => layout.shadow.offset_x = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowOffsetY(v) => layout.shadow.offset_y = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowDrawBehindWindow(v) => layout.shadow.draw_behind_window = v,
            LayoutExtrasMessage::SetShadowColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.color = color;
                }
            }
            LayoutExtrasMessage::SetShadowInactiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.inactive_color = color;
                }
            }

            // Tab indicator
            LayoutExtrasMessage::SetTabIndicatorEnabled(v) => layout.tab_indicator.enabled = v,
            LayoutExtrasMessage::SetTabIndicatorHideWhenSingleTab(v) => layout.tab_indicator.hide_when_single_tab = v,
            LayoutExtrasMessage::SetTabIndicatorPlaceWithinColumn(v) => layout.tab_indicator.place_within_column = v,
            LayoutExtrasMessage::SetTabIndicatorGap(v) => layout.tab_indicator.gap = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorWidth(v) => layout.tab_indicator.width = v.clamp(1, 50),
            LayoutExtrasMessage::SetTabIndicatorLengthProportion(v) => layout.tab_indicator.length_proportion = v.clamp(0.1, 1.0),
            LayoutExtrasMessage::SetTabIndicatorCornerRadius(v) => layout.tab_indicator.corner_radius = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorGapsBetweenTabs(v) => layout.tab_indicator.gaps_between_tabs = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorPosition(v) => layout.tab_indicator.position = v,
            LayoutExtrasMessage::SetTabIndicatorActiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.active = ColorOrGradient::Color(color);
                }
            }
            LayoutExtrasMessage::SetTabIndicatorInactiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.inactive = ColorOrGradient::Color(color);
                }
            }
            LayoutExtrasMessage::SetTabIndicatorUrgentColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.urgent = ColorOrGradient::Color(color);
                }
            }

            // Insert hint
            LayoutExtrasMessage::SetInsertHintEnabled(v) => layout.insert_hint.enabled = v,
            LayoutExtrasMessage::SetInsertHintColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.insert_hint.color = ColorOrGradient::Color(color);
                }
            }

            // Preset widths/heights
            LayoutExtrasMessage::AddPresetWidth => {
                layout.preset_column_widths.push(crate::config::models::PresetWidth::Proportion(0.5));
            }
            LayoutExtrasMessage::RemovePresetWidth(idx) => {
                if idx < layout.preset_column_widths.len() {
                    layout.preset_column_widths.remove(idx);
                }
            }
            LayoutExtrasMessage::SetPresetWidth(idx, width) => {
                if let Some(w) = layout.preset_column_widths.get_mut(idx) {
                    *w = width;
                }
            }
            LayoutExtrasMessage::AddPresetHeight => {
                layout.preset_window_heights.push(crate::config::models::PresetHeight::Proportion(0.5));
            }
            LayoutExtrasMessage::RemovePresetHeight(idx) => {
                if idx < layout.preset_window_heights.len() {
                    layout.preset_window_heights.remove(idx);
                }
            }
            LayoutExtrasMessage::SetPresetHeight(idx, height) => {
                if let Some(h) = layout.preset_window_heights.get_mut(idx) {
                    *h = height;
                }
            }

            // Default column display
            LayoutExtrasMessage::SetDefaultColumnDisplay(v) => layout.default_column_display = v,
        }

        drop(settings);
        self.dirty_tracker.mark(SettingsCategory::LayoutExtras);
        self.save_manager.mark_changed();
        Task::none()
    }
}
