//! Workspace settings KDL generation
//!
//! Generates KDL for named workspaces with layout overrides.

use super::builder::KdlBuilder;
use super::display::generate_layout_override_kdl;
use crate::config::models::WorkspacesSettings;

/// Generate workspaces.kdl from named workspaces settings
///
/// Creates KDL configuration for named workspaces (v0.1.6+), including:
/// - Named workspace declarations
/// - open-on-output to pin to specific monitors
/// - layout overrides per workspace (v25.11+)
pub fn generate_workspaces_kdl(settings: &WorkspacesSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Named workspaces - managed by Nirify");
    kdl.comment("Workspaces declared here will always exist.");
    kdl.newline();

    if settings.workspaces.is_empty() {
        kdl.comment("No named workspaces configured yet.");
        kdl.comment("Add workspaces through the UI or manually here.");
        kdl.comment("Example:");
        kdl.comment("workspace \"browser\"");
        kdl.comment("workspace \"coding\" {");
        kdl.comment("    open-on-output \"DP-1\"");
        kdl.comment("}");
    } else {
        for workspace in &settings.workspaces {
            let has_children =
                workspace.open_on_output.is_some() || workspace.layout_override.is_some();

            if has_children {
                kdl.node_with_arg("workspace", &workspace.name, |b| {
                    // open-on-output
                    if let Some(ref output) = workspace.open_on_output {
                        b.field_string("open-on-output", output);
                    }

                    // layout override block - uses existing helper for now
                    if let Some(ref layout) = workspace.layout_override {
                        b.raw(&generate_layout_override_kdl(layout, ""));
                    }
                });
            } else {
                // Simple workspace with just a name
                kdl.field_string("workspace", &workspace.name);
            }
        }
    }

    kdl.build()
}
