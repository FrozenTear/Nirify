//! Recent windows switcher settings (v25.05+)

use crate::types::Color;

/// Scope for recent windows filter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecentWindowsScope {
    /// All windows across all outputs and workspaces
    #[default]
    All,
    /// Windows on the current output
    Output,
    /// Windows on the current workspace
    Workspace,
}

impl RecentWindowsScope {
    /// Convert to KDL string
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Output => "output",
            Self::Workspace => "workspace",
        }
    }

    /// Parse from KDL string
    pub fn from_kdl(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "output" => Some(Self::Output),
            "workspace" => Some(Self::Workspace),
            _ => None,
        }
    }
}

/// A keybind for the recent windows switcher
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsBind {
    /// Key combination (e.g., "Alt+Tab")
    pub key_combo: String,
    /// Whether this is next-window (true) or previous-window (false)
    pub is_next: bool,
    /// Filter to current app's windows
    pub filter_app_id: bool,
    /// Scope for the switcher
    pub scope: Option<RecentWindowsScope>,
    /// Cooldown in milliseconds
    pub cooldown_ms: Option<i32>,
}

impl Default for RecentWindowsBind {
    fn default() -> Self {
        Self {
            key_combo: String::from("Alt+Tab"),
            is_next: true,
            filter_app_id: false,
            scope: None,
            cooldown_ms: None,
        }
    }
}

/// Highlight style settings for recent windows switcher
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsHighlight {
    /// Active window highlight color
    pub active_color: Color,
    /// Urgent window highlight color
    pub urgent_color: Color,
    /// Highlight padding in logical pixels
    pub padding: i32,
    /// Corner radius for highlight
    pub corner_radius: i32,
}

impl Default for RecentWindowsHighlight {
    fn default() -> Self {
        Self {
            // Niri defaults from docs
            active_color: Color::from_hex("#999999ff").unwrap_or_default(),
            urgent_color: Color::from_hex("#ff9999ff").unwrap_or_default(),
            padding: 30,
            corner_radius: 0,
        }
    }
}

/// Preview settings for recent windows switcher
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsPreviews {
    /// Maximum height of previews in logical pixels
    pub max_height: i32,
    /// Maximum scale factor for previews (0.0-1.0)
    pub max_scale: f64,
}

impl Default for RecentWindowsPreviews {
    fn default() -> Self {
        Self {
            // Niri defaults from docs
            max_height: 480,
            max_scale: 0.5,
        }
    }
}

/// Recent windows switcher settings (v25.05+)
///
/// Configures the recent windows (Alt-Tab) switcher appearance and behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct RecentWindowsSettings {
    /// Whether the recent windows switcher is disabled
    pub off: bool,
    /// Delay before the window is committed to the recent list (ms)
    pub debounce_ms: i32,
    /// Delay before the switcher UI appears (ms)
    pub open_delay_ms: i32,
    /// Highlight styling
    pub highlight: RecentWindowsHighlight,
    /// Preview settings
    pub previews: RecentWindowsPreviews,
    /// Custom keybinds for the recent windows switcher
    pub binds: Vec<RecentWindowsBind>,
}

impl Default for RecentWindowsSettings {
    fn default() -> Self {
        Self {
            off: false,
            // Niri defaults from docs
            debounce_ms: 750,
            open_delay_ms: 150,
            highlight: RecentWindowsHighlight::default(),
            previews: RecentWindowsPreviews::default(),
            binds: vec![],
        }
    }
}
