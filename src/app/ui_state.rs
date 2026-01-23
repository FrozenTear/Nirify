//! UI State - separated from domain state for cleaner architecture
//!
//! This module contains all UI-only state that doesn't affect the saved settings.
//! Separating UI state from domain state makes it easier to:
//! - Reason about what state affects saved config vs. just UI display
//! - Test business logic without UI concerns
//! - Potentially serialize/restore UI state separately

use std::collections::HashMap;

use iced::widget::text_editor;

use crate::messages::{DialogState, Page};
use crate::views;

/// UI-only state that doesn't affect saved settings
#[derive(Default)]
pub struct UiState {
    // Navigation & Display
    /// Current active page
    pub current_page: Page,
    /// Search query
    pub search_query: String,
    /// Search results
    pub search_results: Vec<crate::search::SearchResult>,
    /// Last search timestamp for debouncing
    pub last_search_time: Option<std::time::Instant>,
    /// Whether sidebar is expanded (for responsive design)
    pub sidebar_expanded: bool,
    /// Widget demo state for testing
    pub widget_demo_state: views::widget_demo::DemoState,
    /// Toast notification message
    pub toast: Option<String>,
    /// When the toast was shown (for auto-clear)
    pub toast_shown_at: Option<std::time::Instant>,
    /// Active modal dialog (if any)
    pub dialog_state: DialogState,
    /// Current theme
    pub current_theme: crate::theme::AppTheme,
    /// Niri compositor connection status
    pub niri_status: crate::views::status_bar::NiriStatus,

    // Outputs state
    /// Selected output index for list-detail view
    pub selected_output_index: Option<usize>,
    /// Expanded sections in outputs view
    pub output_sections_expanded: HashMap<String, bool>,

    // Layer Rules state
    /// Selected layer rule ID for list-detail view
    pub selected_layer_rule_id: Option<u32>,
    /// Expanded sections in layer rules view
    pub layer_rule_sections_expanded: HashMap<(u32, String), bool>,
    /// Regex validation errors
    pub layer_rule_regex_errors: HashMap<(u32, String), String>,

    // Window Rules state
    /// Selected window rule ID for list-detail view
    pub selected_window_rule_id: Option<u32>,
    /// Expanded sections in window rules view
    pub window_rule_sections_expanded: HashMap<(u32, String), bool>,
    /// Regex validation errors
    pub window_rule_regex_errors: HashMap<(u32, String), String>,

    // Keybindings state
    /// Selected keybinding index for list-detail view
    pub selected_keybinding_index: Option<usize>,
    /// Expanded sections in keybindings view
    pub keybinding_sections_expanded: HashMap<String, bool>,
    /// Which keybinding is currently capturing key input
    pub key_capture_active: Option<usize>,

    // Calibration matrix caches
    /// Cached formatted values for tablet calibration matrix
    pub tablet_calibration_cache: [String; 6],
    /// Cached formatted values for touch calibration matrix
    pub touch_calibration_cache: [String; 6],

    // Page-specific state
    /// State for the Tools page
    pub tools_state: views::tools::ToolsState,
    /// State for the Config Editor page
    pub config_editor_state: views::config_editor::ConfigEditorState,
    /// Text editor content for Config Editor (stored here because Content isn't Clone)
    pub config_editor_content: text_editor::Content,
    /// State for the Backups page
    pub backups_state: views::backups::BackupsState,
    /// Pending restore index (for confirmation dialog)
    pub pending_restore_idx: Option<usize>,
    /// Consolidation suggestions for the first-run wizard
    pub wizard_suggestions: Vec<crate::messages::ConsolidationSuggestion>,
}

impl UiState {
    /// Create new UiState with initial values
    pub fn new(
        current_theme: crate::theme::AppTheme,
        tablet_calibration_cache: [String; 6],
        touch_calibration_cache: [String; 6],
    ) -> Self {
        Self {
            current_theme,
            tablet_calibration_cache,
            touch_calibration_cache,
            sidebar_expanded: true,
            ..Default::default()
        }
    }
}
