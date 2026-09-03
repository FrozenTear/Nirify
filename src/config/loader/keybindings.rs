//! Keybindings loader - reads keybindings from KDL files
//!
//! This module parses keybindings from KDL configuration files, including
//! the managed keybindings.kdl and user's niri config (for import).
//! Supports following include directives within the niri config directory.

use super::helpers::read_kdl_file;
use crate::config::include::{
    default_niri_config_dir, open_include_for_import, parse_include_node, IncludeOpen,
};
use crate::config::models::{
    last_wins_keybindings, ActionNode, ActionValue, HotkeyOverlayTitle, KeybindAction, Keybinding,
    KeybindingsSettings,
};
use crate::config::parser::parse_document;
use crate::config::replace::is_nirify_include_path;
use kdl::{KdlDocument, KdlNode};
use log::debug;
use std::fs;
use std::path::Path;

/// Maximum include depth (same as settings import) to prevent circular includes.
const MAX_INCLUDE_DEPTH: usize = 10;

/// Load keybindings from the user's niri config file
pub fn load_keybindings(niri_config_path: &Path, settings: &mut KeybindingsSettings) {
    settings.bindings.clear();
    settings.loaded = false;
    settings.error = None;
    settings.source_file = None;

    debug!("Loading keybindings from {:?}", niri_config_path);

    // Try to read the config file
    let config_content = match fs::read_to_string(niri_config_path) {
        Ok(content) => content,
        Err(e) => {
            debug!("Could not read keybindings config: {}", e);
            // File not existing is normal for first run, don't treat as error
            if e.kind() != std::io::ErrorKind::NotFound {
                settings.error = Some(format!("Could not read keybindings config: {}", e));
            }
            return;
        }
    };

    let doc = match crate::config::include::parse_kdl_with_niri_includes(&config_content) {
        Ok(doc) => doc,
        Err(e) => {
            debug!("Could not parse keybindings config: {}", e);
            settings.error = Some(format!(
                "Could not parse keybindings config ({}): {}",
                niri_config_path.display(),
                e
            ));
            return;
        }
    };

    let config_dir = niri_config_path.parent().unwrap_or(Path::new("."));
    let mut id_counter = 0u32;

    load_keybindings_walk(
        &doc,
        config_dir,
        niri_config_path,
        settings,
        &mut id_counter,
        0,
    );

    // niri last-wins: later duplicate combos (same file or later include) win.
    settings.bindings = last_wins_keybindings(std::mem::take(&mut settings.bindings));
    settings.loaded = !settings.bindings.is_empty();
    debug!(
        "Loaded {} keybindings from {:?}",
        settings.bindings.len(),
        settings.source_file
    );
}

/// Walk top-level nodes in document order, interleaving `binds` with includes.
fn load_keybindings_walk(
    doc: &KdlDocument,
    config_dir: &Path,
    source_path: &Path,
    settings: &mut KeybindingsSettings,
    id_counter: &mut u32,
    depth: usize,
) {
    if depth > MAX_INCLUDE_DEPTH {
        debug!(
            "Keybindings include depth exceeded maximum of {}, stopping",
            MAX_INCLUDE_DEPTH
        );
        return;
    }

    for node in doc.nodes() {
        match node.name().value() {
            "binds" => {
                debug!("Found binds block in {:?}", source_path);
                if let Some(binds_doc) = node.children() {
                    parse_binds_block(binds_doc, &mut settings.bindings, id_counter);
                    if settings.source_file.is_none() {
                        settings.source_file = Some(source_path.display().to_string());
                    }
                }
            }
            "include" => {
                let Some(directive) = parse_include_node(node) else {
                    continue;
                };
                if is_nirify_include_path(&directive.path) {
                    debug!(
                        "Skipping Nirify include while loading keybindings: {}",
                        directive.path
                    );
                    continue;
                }
                let IncludeOpen::Ready(resolved_path) = open_include_for_import(
                    &directive,
                    config_dir,
                    dirs::home_dir().as_deref(),
                    default_niri_config_dir().as_deref(),
                ) else {
                    continue;
                };
                debug!("Found include: {} -> {:?}", directive.path, resolved_path);
                if let Some(included_doc) = read_kdl_file(&resolved_path) {
                    let included_dir = resolved_path.parent().unwrap_or(config_dir);
                    load_keybindings_walk(
                        &included_doc,
                        included_dir,
                        &resolved_path,
                        settings,
                        id_counter,
                        depth + 1,
                    );
                } else {
                    debug!("Could not read/parse included file {:?}", resolved_path);
                }
            }
            _ => {}
        }
    }
}

