//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point that initializes the application, loads settings,
//! and sets up all UI callbacks.

use anyhow::Result;
use log::{debug, error, info, warn};
use niri_settings::{config, handlers, ui, wizard, MainWindow};
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    init_logging();

    let paths = Arc::new(config::ConfigPaths::new()?);
    let is_first_run = paths.is_first_run();

    let (settings, import_result) = init_settings(&paths, is_first_run);

    let ui = MainWindow::new()?;

    // Set import summary for first-run wizard and import details dialog
    if let Some(ref result) = import_result {
        wizard::set_import_summary(&ui, result);
        // Analyze imported rules for consolidation opportunities
        wizard::set_consolidation_suggestions(&ui, &result.settings);
    }

    wizard::setup_first_run(&ui, &paths, is_first_run);
    check_config_health_on_startup(&ui, &paths);
    wizard::setup_wizard_callbacks(&ui, settings.clone());
    wizard::setup_consolidation_callbacks(&ui, settings.clone(), paths.clone());
    wizard::setup_include_line_handler(
        &ui,
        paths.clone(),
        settings.clone(),
        handlers::show_error,
        handlers::show_status,
    );
    handlers::setup_error_handler(&ui);
    handlers::setup_search_handler(&ui);

    let saved_on_close = handlers::setup_close_handler(&ui, settings.clone(), paths.clone());

    handlers::setup_category_handler(&ui);
    handlers::setup_search_result_handler(&ui);
    ui::bridge::setup_callbacks(&ui, settings.clone(), paths.clone());

    handlers::setup_hardware_detection(&ui);
    handlers::setup_niri_info(&ui);
    handlers::setup_validation_handler(&ui);
    handlers::setup_tools_handler(&ui);
    handlers::setup_config_editor_handler(&ui, paths.clone());

    ensure_directories(&paths);

    ui.run()?;

    final_save_on_exit(&saved_on_close, &settings, &paths);

    Ok(())
}

/// Initialize logging with env_logger
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

/// Load settings from config files
///
/// On first run, imports settings from the user's existing niri config.kdl
/// to preserve their existing configuration. On subsequent runs, loads from
/// the managed config files in ~/.config/niri/niri-settings/.
///
/// Returns the settings and optionally the import result (on first run).
fn init_settings(
    paths: &config::ConfigPaths,
    is_first_run: bool,
) -> (Arc<Mutex<config::Settings>>, Option<config::ImportResult>) {
    let (loaded_settings, import_result) = if is_first_run {
        // First run: import from user's existing niri config
        info!("First run - importing settings from existing niri config");
        let result = config::import_from_niri_config_with_result(&paths.niri_config);
        info!("Import result: {}", result.summary());

        // IMPORTANT: Save settings immediately so the files exist before wizard modifies config.kdl
        // This ensures niri can load the included files even if it reloads mid-wizard
        if let Err(e) = config::save_settings(paths, &result.settings) {
            error!("Failed to save imported settings: {}", e);
        } else {
            info!("Imported settings saved to niri-settings directory");
        }

        (result.settings.clone(), Some(result))
    } else {
        // Normal: load from managed config files with detailed result
        let load_result = config::load_settings_with_result(paths);
        info!("{}", load_result.summary());

        // Log any warnings about failed files
        if load_result.has_failures() {
            for warning in &load_result.warnings {
                warn!("{}", warning);
            }
        }

        (load_result.settings, None)
    };

    info!(
        "Loaded settings (first_run: {}, {} outputs, {} window rules, {} keybindings)",
        is_first_run,
        loaded_settings.outputs.outputs.len(),
        loaded_settings.window_rules.rules.len(),
        loaded_settings.keybindings.bindings.len()
    );
    (Arc::new(Mutex::new(loaded_settings)), import_result)
}

/// Check config health and warn about corrupted files
fn check_config_health_on_startup(ui: &MainWindow, paths: &config::ConfigPaths) {
    // Skip health check if this is a first run (no config files exist yet)
    if paths.is_first_run() {
        return;
    }

    let health = config::check_config_health(paths);
    if !health.is_healthy() {
        let corrupted = health.corrupted_files();
        let unreadable = health.unreadable_files();

        let mut issues = Vec::new();
        if !corrupted.is_empty() {
            issues.push(format!("{} corrupted", corrupted.len()));
            warn!("Corrupted config files: {:?}", corrupted);
        }
        if !unreadable.is_empty() {
            issues.push(format!("{} unreadable", unreadable.len()));
            warn!("Unreadable config files: {:?}", unreadable);
        }

        let message = format!(
            "{} config file(s) have issues, using defaults",
            corrupted.len() + unreadable.len()
        );
        handlers::show_status(ui, &message, true);
    }
}

/// Ensure config directories exist
fn ensure_directories(paths: &config::ConfigPaths) {
    if let Err(e) = paths.ensure_directories() {
        warn!("Failed to create config directories: {}", e);
    }
}

/// Final save on normal exit (only if close handler didn't already save)
fn final_save_on_exit(
    saved_on_close: &Arc<AtomicBool>,
    settings: &Arc<Mutex<config::Settings>>,
    paths: &Arc<config::ConfigPaths>,
) {
    // Relaxed ordering is sufficient for this simple boolean flag
    if !saved_on_close.load(Ordering::Relaxed) {
        // Clone settings while holding lock, then release before I/O
        // Use into_inner() to recover data even if mutex was poisoned by a panic
        let settings_copy = match settings.lock() {
            Ok(s) => s.clone(),
            Err(poisoned) => {
                warn!(
                    "Settings mutex was poisoned on exit - a callback likely panicked. \
                     Recovering data to save. Check logs for panic details."
                );
                poisoned.into_inner().clone()
            }
        }; // Lock released here

        // Save settings to disk (no mutex held during I/O)
        if let Err(e) = config::save_settings(paths, &settings_copy) {
            warn!("Failed to save settings on exit: {}", e);
        } else {
            debug!("Final save on exit completed");
        }
    }
}
