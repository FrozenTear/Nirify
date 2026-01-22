//! Main application module - Elm Architecture implementation
//!
//! This module implements the core Application logic:
//! - State management (App struct)
//! - Message handling (update function)
//! - UI construction (view function)

mod handlers;
mod helpers;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Task};
use iced::time;

use crate::config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::messages::{DialogState, Message, Page, SaveMessage};
use crate::save_manager::{SaveManager, SaveResult, ReloadResult};
use crate::views;
use crate::theme::fonts;

/// Main application state
pub struct App {
    /// Shared settings (preserved from Slint architecture)
    settings: Arc<Mutex<Settings>>,

    /// Config paths for loading/saving
    paths: Arc<ConfigPaths>,

    /// Tracks which settings categories have unsaved changes
    dirty_tracker: Arc<DirtyTracker>,

    /// Current active page
    current_page: Page,

    /// Search query (for Phase 9)
    search_query: String,

    /// Search results
    search_results: Vec<crate::search::SearchResult>,

    /// Search index
    search_index: crate::search::SearchIndex,

    /// Last search timestamp for debouncing
    last_search_time: Option<std::time::Instant>,

    /// Whether sidebar is expanded (for responsive design)
    sidebar_expanded: bool,

    /// Widget demo state for Phase 2 testing
    widget_demo_state: views::widget_demo::DemoState,

    /// Save manager for debounced auto-save
    save_manager: SaveManager,

    /// Toast notification message
    toast: Option<String>,

    /// When the toast was shown (for auto-clear)
    toast_shown_at: Option<std::time::Instant>,

    /// Active modal dialog (if any)
    dialog_state: DialogState,

    /// Current theme
    current_theme: crate::theme::AppTheme,

    /// Niri compositor connection status
    niri_status: crate::views::status_bar::NiriStatus,

    // Outputs state
    /// Selected output index for list-detail view
    selected_output_index: Option<usize>,
    /// Expanded sections in outputs view (section_name -> is_expanded)
    output_sections_expanded: std::collections::HashMap<String, bool>,
    /// Cached outputs data for view borrowing (avoids mutex lock lifetime issues)
    outputs_cache: crate::config::models::OutputSettings,

    // Layer Rules state
    /// Selected layer rule ID for list-detail view
    selected_layer_rule_id: Option<u32>,
    /// Expanded sections in layer rules view ((rule_id, section_name) -> is_expanded)
    layer_rule_sections_expanded: std::collections::HashMap<(u32, String), bool>,
    /// Regex validation errors ((rule_id, field_name) -> error_message)
    layer_rule_regex_errors: std::collections::HashMap<(u32, String), String>,

    // Window Rules state
    /// Selected window rule ID for list-detail view
    selected_window_rule_id: Option<u32>,
    /// Expanded sections in window rules view ((rule_id, section_name) -> is_expanded)
    window_rule_sections_expanded: std::collections::HashMap<(u32, String), bool>,
    /// Regex validation errors ((rule_id, field_name) -> error_message)
    window_rule_regex_errors: std::collections::HashMap<(u32, String), String>,
    /// Cached window rules data for view borrowing
    window_rules_cache: crate::config::models::WindowRulesSettings,

    // Cursor state
    /// Cached cursor data for view borrowing (avoids mutex lock lifetime issues)
    cursor_cache: crate::config::models::CursorSettings,

    // Keybindings state
    /// Selected keybinding index for list-detail view
    selected_keybinding_index: Option<usize>,
    /// Expanded sections in keybindings view
    keybinding_sections_expanded: std::collections::HashMap<String, bool>,
    /// Which keybinding is currently capturing key input (if any)
    key_capture_active: Option<usize>,
    /// Cached keybindings data for view borrowing
    keybindings_cache: crate::config::models::KeybindingsSettings,

    // Calibration matrix state
    /// Cached formatted values for tablet calibration matrix (avoids memory leak in view)
    tablet_calibration_cache: [String; 6],
    /// Cached formatted values for touch calibration matrix (avoids memory leak in view)
    touch_calibration_cache: [String; 6],

    // Tools page state
    /// State for the Tools page (IPC data and loading states)
    tools_state: views::tools::ToolsState,
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

        let settings = Arc::new(Mutex::new(settings));

        // Create dirty tracker
        let dirty_tracker = Arc::new(DirtyTracker::new());

        // Create save manager
        let save_manager = SaveManager::new(
            settings.clone(),
            paths.clone(),
            dirty_tracker.clone(),
        );

