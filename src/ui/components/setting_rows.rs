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

// ============================================================================
// Select Row - Dropdown selection with popup menu
// ============================================================================

use crate::ui::app::ReactiveState;
use std::cell::RefCell;
use std::rc::Rc;

/// Create a setting row with a dropdown selection control.
/// Shows a floating popup menu when clicked that overlays other content.
pub fn select_row_with_state(
    title: &str,
    description: &str,
    options: &[&str],
    selected: usize,
    dropdown_id: usize,
    state: &ReactiveState,
    on_change: impl FnMut(usize) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();
    let options_vec: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    let current_value = options_vec.get(selected).cloned().unwrap_or_default();

    let is_open = *state.open_dropdown.read() == Some(dropdown_id);
    let mut open_dropdown = state.open_dropdown.clone();
    let open_dropdown_for_close = state.open_dropdown.clone();
    let mut refresh = state.refresh.clone();

    // Wrap on_change in Rc<RefCell> so it can be shared across multiple closures
    let on_change = Rc::new(RefCell::new(on_change));

    // Outer container - fixed height row
    let mut outer = rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(ROW_HEIGHT))
        .cross_align(Alignment::Center)
        .spacing(SPACING_LG);

    // Labels - take remaining space
    outer = outer.child(
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
    );

    // Dropdown button container - fixed width like other controls
    let mut button_container = rect()
        .content(Content::flex())
        .direction(Direction::Vertical)
        .width(Size::px(160.0));

    // Dropdown button
    button_container = button_container.child(
        rect()
            .content(Content::flex())
            .direction(Direction::Horizontal)
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
            .corner_radius(RADIUS_MD)
            .background(if is_open { ACCENT_VIVID } else { BG_ELEVATED })
            .spacing(SPACING_SM)
            .on_pointer_down(move |_| {
                if is_open {
                    *open_dropdown.write() = None;
                } else {
                    *open_dropdown.write() = Some(dropdown_id);
                }
                *refresh.write() += 1;
            })
            .child(
                label()
                    .width(Size::flex(1.0))
                    .text(current_value)
                    .color(if is_open { BG_DEEP } else { TEXT_BRIGHT })
                    .font_size(FONT_SIZE_SM)
                    .max_lines(1),
            )
            .child(
                label()
                    .text(if is_open { "▲" } else { "▼" })
                    .color(if is_open { BG_DEEP } else { TEXT_DIM })
                    .font_size(FONT_SIZE_XS),
            ),
    );

    // Options menu (absolute positioned overlay)
    if is_open {
        let mut menu = rect()
            .content(Content::flex())
            .direction(Direction::Vertical)
            .position(Position::new_absolute().top(36.0).left(0.0))
            .width(Size::fill())
            .layer(Layer::Overlay)
            .background(BG_SURFACE)
            .corner_radius(RADIUS_MD)
            .padding((SPACING_XS, SPACING_XS, SPACING_XS, SPACING_XS))
            .shadow((0.0, 4.0, 12.0, 0.0, (0, 0, 0, 120)));

        for (idx, opt) in options_vec.iter().enumerate() {
            let is_selected = idx == selected;
            let opt_text = opt.clone();
            let mut open_dropdown_item = open_dropdown_for_close.clone();
            let mut refresh_item = state.refresh.clone();
            let on_change_clone = on_change.clone();

            menu = menu.child(
                rect()
                    .content(Content::flex())
                    .width(Size::fill())
                    .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                    .corner_radius(RADIUS_SM)
                    .background(if is_selected { SELECTED_BG } else { (0, 0, 0, 0) })
                    .on_pointer_down(move |_| {
                        (on_change_clone.borrow_mut())(idx);
                        *open_dropdown_item.write() = None;
                        *refresh_item.write() += 1;
                    })
                    .child(
                        label()
                            .text(opt_text)
                            .color(if is_selected { ACCENT_VIVID } else { TEXT_BRIGHT })
                            .font_size(FONT_SIZE_SM),
                    ),
            );
        }

        button_container = button_container.child(menu);
    }

    outer = outer.child(button_container);
    outer.into()
}

/// Simple select row that cycles through options (no state required)
pub fn select_row(
    title: &str,
    description: &str,
    options: &[&str],
    selected: usize,
    mut on_change: impl FnMut(usize) + 'static,
) -> Element {
    let title = title.to_string();
    let description = description.to_string();
    let options: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    let current_value = options.get(selected).cloned().unwrap_or_default();
    let num_options = options.len();

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
            // Dropdown-style button (click to cycle)
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                .corner_radius(RADIUS_MD)
                .background(BG_ELEVATED)
                .min_width(Size::px(140.0))
                .spacing(SPACING_SM)
                .on_pointer_down(move |_| {
                    let next = (selected + 1) % num_options;
                    on_change(next);
                })
                .child(
                    label()
                        .text(current_value)
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text("◀▶")
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_XS),
                ),
        )
        .into()
}

