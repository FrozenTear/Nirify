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

/// An `include` directive whose target file overlaps with Nirify-managed sections.
///
/// Niri merges top-level nodes across includes; for repeated single-instance
/// blocks (e.g. `layout`), the later include wins. So a user-managed include
/// that defines `layout` will silently override Nirify's layout settings unless
/// Nirify's own include comes after it in `config.kdl`.
#[derive(Debug, Clone)]
pub struct ConflictingInclude {
    /// Path as it appears in the include directive (e.g. `./cfg/layout.kdl`).
    pub include_path: String,
    /// Resolved absolute path of the included file.
    pub resolved_path: PathBuf,
    /// Names of top-level managed nodes found in the included file.
    pub conflicting_nodes: Vec<String>,
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
    /// Number of Nirify include lines found (for duplicate detection)
    pub nirify_include_count: usize,
    /// Index of the last Nirify include line (for "include is last" check)
    pub nirify_include_idx: Option<usize>,
    /// Count of managed nodes that will be removed
    pub managed_count: usize,
    /// Count of unmanaged nodes that will be preserved
    pub unmanaged_count: usize,
    /// Original file content
    pub original_content: String,
    /// Other includes whose contents declare top-level managed nodes.
    /// Even after we put Nirify's include last, surfacing these lets the UI
    /// offer a one-shot import flow.
    pub conflicting_includes: Vec<ConflictingInclude>,
}

