//! Main application module - Elm Architecture implementation
//!
//! This module implements the core Application logic:
//! - State management (App struct)
//! - Message handling (update function)
//! - UI construction (view function)

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
}

/// Formats a key press event into a niri-compatible key combo string
/// e.g., "Mod+Shift+Return" or "Ctrl+Alt+Delete"
fn format_key_combo(key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> String {
    use iced::keyboard::{Key, key::Named};

    // Skip if this is just a modifier key by itself
    let is_modifier_key = matches!(
        key,
        Key::Named(Named::Shift | Named::Control | Named::Alt | Named::Super)
    );
    if is_modifier_key {
        return String::new();
    }

    let mut parts = Vec::new();

    // Add modifiers in niri's expected order
    // Note: niri uses "Mod" for Super/Logo key
    if modifiers.logo() {
        parts.push("Mod");
    }
    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        parts.push("Alt");
    }
    if modifiers.shift() {
        parts.push("Shift");
    }

    // Add the key name
    let key_name = match key {
        Key::Named(named) => match named {
            Named::Enter => "Return",
            Named::Tab => "Tab",
            Named::Space => "space",
            Named::Backspace => "BackSpace",
            Named::Delete => "Delete",
            Named::Escape => "Escape",
            Named::Home => "Home",
            Named::End => "End",
            Named::PageUp => "Page_Up",
            Named::PageDown => "Page_Down",
            Named::ArrowUp => "Up",
            Named::ArrowDown => "Down",
            Named::ArrowLeft => "Left",
            Named::ArrowRight => "Right",
            Named::F1 => "F1",
            Named::F2 => "F2",
            Named::F3 => "F3",
            Named::F4 => "F4",
            Named::F5 => "F5",
            Named::F6 => "F6",
            Named::F7 => "F7",
            Named::F8 => "F8",
            Named::F9 => "F9",
            Named::F10 => "F10",
            Named::F11 => "F11",
            Named::F12 => "F12",
            Named::Insert => "Insert",
            Named::PrintScreen => "Print",
            Named::ScrollLock => "Scroll_Lock",
            Named::Pause => "Pause",
            Named::AudioVolumeUp => "XF86AudioRaiseVolume",
            Named::AudioVolumeDown => "XF86AudioLowerVolume",
            Named::AudioVolumeMute => "XF86AudioMute",
            Named::MediaPlayPause => "XF86AudioPlay",
            Named::MediaStop => "XF86AudioStop",
            Named::MediaTrackNext => "XF86AudioNext",
            Named::MediaTrackPrevious => "XF86AudioPrev",
            Named::BrightnessUp => "XF86MonBrightnessUp",
            Named::BrightnessDown => "XF86MonBrightnessDown",
            _ => return String::new(), // Unknown named key
        },
        Key::Character(c) => {
            // For character keys, uppercase for consistent display
            let s = c.as_str();
            if s.len() == 1 {
                // Single character - uppercase it for display
                let upper = s.to_uppercase();
                if parts.is_empty() {
                    return upper;
                } else {
                    return format!("{}+{}", parts.join("+"), upper);
                }
            } else {
                return String::new();
            }
        }
        Key::Unidentified => return String::new(),
    };

    parts.push(key_name);
    parts.join("+")
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
        let current_theme = crate::theme::AppTheme::from_str(&settings.preferences.theme);

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
        let outputs_cache = settings.lock().unwrap().outputs.clone();
        let keybindings_cache = settings.lock().unwrap().keybindings.clone();
        let window_rules_cache = settings.lock().unwrap().window_rules.clone();
        let cursor_cache = settings.lock().unwrap().cursor.clone();

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
        };

        (app, Task::none())
    }

    /// Updates application state based on messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateToPage(page) => {
                self.current_page = page;
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
                let mut settings = self.settings.lock().unwrap();
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
                    let settings = self.settings.lock().unwrap().clone();
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

            Message::None => Task::none(),
        }
    }

    /// Handle debug settings messages
    fn update_debug(&mut self, msg: crate::messages::DebugMessage) -> Task<Message> {
        use crate::messages::DebugMessage;

        let mut settings = self.settings.lock().unwrap();
        let debug = &mut settings.debug;

        match msg {
            DebugMessage::SetExpertMode(v) => debug.expert_mode = v,
            DebugMessage::SetPreviewRender(v) => debug.preview_render = v,
            DebugMessage::SetEnableOverlayPlanes(v) => debug.enable_overlay_planes = v,
            DebugMessage::SetDisableCursorPlane(v) => debug.disable_cursor_plane = v,
            DebugMessage::SetDisableDirectScanout(v) => debug.disable_direct_scanout = v,
            DebugMessage::SetRestrictPrimaryScanoutToMatchingFormat(v) => debug.restrict_primary_scanout_to_matching_format = v,
            DebugMessage::SetRenderDrmDevice(v) => debug.render_drm_device = v,
            DebugMessage::AddIgnoreDrmDevice(device) => {
                if !device.is_empty() {
                    debug.ignore_drm_devices.push(device);
                }
            }
            DebugMessage::RemoveIgnoreDrmDevice(idx) => {
                if idx < debug.ignore_drm_devices.len() {
                    debug.ignore_drm_devices.remove(idx);
                }
            }
            DebugMessage::SetWaitForFrameCompletionBeforeQueueing(v) => debug.wait_for_frame_completion_before_queueing = v,
            DebugMessage::SetDisableResizeThrottling(v) => debug.disable_resize_throttling = v,
            DebugMessage::SetDisableTransactions(v) => debug.disable_transactions = v,
            DebugMessage::SetEmulateZeroPresentationTime(v) => debug.emulate_zero_presentation_time = v,
            DebugMessage::SetSkipCursorOnlyUpdatesDuringVrr(v) => debug.skip_cursor_only_updates_during_vrr = v,
            DebugMessage::SetDbusInterfacesInNonSessionInstances(v) => debug.dbus_interfaces_in_non_session_instances = v,
            DebugMessage::SetKeepLaptopPanelOnWhenLidIsClosed(v) => debug.keep_laptop_panel_on_when_lid_is_closed = v,
            DebugMessage::SetDisableMonitorNames(v) => debug.disable_monitor_names = v,
            DebugMessage::SetForceDisableConnectorsOnResume(v) => debug.force_disable_connectors_on_resume = v,
            DebugMessage::SetStrictNewWindowFocusPolicy(v) => debug.strict_new_window_focus_policy = v,
            DebugMessage::SetHonorXdgActivationWithInvalidSerial(v) => debug.honor_xdg_activation_with_invalid_serial = v,
            DebugMessage::SetDeactivateUnfocusedWindows(v) => debug.deactivate_unfocused_windows = v,
            DebugMessage::SetForcePipewireInvalidModifier(v) => debug.force_pipewire_invalid_modifier = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Debug);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle miscellaneous settings messages
    fn update_miscellaneous(&mut self, msg: crate::messages::MiscellaneousMessage) -> Task<Message> {
        use crate::messages::MiscellaneousMessage;

        let mut settings = self.settings.lock().unwrap();
        let misc = &mut settings.miscellaneous;

        match msg {
            MiscellaneousMessage::SetPreferNoCsd(v) => misc.prefer_no_csd = v,
            MiscellaneousMessage::SetScreenshotPath(v) => misc.screenshot_path = v,
            MiscellaneousMessage::SetDisablePrimaryClipboard(v) => misc.disable_primary_clipboard = v,
            MiscellaneousMessage::SetHotkeyOverlaySkipAtStartup(v) => misc.hotkey_overlay_skip_at_startup = v,
            MiscellaneousMessage::SetHotkeyOverlayHideNotBound(v) => misc.hotkey_overlay_hide_not_bound = v,
            MiscellaneousMessage::SetConfigNotificationDisableFailed(v) => misc.config_notification_disable_failed = v,
            MiscellaneousMessage::SetSpawnShAtStartup(v) => misc.spawn_sh_at_startup = v,
            MiscellaneousMessage::SetXWaylandSatellite(v) => misc.xwayland_satellite = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Miscellaneous);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle environment settings messages
    fn update_environment(&mut self, msg: crate::messages::EnvironmentMessage) -> Task<Message> {
        use crate::messages::EnvironmentMessage;

        let mut settings = self.settings.lock().unwrap();
        let env = &mut settings.environment;

        match msg {
            EnvironmentMessage::AddVariable => {
                let id = env.next_id;
                env.next_id += 1;
                env.variables.push(crate::config::models::EnvironmentVariable {
                    id,
                    name: String::new(),
                    value: String::new(),
                });
            }
            EnvironmentMessage::RemoveVariable(id) => {
                env.variables.retain(|v| v.id != id);
            }
            EnvironmentMessage::SetVariableName(id, name) => {
                if let Some(var) = env.variables.iter_mut().find(|v| v.id == id) {
                    var.name = name;
                }
            }
            EnvironmentMessage::SetVariableValue(id, value) => {
                if let Some(var) = env.variables.iter_mut().find(|v| v.id == id) {
                    var.value = value;
                }
            }
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Environment);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle switch events settings messages
    fn update_switch_events(&mut self, msg: crate::messages::SwitchEventsMessage) -> Task<Message> {
        use crate::messages::SwitchEventsMessage;

        let mut settings = self.settings.lock().unwrap();
        let switch = &mut settings.switch_events;

        // Helper to parse command string into Vec<String>
        fn parse_command(cmd: &str) -> Vec<String> {
            if cmd.trim().is_empty() {
                Vec::new()
            } else {
                // Simple split by whitespace - could be enhanced with proper shell parsing
                cmd.split_whitespace().map(String::from).collect()
            }
        }

        match msg {
            SwitchEventsMessage::SetLidCloseCommand(cmd) => {
                switch.lid_close.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetLidOpenCommand(cmd) => {
                switch.lid_open.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetTabletModeOnCommand(cmd) => {
                switch.tablet_mode_on.spawn = parse_command(&cmd);
            }
            SwitchEventsMessage::SetTabletModeOffCommand(cmd) => {
                switch.tablet_mode_off.spawn = parse_command(&cmd);
            }
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::SwitchEvents);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle recent windows settings messages
    fn update_recent_windows(&mut self, msg: crate::messages::RecentWindowsMessage) -> Task<Message> {
        use crate::messages::RecentWindowsMessage;

        let mut settings = self.settings.lock().unwrap();
        let recent = &mut settings.recent_windows;

        match msg {
            // Top-level settings
            RecentWindowsMessage::SetOff(v) => recent.off = v,
            RecentWindowsMessage::SetDebounceMs(v) => recent.debounce_ms = v.clamp(0, 5000),
            RecentWindowsMessage::SetOpenDelayMs(v) => recent.open_delay_ms = v.clamp(0, 5000),

            // Highlight settings
            RecentWindowsMessage::SetActiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    recent.highlight.active_color = color;
                }
            }
            RecentWindowsMessage::SetUrgentColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    recent.highlight.urgent_color = color;
                }
            }
            RecentWindowsMessage::SetHighlightPadding(v) => recent.highlight.padding = v.clamp(0, 100),
            RecentWindowsMessage::SetHighlightCornerRadius(v) => recent.highlight.corner_radius = v.clamp(0, 100),

            // Preview settings
            RecentWindowsMessage::SetPreviewMaxHeight(v) => recent.previews.max_height = v.clamp(50, 1000),
            RecentWindowsMessage::SetPreviewMaxScale(v) => recent.previews.max_scale = v.clamp(0.1, 1.0),

            // Keybind management
            RecentWindowsMessage::AddBind => {
                recent.binds.push(crate::config::models::RecentWindowsBind::default());
            }
            RecentWindowsMessage::RemoveBind(idx) => {
                if idx < recent.binds.len() {
                    recent.binds.remove(idx);
                }
            }
            RecentWindowsMessage::SetBindKeyCombo(idx, combo) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.key_combo = combo;
                }
            }
            RecentWindowsMessage::SetBindIsNext(idx, is_next) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.is_next = is_next;
                }
            }
            RecentWindowsMessage::SetBindFilterAppId(idx, filter) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.filter_app_id = filter;
                }
            }
            RecentWindowsMessage::SetBindScope(idx, scope) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.scope = scope;
                }
            }
            RecentWindowsMessage::SetBindCooldown(idx, cooldown) => {
                if let Some(bind) = recent.binds.get_mut(idx) {
                    bind.cooldown_ms = cooldown;
                }
            }
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::RecentWindows);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle trackpoint settings messages
    fn update_trackpoint(&mut self, msg: crate::messages::TrackpointMessage) -> Task<Message> {
        use crate::messages::TrackpointMessage;

        let mut settings = self.settings.lock().unwrap();
        let trackpoint = &mut settings.trackpoint;

        match msg {
            TrackpointMessage::SetOff(v) => trackpoint.off = v,
            TrackpointMessage::SetNaturalScroll(v) => trackpoint.natural_scroll = v,
            TrackpointMessage::SetAccelSpeed(v) => trackpoint.accel_speed = v.clamp(-1.0, 1.0) as f64,
            TrackpointMessage::SetAccelProfile(v) => trackpoint.accel_profile = v,
            TrackpointMessage::SetScrollMethod(v) => trackpoint.scroll_method = v,
            TrackpointMessage::SetLeftHanded(v) => trackpoint.left_handed = v,
            TrackpointMessage::SetMiddleEmulation(v) => trackpoint.middle_emulation = v,
            TrackpointMessage::SetScrollButtonLock(v) => trackpoint.scroll_button_lock = v,
            TrackpointMessage::SetScrollButton(v) => trackpoint.scroll_button = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Trackpoint);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle trackball settings messages
    fn update_trackball(&mut self, msg: crate::messages::TrackballMessage) -> Task<Message> {
        use crate::messages::TrackballMessage;

        let mut settings = self.settings.lock().unwrap();
        let trackball = &mut settings.trackball;

        match msg {
            TrackballMessage::SetOff(v) => trackball.off = v,
            TrackballMessage::SetNaturalScroll(v) => trackball.natural_scroll = v,
            TrackballMessage::SetAccelSpeed(v) => trackball.accel_speed = v.clamp(-1.0, 1.0) as f64,
            TrackballMessage::SetAccelProfile(v) => trackball.accel_profile = v,
            TrackballMessage::SetScrollMethod(v) => trackball.scroll_method = v,
            TrackballMessage::SetLeftHanded(v) => trackball.left_handed = v,
            TrackballMessage::SetMiddleEmulation(v) => trackball.middle_emulation = v,
            TrackballMessage::SetScrollButtonLock(v) => trackball.scroll_button_lock = v,
            TrackballMessage::SetScrollButton(v) => trackball.scroll_button = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Trackball);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle tablet settings messages
    fn update_tablet(&mut self, msg: crate::messages::TabletMessage) -> Task<Message> {
        use crate::messages::TabletMessage;

        let mut settings = self.settings.lock().unwrap();
        let tablet = &mut settings.tablet;

        match msg {
            TabletMessage::SetOff(v) => tablet.off = v,
            TabletMessage::SetLeftHanded(v) => tablet.left_handed = v,
            TabletMessage::SetMapToOutput(v) => tablet.map_to_output = v,
            TabletMessage::SetCalibrationMatrix(v) => tablet.calibration_matrix = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Tablet);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle touch settings messages
    fn update_touch(&mut self, msg: crate::messages::TouchMessage) -> Task<Message> {
        use crate::messages::TouchMessage;

        let mut settings = self.settings.lock().unwrap();
        let touch = &mut settings.touch;

        match msg {
            TouchMessage::SetOff(v) => touch.off = v,
            TouchMessage::SetMapToOutput(v) => touch.map_to_output = v,
            TouchMessage::SetCalibrationMatrix(v) => touch.calibration_matrix = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Touch);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle gestures settings messages
    fn update_gestures(&mut self, msg: crate::messages::GesturesMessage) -> Task<Message> {
        use crate::messages::GesturesMessage;

        let mut settings = self.settings.lock().unwrap();
        let gestures = &mut settings.gestures;

        match msg {
            // Hot corners
            GesturesMessage::SetHotCornersEnabled(v) => gestures.hot_corners.enabled = v,
            GesturesMessage::SetHotCornerTopLeft(v) => gestures.hot_corners.top_left = v,
            GesturesMessage::SetHotCornerTopRight(v) => gestures.hot_corners.top_right = v,
            GesturesMessage::SetHotCornerBottomLeft(v) => gestures.hot_corners.bottom_left = v,
            GesturesMessage::SetHotCornerBottomRight(v) => gestures.hot_corners.bottom_right = v,

            // DnD edge view scroll
            GesturesMessage::SetDndScrollEnabled(v) => gestures.dnd_edge_view_scroll.enabled = v,
            GesturesMessage::SetDndScrollTriggerWidth(v) => gestures.dnd_edge_view_scroll.trigger_size = v.clamp(10, 200),
            GesturesMessage::SetDndScrollDelayMs(v) => gestures.dnd_edge_view_scroll.delay_ms = v.clamp(0, 2000),
            GesturesMessage::SetDndScrollMaxSpeed(v) => gestures.dnd_edge_view_scroll.max_speed = v.clamp(100, 5000),

            // DnD edge workspace switch
            GesturesMessage::SetDndWorkspaceEnabled(v) => gestures.dnd_edge_workspace_switch.enabled = v,
            GesturesMessage::SetDndWorkspaceTriggerHeight(v) => gestures.dnd_edge_workspace_switch.trigger_size = v.clamp(10, 200),
            GesturesMessage::SetDndWorkspaceDelayMs(v) => gestures.dnd_edge_workspace_switch.delay_ms = v.clamp(0, 2000),
            GesturesMessage::SetDndWorkspaceMaxSpeed(v) => gestures.dnd_edge_workspace_switch.max_speed = v.clamp(100, 5000),
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Gestures);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle layout extras settings messages
    fn update_layout_extras(&mut self, msg: crate::messages::LayoutExtrasMessage) -> Task<Message> {
        use crate::messages::LayoutExtrasMessage;
        use crate::types::ColorOrGradient;

        let mut settings = self.settings.lock().unwrap();
        let layout = &mut settings.layout_extras;

        match msg {
            // Shadow settings
            LayoutExtrasMessage::SetShadowEnabled(v) => layout.shadow.enabled = v,
            LayoutExtrasMessage::SetShadowSoftness(v) => layout.shadow.softness = v.clamp(0, 100),
            LayoutExtrasMessage::SetShadowSpread(v) => layout.shadow.spread = v.clamp(0, 100),
            LayoutExtrasMessage::SetShadowOffsetX(v) => layout.shadow.offset_x = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowOffsetY(v) => layout.shadow.offset_y = v.clamp(-100, 100),
            LayoutExtrasMessage::SetShadowDrawBehindWindow(v) => layout.shadow.draw_behind_window = v,
            LayoutExtrasMessage::SetShadowColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.color = color;
                }
            }
            LayoutExtrasMessage::SetShadowInactiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.shadow.inactive_color = color;
                }
            }

            // Tab indicator
            LayoutExtrasMessage::SetTabIndicatorEnabled(v) => layout.tab_indicator.enabled = v,
            LayoutExtrasMessage::SetTabIndicatorHideWhenSingleTab(v) => layout.tab_indicator.hide_when_single_tab = v,
            LayoutExtrasMessage::SetTabIndicatorPlaceWithinColumn(v) => layout.tab_indicator.place_within_column = v,
            LayoutExtrasMessage::SetTabIndicatorGap(v) => layout.tab_indicator.gap = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorWidth(v) => layout.tab_indicator.width = v.clamp(1, 50),
            LayoutExtrasMessage::SetTabIndicatorLengthProportion(v) => layout.tab_indicator.length_proportion = v.clamp(0.1, 1.0),
            LayoutExtrasMessage::SetTabIndicatorCornerRadius(v) => layout.tab_indicator.corner_radius = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorGapsBetweenTabs(v) => layout.tab_indicator.gaps_between_tabs = v.clamp(0, 50),
            LayoutExtrasMessage::SetTabIndicatorPosition(v) => layout.tab_indicator.position = v,
            LayoutExtrasMessage::SetTabIndicatorActiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.active = ColorOrGradient::Color(color);
                }
            }
            LayoutExtrasMessage::SetTabIndicatorInactiveColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.inactive = ColorOrGradient::Color(color);
                }
            }
            LayoutExtrasMessage::SetTabIndicatorUrgentColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.tab_indicator.urgent = ColorOrGradient::Color(color);
                }
            }

            // Insert hint
            LayoutExtrasMessage::SetInsertHintEnabled(v) => layout.insert_hint.enabled = v,
            LayoutExtrasMessage::SetInsertHintColor(hex) => {
                if let Some(color) = crate::types::Color::from_hex(&hex) {
                    layout.insert_hint.color = ColorOrGradient::Color(color);
                }
            }

            // Preset widths/heights
            LayoutExtrasMessage::AddPresetWidth => {
                layout.preset_column_widths.push(crate::config::models::PresetWidth::Proportion(0.5));
            }
            LayoutExtrasMessage::RemovePresetWidth(idx) => {
                if idx < layout.preset_column_widths.len() {
                    layout.preset_column_widths.remove(idx);
                }
            }
            LayoutExtrasMessage::SetPresetWidth(idx, width) => {
                if let Some(w) = layout.preset_column_widths.get_mut(idx) {
                    *w = width;
                }
            }
            LayoutExtrasMessage::AddPresetHeight => {
                layout.preset_window_heights.push(crate::config::models::PresetHeight::Proportion(0.5));
            }
            LayoutExtrasMessage::RemovePresetHeight(idx) => {
                if idx < layout.preset_window_heights.len() {
                    layout.preset_window_heights.remove(idx);
                }
            }
            LayoutExtrasMessage::SetPresetHeight(idx, height) => {
                if let Some(h) = layout.preset_window_heights.get_mut(idx) {
                    *h = height;
                }
            }

            // Default column display
            LayoutExtrasMessage::SetDefaultColumnDisplay(v) => layout.default_column_display = v,
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::LayoutExtras);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Handle startup commands messages
    fn update_startup(&mut self, msg: crate::messages::StartupMessage) -> Task<Message> {
        use crate::messages::StartupMessage;

        let mut settings = self.settings.lock().unwrap();
        let startup = &mut settings.startup;

        match msg {
            StartupMessage::AddCommand => {
                let id = startup.next_id;
                startup.next_id += 1;
                startup.commands.push(crate::config::models::StartupCommand {
                    id,
                    command: vec![String::new()],
                });
            }
            StartupMessage::RemoveCommand(id) => {
                startup.commands.retain(|c| c.id != id);
            }
            StartupMessage::SetCommand(id, cmd) => {
                if let Some(command) = startup.commands.iter_mut().find(|c| c.id == id) {
                    // Split by whitespace for the command vector
                    command.command = cmd.split_whitespace().map(String::from).collect();
                    if command.command.is_empty() {
                        command.command.push(String::new());
                    }
                }
            }
        }

        drop(settings);
        self.dirty_tracker.mark(crate::config::SettingsCategory::Startup);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Returns the subscription for periodic save checks and keyboard capture
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        // Base subscription: periodic save checks (every 50ms)
        let save_check = time::every(Duration::from_millis(50))
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
                        let key_combo = format_key_combo(&key, modifiers);

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
                let appearance = self.settings.lock().unwrap().appearance.clone();
                views::appearance::view(appearance)
            }
            Page::Behavior => {
                let behavior = self.settings.lock().unwrap().behavior.clone();
                views::behavior::view(behavior)
            }
            Page::Keyboard => {
                let keyboard = self.settings.lock().unwrap().keyboard.clone();
                views::keyboard::view(keyboard)
            }
            Page::Mouse => {
                let mouse = self.settings.lock().unwrap().mouse.clone();
                views::mouse::view(mouse)
            }
            Page::Touchpad => {
                let touchpad = self.settings.lock().unwrap().touchpad.clone();
                views::touchpad::view(touchpad)
            }
            Page::Trackpoint => {
                let trackpoint = self.settings.lock().unwrap().trackpoint.clone();
                views::trackpoint::view(trackpoint)
            }
            Page::Trackball => {
                let trackball = self.settings.lock().unwrap().trackball.clone();
                views::trackball::view(trackball)
            }
            Page::Tablet => {
                let tablet = self.settings.lock().unwrap().tablet.clone();
                views::tablet::view(tablet)
            }
            Page::Touch => {
                let touch = self.settings.lock().unwrap().touch.clone();
                views::touch::view(touch)
            }
            Page::Animations => return views::animations::view(),
            Page::Cursor => {
                return views::cursor::view(&self.cursor_cache);
            }
            Page::LayoutExtras => {
                let layout = self.settings.lock().unwrap().layout_extras.clone();
                return views::layout_extras::view(&layout);
            }
            Page::Gestures => {
                let gestures = self.settings.lock().unwrap().gestures.clone();
                return views::gestures::view(&gestures);
            }
            Page::Workspaces => {
                let workspaces = self.settings.lock().unwrap().workspaces.clone();
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
                );
            }
            Page::Miscellaneous => {
                let misc = self.settings.lock().unwrap().miscellaneous.clone();
                return views::miscellaneous::view(&misc);
            }
            Page::Startup => {
                let startup = self.settings.lock().unwrap().startup.clone();
                return views::startup::view(&startup);
            }
            Page::Environment => {
                let env = self.settings.lock().unwrap().environment.clone();
                return views::environment::view(&env);
            }
            Page::Debug => {
                let debug = self.settings.lock().unwrap().debug.clone();
                return views::debug::view(&debug);
            }
            Page::SwitchEvents => {
                let switch = self.settings.lock().unwrap().switch_events.clone();
                return views::switch_events::view(&switch);
            }
            Page::RecentWindows => {
                let recent = self.settings.lock().unwrap().recent_windows.clone();
                return views::recent_windows::view(&recent);
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

        let settings = self.settings.lock().unwrap();

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
            text("Choose your preferred color theme for the application").size(12).color([0.6, 0.6, 0.6]),
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

    /// Updates appearance settings
    fn update_appearance(&mut self, msg: crate::messages::AppearanceMessage) -> Task<Message> {
        use crate::messages::AppearanceMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            // Focus ring
            AppearanceMessage::ToggleFocusRing(value) => {
                settings.appearance.focus_ring_enabled = value;
            }
            AppearanceMessage::SetFocusRingWidth(value) => {
                settings.appearance.focus_ring_width = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::FocusRingActive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_active, gradient_msg);
            }
            AppearanceMessage::FocusRingInactive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_inactive, gradient_msg);
            }
            AppearanceMessage::FocusRingUrgent(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.focus_ring_urgent, gradient_msg);
            }

            // Border
            AppearanceMessage::ToggleBorder(value) => {
                settings.appearance.border_enabled = value;
            }
            AppearanceMessage::SetBorderThickness(value) => {
                settings.appearance.border_thickness = value.clamp(1.0, 20.0);
            }
            AppearanceMessage::BorderActive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_active, gradient_msg);
            }
            AppearanceMessage::BorderInactive(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_inactive, gradient_msg);
            }
            AppearanceMessage::BorderUrgent(gradient_msg) => {
                self.apply_gradient_message(&mut settings.appearance.border_urgent, gradient_msg);
            }

            // Layout
            AppearanceMessage::SetGaps(value) => {
                settings.appearance.gaps = value.clamp(0.0, 64.0);
            }
            AppearanceMessage::SetCornerRadius(value) => {
                settings.appearance.corner_radius = value.clamp(0.0, 32.0);
            }

            // Background
            AppearanceMessage::SetBackgroundColor(hex_opt) => {
                use crate::types::Color;
                settings.appearance.background_color = hex_opt.and_then(|hex| Color::from_hex(&hex));
            }
        }

        drop(settings); // Release lock

        // Mark as dirty for auto-save
        self.dirty_tracker.mark(SettingsCategory::Appearance);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Helper to apply GradientPickerMessage to a ColorOrGradient field
    fn apply_gradient_message(&self, target: &mut crate::types::ColorOrGradient, msg: crate::views::widgets::GradientPickerMessage) {
        use crate::types::{Color, ColorOrGradient, Gradient};
        use crate::views::widgets::GradientPickerMessage;

        match msg {
            GradientPickerMessage::ToggleSolidGradient(is_gradient) => {
                *target = if is_gradient {
                    // Convert to gradient
                    match target {
                        ColorOrGradient::Color(color) => {
                            ColorOrGradient::Gradient(Gradient {
                                from: color.clone(),
                                to: color.clone(),
                                angle: 0,
                                ..Default::default()
                            })
                        }
                        ColorOrGradient::Gradient(_) => target.clone(),
                    }
                } else {
                    // Convert to solid color
                    match target {
                        ColorOrGradient::Color(_) => target.clone(),
                        ColorOrGradient::Gradient(gradient) => {
                            ColorOrGradient::Color(gradient.from.clone())
                        }
                    }
                };
            }
            GradientPickerMessage::SetFromColor(hex) => {
                if let Some(color) = Color::from_hex(&hex) {
                    match target {
                        ColorOrGradient::Color(c) => *c = color,
                        ColorOrGradient::Gradient(g) => g.from = color,
                    }
                }
            }
            GradientPickerMessage::SetToColor(hex) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    if let Some(color) = Color::from_hex(&hex) {
                        gradient.to = color;
                    }
                }
            }
            GradientPickerMessage::SetAngle(angle) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.angle = angle;
                }
            }
            GradientPickerMessage::SetColorSpace(color_space) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.color_space = color_space;
                }
            }
            GradientPickerMessage::SetRelativeTo(relative_to) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.relative_to = relative_to;
                }
            }
            GradientPickerMessage::SetHueInterpolation(hue_interp) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.hue_interpolation = Some(hue_interp);
                }
            }
        }
    }

    /// Updates behavior settings
    fn update_behavior(&mut self, msg: crate::messages::BehaviorMessage) -> Task<Message> {
        use crate::messages::BehaviorMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            BehaviorMessage::ToggleFocusFollowsMouse(value) => {
                settings.behavior.focus_follows_mouse = value;
            }
            BehaviorMessage::SetWarpMouseToFocus(mode) => {
                settings.behavior.warp_mouse_to_focus = mode;
            }
            BehaviorMessage::ToggleWorkspaceAutoBackAndForth(value) => {
                settings.behavior.workspace_auto_back_and_forth = value;
            }
            BehaviorMessage::ToggleAlwaysCenterSingleColumn(value) => {
                settings.behavior.always_center_single_column = value;
            }
            BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(value) => {
                settings.behavior.empty_workspace_above_first = value;
            }
            BehaviorMessage::SetCenterFocusedColumn(mode) => {
                settings.behavior.center_focused_column = mode;
            }
            BehaviorMessage::SetDefaultColumnWidthType(width_type) => {
                settings.behavior.default_column_width_type = width_type;
            }
            BehaviorMessage::SetStrutLeft(value) => {
                settings.behavior.strut_left = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutRight(value) => {
                settings.behavior.strut_right = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutTop(value) => {
                settings.behavior.strut_top = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetStrutBottom(value) => {
                settings.behavior.strut_bottom = value.clamp(0.0, 200.0);
            }
            BehaviorMessage::SetModKey(key) => {
                settings.behavior.mod_key = key;
            }
            BehaviorMessage::SetModKeyNested(key) => {
                settings.behavior.mod_key_nested = key;
            }
            BehaviorMessage::ToggleDisablePowerKeyHandling(value) => {
                settings.behavior.disable_power_key_handling = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Behavior);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates keyboard settings
    fn update_keyboard(&mut self, msg: crate::messages::KeyboardMessage) -> Task<Message> {
        use crate::messages::KeyboardMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            KeyboardMessage::SetXkbLayout(value) => {
                settings.keyboard.xkb_layout = value;
            }
            KeyboardMessage::SetXkbVariant(value) => {
                settings.keyboard.xkb_variant = value;
            }
            KeyboardMessage::SetXkbOptions(value) => {
                settings.keyboard.xkb_options = value;
            }
            KeyboardMessage::SetXkbModel(value) => {
                settings.keyboard.xkb_model = value;
            }
            KeyboardMessage::SetRepeatDelay(value) => {
                settings.keyboard.repeat_delay = value.clamp(100, 2000);
            }
            KeyboardMessage::SetRepeatRate(value) => {
                settings.keyboard.repeat_rate = value.clamp(1, 100);
            }
            KeyboardMessage::SetTrackLayout(value) => {
                settings.keyboard.track_layout = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Keyboard);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates mouse settings
    fn update_mouse(&mut self, msg: crate::messages::MouseMessage) -> Task<Message> {
        use crate::messages::MouseMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            MouseMessage::ToggleOffOnTouchpad(value) => {
                settings.mouse.off = value;
            }
            MouseMessage::ToggleNaturalScroll(value) => {
                settings.mouse.natural_scroll = value;
            }
            MouseMessage::SetAccelSpeed(value) => {
                settings.mouse.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            MouseMessage::SetAccelProfile(profile) => {
                settings.mouse.accel_profile = profile;
            }
            MouseMessage::SetScrollFactor(value) => {
                settings.mouse.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            MouseMessage::SetScrollMethod(method) => {
                settings.mouse.scroll_method = method;
            }
            MouseMessage::ToggleLeftHanded(value) => {
                settings.mouse.left_handed = value;
            }
            MouseMessage::ToggleMiddleEmulation(value) => {
                settings.mouse.middle_emulation = value;
            }
            MouseMessage::ToggleScrollButtonLock(value) => {
                settings.mouse.scroll_button_lock = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Mouse);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates touchpad settings
    fn update_touchpad(&mut self, msg: crate::messages::TouchpadMessage) -> Task<Message> {
        use crate::messages::TouchpadMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            TouchpadMessage::ToggleTapToClick(value) => {
                settings.touchpad.tap = value;
            }
            TouchpadMessage::ToggleDwt(value) => {
                settings.touchpad.dwt = value;
            }
            TouchpadMessage::ToggleDwtp(value) => {
                settings.touchpad.dwtp = value;
            }
            TouchpadMessage::ToggleNaturalScroll(value) => {
                settings.touchpad.natural_scroll = value;
            }
            TouchpadMessage::SetAccelSpeed(value) => {
                settings.touchpad.accel_speed = value.clamp(-1.0, 1.0) as f64;
            }
            TouchpadMessage::SetAccelProfile(profile) => {
                settings.touchpad.accel_profile = profile;
            }
            TouchpadMessage::SetScrollFactor(value) => {
                settings.touchpad.scroll_factor = value.clamp(0.1, 10.0) as f64;
            }
            TouchpadMessage::SetScrollMethod(method) => {
                settings.touchpad.scroll_method = method;
            }
            TouchpadMessage::SetClickMethod(method) => {
                settings.touchpad.click_method = method;
            }
            TouchpadMessage::SetTapButtonMap(map) => {
                settings.touchpad.tap_button_map = map;
            }
            TouchpadMessage::ToggleLeftHanded(value) => {
                settings.touchpad.left_handed = value;
            }
            TouchpadMessage::ToggleDrag(value) => {
                settings.touchpad.drag = value;
            }
            TouchpadMessage::ToggleDragLock(value) => {
                settings.touchpad.drag_lock = value;
            }
            TouchpadMessage::ToggleMiddleEmulation(value) => {
                settings.touchpad.middle_emulation = value;
            }
            TouchpadMessage::ToggleDisabledOnExternalMouse(value) => {
                settings.touchpad.disabled_on_external_mouse = value;
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Touchpad);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates cursor settings
    fn update_cursor(&mut self, msg: crate::messages::CursorMessage) -> Task<Message> {
        use crate::messages::CursorMessage;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            CursorMessage::SetTheme(value) => {
                settings.cursor.theme = value;
            }
            CursorMessage::SetSize(value) => {
                settings.cursor.size = value.clamp(16, 48);
            }
            CursorMessage::ToggleHideWhenTyping(value) => {
                settings.cursor.hide_when_typing = value;
            }
            CursorMessage::SetHideAfterInactive(value) => {
                settings.cursor.hide_after_inactive_ms = value;
            }
        }

        // Update cache for view borrowing
        self.cursor_cache = settings.cursor.clone();

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Cursor);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates animation settings
    fn update_animations(&mut self, msg: crate::messages::AnimationsMessage) -> Task<Message> {
        use crate::messages::AnimationsMessage;
        use crate::config::models::{AnimationType, EasingCurve};

        let mut settings = self.settings.lock().unwrap();

        match msg {
            AnimationsMessage::ToggleSlowdown(enabled) => {
                // Toggle between slowdown factor and normal speed (1.0)
                if enabled {
                    // Enable slowdown (if it's at 1.0, set to default 3.0)
                    if (settings.animations.slowdown - 1.0).abs() < 0.01 {
                        settings.animations.slowdown = 3.0;
                    }
                } else {
                    // Disable slowdown (set to 1.0 = normal speed)
                    settings.animations.slowdown = 1.0;
                }
            }
            AnimationsMessage::SetSlowdownFactor(value) => {
                settings.animations.slowdown = value.clamp(0.1, 10.0) as f64;
            }
            AnimationsMessage::SetAnimationEnabled(name, enabled) => {
                // Parse animation name to AnimationId
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.animation_type = if enabled {
                        AnimationType::Spring  // Default to spring when enabled
                    } else {
                        AnimationType::Off
                    };
                }
            }
            AnimationsMessage::SetAnimationDuration(name, duration) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.easing.duration_ms = duration.clamp(50, 5000);
                }
            }
            AnimationsMessage::SetAnimationCurve(name, curve_name) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.easing.curve = EasingCurve::from_kdl(&curve_name);
                }
            }
            AnimationsMessage::SetAnimationSpringDampingRatio(name, ratio) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.spring.damping_ratio = ratio.clamp(0.1, 2.0) as f64;
                }
            }
            AnimationsMessage::SetAnimationSpringEpsilon(name, epsilon) => {
                if let Some(anim_id) = Self::parse_animation_name(&name) {
                    let anim_config = anim_id.get_mut(&mut settings.animations.per_animation);
                    anim_config.spring.epsilon = epsilon.clamp(0.0001, 1.0) as f64;
                }
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Animations);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Helper to parse animation name string to AnimationId
    fn parse_animation_name(name: &str) -> Option<crate::config::models::AnimationId> {
        use crate::config::models::AnimationId;

        match name.to_lowercase().as_str() {
            "workspace_switch" | "workspace-switch" => Some(AnimationId::WorkspaceSwitch),
            "overview" => Some(AnimationId::Overview),
            "window_open" | "window-open" => Some(AnimationId::WindowOpen),
            "window_close" | "window-close" => Some(AnimationId::WindowClose),
            "window_movement" | "window-movement" => Some(AnimationId::WindowMovement),
            "window_resize" | "window-resize" => Some(AnimationId::WindowResize),
            "horizontal_view" | "horizontal-view" | "horizontal_view_movement" | "horizontal-view-movement" => Some(AnimationId::HorizontalViewMovement),
            "config_notification" | "config-notification" => Some(AnimationId::ConfigNotification),
            "exit_confirmation" | "exit-confirmation" => Some(AnimationId::ExitConfirmation),
            "screenshot_ui" | "screenshot-ui" => Some(AnimationId::ScreenshotUi),
            "recent_windows" | "recent-windows" => Some(AnimationId::RecentWindows),
            _ => None,
        }
    }

    /// Updates workspaces settings
    fn update_workspaces(&mut self, msg: crate::messages::WorkspacesMessage) -> Task<Message> {
        use crate::messages::WorkspacesMessage;
        use crate::config::models::NamedWorkspace;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            WorkspacesMessage::AddWorkspace => {
                let id = settings.workspaces.next_id;
                settings.workspaces.next_id += 1;

                let new_workspace = NamedWorkspace {
                    id,
                    name: format!("Workspace {}", settings.workspaces.workspaces.len() + 1),
                    open_on_output: None,
                    layout_override: None,
                };

                settings.workspaces.workspaces.push(new_workspace);
            }
            WorkspacesMessage::RemoveWorkspace(index) => {
                if index < settings.workspaces.workspaces.len() {
                    settings.workspaces.workspaces.remove(index);
                }
            }
            WorkspacesMessage::UpdateWorkspaceName(index, name) => {
                if let Some(workspace) = settings.workspaces.workspaces.get_mut(index) {
                    workspace.name = name;
                }
            }
            WorkspacesMessage::UpdateWorkspaceOutput(index, output) => {
                if let Some(workspace) = settings.workspaces.workspaces.get_mut(index) {
                    workspace.open_on_output = output;
                }
            }
            WorkspacesMessage::MoveWorkspaceUp(index) => {
                if index > 0 && index < settings.workspaces.workspaces.len() {
                    settings.workspaces.workspaces.swap(index - 1, index);
                }
            }
            WorkspacesMessage::MoveWorkspaceDown(index) => {
                if index < settings.workspaces.workspaces.len().saturating_sub(1) {
                    settings.workspaces.workspaces.swap(index, index + 1);
                }
            }
        }

        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Workspaces);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates window rules settings
    fn update_window_rules(&mut self, msg: crate::messages::WindowRulesMessage) -> Task<Message> {
        use crate::messages::WindowRulesMessage as M;
        use crate::config::models::{WindowRule, WindowRuleMatch};

        let mut settings = self.settings.lock().unwrap();
        let mut should_mark_dirty = true;

        match msg {
            M::AddRule => {
                let new_id = settings.window_rules.next_id;
                settings.window_rules.next_id += 1;
                let new_rule = WindowRule {
                    id: new_id,
                    name: format!("Rule {}", new_id + 1),
                    ..Default::default()
                };
                settings.window_rules.rules.push(new_rule);
                self.selected_window_rule_id = Some(new_id);
            }

            M::DeleteRule(id) => {
                settings.window_rules.rules.retain(|r| r.id != id);
                if self.selected_window_rule_id == Some(id) {
                    self.selected_window_rule_id = settings.window_rules.rules.first().map(|r| r.id);
                }
            }

            M::SelectRule(id) => {
                self.selected_window_rule_id = Some(id);
                should_mark_dirty = false;
            }

            M::DuplicateRule(id) => {
                if let Some(rule) = settings.window_rules.rules.iter().find(|r| r.id == id).cloned() {
                    let new_id = settings.window_rules.next_id;
                    settings.window_rules.next_id += 1;
                    let mut new_rule = rule;
                    new_rule.id = new_id;
                    new_rule.name = format!("{} (copy)", new_rule.name);
                    settings.window_rules.rules.push(new_rule);
                    self.selected_window_rule_id = Some(new_id);
                }
            }

            M::SetRuleName(id, name) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.name = name;
                }
            }

            M::AddMatch(id) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.matches.push(WindowRuleMatch::default());
                }
            }

            M::RemoveMatch(id, match_idx) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if match_idx < rule.matches.len() && rule.matches.len() > 1 {
                        rule.matches.remove(match_idx);
                    }
                }
            }

            M::SetMatchAppId(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.app_id = value;
                    }
                }
            }

            M::SetMatchTitle(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.title = value;
                    }
                }
            }

            M::SetMatchIsFloating(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_floating = value;
                    }
                }
            }

            M::SetMatchIsFocused(id, match_idx, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    if let Some(m) = rule.matches.get_mut(match_idx) {
                        m.is_focused = value;
                    }
                }
            }

            M::SetOpenBehavior(id, behavior) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_behavior = behavior;
                }
            }

            M::SetOpenFocused(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_focused = value;
                }
            }

            M::SetOpenOnOutput(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_on_output = value;
                }
            }

            M::SetOpenOnWorkspace(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.open_on_workspace = value;
                }
            }

            M::SetBlockScreencast(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.block_out_from_screencast = value;
                }
            }

            M::SetDefaultColumnWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.default_column_width = value;
                }
            }

            M::SetDefaultWindowHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.default_window_height = value;
                }
            }

            M::SetMinWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.min_width = value;
                }
            }

            M::SetMaxWidth(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.max_width = value;
                }
            }

            M::SetMinHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.min_height = value;
                }
            }

            M::SetMaxHeight(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.max_height = value;
                }
            }

            M::SetOpacity(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.opacity = value;
                }
            }

            M::SetCornerRadius(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.corner_radius = value;
                }
            }

            M::SetClipToGeometry(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.clip_to_geometry = value;
                }
            }

            M::SetDrawBorderWithBackground(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.draw_border_with_background = value;
                }
            }

            M::SetVariableRefreshRate(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.variable_refresh_rate = value;
                }
            }

            M::SetBabaIsFloat(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.baba_is_float = value;
                }
            }

            M::SetTiledState(id, value) => {
                if let Some(rule) = settings.window_rules.rules.iter_mut().find(|r| r.id == id) {
                    rule.tiled_state = value;
                }
            }

            M::ToggleSection(id, section) => {
                let key = (id, section);
                let current = self.window_rule_sections_expanded.get(&key).copied().unwrap_or(false);
                self.window_rule_sections_expanded.insert(key, !current);
                should_mark_dirty = false;
            }
        }

        // Update cache
        self.window_rules_cache = settings.window_rules.clone();

        drop(settings);

        if should_mark_dirty {
            self.dirty_tracker.mark(SettingsCategory::WindowRules);
            self.save_manager.mark_changed();
        }

        Task::none()
    }

    /// Updates keybindings settings (basic stub for Phase 2)
    /// TODO: Implement full keybinding editor UI in Phase 3
    fn update_keybindings(&mut self, msg: crate::messages::KeybindingsMessage) -> Task<Message> {
        use crate::messages::KeybindingsMessage as M;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            M::AddKeybinding => {
                let new_binding = crate::config::models::Keybinding {
                    id: settings.keybindings.bindings.len() as u32,
                    key_combo: String::new(),
                    action: crate::config::models::KeybindAction::NiriAction("close-window".to_string()),
                    ..Default::default()
                };
                settings.keybindings.bindings.push(new_binding);
                self.selected_keybinding_index = Some(settings.keybindings.bindings.len() - 1);
                log::info!("Added new keybinding");
            }

            M::RemoveKeybinding(idx) => {
                if idx < settings.keybindings.bindings.len() {
                    settings.keybindings.bindings.remove(idx);
                    if self.selected_keybinding_index == Some(idx) {
                        self.selected_keybinding_index = if settings.keybindings.bindings.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                    }
                    log::info!("Removed keybinding at index {}", idx);
                }
            }

            M::SelectKeybinding(idx) => {
                self.selected_keybinding_index = Some(idx);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::UpdateModifiers(_idx, _modifiers) => {
                // TODO: Implement modifier updates
                log::info!("UpdateModifiers not yet implemented");
            }

            M::StartKeyCapture(idx) => {
                self.key_capture_active = Some(idx);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::CapturedKey(key_combo) => {
                if let Some(idx) = self.key_capture_active {
                    if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                        binding.key_combo = key_combo;
                        log::info!("Captured key combo for binding {}", idx);
                    }
                }
                self.key_capture_active = None;
            }

            M::CancelKeyCapture => {
                self.key_capture_active = None;
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::UpdateAction(idx, action_str) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    if action_str == "spawn" || action_str.starts_with("spawn ") {
                        let args: Vec<String> = if action_str == "spawn" {
                            Vec::new()
                        } else {
                            action_str.strip_prefix("spawn ")
                                .unwrap_or("")
                                .split_whitespace()
                                .map(String::from)
                                .collect()
                        };
                        binding.action = crate::config::models::KeybindAction::Spawn(args);
                    } else {
                        binding.action = crate::config::models::KeybindAction::NiriAction(action_str);
                    }
                    log::info!("Updated action for binding {}", idx);
                }
            }

            M::SetCommand(idx, command) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    let args: Vec<String> = command
                        .split_whitespace()
                        .map(String::from)
                        .collect();
                    binding.action = crate::config::models::KeybindAction::Spawn(args);
                    log::info!("Updated command for binding {}", idx);
                }
            }

            M::SetAllowWhenLocked(idx, value) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    binding.allow_when_locked = value;
                    log::info!("Set allow_when_locked={} for binding {}", value, idx);
                }
            }

            M::SetRepeat(idx, value) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    binding.repeat = value;
                    log::info!("Set repeat={} for binding {}", value, idx);
                }
            }

            M::SetCooldown(idx, cooldown) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    binding.cooldown_ms = cooldown;
                    log::info!("Set cooldown={:?} for binding {}", cooldown, idx);
                }
            }

            M::SetHotkeyOverlayTitle(idx, title) => {
                if let Some(binding) = settings.keybindings.bindings.get_mut(idx) {
                    binding.hotkey_overlay_title = title;
                    log::info!("Set hotkey_overlay_title for binding {}", idx);
                }
            }

            M::ToggleSection(section) => {
                let expanded = self.keybinding_sections_expanded.get(&section).copied().unwrap_or(false);
                self.keybinding_sections_expanded.insert(section, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }
        }

        // Update the cache for view borrowing
        self.keybindings_cache = settings.keybindings.clone();
        drop(settings);

        self.dirty_tracker.mark(SettingsCategory::Keybindings);
        self.save_manager.mark_changed();

        Task::none()
    }

    /// Updates layer rules settings
    fn update_layer_rules(&mut self, msg: crate::messages::LayerRulesMessage) -> Task<Message> {
        use crate::messages::LayerRulesMessage as M;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            M::AddRule => {
                // Create a new rule with default values
                let mut new_rule = crate::config::models::LayerRule::default();
                new_rule.id = settings.layer_rules.next_id;
                settings.layer_rules.next_id += 1;
                settings.layer_rules.rules.push(new_rule.clone());
                self.selected_layer_rule_id = Some(new_rule.id);
                log::info!("Added new layer rule with ID {}", new_rule.id);
            }

            M::DeleteRule(rule_id) => {
                settings.layer_rules.rules.retain(|r| r.id != rule_id);
                if self.selected_layer_rule_id == Some(rule_id) {
                    self.selected_layer_rule_id = settings.layer_rules.rules.first().map(|r| r.id);
                }
                log::info!("Deleted layer rule {}", rule_id);
            }

            M::SelectRule(rule_id) => {
                self.selected_layer_rule_id = Some(rule_id);
            }

            M::DuplicateRule(rule_id) => {
                if let Some(rule) = settings.layer_rules.rules.iter().find(|r| r.id == rule_id).cloned() {
                    let mut new_rule = rule;
                    new_rule.id = settings.layer_rules.next_id;
                    settings.layer_rules.next_id += 1;
                    new_rule.name = format!("{} (copy)", new_rule.name);
                    settings.layer_rules.rules.push(new_rule.clone());
                    self.selected_layer_rule_id = Some(new_rule.id);
                    log::info!("Duplicated layer rule {} to {}", rule_id, new_rule.id);
                }
            }

            M::ReorderRule(rule_id, move_up) => {
                if let Some(idx) = settings.layer_rules.rules.iter().position(|r| r.id == rule_id) {
                    let new_idx = if move_up && idx > 0 {
                        idx - 1
                    } else if !move_up && idx < settings.layer_rules.rules.len() - 1 {
                        idx + 1
                    } else {
                        idx
                    };

                    if new_idx != idx {
                        let rule = settings.layer_rules.rules.remove(idx);
                        settings.layer_rules.rules.insert(new_idx, rule);
                    }
                }
            }

            M::SetRuleName(rule_id, name) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.name = name;
                }
            }

            M::AddMatch(rule_id) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.matches.push(crate::config::models::LayerRuleMatch::default());
                }
            }

            M::RemoveMatch(rule_id, match_idx) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if match_idx < rule.matches.len() {
                        rule.matches.remove(match_idx);
                    }
                }
            }

            M::SetMatchNamespace(rule_id, match_idx, namespace) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if let Some(match_data) = rule.matches.get_mut(match_idx) {
                        match_data.namespace = if namespace.is_empty() { None } else { Some(namespace) };
                    }
                }
            }

            M::SetMatchAtStartup(rule_id, match_idx, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    if let Some(match_data) = rule.matches.get_mut(match_idx) {
                        match_data.at_startup = value;
                    }
                }
            }

            M::SetBlockOutFrom(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.block_out_from = value;
                }
            }

            M::SetOpacity(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.opacity = value;
                }
            }

            M::SetCornerRadius(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.geometry_corner_radius = value;
                }
            }

            M::SetPlaceWithinBackdrop(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.place_within_backdrop = value;
                }
            }

            M::SetBabaIsFloat(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.baba_is_float = value;
                }
            }

            M::SetShadow(rule_id, value) => {
                if let Some(rule) = settings.layer_rules.rules.iter_mut().find(|r| r.id == rule_id) {
                    rule.shadow = value;
                }
            }

            M::ToggleSection(rule_id, section_name) => {
                let key = (rule_id, section_name);
                let expanded = self.layer_rule_sections_expanded.get(&key).copied().unwrap_or(true);
                self.layer_rule_sections_expanded.insert(key, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::ValidateRegex(rule_id, _match_idx, field_name, regex) => {
                // Validate regex pattern
                if regex.is_empty() {
                    self.layer_rule_regex_errors.remove(&(rule_id, field_name));
                } else {
                    match regex_syntax::Parser::new().parse(&regex) {
                        Ok(_) => {
                            self.layer_rule_regex_errors.remove(&(rule_id, field_name));
                        }
                        Err(e) => {
                            self.layer_rule_regex_errors.insert((rule_id, field_name), e.to_string());
                        }
                    }
                }
                // Don't mark dirty for validation-only changes
                return Task::none();
            }
        }

        self.dirty_tracker.mark(SettingsCategory::LayerRules);
        self.save_manager.mark_changed();
        Task::none()
    }

    /// Updates outputs (displays) settings
    fn update_outputs(&mut self, msg: crate::messages::OutputsMessage) -> Task<Message> {
        use crate::messages::OutputsMessage as M;

        let mut settings = self.settings.lock().unwrap();

        match msg {
            M::AddOutput => {
                settings.outputs.outputs.push(crate::config::models::OutputConfig::default());
                self.selected_output_index = Some(settings.outputs.outputs.len() - 1);
                log::info!("Added new output");
            }

            M::RemoveOutput(idx) => {
                if idx < settings.outputs.outputs.len() {
                    settings.outputs.outputs.remove(idx);
                    if self.selected_output_index == Some(idx) {
                        self.selected_output_index = if settings.outputs.outputs.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                    }
                    log::info!("Removed output at index {}", idx);
                }
            }

            M::SelectOutput(idx) => {
                self.selected_output_index = Some(idx);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }

            M::SetOutputName(idx, name) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.name = name;
                }
            }

            M::SetEnabled(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.enabled = value;
                }
            }

            M::SetScale(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.scale = value;
                }
            }

            M::SetMode(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.mode = value;
                }
            }

            M::SetModeCustom(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.mode_custom = value;
                }
            }

            M::SetModeline(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.modeline = value;
                }
            }

            M::SetPositionX(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.position_x = value;
                }
            }

            M::SetPositionY(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.position_y = value;
                }
            }

            M::SetTransform(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.transform = value;
                }
            }

            M::SetVrr(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.vrr = value;
                }
            }

            M::SetFocusAtStartup(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.focus_at_startup = value;
                }
            }

            M::SetBackdropColor(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.backdrop_color = value;
                }
            }

            M::SetHotCornersEnabled(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.enabled = value;
                    } else if value.is_some() {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            enabled: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerTopLeft(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.top_left = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            top_left: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerTopRight(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.top_right = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            top_right: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerBottomLeft(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.bottom_left = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            bottom_left: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetHotCornerBottomRight(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    if let Some(ref mut hot_corners) = output.hot_corners {
                        hot_corners.bottom_right = value;
                    } else {
                        output.hot_corners = Some(crate::config::models::OutputHotCorners {
                            bottom_right: value,
                            ..Default::default()
                        });
                    }
                }
            }

            M::SetLayoutOverride(idx, value) => {
                if let Some(output) = settings.outputs.outputs.get_mut(idx) {
                    output.layout_override = value;
                }
            }

            M::ToggleSection(section_name) => {
                let expanded = self.output_sections_expanded.get(&section_name).copied().unwrap_or(true);
                self.output_sections_expanded.insert(section_name, !expanded);
                // Don't mark dirty for UI-only changes
                return Task::none();
            }
        }

        // Update the cache for view borrowing
        self.outputs_cache = settings.outputs.clone();
        drop(settings); // Explicitly drop the lock before other operations

        self.dirty_tracker.mark(SettingsCategory::Outputs);
        self.save_manager.mark_changed();
        Task::none()
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
