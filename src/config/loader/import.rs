//! Import settings from user's niri config
//!
//! This module handles importing settings from the user's existing niri config.kdl
//! on first run, preserving their existing configuration.
//!
//! This module uses shared parsing functions from the loader modules to avoid
//! code duplication. The import functions primarily delegate to these shared
//! parsers, with some import-specific handling (e.g., global corner radius).

use super::super::models::{LayerRule, NamedWorkspace, OutputConfig, Settings, WindowRule};
use super::super::parser::get_i64;
use super::{
    helpers, load_keybindings, parse_animations_from_children, parse_appearance_from_doc,
    parse_behavior_from_doc, parse_cursor_from_children, parse_debug_from_doc,
    parse_environment_from_doc, parse_gestures_from_doc, parse_keyboard_from_children,
    parse_layer_rule_node_children, parse_layout_extras_from_children, parse_misc_from_doc,
    parse_mouse_from_children, parse_output_node_children, parse_overview_from_children,
    parse_startup_from_doc, parse_switch_events_from_doc, parse_tablet_from_children,
    parse_touch_from_children, parse_touchpad_from_children, parse_trackball_from_children,
    parse_trackpoint_from_children, parse_window_rule_node_children, parse_workspace_node_children,
};
use kdl::KdlDocument;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Maximum depth for include file traversal to prevent circular includes
const MAX_INCLUDE_DEPTH: usize = 10;

/// Result of importing settings from user's niri config
///
/// Provides detailed feedback about what was imported, what used defaults,
/// and any warnings encountered during import.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The imported settings
    pub settings: Settings,
    /// Sections that were successfully imported from the config
    pub imported_sections: Vec<String>,
    /// Sections that used default values (not found in config)
    pub defaulted_sections: Vec<String>,
    /// Warnings encountered during import (e.g., skipped includes, parse errors)
    pub warnings: Vec<String>,
    /// Number of include files processed
    pub includes_processed: usize,
}

impl ImportResult {
    /// Returns true if any settings were imported (not all defaults)
    pub fn has_imports(&self) -> bool {
        !self.imported_sections.is_empty()
    }

    /// Returns a summary string suitable for display
    pub fn summary(&self) -> String {
        if self.imported_sections.is_empty() {
            "No settings imported, using defaults".to_string()
        } else {
            format!(
                "Imported {} sections: {}",
                self.imported_sections.len(),
                self.imported_sections.join(", ")
            )
        }
    }
}

/// Import settings from user's existing niri config.kdl
///
/// Called on first run to preserve user's existing configuration.
/// Parses the user's main config and any included files, extracting
/// all recognized settings.
///
/// # Arguments
/// * `niri_config` - Path to the user's main config.kdl file
///
/// # Returns
/// A Settings struct populated with values from the user's config,
/// with defaults for any missing settings.
pub fn import_from_niri_config(niri_config: &Path) -> Settings {
    import_from_niri_config_with_result(niri_config).settings
}

