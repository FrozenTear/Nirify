//! UI callback handlers for the main window
//!
//! This module contains setup functions for various UI callbacks including:
//! - Error and status display
//! - Search functionality
//! - Window close handling
//! - Hardware detection
//! - Niri IPC integration (tools, validation)
//! - Config editor and backup browser

use log::{debug, info, warn};
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::{self, ConfigPaths, Settings};
use crate::{ipc, ui, MainWindow};

// ============================================================================
// Status and Error Display
// ============================================================================

/// Show an error dialog
pub fn show_error(ui: &MainWindow, title: &str, message: &str, details: &str) {
    ui.set_error_title(title.into());
    ui.set_error_message(message.into());
    ui.set_error_details(details.into());
    ui.set_error_dialog_visible(true);
}

/// Show a status notification that auto-hides
pub fn show_status(ui: &MainWindow, message: &str, is_error: bool) {
    use crate::constants::STATUS_AUTO_HIDE_SECS;

    ui.set_status_message(message.into());
    ui.set_status_is_error(is_error);
    ui.set_status_visible(true);

    // Auto-hide after delay
    let ui_weak = ui.as_weak();
    slint::Timer::single_shot(
        std::time::Duration::from_secs(STATUS_AUTO_HIDE_SECS),
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_status_visible(false);
            }
        },
    );
}

// ============================================================================
// Core UI Handlers
// ============================================================================

/// Set up error dialog dismissed callback
pub fn setup_error_handler(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_error_dismissed(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_error_dialog_visible(false);
        }
    });
}

/// Set up search callback for navigating to settings
///
/// Uses debouncing (200ms) to avoid redundant searches while typing quickly.
/// Populates the search results panel with matching settings.
pub fn setup_search_handler(ui: &MainWindow) {
    use slint::{ModelRc, VecModel};
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::SearchResultItem;

    // Debounce timer and pending query
    let timer = Rc::new(slint::Timer::default());
    let pending_query = Rc::new(RefCell::new(String::new()));

    let ui_weak = ui.as_weak();
    let timer_clone = timer.clone();
    let pending_clone = pending_query.clone();

    ui.on_search_changed(move |query| {
        let query_str: String = query.into();

        // Store the pending query
        *pending_clone.borrow_mut() = query_str.clone();

        // Clear results immediately when query is empty
        if query_str.is_empty() {
            if let Some(ui) = ui_weak.upgrade() {
                let empty_model = Rc::new(VecModel::<SearchResultItem>::default());
                ui.set_search_results(ModelRc::from(empty_model));
                ui.set_show_search_results(false);
            }
            return;
        }

        // Set up debounced search (200ms delay)
        let ui_weak_inner = ui_weak.clone();
        let pending_inner = pending_clone.clone();

        timer_clone.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(200),
            move || {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    let query = pending_inner.borrow().clone();
                    if !query.is_empty() {
                        // Get all search results
                        let results = ui::search::search_settings(&query);

                        // Convert to Slint SearchResultItem array
                        let search_items: Vec<SearchResultItem> = results
                            .into_iter()
                            .take(10) // Limit to 10 results
                            .map(|r| SearchResultItem {
                                category: r.category,
                                label: r.label.into(),
                                description: r.description.into(),
                                score: r.score,
                            })
                            .collect();

                        let has_results = !search_items.is_empty();
                        debug!("Search '{}' -> {} results", query, search_items.len());

                        // Set the search results model
                        let model = Rc::new(VecModel::from(search_items));
                        ui.set_search_results(ModelRc::from(model));
                        ui.set_show_search_results(has_results);
                    }
                }
            },
        );
    });
}

/// Set up close handler for saving settings on window close
pub fn setup_close_handler(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    paths: Arc<ConfigPaths>,
) -> Arc<AtomicBool> {
    let saved_on_close = Arc::new(AtomicBool::new(false));

    let ui_weak = ui.as_weak();
    let settings_for_close = settings;
    let paths_for_close = paths;
    let saved_flag = saved_on_close.clone();

    ui.on_close_requested(move || {
        // Hide window immediately for responsive UX
        if let Some(ui) = ui_weak.upgrade() {
            ui.window().hide().ok();
        }

        // Clone settings while holding lock, then release before I/O
        // Use into_inner() to recover data even if mutex was poisoned by a panic
        let settings_copy = {
            match settings_for_close.lock() {
                Ok(s) => s.clone(),
                Err(poisoned) => {
                    warn!(
                        "Settings mutex was poisoned during close handler - \
                         a callback likely panicked. Recovering data to save. \
                         Check logs for panic details."
                    );
                    poisoned.into_inner().clone()
                }
            }
        }; // Lock released here

        // Save settings to disk (no mutex held during I/O)
        if let Err(e) = config::save_settings(&paths_for_close, &settings_copy) {
            warn!("Failed to save settings on close: {}", e);
        } else {
            info!("Settings saved successfully");
            // Relaxed ordering is sufficient for this simple boolean flag
            saved_flag.store(true, Ordering::Relaxed);
            // Try to reload niri config if running (async to prevent freeze on close)
            if ipc::is_niri_running() {
                ipc::async_ops::reload_config_async(|result| {
                    if let Err(e) = result {
                        debug!("Could not reload niri config: {}", e);
                    }
                });
            }
        }
    });

    saved_on_close
}

