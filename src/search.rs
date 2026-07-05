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
        results.sort_by_key(|r| std::cmp::Reverse(r.relevance_score));

        // Limit results (matches UI display limit in search modal)
        results.truncate(MAX_VISIBLE_RESULTS);

        results
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn build_settings_index() -> Vec<SettingEntry> {
    Page::ALL.iter().flat_map(|p| page_entries(*p)).collect()
}

/// Maximum number of search results shown (and selectable via keyboard) at once.
/// Shared by the modal list, the dropdown, and the keyboard-nav clamp.
pub const MAX_VISIBLE_RESULTS: usize = 8;

/// Clamps `selected` to the range of visible results for keyboard navigation.
/// Returns 0 when there are no results (nav is a no-op).
pub fn clamp_selected_index(selected: usize, result_count: usize) -> usize {
    let visible = result_count.min(MAX_VISIBLE_RESULTS);
    if visible == 0 {
        0
    } else {
        selected.min(visible - 1)
    }
}

/// Returns the searchable settings for a single page.
///
/// This match is intentionally wildcard-free: adding a `Page` variant will fail
/// to compile here until a corresponding arm (with at least one entry) is added,
/// guaranteeing that every page is reachable through search.
fn page_entries(page: Page) -> Vec<SettingEntry> {
    match page {
        Page::Overview => vec![SettingEntry::new(
            Page::Overview,
            "Overview",
            "Summary of your niri configuration",
            &["overview", "summary", "home", "dashboard"],
        )],
        Page::Appearance => vec![
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
        ],
        Page::LayoutExtras => vec![
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
            SettingEntry::new(
                Page::LayoutExtras,
                "Insert Hint",
                "Highlight shown where a window will be inserted",
                &["insert", "hint", "struts", "gaps", "layout"],
            ),
        ],
        Page::Behavior => vec![
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
        ],
        Page::Keyboard => vec![
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
        ],
        Page::Mouse => vec![
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
        ],
        Page::Touchpad => vec![
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
        ],
        Page::Trackpoint => vec![
            SettingEntry::new(
                Page::Trackpoint,
                "Pointer Speed",
                "Base speed multiplier for the trackpoint",
                &["trackpoint", "nipple", "pointer", "speed", "sensitivity"],
            ),
            SettingEntry::new(
                Page::Trackpoint,
                "Scroll Method",
                "How scrolling works with the trackpoint",
                &["trackpoint", "scroll", "method", "on-button-down"],
            ),
            SettingEntry::new(
                Page::Trackpoint,
                "Acceleration Profile",
                "Pointer acceleration profile for the trackpoint",
                &[
                    "trackpoint",
                    "acceleration",
                    "accel",
                    "profile",
                    "flat",
                    "adaptive",
                ],
            ),
        ],
        Page::Trackball => vec![
            SettingEntry::new(
                Page::Trackball,
                "Scroll Method",
                "How scrolling works with the trackball",
                &["trackball", "scroll", "method", "on-button-down"],
            ),
            SettingEntry::new(
                Page::Trackball,
                "Middle Button Emulation",
                "Emulate a middle click on the trackball",
                &["trackball", "middle", "button", "emulation"],
            ),
            SettingEntry::new(
                Page::Trackball,
                "Left Handed",
                "Swap buttons for left-handed trackball use",
                &["trackball", "left", "handed", "swap", "buttons"],
            ),
        ],
        Page::Tablet => vec![
            SettingEntry::new(
                Page::Tablet,
                "Map to Output",
                "Bind the drawing tablet to a specific monitor",
                &[
                    "tablet", "pen", "stylus", "wacom", "map", "output", "monitor",
                ],
            ),
            SettingEntry::new(
                Page::Tablet,
                "Map to Focused Output/Window",
                "Follow the focused output or window with the tablet",
                &[
                    "tablet", "pen", "stylus", "wacom", "focused", "output", "window",
                ],
            ),
            SettingEntry::new(
                Page::Tablet,
                "Left Handed Mode",
                "Rotate tablet input for left-handed use",
                &["tablet", "pen", "stylus", "wacom", "left", "handed"],
            ),
            SettingEntry::new(
                Page::Tablet,
                "Calibration Matrix",
                "Calibration matrix for the drawing tablet",
                &["tablet", "pen", "stylus", "wacom", "calibration", "matrix"],
            ),
        ],
        Page::Touch => vec![
            SettingEntry::new(
                Page::Touch,
                "Enable Touch",
                "Enable or disable the touchscreen",
                &["touch", "touchscreen", "enable", "disable"],
            ),
            SettingEntry::new(
                Page::Touch,
                "Map to Output",
                "Bind the touchscreen to a specific monitor",
                &["touch", "touchscreen", "map", "output", "monitor"],
            ),
            SettingEntry::new(
                Page::Touch,
                "Calibration",
                "Calibration matrix for the touchscreen",
                &["touch", "touchscreen", "calibration", "matrix"],
            ),
        ],
        Page::Cursor => vec![
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
        ],
        Page::Animations => vec![
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
        ],
        Page::Workspaces => vec![SettingEntry::new(
            Page::Workspaces,
            "Named Workspaces",
            "Create workspaces with custom names",
            &["workspace", "name", "named", "label", "create"],
        )],
        Page::WindowRules => vec![
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
        ],
        Page::LayerRules => vec![SettingEntry::new(
            Page::LayerRules,
            "Layer Rules",
            "Rules for panels, bars, and overlays",
            &["layer", "rules", "panel", "bar", "waybar", "overlay"],
        )],
        Page::Keybindings => vec![
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
        ],
        Page::Outputs => vec![
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
        ],
        Page::Startup => vec![SettingEntry::new(
            Page::Startup,
            "Startup Applications",
            "Programs to launch when niri starts",
            &["startup", "autostart", "launch", "boot", "programs"],
        )],
        Page::Environment => vec![SettingEntry::new(
            Page::Environment,
            "Environment Variables",
            "Set environment variables for niri session",
            &["environment", "variables", "env", "export", "path"],
        )],
        Page::Debug => vec![
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
        ],
        Page::Miscellaneous => vec![
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
        ],
        Page::Gestures => vec![
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
            SettingEntry::new(
                Page::Gestures,
                "Hot Corners",
                "Trigger actions by moving the cursor to a screen corner",
                &["hot", "corner", "corners", "dnd", "edge"],
            ),
        ],
        Page::SwitchEvents => vec![
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
        ],
        Page::Blur => vec![
            SettingEntry::new(
                Page::Blur,
                "Background Blur",
                "Enable or disable background blur (niri 26.04+)",
                &[
                    "blur",
                    "background",
                    "transparency",
                    "frosted",
                    "glass",
                    "effect",
                ],
            ),
            SettingEntry::new(
                Page::Blur,
                "Blur Passes",
                "Number of blur passes — quality vs GPU cost",
                &["passes", "quality", "kawase"],
            ),
            SettingEntry::new(
                Page::Blur,
                "Blur Offset",
                "Blur offset multiplier per pass",
                &["offset", "radius", "strength"],
            ),
            SettingEntry::new(
                Page::Blur,
                "Blur Noise",
                "Noise added to reduce color banding",
                &["noise", "banding", "grain"],
            ),
            SettingEntry::new(
                Page::Blur,
                "Blur Saturation",
                "Color saturation behind blur",
                &["saturation", "color", "vibrance"],
            ),
        ],
        Page::RecentWindows => vec![SettingEntry::new(
            Page::RecentWindows,
            "Recent Windows",
            "Alt-Tab style recently used window switcher",
            &["recent", "windows", "alt-tab", "switcher", "mru"],
        )],
        Page::Tools => vec![
            SettingEntry::new(
                Page::Tools,
                "Niri Version",
                "Detected niri compositor version",
                &["version", "niri", "about", "compositor"],
            ),
            SettingEntry::new(
                Page::Tools,
                "IPC Tools",
                "Inspect niri over its IPC socket",
                &["ipc", "msg", "socket", "tools", "debug"],
            ),
        ],
        Page::Preferences => vec![
            SettingEntry::new(
                Page::Preferences,
                "App Theme",
                "Light, dark, or system app theme",
                &[
                    "theme",
                    "dark",
                    "light",
                    "system",
                    "appearance",
                    "preferences",
                ],
            ),
            SettingEntry::new(
                Page::Preferences,
                "Search Hotkey",
                "Keyboard shortcut to open settings search",
                &["hotkey", "shortcut", "search", "keybinding", "preferences"],
            ),
        ],
        Page::ConfigEditor => vec![SettingEntry::new(
            Page::ConfigEditor,
            "Config Editor",
            "Edit the raw KDL configuration files",
            &["kdl", "editor", "raw", "file", "config"],
        )],
        Page::Backups => vec![
            SettingEntry::new(
                Page::Backups,
                "Create Backup",
                "Save a snapshot of your current configuration",
                &["backup", "snapshot", "save", "create"],
            ),
            SettingEntry::new(
                Page::Backups,
                "Restore Backup",
                "Recover a previous configuration snapshot",
                &["backup", "restore", "recovery", "undo", "snapshot"],
            ),
        ],
    }
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

    #[test]
    fn test_every_page_has_entries() {
        for p in Page::ALL {
            assert!(
                !page_entries(*p).is_empty(),
                "page {:?} has no search entries",
                p
            );
        }
    }

    #[test]
    fn test_page_all_no_duplicates_and_count() {
        // Bump this when adding a Page variant (page_entries' exhaustive match
        // is the compile-time guard that a new page gets entries).
        const EXPECTED_PAGE_COUNT: usize = 30;
        assert_eq!(Page::ALL.len(), EXPECTED_PAGE_COUNT);
        for (i, a) in Page::ALL.iter().enumerate() {
            for b in &Page::ALL[i + 1..] {
                assert_ne!(a, b, "duplicate page in Page::ALL: {:?}", a);
            }
        }
    }

    fn search_targets_page(query: &str, page: Page) -> bool {
        let index = SearchIndex::new();
        index.search(query).iter().any(|r| r.page == page)
    }

    #[test]
    fn test_search_trackball() {
        assert!(search_targets_page("trackball", Page::Trackball));
    }

    #[test]
    fn test_search_tablet() {
        assert!(search_targets_page("tablet", Page::Tablet));
    }

    #[test]
    fn test_search_calibration() {
        let index = SearchIndex::new();
        let results = index.search("calibration");
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.page == Page::Tablet || r.page == Page::Touch));
    }

    #[test]
    fn test_search_backup() {
        assert!(search_targets_page("backup", Page::Backups));
    }

    #[test]
    fn test_selected_index_cap() {
        // For 20 results the max selectable index is MAX_VISIBLE_RESULTS - 1.
        assert_eq!(clamp_selected_index(19, 20), MAX_VISIBLE_RESULTS - 1);
        assert_eq!(clamp_selected_index(3, 20), 3);
        // For 0 results nav is a no-op.
        assert_eq!(clamp_selected_index(5, 0), 0);
    }
}