/// Import settings with detailed result information
///
/// Like `import_from_niri_config`, but returns an `ImportResult` with
/// information about what was imported, what used defaults, and any warnings.
pub fn import_from_niri_config_with_result(niri_config: &Path) -> ImportResult {
    let mut settings = Settings::default();
    let default_settings = Settings::default();
    let mut warnings = Vec::new();
    let mut includes_processed = 0;

    info!("Importing settings from {:?}", niri_config);
    import_from_niri_config_recursive_tracked(
        niri_config,
        &mut settings,
        0,
        &mut warnings,
        &mut includes_processed,
    );

    // Always load keybindings (already has its own include traversal)
    load_keybindings(niri_config, &mut settings.keybindings);

    // Validate and clamp all values to valid ranges
    settings.validate();

    // Determine which sections were imported vs defaulted
    // Use &'static str to avoid allocations, convert to String only at the end
    let mut imported: Vec<&'static str> = Vec::new();
    let mut defaulted: Vec<&'static str> = Vec::new();

    // Helper macro to check section and categorize as imported or defaulted
    macro_rules! check_section {
        ($field:expr, $default:expr, $name:literal) => {
            if $field != $default {
                imported.push($name);
            } else {
                defaulted.push($name);
            }
        };
    }

    // Check each major section for changes from defaults
    check_section!(
        settings.appearance,
        default_settings.appearance,
        "appearance"
    );
    check_section!(settings.behavior, default_settings.behavior, "behavior");
    check_section!(settings.keyboard, default_settings.keyboard, "keyboard");
    check_section!(settings.mouse, default_settings.mouse, "mouse");
    check_section!(settings.touchpad, default_settings.touchpad, "touchpad");
    check_section!(
        settings.animations,
        default_settings.animations,
        "animations"
    );
    check_section!(settings.cursor, default_settings.cursor, "cursor");
    check_section!(
        settings.miscellaneous,
        default_settings.miscellaneous,
        "miscellaneous"
    );
    check_section!(settings.debug, default_settings.debug, "debug");
    check_section!(
        settings.switch_events,
        default_settings.switch_events,
        "switch-events"
    );

    // Convert to owned Strings (only allocates here, at the end)
    let mut imported_sections: Vec<String> = imported.into_iter().map(String::from).collect();
    let defaulted_sections: Vec<String> = defaulted.into_iter().map(String::from).collect();

    // Add collection-based sections with counts (these need format! anyway)
    if !settings.outputs.outputs.is_empty() {
        imported_sections.push(format!("outputs ({})", settings.outputs.outputs.len()));
    }
    if !settings.window_rules.rules.is_empty() {
        imported_sections.push(format!(
            "window-rules ({})",
            settings.window_rules.rules.len()
        ));
    }
    if !settings.workspaces.workspaces.is_empty() {
        imported_sections.push(format!(
            "workspaces ({})",
            settings.workspaces.workspaces.len()
        ));
    }
    if !settings.startup.commands.is_empty() {
        imported_sections.push(format!("startup ({})", settings.startup.commands.len()));
    }
    if !settings.environment.variables.is_empty() {
        imported_sections.push(format!(
            "environment ({})",
            settings.environment.variables.len()
        ));
    }

    info!(
        "Import complete: {} sections imported, {} includes processed",
        imported_sections.len(),
        includes_processed
    );

    ImportResult {
        settings,
        imported_sections,
        defaulted_sections,
        warnings,
        includes_processed,
    }
}

/// Recursive helper for import with depth tracking and warning collection
fn import_from_niri_config_recursive_tracked(
    niri_config: &Path,
    settings: &mut Settings,
    depth: usize,
    warnings: &mut Vec<String>,
    includes_processed: &mut usize,
) {
    if depth > MAX_INCLUDE_DEPTH {
        let msg = format!(
            "Include depth exceeded maximum of {}, stopping traversal",
            MAX_INCLUDE_DEPTH
        );
        warn!("{}", msg);
        warnings.push(msg);
        return;
    }

    let config_dir = niri_config.parent().unwrap_or(Path::new("."));

    // Read config file
    let Some(doc) = helpers::read_kdl_file(niri_config) else {
        if depth == 0 {
            let msg = "Could not read niri config for import, using defaults".to_string();
            info!("{}", msg);
            warnings.push(msg);
        } else {
            let msg = format!("Could not read included file: {:?}", niri_config);
            warn!("{}", msg);
            warnings.push(msg);
        }
        return;
    };

    // Import from this document
    import_from_document(&doc, settings);

    // Traverse includes and import from them too
    for node in doc.nodes() {
        if node.name().value() == "include" {
            if let Some(path_str) = node.entries().first().and_then(|e| e.value().as_string()) {
                if let Some(resolved) = resolve_include_path(path_str, config_dir) {
                    debug!(
                        "Following include (depth {}): {} -> {:?}",
                        depth, path_str, resolved
                    );
                    *includes_processed += 1;
                    import_from_niri_config_recursive_tracked(
                        &resolved,
                        settings,
                        depth + 1,
                        warnings,
                        includes_processed,
                    );
                } else {
                    let msg = format!("Skipped include (security): {}", path_str);
                    debug!("{}", msg);
                    warnings.push(msg);
                }
            }
        }
    }
}

