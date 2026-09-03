//! Keybindings KDL generation
//!
//! Generates KDL configuration for keybindings managed by Nirify.

use crate::config::models::{
    last_wins_keybindings, lookup_action, ActionNode, ActionValue, ArgKind, HotkeyOverlayTitle,
    KeybindAction, Keybinding, KeybindingsSettings,
};
use crate::config::parser::parse_document;

/// Generate keybindings.kdl content from settings.
///
/// Duplicate combos are **last-wins**, matching niri's `binds` merge
/// (later entry overrides the same key). Earlier first-wins disagreed with
/// niri and could persist the wrong action after first-run / absorb.
pub fn generate_keybindings_kdl(settings: &KeybindingsSettings) -> String {
    let mut content = String::with_capacity(2048);
    content.push_str("// Keybindings - managed by Nirify-rust\n");
    content.push_str("// Edit these bindings in the Nirify app\n\n");

    if settings.bindings.is_empty() {
        content.push_str("// No keybindings configured yet.\n");
        content.push_str("// Add keybindings using the Nirify app.\n");
        return content;
    }

    content.push_str("binds {\n");

    // niri last-wins: keep the last valid binding for each normalized combo.
    let last_wins: Vec<Keybinding> = last_wins_keybindings(
        settings
            .bindings
            .iter()
            .filter(|b| is_valid_keybinding(b))
            .cloned()
            .collect(),
    );
    if last_wins.len()
        < settings
            .bindings
            .iter()
            .filter(|b| is_valid_keybinding(b))
            .count()
    {
        log::warn!(
            "Dropped {} earlier duplicate keybinding(s) (niri last-wins)",
            settings
                .bindings
                .iter()
                .filter(|b| is_valid_keybinding(b))
                .count()
                - last_wins.len()
        );
    }

    for binding in &last_wins {
        content.push_str(&generate_keybinding(binding));
    }

    content.push_str("}\n");
    content
}

/// Whether the action is a spawn (allow-when-locked is spawn-only in niri).
fn is_spawn_action(action: &KeybindAction) -> bool {
    match action {
        KeybindAction::Spawn(_) | KeybindAction::SpawnSh(_) => true,
        KeybindAction::Custom(raw) => {
            let first = raw.split_whitespace().next().unwrap_or("");
            first == "spawn" || first == "spawn-sh"
        }
        KeybindAction::NiriAction(_) => false,
    }
}

/// Check if a keybinding is valid for saving to config.
pub fn is_valid_keybinding(binding: &Keybinding) -> bool {
    if binding.key_combo.trim().is_empty() {
        return false;
    }

    match &binding.action {
        KeybindAction::Spawn(args) => !args.is_empty() && !args[0].trim().is_empty(),
        KeybindAction::SpawnSh(cmd) => !cmd.trim().is_empty(),
        KeybindAction::Custom(raw) => {
            // Must parse as exactly one KDL node.
            match parse_document(raw) {
                Ok(doc) => doc.nodes().len() == 1,
                Err(_) => false,
            }
        }
        KeybindAction::NiriAction(node) => {
            if node.name.trim().is_empty() {
                return false;
            }
            // Placeholder produced by the loader for an empty child block
            // (e.g. hand-edited `Mod+X { }`). Not a real niri action and
            // `(unknown)` is KDL type-annotation syntax that would fail the
            // reparse gate, so never emit it.
            if node.name == "(unknown)" {
                return false;
            }
            // If catalog says a primary argument is required, ensure it is present.
            if let Some(spec) = lookup_action(&node.name) {
                if spec.args.requires_primary_arg() {
                    match node.primary_arg() {
                        None => return false,
                        Some(ActionValue::Str(s)) if s.trim().is_empty() => return false,
                        Some(v) => {
                            if let ArgKind::SizeChange = spec.args {
                                return crate::config::models::is_valid_size_change(
                                    &v.as_display(),
                                );
                            }
                        }
                    }
                }
            }
            true
        }
    }
}

/// niri's default value for a given per-action boolean property, if any.
fn prop_default(action: &str, prop: &str) -> Option<ActionValue> {
    match (action, prop) {
        (_, "focus") => Some(ActionValue::Bool(true)),
        ("screenshot", "show-pointer") => Some(ActionValue::Bool(true)),
        ("screenshot-screen", "show-pointer") => Some(ActionValue::Bool(true)),
        ("screenshot-screen", "write-to-disk") => Some(ActionValue::Bool(true)),
        ("screenshot-window", "show-pointer") => Some(ActionValue::Bool(false)),
        ("screenshot-window", "write-to-disk") => Some(ActionValue::Bool(true)),
        ("quit", "skip-confirmation") => Some(ActionValue::Bool(false)),
        _ => None,
    }
}

