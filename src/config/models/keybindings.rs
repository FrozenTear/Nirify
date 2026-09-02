//! Keybindings settings - keyboard shortcuts and their actions

/// A single value that can appear as a positional argument or property value
/// on a niri action node. Captured losslessly from the KDL so it can be
/// re-emitted with the original type.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Null,
}

impl ActionValue {
    /// Render this value as KDL text (bare bool/int/float/null, quoted string).
    pub fn to_kdl(&self) -> String {
        match self {
            ActionValue::Bool(b) => b.to_string(),
            ActionValue::Int(i) => i.to_string(),
            ActionValue::Float(f) => f.to_string(),
            ActionValue::Str(s) => quote_kdl_string(s),
            ActionValue::Null => "null".to_string(),
        }
    }

    /// Best-effort display string (unquoted) for UI text inputs.
    pub fn as_display(&self) -> String {
        match self {
            ActionValue::Bool(b) => b.to_string(),
            ActionValue::Int(i) => i.to_string(),
            ActionValue::Float(f) => f.to_string(),
            ActionValue::Str(s) => s.clone(),
            ActionValue::Null => String::new(),
        }
    }
}

/// Quote a string for KDL format (escapes backslashes and quotes).
pub fn quote_kdl_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// A niri action node captured losslessly: a name plus ordered positional
/// arguments and named properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActionNode {
    /// kebab-case action name
    pub name: String,
    /// positional arguments, in order
    pub args: Vec<ActionValue>,
    /// named properties, in file order
    pub props: Vec<(String, ActionValue)>,
}

impl ActionNode {
    /// Construct a bare action node with just a name.
    pub fn bare(name: &str) -> Self {
        ActionNode {
            name: name.to_string(),
            args: Vec::new(),
            props: Vec::new(),
        }
    }

    /// Set (or clear) the single primary positional argument.
    pub fn set_primary_arg(&mut self, value: Option<ActionValue>) {
        match value {
            Some(v) => {
                if self.args.is_empty() {
                    self.args.push(v);
                } else {
                    self.args[0] = v;
                }
            }
            None => {
                self.args.clear();
            }
        }
    }

    /// Get the primary positional argument, if any.
    pub fn primary_arg(&self) -> Option<&ActionValue> {
        self.args.first()
    }

    /// Display string of the primary argument for text inputs.
    pub fn primary_arg_display(&self) -> String {
        self.args
            .first()
            .map(|v| v.as_display())
            .unwrap_or_default()
    }

    /// Get a named property value.
    pub fn get_prop(&self, name: &str) -> Option<&ActionValue> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Set or remove a named property.
    pub fn set_prop(&mut self, name: &str, value: Option<ActionValue>) {
        self.props.retain(|(n, _)| n != name);
        if let Some(v) = value {
            self.props.push((name.to_string(), v));
        }
    }
}

/// Action type for a keybinding
#[derive(Debug, Clone, PartialEq)]
pub enum KeybindAction {
    /// Spawn a command with arguments (`spawn "cmd" "arg"…`)
    Spawn(Vec<String>),
    /// Spawn a shell command (`spawn-sh "shell command"`)
    SpawnSh(String),
    /// Built-in niri action, captured losslessly
    NiriAction(ActionNode),
    /// Raw single-node KDL text typed by the user (advanced)
    Custom(String),
}

impl Default for KeybindAction {
    fn default() -> Self {
        Self::NiriAction(ActionNode::bare("close-window"))
    }
}

impl KeybindAction {
    /// The action name (first token) if determinable.
    pub fn name(&self) -> String {
        match self {
            KeybindAction::Spawn(_) => "spawn".to_string(),
            KeybindAction::SpawnSh(_) => "spawn-sh".to_string(),
            KeybindAction::NiriAction(node) => node.name.clone(),
            KeybindAction::Custom(raw) => raw
                .split_whitespace()
                .next()
                .unwrap_or("custom")
                .to_string(),
        }
    }

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
            KeybindAction::SpawnSh(cmd) => {
                if cmd.trim().is_empty() {
                    "spawn-sh (empty)".to_string()
                } else {
                    format!("spawn-sh {}", cmd)
                }
            }
            KeybindAction::NiriAction(node) => {
                let mut s = node.name.clone();
                for a in &node.args {
                    s.push(' ');
                    s.push_str(&a.as_display());
                }
                s
            }
            KeybindAction::Custom(raw) => raw.trim().to_string(),
        }
    }
}

