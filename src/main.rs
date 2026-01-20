//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point that initializes the application with Floem UI.

use anyhow::Result;
use log::{error, info, warn};
use std::sync::{Arc, Mutex};

use niri_settings::{config, ui};

fn main() -> Result<()> {
    init_logging();

    let paths = Arc::new(config::ConfigPaths::new()?);
    let is_first_run = paths.is_first_run();

    let (settings, import_result) = init_settings(&paths, is_first_run);

    // Ensure directories exist
    if let Err(e) = paths.ensure_directories() {
        warn!("Failed to create config directories: {}", e);
    }

    // Load app preferences (UI theme, etc.)
    let prefs = config::load_preferences(&paths.preferences_json);
    let initial_theme = prefs.theme;
    info!("Loaded app preferences, theme: {:?}", initial_theme);

    // Create app state
    let state = ui::AppState::new(settings.clone(), paths.clone());

    // Launch Floem app with wizard on first run
    floem::launch(move || {
        ui::app_view(state.clone(), initial_theme, is_first_run, import_result.clone())
    });

    // Note: We don't save all settings on exit anymore.
    // Auto-save handles changes as they happen via mark_dirty_and_save().
    // This prevents overwriting manual edits to config files.

    Ok(())
}

/// Initialize logging with env_logger
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

/// Load settings from config files
/// Returns (settings, import_result) where import_result is Some on first run
fn init_settings(
    paths: &config::ConfigPaths,
    is_first_run: bool,
) -> (Arc<Mutex<config::Settings>>, Option<config::ImportResult>) {
    let (loaded_settings, import_result) = if is_first_run {
        // First run: import from user's existing niri config
        info!("First run - importing settings from existing niri config");
        let result = config::import_from_niri_config_with_result(&paths.niri_config);
        info!("Import result: {}", result.summary());

        // Save settings immediately so the files exist
        if let Err(e) = config::save_settings(paths, &result.settings) {
            error!("Failed to save imported settings: {}", e);
        } else {
            info!("Imported settings saved to niri-settings directory");
        }

        let settings = result.settings.clone();
        (settings, Some(result))
    } else {
        // Normal: load from managed config files
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

