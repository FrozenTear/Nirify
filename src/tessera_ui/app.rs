//! Main Tessera application entry point

use anyhow::Result;
use std::sync::{Arc, Mutex};
use tessera_ui::{remember, TesseraApp};
use winit::event_loop::EventLoop;

use crate::config::Settings;

/// Run the Tessera application
pub fn run_app(settings: Arc<Mutex<Settings>>) -> Result<()> {
    // Create the event loop
    let event_loop = EventLoop::new()?;

    // TODO: Initialize Tessera window and components
    // This is a placeholder structure - will be implemented with actual Tessera API

    log::info!("Starting Tessera UI...");
    log::warn!("Tessera UI implementation is a work in progress");

    // Placeholder: In a real implementation, we would:
    // 1. Create a TesseraApp instance
    // 2. Set up the main window with settings pages
    // 3. Run the event loop

    Ok(())
}

/// Main application component (placeholder)
/// This will be replaced with actual Tessera component implementation
#[tessera_ui::tessera]
fn app_root(settings: Arc<Mutex<Settings>>) -> impl tessera_ui::Component {
    // Placeholder - will implement actual UI here
    tessera_components::text("niri-settings (Tessera)")
}
