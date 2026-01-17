//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point. Uses Dioxus with the Blitz native renderer.

use anyhow::Result;
use dioxus_native::prelude::*;
use log::info;
use std::sync::{Arc, Mutex};

use niri_settings::config;

fn main() -> Result<()> {
    init_logging();

    let paths = Arc::new(config::ConfigPaths::new()?);
    let is_first_run = paths.is_first_run();

    let settings = init_settings(&paths, is_first_run);

    info!(
        "Starting niri-settings (first_run: {}, {} outputs, {} window rules, {} keybindings)",
        is_first_run,
        settings.lock().unwrap().outputs.outputs.len(),
        settings.lock().unwrap().window_rules.rules.len(),
        settings.lock().unwrap().keybindings.bindings.len()
    );

    // Launch Dioxus app
    dioxus_native::launch(App);

    Ok(())
}

/// Initialize logging with env_logger
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

/// Load settings from config files
fn init_settings(
    paths: &config::ConfigPaths,
    is_first_run: bool,
) -> Arc<Mutex<config::Settings>> {
    let loaded_settings = if is_first_run {
        info!("First run - importing settings from existing niri config");
        let result = config::import_from_niri_config_with_result(&paths.niri_config);
        info!("Import result: {}", result.summary());

        if let Err(e) = config::save_settings(paths, &result.settings) {
            log::error!("Failed to save imported settings: {}", e);
        }

        result.settings
    } else {
        let load_result = config::load_settings_with_result(paths);
        info!("{}", load_result.summary());
        load_result.settings
    };

    Arc::new(Mutex::new(loaded_settings))
}

/// Root application component
#[component]
fn App() -> Element {
    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            Sidebar {}
            main { class: "content",
                h1 { "niri settings" }
                p { "Dioxus migration in progress..." }
                p { "The core config loading is working. UI components coming soon." }
            }
        }
    }
}

/// Sidebar navigation component
#[component]
fn Sidebar() -> Element {
    rsx! {
        nav { class: "sidebar",
            h2 { "Settings" }
            ul {
                li { class: "active", "Appearance" }
                li { "Layout" }
                li { "Animations" }
                li { "Keyboard" }
                li { "Mouse" }
                li { "Touchpad" }
                li { "Window Rules" }
                li { "Keybindings" }
                li { "Outputs" }
                li { "Startup" }
            }
        }
    }
}
