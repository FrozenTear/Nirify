//! Tablet (drawing tablet) settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, text_row, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the tablet settings page
pub fn tablet_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let tablet = settings.tablet;

    let off = RwSignal::new(tablet.off);
    let left_handed = RwSignal::new(tablet.left_handed);
    let map_to_output = RwSignal::new(tablet.map_to_output);

    // Callbacks
    let on_left_handed = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.tablet.left_handed = val);
            state.mark_dirty_and_save(SettingsCategory::Tablet);
        })
    };

    let on_off = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.tablet.off = val);
            state.mark_dirty_and_save(SettingsCategory::Tablet);
        })
    };

    Stack::vertical((
        section(
            "Display Mapping",
            Stack::vertical((text_row(
                "Map to output",
                Some("Monitor name to map tablet to (e.g., HDMI-1)"),
                map_to_output,
                "",
            ),)),
        ),
        section(
            "Options",
            Stack::vertical((
                toggle_row_with_callback(
                    "Left-handed mode",
                    Some("Flip tablet orientation"),
                    left_handed,
                    Some(on_left_handed),
                ),
                toggle_row_with_callback(
                    "Disable tablet",
                    Some("Turn off tablet input"),
                    off,
                    Some(on_off),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
