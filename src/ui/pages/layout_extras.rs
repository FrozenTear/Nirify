//! Layout extras settings page (shadows, tab indicator, insert hint)

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::{Color, ColorOrGradient};
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the layout extras settings page
pub fn layout_extras_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let layout = settings.layout_extras;

    let state_shadow_en = state.clone();
    let state_shadow_soft = state.clone();
    let state_shadow_spread = state.clone();
    let state_shadow_ox = state.clone();
    let state_shadow_oy = state.clone();
    let state_shadow_color = state.clone();
    let state_shadow_inactive = state.clone();
    let state_tab_en = state.clone();
    let state_tab_hide = state.clone();
    let state_tab_width = state.clone();
    let state_tab_gap = state.clone();
    let state_hint_en = state.clone();
    let state_hint_color = state.clone();

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
                        state_shadow_en.update_settings(|s| s.layout_extras.shadow.enabled = val);
                        state_shadow_en.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_shadow_soft
                            .update_settings(|s| s.layout_extras.shadow.softness = val as i32);
                        state_shadow_soft.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_shadow_spread
                            .update_settings(|s| s.layout_extras.shadow.spread = val as i32);
                        state_shadow_spread.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_shadow_ox
                            .update_settings(|s| s.layout_extras.shadow.offset_x = val as i32);
                        state_shadow_ox.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_shadow_oy
                            .update_settings(|s| s.layout_extras.shadow.offset_y = val as i32);
                        state_shadow_oy.mark_dirty_and_save(SettingsCategory::LayoutExtras);
                    },
                ))
                .child(text_row(
                    "Active shadow color",
                    "Shadow color for focused windows (hex)",
                    &layout.shadow.color.to_hex(),
                    "#00000080",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state_shadow_color
                                .update_settings(|s| s.layout_extras.shadow.color = color);
                            state_shadow_color.mark_dirty_and_save(SettingsCategory::LayoutExtras);
                        }
                    },
                ))
                .child(text_row(
                    "Inactive shadow color",
                    "Shadow color for unfocused windows (hex)",
                    &layout.shadow.inactive_color.to_hex(),
                    "#00000040",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state_shadow_inactive
                                .update_settings(|s| s.layout_extras.shadow.inactive_color = color);
                            state_shadow_inactive
                                .mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_tab_en.update_settings(|s| s.layout_extras.tab_indicator.enabled = val);
                        state_tab_en.mark_dirty_and_save(SettingsCategory::LayoutExtras);
                    },
                ))
                .child(toggle_row(
                    "Hide when single tab",
                    "Hide indicator when only one tab",
                    layout.tab_indicator.hide_when_single_tab,
                    move |val| {
                        state_tab_hide
                            .update_settings(|s| s.layout_extras.tab_indicator.hide_when_single_tab = val);
                        state_tab_hide.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_tab_width
                            .update_settings(|s| s.layout_extras.tab_indicator.width = val as i32);
                        state_tab_width.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_tab_gap
                            .update_settings(|s| s.layout_extras.tab_indicator.gap = val as i32);
                        state_tab_gap.mark_dirty_and_save(SettingsCategory::LayoutExtras);
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
                        state_hint_en.update_settings(|s| s.layout_extras.insert_hint.enabled = val);
                        state_hint_en.mark_dirty_and_save(SettingsCategory::LayoutExtras);
                    },
                ))
                .child(text_row(
                    "Hint color",
                    "Color of the insertion hint (hex)",
                    &layout.insert_hint.color.to_hex(),
                    "#ffffff40",
                    move |val| {
                        if let Some(color) = Color::from_hex(&val) {
                            state_hint_color.update_settings(|s| {
                                s.layout_extras.insert_hint.color = ColorOrGradient::Color(color)
                            });
                            state_hint_color.mark_dirty_and_save(SettingsCategory::LayoutExtras);
                        }
                    },
                )),
        ))
}
