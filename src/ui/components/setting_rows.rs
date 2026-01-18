//! Setting row components for consistent UI across all settings pages

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{empty, text_input, Button, Checkbox, Container, Label, Stack};

use crate::ui::theme::{
    setting_row_style, BG_ELEVATED, BORDER, SPACING_MD, SPACING_SM, TEXT_PRIMARY, TEXT_SECONDARY,
};

/// A row with a toggle switch
pub fn toggle_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<bool>,
) -> impl IntoView {
    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT_PRIMARY)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(TEXT_SECONDARY))
                    .into_any(),
                None => empty().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        Checkbox::new_rw(value).style(|s| s.margin_left(SPACING_MD)),
    ))
    .style(setting_row_style)
}

/// A row with a slider and value display
pub fn slider_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<f64>,
    min: f64,
    max: f64,
    step: f64,
) -> impl IntoView {
    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT_PRIMARY)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(TEXT_SECONDARY))
                    .into_any(),
                None => empty().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0).min_width(150.0)),
        Stack::horizontal((
            // Decrease button
            Button::new("-")
                .style(|s| {
                    s.padding_horiz(SPACING_SM)
                        .padding_vert(2.0)
                        .background(BG_ELEVATED)
                        .border_radius(4.0)
                })
                .action(move || {
                    let current = value.get();
                    let new_val = (current - step).max(min);
                    value.set(new_val);
                }),
            // Value display (instead of slider for now)
            Label::derived(move || format!("{:.1}", value.get()))
                .style(|s| {
                    s.min_width(60.0)
                        .padding_horiz(SPACING_MD)
                        .color(TEXT_SECONDARY)
                }),
            // Increase button
            Button::new("+")
                .style(|s| {
                    s.padding_horiz(SPACING_SM)
                        .padding_vert(2.0)
                        .background(BG_ELEVATED)
                        .border_radius(4.0)
                })
                .action(move || {
                    let current = value.get();
                    let new_val = (current + step).min(max);
                    value.set(new_val);
                }),
        )),
    ))
    .style(setting_row_style)
}

/// A row with a text input
pub fn text_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
    _placeholder: &'static str,
) -> impl IntoView {
    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT_PRIMARY)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(TEXT_SECONDARY))
                    .into_any(),
                None => empty().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        text_input(value).style(|s| {
            s.width(200.0)
                .padding(SPACING_SM)
                .background(BG_ELEVATED)
                .border_radius(4.0)
                .border(1.0)
                .border_color(BORDER)
        }),
    ))
    .style(setting_row_style)
}

/// A row with a dropdown/select
pub fn dropdown_row<T: Clone + PartialEq + 'static>(
    label_text: &'static str,
    description: Option<&'static str>,
    options: Vec<(&'static str, T)>,
    selected: RwSignal<T>,
) -> impl IntoView {
    let options_for_label = options.clone();

    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT_PRIMARY)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(TEXT_SECONDARY))
                    .into_any(),
                None => empty().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        // Simple label display for now - can be enhanced with proper dropdown widget
        Label::derived(move || {
            let current = selected.get();
            options_for_label
                .iter()
                .find(|(_, v)| *v == current)
                .map(|(name, _)| (*name).to_string())
                .unwrap_or_else(|| "Select...".to_string())
        })
        .style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .background(BG_ELEVATED)
                .border_radius(4.0)
                .border(1.0)
                .border_color(BORDER)
                .min_width(150.0)
        }),
    ))
    .style(setting_row_style)
}

/// A row with a color picker
pub fn color_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
) -> impl IntoView {
    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT_PRIMARY)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(TEXT_SECONDARY))
                    .into_any(),
                None => empty().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        Stack::horizontal((
            // Color preview box
            Container::new(empty())
                .style(move |s| {
                    // Parse the hex color for preview
                    let hex = value.get();
                    let color = parse_hex_color(&hex);
                    s.width(32.0)
                        .height(32.0)
                        .border_radius(4.0)
                        .background(color)
                        .border(1.0)
                        .border_color(BORDER)
                }),
            // Hex input
            text_input(value).style(|s| {
                s.width(100.0)
                    .padding(SPACING_SM)
                    .background(BG_ELEVATED)
                    .border_radius(4.0)
                    .border(1.0)
                    .border_color(BORDER)
                    .margin_left(SPACING_SM)
            }),
        )),
    ))
    .style(setting_row_style)
}

/// Parse a hex color string into a Floem Color
fn parse_hex_color(hex: &str) -> floem::peniko::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        floem::peniko::Color::from_rgb8(r, g, b)
    } else {
        floem::peniko::Color::from_rgb8(0, 0, 0)
    }
}