/// Load keybindings from a KDL document without following includes.
///
/// All `binds` blocks are read in document order; duplicate combos last-win
/// (same as niri and [`load_keybindings`]).
pub fn load_keybindings_from_doc(doc: &KdlDocument, settings: &mut KeybindingsSettings) {
    settings.bindings.clear();
    settings.loaded = false;
    settings.error = None;
    settings.source_file = None;

    let mut id_counter = 0u32;
    for node in doc.nodes() {
        if node.name().value() == "binds" {
            if let Some(binds_doc) = node.children() {
                parse_binds_block(binds_doc, &mut settings.bindings, &mut id_counter);
            }
        }
    }
    settings.bindings = last_wins_keybindings(std::mem::take(&mut settings.bindings));
    settings.loaded = !settings.bindings.is_empty();
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

fn parse_single_binding(node: &KdlNode, id_counter: &mut u32) -> Option<Keybinding> {
    let key_combo = node.name().value().to_string();

    // Skip comment-like nodes or invalid names
    if key_combo.is_empty() || key_combo.starts_with("//") {
        return None;
    }

    let mut binding = Keybinding {
        id: *id_counter,
        key_combo,
        ..Default::default()
    };
    *id_counter += 1;

    // Parse properties from entries (niri defaults: repeat=true, allow-inhibiting=true)
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            match name.value() {
                "hotkey-overlay-title" => {
                    if matches!(entry.value(), kdl::KdlValue::Null) {
                        binding.hotkey_overlay_title = HotkeyOverlayTitle::Hidden;
                    } else if let Some(title) = entry.value().as_string() {
                        binding.hotkey_overlay_title =
                            HotkeyOverlayTitle::Custom(title.to_string());
                    }
                }
                "allow-when-locked" => {
                    if let Some(val) = entry.value().as_bool() {
                        binding.allow_when_locked = val;
                    }
                }
                "allow-inhibiting" => {
                    if let Some(val) = entry.value().as_bool() {
                        binding.allow_inhibiting = val;
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

    // Parse action from children (exactly one action node per bind)
    if let Some(children) = node.children() {
        binding.action = parse_action(children);
    }

    Some(binding)
}

fn parse_action(doc: &KdlDocument) -> KeybindAction {
    // Get the first action node (only one action per binding)
    let Some(node) = doc.nodes().first() else {
        return KeybindAction::NiriAction(ActionNode::bare("(unknown)"));
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

    if action_name == "spawn-sh" {
        let cmd = node
            .entries()
            .iter()
            .find_map(|e| e.value().as_string().map(|s| s.to_string()))
            .unwrap_or_default();
        return KeybindAction::SpawnSh(cmd);
    }

    // Generic lossless capture: positional args + named props, preserving types.
    let mut action = ActionNode::bare(action_name);
    for entry in node.entries() {
        let value = kdl_to_action_value(entry.value());
        if let Some(name) = entry.name() {
            action.props.push((name.value().to_string(), value));
        } else {
            action.args.push(value);
        }
    }

    // Action nodes have no children in niri's grammar; ignore any (log a warning).
    if node.children().is_some() {
        log::warn!(
            "Ignoring unexpected child nodes on action '{}' (not valid niri grammar)",
            action_name
        );
    }

    KeybindAction::NiriAction(action)
}

/// Convert a KDL value into a lossless `ActionValue`.
fn kdl_to_action_value(value: &kdl::KdlValue) -> ActionValue {
    if matches!(value, kdl::KdlValue::Null) {
        ActionValue::Null
    } else if let Some(b) = value.as_bool() {
        ActionValue::Bool(b)
    } else if let Some(i) = value.as_integer() {
        ActionValue::Int(i as i64)
    } else if let Some(f) = value.as_float() {
        ActionValue::Float(f)
    } else if let Some(s) = value.as_string() {
        ActionValue::Str(s.to_string())
    } else {
        ActionValue::Null
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
            HotkeyOverlayTitle::Custom("App Launcher".to_string())
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
            matches!(&bindings[0].action, KeybindAction::NiriAction(node) if node.name == "close-window" && node.args.is_empty() && node.props.is_empty())
        );
        // niri defaults preserved
        assert!(bindings[0].repeat);
        assert!(bindings[0].allow_inhibiting);
    }

    #[test]
    fn test_parse_action_with_args_and_props() {
        let content = r#"
binds {
    Mod+1 { focus-workspace 1; }
    Mod+2 { move-column-to-workspace-down focus=false; }
    Mod+3 { set-column-width "+10%"; }
}
"#;
        let doc = parse_document(content).unwrap();
        let binds_doc = doc.get("binds").unwrap().children().unwrap();
        let mut bindings = Vec::new();
        let mut id = 0;
        parse_binds_block(binds_doc, &mut bindings, &mut id);

        assert_eq!(bindings.len(), 3);
        match &bindings[0].action {
            KeybindAction::NiriAction(node) => {
                assert_eq!(node.name, "focus-workspace");
                assert_eq!(node.args, vec![ActionValue::Int(1)]);
            }
            other => panic!("expected NiriAction, got {other:?}"),
        }
        match &bindings[1].action {
            KeybindAction::NiriAction(node) => {
                assert_eq!(node.name, "move-column-to-workspace-down");
                assert_eq!(
                    node.props,
                    vec![("focus".to_string(), ActionValue::Bool(false))]
                );
            }
            other => panic!("expected NiriAction, got {other:?}"),
        }
        match &bindings[2].action {
            KeybindAction::NiriAction(node) => {
                assert_eq!(node.args, vec![ActionValue::Str("+10%".to_string())]);
            }
            other => panic!("expected NiriAction, got {other:?}"),
        }
    }

    #[test]
    fn load_duplicate_mod_q_last_wins() {
        let content = r#"
binds {
    Mod+Q { close-window; }
    Mod+Return { spawn "alacritty"; }
    mod+q { spawn "kitty"; }
}
"#;
        let doc = parse_document(content).unwrap();
        let mut loaded = KeybindingsSettings::default();
        load_keybindings_from_doc(&doc, &mut loaded);
        assert_eq!(loaded.bindings.len(), 2);
        let mod_q = loaded
            .bindings
            .iter()
            .find(|b| {
                crate::config::models::normalized_key_combo(&b.key_combo)
                    == normalized_combo("Mod+Q")
            })
            .expect("Mod+Q kept");
        assert!(
            matches!(&mod_q.action, KeybindAction::Spawn(args) if args == &["kitty"]),
            "last Mod+Q action must win, got {:?}",
            mod_q.action
        );
    }

    fn normalized_combo(s: &str) -> String {
        crate::config::models::normalized_key_combo(s)
    }
}