        // Clone data for view caches (avoids mutex lock lifetime issues)
        let outputs_cache = settings.lock().expect("settings mutex poisoned").outputs.clone();
        let keybindings_cache = settings.lock().expect("settings mutex poisoned").keybindings.clone();
        let window_rules_cache = settings.lock().expect("settings mutex poisoned").window_rules.clone();
        let cursor_cache = settings.lock().expect("settings mutex poisoned").cursor.clone();

        // Initialize calibration matrix caches
        let tablet_calibration_cache = crate::views::widgets::format_matrix_values(
            settings.lock().expect("settings mutex poisoned").tablet.calibration_matrix
        );
        let touch_calibration_cache = crate::views::widgets::format_matrix_values(
            settings.lock().expect("settings mutex poisoned").touch.calibration_matrix
        );

        // Check initial niri connection status
        let niri_status = if crate::ipc::is_niri_running() {
            crate::views::status_bar::NiriStatus::Connected
        } else {
            crate::views::status_bar::NiriStatus::Disconnected
        };

        let app = Self {
            settings,
            paths,
            dirty_tracker,
            current_page: Page::Overview,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: crate::search::SearchIndex::new(),
            last_search_time: None,
            sidebar_expanded: true,
            widget_demo_state: views::widget_demo::DemoState::default(),
            save_manager,
            toast: None,
            toast_shown_at: None,
            dialog_state: DialogState::default(),
            current_theme,
            niri_status,
            selected_output_index: None,
            output_sections_expanded: std::collections::HashMap::new(),
            outputs_cache,
            selected_layer_rule_id: None,
            layer_rule_sections_expanded: std::collections::HashMap::new(),
            layer_rule_regex_errors: std::collections::HashMap::new(),
            selected_window_rule_id: None,
            window_rule_sections_expanded: std::collections::HashMap::new(),
            window_rule_regex_errors: std::collections::HashMap::new(),
            window_rules_cache,
            selected_keybinding_index: None,
            keybinding_sections_expanded: std::collections::HashMap::new(),
            key_capture_active: None,
            keybindings_cache,
            cursor_cache,
            tablet_calibration_cache,
            touch_calibration_cache,
            tools_state: views::tools::ToolsState::default(),
        };