/// Set up category change handler for sidebar navigation
pub fn setup_category_handler(ui: &MainWindow) {
    ui.on_category_changed(|index| {
        info!("Category changed to: {}", index);
    });
}

/// Set up search result selection handler
///
/// This handles navigation when a search result is clicked.
/// We handle this in Rust because setting properties from inside a
/// conditionally-rendered component's callback in Slint can have
/// reactivity issues.
pub fn setup_search_result_handler(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_search_result_selected(move |category| {
        info!("Search result selected: category {}", category);
        if let Some(ui) = ui_weak.upgrade() {
            // Set the selected category - this controls which page is shown
            ui.set_selected_category(category);

            // Also update nav-group and nav-page for CategoryNav sync
            // These are computed from category using the same logic as Slint
            let (nav_group, nav_page) = category_to_nav(category);
            ui.set_nav_group(nav_group);
            ui.set_nav_page(nav_page);

            // Hide search results and clear search text
            ui.set_show_search_results(false);
            ui.set_search_text("".into());

            info!(
                "Navigation set: category={}, nav_group={}, nav_page={}",
                category, nav_group, nav_page
            );
        }
    });
}

/// Convert category index to (nav_group, nav_page)
/// This mirrors the logic in main.slint's category-to-group and category-to-page
fn category_to_nav(category: i32) -> (i32, i32) {
    // SidebarIndices values (from sidebar.slint)
    const APPEARANCE: i32 = 0;
    const BEHAVIOR: i32 = 1;
    const KEYBOARD: i32 = 2;
    const MOUSE: i32 = 3;
    const TOUCHPAD: i32 = 4;
    const DISPLAYS: i32 = 5;
    const ANIMATIONS: i32 = 6;
    const CURSOR: i32 = 7;
    const OVERVIEW: i32 = 8;
    const LAYOUT_EXTRAS: i32 = 9;
    const GESTURES: i32 = 10;
    const MISCELLANEOUS: i32 = 11;
    const WINDOW_RULES: i32 = 12;
    const WORKSPACES: i32 = 14;
    const LAYER_RULES: i32 = 15;
    const TRACKPOINT: i32 = 16;
    const TRACKBALL: i32 = 17;
    const TABLET: i32 = 18;
    const TOUCH: i32 = 19;
    const KEYBINDINGS: i32 = 20;
    const STARTUP: i32 = 21;
    const ENVIRONMENT: i32 = 22;
    const DEBUG: i32 = 23;
    const SWITCH_EVENTS: i32 = 24;
    const RECENT_WINDOWS: i32 = 25;

    let nav_group = match category {
        APPEARANCE => 0,
        KEYBOARD | MOUSE | TOUCHPAD | TRACKPOINT | TRACKBALL | TABLET | TOUCH | DISPLAYS => 1,
        ANIMATIONS | CURSOR | OVERVIEW | RECENT_WINDOWS => 2,
        BEHAVIOR | LAYOUT_EXTRAS | WORKSPACES => 3,
        WINDOW_RULES | LAYER_RULES | GESTURES => 4,
        _ => 5, // System group
    };

    let nav_page = match category {
        APPEARANCE => 0,
        KEYBOARD => 0,
        MOUSE => 1,
        TOUCHPAD => 2,
        TRACKPOINT => 3,
        TRACKBALL => 4,
        TABLET => 5,
        TOUCH => 6,
        DISPLAYS => 7,
        ANIMATIONS => 0,
        CURSOR => 1,
        OVERVIEW => 2,
        RECENT_WINDOWS => 3,
        BEHAVIOR => 0,
        LAYOUT_EXTRAS => 1,
        WORKSPACES => 2,
        WINDOW_RULES => 0,
        LAYER_RULES => 1,
        GESTURES => 2,
        KEYBINDINGS => 0,
        STARTUP => 1,
        ENVIRONMENT => 2,
        SWITCH_EVENTS => 3,
        MISCELLANEOUS => 4,
        DEBUG => 5,
        _ => 0,
    };

    (nav_group, nav_page)
}

