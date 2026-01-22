//! Miscellaneous settings view
//!
//! Shows various settings that don't fit into other categories.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::{MiscSettings, XWaylandSatelliteConfig};
use crate::messages::Message;
use crate::theme::fonts;

/// Creates the miscellaneous settings view
pub fn view(settings: &MiscSettings) -> Element<'static, Message> {
    let xwayland_display = match &settings.xwayland_satellite {
        XWaylandSatelliteConfig::Default => "Default".to_string(),
        XWaylandSatelliteConfig::Off => "Disabled".to_string(),
        XWaylandSatelliteConfig::CustomPath(path) => format!("Custom: {}", path),
    };

    let content = column![
        section_header("Miscellaneous Settings"),
        info_text(
            "Additional niri settings that don't fit into other categories."
        ),
        spacer(16.0),

        // Window Decorations
        subsection_header("Window Decorations"),
        display_toggle("Prefer No Client-Side Decorations", settings.prefer_no_csd),
        info_text("Ask applications to use server-side (compositor) decorations when possible."),
        spacer(16.0),

        // Screenshots
        subsection_header("Screenshots"),
        display_value("Screenshot Path", &settings.screenshot_path),
        info_text("Path pattern for saved screenshots. Supports strftime format codes."),
        spacer(16.0),

        // Clipboard
        subsection_header("Clipboard"),
        display_toggle("Disable Primary Clipboard", settings.disable_primary_clipboard),
        info_text("Disable the middle-click paste (primary selection) clipboard."),
        spacer(16.0),

        // Hotkey Overlay
        subsection_header("Hotkey Overlay"),
        display_toggle("Skip at Startup", settings.hotkey_overlay_skip_at_startup),
        display_toggle("Hide Unbound Actions", settings.hotkey_overlay_hide_not_bound),
        info_text("Controls the hotkey overlay that shows available keyboard shortcuts."),
        spacer(16.0),

        // Startup Behavior
        subsection_header("Startup Behavior"),
        display_toggle("Spawn Through Shell at Startup", settings.spawn_sh_at_startup),
        info_text("Execute startup commands through the shell (enables shell features like ~)."),
        spacer(16.0),

        // Notifications
        subsection_header("Notifications"),
        display_toggle("Disable Config Parse Failed Notification", settings.config_notification_disable_failed),
        spacer(16.0),

        // XWayland
        subsection_header("XWayland"),
        display_value("XWayland Satellite", &xwayland_display),
        info_text("Configuration for xwayland-satellite (X11 compatibility layer)."),
        spacer(16.0),

        info_text("Edit miscellaneous.kdl directly to change these settings. Full editing UI coming in a future update."),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}

/// Display a toggle value (read-only)
fn display_toggle(label: &str, value: bool) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        text(if value { "Yes" } else { "No" })
            .size(14)
            .color(if value { [0.5, 0.8, 0.5] } else { [0.6, 0.6, 0.6] }),
    ]
    .spacing(16)
    .into()
}

/// Display a value (read-only)
fn display_value(label: &str, value: &str) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        text(value.to_string()).size(14).font(fonts::MONO_FONT).color([0.8, 0.8, 0.8]),
    ]
    .spacing(16)
    .into()
}
