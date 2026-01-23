//! Debug settings view
//!
//! Shows advanced debug and development options for troubleshooting niri.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::PreviewRenderMode;
use crate::config::models::DebugSettings;
use crate::messages::{DebugMessage, Message};

/// Creates the debug settings view
pub fn view(settings: &DebugSettings) -> Element<'_, Message> {
    // Clone values for closures
    let expert_mode = settings.expert_mode;
    let preview_render = settings.preview_render;
    let enable_overlay_planes = settings.enable_overlay_planes;
    let disable_cursor_plane = settings.disable_cursor_plane;
    let disable_direct_scanout = settings.disable_direct_scanout;
    let restrict_scanout = settings.restrict_primary_scanout_to_matching_format;
    let render_drm = settings.render_drm_device.clone().unwrap_or_default();
    let ignore_drm = settings.ignore_drm_devices.clone();
    let wait_frame = settings.wait_for_frame_completion_before_queueing;
    let disable_resize = settings.disable_resize_throttling;
    let disable_trans = settings.disable_transactions;
    let emulate_zero = settings.emulate_zero_presentation_time;
    let skip_cursor = settings.skip_cursor_only_updates_during_vrr;
    let dbus_non_session = settings.dbus_interfaces_in_non_session_instances;
    let keep_panel = settings.keep_laptop_panel_on_when_lid_is_closed;
    let disable_names = settings.disable_monitor_names;
    let force_disable = settings.force_disable_connectors_on_resume;
    let strict_focus = settings.strict_new_window_focus_policy;
    let honor_xdg = settings.honor_xdg_activation_with_invalid_serial;
    let deactivate = settings.deactivate_unfocused_windows;
    let force_pipewire = settings.force_pipewire_invalid_modifier;

    let mut content = column![
        page_title("Debug Settings"),
        info_text(
            "Advanced settings for debugging and development. \
             Most users won't need to change these."
        ),
        subsection_header("Expert Mode"),
        toggle_row(
            "Expert Mode",
            "Enable to show potentially dangerous advanced settings across the app",
            expert_mode,
            |v| Message::Debug(DebugMessage::SetExpertMode(v)),
        ),
        subsection_header("Rendering Options"),
        picker_row(
            "Preview Render",
            "Render monitors as if recording to test screencast appearance",
            PreviewRenderMode::all(),
            Some(preview_render),
            |mode| Message::Debug(DebugMessage::SetPreviewRender(mode)),
        ),
        toggle_row(
            "Enable Overlay Planes",
            "Enable direct scanout into overlay planes",
            enable_overlay_planes,
            |v| Message::Debug(DebugMessage::SetEnableOverlayPlanes(v)),
        ),
        toggle_row(
            "Disable Cursor Plane",
            "Disable cursor plane usage (may improve compatibility)",
            disable_cursor_plane,
            |v| Message::Debug(DebugMessage::SetDisableCursorPlane(v)),
        ),
        toggle_row(
            "Disable Direct Scanout",
            "Disable direct scanout to primary and overlay planes",
            disable_direct_scanout,
            |v| Message::Debug(DebugMessage::SetDisableDirectScanout(v)),
        ),
        toggle_row(
            "Restrict Scanout to Matching Format",
            "Only scanout when buffer format matches composition format",
            restrict_scanout,
            |v| Message::Debug(DebugMessage::SetRestrictPrimaryScanoutToMatchingFormat(v)),
        ),
        subsection_header("Device Configuration"),
        // Custom text input to avoid lifetime issues with render_drm
        column![
            text("Render DRM Device").size(16),
            text("Override DRM device for rendering (e.g., /dev/dri/renderD128)").size(12).color([0.7, 0.7, 0.7]),
            text_input("", &render_drm)
                .on_input(|s| Message::Debug(DebugMessage::SetRenderDrmDevice(if s.is_empty() { None } else { Some(s) })))
                .padding(8),
        ]
        .spacing(6)
        .padding(12),
    ]
    .spacing(4);

    // Ignored DRM devices list
    content = content.push(
        container(
            column![
                text("Ignored DRM Devices").size(16),
                text("DRM devices to ignore (useful for GPU passthrough)").size(12).color([0.7, 0.7, 0.7]),
            ].spacing(4)
        ).padding(12)
    );

    if ignore_drm.is_empty() {
        content = content.push(
            container(text("None configured").size(14).color([0.5, 0.5, 0.5])).padding([0, 12])
        );
    } else {
        for (idx, device) in ignore_drm.iter().enumerate() {
            content = content.push(
                container(
                    row![
                        text(device.clone()).size(14).width(Length::Fill),
                        button(text("Remove").size(12))
                            .on_press(Message::Debug(DebugMessage::RemoveIgnoreDrmDevice(idx)))
                            .padding([4, 12])
                            .style(delete_button_style),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                ).padding([4, 12])
            );
        }
    }

    content = content.push(column![
        subsection_header("Performance & Synchronization"),
        toggle_row(
            "Wait for Frame Completion",
            "Wait until every frame is done rendering before handing to DRM",
            wait_frame,
            |v| Message::Debug(DebugMessage::SetWaitForFrameCompletionBeforeQueueing(v)),
        ),
        toggle_row(
            "Disable Resize Throttling",
            "Send resize events as quickly as possible",
            disable_resize,
            |v| Message::Debug(DebugMessage::SetDisableResizeThrottling(v)),
        ),
        toggle_row(
            "Disable Transactions",
            "Disable synchronized window resizing",
            disable_trans,
            |v| Message::Debug(DebugMessage::SetDisableTransactions(v)),
        ),
        toggle_row(
            "Emulate Zero Presentation Time",
            "Simulate unknown presentation time",
            emulate_zero,
            |v| Message::Debug(DebugMessage::SetEmulateZeroPresentationTime(v)),
        ),
        toggle_row(
            "Skip Cursor-Only Updates (VRR)",
            "Skip redraws caused by cursor movement during VRR",
            skip_cursor,
            |v| Message::Debug(DebugMessage::SetSkipCursorOnlyUpdatesDuringVrr(v)),
        ),
        subsection_header("Hardware & Compatibility"),
        toggle_row(
            "D-Bus in Non-Session Instances",
            "Create D-Bus interfaces in non-session instances",
            dbus_non_session,
            |v| Message::Debug(DebugMessage::SetDbusInterfacesInNonSessionInstances(v)),
        ),
        toggle_row(
            "Keep Panel On (Lid Closed)",
            "Keep laptop panel on when lid is closed",
            keep_panel,
            |v| Message::Debug(DebugMessage::SetKeepLaptopPanelOnWhenLidIsClosed(v)),
        ),
        toggle_row(
            "Disable Monitor Names",
            "Disable EDID monitor name reading",
            disable_names,
            |v| Message::Debug(DebugMessage::SetDisableMonitorNames(v)),
        ),
        toggle_row(
            "Force Disable Connectors on Resume",
            "Force blank all outputs on TTY switch/resume",
            force_disable,
            |v| Message::Debug(DebugMessage::SetForceDisableConnectorsOnResume(v)),
        ),
        subsection_header("Window Behavior"),
        toggle_row(
            "Strict New Window Focus Policy",
            "Only focus windows with valid xdg-activation token",
            strict_focus,
            |v| Message::Debug(DebugMessage::SetStrictNewWindowFocusPolicy(v)),
        ),
        toggle_row(
            "Honor XDG Activation (Invalid Serial)",
            "Allow focus via invalid xdg-activation serial",
            honor_xdg,
            |v| Message::Debug(DebugMessage::SetHonorXdgActivationWithInvalidSerial(v)),
        ),
        toggle_row(
            "Deactivate Unfocused Windows",
            "Drop activated state for unfocused windows",
            deactivate,
            |v| Message::Debug(DebugMessage::SetDeactivateUnfocusedWindows(v)),
        ),
        subsection_header("Screencasting"),
        toggle_row(
            "Force PipeWire Invalid Modifier",
            "Force invalid DRM modifier for PipeWire",
            force_pipewire,
            |v| Message::Debug(DebugMessage::SetForcePipewireInvalidModifier(v)),
        ),
        spacer(32.0),
    ].spacing(4));

    scrollable(container(content).padding(20).width(Length::Fill))
        .height(Length::Fill)
        .into()
}
