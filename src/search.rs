//! Search functionality for settings
//!
//! Indexes individual settings with human-readable labels for intuitive search.
//! Users can search for things like "shadow", "border", "speed" and find the actual
//! setting they're looking for.

use crate::messages::Page;

/// Search result pointing to a specific setting
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// The page containing this setting
    pub page: Page,
    /// Human-readable setting name (e.g., "Enable Focus Ring")
    pub setting_name: String,
    /// Brief description of what the setting does
    pub description: String,
    /// Relevance score for sorting
    pub relevance_score: u32,
}

/// A searchable setting entry
struct SettingEntry {
    page: Page,
    setting_name: &'static str,
    description: &'static str,
    /// Lowercase versions for faster matching
    name_lower: String,
    desc_lower: String,
    /// Additional search keywords
    keywords: &'static [&'static str],
}

impl SettingEntry {
    fn new(
        page: Page,
        setting_name: &'static str,
        description: &'static str,
        keywords: &'static [&'static str],
    ) -> Self {
        Self {
            page,
            setting_name,
            description,
            name_lower: setting_name.to_lowercase(),
            desc_lower: description.to_lowercase(),
            keywords,
        }
    }
}

/// Search index containing all searchable settings
pub struct SearchIndex {
    entries: Vec<SettingEntry>,
}

impl SearchIndex {
    /// Creates a new search index with all settings
    pub fn new() -> Self {
        Self {
            entries: build_settings_index(),
        }
    }

    /// Searches for settings matching the query
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let mut score = 0u32;

                // Check setting name (highest priority)
                if entry.name_lower.contains(&query_lower) {
                    score += 100;
                } else {
                    // Check individual terms in name
                    for term in &query_terms {
                        if entry.name_lower.contains(term) {
                            score += 40;
                        }
                    }
                }

                // Check description
                for term in &query_terms {
                    if entry.desc_lower.contains(term) {
                        score += 20;
                    }
                }

                // Check keywords
                for keyword in entry.keywords {
                    for term in &query_terms {
                        if keyword.contains(term) {
                            score += if *keyword == *term { 30 } else { 15 };
                        }
                    }
                }

