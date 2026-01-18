//! Touch screen settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the touch screen settings page
pub fn touch_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let touch = settings.touch;

    let state_map = state.clone();
    let state_off = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Display Mapping section
        .child(section(
            "Display Mapping",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(text_row(
                    "Map to output",
                    "Monitor name to map touch to (e.g., eDP-1)",
                    &touch.map_to_output,
                    "",
                    move |val| {
                        state_map.update_settings(|s| s.touch.map_to_output = val);
                        state_map.mark_dirty_and_save(SettingsCategory::Touch);
                    },
                )),
        ))
        // Device section
        .child(section(
            "Device",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable touch",
                    "Turn off touch input",
                    touch.off,
                    move |val| {
                        state_off.update_settings(|s| s.touch.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::Touch);
                    },
                )),
        ))
}
