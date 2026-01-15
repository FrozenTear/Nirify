//! Gesture settings loader
//!
//! Handles hot corners and DND edge scrolling/workspace switching.

use super::helpers::read_kdl_file;
use crate::config::models::Settings;
use crate::config::parser::{get_i64, has_flag};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Parse gestures from a document
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_gestures_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // Hot corners
    if let Some(hc) = doc.get("hot-corners") {
        settings.gestures.hot_corners.enabled = true;
        if let Some(hc_children) = hc.children() {
            settings.gestures.hot_corners.top_left = has_flag(hc_children, &["top-left"]);
            settings.gestures.hot_corners.top_right = has_flag(hc_children, &["top-right"]);
            settings.gestures.hot_corners.bottom_left = has_flag(hc_children, &["bottom-left"]);
            settings.gestures.hot_corners.bottom_right = has_flag(hc_children, &["bottom-right"]);
        }
    }

    // DND settings
    if let Some(dnd) = doc.get("dnd") {
        if let Some(dnd_children) = dnd.children() {
            // Edge view scroll
            if let Some(evs) = dnd_children.get("edge-view-scroll") {
                if let Some(evs_children) = evs.children() {
                    if has_flag(evs_children, &["off"]) {
                        settings.gestures.dnd_edge_view_scroll.enabled = false;
                    } else {
                        settings.gestures.dnd_edge_view_scroll.enabled = true;
                        if let Some(v) = get_i64(evs_children, &["trigger-width"]) {
                            settings.gestures.dnd_edge_view_scroll.trigger_size = v as i32;
                        }
                        if let Some(v) = get_i64(evs_children, &["delay-ms"]) {
                            settings.gestures.dnd_edge_view_scroll.delay_ms = v as i32;
                        }
                        if let Some(v) = get_i64(evs_children, &["max-speed"]) {
                            settings.gestures.dnd_edge_view_scroll.max_speed = v as i32;
                        }
                    }
                }
            }

            // Edge workspace switch
            if let Some(ews) = dnd_children.get("edge-workspace-switch") {
                if let Some(ews_children) = ews.children() {
                    if has_flag(ews_children, &["off"]) {
                        settings.gestures.dnd_edge_workspace_switch.enabled = false;
                    } else {
                        settings.gestures.dnd_edge_workspace_switch.enabled = true;
                        if let Some(v) = get_i64(ews_children, &["trigger-height"]) {
                            settings.gestures.dnd_edge_workspace_switch.trigger_size = v as i32;
                        }
                        if let Some(v) = get_i64(ews_children, &["delay-ms"]) {
                            settings.gestures.dnd_edge_workspace_switch.delay_ms = v as i32;
                        }
                        if let Some(v) = get_i64(ews_children, &["max-speed"]) {
                            settings.gestures.dnd_edge_workspace_switch.max_speed = v as i32;
                        }
                    }
                }
            }
        }
    }
}

/// Load gesture settings from KDL file
pub fn load_gestures(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    parse_gestures_from_doc(&doc, settings);

    debug!("Loaded gesture settings from {:?}", path);
}
