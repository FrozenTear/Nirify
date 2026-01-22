//! Debug settings view
//!
//! Shows advanced debug and development options for troubleshooting niri.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::DebugSettings;
use crate::messages::Message;

/// Creates the debug settings view
pub fn view(settings: &DebugSettings) -> Element<'static, Message> {
    let content = column![
        section_header("Debug Settings"),
        info_text(
            "Advanced settings for debugging and development. \
             Most users won't need to change these."
        ),
        spacer(16.0),

        // Expert Mode
        subsection_header("Expert Mode"),
        display_toggle("Expert Mode", settings.expert_mode),
        info_text("Enable to show potentially dangerous advanced settings across the app."),
        spacer(16.0),

        // Rendering Options
        subsection_header("Rendering Options"),
        display_toggle("Preview Render", settings.preview_render),
        display_toggle("Enable Overlay Planes", settings.enable_overlay_planes),
        display_toggle("Disable Cursor Plane", settings.disable_cursor_plane),
        display_toggle("Disable Direct Scanout", settings.disable_direct_scanout),
        display_toggle("Restrict Primary Scanout to Matching Format", settings.restrict_primary_scanout_to_matching_format),
        spacer(16.0),

        // Device Configuration
        subsection_header("Device Configuration"),
        display_optional("Render DRM Device", settings.render_drm_device.as_deref()),
        display_list("Ignored DRM Devices", &settings.ignore_drm_devices),
        spacer(16.0),

        // Performance & Synchronization
        subsection_header("Performance & Synchronization"),
        display_toggle("Wait for Frame Completion Before Queueing", settings.wait_for_frame_completion_before_queueing),
        display_toggle("Disable Resize Throttling", settings.disable_resize_throttling),
        display_toggle("Disable Transactions", settings.disable_transactions),
        display_toggle("Emulate Zero Presentation Time", settings.emulate_zero_presentation_time),
        display_toggle("Skip Cursor-Only Updates During VRR", settings.skip_cursor_only_updates_during_vrr),
        spacer(16.0),

        // Hardware & Compatibility
        subsection_header("Hardware & Compatibility"),
        display_toggle("D-Bus Interfaces in Non-Session Instances", settings.dbus_interfaces_in_non_session_instances),
        display_toggle("Keep Laptop Panel On When Lid is Closed", settings.keep_laptop_panel_on_when_lid_is_closed),
        display_toggle("Disable Monitor Names", settings.disable_monitor_names),
        display_toggle("Force Disable Connectors on Resume", settings.force_disable_connectors_on_resume),
        spacer(16.0),

        // Window Behavior
        subsection_header("Window Behavior"),
        display_toggle("Strict New Window Focus Policy", settings.strict_new_window_focus_policy),
        display_toggle("Honor XDG Activation with Invalid Serial", settings.honor_xdg_activation_with_invalid_serial),
        display_toggle("Deactivate Unfocused Windows", settings.deactivate_unfocused_windows),
        spacer(16.0),

        // Screencasting
        subsection_header("Screencasting"),
        display_toggle("Force PipeWire Invalid Modifier", settings.force_pipewire_invalid_modifier),
        spacer(16.0),

        info_text("Edit debug.kdl directly to change these settings. Full editing UI coming in a future update."),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}

/// Display a toggle value (read-only)
fn display_toggle(label: &str, value: bool) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(350.0)),
        text(if value { "Yes" } else { "No" })
            .size(14)
            .color(if value { [0.5, 0.8, 0.5] } else { [0.6, 0.6, 0.6] }),
    ]
    .spacing(16)
    .into()
}

/// Display an optional string value (read-only)
fn display_optional(label: &str, value: Option<&str>) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(350.0)),
        text(value.unwrap_or("Not set").to_string())
            .size(14)
            .color(if value.is_some() { [0.8, 0.8, 0.8] } else { [0.5, 0.5, 0.5] }),
    ]
    .spacing(16)
    .into()
}

/// Display a list of strings (read-only)
fn display_list(label: &str, values: &[String]) -> Element<'static, Message> {
    let display = if values.is_empty() {
        "None".to_string()
    } else {
        values.join(", ")
    };

    row![
        text(label.to_string()).size(14).width(Length::Fixed(350.0)),
        text(display)
            .size(14)
            .color(if values.is_empty() { [0.5, 0.5, 0.5] } else { [0.8, 0.8, 0.8] }),
    ]
    .spacing(16)
    .into()
}
