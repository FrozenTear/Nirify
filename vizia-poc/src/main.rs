//! Niri Settings - Vizia Proof of Concept
//!
//! This is a minimal PoC to demonstrate Vizia UI framework for the niri-settings app.
//! The goal is to measure compile times and validate the Lens + Event pattern.

mod app_state;
mod constants;
mod types;
mod ui;

use vizia::prelude::*;
use app_state::{AppState, Panel};
use ui::*;

const DARK_THEME: &str = r#"
* {
    font-family: "Inter", sans-serif;
    font-size: 14;
}

.sidebar {
    background-color: #1e1e2e;
    border-right: 1px solid #313244;
}

.sidebar-title {
    color: #cdd6f4;
    child-top: 16px;
    child-left: 16px;
    child-bottom: 16px;
}

.nav-item {
    background-color: transparent;
    color: #a6adc8;
    child-left: 16px;
    child-right: 16px;
    child-top: 8px;
    child-bottom: 8px;
    border-radius: 4px;
}

.nav-item:hover {
    background-color: #313244;
}

.nav-item:checked {
    background-color: #45475a;
    color: #cdd6f4;
}

.spacer {
    height: 1s;
}

.theme-toggle {
    child-left: 16px;
    child-right: 16px;
    child-bottom: 16px;
    color: #cdd6f4;
}

.status-bar {
    child-left: 16px;
    child-right: 16px;
    child-bottom: 8px;
    color: #89b4fa;
    font-size: 12;
}

.page-content {
    background-color: #181825;
    child-left: 24px;
    child-right: 24px;
    child-top: 24px;
    child-bottom: 24px;
}

.page-title {
    color: #cdd6f4;
    child-bottom: 24px;
}

.setting-row {
    child-bottom: 12px;
    border-bottom: 1px solid #313244;
}

.setting-label {
    color: #cdd6f4;
}

.save-button {
    background-color: #89b4fa;
    color: #1e1e2e;
    width: 150px;
    height: 36px;
    child-top: 24px;
    border-radius: 6px;
}

.save-button:hover {
    background-color: #74c7ec;
}

.save-button:disabled {
    background-color: #45475a;
    color: #6c7086;
}
"#;

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting niri-settings Vizia PoC...");

    // Load settings (in PoC, just uses defaults)
    let state = AppState::load().unwrap_or_else(|e| {
        log::error!("Failed to load settings: {}", e);
        AppState::default()
    });

    log::info!("Settings loaded successfully");

    // Create application
    Application::new(|cx| {
        // Apply dark theme
        cx.add_stylesheet(DARK_THEME)
            .expect("Failed to load theme");

        // Build state into context
        state.build(cx);

        // Main layout: sidebar + content area
        HStack::new(cx, |cx| {
            // Sidebar navigation
            build_sidebar(cx);

            // Content area - switches based on current_panel
            Binding::new(cx, AppState::current_panel, |cx, panel| {
                let panel = panel.get(cx);
                match panel {
                    Panel::Keyboard => build_keyboard_page(cx),
                    Panel::Mouse => build_mouse_page(cx),
                    Panel::Touchpad => build_touchpad_page(cx),
                }
            });
        })
        .background_color(Color::rgb(24, 24, 37));
    })
    .title("Niri Settings (Vizia PoC)")
    .inner_size((900, 600))
    .run();

    log::info!("Application closed");
}
"#;