/// Whether/how a binding appears in niri's hotkey overlay.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HotkeyOverlayTitle {
    /// Absent property — niri auto-generates a title.
    #[default]
    Auto,
    /// `hotkey-overlay-title=null` — hidden from the overlay.
    Hidden,
    /// Custom title string.
    Custom(String),
}

/// A single keybinding entry
#[derive(Debug, Clone, PartialEq)]
pub struct Keybinding {
    /// Unique identifier for UI
    pub id: u32,
    /// Key combination (e.g., "Mod+Space", "XF86AudioMute")
    pub key_combo: String,
    /// How the binding appears in niri's hotkey overlay
    pub hotkey_overlay_title: HotkeyOverlayTitle,
    /// Whether binding works when screen is locked (spawn-only in niri)
    pub allow_when_locked: bool,
    /// Whether apps may inhibit this shortcut (niri default true)
    pub allow_inhibiting: bool,
    /// Cooldown in milliseconds between activations
    pub cooldown_ms: Option<i32>,
    /// Whether the binding repeats when held (niri default true)
    pub repeat: bool,
    /// The action to perform
    pub action: KeybindAction,
}

impl Default for Keybinding {
    fn default() -> Self {
        Keybinding {
            id: 0,
            key_combo: String::new(),
            hotkey_overlay_title: HotkeyOverlayTitle::Auto,
            allow_when_locked: false,
            allow_inhibiting: true,
            cooldown_ms: None,
            repeat: true,
            action: KeybindAction::default(),
        }
    }
}

impl Keybinding {
    /// Get the display name (overlay title or action description)
    pub fn display_name(&self) -> String {
        match &self.hotkey_overlay_title {
            HotkeyOverlayTitle::Custom(s) => s.clone(),
            _ => self.action.description(),
        }
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

/// Keep the last binding for each normalized combo (niri last-wins).
///
/// niri merges `binds` so a later entry for the same key combination
/// overrides earlier ones ([Include § Binds](https://niri-wm.github.io/niri/Configuration%3A-Include.html#binds)).
/// Nirify used to keep the first and drop later duplicates; that disagreed
/// with niri and could persist the wrong action after first-run / absorb.
///
/// Surviving bindings stay in their original relative order (the last
/// occurrence of each combo is kept).
#[must_use]
pub fn last_wins_keybindings(bindings: Vec<Keybinding>) -> Vec<Keybinding> {
    use std::collections::HashMap;

    let mut last_index: HashMap<String, usize> = HashMap::new();
    for (i, binding) in bindings.iter().enumerate() {
        last_index.insert(normalized_key_combo(&binding.key_combo), i);
    }
    bindings
        .into_iter()
        .enumerate()
        .filter(|(i, binding)| last_index.get(&normalized_key_combo(&binding.key_combo)) == Some(i))
        .map(|(_, binding)| binding)
        .collect()
}

/// Normalize a key-combo string for duplicate detection.
///
/// Splits on `+`, canonicalizes modifier aliases case-insensitively
/// (`control`→`ctrl`, `win`→`super`, `mod5`→`iso_level3_shift`,
/// `mod3`→`iso_level5_shift`), sorts modifiers, lowercases the trigger.
/// `Mod+Q` == `mod+q`; `Mod+Q` != `Super+Q`.
pub fn normalized_key_combo(combo: &str) -> String {
    let parts: Vec<&str> = combo
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return String::new();
    }
    let (trigger, mods) = parts.split_last().unwrap();
    let mut norm_mods: Vec<String> = mods
        .iter()
        .map(|m| {
            let lower = m.to_lowercase();
            match lower.as_str() {
                "control" | "ctrl" => "ctrl".to_string(),
                "win" | "super" => "super".to_string(),
                "mod5" => "iso_level3_shift".to_string(),
                "mod3" => "iso_level5_shift".to_string(),
                other => other.to_string(),
            }
        })
        .collect();
    norm_mods.sort();
    norm_mods.dedup();
    norm_mods.push(trigger.to_lowercase());
    norm_mods.join("+")
}

// ── Action catalog ─────────────────────────────────────────────────────────

/// Which screenshot action a `ScreenshotFlags` arg kind applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotKind {
    /// `screenshot` (interactive region) — show-pointer only
    Region,
    /// `screenshot-screen` — write-to-disk + show-pointer
    Screen,
    /// `screenshot-window` — write-to-disk + show-pointer (26.04+)
    Window,
}

