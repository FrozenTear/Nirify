//! Keybindings settings - keyboard shortcuts and their actions

/// Action type for a keybinding
#[derive(Debug, Clone, PartialEq)]
pub enum KeybindAction {
    /// Spawn a command with arguments
    Spawn(Vec<String>),
    /// Built-in niri action (e.g., "close-window", "toggle-overview")
    NiriAction(String),
    /// Action with arguments (e.g., "focus-workspace" with workspace name)
    NiriActionWithArgs(String, Vec<String>),
}

impl Default for KeybindAction {
    fn default() -> Self {
        Self::NiriAction(String::new())
    }
}

impl KeybindAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            KeybindAction::Spawn(args) => {
                if args.is_empty() {
                    "spawn (empty)".to_string()
                } else {
                    format!("spawn {}", args.join(" "))
                }
            }
            KeybindAction::NiriAction(action) => action.clone(),
            KeybindAction::NiriActionWithArgs(action, args) => {
                format!("{} {}", action, args.join(" "))
            }
        }
    }
}

/// A single keybinding entry
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keybinding {
    /// Unique identifier for UI
    pub id: u32,
    /// Key combination (e.g., "Mod+Space", "XF86AudioMute")
    pub key_combo: String,
    /// Optional title shown in niri's hotkey overlay
    pub hotkey_overlay_title: Option<String>,
    /// Whether binding works when screen is locked
    pub allow_when_locked: bool,
    /// Cooldown in milliseconds between activations
    pub cooldown_ms: Option<i32>,
    /// Whether the binding repeats when held
    pub repeat: bool,
    /// The action to perform
    pub action: KeybindAction,
}

impl Keybinding {
    /// Get the display name (overlay title or action description)
    pub fn display_name(&self) -> String {
        self.hotkey_overlay_title
            .clone()
            .unwrap_or_else(|| self.action.description())
    }
}

/// Keybindings settings - managed keyboard shortcuts
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeybindingsSettings {
    /// All keybindings found in user's config
    pub bindings: Vec<Keybinding>,
    /// Source file where bindings were loaded from
    pub source_file: Option<String>,
    /// Whether bindings were successfully loaded
    pub loaded: bool,
    /// Error message if loading failed
    pub error: Option<String>,
}
