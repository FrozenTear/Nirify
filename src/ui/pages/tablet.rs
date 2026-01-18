//! Tablet (drawing tablet) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the tablet settings page
pub fn tablet_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let tablet = settings.tablet;

    let state_map = state.clone();
    let state_left = state.clone();
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
                    "Monitor name to map tablet to (e.g., HDMI-1)",
                    &tablet.map_to_output,
                    "",
                    move |val| {
                        state_map.update_settings(|s| s.tablet.map_to_output = val);
                        state_map.mark_dirty_and_save(SettingsCategory::Tablet);
                    },
                )),
        ))
        // Options section
        .child(section(
            "Options",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Left-handed mode",
                    "Flip tablet orientation",
                    tablet.left_handed,
                    move |val| {
                        state_left.update_settings(|s| s.tablet.left_handed = val);
                        state_left.mark_dirty_and_save(SettingsCategory::Tablet);
                    },
                ))
                .child(toggle_row(
                    "Disable tablet",
                    "Turn off tablet input",
                    tablet.off,
                    move |val| {
                        state_off.update_settings(|s| s.tablet.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::Tablet);
                    },
                )),
        ))
}
