//! Smart config replacement
//!
//! This module handles intelligent replacement of the user's niri config.kdl file.
//! Instead of just appending an include line (which can cause conflicts), it:
//! 1. Parses the config and classifies each top-level node
//! 2. Generates a clean config that preserves unmanaged content
//! 3. Replaces managed sections with a single include to nirify
//!
//! **Note on preservation:** Unmanaged nodes retain their semantic structure (names,
//! values, children) but formatting (whitespace, indentation, inline comments) may
//! be normalized by the KDL library. The original config is always backed up first.

use anyhow::{Context, Result};
use chrono::Local;
use kdl::{KdlDocument, KdlNode};
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use super::storage::atomic_write;

/// Classification of a top-level KDL node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeClassification {
    /// Managed by Nirify - will be replaced with include
    Managed,
    /// User's include directive for Nirify (skip it)
    NirifyInclude,
    /// Other include directives (preserve)
    OtherInclude,
    /// Not managed by Nirify - preserve structure (formatting may be normalized)
    Unmanaged,
}

/// Result of analyzing a config.kdl file
#[derive(Debug)]
pub struct ConfigAnalysis {
    /// The original KDL document
    pub document: KdlDocument,
    /// Classification for each top-level node (by index)
    pub node_classifications: Vec<(usize, NodeClassification)>,
    /// Whether our include line already exists
    pub has_nirify_include: bool,
    /// Count of managed nodes that will be removed
    pub managed_count: usize,
    /// Count of unmanaged nodes that will be preserved
    pub unmanaged_count: usize,
    /// Original file content
    pub original_content: String,
}

/// Result of the smart replace operation
#[derive(Debug)]
pub struct SmartReplaceResult {
    /// Path to the backup file created (empty if no backup needed)
    pub backup_path: PathBuf,
    /// Number of managed sections replaced
    pub replaced_count: usize,
    /// Number of unmanaged sections preserved
    pub preserved_count: usize,
    /// Whether the include line was added
    pub include_added: bool,
    /// Any warnings generated during the process
    pub warnings: Vec<String>,
}

/// Top-level node names managed by Nirify
///
/// These nodes will be removed from the user's config and replaced
/// with an include to nirify/main.kdl
const MANAGED_NODES: &[&str] = &[
    // Core layout (contains gaps, focus-ring, border, struts, etc.)
    "layout",
    // Input devices
    "input",
    // Display/Visual
    "animations",
    "cursor",
    "overview",
    // Per-instance nodes (multiple allowed)
    "output",
    "workspace",
    "window-rule",
    "layer-rule",
    // Startup/Environment
    "spawn-at-startup",
    "environment",
    // Debug/Advanced
    "debug",
    "switch-events",
    "hotkey-overlay",
    // Top-level behavior flags
    "screenshot-path",
    "prefer-no-csd",
    "focus-follows-mouse",
    "warp-mouse-to-focus",
    "workspace-auto-back-and-forth",
    // Keybindings
    "binds",
];

/// Check if a node name is managed by Nirify
fn is_managed_node(name: &str) -> bool {
    MANAGED_NODES.contains(&name)
}

/// Check if this is a Nirify include line
fn is_nirify_include(node: &KdlNode) -> bool {
    if node.name().value() != "include" {
        return false;
    }
    node.entries()
        .first()
        .and_then(|e| e.value().as_string())
        .map(|s| s.contains("nirify"))
        .unwrap_or(false)
}

/// Analyze a config.kdl file and classify its nodes
///
/// Returns a ConfigAnalysis with node classifications for smart replacement.
pub fn analyze_config(config_path: &Path) -> Result<ConfigAnalysis> {
    let original_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {:?}", config_path))?;

    let document: KdlDocument = original_content
        .parse()
        .with_context(|| format!("Failed to parse {:?} as KDL", config_path))?;

    let mut node_classifications = Vec::new();
    let mut has_nirify_include = false;
    let mut nirify_include_count = 0;
    let mut managed_count = 0;
    let mut unmanaged_count = 0;

    for (idx, node) in document.nodes().iter().enumerate() {
        let name = node.name().value();

        let classification = if is_nirify_include(node) {
            has_nirify_include = true;
            nirify_include_count += 1;
            debug!("Found existing Nirify include at index {}", idx);
            NodeClassification::NirifyInclude
        } else if name == "include" {
            unmanaged_count += 1;
            debug!("Found other include at index {}: preserving", idx);
            NodeClassification::OtherInclude
        } else if is_managed_node(name) {
            managed_count += 1;
            debug!(
                "Found managed node '{}' at index {}: will replace",
                name, idx
            );
            NodeClassification::Managed
        } else {
            unmanaged_count += 1;
            debug!(
                "Found unmanaged node '{}' at index {}: will preserve",
                name, idx
            );
            NodeClassification::Unmanaged
        };

        node_classifications.push((idx, classification));
    }

    info!(
        "Config analysis: {} managed, {} unmanaged, Nirify include: {}",
        managed_count, unmanaged_count, has_nirify_include
    );

    // Warn about duplicate includes (they will all be removed and replaced with one)
    if nirify_include_count > 1 {
        warn!(
            "Found {} duplicate Nirify include lines - will consolidate to single include",
            nirify_include_count
        );
    }

    Ok(ConfigAnalysis {
        document,
        node_classifications,
        has_nirify_include,
        managed_count,
        unmanaged_count,
        original_content,
    })
}

