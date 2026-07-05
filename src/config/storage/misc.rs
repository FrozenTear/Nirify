//! Miscellaneous settings KDL generation
//!
//! Generates KDL for prefer-no-csd, screenshot-path, clipboard, etc.

use super::builder::KdlBuilder;
use crate::config::models::{MiscSettings, ScreenshotPathConfig, XWaylandSatelliteConfig};

/// Generate misc.kdl content from settings.
///
/// Creates KDL configuration for miscellaneous settings including:
/// - Prefer no CSD
/// - Screenshot path
/// - Disable primary clipboard
/// - Hotkey overlay skip at startup
pub fn generate_misc_kdl(settings: &MiscSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Miscellaneous settings - managed by Nirify");

    kdl.optional_flag("prefer-no-csd", settings.prefer_no_csd);
    match &settings.screenshot_path {
        ScreenshotPathConfig::Default => {} // Omit node, niri uses its default path
        ScreenshotPathConfig::Disabled => {
            kdl.raw("screenshot-path null");
        }
        ScreenshotPathConfig::Custom(path) => {
            if !path.is_empty() {
                kdl.field_string("screenshot-path", path);
            }
        }
    }

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

    // Spawn commands through shell at startup (v25.08+); one node per command
    for cmd in &settings.spawn_sh_at_startup {
        if !cmd.command.is_empty() {
            kdl.field_string("spawn-sh-at-startup", &cmd.command);
        }
    }

    // XWayland satellite settings (v25.08+)
    match &settings.xwayland_satellite {
        XWaylandSatelliteConfig::Default => {} // Don't output, use niri defaults
        XWaylandSatelliteConfig::Off => {
            kdl.block("xwayland-satellite", |b| {
                b.flag("off");
            });
        }
        XWaylandSatelliteConfig::CustomPath(path) => {
            kdl.block("xwayland-satellite", |b| {
                b.field_string("path", path);
            });
        }
    }

    kdl.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{Settings, SpawnShCommand};

    fn roundtrip_misc(misc: &MiscSettings) -> (String, MiscSettings) {
        let kdl = generate_misc_kdl(misc);
        let doc: kdl::KdlDocument = kdl.parse().expect("generated misc KDL must re-parse");
        let mut settings = Settings::default();
        crate::config::loader::parse_misc_from_doc(&doc, &mut settings);
        (kdl, settings.miscellaneous)
    }

    #[test]
    fn test_xwayland_path_round_trip() {
        let misc = MiscSettings {
            xwayland_satellite: XWaylandSatelliteConfig::CustomPath("/usr/bin/xws".into()),
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_misc(&misc);
        assert!(kdl.contains("path \"/usr/bin/xws\""), "{kdl}");
        // Must NOT emit a positional arg on the node itself.
        assert!(!kdl.contains("xwayland-satellite \""), "{kdl}");
        assert_eq!(
            loaded.xwayland_satellite,
            XWaylandSatelliteConfig::CustomPath("/usr/bin/xws".into())
        );
    }

    #[test]
    fn test_xwayland_legacy_string_load() {
        let doc: kdl::KdlDocument = "xwayland-satellite \"/old/path\"".parse().unwrap();
        let mut settings = Settings::default();
        crate::config::loader::parse_misc_from_doc(&doc, &mut settings);
        assert_eq!(
            settings.miscellaneous.xwayland_satellite,
            XWaylandSatelliteConfig::CustomPath("/old/path".into())
        );
    }

    #[test]
    fn test_spawn_sh_multiple_round_trip() {
        let misc = MiscSettings {
            spawn_sh_at_startup: vec![
                SpawnShCommand {
                    id: 0,
                    command: "waybar".into(),
                },
                SpawnShCommand {
                    id: 1,
                    command: "qs -c ~/.config/quickshell".into(),
                },
                SpawnShCommand {
                    id: 2,
                    command: "mako".into(),
                },
            ],
            spawn_sh_next_id: 3,
            ..Default::default()
        };
        let (_kdl, loaded) = roundtrip_misc(&misc);
        let cmds: Vec<&str> = loaded
            .spawn_sh_at_startup
            .iter()
            .map(|c| c.command.as_str())
            .collect();
        assert_eq!(cmds, vec!["waybar", "qs -c ~/.config/quickshell", "mako"]);
    }

    #[test]
    fn test_screenshot_path_null_round_trip() {
        // Disabled
        let misc = MiscSettings {
            screenshot_path: ScreenshotPathConfig::Disabled,
            ..Default::default()
        };
        let (kdl, loaded) = roundtrip_misc(&misc);
        assert!(kdl.contains("screenshot-path null"), "{kdl}");
        assert_eq!(loaded.screenshot_path, ScreenshotPathConfig::Disabled);

        // Custom
        let misc = MiscSettings {
            screenshot_path: ScreenshotPathConfig::Custom("~/shots/%Y.png".into()),
            ..Default::default()
        };
        let (_kdl, loaded) = roundtrip_misc(&misc);
        assert_eq!(
            loaded.screenshot_path,
            ScreenshotPathConfig::Custom("~/shots/%Y.png".into())
        );

        // Default -> no node
        let misc = MiscSettings::default();
        let (kdl, loaded) = roundtrip_misc(&misc);
        assert!(!kdl.contains("screenshot-path"), "{kdl}");
        assert_eq!(loaded.screenshot_path, ScreenshotPathConfig::Default);
    }
}
