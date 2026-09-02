//! Search functionality for settings
//!
//! Indexes individual settings with human-readable labels. Results land on a
//! redesigned [`Screen`] plus an [`EditableSection`] / [`EditableDevice`] /
//! gear or rules tab — not a leftover full [`Page`].

use crate::messages::{EditableDevice, EditableSection, GearSubTab, RulesSubTab, Screen};

/// Where a search result should take the user in the redesigned chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDestination {
    /// Open a section editor modal on its host screen
    Section(EditableSection),
    /// Open an input-device editor modal
    Device(EditableDevice),
    /// Input screen (keybindings table — no device modal)
    Keybindings,
    /// Displays screen (pick a monitor to edit)
    Displays,
    /// Rules screen on a specific sub-tab
    Rules(RulesSubTab),
    /// Gear / Settings screen on a specific sub-tab
    Gear(GearSubTab),
}

impl SearchDestination {
    /// Screen the redesigned chrome should switch to
    pub fn screen(self) -> Screen {
        match self {
            Self::Section(section) => section.screen(),
            Self::Device(_) | Self::Keybindings => Screen::Input,
            Self::Displays => Screen::Displays,
            Self::Rules(_) => Screen::Rules,
            Self::Gear(_) => Screen::Gear,
        }
    }

    /// Short location label shown next to a result (sidebar / modal)
    pub fn location_label(self) -> &'static str {
        match self {
            Self::Section(section) => section.name(),
            Self::Device(device) => device.name(),
            Self::Keybindings => "Keybindings",
            Self::Displays => "Displays",
            Self::Rules(tab) => tab.name(),
            Self::Gear(tab) => tab.name(),
        }
    }
}

/// Search result pointing to a specific setting
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub destination: SearchDestination,
    /// Human-readable setting name (e.g., "Enable Focus Ring")
    pub setting_name: String,
    /// Brief description of what the setting does
    pub description: String,
    /// Relevance score for sorting
    pub relevance_score: u32,
}

/// A searchable setting entry
struct SettingEntry {
    destination: SearchDestination,
    setting_name: &'static str,
    description: &'static str,
    name_lower: String,
    desc_lower: String,
    keywords: &'static [&'static str],
}