// ============================================================================
// Hardware and System Info
// ============================================================================

/// Detect available input hardware and update UI visibility
pub fn setup_hardware_detection(ui: &MainWindow) {
    use crate::hardware::InputDevices;

    let devices = InputDevices::detect();
    info!(
        "Hardware detection: touchpad={}, trackpoint={}, trackball={}, tablet={}, touch={}",
        devices.has_touchpad,
        devices.has_trackpoint,
        devices.has_trackball,
        devices.has_tablet,
        devices.has_touch
    );

    ui.set_has_touchpad(devices.has_touchpad);
    ui.set_has_trackpoint(devices.has_trackpoint);
    ui.set_has_trackball(devices.has_trackball);
    ui.set_has_tablet(devices.has_tablet);
    ui.set_has_touch(devices.has_touch);
}

/// Get niri version and running status
pub fn setup_niri_info(ui: &MainWindow) {
    let running = ipc::is_niri_running();
    ui.set_niri_running(running);

    if running {
        match ipc::get_version() {
            Ok(version) => {
                info!("Niri version: {}", version);
                ui.set_niri_version(version.into());
            }
            Err(e) => {
                warn!("Failed to get niri version: {}", e);
                ui.set_niri_version("unknown".into());
            }
        }
    } else {
        info!("Niri is not running");
        ui.set_niri_version("not running".into());
    }
}

// ============================================================================
// Validation and Tools
// ============================================================================

/// Set up config validation callback
pub fn setup_validation_handler(ui: &MainWindow) {
    let ui_weak = ui.as_weak();
    ui.on_validate_config_requested(move || {
        if let Some(ui) = ui_weak.upgrade() {
            match ipc::validate_config() {
                Ok(msg) => {
                    info!("Config validation passed: {}", msg);
                    ui.set_validation_result(msg.into());
                    ui.set_validation_success(true);
                    show_status(&ui, "Configuration is valid", false);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Config validation failed: {}", error_msg);
                    ui.set_validation_result(error_msg.clone().into());
                    ui.set_validation_success(false);
                    show_status(&ui, &format!("Validation failed: {}", error_msg), true);
                }
            }
        }
    });
}

/// Set up tools page callbacks for querying niri state
///
/// All IPC calls run on background threads to prevent UI freezes.
pub fn setup_tools_handler(ui: &MainWindow) {
    // List windows (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_list_windows(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Loading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::get_windows_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(windows) => {
                            let output = if windows.is_empty() {
                                "No windows found".to_string()
                            } else {
                                windows
                                    .iter()
                                    .map(|w| {
                                        format!(
                                            "ID: {}\n  Title: {}\n  App ID: {}\n  Floating: {}",
                                            w.id, w.title, w.app_id, w.is_floating
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            };
                            ui.set_tools_output(output.into());
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                        }
                    }
                }
            });
        }
    });

    // List workspaces (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_list_workspaces(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Loading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::get_workspaces_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(workspaces) => {
                            let output = if workspaces.is_empty() {
                                "No workspaces found".to_string()
                            } else {
                                workspaces
                                    .iter()
                                    .map(|ws| {
                                        let name = ws.name.as_deref().unwrap_or("(unnamed)");
                                        let output = ws.output.as_deref().unwrap_or("(none)");
                                        let focused =
                                            if ws.is_focused { " [FOCUSED]" } else { "" };
                                        let active = if ws.is_active { " [ACTIVE]" } else { "" };
                                        format!(
                                            "ID: {} (idx: {}){}{}\n  Name: {}\n  Output: {}",
                                            ws.id, ws.idx, focused, active, name, output
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            };
                            ui.set_tools_output(output.into());
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                        }
                    }
                }
            });
        }
    });

    // List outputs (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_list_outputs(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Loading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::get_full_outputs_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(outputs) => {
                            let output = if outputs.is_empty() {
                                "No outputs found".to_string()
                            } else {
                                outputs
                                    .iter()
                                    .map(|o| {
                                        format!(
                                            "{}\n  Make: {}\n  Model: {}\n  Mode: {}\n  Scale: {:.2}x\n  Position: ({}, {})\n  VRR: {}",
                                            o.name,
                                            if o.make.is_empty() { "(unknown)" } else { &o.make },
                                            if o.model.is_empty() { "(unknown)" } else { &o.model },
                                            o.current_mode_string(),
                                            o.scale(),
                                            o.position_x(),
                                            o.position_y(),
                                            if o.vrr_enabled { "enabled" } else { "disabled" }
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            };
                            ui.set_tools_output(output.into());
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                        }
                    }
                }
            });
        }
    });

    // Focused window (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_focused_window(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Loading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::get_focused_window_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(Some(w)) => {
                            let output = format!(
                                "ID: {}\nTitle: {}\nApp ID: {}\nFloating: {}",
                                w.id, w.title, w.app_id, w.is_floating
                            );
                            ui.set_tools_output(output.into());
                        }
                        Ok(None) => {
                            ui.set_tools_output("No window is currently focused".into());
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                        }
                    }
                }
            });
        }
    });

    // Focused output (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_focused_output(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Loading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::get_focused_output_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(Some(name)) => {
                            ui.set_tools_output(format!("Focused output: {}", name).into());
                        }
                        Ok(None) => {
                            ui.set_tools_output("No output is currently focused".into());
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                        }
                    }
                }
            });
        }
    });

    // Reload config (async)
    let ui_weak = ui.as_weak();
    ui.on_tools_reload_config(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_tools_output("Reloading...".into());

            let ui_weak_inner = ui_weak.clone();
            ipc::async_ops::reload_config_async(move |result| {
                if let Some(ui) = ui_weak_inner.upgrade() {
                    match result {
                        Ok(()) => {
                            ui.set_tools_output("Configuration reloaded successfully".into());
                            show_status(&ui, "Config reloaded", false);
                        }
                        Err(e) => {
                            ui.set_tools_output(format!("Error: {}", e).into());
                            show_status(&ui, &format!("Reload failed: {}", e), true);
                        }
                    }
                }
            });
        }
    });
}

