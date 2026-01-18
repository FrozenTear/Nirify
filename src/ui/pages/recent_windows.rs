//! Recent windows (Alt-Tab) settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the recent windows settings page
pub fn recent_windows_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let recent = settings.recent_windows;

    let off = RwSignal::new(recent.off);
    let debounce_ms = RwSignal::new(recent.debounce_ms as f64);
    let open_delay_ms = RwSignal::new(recent.open_delay_ms as f64);

    // Callbacks
    let on_off = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.recent_windows.off = val);
        state.mark_dirty_and_save(SettingsCategory::RecentWindows);
    })};

    let on_debounce_ms = { let state = state.clone(); Rc::new(move |val: f64| {
        state.update_settings(|s| s.recent_windows.debounce_ms = val as i32);
        state.mark_dirty_and_save(SettingsCategory::RecentWindows);
    })};

    let on_open_delay_ms = { Rc::new(move |val: f64| {
        state.update_settings(|s| s.recent_windows.open_delay_ms = val as i32);
        state.mark_dirty_and_save(SettingsCategory::RecentWindows);
    })};

    Stack::vertical((
        section(
            "General",
            Stack::vertical((
                toggle_row_with_callback("Disable recent windows", Some("Turn off the Alt-Tab switcher"), off, Some(on_off)),
            )),
        ),
        section(
            "Timing",
            Stack::vertical((
                slider_row_with_callback("Debounce delay", Some("Delay before adding to recent list"), debounce_ms, 0.0, 1000.0, 50.0, "ms", Some(on_debounce_ms)),
                slider_row_with_callback("Open delay", Some("Delay before UI appears"), open_delay_ms, 0.0, 500.0, 25.0, "ms", Some(on_open_delay_ms)),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
