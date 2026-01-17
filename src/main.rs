//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point. Uses Dioxus with the Blitz native renderer.

use anyhow::Result;
use dioxus_native::prelude::*;
use log::info;
use std::sync::{Arc, Mutex};

use niri_settings::config;
use niri_settings::config::models::AppearanceSettings;
use niri_settings::constants::*;

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

// ============================================================================
// Navigation
// ============================================================================

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

// ============================================================================
// Root App
// ============================================================================

#[component]
fn App() -> Element {
    let selected = use_signal(|| Category::Appearance);

    // Initialize appearance settings with defaults (will load from config later)
    let appearance = use_signal(AppearanceSettings::default);

    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            Sidebar { selected }
            PageContent { selected: selected(), appearance }
        }
    }
}

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

#[component]
fn PageContent(selected: Category, appearance: Signal<AppearanceSettings>) -> Element {
    rsx! {
        main { class: "content",
            match selected {
                Category::Appearance => rsx! { AppearancePage { settings: appearance } },
                _ => rsx! {
                    h1 { "{selected.label()}" }
                    p { class: "placeholder", "Settings for {selected.label()} coming soon." }
                }
            }
        }
    }
}

// ============================================================================
// Reusable Components
// ============================================================================

#[component]
fn Section(title: &'static str, children: Element) -> Element {
    rsx! {
        section { class: "settings-section",
            h2 { class: "section-title", "{title}" }
            div { class: "section-content", {children} }
        }
    }
}

#[component]
fn ToggleRow(label: &'static str, description: Option<&'static str>, value: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
                if let Some(desc) = description {
                    span { class: "setting-description", "{desc}" }
                }
            }
            button {
                class: if value { "toggle-btn on" } else { "toggle-btn off" },
                onclick: move |_| on_change.call(!value),
                if value { "On" } else { "Off" }
            }
        }
    }
}

#[component]
fn SliderRow(
    label: &'static str,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    unit: &'static str,
    on_change: EventHandler<f32>,
) -> Element {
    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
            }
            div { class: "slider-control",
                button {
                    class: "slider-btn",
                    onclick: move |_| {
                        let new_val = (value - step).max(min);
                        on_change.call(new_val);
                    },
                    "-"
                }
                span { class: "slider-value", "{value:.0}{unit}" }
                button {
                    class: "slider-btn",
                    onclick: move |_| {
                        let new_val = (value + step).min(max);
                        on_change.call(new_val);
                    },
                    "+"
                }
            }
        }
    }
}

// ============================================================================
// Appearance Page
// ============================================================================

#[component]
fn AppearancePage(settings: Signal<AppearanceSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Appearance" }

        Section { title: "Focus Ring",
            ToggleRow {
                label: "Enable focus ring",
                description: Some("Show a colored ring around the focused window"),
                value: s.focus_ring_enabled,
                on_change: move |v| {
                    settings.write().focus_ring_enabled = v;
                }
            }

            if s.focus_ring_enabled {
                SliderRow {
                    label: "Width",
                    value: s.focus_ring_width,
                    min: FOCUS_RING_WIDTH_MIN,
                    max: FOCUS_RING_WIDTH_MAX,
                    step: 1.0,
                    unit: "px",
                    on_change: move |v| {
                        settings.write().focus_ring_width = v;
                    }
                }
            }
        }

        Section { title: "Window Border",
            ToggleRow {
                label: "Enable window border",
                description: Some("Draw a border around windows"),
                value: s.border_enabled,
                on_change: move |v| {
                    settings.write().border_enabled = v;
                }
            }

            if s.border_enabled {
                SliderRow {
                    label: "Thickness",
                    value: s.border_thickness,
                    min: BORDER_THICKNESS_MIN,
                    max: BORDER_THICKNESS_MAX,
                    step: 0.5,
                    unit: "px",
                    on_change: move |v| {
                        settings.write().border_thickness = v;
                    }
                }
            }
        }

        Section { title: "Layout",
            SliderRow {
                label: "Window gaps",
                value: s.gaps,
                min: GAP_SIZE_MIN,
                max: GAP_SIZE_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().gaps = v;
                }
            }

            SliderRow {
                label: "Corner radius",
                value: s.corner_radius,
                min: CORNER_RADIUS_MIN,
                max: CORNER_RADIUS_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().corner_radius = v;
                }
            }
        }
    }
}
