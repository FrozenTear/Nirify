//! Debug settings view
//!
//! Shows advanced debug and development options for troubleshooting niri.

use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::DebugSettings;
use crate::messages::{DebugMessage, Message};

/// Creates the debug settings view
pub fn view(settings: &DebugSettings) -> Element<'static, Message> {
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
        section_header("Debug Settings"),
        info_text(
            "Advanced settings for debugging and development. \
             Most users won't need to change these."
        ),
        spacer(16.0),

        // Expert Mode
        subsection_header("Expert Mode"),
        toggle_setting("Expert Mode", expert_mode, DebugMessage::SetExpertMode),
        info_text("Enable to show potentially dangerous advanced settings across the app."),
        spacer(16.0),

        // Rendering Options
        subsection_header("Rendering Options"),
        toggle_setting("Preview Render", preview_render, DebugMessage::SetPreviewRender),
        toggle_setting("Enable Overlay Planes", enable_overlay_planes, DebugMessage::SetEnableOverlayPlanes),
        toggle_setting("Disable Cursor Plane", disable_cursor_plane, DebugMessage::SetDisableCursorPlane),
        toggle_setting("Disable Direct Scanout", disable_direct_scanout, DebugMessage::SetDisableDirectScanout),
        toggle_setting("Restrict Scanout to Matching Format", restrict_scanout, DebugMessage::SetRestrictPrimaryScanoutToMatchingFormat),
        spacer(16.0),

        // Device Configuration
        subsection_header("Device Configuration"),
        string_setting("Render DRM Device", &render_drm, "e.g., /dev/dri/renderD128", |s| {
            DebugMessage::SetRenderDrmDevice(if s.is_empty() { None } else { Some(s) })
        }),
    ]
    .spacing(4);

    // Ignored DRM devices list
    content = content.push(text("Ignored DRM Devices").size(14));
    if ignore_drm.is_empty() {
        content = content.push(text("None").size(13).color([0.5, 0.5, 0.5]));
    } else {
        for (idx, device) in ignore_drm.iter().enumerate() {
            content = content.push(
                row![
                    text(device.clone()).size(13).width(Length::Fill),
                    button(text("×").size(14))
                        .on_press(Message::Debug(DebugMessage::RemoveIgnoreDrmDevice(idx)))
                        .padding([2, 8])
                        .style(delete_button_style),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
            );
        }
    }
    content = content.push(spacer(16.0));

    content = content.push(column![
        // Performance & Synchronization
        subsection_header("Performance & Synchronization"),
        toggle_setting("Wait for Frame Completion", wait_frame, DebugMessage::SetWaitForFrameCompletionBeforeQueueing),
        toggle_setting("Disable Resize Throttling", disable_resize, DebugMessage::SetDisableResizeThrottling),
        toggle_setting("Disable Transactions", disable_trans, DebugMessage::SetDisableTransactions),
        toggle_setting("Emulate Zero Presentation Time", emulate_zero, DebugMessage::SetEmulateZeroPresentationTime),
        toggle_setting("Skip Cursor-Only Updates (VRR)", skip_cursor, DebugMessage::SetSkipCursorOnlyUpdatesDuringVrr),
        spacer(16.0),

        // Hardware & Compatibility
        subsection_header("Hardware & Compatibility"),
        toggle_setting("D-Bus in Non-Session Instances", dbus_non_session, DebugMessage::SetDbusInterfacesInNonSessionInstances),
        toggle_setting("Keep Panel On (Lid Closed)", keep_panel, DebugMessage::SetKeepLaptopPanelOnWhenLidIsClosed),
        toggle_setting("Disable Monitor Names", disable_names, DebugMessage::SetDisableMonitorNames),
        toggle_setting("Force Disable Connectors on Resume", force_disable, DebugMessage::SetForceDisableConnectorsOnResume),
        spacer(16.0),

        // Window Behavior
        subsection_header("Window Behavior"),
        toggle_setting("Strict New Window Focus Policy", strict_focus, DebugMessage::SetStrictNewWindowFocusPolicy),
        toggle_setting("Honor XDG Activation (Invalid Serial)", honor_xdg, DebugMessage::SetHonorXdgActivationWithInvalidSerial),
        toggle_setting("Deactivate Unfocused Windows", deactivate, DebugMessage::SetDeactivateUnfocusedWindows),
        spacer(16.0),

        // Screencasting
        subsection_header("Screencasting"),
        toggle_setting("Force PipeWire Invalid Modifier", force_pipewire, DebugMessage::SetForcePipewireInvalidModifier),
        spacer(16.0),
    ].spacing(4));

    scrollable(container(content).padding(20)).into()
}

/// Create a toggle setting row
fn toggle_setting<F>(label: &str, value: bool, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(bool) -> DebugMessage + 'static,
{
    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        toggler(value)
            .on_toggle(move |v| Message::Debug(msg_fn(v)))
            .size(20),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Create a string setting row
fn string_setting<F>(label: &str, value: &str, placeholder: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(String) -> DebugMessage + 'static,
{
    let value_owned = value.to_string();
    let placeholder_owned = placeholder.to_string();

    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        text_input(&placeholder_owned, &value_owned)
            .on_input(move |s| Message::Debug(msg_fn(s)))
            .padding(8)
            .width(Length::Fixed(250.0)),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Style for delete buttons
fn delete_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
        ..Default::default()
    }
}
