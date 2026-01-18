//! Setting row components for Freya
//!
//! Each row follows a two-column layout:
//! - Left: Label + optional description
//! - Right: Interactive control (toggle, slider, input, etc.)
//!
//! These are plain functions that return Elements directly - no Component trait,
//! no hooks. The parent manages state and passes value + callback.

use freya::prelude::*;

use crate::ui::theme::*;

// ============================================================================
// Toggle Row - Switch control (plain function, no hooks)
// ============================================================================

/// Create a setting row with a toggle switch.
/// Parent manages state and passes value + on_change callback.
pub fn toggle_row(
    title: &str,
    description: &str,
    value: bool,
    on_change: impl Fn(bool) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
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
// Slider Row - Slider with value display (plain function, no hooks)
// ============================================================================

/// Create a setting row with a slider control.
/// Parent manages state and passes value + on_change callback.
pub fn slider_row(
    title: &str,
    description: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    on_change: impl Fn(f64) + 'static,
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
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            Slider::new(move |v| {
                let actual = from_slider(v);
                on_change(actual);
            })
            .value(to_slider(value))
            .size(Size::px(100.0)),
        )
        .child(
            label()
                .text(format!("{:.0}{}", value, unit))
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_SM)
                .width(Size::px(40.0))
                .max_lines(1),
        )
        .into()
}

// ============================================================================
// Text Row - Text input field (plain function, no hooks)
// ============================================================================

/// Create a setting row with a text input.
/// Parent manages state and passes value + on_change callback.
pub fn text_row(
    title: &str,
    description: &str,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();
    let value = value.to_string();
    let placeholder = placeholder.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            // Labels - take remaining space
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            Input::new()
                .value(value)
                .placeholder(placeholder)
                .width(Size::px(140.0))
                .on_change(move |v: String| {
                    on_change(v);
                }),
        )
        .into()
}

// ============================================================================
// Display-only rows (no hooks needed)
// ============================================================================

/// A simple toggle row that just displays a value (no interactivity)
pub fn toggle_row_display(title: &str, description: &str, value: bool) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value_text = if value { "On" } else { "Off" };

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value_text)
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
                .max_lines(1),
        )
}

/// A slider row that just displays a value (no interactivity)
pub fn slider_row_display(title: &str, description: &str, value: f64, unit: &str) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value_text = format!("{:.0} {}", value, unit);

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value_text)
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
                .max_lines(1),
        )
}

/// A text row that just displays a value (no interactivity)
pub fn text_row_display(title: &str, description: &str, value: &str) -> impl IntoElement {
    let title = title.to_string();
    let description = description.to_string();
    let value = value.to_string();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value)
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
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
        .height(Size::px(50.0))
        .cross_align(Alignment::Center)
        .spacing(SPACING_MD)
        .child(
            rect()
                .width(Size::flex(1.0))
                .child(
                    label()
                        .text(title)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(description)
                        .color(TEXT_SECONDARY)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
        .child(
            label()
                .text(value)
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_BASE)
                .max_lines(1),
        )
}
