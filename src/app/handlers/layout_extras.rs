//! Layout extras settings message handler

use crate::app::helpers::apply_gradient_message;
use crate::config::SettingsCategory;
use crate::messages::{LayoutExtrasMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle layout extras settings messages
    pub(in crate::app) fn update_layout_extras(
        &mut self,
        msg: LayoutExtrasMessage,
    ) -> Task<Message> {
        let layout = &mut self.settings.layout_extras;

        match msg {
            // Shadow settings
            LayoutExtrasMessage::SetShadowEnabled(v) => layout.shadow.enabled = v,
            LayoutExtrasMessage::SetShadowSoftness(v) => layout.shadow.softness = v.clamp(0, 100),
            LayoutExtrasMessage::SetShadowSpread(v) => layout.shadow.spread = v.clamp(-1024, 1024),
            LayoutExtrasMessage::SetShadowOffsetX(v) => layout.shadow.offset_x = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowOffsetY(v) => layout.shadow.offset_y = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowDrawBehindWindow(v) => {
                layout.shadow.draw_behind_window = v
            }
            LayoutExtrasMessage::SetShadowColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.color = color;
                }
            }
            LayoutExtrasMessage::SetShadowInactiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.inactive_color = color;
                    layout.shadow.use_inactive_color = true;
                }
            }
            LayoutExtrasMessage::SetShadowUseInactiveColor(v) => {
                layout.shadow.use_inactive_color = v
            }

            // Tab indicator
            LayoutExtrasMessage::SetTabIndicatorEnabled(v) => layout.tab_indicator.enabled = v,
            LayoutExtrasMessage::SetTabIndicatorHideWhenSingleTab(v) => {
                layout.tab_indicator.hide_when_single_tab = v
            }
            LayoutExtrasMessage::SetTabIndicatorPlaceWithinColumn(v) => {
                layout.tab_indicator.place_within_column = v
            }
            LayoutExtrasMessage::SetTabIndicatorGap(v) => layout.tab_indicator.gap = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorWidth(v) => {
                layout.tab_indicator.width = v.clamp(1, 50)
            }
            LayoutExtrasMessage::SetTabIndicatorLengthProportion(v) => {
                layout.tab_indicator.length_proportion = v.clamp(0.1, 2.0)
            }
            LayoutExtrasMessage::SetTabIndicatorCornerRadius(v) => {
                layout.tab_indicator.corner_radius = v.clamp(0, 50)
            }
            LayoutExtrasMessage::SetTabIndicatorGapsBetweenTabs(v) => {
                layout.tab_indicator.gaps_between_tabs = v.clamp(0, 50)
            }
            LayoutExtrasMessage::SetTabIndicatorPosition(v) => layout.tab_indicator.position = v,
            LayoutExtrasMessage::SetTabIndicatorActiveColor(msg) => {
                apply_gradient_message(&mut layout.tab_indicator.active, msg);
                layout.tab_indicator.use_active_color = true;
            }
            LayoutExtrasMessage::SetTabIndicatorInactiveColor(msg) => {
                apply_gradient_message(&mut layout.tab_indicator.inactive, msg);
                layout.tab_indicator.use_inactive_color = true;
            }
            LayoutExtrasMessage::SetTabIndicatorUrgentColor(msg) => {
                apply_gradient_message(&mut layout.tab_indicator.urgent, msg);
                layout.tab_indicator.use_urgent_color = true;
            }
            LayoutExtrasMessage::SetTabIndicatorUseActiveColor(v) => {
                layout.tab_indicator.use_active_color = v
            }
            LayoutExtrasMessage::SetTabIndicatorUseInactiveColor(v) => {
                layout.tab_indicator.use_inactive_color = v
            }
            LayoutExtrasMessage::SetTabIndicatorUseUrgentColor(v) => {
                layout.tab_indicator.use_urgent_color = v
            }

            // Insert hint
            LayoutExtrasMessage::SetInsertHintEnabled(v) => layout.insert_hint.enabled = v,
            LayoutExtrasMessage::SetInsertHintColor(msg) => {
                apply_gradient_message(&mut layout.insert_hint.color, msg);
            }

            // Preset widths/heights
            LayoutExtrasMessage::AddPresetWidth => {
                layout
                    .preset_column_widths
                    .push(crate::config::models::PresetWidth::Proportion(0.5));
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
                layout
                    .preset_window_heights
                    .push(crate::config::models::PresetHeight::Proportion(0.5));
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

        self.save.dirty_tracker.mark(SettingsCategory::LayoutExtras);
        self.mark_changed();
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::models::LayoutExtrasSettings;
    use crate::types::{Color, ColorOrGradient, Gradient};
    use crate::views::widgets::{apply_gradient_message, GradientPickerMessage};

    fn gradient() -> ColorOrGradient {
        ColorOrGradient::Gradient(Gradient {
            from: Color::from_hex("#ff0000").unwrap(),
            to: Color::from_hex("#0000ff").unwrap(),
            angle: 45,
            ..Default::default()
        })
    }

    #[test]
    fn tab_and_insert_hint_edits_keep_imported_gradients() {
        let mut layout = LayoutExtrasSettings::default();
        layout.tab_indicator.urgent = gradient();
        layout.insert_hint.color = gradient();

        apply_gradient_message(
            &mut layout.tab_indicator.urgent,
            GradientPickerMessage::SetFromColor("#00ff00".into()),
        );
        apply_gradient_message(
            &mut layout.insert_hint.color,
            GradientPickerMessage::SetToColor("#ffffff".into()),
        );

        assert!(layout.tab_indicator.urgent.is_gradient());
        assert!(layout.insert_hint.color.is_gradient());
        match &layout.tab_indicator.urgent {
            ColorOrGradient::Gradient(g) => {
                assert_eq!(g.from, Color::from_hex("#00ff00").unwrap());
                assert_eq!(g.angle, 45);
            }
            ColorOrGradient::Color(_) => panic!("tab urgent flattened"),
        }
    }
}
