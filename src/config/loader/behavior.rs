//! Behavior settings loader
//!
//! Loads behavior and interaction settings from KDL configuration.

use super::super::parser::{get_string, has_flag};
use super::helpers::read_kdl_file;
use crate::config::models::Settings;
use crate::types::{ModKey, WarpMouseMode};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Load behavior settings from KDL file
pub fn load_behavior(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_behavior_from_doc(&doc, settings);

    debug!("Loaded behavior settings from {:?}", path);
}

/// Parse behavior settings from a KDL document
///
/// This is the shared parsing logic used by both `load_behavior()` (for managed files)
/// and `import_behavior_from_doc()` (for importing from user's config).
pub fn parse_behavior_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // Parse settings from input block (where our generator puts them)
    if let Some(input) = doc.get("input") {
        if let Some(input_children) = input.children() {
            parse_input_behavior_settings(input_children, settings);
        }
    }

    // Also check top-level for backwards compatibility and user's main config
    parse_input_behavior_settings(doc, settings);

    // Note: prefer_no_csd, screenshot_path, and hotkey_overlay_skip_at_startup
    // are loaded in load_misc() but we also check here for backwards compatibility
    // with behavior.kdl files that may contain these settings
    if let Some(ho) = doc.get("hotkey-overlay") {
        if let Some(ho_children) = ho.children() {
            if has_flag(ho_children, &["skip-at-startup"]) {
                settings.miscellaneous.hotkey_overlay_skip_at_startup = true;
            }
        }
    }
    if has_flag(doc, &["prefer-no-csd"]) {
        settings.miscellaneous.prefer_no_csd = true;
    }
    if let Some(sp) = get_string(doc, &["screenshot-path"]) {
        settings.miscellaneous.screenshot_path = sp;
    }
}

/// Parse input-level behavior settings (works on both input block children and top-level doc)
fn parse_input_behavior_settings(doc: &KdlDocument, settings: &mut Settings) {
    // mod-key
    if let Some(mod_key_str) = get_string(doc, &["mod-key"]) {
        if let Some(mod_key) = ModKey::from_kdl(&mod_key_str) {
            settings.behavior.mod_key = mod_key;
        }
    }

    // mod-key-nested
    if let Some(mod_key_nested_str) = get_string(doc, &["mod-key-nested"]) {
        if let Some(mod_key) = ModKey::from_kdl(&mod_key_nested_str) {
            settings.behavior.mod_key_nested = Some(mod_key);
        }
    }

    // disable-power-key-handling
    if has_flag(doc, &["disable-power-key-handling"]) {
        settings.behavior.disable_power_key_handling = true;
    }

    // Focus follows mouse
    if let Some(ffm) = doc.get("focus-follows-mouse") {
        settings.behavior.focus_follows_mouse = true;
        // Check for max-scroll-amount
        for entry in ffm.entries() {
            if let Some(name) = entry.name() {
                if name.value() == "max-scroll-amount" {
                    if let Some(val) = entry.value().as_string() {
                        // Parse "50%" format
                        let val = val.trim_end_matches('%');
                        if let Ok(v) = val.parse::<f32>() {
                            settings.behavior.focus_follows_mouse_max_scroll_amount = Some(v);
                        }
                    }
                }
            }
        }
    }

    // Warp mouse to focus
    if let Some(wmtf) = doc.get("warp-mouse-to-focus") {
        for entry in wmtf.entries() {
            if let Some(name) = entry.name() {
                if name.value() == "mode" {
                    if let Some(val) = entry.value().as_string() {
                        settings.behavior.warp_mouse_to_focus = match val {
                            "center-xy" => WarpMouseMode::CenterXY,
                            "center-xy-always" => WarpMouseMode::CenterXYAlways,
                            _ => WarpMouseMode::Off,
                        };
                    }
                }
            }
        }
    }

    // Workspace auto back and forth
    if has_flag(doc, &["workspace-auto-back-and-forth"]) {
        settings.behavior.workspace_auto_back_and_forth = true;
    }
}