/// Import all settings from a single KDL document
fn import_from_document(doc: &KdlDocument, settings: &mut Settings) {
    // Macro to reduce boilerplate for "get node, get children, call parser" pattern
    macro_rules! parse_node_children {
        ($doc:expr, $node_name:literal, $parser:ident, $settings:expr) => {
            if let Some(node) = $doc.get($node_name) {
                if let Some(children) = node.children() {
                    $parser(children, $settings);
                }
            }
        };
    }

    // Appearance - has import-specific corner_radius handling
    import_appearance_from_doc(doc, settings);

    // Behavior - direct call to shared parser
    parse_behavior_from_doc(doc, settings);

    // Input devices - has import-specific mod-key handling
    import_input_from_doc(doc, settings);

    // Display settings - use macro for node children parsing
    parse_node_children!(doc, "animations", parse_animations_from_children, settings);
    parse_node_children!(doc, "cursor", parse_cursor_from_children, settings);
    parse_node_children!(doc, "overview", parse_overview_from_children, settings);
    import_outputs_from_doc(doc, settings);

    // Layout extras - nested in layout node
    parse_node_children!(doc, "layout", parse_layout_extras_from_children, settings);

    // Direct calls to shared parsers (no wrapper needed)
    parse_gestures_from_doc(doc, settings);
    parse_misc_from_doc(doc, settings);

    // Collection-based settings (need import-specific ID management)
    import_workspaces_from_doc(doc, settings);
    import_layer_rules_from_doc(doc, settings);
    import_window_rules_from_doc(doc, settings);

    // Direct calls to shared parsers
    parse_startup_from_doc(doc, settings);
    parse_environment_from_doc(doc, settings);
    parse_debug_from_doc(doc, settings);
    parse_switch_events_from_doc(doc, settings);
}

/// Resolve an include path relative to the config directory
///
/// Returns None if the path escapes the allowed config directories for security.
fn resolve_include_path(include_path: &str, config_dir: &Path) -> Option<PathBuf> {
    let path = include_path.trim_matches('"');

    let resolved = if let Some(stripped) = path.strip_prefix("~/") {
        // Expand ~ to home directory
        dirs::home_dir()?.join(stripped)
    } else if path.starts_with('/') {
        // Absolute path
        PathBuf::from(path)
    } else {
        // Relative to config directory
        config_dir.join(path)
    };

    // Canonicalize to resolve .. and symlinks, verify within allowed directories
    let canonical = match resolved.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            debug!("Include path {:?} cannot be resolved: {}", path, e);
            return None;
        }
    };

    // Get allowed base directories - respect $XDG_CONFIG_HOME
    let config_base = dirs::config_dir()?;
    let niri_config_dir = config_base.join("niri");

    // Only allow paths under the XDG-compliant niri config dir
    if canonical.starts_with(&niri_config_dir) {
        Some(canonical)
    } else {
        warn!(
            "Include path {:?} escapes allowed config directory, ignoring for security",
            path
        );
        None
    }
}

// Import helper functions for each settings category
// These parse a KdlDocument and populate the relevant Settings fields

fn import_appearance_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // Use shared parsing logic for layout settings
    parse_appearance_from_doc(doc, settings);

    // Import-specific: Window rule for corner radius (global)
    // Note: We only import the first window-rule with geometry-corner-radius as the global one
    // This differs from the loader which expects a single window-rule in the managed file
    for node in doc.nodes() {
        if node.name().value() == "window-rule" {
            if let Some(wr_children) = node.children() {
                // Check if this is a "catch-all" rule (no match criteria)
                let has_match = wr_children
                    .nodes()
                    .iter()
                    .any(|n| n.name().value() == "match");
                if !has_match {
                    if let Some(cr) = get_i64(wr_children, &["geometry-corner-radius"]) {
                        settings.appearance.corner_radius = cr as f32;
                        break; // Only use first catch-all rule
                    }
                }
            }
        }
    }
}

