//! Appearance settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row, value_row};
use crate::ui::theme::SPACING_LG;

/// Create the appearance settings page
pub fn appearance_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let appearance = &settings.appearance;

    // Clone state for callbacks
    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();
    let state6 = state.clone();
    let mut refresh6 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Focus Ring section
        .child(section(
            "Focus Ring",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable focus ring",
                    "Show a colored ring around the focused window",
                    appearance.focus_ring_enabled,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.focus_ring_enabled = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Ring width",
                    "Thickness of the focus ring in pixels",
                    appearance.focus_ring_width as f64,
                    1.0,
                    20.0,
                    "px",
                    move |val| {
                        state2.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.focus_ring_width = val as f32
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(value_row(
                    "Active color",
                    "Color when window is focused",
                    appearance.focus_ring_active.to_hex(),
                ))
                .child(value_row(
                    "Inactive color",
                    "Color when window is not focused",
                    appearance.focus_ring_inactive.to_hex(),
                )),
        ))
        // Window Border section
        .child(section(
            "Window Border",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable window border",
                    "Show a border around windows (inside the focus ring)",
                    appearance.border_enabled,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.border_enabled = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Border thickness",
                    "Thickness of the window border in pixels",
                    appearance.border_thickness as f64,
                    1.0,
                    15.0,
                    "px",
                    move |val| {
                        state4.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.border_thickness = val as f32
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Gaps section
        .child(section(
            "Gaps",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Window gaps",
                    "Space between windows",
                    appearance.gaps as f64,
                    0.0,
                    64.0,
                    "px",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.gaps = val as f32
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Corners section
        .child(section(
            "Corners",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Corner radius",
                    "Window corner rounding",
                    appearance.corner_radius as f64,
                    0.0,
                    40.0,
                    "px",
                    move |val| {
                        state6.update_and_save(SettingsCategory::Appearance, |s| {
                            s.appearance.corner_radius = val as f32
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
