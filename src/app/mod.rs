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

use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Task};
use iced::time;

use crate::config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::messages::{DialogState, Message, Page, SaveMessage};
use crate::save_manager::{SaveResult, ReloadResult};
use crate::views;
use crate::theme::fonts;

/// Main application state
pub struct App {
    /// Settings - direct ownership (no mutex needed, iced is single-threaded)
    settings: Settings,

    /// Config paths for loading/saving
    paths: Arc<ConfigPaths>,

    /// Tracks which settings categories have unsaved changes
    dirty_tracker: DirtyTracker,

    /// Search index (domain data, not UI state)
    search_index: crate::search::SearchIndex,

    /// Last time a change was made (for debounced save)
    last_change_time: Option<std::time::Instant>,

    /// Whether a save is currently in progress
    save_in_progress: bool,

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
                panic!("Cannot proceed without valid config paths");
            }
        };

        // Load settings from disk (load_settings returns Settings, not Result)
        let settings = crate::config::load_settings(&paths);
        log::info!("Settings loaded successfully");

        // Parse theme from settings
        let current_theme = settings.preferences.theme.parse::<crate::theme::AppTheme>()
            .unwrap_or_default();

        // Initialize calibration matrix caches from settings
        let tablet_calibration_cache = crate::views::widgets::format_matrix_values(
            settings.tablet.calibration_matrix
        );
        let touch_calibration_cache = crate::views::widgets::format_matrix_values(
            settings.touch.calibration_matrix
        );

        // Check initial niri connection status
        let niri_status = if crate::ipc::is_niri_running() {
            crate::views::status_bar::NiriStatus::Connected
        } else {
            crate::views::status_bar::NiriStatus::Disconnected
        };

        // Create UI state
        let mut ui = UiState::new(current_theme, tablet_calibration_cache, touch_calibration_cache);
        ui.niri_status = niri_status;

        let app = Self {
            settings,
            paths,
            dirty_tracker: DirtyTracker::new(),
            search_index: crate::search::SearchIndex::new(),
            last_change_time: None,
            save_in_progress: false,
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
                // Auto-refresh IPC outputs when navigating to Outputs page
                if page == Page::Outputs {
                    let is_connected = matches!(
                        self.ui.niri_status,
                        crate::views::status_bar::NiriStatus::Connected
                    );
                    if is_connected {
                        return Task::perform(
                            async { crate::ipc::get_full_outputs().map_err(|e| e.to_string()) },
                            |result| Message::Tools(crate::messages::ToolsMessage::OutputsLoaded(result)),
                        );
                    }
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

                Task::none()
            }

            Message::SearchResultSelected(index) => {
                // Navigate to the selected page
                if let Some(result) = self.ui.search_results.get(index) {
                    self.ui.current_page = result.page;
                    // Clear search after navigation
                    self.ui.search_query.clear();
                    self.ui.search_results.clear();
                }
                Task::none()
            }

            Message::ClearSearch => {
                self.ui.search_query.clear();
                self.ui.search_results.clear();
                self.ui.last_search_time = None;
                Task::none()
            }

            // Theme
            Message::ChangeTheme(theme) => {
                self.ui.current_theme = theme;

                // Save theme to settings (direct access, no mutex needed)
                self.settings.preferences.theme = theme.to_str().to_string();

                // Mark preferences as dirty for auto-save
                self.dirty_tracker.mark(SettingsCategory::Preferences);
                self.mark_changed();

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
                self.save_in_progress = false;
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
                                        self.ui.selected_window_rule_id = self.settings.window_rules.rules.first().map(|r| r.id);
                                    }
                                    self.dirty_tracker.mark(crate::config::SettingsCategory::WindowRules);
                                } else if self.settings.layer_rules.remove(rule_id) {
                                    log::info!("Deleted layer rule {}", rule_id);
                                    if self.ui.selected_layer_rule_id == Some(rule_id) {
                                        self.ui.selected_layer_rule_id = self.settings.layer_rules.rules.first().map(|r| r.id);
                                    }
                                    self.dirty_tracker.mark(crate::config::SettingsCategory::LayerRules);
                                }
                                self.mark_changed();
                            }
                            ConfirmAction::ResetSettings => {
                                log::info!("Resetting all settings to defaults");
                                self.settings = crate::config::models::Settings::default();
                                self.dirty_tracker.mark_all();
                                self.mark_changed();
                            }
                            ConfirmAction::ClearAllKeybindings => {
                                log::info!("Clearing all keybindings");
                                self.settings.keybindings.bindings.clear();
                                self.dirty_tracker.mark(crate::config::SettingsCategory::Keybindings);
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
                        WizardStep::ImportResults => WizardStep::Complete,
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
                        WizardStep::Complete => WizardStep::ImportResults,
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

                // Add include line to user's config.kdl
                if let Err(e) = self.paths.add_include_line() {
                    log::error!("Failed to add include line: {}", e);
                    self.ui.dialog_state = DialogState::Error {
                        title: "Setup Error".to_string(),
                        message: "Failed to add include line to config.kdl.".to_string(),
                        details: Some(e.to_string()),
                    };
                    return Task::none();
                }

                // Trigger initial save to create all config files
                self.dirty_tracker.mark_all();
                self.mark_changed();

                log::info!("Wizard: Config setup complete");

                // Progress to next step
                if let DialogState::FirstRunWizard { .. } = &self.ui.dialog_state {
                    self.ui.dialog_state = DialogState::FirstRunWizard {
                        step: crate::messages::WizardStep::ImportResults,
                    };
                }
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
                    let selected: Vec<_> = suggestions.iter()
                        .filter(|s| s.selected)
                        .cloned()
                        .collect();

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
                            self.dirty_tracker.mark(crate::config::SettingsCategory::WindowRules);
                        }
                        if layer_rules_changed {
                            self.dirty_tracker.mark(crate::config::SettingsCategory::LayerRules);
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
                if self.dirty_tracker.is_dirty() {
                    log::info!("Window closing with unsaved changes, performing blocking save...");

                    // Take dirty categories for blocking save
                    let dirty = self.dirty_tracker.take();

                    // Perform blocking save (acceptable since typically <100ms)
                    match crate::config::save_dirty(&self.paths, &self.settings, &dirty) {
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
                std::process::exit(0);
                #[allow(unreachable_code)]
                Task::none()
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
        use iced::keyboard;

        // Base subscription: periodic save checks (every 200ms - sufficient with 300ms debounce)
        let save_check = time::every(Duration::from_millis(200))
            .map(|_| Message::Save(SaveMessage::CheckSave));

        // Niri status check (every 5 seconds)
        let niri_check = time::every(Duration::from_secs(5))
            .map(|_| Message::CheckNiriStatus);

        // Toast auto-clear check (every 500ms, only when toast is showing)
        let toast_check = if self.ui.toast.is_some() {
            Some(time::every(Duration::from_millis(500))
                .map(|_| Message::ClearToast))
        } else {
            None
        };

        // Keyboard capture subscription (only active when capturing)
        if self.ui.key_capture_active.is_some() {
            let key_capture = keyboard::listen().map(|event| {
                match event {
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
                            Message::Keybindings(
                                crate::messages::KeybindingsMessage::CapturedKey(key_combo),
                            )
                        } else {
                            Message::None
                        }
                    }
                    _ => Message::None,
                }
            });

            let mut subs = vec![save_check, niri_check, key_capture];
            if let Some(toast) = toast_check {
                subs.push(toast);
            }
            Subscription::batch(subs)
        } else {
            let mut subs = vec![save_check, niri_check];
            if let Some(toast) = toast_check {
                subs.push(toast);
            }
            Subscription::batch(subs)
        }
    }

    /// Constructs the UI from current state
    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::column;

        // Primary navigation bar (top)
        let primary_nav = views::navigation::primary_nav(self.ui.current_page, &self.ui.search_query);

        // Secondary navigation bar (sub-tabs)
        let secondary_nav = views::navigation::secondary_nav(self.ui.current_page);

        // Main content area (or search results if searching)
        let content_area = if !self.ui.search_query.is_empty() && !self.ui.search_results.is_empty() {
            // Show search results in the content area
            views::search_results::view(&self.ui.search_results, &self.ui.search_query)
        } else {
            self.page_content()
        };

        // Status bar (bottom)
        let is_dirty = self.dirty_tracker.is_dirty();
        let save_status = self.ui.toast.clone();
        let status_bar = views::status_bar::view(is_dirty, save_status, self.ui.current_theme, self.ui.niri_status);

        // Stack everything vertically
        let layout = column![
            primary_nav,
            secondary_nav,
            content_area,
            status_bar,
        ]
        .spacing(0);

        let main_view = container(layout)
            .width(Length::Fill)
            .height(Length::Fill);

        // If there's an active dialog, render it on top
        if let Some(dialog) = views::dialogs::view(&self.ui.dialog_state) {
            // For now, use iced's Stack widget or similar approach
            // Since iced doesn't have perfect z-layering, dialogs handle their own backdrop
            dialog
        } else {
            main_view.into()
        }
    }

    /// Creates the content area for the current page
    fn page_content(&self) -> Element<'_, Message> {
        use iced::widget::scrollable;

        // Get the page-specific content (without the page title - nav shows it)
        let page_view = match self.ui.current_page {
            Page::Overview => return self.overview_page(),
            Page::Appearance => {
                views::appearance::view(&self.settings.appearance)
            }
            Page::Behavior => {
                views::behavior::view(&self.settings.behavior)
            }
            Page::Keyboard => {
                views::keyboard::view(&self.settings.keyboard)
            }
            Page::Mouse => {
                views::mouse::view(&self.settings.mouse)
            }
            Page::Touchpad => {
                views::touchpad::view(&self.settings.touchpad)
            }
            Page::Trackpoint => {
                views::trackpoint::view(&self.settings.trackpoint)
            }
            Page::Trackball => {
                views::trackball::view(&self.settings.trackball)
            }
            Page::Tablet => {
                return views::tablet::view(&self.settings.tablet, &self.ui.tablet_calibration_cache);
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
                    &self.ui.tools_state.outputs,  // IPC data for available modes
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
                return views::preferences::view(self.settings.preferences.float_settings_app);
            }
            Page::ConfigEditor => {
                return views::config_editor::view(&self.ui.config_editor_state);
            }
            Page::Backups => {
                return views::backups::view(&self.ui.backups_state);
            }
        };

        // Wrap in scrollable container with padding
        scrollable(
            container(page_view)
                .padding(24)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Overview page - summary dashboard with widget demo
    fn overview_page(&self) -> Element<'_, Message> {
        use iced::widget::{pick_list, row};
        use iced::Alignment;

        let settings = &self.settings;

        let summary = column![
            text("Welcome to Niri Settings").size(24),
            text("A modern GUI for configuring the niri Wayland compositor").size(14).color([0.7, 0.7, 0.7]),
            text("").size(16),

            // Preferences Section
            text("Preferences").size(18),
            text("").size(8),
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
            text("Choose your preferred color theme for the application").size(12).color([0.75, 0.75, 0.75]),
            text("").size(16),

            // Current Settings Summary
            text("Current Configuration").size(18),
            text("").size(8),
            text(format!("Focus Ring: {} ({}px)",
                if settings.appearance.focus_ring_enabled { "Enabled" } else { "Disabled" },
                settings.appearance.focus_ring_width as i32
            )).size(14).font(fonts::MONO_FONT),
            text(format!("Border: {} ({}px)",
                if settings.appearance.border_enabled { "Enabled" } else { "Disabled" },
                settings.appearance.border_thickness as i32
            )).size(14).font(fonts::MONO_FONT),
            text(format!("Window Gaps: {}px", settings.appearance.gaps as i32)).size(14).font(fonts::MONO_FONT),
            text(format!("Corner Radius: {}px", settings.appearance.corner_radius as i32)).size(14).font(fonts::MONO_FONT),
            text("").size(12),

            text(format!("Focus Follows Mouse: {}",
                if settings.behavior.focus_follows_mouse { "Yes" } else { "No" }
            )).size(14),
            text(format!("Workspace Auto Back-and-Forth: {}",
                if settings.behavior.workspace_auto_back_and_forth { "Yes" } else { "No" }
            )).size(14),
            text("").size(12),

            text(format!("Keyboard Layout: {}", settings.keyboard.xkb_layout)).size(14).font(fonts::MONO_FONT),
            text(format!("Repeat Rate: {}/sec, Delay: {}ms",
                settings.keyboard.repeat_rate, settings.keyboard.repeat_delay
            )).size(14).font(fonts::MONO_FONT),
            text("").size(12),

            text(format!("Mouse: Natural Scroll {}",
                if settings.mouse.natural_scroll { "ON" } else { "OFF" }
            )).size(14).font(fonts::MONO_FONT),
            text(format!("Touchpad: Tap-to-Click {}",
                if settings.touchpad.tap { "ON" } else { "OFF" }
            )).size(14).font(fonts::MONO_FONT),
            text(format!("Cursor: {} ({}px)",
                settings.cursor.theme, settings.cursor.size
            )).size(14).font(fonts::MONO_FONT),
            text("").size(16),

            // Migration Status
            text("iced 0.14 Migration - Phase 6 Complete").size(16).color([0.6, 0.8, 0.6]),
            text("✓ Phase 1-4: Foundation, Widgets, Appearance, SaveManager").size(13).color([0.7, 0.7, 0.7]),
            text("✓ Phase 5-6: All 25 pages implemented").size(13).color([0.7, 0.7, 0.7]),
            text("✓ Navigation: Modern horizontal tabs with search").size(13).color([0.7, 0.7, 0.7]),
            text("→ Next: Complex widgets (Phase 7) & Search (Phase 9)").size(13).color([0.7, 0.7, 0.7]),
            text("").size(12),
            text("Scroll down to see widget demonstration →").size(13).color([0.6, 0.7, 0.9]),
            text("").size(16),
        ]
        .spacing(6);

        // Combine summary with widget demo
        let page_view = column![summary, views::widget_demo::view(&self.ui.widget_demo_state),]
            .spacing(20);

        container(page_view)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Mark that settings have changed (triggers debounced save)
    pub(crate) fn mark_changed(&mut self) {
        self.last_change_time = Some(std::time::Instant::now());
    }

    /// Check if we should save now (debounce: 300ms since last change)
    fn should_save(&self) -> bool {
        if self.save_in_progress || !self.dirty_tracker.is_dirty() {
            return false;
        }

        match self.last_change_time {
            Some(t) => t.elapsed() >= Duration::from_millis(300),
            None => false,
        }
    }

    /// Create an async save task
    fn save_task(&mut self) -> Task<Message> {
        self.save_in_progress = true;
        let settings = self.settings.clone();
        let dirty = self.dirty_tracker.take();
        let paths = self.paths.clone();

        Task::perform(
            async move {
                match crate::config::save_dirty(&paths, &settings, &dirty) {
                    Ok(count) => SaveResult::Success {
                        files_written: count,
                        categories: dirty.into_iter().collect(),
                    },
                    Err(e) => SaveResult::Error { message: e.to_string() },
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
                    Err(e) => ReloadResult::Error { message: e.to_string() },
                }
            },
            Message::ReloadCompleted,
        )
    }

    /// Apply window rule consolidation - merge multiple rules into one
    fn apply_window_rule_consolidation(&mut self, suggestion: &crate::messages::ConsolidationSuggestion) {
        use crate::config::models::WindowRuleMatch;

        // Get the first rule ID to keep (will be modified to use merged pattern)
        let Some(&first_id) = suggestion.rule_ids.first() else { return };

        // Find the first rule and update its match pattern
        if let Some(rule) = self.settings
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
    fn apply_layer_rule_consolidation(&mut self, suggestion: &crate::messages::ConsolidationSuggestion) {
        use crate::config::models::LayerRuleMatch;

        // Get the first rule ID to keep
        let Some(&first_id) = suggestion.rule_ids.first() else { return };

        // Find the first rule and update its match pattern
        if let Some(rule) = self.settings
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
        .theme(|app: &App| app.ui.current_theme.to_iced_theme())
        .settings(iced::Settings {
            id: Some("niri-settings".to_string()),
            ..Default::default()
        })
        .window(iced::window::Settings {
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "niri-settings".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}
