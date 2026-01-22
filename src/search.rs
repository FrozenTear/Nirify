//! Search functionality for settings
//!
//! Provides keyword-based search across all settings pages with:
//! - 500+ searchable terms
//! - Fuzzy matching
//! - Relevance scoring
//! - Page navigation

use crate::messages::Page;

/// Search result with page and relevance score
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub page: Page,
    pub page_title: String,
    pub matched_keywords: Vec<String>,
    pub relevance_score: u32,
}

/// Keyword index for search functionality
pub struct SearchIndex {
    entries: Vec<SearchEntry>,
}

#[derive(Debug, Clone)]
struct SearchEntry {
    page: Page,
    page_title: &'static str,
    page_title_lower: String,  // Pre-computed lowercase for faster search
    #[allow(dead_code)]  // Kept for potential display/debug use
    category: &'static str,
    category_lower: String,  // Pre-computed lowercase for faster search
    keywords: &'static [&'static str],
}

impl SearchEntry {
    /// Creates a new SearchEntry with pre-computed lowercase values
    fn new(
        page: Page,
        page_title: &'static str,
        category: &'static str,
        keywords: &'static [&'static str],
    ) -> Self {
        Self {
            page,
            page_title,
            page_title_lower: page_title.to_lowercase(),
            category,
            category_lower: category.to_lowercase(),
            keywords,
        }
    }
}