// ============================================================================
// Config Editor
// ============================================================================

/// Set up config editor callbacks for viewing KDL files
pub fn setup_config_editor_handler(ui: &MainWindow, paths: Arc<ConfigPaths>) {
    use std::cell::Cell;
    use std::rc::Rc;

    // File list in same order as UI combobox. This order is intentionally different
    // from ConfigFile::ALL to group related files together in the dropdown (e.g., all
    // input files together, advanced files at the end). Keep in sync with ConfigFile
    // when adding new config files.
    let file_list: Vec<&'static str> = vec![
        "main.kdl",
        "appearance.kdl",
        "behavior.kdl",
        "animations.kdl",
        "cursor.kdl",
        "overview.kdl",
        "outputs.kdl",
        "workspaces.kdl",
        "keybindings.kdl",
        "input/keyboard.kdl",
        "input/mouse.kdl",
        "input/touchpad.kdl",
        "input/trackpoint.kdl",
        "input/trackball.kdl",
        "input/tablet.kdl",
        "input/touch.kdl",
        "advanced/layout-extras.kdl",
        "advanced/gestures.kdl",
        "advanced/misc.kdl",
        "advanced/startup.kdl",
        "advanced/environment.kdl",
        "advanced/debug.kdl",
        "advanced/switch-events.kdl",
        "advanced/window-rules.kdl",
        "advanced/layer-rules.kdl",
        "advanced/recent-windows.kdl",
    ];

    // Track current file index for refresh (Cell is simpler than RefCell for Copy types)
    let current_index = Rc::new(Cell::new(0i32));

    // Load file callback
    let paths_load = paths.clone();
    let file_list_load = file_list.clone();
    let current_index_load = current_index.clone();
    let ui_weak = ui.as_weak();
    ui.on_config_editor_load_file(move |idx| {
        current_index_load.set(idx);

        if let Some(ui) = ui_weak.upgrade() {
            let content = load_config_file(&paths_load.managed_dir, &file_list_load, idx);
            ui.set_config_editor_content(content.into());
        }
    });

    // Refresh callback (reloads current file)
    let paths_refresh = paths;
    let file_list_refresh = file_list;
    let ui_weak = ui.as_weak();
    ui.on_config_editor_refresh(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let idx = current_index.get();
            let content = load_config_file(&paths_refresh.managed_dir, &file_list_refresh, idx);
            ui.set_config_editor_content(content.into());
            show_status(&ui, "File refreshed", false);
        }
    });
}

/// Load a config file by index
fn load_config_file(managed_dir: &std::path::Path, file_list: &[&str], idx: i32) -> String {
    let idx = idx as usize;
    if idx >= file_list.len() {
        return "Invalid file index".to_string();
    }

    let file_path = managed_dir.join(file_list[idx]);

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            if content.is_empty() {
                format!("// File is empty: {}", file_path.display())
            } else {
                content
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            format!(
                "// File not found: {}\n// This file will be created when you configure settings for this category.",
                file_path.display()
            )
        }
        Err(e) => {
            format!("// Error reading file: {}", e)
        }
    }
}
