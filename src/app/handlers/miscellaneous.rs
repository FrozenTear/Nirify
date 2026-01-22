//! Miscellaneous settings message handler

use crate::config::SettingsCategory;
use crate::messages::{MiscellaneousMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle miscellaneous settings messages
    pub(in crate::app) fn update_miscellaneous(&mut self, msg: MiscellaneousMessage) -> Task<Message> {
        let mut settings = self.settings.lock().expect("settings mutex poisoned");
        let misc = &mut settings.miscellaneous;

        match msg {
            MiscellaneousMessage::SetPreferNoCsd(v) => misc.prefer_no_csd = v,
            MiscellaneousMessage::SetScreenshotPath(v) => misc.screenshot_path = v,
            MiscellaneousMessage::SetDisablePrimaryClipboard(v) => misc.disable_primary_clipboard = v,
            MiscellaneousMessage::SetHotkeyOverlaySkipAtStartup(v) => misc.hotkey_overlay_skip_at_startup = v,
            MiscellaneousMessage::SetHotkeyOverlayHideNotBound(v) => misc.hotkey_overlay_hide_not_bound = v,
            MiscellaneousMessage::SetConfigNotificationDisableFailed(v) => misc.config_notification_disable_failed = v,
            MiscellaneousMessage::SetSpawnShAtStartup(v) => misc.spawn_sh_at_startup = v,
            MiscellaneousMessage::SetXWaylandSatellite(v) => misc.xwayland_satellite = v,
        }

        drop(settings);
        self.dirty_tracker.mark(SettingsCategory::Miscellaneous);
        self.save_manager.mark_changed();
        Task::none()
    }
}
