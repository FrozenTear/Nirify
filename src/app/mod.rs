//! Main application module - Elm Architecture implementation
//!
//! This module implements the core Application logic:
//! - State management (App struct)
//! - Message handling (update function)
//! - UI construction (view function)

mod handlers;
mod helpers;
pub mod ui_state;

pub use ui_state::UiState;
use ui_state::{ErrorBanner, ErrorBannerKind};

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use iced::time;
use iced::widget::{column, container, row, stack, text};
use iced::{alignment::Horizontal, Element, Length, Subscription, Task};

use crate::config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::messages::{DialogState, Message, Page, SaveMessage};
use crate::save_manager::{ReloadResult, SaveResult};
use crate::theme::{fonts, neon};
use crate::views;

pub struct SaveState {
    /// Tracks which settings categories have unsaved changes
    pub dirty_tracker: DirtyTracker,
    /// Last time a change was made (for debounced save)
    pub last_change_time: Option<std::time::Instant>,
    /// Whether a save is currently in progress
    pub in_progress: bool,
    /// Categories taken by the currently in-air async save (recovered on failure/exit)
    pub in_flight: HashSet<SettingsCategory>,
    /// Categories already backed up this session (first-write-per-session policy)
    pub backed_up: HashSet<SettingsCategory>,
    /// Load-failed categories excluded from saving (would overwrite an unread file)
    pub blocked: HashSet<SettingsCategory>,
    /// Last save-failure time, for retry backoff (retry at most every 5s)
    pub last_failure_time: Option<std::time::Instant>,
}

