//! Tablet (drawing tablet) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the tablet settings page
pub fn tablet_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let tablet = &settings.tablet;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Tablet, |s| {
                            s.tablet.map_to_output = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::Tablet, |s| {
                            s.tablet.left_handed = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable tablet",
                    "Turn off tablet input",
                    tablet.off,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Tablet, |s| {
                            s.tablet.off = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
