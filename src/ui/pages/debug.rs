//! Debug settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Label, Stack};

use crate::ui::components::{section, toggle_row};
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
                toggle_row(
                    "Preview render",
                    Some("Render monitors like screencast"),
                    preview_render,
                ),
                toggle_row(
                    "Disable cursor plane",
                    Some("Force cursor through compositor"),
                    disable_cursor_plane,
                ),
                toggle_row(
                    "Disable direct scanout",
                    Some("Always composite fullscreen windows"),
                    disable_direct_scanout,
                ),
            )),
        ),
        section(
            "Window Management",
            Stack::vertical((
                toggle_row(
                    "Disable resize throttling",
                    Some("Send resize events immediately"),
                    disable_resize_throttling,
                ),
                toggle_row(
                    "Disable transactions",
                    Some("Don't wait for synchronized resizing"),
                    disable_transactions,
                ),
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
