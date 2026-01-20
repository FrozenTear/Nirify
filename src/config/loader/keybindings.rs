//! Keybindings loader - reads keybindings from KDL files
//!
//! This module parses keybindings from KDL configuration files, including
//! the managed keybindings.kdl and user's niri config (for import).
//! Supports following include directives within the niri config directory.

use super::helpers::read_kdl_file;
use crate::config::models::{KeybindAction, Keybinding, KeybindingsSettings};
use crate::config::parser::parse_document;
use kdl::{KdlDocument, KdlNode};
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};

/// Load keybindings from the user's niri config file
pub fn load_keybindings(niri_config_path: &Path, settings: &mut KeybindingsSettings) {
    settings.bindings.clear();
    settings.loaded = false;
    settings.error = None;
    settings.source_file = None;

    debug!("Loading keybindings from {:?}", niri_config_path);

    // Try to read the main config
    let config_content = match fs::read_to_string(niri_config_path) {
        Ok(content) => content,
        Err(e) => {
            debug!("Could not read niri config: {}", e);
            settings.error = Some(format!("Could not read niri config: {}", e));
            return;
        }
    };

    let doc = match parse_document(&config_content) {
        Ok(doc) => doc,
        Err(e) => {
            debug!("Could not parse niri config: {}", e);
            settings.error = Some(format!("Could not parse niri config: {}", e));
            return;
        }
    };

    let config_dir = niri_config_path.parent().unwrap_or(Path::new("."));
    let mut id_counter = 0u32;

    // Check for binds in the main config
    if let Some(binds_node) = doc.get("binds") {
        debug!("Found binds block in main config");
        if let Some(binds_doc) = binds_node.children() {
            parse_binds_block(binds_doc, &mut settings.bindings, &mut id_counter);
            settings.source_file = Some(niri_config_path.display().to_string());
        }
    }

    // Also look for include directives that might contain bindings
    for node in doc.nodes() {
        if node.name().value() == "include" {
            if let Some(include_path) = node.entries().first().and_then(|e| e.value().as_string()) {
                // resolve_include_path returns None if the path is outside allowed directories
                let Some(resolved_path) = resolve_include_path(include_path, config_dir) else {
                    continue;
                };
                debug!("Found include: {} -> {:?}", include_path, resolved_path);

                if let Some(included_doc) = read_kdl_file(&resolved_path) {
                    // Check if the included file has a binds block
                    if let Some(binds_node) = included_doc.get("binds") {
                        debug!("Found binds block in {:?}", resolved_path);
                        if let Some(binds_doc) = binds_node.children() {
                            let count_before = settings.bindings.len();
                            parse_binds_block(binds_doc, &mut settings.bindings, &mut id_counter);
                            debug!(
                                "Parsed {} bindings from {:?}",
                                settings.bindings.len() - count_before,
                                resolved_path
                            );
                            if settings.bindings.len() > count_before {
                                // Update source file to include this path
                                if settings.source_file.is_none() {
                                    settings.source_file =
                                        Some(resolved_path.display().to_string());
                                }
                            }
                        }
                    }
                } else {
                    debug!("Could not read/parse included file {:?}", resolved_path);
                }
            }
        }
    }

    settings.loaded = !settings.bindings.is_empty();
    debug!(
        "Loaded {} keybindings from {:?}",
        settings.bindings.len(),
        settings.source_file
    );
}

/// Resolve an include path relative to the config directory
///
/// Returns None if the path escapes the allowed config directories for security.
/// Only paths under the XDG-compliant niri config dir are allowed.
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
            log::debug!("Include path {:?} cannot be resolved: {}", path, e);
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
        log::warn!(
            "Include path {:?} escapes allowed config directory, ignoring for security",
            path
        );
        None
    }
}

/// Parse a binds block and extract all keybindings
fn parse_binds_block(
    binds_doc: &KdlDocument,
    bindings: &mut Vec<Keybinding>,
    id_counter: &mut u32,
) {
    for node in binds_doc.nodes() {
        if let Some(binding) = parse_single_binding(node, id_counter) {
            bindings.push(binding);
        }
    }
}

/// Parse a single keybinding node
fn parse_single_binding(node: &KdlNode, id_counter: &mut u32) -> Option<Keybinding> {
    let key_combo = node.name().value().to_string();

    // Skip comment-like nodes or invalid names
    if key_combo.is_empty() || key_combo.starts_with("//") {
        return None;
    }

    let mut binding = Keybinding {
        id: *id_counter,
        key_combo,
        hotkey_overlay_title: None,
        allow_when_locked: false,
        cooldown_ms: None,
        repeat: false,
        action: KeybindAction::NiriAction(String::new()),
    };
    *id_counter += 1;

    // Parse properties from entries
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "hotkey-overlay-title" => {
                    if let Some(title) = entry.value().as_string() {
                        binding.hotkey_overlay_title = Some(title.to_string());
                    }
                }
                "allow-when-locked" => {
                    if let Some(val) = entry.value().as_bool() {
                        binding.allow_when_locked = val;
                    }
                }
                "cooldown-ms" => {
                    if let Some(val) = entry.value().as_integer() {
                        binding.cooldown_ms = Some(val as i32);
                    }
                }
                "repeat" => {
                    if let Some(val) = entry.value().as_bool() {
                        binding.repeat = val;
                    }
                }
                _ => {}
            }
        }
    }

    // Parse action from children
    if let Some(children) = node.children() {
        binding.action = parse_action(children);
    }

    Some(binding)
}