impl SaveState {
    fn new() -> Self {
        Self {
            dirty_tracker: DirtyTracker::new(),
            last_change_time: None,
            in_progress: false,
            in_flight: HashSet::new(),
            backed_up: HashSet::new(),
            blocked: HashSet::new(),
            last_failure_time: None,
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

/// Whether an outputs message is a "risky" live-applied change that should arm
/// the revert countdown (disabling an output or changing its mode).
fn output_msg_is_risky(m: &crate::messages::OutputsMessage) -> bool {
    use crate::messages::OutputsMessage as O;
    matches!(
        m,
        O::SetEnabled(_, false)
            | O::SetMode(_, _)
            | O::SetModeCustom(_, _)
            | O::SetModeline(_, _)
            // Bulk layout import can move every monitor; arm the same 15s
            // apply-then-confirm used for mode / disable. Per-output
            // SetPositionX/Y stay unarmed so the drag canvas does not
            // pop the countdown on every snap.
            | O::LiveOutputsSnapshotLoaded(Ok(_))
    )
}

/// Whether a keybindings message commits a new key combo / modifier set and
/// should arm the revert countdown.
fn keybinding_msg_is_risky(m: &crate::messages::KeybindingsMessage) -> bool {
    use crate::messages::KeybindingsMessage as K;
    matches!(m, K::CapturedKey(_) | K::UpdateModifiers(_, _))
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

        // Absorb any hand-edited managed nodes still sitting in config.kdl
        // into nirify/*.kdl, then strip them. Safe to call every time — it
        // early-returns when there is nothing to merge or rewrite.
        if paths.niri_config.exists() && paths.managed_dir.exists() {
            // Version is not known yet (async IPC). all_enabled lets adopted
            // blur/recent-windows files be written; main.kdl is not rewritten
            // when it already exists.
            match crate::config::absorb_stripped_nodes(
                &paths,
                crate::version::FeatureCompat::all_enabled(),
            ) {
                Ok(result) => {
                    if result.replace.replaced_count > 0
                        || result.replace.include_added
                        || !result.adopted.is_empty()
                    {
                        log::info!(
                            "Config updated: {} managed nodes replaced, {} preserved, include added: {}, {} categor(ies) absorbed",
                            result.replace.replaced_count,
                            result.replace.preserved_count,
                            result.replace.include_added,
                            result.adopted.len()
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

        // Load settings from disk, tracking which files failed to parse/read so we
        // can pause saving those categories (avoids overwriting an unread file).
        let load_result = crate::config::load_settings_with_result(&paths);
        let settings = load_result.settings;
        let blocked: HashSet<SettingsCategory> = load_result
            .failed_files
            .iter()
            .filter_map(|f| SettingsCategory::from_relative_path(f))
            .collect();
        log::info!("Settings loaded ({} file(s) failed to read)", blocked.len());

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

        // Startup IPC is async to avoid blocking the UI thread on
        // startup. Begin Disconnected with unknown version; the async
        // NiriStatusChecked → VersionLoaded flow refreshes status/version/compat.
        let niri_status = crate::views::status_bar::NiriStatus::Disconnected;
        let niri_version: Option<crate::version::NiriVersion> = None;
        let feature_compat = crate::version::FeatureCompat::from_version(niri_version);

        // Ensure all required config files exist (handles upgrades from older versions)
        // This creates any missing .kdl files that main.kdl includes
        if paths.managed_dir.exists() {
            match crate::config::ensure_required_files_exist(&paths, &settings, feature_compat) {
                Ok(created) if !created.is_empty() => {
                    log::info!(
                        "Created {} missing config file(s): {:?}",
                        created.len(),
                        created
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("Failed to create missing config files: {}", e);
                }
            }
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
        // Seed the exact scroll-factor entry buffers from the loaded model.
        ui.mouse_scroll_factor_text = format!("{}", settings.mouse.scroll_factor);
        ui.touchpad_scroll_factor_text = format!("{}", settings.touchpad.scroll_factor);

        // Check if this is the first run and show the wizard
        if paths.is_first_run() {
            log::info!("First run detected - showing setup wizard");
            ui.dialog_state = DialogState::FirstRunWizard {
                step: crate::messages::WizardStep::Welcome,
            };
        }

        let mut app = Self {
            settings,
            paths,
            save: SaveState::new(),
            search_index: crate::search::SearchIndex::new(),
            ui,
        };

        // Pause saving for categories whose file could not be read, and surface it.
        app.save.blocked = blocked;
        if !app.save.blocked.is_empty() {
            let mut details: Vec<String> = load_result.failed_files.clone();
            details.push(
                "Those pages show defaults. Saving them is paused so your file on disk \
                 is not overwritten."
                    .to_string(),
            );
            app.ui.error_banner = Some(ErrorBanner {
                kind: ErrorBannerKind::LoadFailed,
                title: "Some settings files could not be read".to_string(),
                details,
            });
        }

        // Record whether niri's config already includes our settings file, to
        // drive the "setup incomplete" banner.
        app.refresh_include_line_present();

        // Kick off async niri detection; on first connection NiriStatusChecked
        // fetches windows/workspaces and the version.
        let startup_task = crate::ipc::tasks::check_niri_running(Message::NiriStatusChecked);

        (app, startup_task)
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
        // Arm the revert countdown BEFORE the handler applies any risky change,
        // so the snapshot captures pre-change state.
        self.maybe_arm_revert(&message);

        match message {
            Message::NoOp => Task::none(),

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
                    return crate::ipc::tasks::get_full_outputs_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::OutputsLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                }

                // Auto-refresh workspaces when navigating to Window Rules page
                // (for the workspace dropdown)
                if page == Page::WindowRules && is_connected {
                    return crate::ipc::tasks::get_workspaces_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::WorkspacesLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
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
                self.ui.search_selected_index = 0;
                self.ui.search_focused = true;

                // Perform search immediately
                self.ui.search_results = self.search_index.search(&self.ui.search_query);

                // Re-focus search input to maintain typing focus
                iced::widget::operation::focus(views::navigation::search_input_id())
            }

            Message::SearchResultSelected(index) => {
                if let Some(result) = self.ui.search_results.get(index).cloned() {
                    self.apply_search_destination(result.destination);
                    self.ui.search_query.clear();
                    self.ui.search_results.clear();
                    self.ui.search_focused = false;
                    self.ui.search_selected_index = 0;
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
                self.ui.search_focused = !self.ui.search_focused;
                if self.ui.search_focused {
                    // Bar path: focus the sidebar field. Modal path: same id
                    // is mounted on the overlay input.
                    iced::widget::operation::focus(views::navigation::search_input_id())
                } else {
                    self.ui.search_query.clear();
                    self.ui.search_results.clear();
                    self.ui.search_selected_index = 0;
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
            Message::Blur(msg) => self.update_blur(msg),

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
                self.save.in_flight.clear();
                match result {
                    SaveResult::Success {
                        files_written,
                        categories,
                    } => {
                        // Remember which categories were backed up so subsequent
                        // saves this session don't re-snapshot them.
                        self.save.backed_up.extend(categories.iter().copied());
                        self.save.last_failure_time = None;
                        // Clear any save/validation banner now that a save succeeded.
                        if matches!(
                            self.ui.error_banner.as_ref().map(|b| &b.kind),
                            Some(ErrorBannerKind::SaveFailed)
                                | Some(ErrorBannerKind::ValidationBlocked)
                        ) {
                            self.ui.error_banner = None;
                        }
                        self.ui.toast = Some(format!("Saved {} file(s)", files_written));
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                        // Trigger niri config reload
                        self.reload_niri_config_task()
                    }
                    SaveResult::Error {
                        message,
                        categories,
                    } => {
                        // Keep the dirty set: re-mark so the changes retry.
                        self.save.dirty_tracker.mark_many(&categories);
                        self.save.last_failure_time = Some(std::time::Instant::now());
                        self.ui.error_banner = Some(ErrorBanner {
                            kind: ErrorBannerKind::SaveFailed,
                            title: "Saving failed".to_string(),
                            details: vec![
                                message,
                                "Your changes are kept in memory and will be retried \
                                 automatically."
                                    .to_string(),
                            ],
                        });
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
                        // A successful reload clears a prior niri-rejection banner.
                        if matches!(
                            self.ui.error_banner.as_ref().map(|b| &b.kind),
                            Some(ErrorBannerKind::NiriRejected)
                        ) {
                            self.ui.error_banner = None;
                        }
                    }
                    ReloadResult::Error { message } => {
                        log::warn!("Failed to reload niri config: {}", message);
                        // Only surface when we believe niri is actually connected;
                        // otherwise it's just "niri not running", which is expected.
                        if matches!(
                            self.ui.niri_status,
                            crate::views::status_bar::NiriStatus::Connected
                        ) {
                            self.ui.error_banner = Some(ErrorBanner {
                                kind: ErrorBannerKind::NiriRejected,
                                title: "niri rejected the configuration".to_string(),
                                details: vec![message],
                            });
                        }
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
                // Defense in depth: never silently dismiss the wizard before the
                // include line exists — divert to the skip-warning step instead.
                if matches!(self.ui.dialog_state, DialogState::FirstRunWizard { .. })
                    && !self.paths.has_include_line()
                {
                    self.ui.dialog_state = DialogState::FirstRunWizard {
                        step: crate::messages::WizardStep::SkipWarning,
                    };
                    return Task::none();
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
                            ConfirmAction::DeleteWindowRule(rule_id) => {
                                let rule_id = *rule_id;
                                self.ui.dialog_state = DialogState::None;
                                return self.update(Message::WindowRules(
                                    crate::messages::WindowRulesMessage::DeleteRule(rule_id),
                                ));
                            }
                            ConfirmAction::DeleteLayerRule(rule_id) => {
                                let rule_id = *rule_id;
                                self.ui.dialog_state = DialogState::None;
                                return self.update(Message::LayerRules(
                                    crate::messages::LayerRulesMessage::DeleteRule(rule_id),
                                ));
                            }
                            ConfirmAction::DeleteKeybinding(idx) => {
                                let idx = *idx;
                                self.ui.dialog_state = DialogState::None;
                                return self.update(Message::Keybindings(
                                    crate::messages::KeybindingsMessage::RemoveKeybinding(idx),
                                ));
                            }
                            ConfirmAction::DeleteOutput(idx) => {
                                let idx = *idx;
                                self.ui.dialog_state = DialogState::None;
                                return self.update(Message::Outputs(
                                    crate::messages::OutputsMessage::RemoveOutput(idx),
                                ));
                            }
                            ConfirmAction::DeleteWorkspace(idx) => {
                                let idx = *idx;
                                self.ui.dialog_state = DialogState::None;
                                return self.update(Message::Workspaces(
                                    crate::messages::WorkspacesMessage::RemoveWorkspace(idx),
                                ));
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
                                self.save
                                    .dirty_tracker
                                    .mark(crate::config::SettingsCategory::Keybindings);
                                self.mark_changed();
                            }
                            ConfirmAction::RestoreBackup(idx) => {
                                let idx = *idx;
                                self.ui.dialog_state = DialogState::None;
                                return self.update_backups(
                                    crate::messages::BackupsMessage::RestoreBackup(idx),
                                );
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
                        // Skip-warning has no "next"; treat as staying on Welcome.
                        WizardStep::SkipWarning => WizardStep::Welcome,
                    };
                    self.ui.dialog_state = DialogState::FirstRunWizard { step: next_step };
                }
                Task::none()
            }

            Message::WizardBack => {
                // Go back to previous wizard step
                if let DialogState::FirstRunWizard { step } = &self.ui.dialog_state {
                    use crate::messages::WizardStep;
                    let step = step.clone();
                    let prev_step = match step {
                        WizardStep::Welcome => {
                            // Do not silently dismiss before the include line exists.
                            if self.paths.has_include_line() {
                                self.ui.dialog_state = DialogState::None;
                                return Task::none();
                            }
                            WizardStep::SkipWarning
                        }
                        // From the skip warning, "Go back to setup" returns to Welcome.
                        WizardStep::SkipWarning => WizardStep::Welcome,
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
                // Import the user's current config *before* stripping managed
                // nodes, then write those imported settings (not empty defaults).
                log::info!("Wizard: Setting up config...");

                match crate::config::first_run_setup(&self.paths, self.ui.feature_compat) {
                    Ok(result) => {
                        log::info!("Wizard import: {}", result.import.summary());
                        for warning in &result.import.warnings {
                            log::warn!("Import warning: {}", warning);
                        }
                        log::info!(
                            "Smart replace complete: {} nodes replaced, {} preserved, backup at {:?}",
                            result.replace.replaced_count,
                            result.replace.preserved_count,
                            result.replace.backup_path
                        );
                        for warning in &result.replace.warnings {
                            log::warn!("Smart replace warning: {}", warning);
                        }
                        self.ui.wizard_import =
                            Some(ui_state::WizardImportSummary::from_import(&result.import));
                        self.settings = result.import.settings;
                    }
                    Err(e) => {
                        log::error!("Failed to set up config: {}", e);
                        self.ui.dialog_state = DialogState::Error {
                            title: "Setup Error".to_string(),
                            message: "Failed to import settings and set up config.kdl.".to_string(),
                            details: Some(e.to_string()),
                        };
                        return Task::none();
                    }
                }

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

                // Config now includes our settings file.
                self.refresh_include_line_present();

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
                            self.save
                                .dirty_tracker
                                .mark(crate::config::SettingsCategory::WindowRules);
                        }
                        if layer_rules_changed {
                            self.save
                                .dirty_tracker
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
                // Flush any changes that would otherwise be lost: still-dirty plus
                // the in-flight (possibly-lost) async save, minus load-blocked.
                let mut cats = self.save.dirty_tracker.take();
                cats.extend(self.save.in_flight.iter().copied());
                cats.retain(|c| !self.save.blocked.contains(c));

                if !cats.is_empty() {
                    let validation = crate::config::validation::validate_settings(&self.settings);
                    if validation.is_valid() {
                        let needs_backup: HashSet<_> =
                            cats.difference(&self.save.backed_up).copied().collect();
                        match crate::config::save_dirty(
                            &self.paths,
                            &self.settings,
                            &cats,
                            self.ui.feature_compat,
                            &needs_backup,
                        ) {
                            Ok(n) => log::info!("Saved {} file(s) before exit", n),
                            Err(e) => log::error!("Failed to save on exit: {}", e),
                        }
                    } else {
                        log::error!(
                            "Skipping exit save: validation errors: {:?}",
                            validation.errors
                        );
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
                let was_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                self.ui.niri_status = if is_connected {
                    crate::views::status_bar::NiriStatus::Connected
                } else {
                    crate::views::status_bar::NiriStatus::Disconnected
                };
                // On first connection, fetch dashboard data
                if is_connected && !was_connected {
                    let t1 = crate::ipc::tasks::get_windows_async(|r| {
                        Message::Tools(crate::messages::ToolsMessage::WindowsLoaded(
                            r.map_err(|e| e.to_string()),
                        ))
                    });
                    let t2 = crate::ipc::tasks::get_workspaces_async(|r| {
                        Message::Tools(crate::messages::ToolsMessage::WorkspacesLoaded(
                            r.map_err(|e| e.to_string()),
                        ))
                    });
                    let t3 = crate::ipc::tasks::get_version_async(|r| {
                        Message::Tools(crate::messages::ToolsMessage::VersionLoaded(
                            r.map_err(|e| e.to_string()),
                        ))
                    });
                    return Task::batch([t1, t2, t3]);
                }
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

            Message::DismissErrorBanner => {
                self.ui.error_banner = None;
                Task::none()
            }

            Message::OverwriteFailedCategories => {
                // Re-enable saving for load-blocked categories and force a save.
                // The first-write backup (F-E) snapshots the corrupt file first.
                let blocked = std::mem::take(&mut self.save.blocked);
                let cats: Vec<SettingsCategory> = blocked.into_iter().collect();
                self.save.dirty_tracker.mark_many(&cats);
                self.mark_changed();
                if matches!(
                    self.ui.error_banner.as_ref().map(|b| &b.kind),
                    Some(ErrorBannerKind::LoadFailed)
                ) {
                    self.ui.error_banner = None;
                }
                Task::none()
            }

            // Redesign navigation
            Message::NavigateToScreen(screen) => {
                use crate::messages::Screen;
                self.ui.current_screen = screen;
                self.ui.highlight_setting = None;
                self.ui.search_focused = false;
                self.ui.search_query.clear();
                self.ui.search_results.clear();

                let is_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );

                // Auto-refresh IPC data for relevant screens
                if screen == Screen::Dashboard && is_connected {
                    // Fetch windows + workspaces for dashboard stats
                    let t1 = crate::ipc::tasks::get_windows_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::WindowsLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                    let t2 = crate::ipc::tasks::get_workspaces_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::WorkspacesLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                    return Task::batch([t1, t2]);
                }
                if screen == Screen::Displays && is_connected {
                    return crate::ipc::tasks::get_full_outputs_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::OutputsLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                }
                if screen == Screen::Rules && is_connected {
                    return crate::ipc::tasks::get_workspaces_async(|result| {
                        Message::Tools(crate::messages::ToolsMessage::WorkspacesLoaded(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                }
                Task::none()
            }
            Message::SetInputSubTab(tab) => {
                self.ui.input_sub_tab = tab;
                self.ui.highlight_setting = None;
                Task::none()
            }
            Message::OpenSectionEditor(section) => {
                self.ui.editing_section = Some(section);
                Task::none()
            }
            Message::CloseSectionEditor => {
                self.ui.editing_section = None;
                Task::none()
            }
            Message::OpenDeviceEditor(device) => {
                self.ui.editing_device = Some(device);
                Task::none()
            }
            Message::CloseDeviceEditor => {
                self.ui.editing_device = None;
                Task::none()
            }
            Message::OpenKeybindingEditor(idx) => {
                self.ui.editing_keybinding_index = Some(idx);
                self.ui.selected_keybinding_index = Some(idx);
                Task::none()
            }
            Message::CloseKeybindingEditor => {
                self.ui.editing_keybinding_index = None;
                Task::none()
            }
            Message::SetKeybindingsSearch(text) => {
                self.ui.keybindings_search = text;
                Task::none()
            }
            Message::SetRulesSubTab(tab) => {
                self.ui.rules_sub_tab = tab;
                self.ui.highlight_setting = None;
                Task::none()
            }
            Message::SetGearSubTab(tab) => {
                self.ui.gear_sub_tab = tab;
                self.ui.highlight_setting = None;
                Task::none()
            }

            // ── UX safety: revert countdown ──────────────────────────────────
            Message::RevertTick => {
                if let Some(pr) = self.ui.pending_revert.as_mut() {
                    pr.seconds_left = pr.seconds_left.saturating_sub(1);
                    if pr.seconds_left == 0 {
                        return self.update(Message::RevertNow);
                    }
                }
                Task::none()
            }
            Message::RevertKeep => {
                self.ui.pending_revert = None;
                if matches!(self.ui.dialog_state, DialogState::RevertCountdown { .. }) {
                    self.ui.dialog_state = DialogState::None;
                }
                Task::none()
            }
            Message::RevertNow => {
                if let Some(pr) = self.ui.pending_revert.take() {
                    use crate::app::ui_state::RevertSnapshot;
                    match pr.snapshot {
                        RevertSnapshot::Outputs(s) => {
                            self.settings.outputs = s;
                            self.save.dirty_tracker.mark(SettingsCategory::Outputs);
                        }
                        RevertSnapshot::Keybindings(s) => {
                            self.settings.keybindings = s;
                            self.save.dirty_tracker.mark(SettingsCategory::Keybindings);
                        }
                    }
                    self.mark_changed();
                    self.ui.toast = Some("Changes reverted".to_string());
                    self.ui.toast_shown_at = Some(std::time::Instant::now());
                }
                if matches!(self.ui.dialog_state, DialogState::RevertCountdown { .. }) {
                    self.ui.dialog_state = DialogState::None;
                }
                Task::none()
            }

            // ── Search keyboard navigation ───────────────────────────────────
            Message::SearchNavUp => {
                if self.ui.search_selected_index > 0 {
                    self.ui.search_selected_index -= 1;
                }
                Task::none()
            }
            Message::SearchNavDown => {
                self.ui.search_selected_index = crate::search::clamp_selected_index(
                    self.ui.search_selected_index + 1,
                    self.ui.search_results.len(),
                );
                Task::none()
            }
            Message::SearchNavActivate => {
                if self.search_ui_visible() && !self.ui.search_results.is_empty() {
                    let idx = crate::search::clamp_selected_index(
                        self.ui.search_selected_index,
                        self.ui.search_results.len(),
                    );
                    return self.update(Message::SearchResultSelected(idx));
                }
                Task::none()
            }
            Message::EscapePressed => self.handle_escape(),

            Message::WizardSkipConfirmed => {
                self.ui.dialog_state = DialogState::None;
                self.refresh_include_line_present();
                Task::none()
            }
        }
    }

    /// True when a search surface the user can type into is actually on screen.
    fn search_ui_visible(&self) -> bool {
        if self.ui.show_search_bar {
            true
        } else {
            self.ui.search_focused
        }
    }

    /// True when search is the active overlay (arrows / Enter should apply).
    fn search_overlay_active(&self) -> bool {
        if self.ui.show_search_bar {
            self.ui.search_focused || !self.ui.search_query.trim().is_empty()
        } else {
            self.ui.search_focused
        }
    }

    /// Switch to the redesigned screen and open the matching editor when possible.
    fn apply_search_destination(&mut self, dest: crate::search::SearchDestination) {
        use crate::search::SearchDestination;

        self.ui.current_screen = dest.screen();
        self.ui.highlight_setting = None;
        self.ui.editing_section = None;
        self.ui.editing_device = None;

        match dest {
            SearchDestination::Section(section) => {
                self.ui.editing_section = Some(section);
            }
            SearchDestination::Device(device) => {
                self.ui.editing_device = Some(device);
            }
            SearchDestination::Keybindings => {}
            SearchDestination::Displays => {}
            SearchDestination::Rules(tab) => {
                self.ui.rules_sub_tab = tab;
            }
            SearchDestination::Gear(tab) => {
                self.ui.gear_sub_tab = tab;
            }
        }
    }

    /// True when any editor modal is currently open.
    fn any_editor_open(&self) -> bool {
        self.ui.editing_window_rule_id.is_some()
            || self.ui.editing_layer_rule_id.is_some()
            || self.ui.editing_keybinding_index.is_some()
            || self.ui.editing_section.is_some()
            || self.ui.editing_output_index.is_some()
            || self.ui.editing_device.is_some()
    }

    /// Refresh whether the niri include line is present (cheap sync read).
    /// `None` when the niri config file does not exist (non-niri machine).
    fn refresh_include_line_present(&mut self) {
        self.ui.include_line_present = if self.paths.niri_config.exists() {
            Some(self.paths.has_include_line())
        } else {
            None
        };
    }

    /// Handle Escape: close the topmost overlay layer in priority order.
    fn handle_escape(&mut self) -> Task<Message> {
        // a. Revert countdown → revert (safe default)
        if matches!(self.ui.dialog_state, DialogState::RevertCountdown { .. }) {
            return self.update(Message::RevertNow);
        }
        // b. Wizard → route through skip-guard, never plain close
        if matches!(self.ui.dialog_state, DialogState::FirstRunWizard { .. }) {
            return self.update(Message::WizardBack);
        }
        // c. Any other non-None dialog → close
        if self.ui.dialog_state != DialogState::None {
            return self.update(Message::CloseDialog);
        }
        // d. Search modal open → close
        if self.ui.search_focused {
            return self.update(Message::ToggleSearch);
        }
        // e. Editor modal open → dispatch its close message
        if self.ui.editing_window_rule_id.is_some() {
            return self.update(Message::WindowRules(
                crate::messages::WindowRulesMessage::CloseEditor,
            ));
        }
        if self.ui.editing_layer_rule_id.is_some() {
            return self.update(Message::LayerRules(
                crate::messages::LayerRulesMessage::CloseEditor,
            ));
        }
        if self.ui.editing_keybinding_index.is_some() {
            return self.update(Message::CloseKeybindingEditor);
        }
        if self.ui.editing_section.is_some() {
            return self.update(Message::CloseSectionEditor);
        }
        if self.ui.editing_output_index.is_some() {
            return self.update(Message::Outputs(
                crate::messages::OutputsMessage::CloseEditor,
            ));
        }
        if self.ui.editing_device.is_some() {
            return self.update(Message::CloseDeviceEditor);
        }
        Task::none()
    }

    /// Arm the revert countdown for risky live-applied changes, taking a
    /// snapshot BEFORE the handler applies the change. Called at the top of
    /// `update()` so `self.settings` still holds the pre-change state.
    fn maybe_arm_revert(&mut self, message: &Message) {
        use crate::app::ui_state::{PendingRevert, RevertSnapshot};

        // Never arm while the first-run wizard is open.
        if matches!(self.ui.dialog_state, DialogState::FirstRunWizard { .. }) {
            return;
        }

        enum Cat {
            Outputs,
            Keybindings,
        }
        let armed: Option<(Cat, &'static str)> = match message {
            Message::Outputs(m) => {
                output_msg_is_risky(m).then_some((Cat::Outputs, "Display settings changed"))
            }
            Message::Keybindings(m) => {
                keybinding_msg_is_risky(m).then_some((Cat::Keybindings, "Keybinding changed"))
            }
            _ => None,
        };

        let Some((cat, description)) = armed else {
            return;
        };

        // Same category already pending: keep original snapshot, reset timer.
        match (&mut self.ui.pending_revert, &cat) {
            (Some(pr), Cat::Outputs) if matches!(pr.snapshot, RevertSnapshot::Outputs(_)) => {
                pr.seconds_left = 15;
                pr.description = description.to_string();
                return;
            }
            (Some(pr), Cat::Keybindings)
                if matches!(pr.snapshot, RevertSnapshot::Keybindings(_)) =>
            {
                pr.seconds_left = 15;
                pr.description = description.to_string();
                return;
            }
            _ => {}
        }

        let snapshot = match cat {
            Cat::Outputs => RevertSnapshot::Outputs(self.settings.outputs.clone()),
            Cat::Keybindings => RevertSnapshot::Keybindings(self.settings.keybindings.clone()),
        };
        self.ui.pending_revert = Some(PendingRevert {
            snapshot,
            seconds_left: 15,
            description: description.to_string(),
        });
        // Open the countdown dialog when nothing else is showing, and also
        // refresh its description if a countdown dialog is ALREADY open (a new
        // risky change for the other category re-armed the revert). Don't
        // clobber unrelated dialog types.
        match &self.ui.dialog_state {
            DialogState::None | DialogState::RevertCountdown { .. } => {
                self.ui.dialog_state = DialogState::RevertCountdown {
                    description: description.to_string(),
                };
            }
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = self.base_subscriptions();

        // Add keyboard subscription based on current mode
        if self.ui.key_capture_active.is_some() {
            subs.push(self.key_capture_subscription());
        } else {
            if !self.settings.preferences.search_hotkey.is_empty() {
                subs.push(self.search_hotkey_subscription());
            }
            // Overlay keys (arrows/Escape/Enter) while any overlay UI is active.
            if self.search_overlay_active()
                || self.ui.dialog_state != DialogState::None
                || self.any_editor_open()
            {
                subs.push(self.overlay_keys_subscription());
            }
        }

        Subscription::batch(subs)
    }

    /// Subscription for overlay navigation keys (search arrows, Escape).
    fn overlay_keys_subscription(&self) -> Subscription<Message> {
        use iced::keyboard;
        use iced::keyboard::key::Named;

        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, .. } => match key {
                keyboard::Key::Named(Named::ArrowUp) => Message::SearchNavUp,
                keyboard::Key::Named(Named::ArrowDown) => Message::SearchNavDown,
                keyboard::Key::Named(Named::Escape) => Message::EscapePressed,
                _ => Message::NoOp,
            },
            _ => Message::NoOp,
        })
    }

    /// Base subscriptions always active: save checks, niri status, toast clearing, system theme
    fn base_subscriptions(&self) -> Vec<Subscription<Message>> {
        let mut subs = vec![
            // Periodic save checks (every 200ms - sufficient with 300ms debounce)
            time::every(Duration::from_millis(200)).map(|_| Message::Save(SaveMessage::CheckSave)),
            // Niri status check (every 5 seconds)
            time::every(Duration::from_secs(5)).map(|_| Message::CheckNiriStatus),
            // System theme detection (portal or file watcher)
            crate::system_theme::subscription().map(Message::SystemThemeEvent),
            // Window close requests (exit_on_close_request is disabled so we can
            // flush unsaved changes before exiting).
            iced::window::close_requests().map(|_id| Message::WindowCloseRequested),
        ];

        // Toast auto-clear check (every 500ms, only when toast is showing)
        if self.ui.toast.is_some() {
            subs.push(time::every(Duration::from_millis(500)).map(|_| Message::ClearToast));
        }

        // Revert countdown tick (every 1s while a revert is pending)
        if self.ui.pending_revert.is_some() {
            subs.push(time::every(Duration::from_secs(1)).map(|_| Message::RevertTick));
        }

        subs
    }

    /// Subscription for key capture mode (when recording keybindings)
    fn key_capture_subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed {
                key,
                modifiers,
                location,
                ..
            } => {
                // ESC cancels capture
                if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    return Message::Keybindings(
                        crate::messages::KeybindingsMessage::CancelKeyCapture,
                    );
                }

                // Convert key and modifiers to a key combo string
                let key_combo = helpers::format_key_combo(&key, modifiers, location);

                // Only capture if we got a valid key (not just a modifier)
                if !key_combo.is_empty() {
                    Message::Keybindings(crate::messages::KeybindingsMessage::CapturedKey(
                        key_combo,
                    ))
                } else {
                    Message::NoOp
                }
            }
            _ => Message::NoOp,
        })
    }

    /// Subscription for search hotkey (when not in key capture mode)
    fn search_hotkey_subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        let search_hotkey = self.settings.preferences.search_hotkey.clone();

        keyboard::listen()
            .with(search_hotkey)
            .map(|(hotkey, event): (String, keyboard::Event)| match event {
                keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    location,
                    ..
                } => {
                    let key_combo = helpers::format_key_combo(&key, modifiers, location);
                    if helpers::hotkey_matches(&key_combo, &hotkey) {
                        Message::ToggleSearch
                    } else {
                        Message::NoOp
                    }
                }
                _ => Message::NoOp,
            })
    }

    /// Constructs the UI from current state
    pub fn view(&self) -> Element<'_, Message> {
        // Sidebar navigation
        let sidebar = views::sidebar::view(
            self.ui.current_screen,
            &self.ui.search_query,
            self.ui.show_search_bar,
        );

        // Main content area
        let content_area = self.screen_content();

        // Status bar (bottom)
        let is_dirty = self.save.dirty_tracker.is_dirty();
        let save_status = self.ui.toast.clone();
        let status_bar = views::status_bar::view(
            is_dirty,
            save_status,
            self.ui.current_theme,
            self.ui.niri_status,
        );

        // Sidebar + content/status stacked horizontally
        let right_side = column![content_area, status_bar].spacing(0);
        let layout = row![sidebar, right_side];

        let main_view: Element<'_, Message> = container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // Check for search modal (when search bar is hidden but search is active)
        let with_search_modal = if !self.ui.show_search_bar && self.ui.search_focused {
            use crate::theme::{fonts, search_dropdown_style};
            use iced::widget::{column as col, row, text_input, Space};

            // Build search input
            let search_input = text_input("Search settings...", &self.ui.search_query)
                .id(views::navigation::search_input_id())
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::SearchNavActivate)
                .padding(12)
                .size(16)
                .width(Length::Fixed(400.0));

            // Build results list if there are results
            let results_content: Element<'_, Message> = if !self.ui.search_query.trim().is_empty() {
                if self.ui.search_results.is_empty() {
                    container(
                        text("No matching settings found")
                            .size(14)
                            .style(crate::views::dialogs::muted_text_style),
                    )
                    .padding(16)
                    .into()
                } else {
                    let mut results_col = col![].spacing(4).padding(8);
                    let selected = self.ui.search_selected_index;
                    for (index, result) in self
                        .ui
                        .search_results
                        .iter()
                        .take(crate::search::MAX_VISIBLE_RESULTS)
                        .enumerate()
                    {
                        let is_selected = index == selected;
                        let item = iced::widget::button(
                            row![
                                col![
                                    text(&result.setting_name)
                                        .size(14)
                                        .font(fonts::UI_FONT_MEDIUM),
                                    text(&result.description)
                                        .size(11)
                                        .style(crate::views::dialogs::muted_text_style),
                                ]
                                .spacing(2)
                                .width(Length::Fill),
                                text(result.destination.location_label())
                                    .size(10)
                                    .style(crate::views::dialogs::muted_text_style),
                            ]
                            .spacing(8)
                            .padding([10, 12]),
                        )
                        .on_press(Message::SearchResultSelected(index))
                        .width(Length::Fill)
                        .style(move |theme: &iced::Theme, status| {
                            if is_selected {
                                let palette = theme.extended_palette();
                                iced::widget::button::Style {
                                    background: Some(iced::Background::Color(
                                        palette.primary.weak.color,
                                    )),
                                    text_color: palette.primary.weak.text,
                                    border: iced::Border {
                                        radius: 6.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            } else {
                                crate::theme::search_dropdown_item_style()(theme, status)
                            }
                        });

                        results_col = results_col.push(item);
                    }
                    if self.ui.search_results.len() > crate::search::MAX_VISIBLE_RESULTS {
                        results_col = results_col.push(
                            container(
                                text(format!(
                                    "and {} more...",
                                    self.ui.search_results.len()
                                        - crate::search::MAX_VISIBLE_RESULTS
                                ))
                                .size(12)
                                .style(crate::views::dialogs::muted_text_style),
                            )
                            .padding([8, 16]),
                        );
                    }
                    results_col.into()
                }
            } else {
                container(
                    text("Type to search settings...")
                        .size(13)
                        .style(crate::views::dialogs::muted_text_style),
                )
                .padding(16)
                .into()
            };

            // Build the modal
            let modal_content = col![
                row![text("🔍").size(16), search_input,]
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
                        .height(Length::Fill),
                )
                .on_press(Message::ToggleSearch), // Click backdrop to close
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.5,
                ))),
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
            if let Some(dropdown) = views::search_dropdown::view(
                &self.ui.search_results,
                &self.ui.search_query,
                self.ui.search_selected_index,
            ) {
                use iced::widget::{column as col, row as row_w, Space};
                // Sit the dropdown next to the sidebar search field
                let dropdown_overlay = row_w![
                    Space::new().width(Length::Fixed(220.0)),
                    col![
                        Space::new().height(Length::Fixed(88.0)),
                        container(dropdown).padding([0, 8]),
                    ],
                ];

                stack![with_search_modal, dropdown_overlay].into()
            } else {
                with_search_modal
            }
        } else {
            with_search_modal
        };

        // If there's a rule editor modal, render it on top
        let with_rule_editor = if let Some(rule_id) = self.ui.editing_window_rule_id {
            if let Some(rule) = self.settings.window_rules.find(rule_id) {
                let modal = views::window_rules::editor_modal(
                    rule,
                    &self.ui.window_rule_sections_expanded,
                    &self.ui.window_rule_regex_errors,
                    &self.ui.available_workspaces,
                    self.ui.feature_compat.background_effects,
                );
                stack![with_dropdown, modal].into()
            } else {
                with_dropdown
            }
        } else if let Some(rule_id) = self.ui.editing_layer_rule_id {
            if let Some(rule) = self.settings.layer_rules.find(rule_id) {
                let modal = views::layer_rules::editor_modal(
                    rule,
                    &self.ui.layer_rule_sections_expanded,
                    &self.ui.layer_rule_regex_errors,
                    self.ui.feature_compat.background_effects,
                );
                stack![with_dropdown, modal].into()
            } else {
                with_dropdown
            }
        } else if let Some(idx) = self.ui.editing_keybinding_index {
            if let Some(binding) = self.settings.keybindings.bindings.get(idx) {
                let modal = views::keybindings::editor_modal(
                    binding,
                    idx,
                    &self.ui.keybinding_sections_expanded,
                    self.ui.key_capture_active,
                    self.ui.niri_version,
                    self.ui.keybinding_capture_conflict.as_ref(),
                );
                stack![with_dropdown, modal].into()
            } else {
                with_dropdown
            }
        } else if let Some(section) = self.ui.editing_section {
            let content = self.section_editor_content(section);
            let modal = views::screens::section_editor_modal(section, content);
            stack![with_dropdown, modal].into()
        } else if let Some(output_idx) = self.ui.editing_output_index {
            if output_idx < self.settings.outputs.outputs.len() {
                let modal = views::screens::displays::output_editor_modal(
                    output_idx,
                    &self.settings.outputs,
                    &self.ui.output_sections_expanded,
                    &self.ui.tools_state.outputs,
                );
                stack![with_dropdown, modal].into()
            } else {
                with_dropdown
            }
        } else if let Some(device) = self.ui.editing_device {
            let modal =
                views::screens::input::device_editor_modal(device, &self.settings, &self.ui);
            stack![with_dropdown, modal].into()
        } else {
            with_dropdown
        };

        // Persistent banners (setup-incomplete + error banner), full width, on
        // every page. They must live in the BASE content so that when a modal
        // dialog is open its full-screen scrim covers them and their buttons
        // can't be clicked underneath the overlay.
        let base: Element<'_, Message> = if let Some(banners) =
            views::widgets::error_banner::error_banners(
                self.ui.error_banner.as_ref(),
                self.ui.include_line_present,
            ) {
            column![banners, with_rule_editor].into()
        } else {
            with_rule_editor
        };

        // If there's an active dialog, render it as a themed overlay on top of
        // everything (not a full-UI replacement).
        if let Some(dialog) = views::dialogs::view(
            &self.ui.dialog_state,
            &self.ui.wizard_suggestions,
            self.ui.niri_version,
            self.ui.pending_revert.as_ref(),
            self.ui.wizard_import.as_ref(),
        ) {
            stack![base, dialog].into()
        } else {
            base
        }
    }

    /// Returns the view content for a given editable section
    fn section_editor_content(
        &self,
        section: crate::messages::EditableSection,
    ) -> Element<'_, Message> {
        use crate::messages::EditableSection as S;
        match section {
            // Layout sections
            S::SpatialGaps => views::appearance::gaps_section(&self.settings.appearance),
            S::CenteringDynamics => iced::widget::row![
                iced::widget::column![views::behavior::focus_section(&self.settings.behavior),]
                    .width(iced::Length::FillPortion(1)),
                iced::widget::column![views::behavior::workspace_section(&self.settings.behavior),]
                    .width(iced::Length::FillPortion(1)),
            ]
            .spacing(32)
            .align_y(iced::Alignment::Start)
            .into(),
            S::ColumnManager => iced::widget::row![
                iced::widget::column![views::behavior::column_section(&self.settings.behavior),]
                    .width(iced::Length::FillPortion(1)),
                iced::widget::column![views::layout_extras::column_display_section(
                    &self.settings.layout_extras
                ),]
                .width(iced::Length::FillPortion(1)),
            ]
            .spacing(32)
            .align_y(iced::Alignment::Start)
            .into(),
            S::ScreenEdgeStruts => views::behavior::struts_section(&self.settings.behavior),
            S::TabIndicator => {
                views::layout_extras::tab_indicator_section(&self.settings.layout_extras)
            }
            S::InsertHint => {
                views::layout_extras::insert_hint_section(&self.settings.layout_extras)
            }
            S::NamedWorkspaces => views::workspaces::view(&self.settings.workspaces),
            S::PresetSizes => iced::widget::row![
                iced::widget::column![views::layout_extras::preset_widths_section(
                    &self.settings.layout_extras
                )]
                .width(iced::Length::FillPortion(1)),
                iced::widget::column![views::layout_extras::preset_heights_section(
                    &self.settings.layout_extras
                )]
                .width(iced::Length::FillPortion(1)),
            ]
            .spacing(32)
            .align_y(iced::Alignment::Start)
            .into(),
            // Visuals sections
            S::FocusRing => views::appearance::focus_ring_section(&self.settings.appearance),
            S::WindowBorder => views::appearance::border_section(&self.settings.appearance),
            S::WindowShadow => views::layout_extras::shadow_section(&self.settings.layout_extras),
            S::ModifierKeys => views::behavior::modifier_keys_section(&self.settings.behavior),
            S::Animations => views::animations::view(&self.settings.animations),
            S::Cursor => views::cursor::view(&self.settings.cursor),
            S::Blur => views::blur::view(&self.settings.blur, self.ui.feature_compat.blur),
            S::WorkspaceBackground => {
                views::appearance::background_color_section(&self.settings.appearance)
            }
            S::Overview => views::overview::section(&self.settings.overview),
            // System sections
            S::StartupPrograms => views::startup::view_section(&self.settings.startup),
            S::EnvironmentVars => views::environment::view_section(&self.settings.environment),
            S::Miscellaneous => views::miscellaneous::view_section(&self.settings.miscellaneous),
            S::SwitchEvents => views::switch_events::view_section(&self.settings.switch_events),
            S::Debug => views::debug::view_section(&self.settings.debug),
            S::RecentWindows => views::recent_windows::view(&self.settings.recent_windows),
        }
    }

    /// Creates the content area for the current screen (redesign)
    fn screen_content(&self) -> Element<'_, Message> {
        use crate::messages::Screen;
        match self.ui.current_screen {
            Screen::Dashboard => views::screens::dashboard::view(
                self.ui.niri_status,
                self.ui.niri_version,
                &self.ui.tools_state,
                &self.settings,
            ),
            Screen::Layout => views::screens::layout::view(
                &self.settings.layout_extras,
                &self.settings.workspaces,
                &self.settings.behavior,
                &self.settings.appearance,
            ),
            Screen::Visuals => views::screens::visuals::view(
                &self.settings.appearance,
                &self.settings.animations,
                &self.settings.cursor,
                &self.settings.layout_extras,
                &self.settings.behavior,
                &self.settings.blur,
                self.ui.feature_compat.blur,
            ),
            Screen::Input => views::screens::input::view(&self.settings, &self.ui),
            Screen::Rules => views::screens::rules::view(
                self.ui.rules_sub_tab,
                &self.settings.window_rules,
                &self.ui.rules_search,
                self.ui.rules_filter,
                &self.ui.window_rule_sections_expanded,
                &self.ui.window_rule_regex_errors,
                &self.ui.available_workspaces,
                &self.settings.layer_rules,
                &self.ui.layer_rule_sections_expanded,
                &self.ui.layer_rule_regex_errors,
            ),
            Screen::Displays => views::screens::displays::view(
                &self.settings.outputs,
                self.ui.selected_output_index,
                &self.ui.output_sections_expanded,
                &self.ui.tools_state.outputs,
                self.ui.monitor_drag.as_ref().map(|drag| drag.index),
            ),
            Screen::System => views::screens::system::view(
                &self.settings.startup,
                &self.settings.environment,
                &self.settings.miscellaneous,
                &self.settings.switch_events,
                &self.settings.debug,
                &self.settings.recent_windows,
            ),
            Screen::Gear => views::screens::gear::view(
                self.ui.gear_sub_tab,
                &self.ui.tools_state,
                self.ui.niri_status,
                &self.settings.preferences,
                self.ui.show_search_bar,
                self.ui.current_theme,
                &self.ui.config_editor_state,
                &self.ui.config_editor_content,
                &self.ui.backups_state,
            ),
        }
    }

    /// Shows the detailed legacy page for a matched search result.
    #[allow(dead_code)]
    fn search_result_content(&self) -> Element<'_, Message> {
        let setting_name = self.ui.highlight_setting.as_deref().unwrap_or_default();

        let banner = container(
            column![
                row![
                    text("SEARCH MATCH")
                        .size(10)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(neon::SECONDARY),
                    text(self.ui.current_page.name())
                        .size(10)
                        .font(fonts::UI_FONT_MEDIUM)
                        .color(neon::ON_SURFACE_VARIANT),
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
                text(setting_name).size(24).font(fonts::UI_FONT_SEMIBOLD),
                text("Showing the detailed page for the matched setting.")
                    .size(12)
                    .color(neon::ON_SURFACE_VARIANT),
            ]
            .spacing(6),
        )
        .padding(24)
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color {
                    a: 0.18,
                    ..neon::SECONDARY
                },
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        });

        column![container(banner).padding(24), self.page_content()]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Creates the content area for the current page (legacy — used during transition)
    #[allow(dead_code)]
    fn page_content(&self) -> Element<'_, Message> {
        // Each page handles its own scrollable container
        match self.ui.current_page {
            Page::Overview => self.overview_page(),
            Page::Appearance => views::appearance::view(&self.settings.appearance),
            Page::Behavior => views::behavior::view(&self.settings.behavior),
            Page::Keyboard => views::keyboard::view(&self.settings.keyboard),
            Page::Mouse => {
                views::mouse::view(&self.settings.mouse, &self.ui.mouse_scroll_factor_text)
            }
            Page::Touchpad => views::touchpad::view(
                &self.settings.touchpad,
                &self.ui.touchpad_scroll_factor_text,
            ),
            Page::Trackpoint => views::trackpoint::view(&self.settings.trackpoint),
            Page::Trackball => views::trackball::view(&self.settings.trackball),
            Page::Tablet => {
                views::tablet::view(&self.settings.tablet, &self.ui.tablet_calibration_cache)
            }
            Page::Touch => {
                views::touch::view(&self.settings.touch, &self.ui.touch_calibration_cache)
            }
            Page::Animations => views::animations::view(&self.settings.animations),
            Page::Cursor => views::cursor::view(&self.settings.cursor),
            Page::Blur => views::blur::view(&self.settings.blur, self.ui.feature_compat.blur),
            Page::LayoutExtras => views::layout_extras::view(&self.settings.layout_extras),
            Page::Gestures => views::gestures::view(&self.settings.gestures),
            Page::Workspaces => views::workspaces::view(&self.settings.workspaces),
            Page::WindowRules => views::window_rules::view(
                &self.settings.window_rules,
                &self.ui.rules_search,
                self.ui.rules_filter,
                &self.ui.window_rule_sections_expanded,
                &self.ui.window_rule_regex_errors,
                &self.ui.available_workspaces,
            ),
            Page::LayerRules => views::layer_rules::view(
                &self.settings.layer_rules,
                &self.ui.rules_search,
                self.ui.rules_filter,
                &self.ui.layer_rule_sections_expanded,
                &self.ui.layer_rule_regex_errors,
            ),
            Page::Keybindings => views::keybindings::view(
                &self.settings.keybindings,
                self.ui.selected_keybinding_index,
                &self.ui.keybinding_sections_expanded,
                self.ui.key_capture_active,
                self.ui.niri_version,
                self.ui.keybinding_capture_conflict.as_ref(),
            ),
            Page::Outputs => {
                return views::outputs::view(
                    &self.settings.outputs,
                    self.ui.selected_output_index,
                    &self.ui.output_sections_expanded,
                    &self.ui.tools_state.outputs, // IPC data for available modes
                );
            }
            Page::Miscellaneous => views::miscellaneous::view(&self.settings.miscellaneous),
            Page::Startup => views::startup::view(&self.settings.startup),
            Page::Environment => views::environment::view(&self.settings.environment),
            Page::Debug => views::debug::view(&self.settings.debug),
            Page::SwitchEvents => views::switch_events::view(&self.settings.switch_events),
            Page::RecentWindows => views::recent_windows::view(&self.settings.recent_windows),
            Page::Tools => {
                let niri_connected = matches!(
                    self.ui.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                views::tools::view(&self.ui.tools_state, niri_connected)
            }
            Page::Preferences => views::preferences::view(
                self.settings.preferences.float_settings_app,
                self.ui.show_search_bar,
                &self.settings.preferences.search_hotkey,
                self.ui.current_theme,
            ),
            Page::ConfigEditor => views::config_editor::view(
                &self.ui.config_editor_state,
                &self.ui.config_editor_content,
            ),
            Page::Backups => views::backups::view(&self.ui.backups_state),
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
                    slider(crate::constants::OVERVIEW_ZOOM_MIN as f32..=crate::constants::OVERVIEW_ZOOM_MAX as f32, zoom as f32, |v| Message::Overview(OverviewMessage::SetZoom(v as f64)))
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
        if self.save.in_progress {
            return false;
        }

        // Only unblocked dirty categories are savable; blocked-only dirt must not
        // spin the debounce loop.
        let has_unblocked = self
            .save
            .dirty_tracker
            .peek()
            .iter()
            .any(|c| !self.save.blocked.contains(c));
        if !has_unblocked {
            return false;
        }

        // After a failed save, back off: retry at most once every 5 seconds.
        if let Some(f) = self.save.last_failure_time {
            if f.elapsed() < Duration::from_secs(5) {
                return false;
            }
        }

        match self.save.last_change_time {
            Some(t) => t.elapsed() >= Duration::from_millis(300),
            None => false,
        }
    }

    /// Create an async save task
    fn save_task(&mut self) -> Task<Message> {
        // Semantic validation gate: never write settings that fail validation.
        let validation = crate::config::validation::validate_settings(&self.settings);
        if !validation.is_valid() {
            // Stop the debounce loop until the user changes something; dirty flags
            // stay set so the fixed value saves on the next change.
            self.save.last_change_time = None;
            self.ui.error_banner = Some(ErrorBanner {
                kind: ErrorBannerKind::ValidationBlocked,
                title: "Not saving: fix invalid values".to_string(),
                details: validation.errors.iter().map(|e| e.to_string()).collect(),
            });
            return Task::none();
        }
        // Validation passed: clear a prior ValidationBlocked banner.
        if matches!(
            self.ui.error_banner.as_ref().map(|b| &b.kind),
            Some(ErrorBannerKind::ValidationBlocked)
        ) {
            self.ui.error_banner = None;
        }

        self.save.in_progress = true;
        let dirty = self.save.dirty_tracker.take_except(&self.save.blocked);
        if dirty.is_empty() {
            self.save.in_progress = false;
            return Task::none();
        }
        self.save.in_flight = dirty.clone();
        let needs_backup: HashSet<SettingsCategory> =
            dirty.difference(&self.save.backed_up).copied().collect();

        let settings = self.settings.clone();
        let paths = self.paths.clone();
        let feature_compat = self.ui.feature_compat;

        Task::perform(
            async move {
                let categories: Vec<SettingsCategory> = dirty.iter().copied().collect();
                let result = tokio::task::spawn_blocking(move || {
                    crate::config::save_dirty(
                        &paths,
                        &settings,
                        &dirty,
                        feature_compat,
                        &needs_backup,
                    )
                })
                .await;
                match result {
                    Ok(Ok(count)) => SaveResult::Success {
                        files_written: count,
                        categories,
                    },
                    Ok(Err(e)) => SaveResult::Error {
                        message: e.to_string(),
                        categories,
                    },
                    Err(e) => SaveResult::Error {
                        message: format!("Save task panicked: {}", e),
                        categories,
                    },
                }
            },
            Message::SaveCompleted,
        )
    }

    /// Create an async task to reload niri config
    fn reload_niri_config_task(&self) -> Task<Message> {
        crate::ipc::tasks::reload_config_async(|result| {
            Message::ReloadCompleted(match result {
                Ok(()) => ReloadResult::Success,
                Err(e) => ReloadResult::Error {
                    message: e.to_string(),
                },
            })
        })
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
            // We handle close requests ourselves so we can flush unsaved changes.
            exit_on_close_request: false,
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "nirify".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

#[cfg(test)]
mod tests {
    use super::{keybinding_msg_is_risky, output_msg_is_risky};
    use crate::messages::{KeybindingsMessage as K, OutputsMessage as O};

    #[test]
    fn test_output_msg_riskiness() {
        assert!(output_msg_is_risky(&O::SetEnabled(0, false)));
        assert!(!output_msg_is_risky(&O::SetEnabled(0, true)));
        assert!(output_msg_is_risky(&O::SetMode(0, "1920x1080".to_string())));
        assert!(output_msg_is_risky(&O::SetModeline(0, None)));
        assert!(output_msg_is_risky(&O::SetModeCustom(0, true)));
        assert!(!output_msg_is_risky(&O::SetScale(0, 1.5)));
        assert!(!output_msg_is_risky(&O::SetPositionX(0, 100)));
        assert!(!output_msg_is_risky(&O::ImportConnectedLayout));
        assert!(output_msg_is_risky(&O::LiveOutputsSnapshotLoaded(Ok(
            vec![]
        ))));
        assert!(!output_msg_is_risky(&O::LiveOutputsSnapshotLoaded(Err(
            "offline".into()
        ))));
    }

    #[test]
    fn test_keybinding_msg_riskiness() {
        assert!(keybinding_msg_is_risky(&K::CapturedKey(
            "Mod+Q".to_string()
        )));
        assert!(keybinding_msg_is_risky(&K::UpdateModifiers(0, vec![])));
        assert!(!keybinding_msg_is_risky(&K::CancelKeyCapture));
    }

    #[test]
    fn test_search_destination_lands_on_screen_and_editor() {
        use crate::messages::{EditableDevice, EditableSection, GearSubTab, Screen};
        use crate::search::SearchDestination;

        let dest = SearchDestination::Section(EditableSection::Overview);
        assert_eq!(dest.screen(), Screen::Dashboard);

        let dest = SearchDestination::Device(EditableDevice::Keyboard);
        assert_eq!(dest.screen(), Screen::Input);

        let dest = SearchDestination::Gear(GearSubTab::Preferences);
        assert_eq!(dest.screen(), Screen::Gear);
        assert_eq!(dest.location_label(), "Preferences");
    }
}