fn import_input_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    if let Some(input) = doc.get("input") {
        if let Some(input_children) = input.children() {
            // Parse global input settings (mod-key, disable-power-key-handling, etc.)
            parse_global_input_settings(input_children, settings);

            // Use shared parsing functions for each input device
            if let Some(kbd) = input_children.get("keyboard") {
                if let Some(kbd_children) = kbd.children() {
                    parse_keyboard_from_children(kbd_children, settings);
                }
            }

            if let Some(mouse) = input_children.get("mouse") {
                if let Some(m_children) = mouse.children() {
                    parse_mouse_from_children(m_children, settings);
                }
            }

            if let Some(tp) = input_children.get("touchpad") {
                if let Some(tp_children) = tp.children() {
                    parse_touchpad_from_children(tp_children, settings);
                }
            }

            if let Some(tp) = input_children.get("trackpoint") {
                if let Some(tp_children) = tp.children() {
                    parse_trackpoint_from_children(tp_children, settings);
                }
            }

            if let Some(tb) = input_children.get("trackball") {
                if let Some(tb_children) = tb.children() {
                    parse_trackball_from_children(tb_children, settings);
                }
            }

            if let Some(tablet) = input_children.get("tablet") {
                if let Some(t_children) = tablet.children() {
                    parse_tablet_from_children(t_children, settings);
                }
            }

            if let Some(touch) = input_children.get("touch") {
                if let Some(t_children) = touch.children() {
                    parse_touch_from_children(t_children, settings);
                }
            }
        }
    }
}

/// Parse global input settings like mod-key, mod-key-nested, disable-power-key-handling
fn parse_global_input_settings(doc: &KdlDocument, settings: &mut Settings) {
    use super::super::parser::{get_string, has_flag};
    use crate::types::ModKey;

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
}

fn import_outputs_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    for node in doc.nodes() {
        if node.name().value() == "output" {
            let name = node
                .entries()
                .first()
                .and_then(|e| e.value().as_string())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let mut output = OutputConfig {
                name,
                ..Default::default()
            };

            if let Some(o_children) = node.children() {
                parse_output_node_children(o_children, &mut output);
            }

            settings.outputs.outputs.push(output);
        }
    }
}

fn import_workspaces_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    for node in doc.nodes() {
        if node.name().value() == "workspace" {
            let name = if let Some(entry) = node.entries().first() {
                if let Some(s) = entry.value().as_string() {
                    s.to_string()
                } else {
                    // Argument present but not a string - warn and skip
                    warn!(
                        "Workspace has non-string name argument during import: {:?}, skipping",
                        entry.value()
                    );
                    continue;
                }
            } else {
                continue;
            };

            if name.is_empty() {
                warn!("Workspace has empty name during import, skipping");
                continue;
            }

            let id = settings.workspaces.next_id;
            settings.workspaces.next_id += 1;

            let mut workspace = NamedWorkspace {
                id,
                name,
                open_on_output: None,
                layout_override: None,
            };

            if let Some(children) = node.children() {
                parse_workspace_node_children(children, &mut workspace);
            }

            settings.workspaces.workspaces.push(workspace);
        }
    }
}

fn import_layer_rules_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    for node in doc.nodes() {
        if node.name().value() == "layer-rule" {
            let id = settings.layer_rules.next_id;
            settings.layer_rules.next_id += 1;

            let mut rule = LayerRule {
                id,
                name: format!("Layer Rule {}", id + 1),
                ..Default::default()
            };

            if let Some(children) = node.children() {
                parse_layer_rule_node_children(children, &mut rule);
            }

            settings.layer_rules.rules.push(rule);
        }
    }
}

fn import_window_rules_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    for node in doc.nodes() {
        if node.name().value() == "window-rule" {
            // Skip catch-all rules (no match criteria) - they're handled in appearance for global corner radius
            if let Some(children) = node.children() {
                let has_match = children.nodes().iter().any(|n| n.name().value() == "match");
                if !has_match {
                    continue;
                }
            }

            let id = settings.window_rules.next_id;
            settings.window_rules.next_id += 1;

            let mut rule = WindowRule {
                id,
                name: format!("Rule {}", id + 1),
                ..Default::default()
            };

            if let Some(wr_children) = node.children() {
                parse_window_rule_node_children(wr_children, &mut rule);
            }

            settings.window_rules.rules.push(rule);
        }
    }
}