impl SettingEntry {
    fn new(
        destination: SearchDestination,
        setting_name: &'static str,
        description: &'static str,
        keywords: &'static [&'static str],
    ) -> Self {
        Self {
            destination,
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

                if entry.name_lower.contains(&query_lower) {
                    score += 100;
                } else {
                    for term in &query_terms {
                        if entry.name_lower.contains(term) {
                            score += 40;
                        }
                    }
                }

                for term in &query_terms {
                    if entry.desc_lower.contains(term) {
                        score += 20;
                    }
                }

                for keyword in entry.keywords {
                    for term in &query_terms {
                        if keyword.contains(term) {
                            score += if *keyword == *term { 30 } else { 15 };
                        }
                    }
                }

                if score > 0 {
                    Some(SearchResult {
                        destination: entry.destination,
                        setting_name: entry.setting_name.to_string(),
                        description: entry.description.to_string(),
                        relevance_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by_key(|r| std::cmp::Reverse(r.relevance_score));
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
    let mut entries = Vec::new();
    for section in EditableSection::ALL {
        entries.extend(section_entries(*section));
    }
    for device in EditableDevice::ALL {
        entries.extend(device_entries(*device));
    }
    entries.extend(screen_entries());
    entries
}

/// Maximum number of search results shown (and selectable via keyboard) at once.
pub const MAX_VISIBLE_RESULTS: usize = 8;

/// Clamps `selected` to the range of visible results for keyboard navigation.
pub fn clamp_selected_index(selected: usize, result_count: usize) -> usize {
    let visible = result_count.min(MAX_VISIBLE_RESULTS);
    if visible == 0 {
        0
    } else {
        selected.min(visible - 1)
    }
}

fn e(
    dest: SearchDestination,
    name: &'static str,
    desc: &'static str,
    keywords: &'static [&'static str],
) -> SettingEntry {
    SettingEntry::new(dest, name, desc, keywords)
}

/// Searchable settings for a single editor section.
///
/// Wildcard-free: adding an [`EditableSection`] variant fails to compile here
/// until at least one entry is added.
fn section_entries(section: EditableSection) -> Vec<SettingEntry> {
    let dest = SearchDestination::Section(section);
    match section {
        EditableSection::SpatialGaps => vec![
            e(
                dest,
                "Window Gaps",
                "Space between windows",
                &["gaps", "spacing", "margin", "windows", "space", "between"],
            ),
            e(
                dest,
                "Corner Radius",
                "Rounded corners on windows",
                &["corner", "radius", "rounded", "curve", "windows"],
            ),
        ],
        EditableSection::CenteringDynamics => vec![
            e(
                dest,
                "Focus Follows Mouse",
                "Window focus follows the mouse cursor",
                &["focus", "mouse", "cursor", "hover", "follow"],
            ),
            e(
                dest,
                "Warp Mouse on Focus",
                "Move cursor to focused window",
                &["warp", "mouse", "cursor", "focus", "move", "teleport"],
            ),
            e(
                dest,
                "Workspace Auto Back-and-Forth",
                "Switching to current workspace goes to previous",
                &["workspace", "back", "forth", "toggle", "previous", "auto"],
            ),
        ],
        EditableSection::ColumnManager => vec![
            e(
                dest,
                "Center Single Column",
                "Center windows when only one column exists",
                &["center", "single", "column", "window", "middle"],
            ),
            e(
                dest,
                "Default Column Width",
                "Default width for new columns",
                &["column", "width", "default", "size"],
            ),
        ],
        EditableSection::ScreenEdgeStruts => vec![e(
            dest,
            "Screen Edge Struts",
            "Reserved space along the edges of the screen",
            &["struts", "edge", "margin", "reserved", "panel"],
        )],
        EditableSection::TabIndicator => vec![e(
            dest,
            "Tab Indicator",
            "Visual indicator for tabbed window columns",
            &["tab", "indicator", "tabs", "column"],
        )],
        EditableSection::InsertHint => vec![e(
            dest,
            "Insert Hint",
            "Highlight shown where a window will be inserted",
            &["insert", "hint", "struts", "gaps", "layout"],
        )],
        EditableSection::NamedWorkspaces => vec![e(
            dest,
            "Named Workspaces",
            "Create workspaces with custom names",
            &["workspace", "name", "named", "label", "create"],
        )],
        EditableSection::PresetSizes => vec![
            e(
                dest,
                "Preset Column Widths",
                "Widths cycled by switch-preset-column-width",
                &["preset", "column", "width", "sizes", "cycle"],
            ),
            e(
                dest,
                "Preset Window Heights",
                "Heights cycled by switch-preset-window-height",
                &["preset", "window", "height", "sizes", "cycle"],
            ),
        ],
        EditableSection::FocusRing => vec![
            e(
                dest,
                "Enable Focus Ring",
                "Show a colored ring around the focused window",
                &["focus", "ring", "border", "highlight", "active", "window"],
            ),
            e(
                dest,
                "Focus Ring Color",
                "Color of the ring around focused windows",
                &["focus", "ring", "color", "active", "highlight"],
            ),
            e(
                dest,
                "Focus Ring Width",
                "Thickness of the focus ring in pixels",
                &["focus", "ring", "width", "thickness", "size", "border"],
            ),
        ],
        EditableSection::WindowBorder => vec![
            e(
                dest,
                "Inactive Window Border",
                "Border color for unfocused windows",
                &["inactive", "border", "unfocused", "color", "window"],
            ),
            e(
                dest,
                "Border Width",
                "Thickness of window borders",
                &["border", "width", "thickness", "outline"],
            ),
        ],
        EditableSection::WindowShadow => vec![
            e(
                dest,
                "Enable Window Shadow",
                "Show shadow behind windows",
                &["shadow", "drop", "window", "effect"],
            ),
            e(
                dest,
                "Shadow Softness",
                "Blur amount for window shadows",
                &["shadow", "softness", "blur", "soft"],
            ),
            e(
                dest,
                "Shadow Color",
                "Color of window shadows",
                &["shadow", "color"],
            ),
            e(
                dest,
                "Shadow Offset",
                "Position offset of window shadows",
                &["shadow", "offset", "position", "x", "y"],
            ),
        ],
        EditableSection::ModifierKeys => vec![e(
            dest,
            "Modifier Key",
            "Key used for window management (Super, Alt, etc.)",
            &["modifier", "mod", "key", "super", "alt", "ctrl", "meta"],
        )],
        EditableSection::Animations => vec![
            e(
                dest,
                "Enable Animations",
                "Turn animations on or off globally",
                &["animations", "enable", "disable", "motion", "effects"],
            ),
            e(
                dest,
                "Animation Speed",
                "How fast animations play",
                &["animation", "speed", "duration", "fast", "slow"],
            ),
            e(
                dest,
                "Window Open Animation",
                "Animation when windows open",
                &["window", "open", "animation", "appear", "spawn"],
            ),
            e(
                dest,
                "Window Close Animation",
                "Animation when windows close",
                &["window", "close", "animation", "disappear", "exit"],
            ),
            e(
                dest,
                "Workspace Switch Animation",
                "Animation when switching workspaces",
                &["workspace", "switch", "animation", "transition"],
            ),
        ],
        EditableSection::Cursor => vec![
            e(
                dest,
                "Cursor Theme",
                "Visual theme for the mouse cursor",
                &["cursor", "theme", "pointer", "icon", "style"],
            ),
            e(
                dest,
                "Cursor Size",
                "Size of the mouse cursor",
                &["cursor", "size", "big", "small", "scale"],
            ),
            e(
                dest,
                "Hide Cursor When Inactive",
                "Hide cursor after period of inactivity",
                &["hide", "cursor", "inactive", "timeout", "disappear"],
            ),
        ],
        EditableSection::Blur => vec![
            e(
                dest,
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
            e(
                dest,
                "Blur Passes",
                "Number of blur passes — quality vs GPU cost",
                &["passes", "quality", "kawase"],
            ),
            e(
                dest,
                "Blur Offset",
                "Blur offset multiplier per pass",
                &["offset", "radius", "strength"],
            ),
            e(
                dest,
                "Blur Noise",
                "Noise added to reduce color banding",
                &["noise", "banding", "grain"],
            ),
            e(
                dest,
                "Blur Saturation",
                "Color saturation behind blur",
                &["saturation", "color", "vibrance"],
            ),
        ],
        EditableSection::WorkspaceBackground => vec![e(
            dest,
            "Workspace Background",
            "Solid color drawn behind windows on every workspace",
            &[
                "workspace",
                "background",
                "wallpaper",
                "desktop",
                "color",
                "backdrop",
            ],
        )],
        EditableSection::Overview => vec![
            e(
                dest,
                "Overview Zoom",
                "How much to scale down windows in workspace overview",
                &[
                    "overview",
                    "zoom",
                    "expose",
                    "exposé",
                    "scale",
                    "toggle-overview",
                ],
            ),
            e(
                dest,
                "Overview Backdrop",
                "Background color behind workspaces in overview",
                &["overview", "backdrop", "background", "color", "expose"],
            ),
            e(
                dest,
                "Overview Workspace Shadow",
                "Shadow behind workspaces in overview (niri 25.05+)",
                &["overview", "shadow", "workspace", "expose"],
            ),
        ],
        EditableSection::StartupPrograms => vec![e(
            dest,
            "Startup Applications",
            "Programs to launch when niri starts",
            &["startup", "autostart", "launch", "boot", "programs"],
        )],
        EditableSection::EnvironmentVars => vec![e(
            dest,
            "Environment Variables",
            "Set environment variables for niri session",
            &["environment", "variables", "env", "export", "path"],
        )],
        EditableSection::Miscellaneous => vec![
            e(
                dest,
                "Screenshot Directory",
                "Where screenshots are saved",
                &["screenshot", "directory", "folder", "path", "save"],
            ),
            e(
                dest,
                "Prefer Server-Side Decorations",
                "Use compositor window decorations",
                &["decoration", "csd", "ssd", "titlebar", "server"],
            ),
        ],
        EditableSection::SwitchEvents => vec![
            e(
                dest,
                "Lid Close Action",
                "What happens when laptop lid closes",
                &["lid", "close", "laptop", "suspend", "sleep", "lock"],
            ),
            e(
                dest,
                "Tablet Mode",
                "Behavior when device enters tablet mode",
                &["tablet", "mode", "convertible", "touch"],
            ),
        ],
        EditableSection::Debug => vec![
            e(
                dest,
                "Show FPS Counter",
                "Display frames per second overlay",
                &["fps", "frames", "performance", "debug", "counter"],
            ),
            e(
                dest,
                "Render Damage Tracking",
                "Visualize screen redraw regions",
                &["damage", "render", "debug", "redraw"],
            ),
        ],
        EditableSection::RecentWindows => vec![e(
            dest,
            "Recent Windows",
            "Alt-Tab style recently used window switcher",
            &["recent", "windows", "alt-tab", "switcher", "mru"],
        )],
    }
}

fn device_entries(device: EditableDevice) -> Vec<SettingEntry> {
    let dest = SearchDestination::Device(device);
    match device {
        EditableDevice::Keyboard => vec![
            e(
                dest,
                "Keyboard Layout",
                "XKB keyboard layout (e.g., us, de, fr)",
                &["keyboard", "layout", "xkb", "language", "qwerty", "azerty"],
            ),
            e(
                dest,
                "Repeat Rate",
                "How fast keys repeat when held",
                &["repeat", "rate", "speed", "key", "hold"],
            ),
            e(
                dest,
                "Repeat Delay",
                "Delay before key repeat starts",
                &["repeat", "delay", "wait", "key", "hold"],
            ),
            e(
                dest,
                "Caps Lock Behavior",
                "What Caps Lock does (e.g., swap with Ctrl)",
                &["caps", "lock", "ctrl", "escape", "swap", "remap"],
            ),
        ],
        EditableDevice::Mouse => vec![
            e(
                dest,
                "Mouse Acceleration",
                "How mouse speed scales with movement",
                &["mouse", "acceleration", "accel", "speed", "sensitivity"],
            ),
            e(
                dest,
                "Mouse Speed",
                "Base speed multiplier for mouse movement",
                &["mouse", "speed", "sensitivity", "fast", "slow"],
            ),
            e(
                dest,
                "Natural Scrolling (Mouse)",
                "Reverse scroll direction",
                &["natural", "scroll", "reverse", "direction", "mouse"],
            ),
            e(
                dest,
                "Left-Handed Mouse",
                "Swap left and right mouse buttons",
                &["left", "handed", "swap", "buttons", "mouse"],
            ),
        ],
        EditableDevice::Touchpad => vec![
            e(
                dest,
                "Tap to Click",
                "Tap the touchpad to click",
                &["tap", "click", "touchpad", "finger"],
            ),
            e(
                dest,
                "Natural Scrolling (Touchpad)",
                "Reverse scroll direction on touchpad",
                &["natural", "scroll", "reverse", "touchpad"],
            ),
            e(
                dest,
                "Two-Finger Scroll",
                "Scroll using two fingers on touchpad",
                &["two", "finger", "scroll", "touchpad"],
            ),
            e(
                dest,
                "Disable While Typing",
                "Disable touchpad while using keyboard",
                &["disable", "typing", "dwt", "touchpad", "palm"],
            ),
            e(
                dest,
                "Touchpad Speed",
                "Cursor speed when using touchpad",
                &["touchpad", "speed", "sensitivity", "acceleration"],
            ),
        ],
        EditableDevice::Trackpoint => vec![
            e(
                dest,
                "Pointer Speed",
                "Base speed multiplier for the trackpoint",
                &["trackpoint", "nipple", "pointer", "speed", "sensitivity"],
            ),
            e(
                dest,
                "Scroll Method",
                "How scrolling works with the trackpoint",
                &["trackpoint", "scroll", "method", "on-button-down"],
            ),
            e(
                dest,
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
        EditableDevice::Trackball => vec![
            e(
                dest,
                "Scroll Method",
                "How scrolling works with the trackball",
                &["trackball", "scroll", "method", "on-button-down"],
            ),
            e(
                dest,
                "Middle Button Emulation",
                "Emulate a middle click on the trackball",
                &["trackball", "middle", "button", "emulation"],
            ),
            e(
                dest,
                "Left Handed",
                "Swap buttons for left-handed trackball use",
                &["trackball", "left", "handed", "swap", "buttons"],
            ),
        ],
        EditableDevice::Tablet => vec![
            e(
                dest,
                "Map to Output",
                "Bind the drawing tablet to a specific monitor",
                &[
                    "tablet", "pen", "stylus", "wacom", "map", "output", "monitor",
                ],
            ),
            e(
                dest,
                "Map to Focused Output/Window",
                "Follow the focused output or window with the tablet",
                &[
                    "tablet", "pen", "stylus", "wacom", "focused", "output", "window",
                ],
            ),
            e(
                dest,
                "Left Handed Mode",
                "Rotate tablet input for left-handed use",
                &["tablet", "pen", "stylus", "wacom", "left", "handed"],
            ),
            e(
                dest,
                "Calibration Matrix",
                "Calibration matrix for the drawing tablet",
                &["tablet", "pen", "stylus", "wacom", "calibration", "matrix"],
            ),
        ],
        EditableDevice::Touch => vec![
            e(
                dest,
                "Enable Touch",
                "Enable or disable the touchscreen",
                &["touch", "touchscreen", "enable", "disable"],
            ),
            e(
                dest,
                "Map to Output",
                "Bind the touchscreen to a specific monitor",
                &["touch", "touchscreen", "map", "output", "monitor"],
            ),
            e(
                dest,
                "Calibration",
                "Calibration matrix for the touchscreen",
                &["touch", "touchscreen", "calibration", "matrix"],
            ),
        ],
        EditableDevice::Gestures => vec![
            e(
                dest,
                "Touchpad Gestures",
                "Configure swipe and pinch gestures",
                &["gesture", "swipe", "pinch", "touchpad", "fingers"],
            ),
            e(
                dest,
                "Workspace Swipe Gesture",
                "Swipe to switch workspaces",
                &["swipe", "workspace", "gesture", "switch"],
            ),
            e(
                dest,
                "Hot Corners",
                "Trigger actions by moving the cursor to a screen corner",
                &["hot", "corner", "corners", "dnd", "edge"],
            ),
        ],
    }
}

/// Screens / tabs that are not a section or device editor.
fn screen_entries() -> Vec<SettingEntry> {
    vec![
        e(
            SearchDestination::Keybindings,
            "Keyboard Shortcuts",
            "Configure keybindings for actions",
            &["keyboard", "shortcuts", "keybindings", "hotkeys", "keys"],
        ),
        e(
            SearchDestination::Keybindings,
            "Close Window Shortcut",
            "Keyboard shortcut to close windows",
            &["close", "window", "shortcut", "quit", "kill"],
        ),
        e(
            SearchDestination::Keybindings,
            "Terminal Shortcut",
            "Keyboard shortcut to open terminal",
            &["terminal", "shortcut", "spawn", "launch", "open"],
        ),
        e(
            SearchDestination::Keybindings,
            "Screenshot Shortcut",
            "Keyboard shortcut for screenshots",
            &["screenshot", "shortcut", "capture", "screen", "print"],
        ),
        e(
            SearchDestination::Displays,
            "Monitor Configuration",
            "Configure display resolution and position",
            &["monitor", "display", "screen", "output", "resolution"],
        ),
        e(
            SearchDestination::Displays,
            "Display Scale",
            "HiDPI scaling factor for monitors",
            &["scale", "hidpi", "dpi", "display"],
        ),
        e(
            SearchDestination::Displays,
            "Refresh Rate",
            "Monitor refresh rate (Hz)",
            &["refresh", "rate", "hz", "hertz", "monitor"],
        ),
        e(
            SearchDestination::Displays,
            "Variable Refresh Rate",
            "VRR/Adaptive sync for monitors",
            &["vrr", "variable", "refresh", "adaptive", "sync", "freesync"],
        ),
        e(
            SearchDestination::Displays,
            "Monitor Rotation",
            "Rotate display orientation",
            &["rotate", "rotation", "orientation", "portrait", "landscape"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::WindowRules),
            "Window Rules",
            "Create rules for specific applications",
            &["window", "rules", "app", "application", "match"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::WindowRules),
            "Open on Workspace",
            "Open specific apps on designated workspaces",
            &["open", "workspace", "app", "application", "assign"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::WindowRules),
            "Default Window Size",
            "Set default size for specific apps",
            &["window", "size", "default", "width", "height", "app"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::WindowRules),
            "Force Floating",
            "Make specific windows always float",
            &["floating", "float", "window", "popup", "dialog"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::WindowRules),
            "Window Opacity",
            "Transparency for specific windows",
            &["opacity", "transparent", "alpha", "window"],
        ),
        e(
            SearchDestination::Rules(RulesSubTab::LayerRules),
            "Layer Rules",
            "Rules for panels, bars, and overlays",
            &["layer", "rules", "panel", "bar", "waybar", "overlay"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Tools),
            "Niri Version",
            "Detected niri compositor version",
            &["version", "niri", "about", "compositor"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Tools),
            "IPC Tools",
            "Inspect niri over its IPC socket",
            &["ipc", "msg", "socket", "tools", "debug"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Preferences),
            "App Theme",
            "Light, dark, or system app theme — picker on Settings → Preferences",
            &[
                "theme",
                "dark",
                "light",
                "system",
                "appearance",
                "preferences",
            ],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Preferences),
            "Search Hotkey",
            "Keyboard shortcut to open settings search",
            &["hotkey", "shortcut", "search", "keybinding", "preferences"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::ConfigEditor),
            "Config Editor",
            "Edit the raw KDL configuration files",
            &["kdl", "editor", "raw", "file", "config"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Backups),
            "Create Backup",
            "Save a snapshot of your current configuration",
            &["backup", "snapshot", "save", "create"],
        ),
        e(
            SearchDestination::Gear(GearSubTab::Backups),
            "Restore Backup",
            "Recover a previous configuration snapshot",
            &["backup", "restore", "recovery", "undo", "snapshot"],
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
        assert!(results.iter().any(|r| r.setting_name.contains("Focus")));
    }

    #[test]
    fn test_search_speed() {
        let index = SearchIndex::new();
        let results = index.search("speed");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_empty() {
        let index = SearchIndex::new();
        let results = index.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_every_section_and_device_has_entries() {
        for section in EditableSection::ALL {
            assert!(
                !section_entries(*section).is_empty(),
                "section {:?} has no search entries",
                section
            );
        }
        for device in EditableDevice::ALL {
            assert!(
                !device_entries(*device).is_empty(),
                "device {:?} has no search entries",
                device
            );
        }
    }

    fn search_targets(query: &str, dest: SearchDestination) -> bool {
        let index = SearchIndex::new();
        index.search(query).iter().any(|r| r.destination == dest)
    }

    #[test]
    fn test_search_trackball() {
        assert!(search_targets(
            "trackball",
            SearchDestination::Device(EditableDevice::Trackball)
        ));
    }

    #[test]
    fn test_search_tablet() {
        assert!(search_targets(
            "tablet",
            SearchDestination::Device(EditableDevice::Tablet)
        ));
    }

    #[test]
    fn test_search_calibration() {
        let index = SearchIndex::new();
        let results = index.search("calibration");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| matches!(
            r.destination,
            SearchDestination::Device(EditableDevice::Tablet)
                | SearchDestination::Device(EditableDevice::Touch)
        )));
    }

    #[test]
    fn test_search_backup() {
        assert!(search_targets(
            "backup",
            SearchDestination::Gear(GearSubTab::Backups)
        ));
    }

    #[test]
    fn test_zoom_hits_overview_not_display_scale() {
        let index = SearchIndex::new();
        let results = index.search("zoom");
        assert!(!results.is_empty());
        assert_eq!(
            results[0].destination,
            SearchDestination::Section(EditableSection::Overview)
        );
        assert!(
            !results
                .iter()
                .any(|r| r.destination == SearchDestination::Displays),
            "display scale must not steal 'zoom'"
        );
    }

    #[test]
    fn test_app_theme_lands_on_preferences() {
        assert!(search_targets(
            "app theme",
            SearchDestination::Gear(GearSubTab::Preferences)
        ));
        let index = SearchIndex::new();
        let results = index.search("theme");
        assert!(
            results
                .iter()
                .any(|r| r.destination == SearchDestination::Gear(GearSubTab::Preferences)),
            "theme should reach Preferences"
        );
    }

    #[test]
    fn test_overview_backdrop_and_presets() {
        assert!(search_targets(
            "overview backdrop",
            SearchDestination::Section(EditableSection::Overview)
        ));
        assert!(search_targets(
            "preset",
            SearchDestination::Section(EditableSection::PresetSizes)
        ));
        assert!(search_targets(
            "workspace background",
            SearchDestination::Section(EditableSection::WorkspaceBackground)
        ));
    }

    #[test]
    fn test_destination_screens() {
        assert_eq!(
            SearchDestination::Section(EditableSection::Overview).screen(),
            Screen::Visuals
        );
        assert_eq!(
            SearchDestination::Section(EditableSection::WorkspaceBackground).screen(),
            Screen::Visuals
        );
        assert_eq!(
            SearchDestination::Section(EditableSection::PresetSizes).screen(),
            Screen::Layout
        );
        assert_eq!(
            SearchDestination::Section(EditableSection::FocusRing).screen(),
            Screen::Visuals
        );
        assert_eq!(
            SearchDestination::Device(EditableDevice::Mouse).screen(),
            Screen::Input
        );
        assert_eq!(
            SearchDestination::Gear(GearSubTab::Preferences).screen(),
            Screen::Gear
        );
    }

    #[test]
    fn test_selected_index_cap() {
        assert_eq!(clamp_selected_index(19, 20), MAX_VISIBLE_RESULTS - 1);
        assert_eq!(clamp_selected_index(3, 20), 3);
        assert_eq!(clamp_selected_index(5, 0), 0);
    }
}
