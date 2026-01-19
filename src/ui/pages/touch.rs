//! Touch screen settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the touch screen settings page
pub fn touch_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let touch = &settings.touch;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Touch, |s| {
                            s.touch.map_to_output = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::Touch, |s| {
                            s.touch.off = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