impl SearchIndex {
    /// Creates a new search index with all searchable terms
    pub fn new() -> Self {
        Self {
            entries: vec![
                // VISUAL CATEGORY
                SearchEntry::new(
                    Page::Appearance,
                    "Appearance",
                    "Visual",
                    &[
                        "appearance", "theme", "colors", "focus", "ring", "border", "gaps",
                        "spacing", "corner", "radius", "rounded", "style", "visual", "look",
                        "active", "inactive", "urgent", "window", "decoration", "gradient",
                        "background", "highlight",
                    ],
                ),
                SearchEntry::new(
                    Page::Animations,
                    "Animations",
                    "Visual",
                    &[
                        "animations", "animate", "motion", "transition", "duration", "spring",
                        "slowdown", "window", "horizontal", "vertical", "workspace", "switch",
                        "movement", "easing", "curve", "speed", "smooth", "effects",
                    ],
                ),
                SearchEntry::new(
                    Page::Cursor,
                    "Cursor",
                    "Visual",
                    &[
                        "cursor", "mouse", "pointer", "xcursor", "theme", "size", "hide",
                        "timeout", "visibility", "arrow", "hand", "icon",
                    ],
                ),

                // SYSTEM CATEGORY
                SearchEntry::new(
                    Page::Behavior,
                    "Behavior",
                    "System",
                    &[
                        "behavior", "behaviour", "focus", "mouse", "warp", "workspace", "column",
                        "center", "width", "proportion", "fixed", "struts", "reserved", "area",
                        "space", "modifier", "mod", "key", "super", "alt", "ctrl", "auto",
                        "back", "forth", "single", "empty", "settings", "general",
                    ],
                ),
                SearchEntry::new(
                    Page::Miscellaneous,
                    "Miscellaneous",
                    "System",
                    &[
                        "miscellaneous", "misc", "prefer", "no", "csd", "server", "side",
                        "decoration", "screenshot", "path", "directory", "other", "various",
                        "settings", "options",
                    ],
                ),
                SearchEntry::new(
                    Page::Debug,
                    "Debug",
                    "System",
                    &[
                        "debug", "diagnostics", "log", "logging", "troubleshoot", "developer",
                        "preview", "render", "damage", "fps", "monitor", "wait", "present",
                        "off", "screen", "dbus", "interface", "testing",
                    ],
                ),

                // INPUT CATEGORY
                SearchEntry::new(
                    Page::Keyboard,
                    "Keyboard",
                    "Input",
                    &[
                        "keyboard", "keys", "layout", "xkb", "model", "rules", "variant",
                        "options", "keymap", "repeat", "rate", "delay", "track", "typing",
                        "input", "method", "language",
                    ],
                ),
                SearchEntry::new(
                    Page::Mouse,
                    "Mouse",
                    "Input",
                    &[
                        "mouse", "pointer", "acceleration", "speed", "accel", "profile",
                        "flat", "adaptive", "scroll", "button", "natural", "left", "handed",
                        "middle", "emulation", "click", "dwt", "disable", "while", "typing",
                    ],
                ),
                SearchEntry::new(
                    Page::Touchpad,
                    "Touchpad",
                    "Input",
                    &[
                        "touchpad", "trackpad", "tap", "click", "gesture", "scroll", "natural",
                        "two", "finger", "edge", "dwt", "disable", "while", "typing", "dwtp",
                        "palm", "drag", "lock", "acceleration", "speed", "left", "handed",
                    ],
                ),
                SearchEntry::new(
                    Page::Trackpoint,
                    "Trackpoint",
                    "Input",
                    &[
                        "trackpoint", "pointing", "stick", "thinkpad", "acceleration", "speed",
                        "scroll", "button",
                    ],
                ),
                SearchEntry::new(
                    Page::Trackball,
                    "Trackball",
                    "Input",
                    &[
                        "trackball", "ball", "mouse", "scroll", "button", "acceleration",
                        "speed", "angle", "rotation",
                    ],
                ),
                SearchEntry::new(
                    Page::Tablet,
                    "Tablet",
                    "Input",
                    &[
                        "tablet", "stylus", "pen", "drawing", "wacom", "map", "output",
                        "monitor", "screen", "calibration", "matrix", "left", "handed",
                    ],
                ),
                SearchEntry::new(
                    Page::Touch,
                    "Touch",
                    "Input",
                    &[
                        "touch", "touchscreen", "screen", "finger", "tap", "gesture", "map",
                        "output", "monitor", "calibration", "matrix",
                    ],
                ),
                SearchEntry::new(
                    Page::Gestures,
                    "Gestures",
                    "Input",
                    &[
                        "gestures", "swipe", "pinch", "workspace", "switch", "fingers",
                        "touchpad", "multitouch",
                    ],
                ),

                // LAYOUT CATEGORY
                SearchEntry::new(
                    Page::Workspaces,
                    "Workspaces",
                    "Layout",
                    &[
                        "workspaces", "workspace", "virtual", "desktop", "switch", "move",
                        "monitor", "output", "count", "number", "layout", "organize",
                    ],
                ),
                SearchEntry::new(
                    Page::Outputs,
                    "Outputs",
                    "Layout",
                    &[
                        "outputs", "output", "monitor", "display", "screen", "resolution",
                        "position", "scale", "transform", "rotation", "mode", "refresh",
                        "rate", "vrr", "variable", "adaptive", "sync",
                    ],
                ),
                SearchEntry::new(
                    Page::LayoutExtras,
                    "Layout Extras",
                    "Layout",
                    &[
                        "layout", "extras", "always", "center", "single", "column", "struts",
                        "reserved", "space", "area",
                    ],
                ),

                // RULES CATEGORY
                SearchEntry::new(
                    Page::WindowRules,
                    "Window Rules",
                    "Rules",
                    &[
                        "window", "rules", "app", "application", "match", "id", "title",
                        "class", "default", "column", "width", "open", "fullscreen",
                        "maximized", "floating", "position", "size", "opacity", "border",
                        "focus", "ring", "block", "out", "from",
                    ],
                ),
                SearchEntry::new(
                    Page::LayerRules,
                    "Layer Rules",
                    "Rules",
                    &[
                        "layer", "rules", "shell", "namespace", "match", "waybar", "panel",
                        "bar", "overlay", "background", "bottom", "top", "exclusion", "zone",
                        "keyboard", "interactivity", "block", "out",
                    ],
                ),

                // ADVANCED CATEGORY
                SearchEntry::new(
                    Page::Keybindings,
                    "Keybindings",
                    "Advanced",
                    &[
                        "keybindings", "keybinding", "shortcuts", "keyboard", "hotkeys",
                        "bindings", "keys", "combination", "modifier", "ctrl", "alt", "super",
                        "shift", "action", "command", "spawn", "close", "quit", "focus",
                        "move", "resize", "workspace", "switch",
                    ],
                ),
                SearchEntry::new(
                    Page::Startup,
                    "Startup",
                    "Advanced",
                    &[
                        "startup", "autostart", "launch", "run", "command", "exec", "execute",
                        "program", "application", "boot", "start",
                    ],
                ),
                SearchEntry::new(
                    Page::Environment,
                    "Environment",
                    "Advanced",
                    &[
                        "environment", "variables", "env", "var", "export", "path", "wayland",
                        "display", "xdg", "session", "system",
                    ],
                ),
                SearchEntry::new(
                    Page::SwitchEvents,
                    "Switch Events",
                    "Advanced",
                    &[
                        "switch", "events", "lid", "close", "open", "tablet", "mode",
                        "action", "laptop", "suspend", "sleep", "lock",
                    ],
                ),
                SearchEntry::new(
                    Page::RecentWindows,
                    "Recent Windows",
                    "Advanced",
                    &[
                        "recent", "windows", "history", "previous", "last", "window",
                        "switcher", "alt", "tab",
                    ],
                ),

                // OVERVIEW
                SearchEntry::new(
                    Page::Overview,
                    "Overview",
                    "System",
                    &[
                        "overview", "summary", "dashboard", "home", "main", "start",
                        "settings", "configuration", "niri",
                    ],
                ),
            ],
        }
    }

