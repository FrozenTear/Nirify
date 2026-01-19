//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point that initializes the application with Freya UI.

use anyhow::Result;
use freya::prelude::*;
use log::{error, info, warn};
use std::sync::{Arc, Mutex};

use niri_settings::{config, ui};

fn main() -> Result<()> {
    init_logging();

    let paths = Arc::new(config::ConfigPaths::new()?);
    let is_first_run = paths.is_first_run();

    let settings = init_settings(&paths, is_first_run);

    // Ensure directories exist
    if let Err(e) = paths.ensure_directories() {
        warn!("Failed to create config directories: {}", e);
    }

    // Create app state
    let state = ui::AppState::new(settings.clone(), paths.clone());

    // Launch Freya app
    launch(
        LaunchConfig::new()
            .with_window(
                WindowConfig::new(move || ui::app_view(state.clone()))
                    .with_size(1100.0, 750.0)
                    .with_min_size(800.0, 500.0)
                    .with_title("Niri Settings")
                    .with_resizable(true),
            ),
    );

    // Save settings on exit
    save_on_exit(&settings, &paths);

    Ok(())
}

/// Initialize logging with env_logger
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

/// Load settings from config files
fn init_settings(paths: &config::ConfigPaths, is_first_run: bool) -> Arc<Mutex<config::Settings>> {
    let loaded_settings = if is_first_run {
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

        result.settings
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

        load_result.settings
    };

    info!(
        "Loaded settings (first_run: {}, {} outputs, {} window rules, {} keybindings)",
        is_first_run,
        loaded_settings.outputs.outputs.len(),
        loaded_settings.window_rules.rules.len(),
        loaded_settings.keybindings.bindings.len()
    );

    Arc::new(Mutex::new(loaded_settings))
}

/// Save settings when the app exits
fn save_on_exit(settings: &Arc<Mutex<config::Settings>>, paths: &Arc<config::ConfigPaths>) {
    let settings_copy = match settings.lock() {
        Ok(s) => s.clone(),
        Err(poisoned) => {
            warn!("Settings mutex was poisoned on exit - recovering data");
            poisoned.into_inner().clone()
        }
    };

    if let Err(e) = config::save_settings(paths, &settings_copy) {
        warn!("Failed to save settings on exit: {}", e);
    } else {
        info!("Settings saved successfully on exit");
    }
}
