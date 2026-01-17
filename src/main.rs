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

    {
        let s = settings.lock().unwrap();
        info!(
            "Starting niri-settings (first_run: {}, {} outputs, {} window rules, {} keybindings)",
            is_first_run,
            s.outputs.outputs.len(),
            s.window_rules.rules.len(),
            s.keybindings.bindings.len()
        );
    }

    // Launch Dioxus app
    dioxus_native::launch(App);
    println!("Dioxus app exited");

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

/// Navigation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Appearance,
    Layout,
    Animations,
    Keyboard,
    Mouse,
    Touchpad,
    WindowRules,
    Keybindings,
    Outputs,
    Startup,
}

impl Category {
    fn label(&self) -> &'static str {
        match self {
            Category::Appearance => "Appearance",
            Category::Layout => "Layout",
            Category::Animations => "Animations",
            Category::Keyboard => "Keyboard",
            Category::Mouse => "Mouse",
            Category::Touchpad => "Touchpad",
            Category::WindowRules => "Window Rules",
            Category::Keybindings => "Keybindings",
            Category::Outputs => "Outputs",
            Category::Startup => "Startup",
        }
    }

    fn all() -> &'static [Category] {
        &[
            Category::Appearance,
            Category::Layout,
            Category::Animations,
            Category::Keyboard,
            Category::Mouse,
            Category::Touchpad,
            Category::WindowRules,
            Category::Keybindings,
            Category::Outputs,
            Category::Startup,
        ]
    }
}

/// Root application component
#[component]
fn App() -> Element {
    let selected = use_signal(|| Category::Appearance);

    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            Sidebar { selected }
            PageContent { selected: selected() }
        }
    }
}

/// Sidebar navigation component
#[component]
fn Sidebar(selected: Signal<Category>) -> Element {
    rsx! {
        nav { class: "sidebar",
            h2 { "Settings" }
            ul {
                for category in Category::all() {
                    li {
                        class: if selected() == *category { "active" } else { "" },
                        onclick: move |_| selected.set(*category),
                        "{category.label()}"
                    }
                }
            }
        }
    }
}

/// Main content area - shows the selected page
#[component]
fn PageContent(selected: Category) -> Element {
    rsx! {
        main { class: "content",
            h1 { "{selected.label()}" }
            p { class: "placeholder", "Settings for {selected.label()} will appear here." }
        }
    }
}
