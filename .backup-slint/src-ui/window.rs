// Window state management

use crate::types::NavigationCategory;

/// Current state of the settings window
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Currently selected category in sidebar
    pub selected_category: NavigationCategory,

    /// Search query (empty = no filter)
    pub search_query: String,

    /// Whether the advanced section is expanded
    pub advanced_expanded: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            selected_category: NavigationCategory::Appearance,
            search_query: String::new(),
            advanced_expanded: false,
        }
    }
}

impl WindowState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_category(&mut self, category: NavigationCategory) {
        self.selected_category = category;
    }

    pub fn set_search(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn toggle_advanced(&mut self) {
        self.advanced_expanded = !self.advanced_expanded;
    }
}