                if score > 0 {
                    Some(SearchResult {
                        page: entry.page,
                        setting_name: entry.setting_name.to_string(),
                        description: entry.description.to_string(),
                        relevance_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by relevance (highest first)
        results.sort_by(|a, b| b.relevance_score.cmp(&a.relevance_score));

        // Limit results
        results.truncate(12);

        results
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the complete settings index
fn build_settings_index() -> Vec<SettingEntry> {
    vec![
        // ═══════════════════════════════════════════════════════════════════
        // APPEARANCE
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Appearance,
            "Enable Focus Ring",
            "Show a colored ring around the focused window",
            &["focus", "ring", "border", "highlight", "active", "window"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Focus Ring Color",
            "Color of the ring around focused windows",
            &["focus", "ring", "color", "active", "highlight"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Focus Ring Width",
            "Thickness of the focus ring in pixels",
            &["focus", "ring", "width", "thickness", "size", "border"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Inactive Window Border",
            "Border color for unfocused windows",
            &["inactive", "border", "unfocused", "color", "window"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Window Gaps",
            "Space between windows",
            &["gaps", "spacing", "margin", "windows", "space", "between"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Corner Radius",
            "Rounded corners on windows",
            &["corner", "radius", "rounded", "curve", "windows"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Background Color",
            "Color behind windows and workspaces",
            &["background", "color", "wallpaper", "desktop"],
        ),
        SettingEntry::new(
            Page::Appearance,
            "Border Width",
            "Thickness of window borders",
            &["border", "width", "thickness", "outline"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // LAYOUT EXTRAS (shadows, etc.)
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::LayoutExtras,
            "Enable Window Shadow",
            "Show shadow behind windows",
            &["shadow", "drop", "window", "effect"],
        ),
        SettingEntry::new(
            Page::LayoutExtras,
            "Shadow Softness",
            "Blur amount for window shadows",
            &["shadow", "softness", "blur", "soft"],
        ),
        SettingEntry::new(
            Page::LayoutExtras,
            "Shadow Color",
            "Color of window shadows",
            &["shadow", "color"],
        ),
        SettingEntry::new(
            Page::LayoutExtras,
            "Shadow Offset",
            "Position offset of window shadows",
            &["shadow", "offset", "position", "x", "y"],
        ),
        SettingEntry::new(
            Page::LayoutExtras,
            "Center Single Column",
            "Center windows when only one column exists",
            &["center", "single", "column", "window", "middle"],
        ),
        SettingEntry::new(
            Page::LayoutExtras,
            "Default Column Width",
            "Default width for new columns",
            &["column", "width", "default", "size"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // BEHAVIOR
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Behavior,
            "Focus Follows Mouse",
            "Window focus follows the mouse cursor",
            &["focus", "mouse", "cursor", "hover", "follow"],
        ),
        SettingEntry::new(
            Page::Behavior,
            "Warp Mouse on Focus",
            "Move cursor to focused window",
            &["warp", "mouse", "cursor", "focus", "move", "teleport"],
        ),
        SettingEntry::new(
            Page::Behavior,
            "Workspace Auto Back-and-Forth",
            "Switching to current workspace goes to previous",
            &["workspace", "back", "forth", "toggle", "previous", "auto"],
        ),
        SettingEntry::new(
            Page::Behavior,
            "Modifier Key",
            "Key used for window management (Super, Alt, etc.)",
            &["modifier", "mod", "key", "super", "alt", "ctrl", "meta"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // KEYBOARD
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Keyboard,
            "Keyboard Layout",
            "XKB keyboard layout (e.g., us, de, fr)",
            &["keyboard", "layout", "xkb", "language", "qwerty", "azerty"],
        ),
        SettingEntry::new(
            Page::Keyboard,
            "Repeat Rate",
            "How fast keys repeat when held",
            &["repeat", "rate", "speed", "key", "hold"],
        ),
        SettingEntry::new(
            Page::Keyboard,
            "Repeat Delay",
            "Delay before key repeat starts",
            &["repeat", "delay", "wait", "key", "hold"],
        ),
        SettingEntry::new(
            Page::Keyboard,
            "Caps Lock Behavior",
            "What Caps Lock does (e.g., swap with Ctrl)",
            &["caps", "lock", "ctrl", "escape", "swap", "remap"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // MOUSE
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Mouse,
            "Mouse Acceleration",
            "How mouse speed scales with movement",
            &["mouse", "acceleration", "accel", "speed", "sensitivity"],
        ),
        SettingEntry::new(
            Page::Mouse,
            "Mouse Speed",
            "Base speed multiplier for mouse movement",
            &["mouse", "speed", "sensitivity", "fast", "slow"],
        ),
        SettingEntry::new(
            Page::Mouse,
            "Natural Scrolling (Mouse)",
            "Reverse scroll direction",
            &["natural", "scroll", "reverse", "direction", "mouse"],
        ),
        SettingEntry::new(
            Page::Mouse,
            "Left-Handed Mouse",
            "Swap left and right mouse buttons",
            &["left", "handed", "swap", "buttons", "mouse"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // TOUCHPAD
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Touchpad,
            "Tap to Click",
            "Tap the touchpad to click",
            &["tap", "click", "touchpad", "finger"],
        ),
        SettingEntry::new(
            Page::Touchpad,
            "Natural Scrolling (Touchpad)",
            "Reverse scroll direction on touchpad",
            &["natural", "scroll", "reverse", "touchpad"],
        ),
        SettingEntry::new(
            Page::Touchpad,
            "Two-Finger Scroll",
            "Scroll using two fingers on touchpad",
            &["two", "finger", "scroll", "touchpad"],
        ),
        SettingEntry::new(
            Page::Touchpad,
            "Disable While Typing",
            "Disable touchpad while using keyboard",
            &["disable", "typing", "dwt", "touchpad", "palm"],
        ),
        SettingEntry::new(
            Page::Touchpad,
            "Touchpad Speed",
            "Cursor speed when using touchpad",
            &["touchpad", "speed", "sensitivity", "acceleration"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // CURSOR
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Cursor,
            "Cursor Theme",
            "Visual theme for the mouse cursor",
            &["cursor", "theme", "pointer", "icon", "style"],
        ),
        SettingEntry::new(
            Page::Cursor,
            "Cursor Size",
            "Size of the mouse cursor",
            &["cursor", "size", "big", "small", "scale"],
        ),
        SettingEntry::new(
            Page::Cursor,
            "Hide Cursor When Inactive",
            "Hide cursor after period of inactivity",
            &["hide", "cursor", "inactive", "timeout", "disappear"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // ANIMATIONS
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Animations,
            "Enable Animations",
            "Turn animations on or off globally",
            &["animations", "enable", "disable", "motion", "effects"],
        ),
        SettingEntry::new(
            Page::Animations,
            "Animation Speed",
            "How fast animations play",
            &["animation", "speed", "duration", "fast", "slow"],
        ),
        SettingEntry::new(
            Page::Animations,
            "Window Open Animation",
            "Animation when windows open",
            &["window", "open", "animation", "appear", "spawn"],
        ),
        SettingEntry::new(
            Page::Animations,
            "Window Close Animation",
            "Animation when windows close",
            &["window", "close", "animation", "disappear", "exit"],
        ),
        SettingEntry::new(
            Page::Animations,
            "Workspace Switch Animation",
            "Animation when switching workspaces",
            &["workspace", "switch", "animation", "transition"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // WORKSPACES
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Workspaces,
            "Named Workspaces",
            "Create workspaces with custom names",
            &["workspace", "name", "named", "label", "create"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // WINDOW RULES
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::WindowRules,
            "Window Rules",
            "Create rules for specific applications",
            &["window", "rules", "app", "application", "match"],
        ),
        SettingEntry::new(
            Page::WindowRules,
            "Open on Workspace",
            "Open specific apps on designated workspaces",
            &["open", "workspace", "app", "application", "assign"],
        ),
        SettingEntry::new(
            Page::WindowRules,
            "Default Window Size",
            "Set default size for specific apps",
            &["window", "size", "default", "width", "height", "app"],
        ),
        SettingEntry::new(
            Page::WindowRules,
            "Force Floating",
            "Make specific windows always float",
            &["floating", "float", "window", "popup", "dialog"],
        ),
        SettingEntry::new(
            Page::WindowRules,
            "Window Opacity",
            "Transparency for specific windows",
            &["opacity", "transparent", "alpha", "window"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // LAYER RULES
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::LayerRules,
            "Layer Rules",
            "Rules for panels, bars, and overlays",
            &["layer", "rules", "panel", "bar", "waybar", "overlay"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // KEYBINDINGS
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Keybindings,
            "Keyboard Shortcuts",
            "Configure keybindings for actions",
            &["keyboard", "shortcuts", "keybindings", "hotkeys", "keys"],
        ),
        SettingEntry::new(
            Page::Keybindings,
            "Close Window Shortcut",
            "Keyboard shortcut to close windows",
            &["close", "window", "shortcut", "quit", "kill"],
        ),
        SettingEntry::new(
            Page::Keybindings,
            "Terminal Shortcut",
            "Keyboard shortcut to open terminal",
            &["terminal", "shortcut", "spawn", "launch", "open"],
        ),
        SettingEntry::new(
            Page::Keybindings,
            "Screenshot Shortcut",
            "Keyboard shortcut for screenshots",
            &["screenshot", "shortcut", "capture", "screen", "print"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // OUTPUTS
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Outputs,
            "Monitor Configuration",
            "Configure display resolution and position",
            &["monitor", "display", "screen", "output", "resolution"],
        ),
        SettingEntry::new(
            Page::Outputs,
            "Display Scale",
            "HiDPI scaling factor for monitors",
            &["scale", "hidpi", "dpi", "zoom", "display"],
        ),
        SettingEntry::new(
            Page::Outputs,
            "Refresh Rate",
            "Monitor refresh rate (Hz)",
            &["refresh", "rate", "hz", "hertz", "monitor"],
        ),
        SettingEntry::new(
            Page::Outputs,
            "Variable Refresh Rate",
            "VRR/Adaptive sync for monitors",
            &["vrr", "variable", "refresh", "adaptive", "sync", "freesync"],
        ),
        SettingEntry::new(
            Page::Outputs,
            "Monitor Rotation",
            "Rotate display orientation",
            &["rotate", "rotation", "orientation", "portrait", "landscape"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // STARTUP
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Startup,
            "Startup Applications",
            "Programs to launch when niri starts",
            &["startup", "autostart", "launch", "boot", "programs"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // ENVIRONMENT
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Environment,
            "Environment Variables",
            "Set environment variables for niri session",
            &["environment", "variables", "env", "export", "path"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // DEBUG
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Debug,
            "Show FPS Counter",
            "Display frames per second overlay",
            &["fps", "frames", "performance", "debug", "counter"],
        ),
        SettingEntry::new(
            Page::Debug,
            "Render Damage Tracking",
            "Visualize screen redraw regions",
            &["damage", "render", "debug", "redraw"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // MISCELLANEOUS
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Miscellaneous,
            "Screenshot Directory",
            "Where screenshots are saved",
            &["screenshot", "directory", "folder", "path", "save"],
        ),
        SettingEntry::new(
            Page::Miscellaneous,
            "Prefer Server-Side Decorations",
            "Use compositor window decorations",
            &["decoration", "csd", "ssd", "titlebar", "server"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // GESTURES
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::Gestures,
            "Touchpad Gestures",
            "Configure swipe and pinch gestures",
            &["gesture", "swipe", "pinch", "touchpad", "fingers"],
        ),
        SettingEntry::new(
            Page::Gestures,
            "Workspace Swipe Gesture",
            "Swipe to switch workspaces",
            &["swipe", "workspace", "gesture", "switch"],
        ),

        // ═══════════════════════════════════════════════════════════════════
        // SWITCH EVENTS
        // ═══════════════════════════════════════════════════════════════════
        SettingEntry::new(
            Page::SwitchEvents,
            "Lid Close Action",
            "What happens when laptop lid closes",
            &["lid", "close", "laptop", "suspend", "sleep", "lock"],
        ),
        SettingEntry::new(
            Page::SwitchEvents,
            "Tablet Mode",
            "Behavior when device enters tablet mode",
            &["tablet", "mode", "convertible", "touch"],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_shadow() {
        let index = SearchIndex::new();
        let results = index.search("shadow");
        assert!(!results.is_empty());
        assert!(results[0].setting_name.to_lowercase().contains("shadow"));
    }

    #[test]
    fn test_search_focus() {
        let index = SearchIndex::new();
        let results = index.search("focus");
        assert!(!results.is_empty());
        // Should find focus ring settings
        assert!(results.iter().any(|r| r.setting_name.contains("Focus")));
    }

    #[test]
    fn test_search_speed() {
        let index = SearchIndex::new();
        let results = index.search("speed");
        assert!(!results.is_empty());
        // Should find mouse/touchpad/animation speed settings
    }

    #[test]
    fn test_search_empty() {
        let index = SearchIndex::new();
        let results = index.search("");
        assert!(results.is_empty());
    }
}