impl ConfigAnalysis {
    /// Whether smart_replace needs to rewrite the file.
    ///
    /// Returns true if any of the following holds:
    /// - There are top-level managed nodes that should be removed.
    /// - The Nirify include is missing.
    /// - There are duplicate Nirify include lines.
    /// - The Nirify include is not the last top-level node (so later content
    ///   could override its values).
    pub fn needs_rewrite(&self) -> bool {
        if self.managed_count > 0 {
            return true;
        }
        if !self.has_nirify_include {
            return true;
        }
        if self.nirify_include_count > 1 {
            return true;
        }
        let nodes_len = self.document.nodes().len();
        if let Some(idx) = self.nirify_include_idx {
            if idx != nodes_len.saturating_sub(1) {
                return true;
            }
        }
        false
    }
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
    /// Other includes whose targets contain managed nodes.
    /// Nirify's include is now placed last so it wins ties, but these
    /// overlaps are surfaced so a future import flow can offer to absorb them.
    pub conflicting_includes: Vec<ConflictingInclude>,
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

/// Resolve the path of an `include` directive against `config.kdl`'s parent.
///
/// Returns `None` for tilde-prefixed paths because niri itself does not expand
/// tildes in includes (see paths.rs::migrate_include_line); we don't want to
/// scan a file niri can't actually load.
fn resolve_include_path(include_value: &str, config_parent: &Path) -> Option<PathBuf> {
    if include_value.starts_with("~/") || include_value == "~" {
        return None;
    }
    let p = Path::new(include_value);
    if p.is_absolute() {
        Some(p.to_path_buf())
    } else {
        Some(config_parent.join(p))
    }
}

/// Read an included file and collect any top-level managed nodes it declares.
///
/// Returns `None` if the file can't be read/parsed or contains no managed
/// nodes — only files that actually conflict are surfaced.
fn scan_include_for_conflicts(
    include_value: &str,
    config_parent: &Path,
) -> Option<ConflictingInclude> {
    let resolved = resolve_include_path(include_value, config_parent)?;
    let content = match fs::read_to_string(&resolved) {
        Ok(c) => c,
        Err(e) => {
            debug!(
                "Could not read included file {:?} for conflict scan: {}",
                resolved, e
            );
            return None;
        }
    };
    let doc: KdlDocument = match content.parse() {
        Ok(d) => d,
        Err(e) => {
            debug!(
                "Could not parse included file {:?} for conflict scan: {}",
                resolved, e
            );
            return None;
        }
    };

    let mut seen = std::collections::HashSet::new();
    let conflicting_nodes: Vec<String> = doc
        .nodes()
        .iter()
        .filter_map(|n| {
            let name = n.name().value();
            if is_managed_node(name) && seen.insert(name.to_string()) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

    if conflicting_nodes.is_empty() {
        None
    } else {
        Some(ConflictingInclude {
            include_path: include_value.to_string(),
            resolved_path: resolved,
            conflicting_nodes,
        })
    }
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

    let config_parent = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut node_classifications = Vec::new();
    let mut has_nirify_include = false;
    let mut nirify_include_count = 0;
    let mut nirify_include_idx: Option<usize> = None;
    let mut managed_count = 0;
    let mut unmanaged_count = 0;
    let mut conflicting_includes: Vec<ConflictingInclude> = Vec::new();

    for (idx, node) in document.nodes().iter().enumerate() {
        let name = node.name().value();

        let classification = if is_nirify_include(node) {
            has_nirify_include = true;
            nirify_include_count += 1;
            nirify_include_idx = Some(idx);
            debug!("Found existing Nirify include at index {}", idx);
            NodeClassification::NirifyInclude
        } else if name == "include" {
            unmanaged_count += 1;
            debug!("Found other include at index {}: preserving", idx);
            if let Some(value) = node.entries().first().and_then(|e| e.value().as_string()) {
                if let Some(conflict) = scan_include_for_conflicts(value, &config_parent) {
                    debug!(
                        "Include {:?} declares managed nodes: {:?}",
                        value, conflict.conflicting_nodes
                    );
                    conflicting_includes.push(conflict);
                }
            }
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
        "Config analysis: {} managed, {} unmanaged, Nirify include: {}, conflicting includes: {}",
        managed_count,
        unmanaged_count,
        has_nirify_include,
        conflicting_includes.len()
    );

    // Warn about duplicate includes (they will all be removed and replaced with one)
    if nirify_include_count > 1 {
        warn!(
            "Found {} duplicate Nirify include lines - will consolidate to single include",
            nirify_include_count
        );
    }

    for conflict in &conflicting_includes {
        warn!(
            "Include {:?} defines managed nodes {:?} — Nirify settings for these sections \
             only apply because the Nirify include is placed last",
            conflict.include_path, conflict.conflicting_nodes
        );
    }

    Ok(ConfigAnalysis {
        document,
        node_classifications,
        has_nirify_include,
        nirify_include_count,
        nirify_include_idx,
        managed_count,
        unmanaged_count,
        original_content,
        conflicting_includes,
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

/// Generate a new config.kdl preserving unmanaged content.
///
/// The Nirify include is placed **last** so its top-level nodes win ties
/// against any earlier `include` directives that declare the same sections
/// (niri uses last-write-wins for repeated single-instance blocks like
/// `layout`).
fn generate_replaced_config(analysis: &ConfigAnalysis) -> String {
    let mut content = String::with_capacity(4096);

    content.push_str("// Configuration managed by Nirify\n");
    content.push_str("// Backup of original config saved before modification\n");
    content.push_str("//\n");
    content.push_str("// Managed settings (appearance, input, behavior, etc.) are in:\n");
    content.push_str("//   nirify/\n");
    content.push_str("//\n");
    content.push_str("// Your custom content is preserved above; the Nirify include is\n");
    content.push_str("// placed at the end so its values win over any earlier includes\n");
    content.push_str("// that define the same sections.\n");

    let unmanaged = extract_unmanaged_nodes(analysis);
    if !unmanaged.is_empty() {
        content.push_str("\n// === Your custom configuration (preserved) ===\n");
        content.push_str(&unmanaged);
    }

    content.push_str("\n// === Nirify managed configuration ===\n");
    content.push_str("include \"nirify/main.kdl\"\n");

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
            conflicting_includes: Vec::new(),
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
                conflicting_includes: Vec::new(),
            });
        }
    };

    // Append conflict warnings to the result so callers can surface them even
    // if no rewrite is needed (e.g. a future "import these settings?" prompt).
    for conflict in &analysis.conflicting_includes {
        warnings.push(format!(
            "Include {:?} defines managed sections {:?}; Nirify's include is placed last so it wins",
            conflict.include_path, conflict.conflicting_nodes
        ));
    }

    // Skip the rewrite when the config is already in the desired shape:
    // include exists, no top-level managed nodes, no duplicates, and the
    // include is the last node.
    if !analysis.needs_rewrite() {
        info!("Config already set up with Nirify include last, no changes needed");
        warnings.push("Config already set up, no changes needed".to_string());
        return Ok(SmartReplaceResult {
            backup_path: PathBuf::new(),
            replaced_count: 0,
            preserved_count: analysis.unmanaged_count,
            include_added: false,
            warnings,
            conflicting_includes: analysis.conflicting_includes.clone(),
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
        conflicting_includes: analysis.conflicting_includes,
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

    /// Helper: write a config and run smart_replace_config against it.
    fn run_smart_replace(
        config_contents: &str,
        extra_files: &[(&str, &str)],
    ) -> (tempfile::TempDir, PathBuf, SmartReplaceResult) {
        let temp_dir = tempfile::tempdir().unwrap();
        let niri_dir = temp_dir.path().join("niri");
        std::fs::create_dir_all(&niri_dir).unwrap();
        let config_path = niri_dir.join("config.kdl");
        std::fs::write(&config_path, config_contents).unwrap();

        for (rel_path, body) in extra_files {
            let p = niri_dir.join(rel_path);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(p, body).unwrap();
        }

        let backup_dir = temp_dir.path().join(".nirify-backups");
        let result = smart_replace_config(&config_path, &backup_dir).unwrap();
        (temp_dir, config_path, result)
    }

    #[test]
    fn test_generated_config_places_nirify_include_last() {
        // User has unmanaged content (other includes). After rewrite, the
        // Nirify include must appear after them so its values win on overlap.
        let original = r#"
            include "./cfg/layout.kdl"
            include "./cfg/animation.kdl"
            custom-marker "hi"
        "#;
        let (_t, config_path, _result) = run_smart_replace(original, &[]);
        let new_content = std::fs::read_to_string(&config_path).unwrap();

        let nirify_pos = new_content
            .find("include \"nirify/main.kdl\"")
            .expect("nirify include present");
        let cfg_layout_pos = new_content
            .find("./cfg/layout.kdl")
            .expect("user include preserved");
        let cfg_anim_pos = new_content
            .find("./cfg/animation.kdl")
            .expect("user include preserved");
        let custom_pos = new_content
            .find("custom-marker")
            .expect("custom node preserved");

        assert!(
            nirify_pos > cfg_layout_pos && nirify_pos > cfg_anim_pos && nirify_pos > custom_pos,
            "Nirify include should be placed after preserved content.\n\nGot:\n{}",
            new_content
        );

        // Sanity check the generated content parses as KDL.
        new_content
            .parse::<KdlDocument>()
            .expect("Generated config should parse");
    }

    #[test]
    fn test_rewrite_when_nirify_include_not_last() {
        // Simulates the user's reported case: Nirify include first, user
        // includes after it (which silently override managed sections).
        let original = "\
include \"nirify/main.kdl\"

include \"./cfg/layout.kdl\"
include \"./cfg/animation.kdl\"
";
        let (_t, config_path, result) = run_smart_replace(original, &[]);

        // It should have rewritten (backup created).
        assert!(
            !result.backup_path.as_os_str().is_empty(),
            "Expected a backup to be written when reordering"
        );

        let new_content = std::fs::read_to_string(&config_path).unwrap();
        let nirify_pos = new_content
            .find("include \"nirify/main.kdl\"")
            .expect("nirify include present");
        let cfg_layout_pos = new_content
            .find("./cfg/layout.kdl")
            .expect("user include preserved");
        assert!(
            nirify_pos > cfg_layout_pos,
            "Nirify include should be reordered after user includes.\n\nGot:\n{}",
            new_content
        );

        // Re-running on the now-correctly-ordered config should be a no-op.
        let backup_dir = config_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".nirify-backups");
        let second = smart_replace_config(&config_path, &backup_dir).unwrap();
        assert!(
            second.backup_path.as_os_str().is_empty(),
            "Second run should be a no-op (no new backup)"
        );
    }

    #[test]
    fn test_no_rewrite_when_already_correctly_ordered() {
        // Nirify include is the only and last node — nothing to do.
        let original = r#"
custom-marker "hi"

include "nirify/main.kdl"
"#;
        let (_t, _config_path, result) = run_smart_replace(original, &[]);
        assert!(
            result.backup_path.as_os_str().is_empty(),
            "Should not rewrite when already correctly ordered"
        );
        assert_eq!(result.replaced_count, 0);
    }

    #[test]
    fn test_detects_conflicting_other_include() {
        // User has `include "./cfg/layout.kdl"` and that file declares a
        // top-level managed `layout` node — we should surface it as a conflict.
        let original = r#"
include "./cfg/layout.kdl"
"#;
        let layout_body = r#"layout {
            gaps 16
        }
        "#;
        let (_t, _config_path, result) =
            run_smart_replace(original, &[("cfg/layout.kdl", layout_body)]);

        assert_eq!(
            result.conflicting_includes.len(),
            1,
            "Expected exactly one conflicting include, got {:?}",
            result.conflicting_includes
        );
        let conflict = &result.conflicting_includes[0];
        assert_eq!(conflict.include_path, "./cfg/layout.kdl");
        assert!(
            conflict.conflicting_nodes.iter().any(|n| n == "layout"),
            "Expected `layout` in conflicting_nodes, got {:?}",
            conflict.conflicting_nodes
        );
    }

    #[test]
    fn test_no_conflict_for_unmanaged_other_include() {
        // Other include exists but only defines a custom (non-managed) node.
        let original = r#"
include "./cfg/custom.kdl"

include "nirify/main.kdl"
"#;
        let custom_body = r#"my-custom-node "value"
"#;
        let (_t, _config_path, result) =
            run_smart_replace(original, &[("cfg/custom.kdl", custom_body)]);
        assert!(
            result.conflicting_includes.is_empty(),
            "Unmanaged include should not be flagged: {:?}",
            result.conflicting_includes
        );
    }

    #[test]
    fn test_resolve_include_path_skips_tilde() {
        let parent = Path::new("/some/dir");
        assert!(resolve_include_path("~/foo.kdl", parent).is_none());
        assert_eq!(
            resolve_include_path("./cfg/x.kdl", parent),
            Some(PathBuf::from("/some/dir/./cfg/x.kdl"))
        );
        assert_eq!(
            resolve_include_path("/abs/x.kdl", parent),
            Some(PathBuf::from("/abs/x.kdl"))
        );
    }
}
