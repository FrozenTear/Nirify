//! Recent windows settings message handler

use crate::config::SettingsCategory;
use crate::messages::{RecentWindowsMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle recent windows settings messages
    pub(in crate::app) fn update_recent_windows(&mut self, msg: RecentWindowsMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let recent = &mut settings.recent_windows;

        match msg {
            // Top-level settings
            RecentWindowsMessage::SetOff(v) => recent.off = v,
            RecentWindowsMessage::SetDebounceMs(v) => recent.debounce_ms = v.clamp(0, 5000),
            RecentWindowsMessage::SetOpenDelayMs(v) => recent.open_delay_ms = v.clamp(0, 5000),

            // Highlight settings
            RecentWindowsMessage::SetActiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    recent.highlight.active_color = color;
                }
            }
            RecentWindowsMessage::SetUrgentColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    recent.highlight.urgent_color = color;
                }
            }
            RecentWindowsMessage::SetHighlightPadding(v) => recent.highlight.padding = v.clamp(0, 100),
            RecentWindowsMessage::SetHighlightCornerRadius(v) => recent.highlight.corner_radius = v.clamp(0, 100),

            // Preview settings
            RecentWindowsMessage::SetPreviewMaxHeight(v) => recent.previews.max_height = v.clamp(50, 1000),
            RecentWindowsMessage::SetPreviewMaxScale(v) => recent.previews.max_scale = v.clamp(0.1, 1.0),

            // Keybind management
            RecentWindowsMessage::AddBind => {
                recent.binds.push(crate::config::models::RecentWindowsBind::default());
            }
            RecentWindowsMessage::RemoveBind(idx) => {
                if idx < recent.binds.len() {
                    recent.binds.remove(idx);
                }
            }
            RecentWindowsMessage::SetBindKeyCombo(idx, combo) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.key_combo = combo;
                }
            }
            RecentWindowsMessage::SetBindIsNext(idx, is_next) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.is_next = is_next;
                }
            }
            RecentWindowsMessage::SetBindFilterAppId(idx, filter) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.filter_app_id = filter;
                }
            }
            RecentWindowsMessage::SetBindScope(idx, scope) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.scope = scope;
                }
            }
            RecentWindowsMessage::SetBindCooldown(idx, cooldown) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.cooldown_ms = cooldown;
                }
            }
        }

        drop(settings);
        self.dirty_tracker.mark(SettingsCategory::RecentWindows);
        self.save_manager.mark_changed();
        Task::none()
    }
}