/// Parse the action from a binding's children
fn parse_action(doc: &KdlDocument) -> KeybindAction {
    // Get the first action node (only one action per binding)
    let Some(node) = doc.nodes().first() else {
        return KeybindAction::NiriAction("(unknown)".to_string());
    };

    let action_name = node.name().value();

    if action_name == "spawn" {
        // Spawn action: collect all string arguments
        let args: Vec<String> = node
            .entries()
            .iter()
            .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
            .collect();
        return KeybindAction::Spawn(args);
    }

    // For other niri actions, check if there are arguments
    let args: Vec<String> = node
        .entries()
        .iter()
        .filter_map(|e| {
            // Get string or integer arguments
            e.value()
                .as_string()
                .map(|s| s.to_string())
                .or_else(|| e.value().as_integer().map(|i| i.to_string()))
        })
        .collect();

    // Also check for children that might be action arguments
    // Niri uses several formats:
    // - focus-workspace { "1"; }          -> child node name IS the argument
    // - focus-workspace { reference "name"; } -> child has name and value
    if let Some(children) = node.children() {
        let mut child_args: Vec<String> = Vec::new();
        for child_node in children.nodes() {
            let child_name = child_node.name().value();
            // Check if child has entries (e.g., reference "name")
            if let Some(entry) = child_node.entries().first() {
                if let Some(s) = entry.value().as_string() {
                    // Format: child-name "value"
                    child_args.push(format!("{}: {}", child_name, s));
                } else if let Some(i) = entry.value().as_integer() {
                    child_args.push(format!("{}: {}", child_name, i));
                }
            } else {
                // The child node name itself is the argument (e.g., "1" in focus-workspace { "1"; })
                child_args.push(child_name.to_string());
            }
        }
        if !child_args.is_empty() {
            return KeybindAction::NiriActionWithArgs(action_name.to_string(), child_args);
        }
    }

    if args.is_empty() {
        KeybindAction::NiriAction(action_name.to_string())
    } else {
        KeybindAction::NiriActionWithArgs(action_name.to_string(), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_binding() {
        let content = r#"
binds {
    Mod+Space hotkey-overlay-title="App Launcher" {
        spawn "dmenu_run";
    }
}
"#;
        let doc = parse_document(content).unwrap();
        let binds_node = doc.get("binds").unwrap();
        let binds_doc = binds_node.children().unwrap();

        let mut bindings = Vec::new();
        let mut id = 0;
        parse_binds_block(binds_doc, &mut bindings, &mut id);

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].key_combo, "Mod+Space");
        assert_eq!(
            bindings[0].hotkey_overlay_title,
            Some("App Launcher".to_string())
        );
        assert!(
            matches!(&bindings[0].action, KeybindAction::Spawn(args) if args == &["dmenu_run"])
        );
    }

    #[test]
    fn test_parse_media_key_binding() {
        // KDL v2 (kdl crate 6.x) uses space-separated properties
        let content = r#"
binds {
    XF86AudioMute allow-when-locked=#true {
        spawn "wpctl" "set-mute" "toggle";
    }
}
"#;
        let doc = parse_document(content).unwrap();
        let binds_node = doc.get("binds").unwrap();
        let binds_doc = binds_node.children().unwrap();

        let mut bindings = Vec::new();
        let mut id = 0;
        parse_binds_block(binds_doc, &mut bindings, &mut id);

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].key_combo, "XF86AudioMute");
        assert!(bindings[0].allow_when_locked);
        assert!(matches!(&bindings[0].action, KeybindAction::Spawn(args) if args.len() == 3));
    }

    #[test]
    fn test_parse_niri_action() {
        let content = r#"
binds {
    Mod+Q {
        close-window;
    }
}
"#;
        let doc = parse_document(content).unwrap();
        let binds_node = doc.get("binds").unwrap();
        let binds_doc = binds_node.children().unwrap();

        let mut bindings = Vec::new();
        let mut id = 0;
        parse_binds_block(binds_doc, &mut bindings, &mut id);

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].key_combo, "Mod+Q");
        assert!(
            matches!(&bindings[0].action, KeybindAction::NiriAction(action) if action == "close-window")
        );
    }
}
