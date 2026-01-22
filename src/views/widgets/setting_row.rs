//! Setting row widgets - reusable components for settings UI
//!
//! These helper functions create consistent, well-styled setting rows
//! that are used throughout the application.

use iced::widget::{column, container, pick_list, row, slider, text, text_input, toggler};
use iced::{Alignment, Element, Length};
use crate::theme::fonts;

/// Creates a toggle row with label and description
///
/// # Example
/// ```rust,ignore
/// toggle_row(
///     "Enable focus ring",
///     "Show a colored ring around the focused window",
///     settings.focus_ring_enabled,
///     AppearanceMessage::ToggleFocusRing,
/// )
/// ```
pub fn toggle_row<'a, Message: 'a>(
    label: &'a str,
    description: &'a str,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        // Left side: Label and description
        column![
            text(label).size(16),
            text(description).size(12).color([0.7, 0.7, 0.7]),
        ]
        .spacing(4)
        .width(Length::Fill),
        // Right side: Toggle switch
        toggler(value).on_toggle(on_toggle).width(Length::Shrink),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}

/// Creates a slider row with label, description, and value display
///
/// Supports an optional `enabled` parameter to grey out the control when disabled.
///
/// # Example
/// ```rust,ignore
/// slider_row(
///     "Ring width",
///     "Thickness in pixels",
///     settings.focus_ring_width,
///     1.0,
///     20.0,
///     "px",
///     AppearanceMessage::SetFocusRingWidth,
/// )
/// ```
pub fn slider_row<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: f32,
    min: f32,
    max: f32,
    unit: &'a str,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        // Top: Label and current value
        row![
            text(label).size(16).width(Length::Fill),
            text(format!("{:.1}{}", value, unit))
                .size(14)
                .color([0.8, 0.8, 0.8])
                .font(fonts::MONO_FONT),
        ]
        .align_y(Alignment::Center),
        // Middle: Description
        text(description).size(12).color([0.7, 0.7, 0.7]),
        // Bottom: Slider
        slider(min..=max, value, on_change).step(0.1),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates a slider row with optional enabled/disabled state
///
/// When disabled, the slider is greyed out and non-interactive
///
/// # Example
/// ```rust,ignore
/// slider_row_with_state(
///     "Ring width",
///     "Thickness in pixels",
///     settings.focus_ring_width,
///     1.0,
///     20.0,
///     "px",
///     settings.focus_ring_enabled,  // enabled parameter
///     AppearanceMessage::SetFocusRingWidth,
/// )
/// ```
#[allow(clippy::too_many_arguments)]
pub fn slider_row_with_state<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: f32,
    min: f32,
    max: f32,
    unit: &'a str,
    enabled: bool,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    let label_color = if enabled { [1.0, 1.0, 1.0] } else { [0.5, 0.5, 0.5] };
    let desc_color = if enabled { [0.7, 0.7, 0.7] } else { [0.4, 0.4, 0.4] };
    let value_color = if enabled { [0.8, 0.8, 0.8] } else { [0.5, 0.5, 0.5] };

    let content = column![
        // Top: Label and current value
        row![
            text(label).size(16).width(Length::Fill).color(label_color),
            text(format!("{:.1}{}", value, unit))
                .size(14)
                .color(value_color)
                .font(fonts::MONO_FONT),
        ]
        .align_y(Alignment::Center),
        // Middle: Description
        text(description).size(12).color(desc_color),
        // Bottom: Slider
        slider(min..=max, value, on_change).step(0.1),
    ]
    .spacing(6)
    .padding(12);

    if enabled {
        content.into()
    } else {
        container(content).style(|_theme| {
            container::Style {
                text_color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                ..Default::default()
            }
        }).into()
    }
}

/// Creates a slider row with integer values
///
/// # Example
/// ```rust,ignore
/// slider_row_int(
///     "Repeat rate",
///     "Characters per second",
///     settings.keyboard.repeat_rate,
///     1,
///     100,
///     "cps",
///     KeyboardMessage::SetRepeatRate,
/// )
/// ```
pub fn slider_row_int<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: i32,
    min: i32,
    max: i32,
    unit: &'a str,
    on_change: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        // Top: Label and current value
        row![
            text(label).size(16).width(Length::Fill),
            text(format!("{}{}", value, unit))
                .size(14)
                .color([0.8, 0.8, 0.8])
                .font(fonts::MONO_FONT),
        ]
        .align_y(Alignment::Center),
        // Middle: Description
        text(description).size(12).color([0.7, 0.7, 0.7]),
        // Bottom: Slider
        slider(min..=max, value, on_change).step(1),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates an integer slider row with optional enabled/disabled state