/// The argument shape of a catalog action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    None,
    /// 1 required positional QUOTED string, validated `^[+-]?\d+(\.\d+)?%?$`
    SizeChange,
    /// 1 required positional int ≥ 1 (unquoted)
    IndexInt,
    /// 1 required positional: int if all-digits else quoted string
    WorkspaceRef,
    /// WorkspaceRef + optional focus=false prop
    WorkspaceRefFocus,
    /// no positional, optional focus=false prop
    FocusFlag,
    /// 1 required positional quoted string
    OutputName,
    /// 0..1 positional quoted string
    OptionalOutputName,
    /// 1 required positional quoted string
    NameString,
    /// pick_list normal|tabbed → quoted string
    ColumnDisplay,
    /// pick_list next|prev|index → quoted string
    LayoutTarget,
    /// optional delay-ms=<u16> prop
    DelayMs,
    /// optional skip-confirmation=true prop
    QuitFlags,
    /// per-action bool props (write-to-disk / show-pointer)
    ScreenshotFlags(ScreenshotKind),
    /// existing command editor → Vec<String>
    SpawnCmd,
    /// single-line shell string
    SpawnShCmd,
}

impl ArgKind {
    /// Whether this action requires a non-empty primary positional argument.
    pub fn requires_primary_arg(&self) -> bool {
        matches!(
            self,
            ArgKind::SizeChange
                | ArgKind::IndexInt
                | ArgKind::WorkspaceRef
                | ArgKind::WorkspaceRefFocus
                | ArgKind::OutputName
                | ArgKind::NameString
                | ArgKind::ColumnDisplay
                | ArgKind::LayoutTarget
        )
    }
}

/// Top-level grouping of actions for the two-level action picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    System,
    Run,
    Screenshot,
    Window,
    Focus,
    Move,
    ColumnDisplay,
    Workspace,
    Monitor,
    Sizing,
    Layout,
    Overview,
    Screencast,
    Custom,
}

impl ActionCategory {
    pub const ALL: &'static [ActionCategory] = &[
        ActionCategory::System,
        ActionCategory::Run,
        ActionCategory::Screenshot,
        ActionCategory::Window,
        ActionCategory::Focus,
        ActionCategory::Move,
        ActionCategory::ColumnDisplay,
        ActionCategory::Workspace,
        ActionCategory::Monitor,
        ActionCategory::Sizing,
        ActionCategory::Layout,
        ActionCategory::Overview,
        ActionCategory::Screencast,
        ActionCategory::Custom,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ActionCategory::System => "Session / System",
            ActionCategory::Run => "Run Command",
            ActionCategory::Screenshot => "Screenshot",
            ActionCategory::Window => "Window",
            ActionCategory::Focus => "Focus",
            ActionCategory::Move => "Move",
            ActionCategory::ColumnDisplay => "Column Display",
            ActionCategory::Workspace => "Workspace",
            ActionCategory::Monitor => "Monitor",
            ActionCategory::Sizing => "Sizing",
            ActionCategory::Layout => "Layout / Floating",
            ActionCategory::Overview => "Overview",
            ActionCategory::Screencast => "Screencast",
            ActionCategory::Custom => "Custom action…",
        }
    }
}

impl std::fmt::Display for ActionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A catalog entry describing one niri action.
#[derive(Debug, Clone, Copy)]
pub struct ActionSpec {
    pub name: &'static str,
    pub category: ActionCategory,
    pub args: ArgKind,
}

impl ActionSpec {
    /// Humanized label derived from the kebab-case name.
    pub fn label(&self) -> String {
        humanize_action(self.name)
    }
}

