//! Setting row components matching the reference design
//!
//! Each row has: Label + Description on left, Control on right

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Button, Checkbox, Container, Empty, Label, Stack};

use crate::ui::theme::{
    setting_row_style, BORDER_RADIUS_SM, OVERLAY0, SPACING_MD, SPACING_SM, SPACING_XS, SUBTEXT0,
    SUBTEXT1, SURFACE0, SURFACE1, TEXT,
};

/// A row with a toggle switch
pub fn toggle_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<bool>,
) -> impl IntoView {
    Stack::horizontal((
        // Left side: label + description
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(SUBTEXT0))
                    .into_any(),
                None => Empty::new().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        // Right side: toggle
        Checkbox::new_rw(value).style(|s| s.margin_left(SPACING_MD)),
    ))
    .style(setting_row_style)
}

/// A row with a slider showing value (e.g., "4px")
pub fn slider_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<f64>,
    min: f64,
    max: f64,
    step: f64,
    unit: &'static str,
) -> impl IntoView {
    Stack::horizontal((
        // Left side: label + description
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(SUBTEXT0))
                    .into_any(),
                None => Empty::new().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        // Right side: slider circle + value
        Stack::horizontal((
            // Simple slider representation (a styled button for now)
            Button::new("●")
                .style(|s| s.color(OVERLAY0).padding(SPACING_XS))
                .action(move || {
                    // Cycle through some values for demo
                    let current = value.get();
                    let new_val = if current + step <= max {
                        current + step
                    } else {
                        min
                    };
                    value.set(new_val);
                }),
            // Value display
            Label::derived(move || format!("{}{}", value.get() as i32, unit))
                .style(|s| s.color(SUBTEXT1).min_width(40.0)),
        ))
        .style(|s| s.items_center().gap(SPACING_SM)),
    ))
    .style(setting_row_style)
}

/// A row with color swatch + hex input
pub fn color_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
) -> impl IntoView {
    Stack::horizontal((
        // Left side: label + description
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(SUBTEXT0))
                    .into_any(),
                None => Empty::new().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        // Right side: color swatch + hex input
        Stack::horizontal((
            // Color swatch
            Container::new(Empty::new()).style(move |s| {
                let hex = value.get();
                let color = parse_hex_color(&hex);
                s.width(24.0)
                    .height(24.0)
                    .border_radius(BORDER_RADIUS_SM)
                    .background(color)
            }),
            // Hex input with clear button
            Stack::horizontal((
                text_input(value).style(|s| {
                    s.width(80.0)
                        .padding(SPACING_XS)
                        .background(SURFACE0)
                        .border_radius(BORDER_RADIUS_SM)
                        .color(TEXT)
                        .font_size(12.0)
                }),
                // Clear/reset button
                Button::new("✕").style(|s| s.color(OVERLAY0).padding(SPACING_XS).font_size(10.0)),
            ))
            .style(|s| {
                s.background(SURFACE1)
                    .border_radius(BORDER_RADIUS_SM)
                    .padding_left(SPACING_SM)
                    .items_center()
            }),
        ))
        .style(|s| s.items_center().gap(SPACING_SM)),
    ))
    .style(setting_row_style)
}

/// A row with a text input
pub fn text_row(
    label_text: &'static str,
    description: Option<&'static str>,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(SUBTEXT0))
                    .into_any(),
                None => Empty::new().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
        text_input(value)
            .placeholder(placeholder)
            .style(|s| {
                s.width(200.0)
                    .padding(SPACING_SM)
                    .background(SURFACE0)
                    .border_radius(BORDER_RADIUS_SM)
                    .color(TEXT)
            }),
    ))
    .style(setting_row_style)
}

/// A row with a dropdown (simplified as label for now)
pub fn dropdown_row<T: Clone + PartialEq + 'static>(
    label_text: &'static str,
    description: Option<&'static str>,
    options: Vec<(&'static str, T)>,
    selected: RwSignal<T>,
) -> impl IntoView {
    let options_for_label = options.clone();

    Stack::horizontal((
        Stack::vertical((
            Label::derived(move || label_text.to_string()).style(|s| s.color(TEXT)),
            match description {
                Some(desc) => Label::derived(move || desc.to_string())
                    .style(|s| s.font_size(12.0).color(SUBTEXT0))
                    .into_any(),
                None => Empty::new().into_any(),
            },
        ))
        .style(|s| s.flex_grow(1.0)),
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
                .background(SURFACE1)
                .border_radius(BORDER_RADIUS_SM)
                .min_width(150.0)
                .color(TEXT)
        }),
    ))
    .style(setting_row_style)
}

/// Parse a hex color string into a Floem Color
fn parse_hex_color(hex: &str) -> floem::peniko::Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
        floem::peniko::Color::from_rgb8(r, g, b)
    } else {
        floem::peniko::Color::from_rgb8(128, 128, 128)
    }
}