/// Generate a minimal config with just the include line
fn generate_minimal_config() -> String {
    r#"// Configuration managed by Nirify
//
// All settings are in the nirify/ subdirectory.
// Edit them using the Nirify app or the files directly.

include "nirify/main.kdl"
"#
    .to_string()
}

/// Generate a new config.kdl preserving unmanaged content
fn generate_replaced_config(analysis: &ConfigAnalysis) -> String {
    let mut content = String::with_capacity(4096);

    // Header comment
    content.push_str("// Configuration managed by Nirify\n");
    content.push_str("// Backup of original config saved before modification\n");
    content.push_str("//\n");
    content.push_str("// Managed settings (appearance, input, behavior, etc.) are in:\n");
    content.push_str("//   nirify/\n");
    content.push_str("//\n");
    content.push_str("// Your custom settings below are preserved.\n\n");

    // Add our include line (relative path from config.kdl location)
    content.push_str("// === Nirify managed configuration ===\n");
    content.push_str("include \"nirify/main.kdl\"\n");

    // Extract and add unmanaged content
    let unmanaged = extract_unmanaged_nodes(analysis);
    if !unmanaged.is_empty() {
        content.push_str("\n// === Your custom configuration (preserved) ===\n");
        content.push_str(&unmanaged);
    }

    content
}

/// Extract unmanaged nodes from the analysis
fn extract_unmanaged_nodes(analysis: &ConfigAnalysis) -> String {
    let mut result = String::new();

    for (idx, classification) in &analysis.node_classifications {
        match classification {
            NodeClassification::Unmanaged | NodeClassification::OtherInclude => {
                // Bounds check to prevent panic on out-of-range index
                let nodes = analysis.document.nodes();
                if *idx >= nodes.len() {
                    warn!(
                        "Node index {} out of bounds (len={}), skipping",
                        idx,
                        nodes.len()
                    );
                    continue;
                }
                let node = &nodes[*idx];
                // Use KDL's Display formatting - preserves structure, may normalize whitespace
                result.push_str(&format!("{}\n", node));
            }
            NodeClassification::Managed | NodeClassification::NirifyInclude => {
                // Skip managed nodes and existing Nirify includes
            }
        }
    }

    result
}