///
/// When disabled, the slider is greyed out and non-interactive
///
/// # Example
/// ```rust,ignore
/// slider_row_int_with_state(
///     "Repeat rate",
///     "Characters per second",
///     settings.keyboard.repeat_rate,
///     1,
///     100,
///     "cps",
///     settings.keyboard_enabled,  // enabled parameter
///     KeyboardMessage::SetRepeatRate,
/// )
/// ```
#[allow(clippy::too_many_arguments)]
pub fn slider_row_int_with_state<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: i32,
    min: i32,
    max: i32,
    unit: &'a str,
    enabled: bool,
    on_change: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let label_color = if enabled { [1.0, 1.0, 1.0] } else { [0.5, 0.5, 0.5] };
    let desc_color = if enabled { [0.7, 0.7, 0.7] } else { [0.4, 0.4, 0.4] };
    let value_color = if enabled { [0.8, 0.8, 0.8] } else { [0.5, 0.5, 0.5] };

    let content = column![
        // Top: Label and current value
        row![
            text(label).size(16).width(Length::Fill).color(label_color),
            text(format!("{}{}", value, unit))
                .size(14)
                .color(value_color)
                .font(fonts::MONO_FONT),
        ]
        .align_y(Alignment::Center),
        // Middle: Description
        text(description).size(12).color(desc_color),
        // Bottom: Slider
        slider(min..=max, value, on_change).step(1),
    ]
    .spacing(6)
    .padding(12);

    if enabled {
        content.into()
    } else {
        container(content).style(|_theme| {
            container::Style {
                text_color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                ..Default::default()
            }
        }).into()
    }
}

/// Creates a text input row with label and description
///
/// # Example
/// ```rust,ignore
/// text_input_row(
///     "XKB Layout",
///     "Keyboard layout (e.g., 'us', 'de')",
///     &settings.keyboard.xkb_layout,
///     KeyboardMessage::SetXkbLayout,
/// )
/// ```
pub fn text_input_row<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(16),
        text(description).size(12).color([0.7, 0.7, 0.7]),
        text_input("", value).on_input(on_change).padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates a section header for grouping related settings
///
/// # Example
/// ```rust,ignore
/// section_header("Focus Ring")
/// ```
pub fn section_header<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    container(
        text(label)
            .size(18)
            .color([0.9, 0.9, 0.9])
    )
    .padding(12)
    .into()
}

/// Creates a subsection header (smaller than section header)
///
/// # Example
/// ```rust,ignore
/// subsection_header("Advanced Options")
/// ```
pub fn subsection_header<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    container(
        text(label)
            .size(15)
            .color([0.8, 0.8, 0.8])
    )
    .padding(12)
    .into()
}

/// Creates a spacer element for vertical spacing
pub fn spacer<'a, Message: 'a>(height: f32) -> Element<'a, Message> {
    container(text(""))
        .height(Length::Fixed(height))
        .into()
}

/// Creates a picker/dropdown row for enum selections
///
/// # Example
/// ```rust,ignore
/// picker_row(
///     "Acceleration Profile",
///     "Controls pointer acceleration behavior",
///     &AccelProfile::all(),
///     Some(settings.mouse.accel_profile),
///     MouseMessage::SetAccelProfile,
/// )
/// ```
pub fn picker_row<'a, T, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: Clone + Eq + std::fmt::Display + 'a,
{
    row![
        // Left side: Label and description
        column![
            text(label).size(16),
            text(description).size(12).color([0.7, 0.7, 0.7]),
        ]
        .spacing(4)
        .width(Length::Fill),
        // Right side: Picker dropdown
        pick_list(options, selected, on_select)
            .width(Length::Fixed(200.0))
            .padding([8, 12]),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}

/// Creates an info text block (for hints, warnings, etc.)
pub fn info_text<'a, Message: 'a>(content: &'a str) -> Element<'a, Message> {
    container(
        text(content)
            .size(13)
            .color([0.6, 0.7, 0.9])
    )
    .padding(12)
    .into()
}
