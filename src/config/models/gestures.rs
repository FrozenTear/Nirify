//! Gesture settings (hot corners, DnD edge triggers)

/// Hot corner positions
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HotCorners {
    pub enabled: bool,
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

/// DND edge trigger settings (shared between scroll and workspace switch)
///
/// The `trigger_size` field represents:
/// - `trigger_width` for edge view scroll
/// - `trigger_height` for workspace switch
#[derive(Debug, Clone, PartialEq)]
pub struct DndEdgeSettings {
    pub enabled: bool,
    /// Trigger zone size in pixels (width for scroll, height for workspace)
    pub trigger_size: i32,
    pub delay_ms: i32,
    pub max_speed: i32,
}

impl DndEdgeSettings {
    /// Create default settings for edge view scroll (trigger_size = 30)
    pub fn default_scroll() -> Self {
        Self {
            enabled: true,
            trigger_size: 30,
            delay_ms: 100,
            max_speed: 1500,
        }
    }

    /// Create default settings for workspace switch (trigger_size = 50)
    pub fn default_workspace() -> Self {
        Self {
            enabled: true,
            trigger_size: 50,
            delay_ms: 100,
            max_speed: 1500,
        }
    }
}

impl Default for DndEdgeSettings {
    fn default() -> Self {
        Self::default_scroll()
    }
}

/// Gesture settings
#[derive(Debug, Clone, PartialEq)]
pub struct GestureSettings {
    pub hot_corners: HotCorners,
    pub dnd_edge_view_scroll: DndEdgeSettings,
    pub dnd_edge_workspace_switch: DndEdgeSettings,
}

impl Default for GestureSettings {
    fn default() -> Self {
        Self {
            hot_corners: HotCorners::default(),
            dnd_edge_view_scroll: DndEdgeSettings::default_scroll(),
            dnd_edge_workspace_switch: DndEdgeSettings::default_workspace(),
        }
    }
}
