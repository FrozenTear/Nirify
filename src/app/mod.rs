//! Main application module - Elm Architecture implementation
//!
//! This module implements the core Application logic:
//! - State management (App struct)
//! - Message handling (update function)
//! - UI construction (view function)

mod handlers;
mod helpers;
mod ui_state;

pub use ui_state::UiState;

use std::sync::Arc;
use std::time::Duration;

use iced::time;
use iced::widget::{column, container, stack, text};
use iced::{alignment::Horizontal, Element, Length, Subscription, Task};

use crate::config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::messages::{DialogState, Message, Page, SaveMessage};
use crate::save_manager::{ReloadResult, SaveResult};
use crate::theme::fonts;
use crate::views;

/// Groups save-related state for cleaner App organization
pub struct SaveState {
    /// Tracks which settings categories have unsaved changes
    pub dirty_tracker: DirtyTracker,
    /// Last time a change was made (for debounced save)
    pub last_change_time: Option<std::time::Instant>,
    /// Whether a save is currently in progress
    pub in_progress: bool,
}

impl SaveState {
    fn new() -> Self {
        Self {
            dirty_tracker: DirtyTracker::new(),
            last_change_time: None,
            in_progress: false,
        }
    }
}

/// Main application state
pub struct App {
    /// Settings - direct ownership (no mutex needed, iced is single-threaded)
    settings: Settings,

    /// Config paths for loading/saving
    paths: Arc<ConfigPaths>,

    /// Save subsystem state (dirty tracking, debounce timing)
    save: SaveState,

    /// Search index (domain data, not UI state)
    search_index: crate::search::SearchIndex,

    /// UI-only state (selections, expansions, dialogs, etc.)
    ui: UiState,
}

impl App {
    /// Creates a new App instance
    pub fn new() -> (Self, Task<Message>) {
        // Load config paths
        let paths = match ConfigPaths::new() {
            Ok(paths) => Arc::new(paths),
            Err(e) => {
                log::error!("Failed to initialize config paths: {}", e);
                return Self::new_with_error(
                    "Could not determine configuration directory. \
                     Please ensure your system has a valid XDG config directory."
                        .to_string(),
                    Some(e.to_string()),
                );
            }
        };

        // Migrate old tilde-based include paths to relative paths
        // This fixes configs created before the XDG_CONFIG_HOME fix
        if let Err(e) = paths.migrate_include_line() {
            log::warn!("Failed to migrate include line: {}", e);
        }

        // Ensure config.kdl is properly set up with include directive
        // This replaces managed nodes with the include, preserving custom content
        // Safe to call every time - it early-returns if no changes needed
        if paths.niri_config.exists() && paths.managed_dir.exists() {
            match crate::config::smart_replace_config(&paths.niri_config, &paths.backup_dir) {
                Ok(result) => {
                    if result.replaced_count > 0 || result.include_added {
                        log::info!(
                            "Config updated: {} managed nodes replaced, {} preserved, include added: {}",
                            result.replaced_count,
                            result.preserved_count,
                            result.include_added
                        );
                    }
                }
                Err(e) => {
                    log::warn!("Failed to update config.kdl: {}", e);
                }
            }
        }

        // Clean up old backups to prevent directory from growing indefinitely
        // Keep the 10 most recent backups
        if let Err(e) = paths.cleanup_old_backups(10) {
            log::warn!("Failed to clean up old backups: {}", e);
        }

        // Load settings from disk (load_settings returns Settings, not Result)
        let settings = crate::config::load_settings(&paths);
        log::info!("Settings loaded successfully");

        // Parse theme from settings
        let current_theme = settings
            .preferences
            .theme
            .parse::<crate::theme::AppTheme>()
            .unwrap_or_default();

        // Initialize calibration matrix caches from settings
        let tablet_calibration_cache =
            crate::views::widgets::format_matrix_values(settings.tablet.calibration_matrix);
        let touch_calibration_cache =
            crate::views::widgets::format_matrix_values(settings.touch.calibration_matrix);

        // Check initial niri connection status and version
        let (niri_status, niri_version) = if crate::ipc::is_niri_running() {
            let version = crate::ipc::get_version()
                .ok()
                .and_then(|v| crate::version::NiriVersion::parse(&v));
            if let Some(v) = version {
                log::info!("Detected niri version: {}", v);
            }
            (crate::views::status_bar::NiriStatus::Connected, version)
        } else {
            (crate::views::status_bar::NiriStatus::Disconnected, None)
        };

        // Determine feature compatibility based on niri version
        let feature_compat = crate::version::FeatureCompat::from_version(niri_version);
        if !feature_compat.recent_windows {
            log::info!(
                "Recent windows feature disabled (requires niri 25.11+, detected: {})",
                niri_version.map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string())
            );
        }

        // Create UI state
        let mut ui = UiState::new(
            current_theme,
            tablet_calibration_cache,
            touch_calibration_cache,
        );
        ui.niri_status = niri_status;
        ui.niri_version = niri_version;
        ui.feature_compat = feature_compat;
        ui.show_search_bar = settings.preferences.show_search_bar;

        // Check if this is the first run and show the wizard
        if paths.is_first_run() {
            log::info!("First run detected - showing setup wizard");
            ui.dialog_state = DialogState::FirstRunWizard {
                step: crate::messages::WizardStep::Welcome,
            };
        }

        let app = Self {
            settings,
            paths,
            save: SaveState::new(),
            search_index: crate::search::SearchIndex::new(),
            ui,
        };

