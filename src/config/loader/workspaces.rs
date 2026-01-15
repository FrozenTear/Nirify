//! Workspace settings loader
//!
//! Handles named workspaces with layout overrides.

use super::display::parse_layout_override;
use super::helpers::read_kdl_file;
use crate::config::models::{NamedWorkspace, Settings};
use crate::config::parser::get_string;
use kdl::KdlDocument;
use log::{debug, warn};
use std::path::Path;

/// Parse workspace node children into a NamedWorkspace
///
/// Shared parsing logic used by both file loader and import.
pub fn parse_workspace_node_children(children: &KdlDocument, workspace: &mut NamedWorkspace) {
    // open-on-output
    if let Some(v) = get_string(children, &["open-on-output"]) {
        workspace.open_on_output = Some(v);
    }

    // layout override block - use the shared parse_layout_override function
    if let Some(layout_node) = children.get("layout") {
        if let Some(layout_children) = layout_node.children() {
            workspace.layout_override = parse_layout_override(layout_children);
        }
    }
}

/// Load named workspaces from KDL file
pub fn load_workspaces(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };

    settings.workspaces.workspaces.clear();
    let mut next_id = 0u32;

    for node in doc.nodes() {
        if node.name().value() == "workspace" {
            // Get workspace name from first argument
            let name = if let Some(entry) = node.entries().first() {
                if let Some(s) = entry.value().as_string() {
                    s.to_string()
                } else {
                    // Argument present but not a string - warn and skip
                    warn!(
                        "Workspace has non-string name argument: {:?}, skipping",
                        entry.value()
                    );
                    continue;
                }
            } else {
                continue; // Skip workspaces without a name
            };

            if name.is_empty() {
                warn!("Workspace has empty name, skipping");
                continue;
            }

            let mut workspace = NamedWorkspace {
                id: next_id,
                name,
                open_on_output: None,
                layout_override: None,
            };
            next_id += 1;

            // Parse children if present
            if let Some(children) = node.children() {
                parse_workspace_node_children(children, &mut workspace);
            }

            settings.workspaces.workspaces.push(workspace);
        }
    }

    settings.workspaces.next_id = next_id;
    debug!(
        "Loaded {} named workspaces from {:?}",
        settings.workspaces.workspaces.len(),
        path
    );
}
