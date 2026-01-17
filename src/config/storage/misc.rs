//! Miscellaneous settings KDL generation
//!
//! Generates KDL for prefer-no-csd, screenshot-path, clipboard, etc.

use super::builder::KdlBuilder;
use crate::config::models::{MiscSettings, XWaylandSatelliteConfig};

/// Generate misc.kdl content from settings.
///
/// Creates KDL configuration for miscellaneous settings including:
/// - Prefer no CSD
/// - Screenshot path
/// - Disable primary clipboard
/// - Hotkey overlay skip at startup
pub fn generate_misc_kdl(settings: &MiscSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Miscellaneous settings - managed by niri-settings-rust");

    kdl.optional_flag("prefer-no-csd", settings.prefer_no_csd);
    kdl.field_string_if_not_empty("screenshot-path", &settings.screenshot_path);

    if settings.disable_primary_clipboard {
        kdl.block("clipboard", |b| {
            b.flag("disable-primary");
        });
    }

    // Hotkey overlay settings (v25.08+)
    kdl.block("hotkey-overlay", |b| {
        b.optional_flag("skip-at-startup", settings.hotkey_overlay_skip_at_startup);
        b.optional_flag("hide-not-bound", settings.hotkey_overlay_hide_not_bound);
    });

    // Config notification settings (v25.08+)
    kdl.block_if(
        "config-notification",
        settings.config_notification_disable_failed,
        |b| {
            b.flag("disable-failed");
        },
    );

    // Note: spawn-sh-at-startup requires a command argument, not a flag.
    // The current boolean model is incorrect - this feature needs redesign.
    // For now, we don't output it to avoid parsing errors.

    // XWayland satellite settings (v25.08+)
    match &settings.xwayland_satellite {
        XWaylandSatelliteConfig::Default => {} // Don't output, use niri defaults
        XWaylandSatelliteConfig::Off => {
            kdl.block("xwayland-satellite", |b| {
                b.flag("off");
            });
        }
        XWaylandSatelliteConfig::CustomPath(path) => {
            kdl.field_string("xwayland-satellite", path);
        }
    }

    kdl.build()
}