/// Turn a kebab-case action name into a human-readable label.
pub fn humanize_action(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (i, word) in name.split('-').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

use ActionCategory as C;
use ArgKind as A;
use ScreenshotKind as S;

/// The full set of knuffel-decodable niri actions (identical in v25.11 and
/// v26.04). 135 entries.
pub const CATALOG: &[ActionSpec] = &[
    // Session / system
    spec("quit", C::System, A::QuitFlags),
    spec("suspend", C::System, A::None),
    spec("power-off-monitors", C::System, A::None),
    spec("power-on-monitors", C::System, A::None),
    spec("show-hotkey-overlay", C::System, A::None),
    spec("toggle-keyboard-shortcuts-inhibit", C::System, A::None),
    spec("do-screen-transition", C::System, A::DelayMs),
    spec("toggle-debug-tint", C::System, A::None),
    spec("debug-toggle-opaque-regions", C::System, A::None),
    spec("debug-toggle-damage", C::System, A::None),
    // Run
    spec("spawn", C::Run, A::SpawnCmd),
    spec("spawn-sh", C::Run, A::SpawnShCmd),
    // Screenshot
    spec("screenshot", C::Screenshot, A::ScreenshotFlags(S::Region)),
    spec(
        "screenshot-screen",
        C::Screenshot,
        A::ScreenshotFlags(S::Screen),
    ),
    spec(
        "screenshot-window",
        C::Screenshot,
        A::ScreenshotFlags(S::Window),
    ),
    // Window
    spec("close-window", C::Window, A::None),
    spec("fullscreen-window", C::Window, A::None),
    spec("toggle-windowed-fullscreen", C::Window, A::None),
    spec("maximize-window-to-edges", C::Window, A::None),
    spec("center-window", C::Window, A::None),
    spec("focus-window-previous", C::Window, A::None),
    spec("focus-window-in-column", C::Window, A::IndexInt),
    spec("toggle-window-rule-opacity", C::Window, A::None),
    // Focus: column/window
    spec("focus-column-left", C::Focus, A::None),
    spec("focus-column-right", C::Focus, A::None),
    spec("focus-column-first", C::Focus, A::None),
    spec("focus-column-last", C::Focus, A::None),
    spec("focus-column-right-or-first", C::Focus, A::None),
    spec("focus-column-left-or-last", C::Focus, A::None),
    spec("focus-column", C::Focus, A::IndexInt),
    spec("focus-window-down", C::Focus, A::None),
    spec("focus-window-up", C::Focus, A::None),
    spec("focus-window-or-workspace-down", C::Focus, A::None),
    spec("focus-window-or-workspace-up", C::Focus, A::None),
    spec("focus-window-top", C::Focus, A::None),
    spec("focus-window-bottom", C::Focus, A::None),
    spec("focus-window-down-or-top", C::Focus, A::None),
    spec("focus-window-up-or-bottom", C::Focus, A::None),
    spec("focus-window-down-or-column-left", C::Focus, A::None),
    spec("focus-window-down-or-column-right", C::Focus, A::None),
    spec("focus-window-up-or-column-left", C::Focus, A::None),
    spec("focus-window-up-or-column-right", C::Focus, A::None),
    spec("focus-window-or-monitor-up", C::Focus, A::None),
    spec("focus-window-or-monitor-down", C::Focus, A::None),
    spec("focus-column-or-monitor-left", C::Focus, A::None),
    spec("focus-column-or-monitor-right", C::Focus, A::None),
    // Move: column/window
    spec("move-column-left", C::Move, A::None),
    spec("move-column-right", C::Move, A::None),
    spec("move-column-to-first", C::Move, A::None),
    spec("move-column-to-last", C::Move, A::None),
    spec("move-column-left-or-to-monitor-left", C::Move, A::None),
    spec("move-column-right-or-to-monitor-right", C::Move, A::None),
    spec("move-column-to-index", C::Move, A::IndexInt),
    spec("move-window-down", C::Move, A::None),
    spec("move-window-up", C::Move, A::None),
    spec("move-window-down-or-to-workspace-down", C::Move, A::None),
    spec("move-window-up-or-to-workspace-up", C::Move, A::None),
    spec("consume-or-expel-window-left", C::Move, A::None),
    spec("consume-or-expel-window-right", C::Move, A::None),
    spec("consume-window-into-column", C::Move, A::None),
    spec("expel-window-from-column", C::Move, A::None),
    spec("swap-window-left", C::Move, A::None),
    spec("swap-window-right", C::Move, A::None),
    spec("center-column", C::Move, A::None),
    spec("center-visible-columns", C::Move, A::None),
    // Column display
    spec("toggle-column-tabbed-display", C::ColumnDisplay, A::None),
    spec("set-column-display", C::ColumnDisplay, A::ColumnDisplay),
    // Workspace
    spec("focus-workspace-down", C::Workspace, A::None),
    spec("focus-workspace-up", C::Workspace, A::None),
    spec("focus-workspace", C::Workspace, A::WorkspaceRef),
    spec("focus-workspace-previous", C::Workspace, A::None),
    spec("move-window-to-workspace-down", C::Workspace, A::FocusFlag),
    spec("move-window-to-workspace-up", C::Workspace, A::FocusFlag),
    spec(
        "move-window-to-workspace",
        C::Workspace,
        A::WorkspaceRefFocus,
    ),
    spec("move-column-to-workspace-down", C::Workspace, A::FocusFlag),
    spec("move-column-to-workspace-up", C::Workspace, A::FocusFlag),
    spec(
        "move-column-to-workspace",
        C::Workspace,
        A::WorkspaceRefFocus,
    ),
    spec("move-workspace-down", C::Workspace, A::None),
    spec("move-workspace-up", C::Workspace, A::None),
    spec("move-workspace-to-index", C::Workspace, A::IndexInt),
    spec("move-workspace-to-monitor", C::Workspace, A::OutputName),
    spec("set-workspace-name", C::Workspace, A::NameString),
    spec("unset-workspace-name", C::Workspace, A::None),
    // Monitor
    spec("focus-monitor-left", C::Monitor, A::None),
    spec("focus-monitor-right", C::Monitor, A::None),
    spec("focus-monitor-down", C::Monitor, A::None),
    spec("focus-monitor-up", C::Monitor, A::None),
    spec("focus-monitor-previous", C::Monitor, A::None),
    spec("focus-monitor-next", C::Monitor, A::None),
    spec("focus-monitor", C::Monitor, A::OutputName),
    spec("move-window-to-monitor-left", C::Monitor, A::None),
    spec("move-window-to-monitor-right", C::Monitor, A::None),
    spec("move-window-to-monitor-down", C::Monitor, A::None),
    spec("move-window-to-monitor-up", C::Monitor, A::None),
    spec("move-window-to-monitor-previous", C::Monitor, A::None),
    spec("move-window-to-monitor-next", C::Monitor, A::None),
    spec("move-window-to-monitor", C::Monitor, A::OutputName),
    spec("move-column-to-monitor-left", C::Monitor, A::None),
    spec("move-column-to-monitor-right", C::Monitor, A::None),
    spec("move-column-to-monitor-down", C::Monitor, A::None),
    spec("move-column-to-monitor-up", C::Monitor, A::None),
    spec("move-column-to-monitor-previous", C::Monitor, A::None),
    spec("move-column-to-monitor-next", C::Monitor, A::None),
    spec("move-column-to-monitor", C::Monitor, A::OutputName),
    spec("move-workspace-to-monitor-left", C::Monitor, A::None),
    spec("move-workspace-to-monitor-right", C::Monitor, A::None),
    spec("move-workspace-to-monitor-down", C::Monitor, A::None),
    spec("move-workspace-to-monitor-up", C::Monitor, A::None),
    spec("move-workspace-to-monitor-previous", C::Monitor, A::None),
    spec("move-workspace-to-monitor-next", C::Monitor, A::None),
    // Sizing
    spec("set-window-width", C::Sizing, A::SizeChange),
    spec("set-window-height", C::Sizing, A::SizeChange),
    spec("reset-window-height", C::Sizing, A::None),
    spec("switch-preset-column-width", C::Sizing, A::None),
    spec("switch-preset-column-width-back", C::Sizing, A::None),
    spec("switch-preset-window-width", C::Sizing, A::None),
    spec("switch-preset-window-width-back", C::Sizing, A::None),
    spec("switch-preset-window-height", C::Sizing, A::None),
    spec("switch-preset-window-height-back", C::Sizing, A::None),
    spec("maximize-column", C::Sizing, A::None),
    spec("set-column-width", C::Sizing, A::SizeChange),
    spec("expand-column-to-available-width", C::Sizing, A::None),
    // Layout / floating
    spec("switch-layout", C::Layout, A::LayoutTarget),
    spec("toggle-window-floating", C::Layout, A::None),
    spec("move-window-to-floating", C::Layout, A::None),
    spec("move-window-to-tiling", C::Layout, A::None),
    spec("focus-floating", C::Layout, A::None),
    spec("focus-tiling", C::Layout, A::None),
    spec(
        "switch-focus-between-floating-and-tiling",
        C::Layout,
        A::None,
    ),
    // Overview
    spec("toggle-overview", C::Overview, A::None),
    spec("open-overview", C::Overview, A::None),
    spec("close-overview", C::Overview, A::None),
    // Screencast
    spec("set-dynamic-cast-window", C::Screencast, A::None),
    spec(
        "set-dynamic-cast-monitor",
        C::Screencast,
        A::OptionalOutputName,
    ),
    spec("clear-dynamic-cast-target", C::Screencast, A::None),
];

/// const-fn helper for building catalog entries.
const fn spec(name: &'static str, category: ActionCategory, args: ArgKind) -> ActionSpec {
    ActionSpec {
        name,
        category,
        args,
    }
}

/// Look up a catalog entry by action name.
pub fn lookup_action(name: &str) -> Option<&'static ActionSpec> {
    CATALOG.iter().find(|s| s.name == name)
}

