//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! Tessera UI version - All UI code written in Rust using Tessera framework
//! Backend logic (config loading, KDL parsing) is unchanged from Slint version

use anyhow::Result;
use log::{debug, error, info, warn};
use niri_settings::{config, tessera_ui, Settings};
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    init_logging();

    let paths = Arc::new(config::ConfigPaths::new()?);
    let is_first_run = paths.is_first_run();

    let (settings, import_result) = init_settings(&paths, is_first_run);

    // TODO: Handle first-run wizard with Tessera
    if let Some(ref result) = import_result {
        info!("Import summary: {}", result.summary());
    }

    ensure_directories(&paths);

    // Run Tessera UI
    tessera_ui::run_app(settings)?;

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
) -> (Arc<Mutex<Settings>>, Option<config::ImportResult>) {
    let (loaded_settings, import_result) = if is_first_run {
        // First run: import from user's existing niri config
        info!("First run - importing settings from existing niri config");
        let result = config::import_from_niri_config_with_result(&paths.niri_config);
        info!("Import result: {}", result.summary());

        // Save settings immediately
        if let Err(e) = config::save_settings(paths, &result.settings) {
            error!("Failed to save imported settings: {}", e);
        } else {
            info!("Imported settings saved to niri-settings directory");
        }

        (result.settings.clone(), Some(result))
    } else {
        // Normal: load from managed config files
        let load_result = config::load_settings_with_result(paths);
        info!("{}", load_result.summary());

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

/// Ensure config directories exist
fn ensure_directories(paths: &config::ConfigPaths) {
    if let Err(e) = paths.ensure_directories() {
        warn!("Failed to create config directories: {}", e);
    }
}
