//! Cursor settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, text_row, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the cursor settings page
pub fn cursor_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let cursor = settings.cursor;

    let theme = RwSignal::new(cursor.theme);
    let size = RwSignal::new(cursor.size as f64);
    let hide_when_typing = RwSignal::new(cursor.hide_when_typing);
    let hide_after_inactive = RwSignal::new(cursor.hide_after_inactive_ms.unwrap_or(0) as f64);
    let auto_hide_enabled = RwSignal::new(cursor.hide_after_inactive_ms.is_some());

    // Callbacks
    let on_size = { let state = state.clone(); Rc::new(move |val: f64| {
        state.update_settings(|s| s.cursor.size = val as i32);
        state.mark_dirty_and_save(SettingsCategory::Cursor);
    })};

    let on_hide_when_typing = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.cursor.hide_when_typing = val);
        state.mark_dirty_and_save(SettingsCategory::Cursor);
    })};

    let on_auto_hide_enabled = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| {
            s.cursor.hide_after_inactive_ms = if val { Some(hide_after_inactive.get() as i32) } else { None };
        });
        state.mark_dirty_and_save(SettingsCategory::Cursor);
    })};

    let on_hide_after_inactive = { Rc::new(move |val: f64| {
        if auto_hide_enabled.get() {
            state.update_settings(|s| s.cursor.hide_after_inactive_ms = Some(val as i32));
            state.mark_dirty_and_save(SettingsCategory::Cursor);
        }
    })};

    Stack::vertical((
        section(
            "Appearance",
            Stack::vertical((
                text_row(
                    "Cursor theme",
                    Some("Theme name (empty = system default)"),
                    theme,
                    "Adwaita",
                ),
                slider_row_with_callback(
                    "Cursor size",
                    Some("Size in pixels"),
                    size,
                    16.0,
                    96.0,
                    4.0,
                    "px",
                    Some(on_size),
                ),
            )),
        ),
        section(
            "Behavior",
            Stack::vertical((
                toggle_row_with_callback(
                    "Hide while typing",
                    Some("Hide cursor when using keyboard"),
                    hide_when_typing,
                    Some(on_hide_when_typing),
                ),
                toggle_row_with_callback(
                    "Auto-hide when idle",
                    Some("Hide cursor after inactivity"),
                    auto_hide_enabled,
                    Some(on_auto_hide_enabled),
                ),
                slider_row_with_callback(
                    "Auto-hide delay",
                    Some("Milliseconds before hiding"),
                    hide_after_inactive,
                    500.0,
                    10000.0,
                    500.0,
                    "ms",
                    Some(on_hide_after_inactive),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
