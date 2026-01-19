//! Setting row components for Freya
//!
//! Refined, minimal setting rows with clean visual hierarchy.
//! Each row follows a two-column layout:
//! - Left: Label + optional description
//! - Right: Interactive control (toggle, slider, input, etc.)

use freya::prelude::*;

use crate::ui::theme::*;

// ============================================================================
// Toggle Row - Switch control
// ============================================================================

/// Create a setting row with a toggle switch
pub fn toggle_row(
    title: &str,
    description: &str,
    value: bool,
    mut on_change: impl FnMut(bool) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            Switch::new()
                .toggled(value)
                .on_toggle(move |_| {
                    on_change(!value);
                }),
        )
        .into()
}

// ============================================================================
// Slider Row - Slider with value display
// ============================================================================

/// Create a setting row with a slider control
pub fn slider_row(
    title: &str,
    description: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    mut on_change: impl FnMut(f64) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();
    let unit = unit.to_string();

    // Normalize value to 0-100 range for slider
    let to_slider = move |v: f64| ((v - min) / (max - min)) * 100.0;
    let from_slider = move |v: f64| (v / 100.0) * (max - min) + min;

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            // Slider control
            Slider::new(move |v| {
                let actual = from_slider(v);
                on_change(actual);
            })
            .value(to_slider(value))
            .size(Size::px(120.0)),
        )
        .child(
            // Value display - monospace style
            rect()
                .width(Size::px(48.0))
                .main_align(Alignment::End)
                .child(
                    label()
                        .text(format!("{:.0}{}", value, unit))
                        .color(ACCENT_VIVID)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .into()
}

// ============================================================================
// Text Row - Text input field
// ============================================================================

/// Create a setting row with a text input
pub fn text_row(
    title: &str,
    description: &str,
    value: &str,
    placeholder: &str,
    mut on_change: impl FnMut(String) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();
    let value = value.to_string();
    let placeholder = placeholder.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            Input::new()
                .value(value)
                .placeholder(placeholder)
                .width(Size::px(160.0))
                .on_change(move |v: String| {
                    on_change(v);
                }),
        )
        .into()
}

// ============================================================================
// Display-only rows (read-only values)
// ============================================================================

/// A simple toggle row that displays a value (no interactivity)
pub fn toggle_row_display(title: &str, description: &str, value: bool) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value_text = if value { "On" } else { "Off" };
    let value_color = if value { ACCENT_VIVID } else { TEXT_DIM };

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value_text)
                .color(value_color)
                .font_size(FONT_SIZE_SM)
                .max_lines(1),
        )
}

/// A slider row that displays a value (no interactivity)
pub fn slider_row_display(title: &str, description: &str, value: f64, unit: &str) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value_text = format!("{:.0}{}", value, unit);

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value_text)
                .color(ACCENT_VIVID)
                .font_size(FONT_SIZE_SM)
                .max_lines(1),
        )
}

/// A text row that displays a value (no interactivity)
pub fn text_row_display(title: &str, description: &str, value: &str) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value = value.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value)
                .color(ACCENT_VIVID)
                .font_size(FONT_SIZE_SM)
                .max_lines(1),
        )
}

/// A simple row displaying a label and value (read-only)
pub fn value_row(title: &str, description: &str, value: impl ToString) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value = value.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG)
        .child(
            rect()
                .width(Size::flex(1.0))
                .spacing(SPACING_2XS)
                .child(
                    label()
                        .text(title)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value)
                .color(ACCENT_VIVID)
                .font_size(FONT_SIZE_SM)
                .max_lines(1),
        )
}