/// Generate KDL for a single keybinding
fn generate_keybinding(binding: &Keybinding) -> String {
    let mut line = String::with_capacity(256);

    line.push_str("    ");
    line.push_str(&binding.key_combo);

    // Property order (stable for tests): hotkey-overlay-title, allow-when-locked,
    // allow-inhibiting, cooldown-ms, repeat.
    match &binding.hotkey_overlay_title {
        HotkeyOverlayTitle::Auto => {}
        HotkeyOverlayTitle::Hidden => line.push_str(" hotkey-overlay-title=null"),
        HotkeyOverlayTitle::Custom(s) => {
            line.push_str(&format!(" hotkey-overlay-title={}", quote_kdl_string(s)));
        }
    }

    // allow-when-locked is spawn-only in niri (decode error otherwise).
    if binding.allow_when_locked && is_spawn_action(&binding.action) {
        line.push_str(" allow-when-locked=true");
    }

    // allow-inhibiting defaults to true; emit only false. niri force-sets it
    // false for toggle-keyboard-shortcuts-inhibit, so never emit it there.
    let is_inhibit_toggle = matches!(
        &binding.action,
        KeybindAction::NiriAction(n) if n.name == "toggle-keyboard-shortcuts-inhibit"
    );
    if !binding.allow_inhibiting && !is_inhibit_toggle {
        line.push_str(" allow-inhibiting=false");
    }

    if let Some(cooldown) = binding.cooldown_ms {
        line.push_str(&format!(" cooldown-ms={}", cooldown));
    }

    // repeat defaults to true; emit only false.
    if !binding.repeat {
        line.push_str(" repeat=false");
    }

    // Action block
    line.push_str(" {\n");
    line.push_str(&generate_action(&binding.action));
    line.push_str("    }\n");

    line
}

/// Generate KDL for a keybinding action
fn generate_action(action: &KeybindAction) -> String {
    match action {
        KeybindAction::Spawn(args) => {
            let mut line = String::from("        spawn");
            for arg in args {
                line.push(' ');
                line.push_str(&quote_kdl_string(arg));
            }
            line.push_str(";\n");
            line
        }
        KeybindAction::SpawnSh(cmd) => {
            format!("        spawn-sh {};\n", quote_kdl_string(cmd))
        }
        KeybindAction::NiriAction(node) => generate_niri_action(node),
        KeybindAction::Custom(raw) => {
            let trimmed = raw.trim();
            if trimmed.ends_with(';') {
                format!("        {}\n", trimmed)
            } else {
                format!("        {};\n", trimmed)
            }
        }
    }
}

/// Generate KDL for a lossless niri action node.
fn generate_niri_action(node: &ActionNode) -> String {
    let mut line = format!("        {}", node.name);
    for arg in &node.args {
        line.push(' ');
        line.push_str(&arg.to_kdl());
    }
    for (name, value) in &node.props {
        // Drop explicit-default flag values (semantically identical to absence).
        if let Some(default) = prop_default(&node.name, name) {
            if &default == value {
                continue;
            }
        }
        // screenshot-window show-pointer is 26.04+; loaded values are preserved
        // (the user's niri already accepts them), so we always re-emit here.
        line.push(' ');
        line.push_str(name);
        line.push('=');
        line.push_str(&value.to_kdl());
    }
    line.push_str(";\n");
    line
}

