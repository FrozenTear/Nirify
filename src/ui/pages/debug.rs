//! Debug settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Label, Stack};
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY, WARNING};

/// Create the debug settings page
pub fn debug_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let debug = settings.debug;

    let preview_render = RwSignal::new(debug.preview_render);
    let disable_cursor_plane = RwSignal::new(debug.disable_cursor_plane);
    let disable_direct_scanout = RwSignal::new(debug.disable_direct_scanout);
    let disable_resize_throttling = RwSignal::new(debug.disable_resize_throttling);
    let disable_transactions = RwSignal::new(debug.disable_transactions);

    // Callbacks
    let on_preview_render = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.debug.preview_render = val);
        state.mark_dirty_and_save(SettingsCategory::Debug);
    })};

    let on_disable_cursor_plane = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.debug.disable_cursor_plane = val);
        state.mark_dirty_and_save(SettingsCategory::Debug);
    })};

    let on_disable_direct_scanout = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.debug.disable_direct_scanout = val);
        state.mark_dirty_and_save(SettingsCategory::Debug);
    })};

    let on_disable_resize_throttling = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.debug.disable_resize_throttling = val);
        state.mark_dirty_and_save(SettingsCategory::Debug);
    })};

    let on_disable_transactions = { Rc::new(move |val: bool| {
        state.update_settings(|s| s.debug.disable_transactions = val);
        state.mark_dirty_and_save(SettingsCategory::Debug);
    })};

    Stack::vertical((
        section(
            "Warning",
            Stack::vertical((Label::derived(|| {
                "These settings are for debugging and troubleshooting. \
                 Changing them may cause visual glitches or performance issues."
                    .to_string()
            })
            .style(|s| s.color(WARNING)),)),
        ),
        section(
            "Rendering",
            Stack::vertical((
                toggle_row_with_callback("Preview render", Some("Render monitors like screencast"), preview_render, Some(on_preview_render)),
                toggle_row_with_callback("Disable cursor plane", Some("Force cursor through compositor"), disable_cursor_plane, Some(on_disable_cursor_plane)),
                toggle_row_with_callback("Disable direct scanout", Some("Always composite fullscreen windows"), disable_direct_scanout, Some(on_disable_direct_scanout)),
            )),
        ),
        section(
            "Window Management",
            Stack::vertical((
                toggle_row_with_callback("Disable resize throttling", Some("Send resize events immediately"), disable_resize_throttling, Some(on_disable_resize_throttling)),
                toggle_row_with_callback("Disable transactions", Some("Don't wait for synchronized resizing"), disable_transactions, Some(on_disable_transactions)),
            )),
        ),
        section(
            "Info",
            Stack::vertical((Label::derived(|| {
                "Additional debug settings are available in the config file. \
                 See the niri documentation for advanced debugging options."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
