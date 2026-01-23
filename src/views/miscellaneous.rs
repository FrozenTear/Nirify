//! Miscellaneous settings view
//!
//! Shows various settings that don't fit into other categories.

use iced::widget::{column, container, pick_list, row, scrollable, text, text_input};
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
        page_title("Miscellaneous Settings"),
        info_text(
            "Additional niri settings that don't fit into other categories."
        ),

        // Window Decorations
        subsection_header("Window Decorations"),
        toggle_row(
            "Prefer No Client-Side Decorations",
            "Ask applications to use server-side (compositor) decorations when possible",
            prefer_no_csd,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetPreferNoCsd(v)),
        ),

        // Screenshots
        subsection_header("Screenshots"),
        // Custom text input row for screenshot path (owned string)
        column![
            text("Screenshot Path").size(16),
            text("Path pattern for saved screenshots. Supports strftime format codes.").size(12).color([0.7, 0.7, 0.7]),
            text_input("~/Pictures/Screenshots/%Y-%m-%d_%H-%M-%S.png", &screenshot_path)
                .on_input(|s| Message::Miscellaneous(MiscellaneousMessage::SetScreenshotPath(s)))
                .padding(8)
                .font(fonts::MONO_FONT),
        ]
        .spacing(6)
        .padding(12),

        // Clipboard
        subsection_header("Clipboard"),
        toggle_row(
            "Disable Primary Clipboard",
            "Disable the middle-click paste (primary selection) clipboard",
            disable_clipboard,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetDisablePrimaryClipboard(v)),
        ),

        // Hotkey Overlay
        subsection_header("Hotkey Overlay"),
        toggle_row(
            "Skip at Startup",
            "Don't show the hotkey overlay when niri starts",
            hotkey_skip,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetHotkeyOverlaySkipAtStartup(v)),
        ),
        toggle_row(
            "Hide Unbound Actions",
            "Hide actions that don't have a keybinding assigned",
            hotkey_hide,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetHotkeyOverlayHideNotBound(v)),
        ),

        // Startup Behavior
        subsection_header("Startup Behavior"),
        toggle_row(
            "Spawn Through Shell at Startup",
            "Execute startup commands through the shell (enables shell features like ~)",
            spawn_sh,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetSpawnShAtStartup(v)),
        ),

        // Notifications
        subsection_header("Notifications"),
        toggle_row(
            "Disable Config Parse Failed Notification",
            "Don't show notification when config parsing fails",
            config_disable,
            |v| Message::Miscellaneous(MiscellaneousMessage::SetConfigNotificationDisableFailed(v)),
        ),

        // XWayland
        subsection_header("XWayland"),
        xwayland_setting(&xwayland, xwayland_options),

        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
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
            column![
                text("XWayland Satellite").size(16),
                text("Configuration for xwayland-satellite (X11 compatibility layer)").size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(4)
            .width(Length::Fill),
            pick_list(
                options.clone(),
                Some(if is_custom { XWaylandSatelliteConfig::Default } else { current.clone() }),
                |selected| Message::Miscellaneous(MiscellaneousMessage::SetXWaylandSatellite(selected)),
            )
            .placeholder(&current_display)
            .width(Length::Fixed(200.0)),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8)
    .padding(12);

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
                    .font(crate::theme::fonts::MONO_FONT)
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
