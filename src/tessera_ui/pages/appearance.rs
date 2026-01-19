//! Appearance settings page
//!
//! This page demonstrates Tessera components for testing:
//! - Checkboxes (focus ring toggle)
//! - Sliders (border width, focus ring width)
//! - Radio buttons (for enum selections - workaround until select component available)

use std::sync::{Arc, Mutex};
use tessera_ui::remember;

use crate::config::Settings;
use crate::types::Color;

/// Appearance settings page component
#[tessera_ui::tessera]
pub fn appearance_page(settings: Arc<Mutex<Settings>>) -> impl tessera_ui::Component {
    // TODO: Implement actual appearance settings
    // This is a placeholder showing the structure we'll build

    tessera_components::column((
        tessera_components::text("Appearance Settings"),
        // Focus ring section
        focus_ring_section(settings.clone()),
        // Border section
        border_section(settings.clone()),
    ))
}

/// Focus ring settings section
#[tessera_ui::tessera]
fn focus_ring_section(settings: Arc<Mutex<Settings>>) -> impl tessera_ui::Component {
    // Get current values from settings
    let enabled = {
        let s = settings.lock().unwrap();
        s.appearance.focus_ring.enabled
    };

    let enabled_state = remember(|| enabled);

    tessera_components::column((
        tessera_components::text("Focus Ring"),
        // TODO: Add actual checkbox component when implementing
        tessera_components::text(format!("Enabled: {}", enabled_state.get())),
    ))
}

/// Border settings section
#[tessera_ui::tessera]
fn border_section(settings: Arc<Mutex<Settings>>) -> impl tessera_ui::Component {
    tessera_components::column((
        tessera_components::text("Borders"),
        // TODO: Add sliders for border width
    ))
}
