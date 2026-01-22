//! Miscellaneous settings view
//!
//! Shows various settings that don't fit into other categories.

use iced::widget::{column, container, pick_list, row, scrollable, text, text_input, toggler};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::{MiscSettings, XWaylandSatelliteConfig};
use crate::messages::{Message, MiscellaneousMessage};
use crate::theme::fonts;

/// Creates the miscellaneous settings view
pub fn view(settings: &MiscSettings) -> Element<'static, Message> {
    // Clone values for closures
    let prefer_no_csd = settings.prefer_no_csd;
    let screenshot_path = settings.screenshot_path.clone();
    let disable_clipboard = settings.disable_primary_clipboard;
    let hotkey_skip = settings.hotkey_overlay_skip_at_startup;
    let hotkey_hide = settings.hotkey_overlay_hide_not_bound;
    let config_disable = settings.config_notification_disable_failed;
    let spawn_sh = settings.spawn_sh_at_startup;
    let xwayland = settings.xwayland_satellite.clone();

    let xwayland_options = vec![
        XWaylandSatelliteConfig::Default,
        XWaylandSatelliteConfig::Off,
    ];

    let content = column![
        section_header("Miscellaneous Settings"),
        info_text(
            "Additional niri settings that don't fit into other categories."
        ),
        spacer(16.0),

        // Window Decorations
        subsection_header("Window Decorations"),
        toggle_setting("Prefer No Client-Side Decorations", prefer_no_csd, MiscellaneousMessage::SetPreferNoCsd),
        info_text("Ask applications to use server-side (compositor) decorations when possible."),
        spacer(16.0),

        // Screenshots
        subsection_header("Screenshots"),
        string_setting("Screenshot Path", &screenshot_path, "~/Pictures/Screenshots/%Y-%m-%d_%H-%M-%S.png", MiscellaneousMessage::SetScreenshotPath),
        info_text("Path pattern for saved screenshots. Supports strftime format codes."),
        spacer(16.0),

        // Clipboard
        subsection_header("Clipboard"),
        toggle_setting("Disable Primary Clipboard", disable_clipboard, MiscellaneousMessage::SetDisablePrimaryClipboard),
        info_text("Disable the middle-click paste (primary selection) clipboard."),
        spacer(16.0),

        // Hotkey Overlay
        subsection_header("Hotkey Overlay"),
        toggle_setting("Skip at Startup", hotkey_skip, MiscellaneousMessage::SetHotkeyOverlaySkipAtStartup),
        toggle_setting("Hide Unbound Actions", hotkey_hide, MiscellaneousMessage::SetHotkeyOverlayHideNotBound),
        info_text("Controls the hotkey overlay that shows available keyboard shortcuts."),
        spacer(16.0),

        // Startup Behavior
        subsection_header("Startup Behavior"),
        toggle_setting("Spawn Through Shell at Startup", spawn_sh, MiscellaneousMessage::SetSpawnShAtStartup),
        info_text("Execute startup commands through the shell (enables shell features like ~)."),
        spacer(16.0),

        // Notifications
        subsection_header("Notifications"),
        toggle_setting("Disable Config Parse Failed Notification", config_disable, MiscellaneousMessage::SetConfigNotificationDisableFailed),
        spacer(16.0),

        // XWayland
        subsection_header("XWayland"),
        xwayland_setting(&xwayland, xwayland_options),
        info_text("Configuration for xwayland-satellite (X11 compatibility layer)."),
        spacer(16.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}

/// Create a toggle setting row
fn toggle_setting<F>(label: &str, value: bool, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(bool) -> MiscellaneousMessage + 'static,
{
    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        toggler(value)
            .on_toggle(move |v| Message::Miscellaneous(msg_fn(v)))
            .size(20),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Create a string setting row
fn string_setting<F>(label: &str, value: &str, placeholder: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(String) -> MiscellaneousMessage + 'static,
{
    let value_owned = value.to_string();
    let placeholder_owned = placeholder.to_string();

    row![
        text(label.to_string()).size(14).width(Length::Fixed(300.0)),
        text_input(&placeholder_owned, &value_owned)
            .on_input(move |s| Message::Miscellaneous(msg_fn(s)))
            .padding(8)
            .font(fonts::MONO_FONT)
            .width(Length::Fill),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Create the XWayland satellite picker
fn xwayland_setting(current: &XWaylandSatelliteConfig, options: Vec<XWaylandSatelliteConfig>) -> Element<'static, Message> {
    let display_name = |config: &XWaylandSatelliteConfig| -> String {
        match config {
            XWaylandSatelliteConfig::Default => "Default".to_string(),
            XWaylandSatelliteConfig::Off => "Disabled".to_string(),
            XWaylandSatelliteConfig::CustomPath(p) => format!("Custom: {}", p),
        }
    };

    let current_display = display_name(current);
    let is_custom = matches!(current, XWaylandSatelliteConfig::CustomPath(_));
    let custom_path = match current {
        XWaylandSatelliteConfig::CustomPath(p) => p.clone(),
        _ => String::new(),
    };

    let mut content = column![
        row![
            text("XWayland Satellite").size(14).width(Length::Fixed(300.0)),
            pick_list(
                options.clone(),
                Some(if is_custom { XWaylandSatelliteConfig::Default } else { current.clone() }),
                |selected| Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(selected)),
            )
            .placeholder(&current_display)
            .width(Length::Fixed(200.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // Show custom path input if currently using custom
    if is_custom {
        content = content.push(
            row![
                text("Custom Path").size(14).width(Length::Fixed(300.0)),
                text_input("Path to xwayland-satellite", &custom_path)
                    .on_input(|s| Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(
                        XWaylandSatelliteConfig::CustomPath(s)
                    )))
                    .padding(8)
                    .font(fonts::MONO_FONT)
                    .width(Length::Fill),
            ]
            .spacing(16)
            .align_y(iced::Alignment::Center)
        );
    }

    content.into()
}

// Implement Display for XWaylandSatelliteConfig for pick_list
impl std::fmt::Display for XWaylandSatelliteConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XWaylandSatelliteConfig::Default => write!(f, "Default"),
            XWaylandSatelliteConfig::Off => write!(f, "Disabled"),
            XWaylandSatelliteConfig::CustomPath(p) => write!(f, "Custom: {}", p),
        }
    }
}
