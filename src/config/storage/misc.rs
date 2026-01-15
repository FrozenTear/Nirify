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
        kdl.raw("clipboard { disable-primary }");
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

    // Spawn commands through shell (v25.08+)
    kdl.optional_flag("spawn-sh-at-startup", settings.spawn_sh_at_startup);

    // XWayland satellite settings (v25.08+)
    match &settings.xwayland_satellite {
        XWaylandSatelliteConfig::Default => {} // Don't output, use niri defaults
        XWaylandSatelliteConfig::Off => {
            kdl.raw("xwayland-satellite { off }");
        }
        XWaylandSatelliteConfig::CustomPath(path) => {
            kdl.field_string("xwayland-satellite", path);
        }
    }

    kdl.build()
}
