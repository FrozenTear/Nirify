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
/// Looks for settings inside a `gestures { }` block.
pub fn parse_gestures_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // All gesture settings are inside the gestures block
    if let Some(gestures) = doc.get("gestures") {
        if let Some(children) = gestures.children() {
            parse_gestures_from_children(children, settings);
        }
    }
}

fn parse_gestures_from_children(children: &KdlDocument, settings: &mut Settings) {
    // Hot corners
    if let Some(hc) = children.get("hot-corners") {
        let hc_children = hc.children();
        let off = hc_children.map(|c| has_flag(c, &["off"])).unwrap_or(false);
        settings.gestures.hot_corners.enabled = !off;

        if let Some(hc_children) = hc_children {
            let tl = has_flag(hc_children, &["top-left"]);
            let tr = has_flag(hc_children, &["top-right"]);
            let bl = has_flag(hc_children, &["bottom-left"]);
            let br = has_flag(hc_children, &["bottom-right"]);
            // niri: if no corner is explicitly set, top-left is active by default.
            if !(off || tl || tr || bl || br) {
                settings.gestures.hot_corners.top_left = true;
                settings.gestures.hot_corners.top_right = false;
                settings.gestures.hot_corners.bottom_left = false;
                settings.gestures.hot_corners.bottom_right = false;
            } else {
                settings.gestures.hot_corners.top_left = tl;
                settings.gestures.hot_corners.top_right = tr;
                settings.gestures.hot_corners.bottom_left = bl;
                settings.gestures.hot_corners.bottom_right = br;
            }
        }
    }

    // DND edge view scroll (inside gestures block as dnd-edge-view-scroll)
    if let Some(evs) = children.get("dnd-edge-view-scroll") {
        if let Some(evs_children) = evs.children() {
            parse_dnd_edge(
                evs_children,
                "trigger-width",
                &mut settings.gestures.dnd_edge_view_scroll,
            );
        }
    }

    // DND edge workspace switch (inside gestures block as dnd-edge-workspace-switch)
    if let Some(ews) = children.get("dnd-edge-workspace-switch") {
        if let Some(ews_children) = ews.children() {
            parse_dnd_edge(
                ews_children,
                "trigger-height",
                &mut settings.gestures.dnd_edge_workspace_switch,
            );
        }
    }
}

/// Parse a single dnd-edge block (view-scroll or workspace-switch).
///
/// niri has no `off` for these blocks; Nirify persists "disabled" as a
/// `trigger-*` of 0. We also still accept a legacy `off` flag that older
/// Nirify versions wrote.
fn parse_dnd_edge(
    children: &KdlDocument,
    trigger_key: &str,
    dnd: &mut crate::config::models::DndEdgeSettings,
) {
    // Read all children first so that a legacy `off` flag does not cause us to
    // drop a custom delay-ms/max-speed that was written alongside it.
    let trigger = get_i64(children, &[trigger_key]);
    if trigger == Some(0) {
        // Zero trigger = functional disable. Leave trigger_size at the model
        // default so re-enabling is sane.
        dnd.enabled = false;
    } else {
        dnd.enabled = true;
        if let Some(v) = trigger {
            dnd.trigger_size = v as i32;
        }
    }

    if let Some(v) = get_i64(children, &["delay-ms"]) {
        dnd.delay_ms = v as i32;
    }
    if let Some(v) = get_i64(children, &["max-speed"]) {
        dnd.max_speed = v as i32;
    }

    // Legacy disable form (older Nirify wrote `off`). Applied last so that any
    // custom delay-ms/max-speed above are preserved through the migration.
    if has_flag(children, &["off"]) {
        dnd.enabled = false;
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