/// Quote a string for KDL format.
fn quote_kdl_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip helper: generate KDL, write it to a temp file, then load it
    /// back through the public loader API.
    fn roundtrip(settings: &KeybindingsSettings) -> Vec<Keybinding> {
        let kdl = generate_keybindings_kdl(settings);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nirify_kb_roundtrip_{}_{}.kdl",
            std::process::id(),
            nanos
        ));
        std::fs::write(&path, &kdl).expect("write temp kdl");
        let mut loaded = KeybindingsSettings::default();
        crate::config::loader::load_keybindings(&path, &mut loaded);
        let _ = std::fs::remove_file(&path);
        loaded.bindings
    }

    fn binding(combo: &str, action: KeybindAction) -> Keybinding {
        Keybinding {
            key_combo: combo.to_string(),
            action,
            ..Default::default()
        }
    }

    fn niri(name: &str) -> KeybindAction {
        KeybindAction::NiriAction(ActionNode::bare(name))
    }

    #[test]
    fn test_generate_empty_keybindings() {
        let settings = KeybindingsSettings::default();
        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("No keybindings configured"));
        assert!(!kdl.contains("binds {"));
    }

    #[test]
    fn unknown_action_is_screened_and_not_emitted() {
        // The loader falls back to `(unknown)` for an empty child block
        // (e.g. hand-edited `Mod+X { }`). It must never be emitted: `(unknown)`
        // is KDL type-annotation syntax and would break the reparse gate.
        let b = binding("Mod+X", niri("(unknown)"));
        assert!(
            !is_valid_keybinding(&b),
            "the (unknown) placeholder action must be screened out"
        );

        let settings = KeybindingsSettings {
            bindings: vec![b, binding("Mod+Q", niri("close-window"))],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&settings);
        assert!(!kdl.contains("(unknown)"), "must not emit the placeholder");
        assert!(kdl.contains("close-window"), "valid binding still emitted");
        // Generated KDL must re-parse.
        assert!(
            parse_document(&kdl).is_ok(),
            "generated keybindings KDL must be valid"
        );
    }

    #[test]
    fn test_generate_spawn_keybinding() {
        let settings = KeybindingsSettings {
            bindings: vec![Keybinding {
                key_combo: "Mod+Space".to_string(),
                hotkey_overlay_title: HotkeyOverlayTitle::Custom("App Launcher".to_string()),
                action: KeybindAction::Spawn(vec!["dmenu_run".to_string()]),
                ..Default::default()
            }],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("binds {"));
        assert!(kdl.contains("Mod+Space"));
        assert!(kdl.contains("hotkey-overlay-title=\"App Launcher\""));
        assert!(kdl.contains("spawn \"dmenu_run\""));
    }

    #[test]
    fn test_generate_niri_action() {
        let settings = KeybindingsSettings {
            bindings: vec![binding("Mod+Q", niri("close-window"))],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("Mod+Q"));
        assert!(kdl.contains("close-window;"));
    }

    #[test]
    fn test_generate_with_all_properties() {
        let settings = KeybindingsSettings {
            bindings: vec![Keybinding {
                key_combo: "XF86AudioMute".to_string(),
                hotkey_overlay_title: HotkeyOverlayTitle::Custom("Mute".to_string()),
                allow_when_locked: true,
                allow_inhibiting: false,
                cooldown_ms: Some(100),
                repeat: false,
                action: KeybindAction::Spawn(vec![
                    "wpctl".to_string(),
                    "set-mute".to_string(),
                    "toggle".to_string(),
                ]),
                ..Default::default()
            }],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("allow-when-locked=true"));
        assert!(kdl.contains("allow-inhibiting=false"));
        assert!(kdl.contains("cooldown-ms=100"));
        assert!(kdl.contains("repeat=false"));
        assert!(!kdl.contains("repeat=true"));
        assert!(kdl.contains("spawn \"wpctl\" \"set-mute\""));
    }

    #[test]
    fn test_roundtrip_repeat() {
        let mut b = binding("Mod+A", niri("close-window"));
        b.repeat = false;
        let kdl = generate_keybindings_kdl(&KeybindingsSettings {
            bindings: vec![b],
            ..Default::default()
        });
        assert!(kdl.contains("repeat=false"));
        let out = roundtrip(&KeybindingsSettings {
            bindings: vec![{
                let mut b = binding("Mod+A", niri("close-window"));
                b.repeat = false;
                b
            }],
            ..Default::default()
        });
        assert!(!out[0].repeat);

        // Default true emits no repeat property and loads back true.
        let out = roundtrip(&KeybindingsSettings {
            bindings: vec![binding("Mod+B", niri("close-window"))],
            ..Default::default()
        });
        let kdl = generate_keybindings_kdl(&KeybindingsSettings {
            bindings: vec![binding("Mod+B", niri("close-window"))],
            ..Default::default()
        });
        assert!(!kdl.contains("repeat="));
        assert!(out[0].repeat);
    }

    #[test]
    fn test_roundtrip_allow_inhibiting() {
        let mut b = binding("Mod+I", niri("close-window"));
        b.allow_inhibiting = false;
        let s = KeybindingsSettings {
            bindings: vec![b],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(kdl.contains("allow-inhibiting=false"));
        assert!(!roundtrip(&s)[0].allow_inhibiting);

        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+J", niri("close-window"))],
            ..Default::default()
        };
        assert!(!generate_keybindings_kdl(&s).contains("allow-inhibiting"));
        assert!(roundtrip(&s)[0].allow_inhibiting);
    }

    #[test]
    fn test_roundtrip_overlay_title() {
        // Auto -> no property
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+Q", niri("close-window"))],
            ..Default::default()
        };
        assert!(!generate_keybindings_kdl(&s).contains("hotkey-overlay-title"));

        // Hidden -> null
        let mut b = binding("Mod+W", niri("close-window"));
        b.hotkey_overlay_title = HotkeyOverlayTitle::Hidden;
        let s = KeybindingsSettings {
            bindings: vec![b],
            ..Default::default()
        };
        assert!(generate_keybindings_kdl(&s).contains("hotkey-overlay-title=null"));
        assert_eq!(
            roundtrip(&s)[0].hotkey_overlay_title,
            HotkeyOverlayTitle::Hidden
        );

        // Custom round-trips
        let mut b = binding("Mod+E", niri("close-window"));
        b.hotkey_overlay_title = HotkeyOverlayTitle::Custom("App Launcher".to_string());
        let s = KeybindingsSettings {
            bindings: vec![b],
            ..Default::default()
        };
        assert_eq!(
            roundtrip(&s)[0].hotkey_overlay_title,
            HotkeyOverlayTitle::Custom("App Launcher".to_string())
        );
    }

    #[test]
    fn test_roundtrip_screenshot_props() {
        // screenshot-window write-to-disk=false round-trips.
        let mut node = ActionNode::bare("screenshot-window");
        node.set_prop("write-to-disk", Some(ActionValue::Bool(false)));
        let s = KeybindingsSettings {
            bindings: vec![binding("Print", KeybindAction::NiriAction(node))],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(kdl.contains("write-to-disk=false"));
        let out = roundtrip(&s);
        if let KeybindAction::NiriAction(n) = &out[0].action {
            assert_eq!(n.get_prop("write-to-disk"), Some(&ActionValue::Bool(false)));
        } else {
            panic!("expected NiriAction");
        }

        // Explicit-default show-pointer=true on `screenshot` is dropped.
        let mut node = ActionNode::bare("screenshot");
        node.set_prop("show-pointer", Some(ActionValue::Bool(true)));
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+Print", KeybindAction::NiriAction(node))],
            ..Default::default()
        };
        assert!(!generate_keybindings_kdl(&s).contains("show-pointer"));
        let out = roundtrip(&s);
        if let KeybindAction::NiriAction(n) = &out[0].action {
            assert!(n.get_prop("show-pointer").is_none());
        }
    }

    #[test]
    fn test_roundtrip_quit_skip_confirmation() {
        let mut node = ActionNode::bare("quit");
        node.set_prop("skip-confirmation", Some(ActionValue::Bool(true)));
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+Shift+E", KeybindAction::NiriAction(node))],
            ..Default::default()
        };
        assert!(generate_keybindings_kdl(&s).contains("skip-confirmation=true"));
        let out = roundtrip(&s);
        if let KeybindAction::NiriAction(n) = &out[0].action {
            assert_eq!(
                n.get_prop("skip-confirmation"),
                Some(&ActionValue::Bool(true))
            );
        }
    }

    #[test]
    fn test_roundtrip_workspace_ref_types() {
        let mut int_node = ActionNode::bare("focus-workspace");
        int_node.set_primary_arg(Some(ActionValue::Int(1)));
        let mut str_node = ActionNode::bare("focus-workspace");
        str_node.set_primary_arg(Some(ActionValue::Str("web".to_string())));
        let s = KeybindingsSettings {
            bindings: vec![
                binding("Mod+1", KeybindAction::NiriAction(int_node)),
                binding("Mod+2", KeybindAction::NiriAction(str_node)),
            ],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(kdl.contains("focus-workspace 1;"));
        assert!(kdl.contains("focus-workspace \"web\";"));
        let out = roundtrip(&s);
        if let KeybindAction::NiriAction(n) = &out[0].action {
            assert_eq!(n.primary_arg(), Some(&ActionValue::Int(1)));
        }
        if let KeybindAction::NiriAction(n) = &out[1].action {
            assert_eq!(n.primary_arg(), Some(&ActionValue::Str("web".to_string())));
        }
    }

    #[test]
    fn test_size_change_args_quoted() {
        let mut a = ActionNode::bare("set-column-width");
        a.set_primary_arg(Some(ActionValue::Str("+10%".to_string())));
        let mut b = ActionNode::bare("set-column-width");
        b.set_primary_arg(Some(ActionValue::Str("500".to_string())));
        let mut c = ActionNode::bare("focus-column");
        c.set_primary_arg(Some(ActionValue::Int(2)));
        let s = KeybindingsSettings {
            bindings: vec![
                binding("Mod+R", KeybindAction::NiriAction(a)),
                binding("Mod+T", KeybindAction::NiriAction(b)),
                binding("Mod+Y", KeybindAction::NiriAction(c)),
            ],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(kdl.contains("set-column-width \"+10%\";"));
        assert!(kdl.contains("set-column-width \"500\";"));
        assert!(kdl.contains("focus-column 2;"));
    }

    #[test]
    fn test_required_arg_action_skipped() {
        // set-column-width with no argument produces no node.
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+U", niri("set-column-width"))],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(!kdl.contains("set-column-width"));
    }

    #[test]
    fn test_duplicate_binds_last_wins() {
        let s = KeybindingsSettings {
            bindings: vec![
                binding("Mod+Q", niri("close-window")),
                binding("mod+q", niri("fullscreen-window")),
            ],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(
            kdl.contains("fullscreen-window"),
            "niri last-wins must keep the later Mod+Q action:\n{kdl}"
        );
        assert!(!kdl.contains("close-window"));
        assert_eq!(
            kdl.matches("Mod+Q").count() + kdl.matches("mod+q").count(),
            1
        );
    }

    #[test]
    fn test_allow_when_locked_spawn_only() {
        let mut nb = binding("Mod+N", niri("close-window"));
        nb.allow_when_locked = true;
        let mut sb = binding("Mod+T", KeybindAction::Spawn(vec!["alacritty".to_string()]));
        sb.allow_when_locked = true;
        let s = KeybindingsSettings {
            bindings: vec![nb, sb],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        // Only one occurrence (the spawn binding).
        assert_eq!(kdl.matches("allow-when-locked=true").count(), 1);
    }

    #[test]
    fn test_focus_flag_roundtrip() {
        let mut node = ActionNode::bare("move-column-to-workspace-down");
        node.set_prop("focus", Some(ActionValue::Bool(false)));
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+D", KeybindAction::NiriAction(node))],
            ..Default::default()
        };
        assert!(generate_keybindings_kdl(&s).contains("focus=false"));
        let out = roundtrip(&s);
        if let KeybindAction::NiriAction(n) = &out[0].action {
            assert_eq!(n.get_prop("focus"), Some(&ActionValue::Bool(false)));
        }

        // focus=true never emitted (default).
        let mut node = ActionNode::bare("move-column-to-workspace-down");
        node.set_prop("focus", Some(ActionValue::Bool(true)));
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+F", KeybindAction::NiriAction(node))],
            ..Default::default()
        };
        assert!(!generate_keybindings_kdl(&s).contains("focus="));
    }

    #[test]
    fn test_custom_action_emitted_verbatim() {
        let s = KeybindingsSettings {
            bindings: vec![binding(
                "Mod+X",
                KeybindAction::Custom("focus-workspace \"web\"".to_string()),
            )],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(kdl.contains("focus-workspace \"web\";"));

        // Invalid custom text is skipped.
        let s = KeybindingsSettings {
            bindings: vec![binding("Mod+Z", KeybindAction::Custom("{{{".to_string()))],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&s);
        assert!(!kdl.contains("{{{"));
    }

    #[test]
    fn test_generated_kdl_reparses() {
        let mut sz = ActionNode::bare("set-window-width");
        sz.set_primary_arg(Some(ActionValue::Str("50%".to_string())));
        let mut ws = ActionNode::bare("focus-workspace");
        ws.set_primary_arg(Some(ActionValue::Int(3)));
        let mut b = binding("Mod+Shift+E", niri("close-window"));
        b.hotkey_overlay_title = HotkeyOverlayTitle::Hidden;
        b.repeat = false;
        b.allow_inhibiting = false;
        let settings = KeybindingsSettings {
            bindings: vec![
                b,
                binding("Mod+T", KeybindAction::Spawn(vec!["alacritty".to_string()])),
                binding(
                    "Mod+G",
                    KeybindAction::SpawnSh("pkill orca || orca".to_string()),
                ),
                binding("Mod+W", KeybindAction::NiriAction(sz)),
                binding("Mod+3", KeybindAction::NiriAction(ws)),
            ],
            ..Default::default()
        };
        let kdl = generate_keybindings_kdl(&settings);
        assert!(
            parse_document(&kdl).is_ok(),
            "generated KDL must reparse:\n{kdl}"
        );
    }

    #[test]
    fn test_quote_kdl_string() {
        assert_eq!(quote_kdl_string("simple"), "\"simple\"");
        assert_eq!(quote_kdl_string("with space"), "\"with space\"");
        assert_eq!(quote_kdl_string("with\"quote"), "\"with\\\"quote\"");
        assert_eq!(quote_kdl_string("path\\to\\file"), "\"path\\\\to\\\\file\"");
    }
}
