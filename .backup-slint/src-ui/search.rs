//! Search functionality for settings
//!
//! Provides search across all settings labels and descriptions.

use log::debug;

/// A search result with category and match information
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Category index (0-11)
    pub category: i32,
    /// Unique setting identifier (e.g., "focus-ring-enabled")
    pub setting_id: String,
    /// Display name of the setting
    pub label: String,
    /// Description of the setting
    pub description: String,
    /// Relevance score (higher is better)
    pub score: i32,
}

/// Searchable setting definition
struct SearchableItem {
    category: i32,
    setting_id: &'static str,
    label: &'static str,
    description: &'static str,
    keywords: &'static [&'static str],
}

/// All searchable settings
const SEARCHABLE_ITEMS: &[SearchableItem] = &[
    // Appearance (0)
    SearchableItem {
        category: 0,
        setting_id: "focus-ring-enabled",
        label: "Focus Ring",
        description: "Show a colored ring around the focused window",
        keywords: &["focus", "ring", "border", "highlight", "active"],
    },
    SearchableItem {
        category: 0,
        setting_id: "focus-ring-width",
        label: "Focus Ring Width",
        description: "Thickness of the focus ring in pixels",
        keywords: &["focus", "ring", "width", "size", "thickness"],
    },
    SearchableItem {
        category: 0,
        setting_id: "focus-ring-color",
        label: "Focus Ring Color",
        description: "Color of the focus ring when window is active",
        keywords: &["focus", "ring", "color", "active", "highlight"],
    },
    SearchableItem {
        category: 0,
        setting_id: "border-enabled",
        label: "Window Border",
        description: "Draw a border around windows",
        keywords: &["border", "window", "outline", "frame"],
    },
    SearchableItem {
        category: 0,
        setting_id: "border-width",
        label: "Border Thickness",
        description: "Width of the window border in pixels",
        keywords: &["border", "width", "thickness", "size"],
    },
    SearchableItem {
        category: 0,
        setting_id: "gaps-inner",
        label: "Inner Gaps",
        description: "Space between tiled windows",
        keywords: &["gap", "spacing", "inner", "between", "windows"],
    },
    SearchableItem {
        category: 0,
        setting_id: "gaps-outer",
        label: "Outer Gaps",
        description: "Space between windows and screen edges",
        keywords: &["gap", "spacing", "outer", "edge", "margin"],
    },
    SearchableItem {
        category: 0,
        setting_id: "corner-radius",
        label: "Corner Radius",
        description: "Rounded corners for windows",
        keywords: &["corner", "radius", "rounded", "round"],
    },
    // Behavior (1)
    SearchableItem {
        category: 1,
        setting_id: "focus-follows-mouse",
        label: "Focus Follows Mouse",
        description: "Windows get focus when the mouse moves over them",
        keywords: &["focus", "mouse", "hover", "follows", "pointer"],
    },
    SearchableItem {
        category: 1,
        setting_id: "warp-mouse-to-focus",
        label: "Warp Mouse to Focus",
        description: "Move mouse cursor to focused window",
        keywords: &["warp", "mouse", "cursor", "focus", "move"],
    },
    SearchableItem {
        category: 1,
        setting_id: "center-focused-column",
        label: "Center Focused Column",
        description: "Keep the focused column centered on screen",
        keywords: &["center", "focused", "column", "scroll"],
    },
    SearchableItem {
        category: 1,
        setting_id: "default-column-width",
        label: "Default Column Width",
        description: "Initial width for new columns",
        keywords: &["column", "width", "default", "size", "proportion"],
    },
    SearchableItem {
        category: 1,
        setting_id: "struts",
        label: "Screen Margins",
        description: "Reserved space at screen edges (struts)",
        keywords: &["strut", "struts", "margin", "reserved", "edge", "space"],
    },
    // Keyboard (2)
    SearchableItem {
        category: 2,
        setting_id: "keyboard-layout",
        label: "Keyboard Layout",
        description: "XKB keyboard layout (us, de, fr, etc.)",
        keywords: &["keyboard", "layout", "xkb", "language", "input"],
    },
    SearchableItem {
        category: 2,
        setting_id: "keyboard-variant",
        label: "Layout Variant",
        description: "Keyboard layout variant (dvorak, colemak, etc.)",
        keywords: &["variant", "dvorak", "colemak", "layout"],
    },
    SearchableItem {
        category: 2,
        setting_id: "keyboard-options",
        label: "XKB Options",
        description: "Additional keyboard options (ctrl:nocaps, etc.)",
        keywords: &["xkb", "options", "caps", "ctrl", "swap"],
    },
    SearchableItem {
        category: 2,
        setting_id: "keyboard-repeat-delay",
        label: "Repeat Delay",
        description: "Delay before key repeat starts (milliseconds)",
        keywords: &["repeat", "delay", "key", "hold", "milliseconds"],
    },
    SearchableItem {
        category: 2,
        setting_id: "keyboard-repeat-rate",
        label: "Repeat Rate",
        description: "Speed of key repetition (keys per second)",
        keywords: &["repeat", "rate", "speed", "key", "fast"],
    },
    SearchableItem {
        category: 2,
        setting_id: "keyboard-numlock",
        label: "NumLock",
        description: "Enable NumLock on startup",
        keywords: &["numlock", "number", "keypad", "startup"],
    },
    // Mouse (3)
    SearchableItem {
        category: 3,
        setting_id: "mouse-natural-scroll",
        label: "Natural Scrolling",
        description: "Scroll direction matches finger movement",
        keywords: &["natural", "scroll", "reverse", "direction"],
    },
    SearchableItem {
        category: 3,
        setting_id: "mouse-left-handed",
        label: "Left-Handed Mode",
        description: "Swap primary and secondary mouse buttons",
        keywords: &["left", "handed", "swap", "buttons"],
    },
    SearchableItem {
        category: 3,
        setting_id: "mouse-accel-speed",
        label: "Acceleration Speed",
        description: "Mouse pointer acceleration",
        keywords: &["acceleration", "speed", "pointer", "fast", "slow"],
    },
    SearchableItem {
        category: 3,
        setting_id: "mouse-accel-profile",
        label: "Acceleration Profile",
        description: "Pointer acceleration curve (flat or adaptive)",
        keywords: &["acceleration", "profile", "flat", "adaptive"],
    },
    SearchableItem {
        category: 3,
        setting_id: "mouse-scroll-factor",
        label: "Scroll Factor",
        description: "Scroll speed multiplier",
        keywords: &["scroll", "factor", "speed", "multiplier"],
    },
    SearchableItem {
        category: 3,
        setting_id: "mouse-middle-emulation",
        label: "Middle Click Emulation",
        description: "Emulate middle click with left+right click",
        keywords: &["middle", "emulation", "click", "button"],
    },
    // Touchpad (4)
    SearchableItem {
        category: 4,
        setting_id: "touchpad-tap",
        label: "Tap to Click",
        description: "Tap the touchpad to click",
        keywords: &["tap", "click", "touchpad", "touch"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-tap-button-map",
        label: "Tap Button Map",
        description: "Multi-finger tap button assignment",
        keywords: &["tap", "button", "map", "finger"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-natural-scroll",
        label: "Natural Scrolling",
        description: "Scroll direction matches finger movement",
        keywords: &["natural", "scroll", "touchpad", "direction"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-scroll-method",
        label: "Scroll Method",
        description: "How to scroll (two-finger, edge, etc.)",
        keywords: &["scroll", "method", "two", "finger", "edge"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-click-method",
        label: "Click Method",
        description: "How clicks are detected (button areas or clickfinger)",
        keywords: &["click", "method", "button", "areas", "clickfinger"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-dwt",
        label: "Disable While Typing",
        description: "Disable touchpad while using keyboard",
        keywords: &["disable", "typing", "dwt", "palm", "rejection"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-dwtp",
        label: "Disable While Trackpointing",
        description: "Disable touchpad while using trackpoint",
        keywords: &["disable", "trackpoint", "dwtp", "thinkpad"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-tap-drag",
        label: "Tap and Drag",
        description: "Tap and hold to drag",
        keywords: &["drag", "tap", "hold", "move"],
    },
    SearchableItem {
        category: 4,
        setting_id: "touchpad-drag-lock",
        label: "Drag Lock",
        description: "Keep dragging after lifting finger",
        keywords: &["drag", "lock", "continue"],
    },
    // Displays (5)
    SearchableItem {
        category: 5,
        setting_id: "display-scale",
        label: "Display Scale",
        description: "Scaling factor for high-DPI displays",
        keywords: &["scale", "dpi", "hidpi", "retina", "display"],
    },
    SearchableItem {
        category: 5,
        setting_id: "display-mode",
        label: "Display Mode",
        description: "Resolution and refresh rate",
        keywords: &["mode", "resolution", "refresh", "rate", "hz"],
    },
    SearchableItem {
        category: 5,
        setting_id: "display-position",
        label: "Display Position",
        description: "Position of display in multi-monitor setup",
        keywords: &["position", "monitor", "multi", "arrangement"],
    },
    SearchableItem {
        category: 5,
        setting_id: "display-transform",
        label: "Display Transform",
        description: "Rotation and flip settings",
        keywords: &["transform", "rotate", "rotation", "flip", "orientation"],
    },
    SearchableItem {
        category: 5,
        setting_id: "display-vrr",
        label: "Variable Refresh Rate",
        description: "VRR/FreeSync/G-Sync for gaming",
        keywords: &["vrr", "freesync", "gsync", "adaptive", "sync", "gaming"],
    },
    // Animations (6)
    SearchableItem {
        category: 6,
        setting_id: "animations-enabled",
        label: "Enable Animations",
        description: "Enable or disable all animations",
        keywords: &["animation", "enable", "disable", "motion"],
    },
    SearchableItem {
        category: 6,
        setting_id: "animations-speed",
        label: "Animation Speed",
        description: "Slowdown factor for animations",
        keywords: &["animation", "speed", "slow", "fast", "duration"],
    },
    // Cursor (7)
    SearchableItem {
        category: 7,
        setting_id: "cursor-theme",
        label: "Cursor Theme",
        description: "Mouse cursor icon theme",
        keywords: &["cursor", "theme", "pointer", "icon"],
    },
    SearchableItem {
        category: 7,
        setting_id: "cursor-size",
        label: "Cursor Size",
        description: "Size of the mouse cursor in pixels",
        keywords: &["cursor", "size", "pointer", "big", "small"],
    },
    SearchableItem {
        category: 7,
        setting_id: "cursor-hide-typing",
        label: "Hide When Typing",
        description: "Hide cursor while typing on keyboard",
        keywords: &["hide", "typing", "cursor", "keyboard"],
    },
    SearchableItem {
        category: 7,
        setting_id: "cursor-hide-inactive",
        label: "Hide After Inactive",
        description: "Hide cursor after period of inactivity",
        keywords: &["hide", "inactive", "timeout", "auto"],
    },
    // Overview (8)
    SearchableItem {
        category: 8,
        setting_id: "overview-zoom",
        label: "Overview Zoom",
        description: "Zoom level in the overview",
        keywords: &["overview", "zoom", "scale", "size"],
    },
    SearchableItem {
        category: 8,
        setting_id: "overview-backdrop",
        label: "Backdrop Color",
        description: "Background color in the overview",
        keywords: &["backdrop", "background", "color", "overview"],
    },
    // Layout Extras (9)
    SearchableItem {
        category: 9,
        setting_id: "shadow-enabled",
        label: "Window Shadows",
        description: "Drop shadows behind windows",
        keywords: &["shadow", "drop", "window", "effect"],
    },
    SearchableItem {
        category: 9,
        setting_id: "shadow-softness",
        label: "Shadow Softness",
        description: "Blur amount for shadow edges",
        keywords: &["shadow", "softness", "blur", "soft"],
    },
    SearchableItem {
        category: 9,
        setting_id: "shadow-spread",
        label: "Shadow Spread",
        description: "How far shadow extends from window",
        keywords: &["shadow", "spread", "size", "extend"],
    },
    SearchableItem {
        category: 9,
        setting_id: "shadow-offset",
        label: "Shadow Offset",
        description: "Position offset for shadows",
        keywords: &["shadow", "offset", "position", "x", "y"],
    },
    SearchableItem {
        category: 9,
        setting_id: "shadow-color",
        label: "Shadow Color",
        description: "Color for window shadows",
        keywords: &["shadow", "color", "dark"],
    },
    SearchableItem {
        category: 9,
        setting_id: "tab-indicator-enabled",
        label: "Tab Indicator",
        description: "Visual indicator for stacked/tabbed windows",
        keywords: &["tab", "indicator", "stack", "tabbed"],
    },
    SearchableItem {
        category: 9,
        setting_id: "tab-indicator-position",
        label: "Tab Indicator Position",
        description: "Which side to show the tab indicator",
        keywords: &[
            "tab",
            "indicator",
            "position",
            "left",
            "right",
            "top",
            "bottom",
        ],
    },
    SearchableItem {
        category: 9,
        setting_id: "insert-hint",
        label: "Insert Hint",
        description: "Visual hint when inserting windows",
        keywords: &["insert", "hint", "visual", "feedback"],
    },
    // Gestures (10)
    SearchableItem {
        category: 10,
        setting_id: "hotcorner-enabled",
        label: "Hot Corners",
        description: "Trigger actions by moving cursor to screen corners",
        keywords: &["hot", "corner", "screen", "trigger", "overview"],
    },
    SearchableItem {
        category: 10,
        setting_id: "gesture-edge-scroll",
        label: "Edge Scrolling",
        description: "Scroll view when dragging near edges",
        keywords: &["edge", "scroll", "drag", "dnd"],
    },
    SearchableItem {
        category: 10,
        setting_id: "gesture-workspace-switch",
        label: "Workspace Switching",
        description: "Switch workspaces when dragging in overview",
        keywords: &["workspace", "switch", "drag", "overview", "dnd"],
    },
    SearchableItem {
        category: 10,
        setting_id: "gesture-dnd-trigger",
        label: "Drag and Drop Trigger",
        description: "Edge trigger zone for drag gestures",
        keywords: &["drag", "drop", "trigger", "zone", "width"],
    },
    SearchableItem {
        category: 10,
        setting_id: "gesture-delay",
        label: "Gesture Delay",
        description: "Delay before gesture activates",
        keywords: &["gesture", "delay", "wait", "time"],
    },
    // Miscellaneous (11)
    SearchableItem {
        category: 11,
        setting_id: "prefer-no-csd",
        label: "Prefer No CSD",
        description: "Ask apps to not draw their own title bars",
        keywords: &["csd", "title", "bar", "decoration", "server"],
    },
    SearchableItem {
        category: 11,
        setting_id: "screenshot-path",
        label: "Screenshot Path",
        description: "Where to save screenshots",
        keywords: &["screenshot", "path", "save", "location"],
    },
    SearchableItem {
        category: 11,
        setting_id: "clipboard-primary",
        label: "Primary Clipboard",
        description: "Middle-click paste clipboard",
        keywords: &["clipboard", "primary", "middle", "paste", "selection"],
    },
    SearchableItem {
        category: 11,
        setting_id: "hotkey-overlay",
        label: "Hotkey Overlay",
        description: "Keyboard shortcuts help overlay",
        keywords: &["hotkey", "overlay", "help", "shortcuts", "startup"],
    },
    // Window Rules (12)
    SearchableItem {
        category: 12,
        setting_id: "window-rules",
        label: "Window Rules",
        description: "Rules to customize window behavior",
        keywords: &["window", "rules", "match", "app", "title"],
    },
    SearchableItem {
        category: 12,
        setting_id: "window-rule-app-id",
        label: "App ID Match",
        description: "Match windows by application identifier",
        keywords: &["app", "id", "match", "regex", "application"],
    },
    SearchableItem {
        category: 12,
        setting_id: "window-rule-title",
        label: "Title Match",
        description: "Match windows by title pattern",
        keywords: &["title", "match", "regex", "window", "name"],
    },
    SearchableItem {
        category: 12,
        setting_id: "window-rule-maximized",
        label: "Open Maximized",
        description: "Start windows in maximized state",
        keywords: &["open", "maximized", "fullscreen", "floating", "behavior"],
    },
    SearchableItem {
        category: 12,
        setting_id: "window-rule-opacity",
        label: "Window Opacity",
        description: "Set window transparency per-application",
        keywords: &["opacity", "transparency", "alpha", "window"],
    },
    SearchableItem {
        category: 12,
        setting_id: "window-rule-screencast",
        label: "Block Screencast",
        description: "Hide window from screen recordings",
        keywords: &["screencast", "block", "hide", "recording", "privacy"],
    },
];

/// Maximum search query length to prevent excessive processing
const MAX_SEARCH_QUERY_LEN: usize = 200;

/// Search for settings matching the query
pub fn search_settings(query: &str) -> Vec<SearchResult> {
    // Limit query length to prevent excessive processing
    if query.len() > MAX_SEARCH_QUERY_LEN {
        return Vec::new();
    }

    let query = query.to_lowercase();
    let query_words: Vec<&str> = query.split_whitespace().collect();

    if query_words.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<SearchResult> = SEARCHABLE_ITEMS
        .iter()
        .filter_map(|item| {
            let mut score = 0;

            // Check label
            let label_lower = item.label.to_lowercase();
            for word in &query_words {
                if label_lower.contains(word) {
                    score += 10; // High score for label match
                    if label_lower.starts_with(word) {
                        score += 5; // Bonus for prefix match
                    }
                }
            }

            // Check description
            let desc_lower = item.description.to_lowercase();
            for word in &query_words {
                if desc_lower.contains(word) {
                    score += 5;
                }
            }

            // Check keywords
            for keyword in item.keywords {
                for word in &query_words {
                    if keyword.contains(word) {
                        score += 3;
                    }
                }
            }

            if score > 0 {
                Some(SearchResult {
                    category: item.category,
                    setting_id: item.setting_id.to_string(),
                    label: item.label.to_string(),
                    description: item.description.to_string(),
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.cmp(&a.score));

    debug!("Search '{}' found {} results", query, results.len());
    results
}

/// Get the best category to navigate to for a search query
pub fn get_best_category(query: &str) -> Option<i32> {
    let results = search_settings(query);
    results.first().map(|r| r.category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_focus() {
        let results = search_settings("focus");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.label.contains("Focus")));
    }

    #[test]
    fn test_search_empty() {
        let results = search_settings("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_case_insensitive() {
        let results_lower = search_settings("animation");
        let results_upper = search_settings("ANIMATION");
        assert_eq!(results_lower.len(), results_upper.len());
    }
}
