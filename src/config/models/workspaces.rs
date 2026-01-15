//! Named workspaces settings

use super::layout::LayoutOverride;

/// A named workspace configuration (v0.1.6+)
#[derive(Debug, Clone, PartialEq)]
pub struct NamedWorkspace {
    /// Unique identifier for UI management
    pub id: u32,
    /// Workspace name (used for identification and display)
    pub name: String,
    /// Pin to specific output (monitor name or serial)
    pub open_on_output: Option<String>,
    /// Per-workspace layout overrides (v25.11+)
    pub layout_override: Option<LayoutOverride>,
}

impl Default for NamedWorkspace {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("New Workspace"),
            open_on_output: None,
            layout_override: None,
        }
    }
}

/// Named workspaces settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspacesSettings {
    pub workspaces: Vec<NamedWorkspace>,
    /// Counter for generating unique IDs
    pub next_id: u32,
}
