//! Layout extras settings page (shadows, tab indicator, insert hint)

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::types::{Color, ColorOrGradient};
use crate::ui::components::{
    color_row_with_callback, section, slider_row_with_callback, toggle_row_with_callback,
};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the layout extras settings page
pub fn layout_extras_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let layout = settings.layout_extras;

    // Shadow settings
    let shadow_enabled = RwSignal::new(layout.shadow.enabled);
    let shadow_softness = RwSignal::new(layout.shadow.softness as f64);
    let shadow_spread = RwSignal::new(layout.shadow.spread as f64);
    let shadow_offset_x = RwSignal::new(layout.shadow.offset_x as f64);
    let shadow_offset_y = RwSignal::new(layout.shadow.offset_y as f64);
    let shadow_color = RwSignal::new(layout.shadow.color.to_hex());
    let shadow_inactive_color = RwSignal::new(layout.shadow.inactive_color.to_hex());

    // Tab indicator
    let tab_enabled = RwSignal::new(layout.tab_indicator.enabled);
    let tab_hide_single = RwSignal::new(layout.tab_indicator.hide_when_single_tab);
    let tab_width = RwSignal::new(layout.tab_indicator.width as f64);
    let tab_gap = RwSignal::new(layout.tab_indicator.gap as f64);

    // Insert hint
    let insert_hint_enabled = RwSignal::new(layout.insert_hint.enabled);
    let insert_hint_color = RwSignal::new(layout.insert_hint.color.to_hex());

    // Shadow callbacks
    let on_shadow_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.layout_extras.shadow.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_shadow_softness = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.shadow.softness = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_shadow_spread = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.shadow.spread = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_shadow_offset_x = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.shadow.offset_x = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_shadow_offset_y = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.shadow.offset_y = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_shadow_color = {
        let state = state.clone();
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| s.layout_extras.shadow.color = color);
                state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
            }
        })
    };

    let on_shadow_inactive_color = {
        let state = state.clone();
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| s.layout_extras.shadow.inactive_color = color);
                state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
            }
        })
    };

    // Tab indicator callbacks
    let on_tab_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.layout_extras.tab_indicator.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_tab_hide_single = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.layout_extras.tab_indicator.hide_when_single_tab = val);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_tab_width = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.tab_indicator.width = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_tab_gap = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.layout_extras.tab_indicator.gap = val as i32);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    // Insert hint callbacks
    let on_insert_hint_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.layout_extras.insert_hint.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
        })
    };

    let on_insert_hint_color = {
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| {
                    s.layout_extras.insert_hint.color = ColorOrGradient::Color(color)
                });
                state.mark_dirty_and_save(SettingsCategory::LayoutExtras);
            }
        })
    };

    Stack::vertical((
        section(
            "Window Shadows",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable shadows",
                    Some("Show shadows behind windows"),
                    shadow_enabled,
                    Some(on_shadow_enabled),
                ),
                slider_row_with_callback(
                    "Softness",
                    Some("Shadow blur radius"),
                    shadow_softness,
                    0.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_shadow_softness),
                ),
                slider_row_with_callback(
                    "Spread",
                    Some("Shadow expansion"),
                    shadow_spread,
                    0.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_shadow_spread),
                ),
                slider_row_with_callback(
                    "Offset X",
                    Some("Horizontal shadow offset"),
                    shadow_offset_x,
                    -100.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_shadow_offset_x),
                ),
                slider_row_with_callback(
                    "Offset Y",
                    Some("Vertical shadow offset"),
                    shadow_offset_y,
                    -100.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_shadow_offset_y),
                ),
                color_row_with_callback(
                    "Active shadow color",
                    Some("Shadow color for focused windows"),
                    shadow_color,
                    Some(on_shadow_color),
                ),
                color_row_with_callback(
                    "Inactive shadow color",
                    Some("Shadow color for unfocused windows"),
                    shadow_inactive_color,
                    Some(on_shadow_inactive_color),
                ),
            )),
        ),
        section(
            "Tab Indicator",
            Stack::vertical((
                toggle_row_with_callback(
                    "Show tab indicator",
                    Some("Display indicator for tabbed windows"),
                    tab_enabled,
                    Some(on_tab_enabled),
                ),
                toggle_row_with_callback(
                    "Hide when single tab",
                    Some("Hide indicator when only one tab"),
                    tab_hide_single,
                    Some(on_tab_hide_single),
                ),
                slider_row_with_callback(
                    "Indicator width",
                    Some("Width of the tab indicator"),
                    tab_width,
                    1.0,
                    20.0,
                    1.0,
                    "px",
                    Some(on_tab_width),
                ),
                slider_row_with_callback(
                    "Gap from window",
                    Some("Space between indicator and window"),
                    tab_gap,
                    0.0,
                    40.0,
                    2.0,
                    "px",
                    Some(on_tab_gap),
                ),
            )),
        ),
        section(
            "Insert Hint",
            Stack::vertical((
                toggle_row_with_callback(
                    "Show insert hint",
                    Some("Display hint for window insertion location"),
                    insert_hint_enabled,
                    Some(on_insert_hint_enabled),
                ),
                color_row_with_callback(
                    "Hint color",
                    Some("Color of the insertion hint"),
                    insert_hint_color,
                    Some(on_insert_hint_color),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