        (app, Task::none())
    }

    /// Creates an App in error state for displaying initialization failures.
    ///
    /// This allows the app to show a user-friendly error dialog instead of
    /// panicking when initialization fails.
    fn new_with_error(error_message: String, details: Option<String>) -> (Self, Task<Message>) {
        let settings = Settings::default();
        let mut ui = UiState::new(
            crate::theme::AppTheme::default(),
            crate::views::widgets::format_matrix_values(None),
            crate::views::widgets::format_matrix_values(None),
        );

        ui.dialog_state = DialogState::Error {
            title: "Initialization Failed".to_string(),
            message: error_message,
            details,
        };

        let paths = Arc::new(ConfigPaths::default());

        let app = Self {
            settings,
            paths,
            save: SaveState::new(),
            search_index: crate::search::SearchIndex::new(),
            ui,
        };

        (app, Task::none())
    }

    /// Updates application state based on messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateToPage(page) => {
                self.ui.current_page = page;
                // Clear any search highlight when navigating manually
                self.ui.highlight_setting = None;
                let is_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );

                // Auto-refresh IPC outputs when navigating to Outputs page
                if page == Page::Outputs && is_connected {
                    return Task::perform(
                        async { crate::ipc::get_full_outputs().map_err(|e| e.to_string()) },
                        |result| {
                            Message::Tools(crate::messages::ToolsMessage::OutputsLoaded(result))
                        },
                    );
                }

                // Auto-refresh workspaces when navigating to Window Rules page
                // (for the workspace dropdown)
                if page == Page::WindowRules && is_connected {
                    return Task::perform(
                        async { crate::ipc::get_workspaces().map_err(|e| e.to_string()) },
                        |result| {
                            Message::Tools(crate::messages::ToolsMessage::WorkspacesLoaded(result))
                        },
                    );
                }

                Task::none()
            }

            Message::ToggleSidebar => {
                self.ui.sidebar_expanded = !self.ui.sidebar_expanded;
                Task::none()
            }

            // Search (Phase 9)
            Message::SearchQueryChanged(query) => {
                self.ui.search_query = query;
                self.ui.last_search_time = Some(std::time::Instant::now());

                // Perform search immediately
                self.ui.search_results = self.search_index.search(&self.ui.search_query);

                // Re-focus search input to maintain typing focus
                iced::widget::operation::focus(views::navigation::search_input_id())
            }

            Message::SearchResultSelected(index) => {
                // Navigate to the selected page
                if let Some(result) = self.ui.search_results.get(index) {
                    self.ui.current_page = result.page;
                    // Store setting name for highlighting on the target page
                    self.ui.highlight_setting = Some(result.setting_name.clone());
                    // Clear search after navigation
                    self.ui.search_query.clear();
                    self.ui.search_results.clear();
                    // Close search modal if open
                    self.ui.search_focused = false;
                }
                Task::none()
            }

            Message::ClearSearch => {
                self.ui.search_query.clear();
                self.ui.search_results.clear();
                self.ui.last_search_time = None;
                Task::none()
            }

            Message::ToggleSearch => {
                // If search bar is visible, just focus it
                // If hidden, show it as focused (modal mode)
                self.ui.search_focused = !self.ui.search_focused;
                if self.ui.search_focused {
                    // Focus the search input
                    iced::widget::operation::focus(views::navigation::search_input_id())
                } else {
                    // Clear search when closing
                    self.ui.search_query.clear();
                    self.ui.search_results.clear();
                    Task::none()
                }
            }

            // Theme
            Message::ChangeTheme(theme) => {
                self.ui.current_theme = theme;

                // Save theme to settings (direct access, no mutex needed)
                self.settings.preferences.theme = theme.to_str().to_string();

                // Mark preferences as dirty for auto-save
                self.save.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

                Task::none()
            }

            // System theme events from portal or file watcher
            Message::SystemThemeEvent(event) => {
                self.ui.system_theme_state.handle_event(event);
                Task::none()
            }

            // Settings category messages
            Message::Appearance(msg) => self.update_appearance(msg),

            Message::Behavior(msg) => self.update_behavior(msg),

            Message::Keyboard(msg) => self.update_keyboard(msg),

            Message::Mouse(msg) => self.update_mouse(msg),

            Message::Touchpad(msg) => self.update_touchpad(msg),

            Message::Animations(msg) => self.update_animations(msg),

            Message::Cursor(msg) => self.update_cursor(msg),

            Message::Workspaces(msg) => self.update_workspaces(msg),

            Message::WindowRules(msg) => self.update_window_rules(msg),

            Message::Keybindings(msg) => self.update_keybindings(msg),

            Message::LayerRules(msg) => self.update_layer_rules(msg),

            Message::Outputs(msg) => self.update_outputs(msg),

            Message::Overview(msg) => self.update_overview(msg),

            // Save subsystem (Phase 4)
            Message::Save(SaveMessage::CheckSave) => {
                if self.should_save() {
                    // Trigger async save
                    self.save_task()
                } else {
                    Task::none()
                }
            }

            Message::SaveCompleted(result) => {
                self.save.in_progress = false;
                match result {
                    SaveResult::Success { files_written, .. } => {
                        self.ui.toast = Some(format!("Saved {} file(s)", files_written));
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                        // Trigger niri config reload
                        self.reload_niri_config_task()
                    }
                    SaveResult::Error { message } => {
                        self.ui.toast = Some(format!("Save failed: {}", message));
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                        Task::none()
                    }
                    SaveResult::NothingToSave => Task::none(),
                }
            }

            Message::ClearToast => {
                // Only clear if toast has been shown for at least 3 seconds
                if let Some(shown_at) = self.ui.toast_shown_at {
                    if shown_at.elapsed() >= std::time::Duration::from_secs(3) {
                        self.ui.toast = None;
                        self.ui.toast_shown_at = None;
                    }
                }
                Task::none()
            }

            Message::ReloadCompleted(result) => {
                match result {
                    ReloadResult::Success => {
                        log::info!("Niri config reloaded");
                    }
                    ReloadResult::Error { message } => {
                        log::warn!("Failed to reload niri config: {}", message);
                        // Don't show error to user - niri might not be running
                    }
                }
                Task::none()
            }

            // Dialogs (Phase 8)
            Message::ShowDialog(dialog_state) => {
                self.ui.dialog_state = dialog_state;
                Task::none()
            }

            Message::CloseDialog => {
                // If this was an initialization error dialog, exit the app gracefully
                if let DialogState::Error { title, .. } = &self.ui.dialog_state {
                    if title == "Initialization Failed" {
                        log::info!("User acknowledged initialization failure, exiting");

                        // Clean up temp fallback directory if it exists
                        let temp_fallback = std::env::temp_dir().join("nirify-error-fallback");
                        if temp_fallback.exists() {
                            if let Err(e) = std::fs::remove_dir_all(&temp_fallback) {
                                log::warn!("Failed to clean up temp fallback directory: {}", e);
                            } else {
                                log::debug!("Cleaned up temp fallback directory");
                            }
                        }

                        return iced::exit();
                    }
                }
                self.ui.dialog_state = DialogState::None;
                Task::none()
            }

            Message::DialogConfirm => {
                // Handle confirmation based on current dialog type
                match &self.ui.dialog_state {
                    DialogState::Confirm { on_confirm, .. } => {
                        use crate::messages::ConfirmAction;
                        match on_confirm {
                            ConfirmAction::DeleteRule(rule_id) => {
                                // Try to delete from window rules first, then layer rules
                                let rule_id = *rule_id;
                                if self.settings.window_rules.remove(rule_id) {
                                    log::info!("Deleted window rule {}", rule_id);
                                    if self.ui.selected_window_rule_id == Some(rule_id) {
                                        self.ui.selected_window_rule_id =
                                            self.settings.window_rules.rules.first().map(|r| r.id);
                                    }
                                    self.save.dirty_tracker
                                        .mark(crate::config::SettingsCategory::WindowRules);
                                } else if self.settings.layer_rules.remove(rule_id) {
                                    log::info!("Deleted layer rule {}", rule_id);
                                    if self.ui.selected_layer_rule_id == Some(rule_id) {
                                        self.ui.selected_layer_rule_id =
                                            self.settings.layer_rules.rules.first().map(|r| r.id);
                                    }
                                    self.save.dirty_tracker
                                        .mark(crate::config::SettingsCategory::LayerRules);
                                }
                                self.mark_changed();
                            }
                            ConfirmAction::ResetSettings => {
                                log::info!("Resetting all settings to defaults");
                                self.settings = crate::config::models::Settings::default();
                                self.save.dirty_tracker.mark_all();
                                self.mark_changed();
                            }
                            ConfirmAction::ClearAllKeybindings => {
                                log::info!("Clearing all keybindings");
                                self.settings.keybindings.bindings.clear();
                                self.save.dirty_tracker
                                    .mark(crate::config::SettingsCategory::Keybindings);
                                self.mark_changed();
                            }
                        }
                    }
                    DialogState::DiffView { .. } => {
                        // For diff view, we don't have specific state to apply
                        // The calling code should handle this via a specific message
                        log::info!("Diff view confirmed - closing dialog");
                    }
                    _ => {
                        log::warn!("DialogConfirm called on non-confirm dialog");
                    }
                }
                self.ui.dialog_state = DialogState::None;
                Task::none()
            }

            Message::WizardNext => {
                // Progress wizard to next step
                if let DialogState::FirstRunWizard { step } = &self.ui.dialog_state {
                    use crate::messages::WizardStep;
                    let next_step = match step {
                        WizardStep::Welcome => WizardStep::ConfigSetup,
                        WizardStep::ConfigSetup => WizardStep::ImportResults,
                        WizardStep::ImportResults => {
                            // Check if there are consolidation suggestions
                            if !self.ui.wizard_suggestions.is_empty() {
                                WizardStep::Consolidation
                            } else {
                                WizardStep::Complete
                            }
                        }
                        WizardStep::Consolidation => WizardStep::Complete,
                        WizardStep::Complete => {
                            self.ui.dialog_state = DialogState::None;
                            return Task::none();
                        }
                    };
                    self.ui.dialog_state = DialogState::FirstRunWizard { step: next_step };
                }
                Task::none()
            }

            Message::WizardBack => {
                // Go back to previous wizard step
                if let DialogState::FirstRunWizard { step } = &self.ui.dialog_state {
                    use crate::messages::WizardStep;
                    let prev_step = match step {
                        WizardStep::Welcome => {
                            self.ui.dialog_state = DialogState::None;
                            return Task::none();
                        }
                        WizardStep::ConfigSetup => WizardStep::Welcome,
                        WizardStep::ImportResults => WizardStep::ConfigSetup,
                        WizardStep::Consolidation => WizardStep::ImportResults,
                        WizardStep::Complete => {
                            // Go back to Consolidation if there are suggestions, otherwise ImportResults
                            if !self.ui.wizard_suggestions.is_empty() {
                                WizardStep::Consolidation
                            } else {
                                WizardStep::ImportResults
                            }
                        }
                    };
                    self.ui.dialog_state = DialogState::FirstRunWizard { step: prev_step };
                }
                Task::none()
            }

            Message::WizardSetupConfig => {
                // Set up the config: create directories and add include line
                log::info!("Wizard: Setting up config...");

                // Ensure directories exist
                if let Err(e) = self.paths.ensure_directories() {
                    log::error!("Failed to create directories: {}", e);
                    self.ui.dialog_state = DialogState::Error {
                        title: "Setup Error".to_string(),
                        message: "Failed to create configuration directories.".to_string(),
                        details: Some(e.to_string()),
                    };
                    return Task::none();
                }

                // Create main.kdl immediately (before adding include line)
                // This ensures niri can load the config even if the app crashes
                // before the debounced save completes
                if !self.paths.main_kdl.exists() {
                    let main_kdl_content = crate::config::storage::generate_main_kdl(self.ui.feature_compat);
                    if let Err(e) = crate::config::storage::atomic_write(&self.paths.main_kdl, &main_kdl_content) {
                        log::error!("Failed to create main.kdl: {}", e);
                        self.ui.dialog_state = DialogState::Error {
                            title: "Setup Error".to_string(),
                            message: "Failed to create main.kdl configuration file.".to_string(),
                            details: Some(e.to_string()),
                        };
                        return Task::none();
                    }
                    log::info!("Created main.kdl");
                }

                // Replace user's config.kdl with our managed version
                // This removes managed nodes and adds the include directive
                match crate::config::smart_replace_config(&self.paths.niri_config, &self.paths.backup_dir) {
                    Ok(result) => {
                        log::info!(
                            "Smart replace complete: {} nodes replaced, {} preserved, backup at {:?}",
                            result.replaced_count,
                            result.preserved_count,
                            result.backup_path
                        );
                        for warning in &result.warnings {
                            log::warn!("Smart replace warning: {}", warning);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to replace config: {}", e);
                        self.ui.dialog_state = DialogState::Error {
                            title: "Setup Error".to_string(),
                            message: "Failed to set up config.kdl.".to_string(),
                            details: Some(e.to_string()),
                        };
                        return Task::none();
                    }
                }

                // Trigger initial save to create all config files
                // Note: This is safe from race conditions because:
                // 1. iced is single-threaded - this handler completes atomically
                // 2. SaveManager uses 300ms debounce before actually saving
                // 3. We've already created main.kdl and config.kdl above
                self.save.dirty_tracker.mark_all();
                self.mark_changed();

                log::info!("Wizard: Config setup complete");

                // Analyze rules for consolidation opportunities
                let analysis = crate::config::analyze_rules(
                    &self.settings.window_rules.rules,
                    &self.settings.layer_rules.rules,
                );

                // Store suggestions for wizard consolidation step
                self.ui.wizard_suggestions.clear();
                if analysis.has_suggestions() {
                    for s in &analysis.window_suggestions {
                        self.ui
                            .wizard_suggestions
                            .push(crate::messages::ConsolidationSuggestion {
                                description: s.description.clone(),
                                rule_ids: s.rule_ids.clone(),
                                rule_count: s.rule_ids.len(),
                                patterns: s.patterns.clone(),
                                merged_pattern: s.merged_pattern.clone(),
                                is_window_rule: true,
                                selected: true, // Pre-select in wizard
                            });
                    }
                    for s in &analysis.layer_suggestions {
                        self.ui
                            .wizard_suggestions
                            .push(crate::messages::ConsolidationSuggestion {
                                description: s.description.clone(),
                                rule_ids: s.rule_ids.clone(),
                                rule_count: s.rule_ids.len(),
                                patterns: s.patterns.clone(),
                                merged_pattern: s.merged_pattern.clone(),
                                is_window_rule: false,
                                selected: true, // Pre-select in wizard
                            });
                    }
                    log::info!(
                        "Wizard: Found {} consolidation suggestions",
                        self.ui.wizard_suggestions.len()
                    );
                }

                // Progress to next step
                if let DialogState::FirstRunWizard { .. } = &self.ui.dialog_state {
                    self.ui.dialog_state = DialogState::FirstRunWizard {
                        step: crate::messages::WizardStep::ImportResults,
                    };
                }
                Task::none()
            }

            Message::WizardConsolidationToggle(index) => {
                // Toggle selection of a wizard consolidation suggestion
                if let Some(suggestion) = self.ui.wizard_suggestions.get_mut(index) {
                    suggestion.selected = !suggestion.selected;
                }
                Task::none()
            }

            Message::WizardConsolidationApply => {
                // Apply selected wizard consolidation suggestions
                let selected: Vec<_> = self
                    .ui
                    .wizard_suggestions
                    .iter()
                    .filter(|s| s.selected)
                    .cloned()
                    .collect();

                if !selected.is_empty() {
                    log::info!(
                        "Wizard: Applying {} consolidation suggestions",
                        selected.len()
                    );

                    for suggestion in &selected {
                        if suggestion.is_window_rule {
                            self.apply_window_rule_consolidation(suggestion);
                        } else {
                            self.apply_layer_rule_consolidation(suggestion);
                        }
                    }
                }

                // Clear suggestions and move to complete
                self.ui.wizard_suggestions.clear();
                self.ui.dialog_state = DialogState::FirstRunWizard {
                    step: crate::messages::WizardStep::Complete,
                };
                Task::none()
            }

            Message::WizardConsolidationSkip => {
                // Skip consolidation, clear suggestions and move to complete
                self.ui.wizard_suggestions.clear();
                self.ui.dialog_state = DialogState::FirstRunWizard {
                    step: crate::messages::WizardStep::Complete,
                };
                Task::none()
            }

            Message::AnalyzeConsolidation => {
                // Analyze rules for consolidation opportunities
                let analysis = crate::config::analyze_rules(
                    &self.settings.window_rules.rules,
                    &self.settings.layer_rules.rules,
                );

                if analysis.has_suggestions() {
                    // Convert config suggestions to UI suggestions
                    let mut suggestions = Vec::new();

                    // Add window rule suggestions
                    for s in &analysis.window_suggestions {
                        suggestions.push(crate::messages::ConsolidationSuggestion {
                            description: s.description.clone(),
                            rule_ids: s.rule_ids.clone(),
                            rule_count: s.rule_ids.len(),
                            patterns: s.patterns.clone(),
                            merged_pattern: s.merged_pattern.clone(),
                            is_window_rule: true,
                            selected: true, // Select all by default
                        });
                    }

                    // Add layer rule suggestions
                    for s in &analysis.layer_suggestions {
                        suggestions.push(crate::messages::ConsolidationSuggestion {
                            description: s.description.clone(),
                            rule_ids: s.rule_ids.clone(),
                            rule_count: s.rule_ids.len(),
                            patterns: s.patterns.clone(),
                            merged_pattern: s.merged_pattern.clone(),
                            is_window_rule: false,
                            selected: true, // Select all by default
                        });
                    }

                    log::info!(
                        "Found {} consolidation suggestions ({} window, {} layer)",
                        suggestions.len(),
                        analysis.window_suggestions.len(),
                        analysis.layer_suggestions.len()
                    );

                    self.ui.dialog_state = DialogState::Consolidation { suggestions };
                } else {
                    log::info!("No consolidation opportunities found");
                    self.ui.toast = Some("No consolidation opportunities found".to_string());
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                }
                Task::none()
            }

            Message::ConsolidationToggle(index) => {
                // Toggle selection of a consolidation suggestion
                if let DialogState::Consolidation { suggestions } = &mut self.ui.dialog_state {
                    if let Some(suggestion) = suggestions.get_mut(index) {
                        suggestion.selected = !suggestion.selected;
                    }
                }
                Task::none()
            }

            Message::ConsolidationApply => {
                // Apply selected consolidation suggestions
                if let DialogState::Consolidation { suggestions } = &self.ui.dialog_state {
                    // Clone selected suggestions to avoid borrow issues
                    let selected: Vec<_> =
                        suggestions.iter().filter(|s| s.selected).cloned().collect();

                    if selected.is_empty() {
                        log::info!("No consolidation suggestions selected");
                    } else {
                        log::info!("Applying {} consolidation suggestions", selected.len());

                        let mut window_rules_changed = false;
                        let mut layer_rules_changed = false;

                        for suggestion in selected {
                            if suggestion.is_window_rule {
                                self.apply_window_rule_consolidation(&suggestion);
                                window_rules_changed = true;
                            } else {
                                self.apply_layer_rule_consolidation(&suggestion);
                                layer_rules_changed = true;
                            }
                        }

                        // Mark affected categories as dirty
                        if window_rules_changed {
                            self.save.dirty_tracker
                                .mark(crate::config::SettingsCategory::WindowRules);
                        }
                        if layer_rules_changed {
                            self.save.dirty_tracker
                                .mark(crate::config::SettingsCategory::LayerRules);
                        }

                        self.mark_changed();
                        self.ui.toast = Some("Rules consolidated successfully".to_string());
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                    }
                }
                self.ui.dialog_state = DialogState::None;
                Task::none()
            }

            // System
            Message::WindowCloseRequested => {
                // Perform final save before exiting (blocking to prevent data loss)
                if self.save.dirty_tracker.is_dirty() {
                    log::info!("Window closing with unsaved changes, performing blocking save...");

                    // Take dirty categories for blocking save
                    let dirty = self.save.dirty_tracker.take();

                    // Perform blocking save (acceptable since typically <100ms)
                    match crate::config::save_dirty(&self.paths, &self.settings, &dirty, self.ui.feature_compat) {
                        Ok(count) => {
                            log::info!("Successfully saved {} file(s) before exit", count);
                        }
                        Err(e) => {
                            log::error!("Failed to save on exit: {}", e);
                            // Exit anyway - user explicitly closed window
                        }
                    }
                }

                log::info!("Exiting application");
                iced::exit()
            }

            Message::CheckNiriStatus => {
                // Run niri status check asynchronously to avoid blocking UI
                crate::ipc::tasks::check_niri_running(Message::NiriStatusChecked)
            }

            Message::NiriStatusChecked(is_connected) => {
                self.ui.niri_status = if is_connected {
                    crate::views::status_bar::NiriStatus::Connected
                } else {
                    crate::views::status_bar::NiriStatus::Disconnected
                };
                Task::none()
            }

            Message::Debug(msg) => self.update_debug(msg),
            Message::Miscellaneous(msg) => self.update_miscellaneous(msg),
            Message::Environment(msg) => self.update_environment(msg),
            Message::SwitchEvents(msg) => self.update_switch_events(msg),
            Message::RecentWindows(msg) => self.update_recent_windows(msg),
            Message::Trackpoint(msg) => self.update_trackpoint(msg),
            Message::Trackball(msg) => self.update_trackball(msg),
            Message::Tablet(msg) => self.update_tablet(msg),
            Message::Touch(msg) => self.update_touch(msg),
            Message::Gestures(msg) => self.update_gestures(msg),
            Message::LayoutExtras(msg) => self.update_layout_extras(msg),
            Message::Startup(msg) => self.update_startup(msg),
            Message::Tools(msg) => self.update_tools(msg),
            Message::Preferences(msg) => self.update_preferences(msg),
            Message::ConfigEditor(msg) => self.update_config_editor(msg),
            Message::Backups(msg) => self.update_backups(msg),

            Message::None => Task::none(),
        }
    }

    /// Returns the subscription for periodic save checks and keyboard capture
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = self.base_subscriptions();

        // Add keyboard subscription based on current mode
        if self.ui.key_capture_active.is_some() {
            subs.push(self.key_capture_subscription());
        } else if !self.settings.preferences.search_hotkey.is_empty() {
            subs.push(self.search_hotkey_subscription());
        }

        Subscription::batch(subs)
    }

    /// Base subscriptions always active: save checks, niri status, toast clearing, system theme
    fn base_subscriptions(&self) -> Vec<Subscription<Message>> {
        let mut subs = vec![
            // Periodic save checks (every 200ms - sufficient with 300ms debounce)
            time::every(Duration::from_millis(200))
                .map(|_| Message::Save(SaveMessage::CheckSave)),
            // Niri status check (every 5 seconds)
            time::every(Duration::from_secs(5)).map(|_| Message::CheckNiriStatus),
            // System theme detection (portal or file watcher)
            crate::system_theme::subscription().map(Message::SystemThemeEvent),
        ];

        // Toast auto-clear check (every 500ms, only when toast is showing)
        if self.ui.toast.is_some() {
            subs.push(time::every(Duration::from_millis(500)).map(|_| Message::ClearToast));
        }

        subs
    }

    /// Subscription for key capture mode (when recording keybindings)
    fn key_capture_subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                // ESC cancels capture
                if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    return Message::Keybindings(
                        crate::messages::KeybindingsMessage::CancelKeyCapture,
                    );
                }

                // Convert key and modifiers to a key combo string
                let key_combo = helpers::format_key_combo(&key, modifiers);

                // Only capture if we got a valid key (not just a modifier)
                if !key_combo.is_empty() {
                    Message::Keybindings(crate::messages::KeybindingsMessage::CapturedKey(
                        key_combo,
                    ))
                } else {
                    Message::None
                }
            }
            _ => Message::None,
        })
    }

    /// Subscription for search hotkey (when not in key capture mode)
    fn search_hotkey_subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        let search_hotkey = self.settings.preferences.search_hotkey.clone();

        keyboard::listen()
            .with(search_hotkey)
            .map(|(hotkey, event): (String, keyboard::Event)| match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    let key_combo = helpers::format_key_combo(&key, modifiers);
                    if helpers::hotkey_matches(&key_combo, &hotkey) {
                        Message::ToggleSearch
                    } else {
                        Message::None
                    }
                }
                _ => Message::None,
            })
    }

    /// Constructs the UI from current state
    pub fn view(&self) -> Element<'_, Message> {
        // Primary navigation bar (top)
        let primary_nav = views::navigation::primary_nav(
            self.ui.current_page,
            &self.ui.search_query,
            self.ui.show_search_bar,
        );

        // Secondary navigation bar (sub-tabs)
        let secondary_nav = views::navigation::secondary_nav(self.ui.current_page);

        // Main content area (always show current page)
        let content_area = self.page_content();

        // Status bar (bottom)
        let is_dirty = self.save.dirty_tracker.is_dirty();
        let save_status = self.ui.toast.clone();
        let status_bar = views::status_bar::view(
            is_dirty,
            save_status,
            self.ui.current_theme,
            self.ui.niri_status,
        );

        // Stack everything vertically
        let layout = column![primary_nav, secondary_nav, content_area, status_bar,].spacing(0);

        let main_view: Element<'_, Message> =
            container(layout).width(Length::Fill).height(Length::Fill).into();

        // Check for search modal (when search bar is hidden but search is active)
        let with_search_modal = if !self.ui.show_search_bar && self.ui.search_focused {
            use iced::widget::{column as col, row, text_input, Space};
            use crate::theme::{search_dropdown_style, fonts};

            // Build search input
            let search_input = text_input("Search settings...", &self.ui.search_query)
                .id(views::navigation::search_input_id())
                .on_input(Message::SearchQueryChanged)
                .padding(12)
                .size(16)
                .width(Length::Fixed(400.0));

            // Build results list if there are results
            let results_content: Element<'_, Message> = if !self.ui.search_query.trim().is_empty() {
                if self.ui.search_results.is_empty() {
                    container(
                        text("No matching settings found")
                            .size(14)
                            .color([0.6, 0.6, 0.6])
                    )
                    .padding(16)
                    .into()
                } else {
                    let mut results_col = col![].spacing(4).padding(8);
                    for (index, result) in self.ui.search_results.iter().take(8).enumerate() {
                        let item = iced::widget::button(
                            row![
                                col![
                                    text(&result.setting_name).size(14).font(fonts::UI_FONT_MEDIUM),
                                    text(&result.description).size(11).color([0.6, 0.6, 0.6]),
                                ]
                                .spacing(2)
                                .width(Length::Fill),
                                text(result.page.name()).size(10).color([0.5, 0.5, 0.5]),
                            ]
                            .spacing(8)
                            .padding([10, 12])
                        )
                        .on_press(Message::SearchResultSelected(index))
                        .width(Length::Fill)
                        .style(crate::theme::search_dropdown_item_style());

                        results_col = results_col.push(item);
                    }
                    if self.ui.search_results.len() > 8 {
                        results_col = results_col.push(
                            container(
                                text(format!("and {} more...", self.ui.search_results.len() - 8))
                                    .size(12)
                                    .color([0.5, 0.5, 0.5])
                            )
                            .padding([8, 16])
                        );
                    }
                    results_col.into()
                }
            } else {
                container(
                    text("Type to search settings...")
                        .size(13)
                        .color([0.5, 0.5, 0.5])
                )
                .padding(16)
                .into()
            };

            // Build the modal
            let modal_content = col![
                row![
                    text("🔍").size(16),
                    search_input,
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
                results_content,
            ]
            .spacing(8);

            let modal = container(modal_content)
                .padding(16)
                .style(search_dropdown_style)
                .width(Length::Fixed(450.0));

            // Center the modal with a semi-transparent backdrop
            let backdrop = container(
                iced::widget::mouse_area(
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .on_press(Message::ToggleSearch) // Click backdrop to close
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            });

            let modal_overlay = stack![
                backdrop,
                col![
                    Space::new().height(Length::Fixed(100.0)),
                    container(modal)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                ],
            ];

            stack![main_view, modal_overlay].into()
        } else {
            main_view
        };

        // Check for search dropdown overlay (when search bar is visible)
        let with_dropdown = if self.ui.show_search_bar {
            if let Some(dropdown) =
                views::search_dropdown::view(&self.ui.search_results, &self.ui.search_query)
            {
                use iced::widget::{column as col, Space};
                // Position dropdown at top-right, below nav bar
                let dropdown_overlay = col![
                    Space::new().height(Length::Fixed(50.0)),
                    container(dropdown)
                        .width(Length::Fill)
                        .padding(20)
                        .align_x(Horizontal::Right),
                ];

                stack![with_search_modal, dropdown_overlay].into()
            } else {
                with_search_modal
            }
        } else {
            with_search_modal
        };

        // If there's an active dialog, render it on top of everything
        if let Some(dialog) =
            views::dialogs::view(&self.ui.dialog_state, &self.ui.wizard_suggestions, self.ui.niri_version)
        {
            dialog
        } else {
            with_dropdown
        }
    }

    /// Creates the content area for the current page
    fn page_content(&self) -> Element<'_, Message> {
        // Each page handles its own scrollable container
        match self.ui.current_page {
            Page::Overview => return self.overview_page(),
            Page::Appearance => {
                return views::appearance::view(&self.settings.appearance);
            }
            Page::Behavior => {
                return views::behavior::view(&self.settings.behavior);
            }
            Page::Keyboard => {
                return views::keyboard::view(&self.settings.keyboard);
            }
            Page::Mouse => {
                return views::mouse::view(&self.settings.mouse);
            }
            Page::Touchpad => {
                return views::touchpad::view(&self.settings.touchpad);
            }
            Page::Trackpoint => {
                return views::trackpoint::view(&self.settings.trackpoint);
            }
            Page::Trackball => {
                return views::trackball::view(&self.settings.trackball);
            }
            Page::Tablet => {
                return views::tablet::view(
                    &self.settings.tablet,
                    &self.ui.tablet_calibration_cache,
                );
            }
            Page::Touch => {
                return views::touch::view(&self.settings.touch, &self.ui.touch_calibration_cache);
            }
            Page::Animations => {
                return views::animations::view(&self.settings.animations);
            }
            Page::Cursor => {
                return views::cursor::view(&self.settings.cursor);
            }
            Page::LayoutExtras => {
                return views::layout_extras::view(&self.settings.layout_extras);
            }
            Page::Gestures => {
                return views::gestures::view(&self.settings.gestures);
            }
            Page::Workspaces => {
                return views::workspaces::view(&self.settings.workspaces);
            }
            Page::WindowRules => {
                return views::window_rules::view(
                    &self.settings.window_rules,
                    self.ui.selected_window_rule_id,
                    &self.ui.window_rule_sections_expanded,
                    &self.ui.window_rule_regex_errors,
                    &self.ui.available_workspaces,
                );
            }
            Page::LayerRules => {
                return views::layer_rules::view(
                    &self.settings.layer_rules,
                    self.ui.selected_layer_rule_id,
                    &self.ui.layer_rule_sections_expanded,
                    &self.ui.layer_rule_regex_errors,
                );
            }
            Page::Keybindings => {
                return views::keybindings::view(
                    &self.settings.keybindings,
                    self.ui.selected_keybinding_index,
                    &self.ui.keybinding_sections_expanded,
                    self.ui.key_capture_active,
                );
            }
            Page::Outputs => {
                return views::outputs::view(
                    &self.settings.outputs,
                    self.ui.selected_output_index,
                    &self.ui.output_sections_expanded,
                    &self.ui.tools_state.outputs, // IPC data for available modes
                );
            }
            Page::Miscellaneous => {
                return views::miscellaneous::view(&self.settings.miscellaneous);
            }
            Page::Startup => {
                return views::startup::view(&self.settings.startup);
            }
            Page::Environment => {
                return views::environment::view(&self.settings.environment);
            }
            Page::Debug => {
                return views::debug::view(&self.settings.debug);
            }
            Page::SwitchEvents => {
                return views::switch_events::view(&self.settings.switch_events);
            }
            Page::RecentWindows => {
                return views::recent_windows::view(&self.settings.recent_windows);
            }
            Page::Tools => {
                let niri_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                return views::tools::view(&self.ui.tools_state, niri_connected);
            }
            Page::Preferences => {
                return views::preferences::view(
                    self.settings.preferences.float_settings_app,
                    self.ui.show_search_bar,
                    &self.settings.preferences.search_hotkey,
                );
            }
            Page::ConfigEditor => {
                return views::config_editor::view(
                    &self.ui.config_editor_state,
                    &self.ui.config_editor_content,
                );
            }
            Page::Backups => {
                return views::backups::view(&self.ui.backups_state);
            }
        }
    }

    /// Overview page - summary dashboard with overview settings
    fn overview_page(&self) -> Element<'_, Message> {
        use crate::messages::OverviewMessage;
        use crate::views::widgets::{page_title, spacer};
        use iced::widget::{pick_list, row, scrollable, slider, text_input, toggler};
        use iced::Alignment;

        let settings = &self.settings;

        // Overview settings section (workspace exposé / overview mode)
        let overview_settings = {
            let zoom = settings.overview.zoom;
            let backdrop_color = settings
                .overview
                .backdrop_color
                .as_ref()
                .map(|c| c.to_hex())
                .unwrap_or_default();
            let shadow_enabled = settings
                .overview
                .workspace_shadow
                .as_ref()
                .map(|s| s.enabled)
                .unwrap_or(false);

            let mut overview_section = column![
                page_title("Workspace Overview Settings"),
                text("Configure the appearance of the workspace overview (toggle-overview action)").size(12).color([0.7, 0.7, 0.7]),

                // Zoom slider
                row![
                    text("Zoom Level:").size(14).width(Length::Fixed(140.0)),
                    slider(0.1..=2.0, zoom as f32, |v| Message::Overview(OverviewMessage::SetZoom(v as f64)))
                        .step(0.05)
                        .width(Length::Fixed(200.0)),
                    text(format!("{:.2}x", zoom)).size(14).width(Length::Fixed(60.0)),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("How much to scale down windows in overview (0.1 = 10%, 1.0 = 100%)").size(12).color([0.7, 0.7, 0.7]),
                spacer(8.0),

                // Backdrop color
                row![
                    text("Backdrop Color:").size(14).width(Length::Fixed(140.0)),
                    text_input("#00000080", &backdrop_color)
                        .on_input(|v| {
                            let color = if v.is_empty() { None } else { Some(v) };
                            Message::Overview(OverviewMessage::SetBackdropColor(color))
                        })
                        .padding(6)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Background color behind workspaces in overview (hex with alpha, e.g., #00000080)").size(12).color([0.7, 0.7, 0.7]),
                spacer(8.0),

                // Workspace shadow toggle
                row![
                    text("Workspace Shadow:").size(14).width(Length::Fixed(140.0)),
                    toggler(shadow_enabled)
                        .on_toggle(|v| Message::Overview(OverviewMessage::ToggleWorkspaceShadow(v))),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text("Add shadow behind workspaces in overview (v25.05+)").size(12).color([0.7, 0.7, 0.7]),
            ]
            .spacing(4);

            // Shadow settings (if enabled)
            if let Some(ref shadow) = settings.overview.workspace_shadow {
                if shadow.enabled {
                    let shadow_color = shadow.color.to_hex();
                    overview_section = overview_section.push(spacer(8.0));
                    overview_section = overview_section.push(
                        row![
                            text("  Softness:").size(14).width(Length::Fixed(140.0)),
                            slider(0..=200, shadow.softness, |v| Message::Overview(
                                OverviewMessage::SetWorkspaceShadowSoftness(v)
                            ))
                            .width(Length::Fixed(150.0)),
                            text(format!("{}", shadow.softness))
                                .size(14)
                                .width(Length::Fixed(40.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    );
                    overview_section = overview_section.push(
                        row![
                            text("  Spread:").size(14).width(Length::Fixed(140.0)),
                            slider(0..=200, shadow.spread, |v| Message::Overview(
                                OverviewMessage::SetWorkspaceShadowSpread(v)
                            ))
                            .width(Length::Fixed(150.0)),
                            text(format!("{}", shadow.spread))
                                .size(14)
                                .width(Length::Fixed(40.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    );
                    overview_section = overview_section.push(
                        row![
                            text("  Offset X:").size(14).width(Length::Fixed(140.0)),
                            slider(-100..=100, shadow.offset_x, |v| Message::Overview(
                                OverviewMessage::SetWorkspaceShadowOffsetX(v)
                            ))
                            .width(Length::Fixed(150.0)),
                            text(format!("{}", shadow.offset_x))
                                .size(14)
                                .width(Length::Fixed(40.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    );
                    overview_section = overview_section.push(
                        row![
                            text("  Offset Y:").size(14).width(Length::Fixed(140.0)),
                            slider(-100..=100, shadow.offset_y, |v| Message::Overview(
                                OverviewMessage::SetWorkspaceShadowOffsetY(v)
                            ))
                            .width(Length::Fixed(150.0)),
                            text(format!("{}", shadow.offset_y))
                                .size(14)
                                .width(Length::Fixed(40.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    );
                    overview_section = overview_section.push(
                        row![
                            text("  Shadow Color:").size(14).width(Length::Fixed(140.0)),
                            text_input("#00000050", &shadow_color)
                                .on_input(|v| Message::Overview(
                                    OverviewMessage::SetWorkspaceShadowColor(v)
                                ))
                                .padding(6)
                                .width(Length::Fixed(150.0)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    );
                }
            }

            overview_section
        };

        let summary = column![
            text("Welcome to Nirify").size(24),
            text("A modern GUI for configuring the niri Wayland compositor")
                .size(14)
                .color([0.7, 0.7, 0.7]),
            spacer(16.0),
            // Preferences Section
            text("Preferences").size(18),
            spacer(8.0),
            row![
                text("Theme:").size(14).width(Length::Fixed(100.0)),
                pick_list(
                    crate::theme::AppTheme::all(),
                    Some(self.ui.current_theme),
                    Message::ChangeTheme,
                )
                .placeholder("Select theme...")
                .width(Length::Fixed(260.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            text("Choose your preferred color theme for the application")
                .size(12)
                .color([0.7, 0.7, 0.7]),
            spacer(16.0),
            // Overview Settings
            overview_settings,
            spacer(16.0),
            // Current Settings Summary
            text("Current Configuration").size(18),
            spacer(8.0),
            text(format!(
                "Focus Ring: {} ({}px)",
                if settings.appearance.focus_ring_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                settings.appearance.focus_ring_width as i32
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            text(format!(
                "Border: {} ({}px)",
                if settings.appearance.border_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                settings.appearance.border_thickness as i32
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            text(format!(
                "Window Gaps: {}px",
                settings.appearance.gaps as i32
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            text(format!(
                "Corner Radius: {}px",
                settings.appearance.corner_radius as i32
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            spacer(12.0),
            text(format!(
                "Focus Follows Mouse: {}",
                if settings.behavior.focus_follows_mouse {
                    "Yes"
                } else {
                    "No"
                }
            ))
            .size(14),
            text(format!(
                "Workspace Auto Back-and-Forth: {}",
                if settings.behavior.workspace_auto_back_and_forth {
                    "Yes"
                } else {
                    "No"
                }
            ))
            .size(14),
            spacer(12.0),
            text(format!("Keyboard Layout: {}", settings.keyboard.xkb_layout))
                .size(14)
                .font(fonts::MONO_FONT),
            text(format!(
                "Repeat Rate: {}/sec, Delay: {}ms",
                settings.keyboard.repeat_rate, settings.keyboard.repeat_delay
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            spacer(12.0),
            text(format!(
                "Mouse: Natural Scroll {}",
                if settings.mouse.natural_scroll {
                    "ON"
                } else {
                    "OFF"
                }
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            text(format!(
                "Touchpad: Tap-to-Click {}",
                if settings.touchpad.tap { "ON" } else { "OFF" }
            ))
            .size(14)
            .font(fonts::MONO_FONT),
            text(format!(
                "Cursor: {} ({}px)",
                settings.cursor.theme, settings.cursor.size
            ))
            .size(14)
            .font(fonts::MONO_FONT),
        ]
        .spacing(6)
        .width(Length::Fill);

        // Wrap in scrollable with full width
        scrollable(container(summary).padding(20).width(Length::Fill))
            .height(Length::Fill)
            .into()
    }

    /// Mark that settings have changed (triggers debounced save)
    pub(crate) fn mark_changed(&mut self) {
        self.save.last_change_time = Some(std::time::Instant::now());
    }

    /// Check if we should save now (debounce: 300ms since last change)
    fn should_save(&self) -> bool {
        if self.save.in_progress || !self.save.dirty_tracker.is_dirty() {
            return false;
        }

        match self.save.last_change_time {
            Some(t) => t.elapsed() >= Duration::from_millis(300),
            None => false,
        }
    }

    /// Create an async save task
    fn save_task(&mut self) -> Task<Message> {
        self.save.in_progress = true;
        let settings = self.settings.clone();
        let dirty = self.save.dirty_tracker.take();
        let paths = self.paths.clone();
        let feature_compat = self.ui.feature_compat;

        Task::perform(
            async move {
                match crate::config::save_dirty(&paths, &settings, &dirty, feature_compat) {
                    Ok(count) => SaveResult::Success {
                        files_written: count,
                        categories: dirty.into_iter().collect(),
                    },
                    Err(e) => SaveResult::Error {
                        message: e.to_string(),
                    },
                }
            },
            Message::SaveCompleted,
        )
    }

    /// Create an async task to reload niri config
    fn reload_niri_config_task(&self) -> Task<Message> {
        Task::perform(
            async move {
                match crate::ipc::reload_config() {
                    Ok(()) => ReloadResult::Success,
                    Err(e) => ReloadResult::Error {
                        message: e.to_string(),
                    },
                }
            },
            Message::ReloadCompleted,
        )
    }

    /// Apply window rule consolidation - merge multiple rules into one
    fn apply_window_rule_consolidation(
        &mut self,
        suggestion: &crate::messages::ConsolidationSuggestion,
    ) {
        use crate::config::models::WindowRuleMatch;

        // Get the first rule ID to keep (will be modified to use merged pattern)
        let Some(&first_id) = suggestion.rule_ids.first() else {
            return;
        };

        // Find the first rule and update its match pattern
        if let Some(rule) = self
            .settings
            .window_rules
            .rules
            .iter_mut()
            .find(|r| r.id == first_id)
        {
            // Update the match to use the merged regex pattern
            if !rule.matches.is_empty() {
                rule.matches[0].app_id = Some(suggestion.merged_pattern.clone());
            } else {
                rule.matches.push(WindowRuleMatch {
                    app_id: Some(suggestion.merged_pattern.clone()),
                    ..Default::default()
                });
            }

            // Update the name to reflect consolidation
            rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
        }

        // Remove all other rules that were consolidated
        let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
        self.settings
            .window_rules
            .rules
            .retain(|r| !other_ids.contains(&r.id));

        log::info!(
            "Consolidated {} window rules into one with pattern: {}",
            suggestion.rule_ids.len(),
            suggestion.merged_pattern
        );
    }

    /// Apply layer rule consolidation - merge multiple rules into one
    fn apply_layer_rule_consolidation(
        &mut self,
        suggestion: &crate::messages::ConsolidationSuggestion,
    ) {
        use crate::config::models::LayerRuleMatch;

        // Get the first rule ID to keep
        let Some(&first_id) = suggestion.rule_ids.first() else {
            return;
        };

        // Find the first rule and update its match pattern
        if let Some(rule) = self
            .settings
            .layer_rules
            .rules
            .iter_mut()
            .find(|r| r.id == first_id)
        {
            // Update the match to use the merged regex pattern
            if !rule.matches.is_empty() {
                rule.matches[0].namespace = Some(suggestion.merged_pattern.clone());
            } else {
                rule.matches.push(LayerRuleMatch {
                    namespace: Some(suggestion.merged_pattern.clone()),
                    ..Default::default()
                });
            }

            // Update the name to reflect consolidation
            rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
        }

        // Remove all other rules that were consolidated
        let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
        self.settings
            .layer_rules
            .rules
            .retain(|r| !other_ids.contains(&r.id));

        log::info!(
            "Consolidated {} layer rules into one with pattern: {}",
            suggestion.rule_ids.len(),
            suggestion.merged_pattern
        );
    }
}

// Note: Default is not needed with iced::application() - it uses App::new() directly

/// Runs the application
pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(|app: &App| {
            use crate::theme::AppTheme;
            match app.ui.current_theme {
                AppTheme::System => app.ui.system_theme_state.build_theme(),
                other => other.to_iced_theme(),
            }
        })
        .settings(iced::Settings {
            id: Some("nirify".to_string()),
            ..Default::default()
        })
        .window(iced::window::Settings {
            min_size: Some(iced::Size::new(650.0, 200.0)),
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "nirify".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}
