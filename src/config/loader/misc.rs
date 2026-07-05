//! Miscellaneous settings loader
//!
//! Handles prefer-no-csd, screenshot-path, clipboard, hotkey overlay, etc.

use super::helpers::read_kdl_file;
use crate::config::models::{
    ScreenshotPathConfig, Settings, SpawnShCommand, XWaylandSatelliteConfig,
};
use crate::config::parser::{get_string, has_flag};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Parse miscellaneous settings from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_misc_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    if has_flag(doc, &["prefer-no-csd"]) {
        settings.miscellaneous.prefer_no_csd = true;
    }

    if let Some(sp_node) = doc.get("screenshot-path") {
        if let Some(entry) = sp_node.entries().iter().find(|e| e.name().is_none()) {
            if entry.value().is_null() {
                settings.miscellaneous.screenshot_path = ScreenshotPathConfig::Disabled;
            } else if let Some(s) = entry.value().as_string() {
                settings.miscellaneous.screenshot_path =
                    ScreenshotPathConfig::Custom(s.to_string());
            }
        }
    }

    // Clipboard
    if let Some(clip) = doc.get("clipboard") {
        if let Some(c_children) = clip.children() {
            if has_flag(c_children, &["disable-primary"]) {
                settings.miscellaneous.disable_primary_clipboard = true;
            }
        }
    }

    // Hotkey overlay
    if let Some(ho) = doc.get("hotkey-overlay") {
        if let Some(ho_children) = ho.children() {
            if has_flag(ho_children, &["skip-at-startup"]) {
                settings.miscellaneous.hotkey_overlay_skip_at_startup = true;
            }
            if has_flag(ho_children, &["hide-not-bound"]) {
                settings.miscellaneous.hotkey_overlay_hide_not_bound = true;
            }
        }
    }

    // Config notification
    if let Some(cn) = doc.get("config-notification") {
        if let Some(cn_children) = cn.children() {
            if has_flag(cn_children, &["disable-failed"]) {
                settings.miscellaneous.config_notification_disable_failed = true;
            }
        }
    }

    // Spawn commands through shell at startup (v25.08+); collect all nodes
    settings.miscellaneous.spawn_sh_at_startup.clear();
    let mut spawn_id = 0u32;
    for node in doc
        .nodes()
        .iter()
        .filter(|n| n.name().value() == "spawn-sh-at-startup")
    {
        if let Some(cmd) = node
            .entries()
            .iter()
            .find(|e| e.name().is_none())
            .and_then(|e| e.value().as_string())
        {
            settings
                .miscellaneous
                .spawn_sh_at_startup
                .push(SpawnShCommand {
                    id: spawn_id,
                    command: cmd.to_string(),
                });
            spawn_id += 1;
        }
    }
    settings.miscellaneous.spawn_sh_next_id = spawn_id;

    // XWayland satellite (v25.08+)
    if let Some(xws) = doc.get("xwayland-satellite") {
        // Block form: xwayland-satellite { off } / { path "..." } / { on }
        if let Some(xws_children) = xws.children() {
            if has_flag(xws_children, &["off"]) {
                if get_string(xws_children, &["path"]).is_some() {
                    debug!(
                        "xwayland-satellite has both `off` and `path`; dropping path (model is single-valued)"
                    );
                }
                settings.miscellaneous.xwayland_satellite = XWaylandSatelliteConfig::Off;
            } else if let Some(path) = get_string(xws_children, &["path"]) {
                settings.miscellaneous.xwayland_satellite =
                    XWaylandSatelliteConfig::CustomPath(path);
            } else {
                // Empty block or `on` child → default behavior
                settings.miscellaneous.xwayland_satellite = XWaylandSatelliteConfig::Default;
            }
        } else if let Some(path_entry) = xws.entries().first() {
            // Legacy string form: xwayland-satellite "/path/to/binary" (migration)
            if let Some(path) = path_entry.value().as_string() {
                settings.miscellaneous.xwayland_satellite =
                    XWaylandSatelliteConfig::CustomPath(path.to_string());
            }
        }
    }
}

/// Load miscellaneous settings from KDL file
pub fn load_misc(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_misc_from_doc(&doc, settings);

    debug!("Loaded miscellaneous settings from {:?}", path);
}
