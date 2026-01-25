//! Debug settings message handler

use crate::config::SettingsCategory;
use crate::messages::{DebugMessage, Message};
use iced::Task;

impl super::super::App {
    /// Handle debug settings messages
    pub(in crate::app) fn update_debug(&mut self, msg: DebugMessage) -> Task<Message> {
        
        let debug = &mut self.settings.debug;

        match msg {
            DebugMessage::SetExpertMode(v) => debug.expert_mode = v,
            DebugMessage::SetPreviewRender(v) => debug.preview_render = v,
            DebugMessage::SetEnableOverlayPlanes(v) => debug.enable_overlay_planes = v,
            DebugMessage::SetDisableCursorPlane(v) => debug.disable_cursor_plane = v,
            DebugMessage::SetDisableDirectScanout(v) => debug.disable_direct_scanout = v,
            DebugMessage::SetRestrictPrimaryScanoutToMatchingFormat(v) => debug.restrict_primary_scanout_to_matching_format = v,
            DebugMessage::SetRenderDrmDevice(v) => debug.render_drm_device = v,
            DebugMessage::AddIgnoreDrmDevice(device) => {
                if !device.is_empty() {
                    debug.ignore_drm_devices.push(device);
                }
            }
            DebugMessage::RemoveIgnoreDrmDevice(idx) => {
                if idx < debug.ignore_drm_devices.len() {
                    debug.ignore_drm_devices.remove(idx);
                }
            }
            DebugMessage::SetWaitForFrameCompletionBeforeQueueing(v) => debug.wait_for_frame_completion_before_queueing = v,
            DebugMessage::SetDisableResizeThrottling(v) => debug.disable_resize_throttling = v,
            DebugMessage::SetDisableTransactions(v) => debug.disable_transactions = v,
            DebugMessage::SetEmulateZeroPresentationTime(v) => debug.emulate_zero_presentation_time = v,
            DebugMessage::SetSkipCursorOnlyUpdatesDuringVrr(v) => debug.skip_cursor_only_updates_during_vrr = v,
            DebugMessage::SetDbusInterfacesInNonSessionInstances(v) => debug.dbus_interfaces_in_non_session_instances = v,
            DebugMessage::SetKeepLaptopPanelOnWhenLidIsClosed(v) => debug.keep_laptop_panel_on_when_lid_is_closed = v,
            DebugMessage::SetDisableMonitorNames(v) => debug.disable_monitor_names = v,
            DebugMessage::SetForceDisableConnectorsOnResume(v) => debug.force_disable_connectors_on_resume = v,
            DebugMessage::SetStrictNewWindowFocusPolicy(v) => debug.strict_new_window_focus_policy = v,
            DebugMessage::SetHonorXdgActivationWithInvalidSerial(v) => debug.honor_xdg_activation_with_invalid_serial = v,
            DebugMessage::SetDeactivateUnfocusedWindows(v) => debug.deactivate_unfocused_windows = v,
            DebugMessage::SetForcePipewireInvalidModifier(v) => debug.force_pipewire_invalid_modifier = v,
        }

        self.save.dirty_tracker.mark(SettingsCategory::Debug);
        self.mark_changed();
        Task::none()
    }
}
