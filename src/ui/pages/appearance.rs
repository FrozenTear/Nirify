//! Appearance settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Container, Empty, Label, Stack};
use std::rc::Rc;

use crate::config::{save_preferences, AppPreferences, SettingsCategory};
use crate::types::{Color, ColorOrGradient};
use crate::ui::components::{
    color_row_with_callback, section, slider_row_with_callback, toggle_row_with_callback,
};
use crate::ui::state::AppState;
use crate::ui::theme::{
    preset_signal, set_theme, theme, ThemePreset, FONT_SIZE_SM, FONT_SIZE_XS, RADIUS_LG, RADIUS_MD,
    SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
};

/// Create the appearance settings page
pub fn appearance_page(state: AppState) -> impl IntoView {
    // Clone paths early before state is moved into closures
    let paths = state.paths.clone();

    // Create local signals from settings
    let settings = state.get_settings();
    let appearance = settings.appearance;

    let focus_ring_enabled = RwSignal::new(appearance.focus_ring_enabled);
    let focus_ring_width = RwSignal::new(appearance.focus_ring_width as f64);
    let focus_ring_active = RwSignal::new(appearance.focus_ring_active.to_hex());
    let focus_ring_inactive = RwSignal::new(appearance.focus_ring_inactive.to_hex());
    let focus_ring_urgent = RwSignal::new(appearance.focus_ring_urgent.to_hex());

    let border_enabled = RwSignal::new(appearance.border_enabled);
    let border_thickness = RwSignal::new(appearance.border_thickness as f64);

    let gaps = RwSignal::new(appearance.gaps as f64);
    let corner_radius = RwSignal::new(appearance.corner_radius as f64);

    // Create callbacks that update settings and save
    let on_focus_ring_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.appearance.focus_ring_enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    let on_focus_ring_width = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.appearance.focus_ring_width = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    let on_focus_ring_active = {
        let state = state.clone();
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| {
                    s.appearance.focus_ring_active = ColorOrGradient::Color(color)
                });
                state.mark_dirty_and_save(SettingsCategory::Appearance);
            }
        })
    };

    let on_focus_ring_inactive = {
        let state = state.clone();
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| {
                    s.appearance.focus_ring_inactive = ColorOrGradient::Color(color)
                });
                state.mark_dirty_and_save(SettingsCategory::Appearance);
            }
        })
    };

    let on_focus_ring_urgent = {
        let state = state.clone();
        Rc::new(move |val: String| {
            if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| {
                    s.appearance.focus_ring_urgent = ColorOrGradient::Color(color)
                });
                state.mark_dirty_and_save(SettingsCategory::Appearance);
            }
        })
    };

    let on_border_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.appearance.border_enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    let on_border_thickness = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.appearance.border_thickness = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    let on_gaps = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.appearance.gaps = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    let on_corner_radius = {
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.appearance.corner_radius = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Appearance);
        })
    };

    Stack::vertical((
        // Theme selector section
        section("App Theme", theme_selector(paths)),
        // Focus Ring section
        section(
            "Focus Ring",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable focus ring",
                    Some("Show a colored ring around the focused window"),
                    focus_ring_enabled,
                    Some(on_focus_ring_enabled),
                ),
                slider_row_with_callback(
                    "Ring width",
                    Some("Thickness of the focus ring in pixels"),
                    focus_ring_width,
                    1.0,
                    20.0,
                    1.0,
                    "px",
                    Some(on_focus_ring_width),
                ),
                color_row_with_callback(
                    "Active color",
                    Some("Color when window is focused"),
                    focus_ring_active,
                    Some(on_focus_ring_active),
                ),
                color_row_with_callback(
                    "Inactive color",
                    Some("Color when window is not focused"),
                    focus_ring_inactive,
                    Some(on_focus_ring_inactive),
                ),
                color_row_with_callback(
                    "Urgent color",
                    Some("Color when window needs attention"),
                    focus_ring_urgent,
                    Some(on_focus_ring_urgent),
                ),
            )),
        ),
        // Window Border section
        section(
            "Window Border",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable window border",
                    Some("Show a border around windows (inside the focus ring)"),
                    border_enabled,
                    Some(on_border_enabled),
                ),
                slider_row_with_callback(
                    "Border thickness",
                    Some("Thickness of the window border in pixels"),
                    border_thickness,
                    1.0,
                    15.0,
                    1.0,
                    "px",
                    Some(on_border_thickness),
                ),
            )),
        ),
        // Gaps section
        section(
            "Gaps",
            Stack::vertical((slider_row_with_callback(
                "Window gaps",
                Some("Space between windows"),
                gaps,
                0.0,
                64.0,
                2.0,
                "px",
                Some(on_gaps),
            ),)),
        ),
        // Corners section
        section(
            "Corners",
            Stack::vertical((slider_row_with_callback(
                "Corner radius",
                Some("Window corner rounding"),
                corner_radius,
                0.0,
                40.0,
                2.0,
                "px",
                Some(on_corner_radius),
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// Theme selector component showing all available themes
fn theme_selector(paths: std::sync::Arc<crate::config::ConfigPaths>) -> impl IntoView {
    let current = preset_signal();

    Stack::vertical((
        // Description
        Label::new("Choose a color theme for the settings app")
            .style(move |s| s.font_size(FONT_SIZE_SM).color(theme().text_secondary).margin_bottom(SPACING_MD)),
        // Theme options grid
        Stack::horizontal(
            ThemePreset::all()
                .iter()
                .map(|preset| theme_option_card(*preset, current, paths.clone()))
                .collect::<Vec<_>>(),
        )
        .style(|s| s.gap(SPACING_MD).flex_wrap(floem::style::FlexWrap::Wrap)),
    ))
    .style(|s| s.width_full())
}

/// Individual theme option card
fn theme_option_card(
    preset: ThemePreset,
    current: RwSignal<ThemePreset>,
    paths: std::sync::Arc<crate::config::ConfigPaths>,
) -> impl IntoView {
    let is_selected = move || current.get() == preset;
    let preview_theme = preset.to_theme();

    Container::new(
        Stack::vertical((
            // Color preview bar showing theme colors
            Stack::horizontal((
                // Background color preview
                Empty::new().style(move |s| {
                    s.width(24.0)
                        .height(24.0)
                        .border_radius(RADIUS_MD)
                        .background(preview_theme.bg_base)
                }),
                // Accent color preview
                Empty::new().style(move |s| {
                    s.width(24.0)
                        .height(24.0)
                        .border_radius(RADIUS_MD)
                        .background(preview_theme.accent)
                }),
                // Text color preview
                Empty::new().style(move |s| {
                    s.width(24.0)
                        .height(24.0)
                        .border_radius(RADIUS_MD)
                        .background(preview_theme.text_primary)
                }),
            ))
            .style(|s| s.gap(SPACING_XS).margin_bottom(SPACING_SM)),
            // Theme name
            Label::new(preset.name()).style(move |s| {
                s.font_size(FONT_SIZE_SM)
                    .font_bold()
                    .color(theme().text_primary)
            }),
            // Theme description
            Label::new(preset.description()).style(move |s| {
                s.font_size(FONT_SIZE_XS)
                    .color(theme().text_tertiary)
            }),
        ))
        .style(|s| s.items_start()),
    )
    .style(move |s| {
        let t = theme();
        let base = s
            .padding(SPACING_MD)
            .border_radius(RADIUS_LG)
            .min_width(160.0)
            .border(2.0)
            .cursor(floem::style::CursorStyle::Pointer);

        if is_selected() {
            base.background(t.bg_elevated)
                .border_color(t.accent)
        } else {
            base.background(t.bg_surface)
                .border_color(t.border_subtle)
        }
    })
    .on_click_stop(move |_| {
        // Apply the theme
        set_theme(preset);

        // Save preference to disk
        let prefs = AppPreferences { theme: preset };
        if let Err(e) = save_preferences(&paths.preferences_json, &prefs) {
            log::warn!("Failed to save theme preference: {}", e);
        } else {
            log::info!("Saved theme preference: {:?}", preset);
        }
    })
}
