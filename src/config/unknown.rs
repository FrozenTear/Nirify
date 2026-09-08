//! Preserve-unknown KDL children for blocks Nirify models only in part.
//!
//! Used by `output { }` (spicy `hdr`, `allow-tearing`, …), `window-rule { }`
//! (`block-minimize`, `allow-tearing`), `debug { }` (`vulkan-renderer`, …),
//! and whitelisted top-level nodes (`minimized-windows`). Modeled siblings
//! are written by the usual generators; everything else is stored as a pretty
//! fragment and re-emitted on save.

use kdl::{KdlDocument, KdlNode};

/// An unmodeled child node, stored as pretty KDL without the parent indent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKdlChild {
    /// Node name (`hdr`, `block-minimize`, …) used to avoid duplicate absorb.
    pub name: String,
    /// Pretty KDL for this node and its descendants.
    pub kdl: String,
}

/// Collect children whose names are not in the modeled set.
pub fn collect_unknown_children(
    doc: &KdlDocument,
    is_modeled: impl Fn(&str) -> bool,
) -> Vec<UnknownKdlChild> {
    doc.nodes()
        .iter()
        .filter(|node| !is_modeled(node.name().value()))
        .map(|node| UnknownKdlChild {
            name: node.name().value().to_string(),
            kdl: format_unknown_kdl_node(node),
        })
        .collect()
}

/// Pretty-print a node at indent 0 (writer adds the parent indent).
pub fn format_unknown_kdl_node(node: &KdlNode) -> String {
    format_kdl_node_pretty(node, 0)
        .trim_end_matches('\n')
        .to_string()
}

fn format_kdl_node_pretty(node: &KdlNode, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let mut out = String::new();
    out.push_str(&pad);
    if let Some(ty) = node.ty() {
        out.push('(');
        out.push_str(ty.value());
        out.push(')');
    }
    out.push_str(node.name().value());
    for entry in node.entries() {
        // KdlEntry Display often includes leading whitespace from the source.
        let rendered = entry.to_string();
        let rendered = rendered.trim();
        if rendered.is_empty() {
            continue;
        }
        out.push(' ');
        out.push_str(rendered);
    }
    if let Some(children) = node.children() {
        if !children.nodes().is_empty() {
            out.push_str(" {\n");
            for child in children.nodes() {
                out.push_str(&format_kdl_node_pretty(child, indent + 1));
            }
            out.push_str(&pad);
            out.push_str("}\n");
            return out;
        }
    }
    out.push('\n');
    out
}

/// Re-emit preserved children at the current (4-space) block indent.
pub fn emit_unknown_children(content: &mut String, children: &[UnknownKdlChild]) {
    for child in children {
        if child.kdl.trim().is_empty() {
            continue;
        }
        for line in child.kdl.lines() {
            if line.is_empty() {
                content.push('\n');
            } else {
                content.push_str("    ");
                content.push_str(line);
                content.push('\n');
            }
        }
    }
}

/// Top-level niri nodes Nirify does not model but must not strip.
pub const PRESERVED_TOP_LEVEL_NAMES: &[&str] = &["minimized-windows"];

/// Collect top-level nodes whose names are in [`PRESERVED_TOP_LEVEL_NAMES`].
pub fn collect_preserved_top_level(doc: &KdlDocument) -> Vec<UnknownKdlChild> {
    doc.nodes()
        .iter()
        .filter(|node| PRESERVED_TOP_LEVEL_NAMES.contains(&node.name().value()))
        .map(|node| UnknownKdlChild {
            name: node.name().value().to_string(),
            kdl: format_unknown_kdl_node(node),
        })
        .collect()
}

/// Append preserved top-level nodes (no extra indent) after existing content.
pub fn emit_top_level_nodes(content: &mut String, nodes: &[UnknownKdlChild]) {
    for node in nodes {
        if node.kdl.trim().is_empty() {
            continue;
        }
        if !content.ends_with('\n') {
            content.push('\n');
        }
        if !content.ends_with("\n\n") {
            content.push('\n');
        }
        content.push_str(&node.kdl);
        if !node.kdl.ends_with('\n') {
            content.push('\n');
        }
    }
}

/// Adopt fragments whose node name is not already present. Returns true if
/// anything was added.
pub fn adopt_unknown_children(
    dest: &mut Vec<UnknownKdlChild>,
    incoming: &[UnknownKdlChild],
) -> bool {
    let mut added = false;
    for child in incoming {
        if child.name.is_empty() {
            continue;
        }
        if !dest.iter().any(|c| c.name == child.name) {
            dest.push(child.clone());
            added = true;
        }
    }
    added
}