/// Perform smart replacement of config.kdl
///
/// This function:
/// 1. Analyzes the existing config (if any)
/// 2. Creates a timestamped backup
/// 3. Generates a new config preserving unmanaged content
/// 4. Writes the new config
///
/// Returns Ok(SmartReplaceResult) on success, describing what was done.
pub fn smart_replace_config(config_path: &Path, backup_dir: &Path) -> Result<SmartReplaceResult> {
    let mut warnings = Vec::new();

    // Handle non-existent config
    if !config_path.exists() {
        info!("Config file doesn't exist, creating minimal config");
        let content = generate_minimal_config();
        atomic_write(config_path, &content)
            .with_context(|| format!("Failed to write {:?}", config_path))?;

        return Ok(SmartReplaceResult {
            backup_path: PathBuf::new(),
            replaced_count: 0,
            preserved_count: 0,
            include_added: true,
            warnings,
        });
    }

    // Try to analyze existing config
    let analysis = match analyze_config(config_path) {
        Ok(a) => a,
        Err(e) => {
            // Config exists but can't be parsed - create backup and replace with minimal
            warn!("Failed to parse config.kdl: {}", e);
            warnings.push(format!("Could not parse config.kdl: {}", e));

            // Create backup of unparseable config using read + atomic_write to avoid TOCTOU race
            let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S");
            let backup_name = format!("config.kdl.backup-{}", timestamp);
            let backup_path = backup_dir.join(&backup_name);

            // Read content first, then write atomically
            let backup_content = fs::read_to_string(config_path)
                .with_context(|| format!("Failed to read {:?} for backup", config_path))?;

            fs::create_dir_all(backup_dir)?;
            atomic_write(&backup_path, &backup_content)
                .with_context(|| format!("Failed to backup to {:?}", backup_path))?;

            // Write minimal config (atomic to prevent corruption)
            let content = generate_minimal_config();
            atomic_write(config_path, &content)?;

            return Ok(SmartReplaceResult {
                backup_path,
                replaced_count: 0,
                preserved_count: 0,
                include_added: true,
                warnings,
            });
        }
    };

    // Check if already set up with no managed nodes to clean
    if analysis.has_nirify_include && analysis.managed_count == 0 {
        info!("Config already set up with Nirify include, no changes needed");
        warnings.push("Config already set up, no changes needed".to_string());
        return Ok(SmartReplaceResult {
            backup_path: PathBuf::new(),
            replaced_count: 0,
            preserved_count: analysis.unmanaged_count,
            include_added: false,
            warnings,
        });
    }

    // Create timestamped backup using read + atomic_write to avoid TOCTOU race
    let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S");
    let backup_name = format!("config.kdl.backup-{}", timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::create_dir_all(backup_dir)
        .with_context(|| format!("Failed to create backup directory {:?}", backup_dir))?;

    // Read content first (we already have it in analysis.original_content)
    // Use atomic_write for safe backup
    atomic_write(&backup_path, &analysis.original_content)
        .with_context(|| format!("Failed to backup to {:?}", backup_path))?;

    // Verify backup was written correctly before proceeding
    let backup_content = fs::read_to_string(&backup_path)
        .with_context(|| format!("Failed to verify backup at {:?}", backup_path))?;
    if backup_content != analysis.original_content {
        return Err(anyhow::anyhow!(
            "Backup verification failed: content mismatch at {:?}. Aborting to prevent data loss.",
            backup_path
        ));
    }

    info!("Created and verified backup at {:?}", backup_path);

    // Generate new config content
    let new_content = generate_replaced_config(&analysis);

    // Validate generated KDL parses correctly
    if let Err(e) = new_content.parse::<KdlDocument>() {
        return Err(anyhow::anyhow!(
            "Generated config is invalid KDL: {}. Original preserved in backup at {:?}",
            e,
            backup_path
        ));
    }

    // Write new config (atomic to prevent corruption on crash)
    atomic_write(config_path, &new_content)
        .with_context(|| format!("Failed to write {:?}", config_path))?;

    info!(
        "Smart replace complete: {} managed replaced, {} unmanaged preserved",
        analysis.managed_count, analysis.unmanaged_count
    );

    Ok(SmartReplaceResult {
        backup_path,
        replaced_count: analysis.managed_count,
        preserved_count: analysis.unmanaged_count,
        include_added: !analysis.has_nirify_include,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_managed_node() {
        assert!(is_managed_node("layout"));
        assert!(is_managed_node("input"));
        assert!(is_managed_node("animations"));
        assert!(is_managed_node("output"));
        assert!(is_managed_node("window-rule"));
        assert!(is_managed_node("binds"));

        assert!(!is_managed_node("custom-node"));
        assert!(!is_managed_node("my-setting"));
        assert!(!is_managed_node("include"));
    }

    #[test]
    fn test_generate_minimal_config() {
        let config = generate_minimal_config();
        assert!(config.contains("include"));
        assert!(config.contains("nirify/main.kdl"));

        // Should be valid KDL
        config.parse::<KdlDocument>().expect("Should be valid KDL");
    }

    #[test]
    fn test_node_classification() {
        let config = r#"
            layout { gaps inner=16 }
            custom-node { foo "bar" }
            include "other-config.kdl"
            animations { off }
        "#;

        let doc: KdlDocument = config.parse().unwrap();
        let nodes = doc.nodes();

        assert_eq!(nodes.len(), 4);

        // layout is managed
        assert!(is_managed_node(nodes[0].name().value()));

        // custom-node is not managed
        assert!(!is_managed_node(nodes[1].name().value()));

        // include is special
        assert!(!is_managed_node(nodes[2].name().value()));

        // animations is managed
        assert!(is_managed_node(nodes[3].name().value()));
    }
}