        (app, Task::none())
    }

    /// Updates application state based on messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateToPage(page) => {
                self.current_page = page;
                // Auto-refresh IPC outputs when navigating to Outputs page
                if page == Page::Outputs {
                    let is_connected = matches!(
                        self.niri_status,
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
                self.sidebar_expanded = !self.sidebar_expanded;
                Task::none()
            }

            // Search (Phase 9)
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                self.last_search_time = Some(std::time::Instant::now());

                // Perform search immediately
                self.search_results = self.search_index.search(&self.search_query);

                Task::none()
            }

            Message::SearchResultSelected(index) => {
                // Navigate to the selected page
                if let Some(result) = self.search_results.get(index) {
                    self.current_page = result.page;
                    // Clear search after navigation
                    self.search_query.clear();
                    self.search_results.clear();
                }
                Task::none()
            }

            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.last_search_time = None;
                Task::none()
            }

            // Theme
            Message::ChangeTheme(theme) => {
                self.current_theme = theme;

                // Save theme to settings
                let mut settings = self.settings.lock().expect("settings mutex poisoned");
                settings.preferences.theme = theme.to_str().to_string();
                drop(settings);

                // Mark preferences as dirty for auto-save
                self.dirty_tracker.mark(SettingsCategory::Preferences);
                self.save_manager.mark_changed();

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
                if self.save_manager.should_save() {
                    // Trigger async save
                    self.save_manager.save_task().map(Message::SaveCompleted)
                } else {
                    Task::none()
                }
            }

            Message::SaveCompleted(result) => {
                match result {
                    SaveResult::Success { files_written, .. } => {
                        self.toast = Some(format!("Saved {} file(s)", files_written));
                        self.toast_shown_at = Some(std::time::Instant::now());
                        // Trigger niri config reload
                        SaveManager::reload_niri_config_task().map(Message::ReloadCompleted)
                    }
                    SaveResult::Error { message } => {
                        self.toast = Some(format!("Save failed: {}", message));
                        self.toast_shown_at = Some(std::time::Instant::now());
                        Task::none()
                    }
                    SaveResult::NothingToSave => Task::none(),
                }
            }

            Message::ClearToast => {
                // Only clear if toast has been shown for at least 3 seconds
                if let Some(shown_at) = self.toast_shown_at {
                    if shown_at.elapsed() >= std::time::Duration::from_secs(3) {
                        self.toast = None;
                        self.toast_shown_at = None;
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
                self.dialog_state = dialog_state;
                Task::none()
            }

            Message::CloseDialog => {
                self.dialog_state = DialogState::None;
                Task::none()
            }

            Message::DialogConfirm => {
                // Handle confirmation based on current dialog
                // TODO: Implement confirmation actions
                self.dialog_state = DialogState::None;
                Task::none()
            }

            Message::WizardNext => {
                // Progress wizard to next step
                if let DialogState::FirstRunWizard { step } = &self.dialog_state {
                    use crate::messages::WizardStep;
                    let next_step = match step {
                        WizardStep::Welcome => WizardStep::ConfigSetup,
                        WizardStep::ConfigSetup => WizardStep::ImportResults,
                        WizardStep::ImportResults => WizardStep::Complete,
                        WizardStep::Complete => {
                            self.dialog_state = DialogState::None;
                            return Task::none();
                        }
                    };
                    self.dialog_state = DialogState::FirstRunWizard { step: next_step };
                }
                Task::none()
            }

            Message::WizardBack => {
                // Go back to previous wizard step
                if let DialogState::FirstRunWizard { step } = &self.dialog_state {
                    use crate::messages::WizardStep;
                    let prev_step = match step {
                        WizardStep::Welcome => {
                            self.dialog_state = DialogState::None;
                            return Task::none();
                        }
                        WizardStep::ConfigSetup => WizardStep::Welcome,
                        WizardStep::ImportResults => WizardStep::ConfigSetup,
                        WizardStep::Complete => WizardStep::ImportResults,
                    };
                    self.dialog_state = DialogState::FirstRunWizard { step: prev_step };
                }
                Task::none()
            }

            Message::WizardSetupConfig => {
                // TODO: Implement config setup
                // For now, just progress to next step
                log::info!("Wizard: Setting up config (not implemented yet)");
                if let DialogState::FirstRunWizard { .. } = &self.dialog_state {
                    self.dialog_state = DialogState::FirstRunWizard {
                        step: crate::messages::WizardStep::ImportResults,
                    };
                }
                Task::none()
            }

            Message::ConsolidationApply => {
                // TODO: Apply selected consolidation suggestions
                log::info!("Applying consolidation suggestions (not implemented yet)");
                self.dialog_state = DialogState::None;
                Task::none()
            }

            // System
            Message::WindowCloseRequested => {
                // Perform final save before exiting (blocking to prevent data loss)
                if self.dirty_tracker.is_dirty() {
                    log::info!("Window closing with unsaved changes, performing blocking save...");

                    // Clone settings and take dirty categories for blocking save
                    let settings = self.settings.lock().expect("settings mutex poisoned").clone();
                    let dirty = self.dirty_tracker.take();

                    // Perform blocking save (acceptable since typically <100ms)
                    match crate::config::save_dirty(&self.paths, &settings, &dirty) {
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
                self.niri_status = if crate::ipc::is_niri_running() {
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
        let toast_check = if self.toast.is_some() {
            Some(time::every(Duration::from_millis(500))
                .map(|_| Message::ClearToast))
        } else {
            None
        };

        // Keyboard capture subscription (only active when capturing)
        if self.key_capture_active.is_some() {
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
        let primary_nav = views::navigation::primary_nav(self.current_page, &self.search_query);

        // Secondary navigation bar (sub-tabs)
        let secondary_nav = views::navigation::secondary_nav(self.current_page);

        // Main content area (or search results if searching)
        let content_area = if !self.search_query.is_empty() && !self.search_results.is_empty() {
            // Show search results in the content area
            views::search_results::view(&self.search_results, &self.search_query)
        } else {
            self.page_content()
        };

        // Status bar (bottom)
        let is_dirty = self.dirty_tracker.is_dirty();
        let save_status = self.toast.clone();
        let status_bar = views::status_bar::view(is_dirty, save_status, self.current_theme, self.niri_status);

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
        if let Some(dialog) = views::dialogs::view(&self.dialog_state) {
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
        let page_view = match self.current_page {
            Page::Overview => return self.overview_page(),
            Page::Appearance => {
                let appearance = self.settings.lock().expect("settings mutex poisoned").appearance.clone();
                views::appearance::view(appearance)
            }
            Page::Behavior => {
                let behavior = self.settings.lock().expect("settings mutex poisoned").behavior.clone();
                views::behavior::view(behavior)
            }
            Page::Keyboard => {
                let keyboard = self.settings.lock().expect("settings mutex poisoned").keyboard.clone();
                views::keyboard::view(keyboard)
            }
            Page::Mouse => {
                let mouse = self.settings.lock().expect("settings mutex poisoned").mouse.clone();
                views::mouse::view(mouse)
            }
            Page::Touchpad => {
                let touchpad = self.settings.lock().expect("settings mutex poisoned").touchpad.clone();
                views::touchpad::view(touchpad)
            }
            Page::Trackpoint => {
                let trackpoint = self.settings.lock().expect("settings mutex poisoned").trackpoint.clone();
                views::trackpoint::view(trackpoint)
            }
            Page::Trackball => {
                let trackball = self.settings.lock().expect("settings mutex poisoned").trackball.clone();
                views::trackball::view(trackball)
            }
            Page::Tablet => {
                let tablet = self.settings.lock().expect("settings mutex poisoned").tablet.clone();
                return views::tablet::view(tablet, &self.tablet_calibration_cache);
            }
            Page::Touch => {
                let touch = self.settings.lock().expect("settings mutex poisoned").touch.clone();
                return views::touch::view(touch, &self.touch_calibration_cache);
            }
            Page::Animations => return views::animations::view(),
            Page::Cursor => {
                return views::cursor::view(&self.cursor_cache);
            }
            Page::LayoutExtras => {
                let layout = self.settings.lock().expect("settings mutex poisoned").layout_extras.clone();
                return views::layout_extras::view(&layout);
            }
            Page::Gestures => {
                let gestures = self.settings.lock().expect("settings mutex poisoned").gestures.clone();
                return views::gestures::view(&gestures);
            }
            Page::Workspaces => {
                let workspaces = self.settings.lock().expect("settings mutex poisoned").workspaces.clone();
                return views::workspaces::view(&workspaces);
            }
            Page::WindowRules => {
                return views::window_rules::view(
                    &self.window_rules_cache,
                    self.selected_window_rule_id,
                    &self.window_rule_sections_expanded,
                    &self.window_rule_regex_errors,
                );
            }
            Page::LayerRules => return views::layer_rules::view(),
            Page::Keybindings => {
                return views::keybindings::view(
                    &self.keybindings_cache,
                    self.selected_keybinding_index,
                    &self.keybinding_sections_expanded,
                    self.key_capture_active,
                );
            }
            Page::Outputs => {
                return views::outputs::view(
                    &self.outputs_cache,
                    self.selected_output_index,
                    &self.output_sections_expanded,
                    &self.tools_state.outputs,  // IPC data for available modes
                );
            }
            Page::Miscellaneous => {
                let misc = self.settings.lock().expect("settings mutex poisoned").miscellaneous.clone();
                return views::miscellaneous::view(&misc);
            }
            Page::Startup => {
                let startup = self.settings.lock().expect("settings mutex poisoned").startup.clone();
                return views::startup::view(&startup);
            }
            Page::Environment => {
                let env = self.settings.lock().expect("settings mutex poisoned").environment.clone();
                return views::environment::view(&env);
            }
            Page::Debug => {
                let debug = self.settings.lock().expect("settings mutex poisoned").debug.clone();
                return views::debug::view(&debug);
            }
            Page::SwitchEvents => {
                let switch = self.settings.lock().expect("settings mutex poisoned").switch_events.clone();
                return views::switch_events::view(&switch);
            }
            Page::RecentWindows => {
                let recent = self.settings.lock().expect("settings mutex poisoned").recent_windows.clone();
                return views::recent_windows::view(&recent);
            }
            Page::Tools => {
                let niri_connected = matches!(
                    self.niri_status,
                    crate::views::status_bar::NiriStatus::Connected
                );
                return views::tools::view(&self.tools_state, niri_connected);
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

        let settings = self.settings.lock().expect("settings mutex poisoned");

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
                    Some(self.current_theme),
                    Message::ChangeTheme,
                )
                .placeholder("Select theme...")
                .width(Length::Fixed(200.0)),
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
        let page_view = column![summary, views::widget_demo::view(&self.widget_demo_state),]
            .spacing(20);

        container(page_view)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

}

// Note: Default is not needed with iced::application() - it uses App::new() directly

/// Runs the application
pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(|app: &App| app.current_theme.to_iced_theme())
        .run()
}
