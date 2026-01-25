//! Gradient picker widget for color or gradient configuration
//!
//! Allows choosing between solid colors and gradients with full control over
//! gradient properties including angle, color space, and interpolation.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use crate::types::{Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo, HueInterpolation};
use crate::theme::muted_text_container;
use super::color_picker::color_picker_row;
use super::setting_row::{info_text, picker_row, section_header, slider_row_int, spacer};

/// Messages for gradient picker interactions
#[derive(Debug, Clone)]
pub enum GradientPickerMessage {
    ToggleSolidGradient(bool),  // true = gradient, false = solid
    SetFromColor(String),
    SetToColor(String),
    SetAngle(i32),
    SetColorSpace(ColorSpace),
    SetRelativeTo(GradientRelativeTo),
    SetHueInterpolation(HueInterpolation),
}

/// Creates an expandable gradient picker widget
///
/// Shows either a simple color picker or full gradient controls based on the value.
pub fn gradient_picker<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: &ColorOrGradient,
    on_change: impl Fn(GradientPickerMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let is_gradient = value.is_gradient();

    let mut content = column![
        section_header(label),
        info_text(description),
    ]
    .spacing(8);

    // Toggle button
    let toggle_text = if is_gradient { "Gradient" } else { "Solid Color" };
    let toggle = button(text(toggle_text))
        .on_press(on_change(GradientPickerMessage::ToggleSolidGradient(!is_gradient)))
        .padding([8, 16]);

    content = content.push(
        row![
            text("Type:").size(14),
            toggle,
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    );

    content = content.push(spacer(8.0));

    match value {
        ColorOrGradient::Color(color) => {
            // Simple color picker
            content = content.push(
                color_picker_row(
                    "Color",
                    "Solid color value",
                    color,
                    move |hex| on_change(GradientPickerMessage::SetFromColor(hex)),
                )
            );
        }
        ColorOrGradient::Gradient(gradient) => {
            // Full gradient controls
            content = content.push(
                gradient_controls(gradient, on_change)
            );
        }
    }

    container(content)
        .padding(12)
        .style(|theme: &iced::Theme| {
            let bg = theme.palette().background;
            let border_color = IcedColor {
                r: bg.r + 0.15,
                g: bg.g + 0.15,
                b: bg.b + 0.15,
                a: 1.0,
            };
            container::Style {
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Creates the full gradient control panel
fn gradient_controls<'a, Message: Clone + 'a>(
    gradient: &Gradient,
    on_change: impl Fn(GradientPickerMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let mut controls = column![]
        .spacing(8);

    // Gradient preview (simple two-color bar)
    let preview = gradient_preview(&gradient.from, &gradient.to);
    controls = controls.push(container(preview).padding(8));

    // From color
    controls = controls.push(
        color_picker_row(
            "From color",
            "Starting color of the gradient",
            &gradient.from,
            move |hex| on_change(GradientPickerMessage::SetFromColor(hex)),
        )
    );

    // To color
    controls = controls.push(
        color_picker_row(
            "To color",
            "Ending color of the gradient",
            &gradient.to,
            move |hex| on_change(GradientPickerMessage::SetToColor(hex)),
        )
    );

    // Angle slider
    controls = controls.push(
        slider_row_int(
            "Angle",
            "Gradient angle in degrees (0=right, 90=down, 180=left, 270=up)",
            gradient.angle,
            0,
            360,
            "°",
            move |value| on_change(GradientPickerMessage::SetAngle(value)),
        )
    );

    // Color space picker
    controls = controls.push(
        picker_row(
            "Color space",
            "Color interpolation space for the gradient",
            ColorSpace::all(),
            Some(gradient.color_space),
            move |value| on_change(GradientPickerMessage::SetColorSpace(value)),
        )
    );

    // Hue interpolation (only for Oklch)
    if gradient.color_space == ColorSpace::Oklch {
        let hue_interp = gradient.hue_interpolation.unwrap_or(HueInterpolation::Shorter);
        controls = controls.push(
            picker_row(
                "Hue interpolation",
                "How hue values are interpolated in Oklch space",
                HueInterpolation::all(),
                Some(hue_interp),
                move |value| on_change(GradientPickerMessage::SetHueInterpolation(value)),
            )
        );
    }

    // Relative to
    controls = controls.push(
        picker_row(
            "Relative to",
            "Whether gradient position is relative to window or workspace view",
            GradientRelativeTo::all(),
            Some(gradient.relative_to),
            move |value| on_change(GradientPickerMessage::SetRelativeTo(value)),
        )
    );

    controls.into()
}

/// Creates a simple gradient preview (horizontal bar with two colors)
fn gradient_preview<'a, Message: 'a>(from: &Color, to: &Color) -> Element<'a, Message> {
    let from_iced = IcedColor::from_rgb8(from.r, from.g, from.b);
    let to_iced = IcedColor::from_rgb8(to.r, to.g, to.b);

    // Create two adjacent colored boxes to simulate gradient
    let from_box = container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(40.0))
        .style(move |theme: &iced::Theme| {
            let bg = theme.palette().background;
            let border_color = IcedColor { r: bg.r + 0.15, g: bg.g + 0.15, b: bg.b + 0.15, a: 1.0 };
            container::Style {
                background: Some(iced::Background::Color(from_iced)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    let to_box = container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(40.0))
        .style(move |theme: &iced::Theme| {
            let bg = theme.palette().background;
            let border_color = IcedColor { r: bg.r + 0.15, g: bg.g + 0.15, b: bg.b + 0.15, a: 1.0 };
            container::Style {
                background: Some(iced::Background::Color(to_iced)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    column![
        container(text("Gradient Preview (From → To)").size(13)).style(muted_text_container),
        row![from_box, to_box].spacing(0),
    ]
    .spacing(4)
    .into()
}