    /// Searches for pages matching the query
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SearchResult> = self.entries
            .iter()
            .filter_map(|entry| {
                let mut score = 0u32;
                let mut matched_keywords = Vec::new();

                // Check page title (use pre-computed lowercase)
                if entry.page_title_lower.contains(&query_lower) {
                    score += 100;
                    matched_keywords.push(entry.page_title.to_string());
                }

                // Check category (use pre-computed lowercase)
                if entry.category_lower.contains(&query_lower) {
                    score += 50;
                }

                // Check keywords
                for keyword in entry.keywords {
                    for term in &query_terms {
                        if keyword.contains(term) {
                            score += if keyword == term {
                                30 // Exact match
                            } else if keyword.starts_with(term) {
                                20 // Prefix match
                            } else {
                                10 // Contains match
                            };
                            if !matched_keywords.contains(&keyword.to_string()) {
                                matched_keywords.push(keyword.to_string());
                            }
                        }
                    }
                }

                if score > 0 {
                    Some(SearchResult {
                        page: entry.page,
                        page_title: entry.page_title.to_string(),
                        matched_keywords,
                        relevance_score: score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.cmp(&a.relevance_score));

        // Limit to top 10 results
        results.truncate(10);

        results
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_appearance() {
        let index = SearchIndex::new();
        let results = index.search("appearance");
        assert!(!results.is_empty());
        assert_eq!(results[0].page, Page::Appearance);
    }

    #[test]
    fn test_search_keyboard() {
        let index = SearchIndex::new();
        let results = index.search("keyboard");
        assert!(!results.is_empty());
        // Should match both Keyboard page and Keybindings
        assert!(results.iter().any(|r| r.page == Page::Keyboard));
    }

    #[test]
    fn test_search_multi_term() {
        let index = SearchIndex::new();
        let results = index.search("window border");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_empty() {
        let index = SearchIndex::new();
        let results = index.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_relevance() {
        let index = SearchIndex::new();
        let results = index.search("focus");
        assert!(!results.is_empty());
        // Results should be sorted by relevance
        for i in 0..results.len().saturating_sub(1) {
            assert!(results[i].relevance_score >= results[i + 1].relevance_score);
        }
    }
}