/// Actions belonging to a category, in catalog order.
pub fn actions_in_category(category: ActionCategory) -> Vec<&'static ActionSpec> {
    CATALOG.iter().filter(|s| s.category == category).collect()
}

/// Validate a SizeChange argument string: `^[+-]?\d+(\.\d+)?%?$`.
pub fn is_valid_size_change(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars().peekable();
    if matches!(chars.peek(), Some('+') | Some('-')) {
        chars.next();
    }
    let mut saw_int_digit = false;
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
        chars.next();
        saw_int_digit = true;
    }
    if !saw_int_digit {
        return false;
    }
    if matches!(chars.peek(), Some('.')) {
        chars.next();
        let mut saw_frac = false;
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            chars.next();
            saw_frac = true;
        }
        if !saw_frac {
            return false;
        }
    }
    if matches!(chars.peek(), Some('%')) {
        chars.next();
    }
    chars.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_case_alias_order() {
        assert_eq!(normalized_key_combo("Mod+Q"), normalized_key_combo("mod+q"));
        assert_eq!(
            normalized_key_combo("Control+A"),
            normalized_key_combo("Ctrl+A")
        );
        assert_eq!(
            normalized_key_combo("Win+X"),
            normalized_key_combo("Super+X")
        );
        assert_eq!(
            normalized_key_combo("Shift+Mod+F"),
            normalized_key_combo("Mod+Shift+F")
        );
    }

    #[test]
    fn last_wins_keeps_later_mod_q() {
        let first = Keybinding {
            id: 0,
            key_combo: "Mod+Q".to_string(),
            action: KeybindAction::NiriAction(ActionNode::bare("close-window")),
            ..Default::default()
        };
        let last = Keybinding {
            id: 1,
            key_combo: "mod+q".to_string(),
            action: KeybindAction::NiriAction(ActionNode::bare("spawn")),
            ..Default::default()
        };
        let kept = last_wins_keybindings(vec![first, last.clone()]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, 1);
        assert_eq!(
            kept[0].action,
            KeybindAction::NiriAction(ActionNode::bare("spawn"))
        );
    }

    #[test]
    fn test_normalized_mod_not_super() {
        assert_ne!(
            normalized_key_combo("Mod+Q"),
            normalized_key_combo("Super+Q")
        );
    }

    #[test]
    fn test_catalog_has_135_actions() {
        assert_eq!(CATALOG.len(), 135);
    }

    #[test]
    fn test_size_change_validation() {
        for good in ["+10%", "-50", "500", "50%", "25.5%", "+0.5"] {
            assert!(is_valid_size_change(good), "{good} should be valid");
        }
        for bad in ["", "abc", "%", "+", "10%%", "1.%", "1.2.3"] {
            assert!(!is_valid_size_change(bad), "{bad} should be invalid");
        }
    }
}
