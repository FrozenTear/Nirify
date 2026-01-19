//! Layout extras settings page (shadows, tab indicator, insert hint)

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::{Color, ColorOrGradient};
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the layout extras settings page
pub fn layout_extras_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let layout = &settings.layout_extras;

    let shadow_color_hex = layout.shadow.color.to_hex();
    let shadow_inactive_hex = layout.shadow.inactive_color.to_hex();
    let hint_color_hex = layout.insert_hint.color.to_hex();

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
    let state7 = state.clone();
    let mut refresh7 = state.refresh.clone();
    let state8 = state.clone();
    let mut refresh8 = state.refresh.clone();
    let state9 = state.clone();
    let mut refresh9 = state.refresh.clone();
    let state10 = state.clone();
    let mut refresh10 = state.refresh.clone();
    let state11 = state.clone();
    let mut refresh11 = state.refresh.clone();
    let state12 = state.clone();
    let mut refresh12 = state.refresh.clone();
    let state13 = state.clone();
    let mut refresh13 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Window Shadows section
        .child(section(
            "Window Shadows",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable shadows",
                    "Show shadows behind windows",
                    layout.shadow.enabled,
                    move |val| {
                        state1.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.shadow.enabled = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Softness",
                    "Shadow blur radius",
                    layout.shadow.softness as f64,
                    0.0,
                    100.0,
                    "px",
                    move |val| {
                        state2.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.shadow.softness = val as i32
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Spread",
                    "Shadow expansion",
                    layout.shadow.spread as f64,
                    0.0,
                    100.0,
                    "px",
                    move |val| {
                        state3.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.shadow.spread = val as i32
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Offset X",
                    "Horizontal shadow offset",
                    layout.shadow.offset_x as f64,
                    -100.0,
                    100.0,
                    "px",
                    move |val| {
                        state4.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.shadow.offset_x = val as i32
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Offset Y",
                    "Vertical shadow offset",
                    layout.shadow.offset_y as f64,
                    -100.0,
                    100.0,
                    "px",
                    move |val| {
                        state5.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.shadow.offset_y = val as i32
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(text_row(
                    "Active shadow color",
                    "Shadow color for focused windows (hex)",
                    &shadow_color_hex,
                    "#00000080",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state6.update_and_save(SettingsCategory::LayoutExtras, |s| {
                                s.layout_extras.shadow.color = color
                            });
                            refresh6.with_mut(|mut v| *v += 1);
                        }
                    },
                ))
                .child(text_row(
                    "Inactive shadow color",
                    "Shadow color for unfocused windows (hex)",
                    &shadow_inactive_hex,
                    "#00000040",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state7.update_and_save(SettingsCategory::LayoutExtras, |s| {
                                s.layout_extras.shadow.inactive_color = color
                            });
                            refresh7.with_mut(|mut v| *v += 1);
                        }
                    },
                )),
        ))
        // Tab Indicator section
        .child(section(
            "Tab Indicator",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Show tab indicator",
                    "Display indicator for tabbed windows",
                    layout.tab_indicator.enabled,
                    move |val| {
                        state8.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.tab_indicator.enabled = val
                        });
                        refresh8.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Hide when single tab",
                    "Hide indicator when only one tab",
                    layout.tab_indicator.hide_when_single_tab,
                    move |val| {
                        state9.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.tab_indicator.hide_when_single_tab = val
                        });
                        refresh9.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Indicator width",
                    "Width of the tab indicator",
                    layout.tab_indicator.width as f64,
                    1.0,
                    20.0,
                    "px",
                    move |val| {
                        state10.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.tab_indicator.width = val as i32
                        });
                        refresh10.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Gap from window",
                    "Space between indicator and window",
                    layout.tab_indicator.gap as f64,
                    0.0,
                    40.0,
                    "px",
                    move |val| {
                        state11.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.tab_indicator.gap = val as i32
                        });
                        refresh11.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Insert Hint section
        .child(section(
            "Insert Hint",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Show insert hint",
                    "Display hint for window insertion location",
                    layout.insert_hint.enabled,
                    move |val| {
                        state12.update_and_save(SettingsCategory::LayoutExtras, |s| {
                            s.layout_extras.insert_hint.enabled = val
                        });
                        refresh12.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(text_row(
                    "Hint color",
                    "Color of the insertion hint (hex)",
                    &hint_color_hex,
                    "#ffffff40",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state13.update_and_save(SettingsCategory::LayoutExtras, |s| {
                                s.layout_extras.insert_hint.color = ColorOrGradient::Color(color)
                            });
                            refresh13.with_mut(|mut v| *v += 1);
                        }
                    },
                )),
        ))
}
