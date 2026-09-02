//! Gradient picker widget for color or gradient configuration
//!
//! Allows choosing between solid colors and gradients with full control over
//! gradient properties including angle, color space, and interpolation.

use iced::widget::{button, column, container, row, text, toggler};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use super::color_picker::color_picker_row;
use super::setting_row::{info_text, picker_row, section_header, slider_row_int, spacer};
use crate::theme::muted_text_container;
use crate::types::{
    Color, ColorOrGradient, ColorSpace, Gradient, GradientRelativeTo, HueInterpolation,
};

/// Messages for gradient picker interactions
#[derive(Debug, Clone)]
pub enum GradientPickerMessage {
    ToggleSolidGradient(bool), // true = gradient, false = solid
    SetFromColor(String),
    SetToColor(String),
    SetAngle(i32),
    SetColorSpace(ColorSpace),
    SetRelativeTo(GradientRelativeTo),
    SetHueInterpolation(HueInterpolation),
}

/// Apply a picker message to a `ColorOrGradient` without collapsing a gradient
/// to a solid color unless the user explicitly toggles "Solid Color".
pub fn apply_gradient_message(target: &mut ColorOrGradient, msg: GradientPickerMessage) {
    match msg {
        GradientPickerMessage::ToggleSolidGradient(is_gradient) => {
            *target = if is_gradient {
                match target {
                    ColorOrGradient::Color(color) => ColorOrGradient::Gradient(Gradient {
                        from: *color,
                        to: *color,
                        angle: 0,
                        ..Default::default()
                    }),
                    ColorOrGradient::Gradient(_) => target.clone(),
                }
            } else {
                match target {
                    ColorOrGradient::Color(_) => target.clone(),
                    ColorOrGradient::Gradient(gradient) => ColorOrGradient::Color(gradient.from),
                }
            };
        }
        GradientPickerMessage::SetFromColor(hex) => {
            if let Some(color) = Color::from_hex(&hex) {
                match target {
                    ColorOrGradient::Color(c) => *c = color,
                    ColorOrGradient::Gradient(g) => g.from = color,
                }
            }
        }
        GradientPickerMessage::SetToColor(hex) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                if let Some(color) = Color::from_hex(&hex) {
                    gradient.to = color;
                }
            }
        }
        GradientPickerMessage::SetAngle(angle) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                gradient.angle = angle;
            }
        }
        GradientPickerMessage::SetColorSpace(color_space) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                gradient.color_space = color_space;
            }
        }
        GradientPickerMessage::SetRelativeTo(relative_to) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                gradient.relative_to = relative_to;
            }
        }
        GradientPickerMessage::SetHueInterpolation(hue_interp) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                gradient.hue_interpolation = Some(hue_interp);
            }
        }
    }
}

/// Apply a picker message to an optional override, enabling it if needed.
pub fn apply_optional_gradient_message(
    target: &mut Option<ColorOrGradient>,
    msg: GradientPickerMessage,
) {
    let mut value = target.clone().unwrap_or_default();
    apply_gradient_message(&mut value, msg);
    *target = Some(value);
}

/// Creates an expandable gradient picker widget
///
/// Shows either a simple color picker or full gradient controls based on the value.
pub fn gradient_picker<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: &ColorOrGradient,
    on_change: impl Fn(GradientPickerMessage) -> Message + 'a + Clone,
) -> Element<'a, Message> {
    container(
        column![
            section_header(label),
            info_text(description),
            gradient_editor(value, on_change),
        ]
        .spacing(8),
    )
    .padding(12)
    .style(picker_card_style)
    .into()
}

/// Optional `ColorOrGradient` override: inherit/off toggle plus a real gradient editor.
///
/// Editing a color never flattens an imported gradient to a solid `Color`.
pub fn optional_gradient_picker<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    value: Option<&ColorOrGradient>,
    on_change: impl Fn(Option<ColorOrGradient>) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_enabled = value.is_some();
    let on_toggle = on_change.clone();

    let mut content = column![row![
        column![
            text(label).size(15),
            container(text(description).size(11)).style(muted_text_container),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(is_enabled).on_toggle(move |enabled| {
            if enabled {
                on_toggle(Some(ColorOrGradient::default()))
            } else {
                on_toggle(None)
            }
        }),
    ]
    .spacing(12)
    .align_y(Alignment::Center),]
    .spacing(8)
    .padding(12);

    if let Some(cog) = value {
        let current = cog.clone();
        let on_picker = on_change;
        content = content.push(gradient_editor(cog, move |msg| {
            let mut updated = current.clone();
            apply_gradient_message(&mut updated, msg);
            on_picker(Some(updated))
        }));
    }

    container(content).style(picker_card_style).into()
}

/// Type toggle + solid/gradient controls (no section header).
fn gradient_editor<'a, Message: Clone + 'a>(
    value: &ColorOrGradient,
    on_change: impl Fn(GradientPickerMessage) -> Message + 'a + Clone,
) -> Element<'a, Message> {
    let is_gradient = value.is_gradient();
    let on_toggle = on_change.clone();

    let toggle_text = if is_gradient {
        "Gradient"
    } else {
        "Solid Color"
    };
    let toggle = button(text(toggle_text))
        .on_press(on_toggle(GradientPickerMessage::ToggleSolidGradient(
            !is_gradient,
        )))
        .padding([8, 16]);

    let mut content = column![row![text("Type:").size(14), toggle,]
        .spacing(12)
        .align_y(Alignment::Center),]
    .spacing(8);

    content = content.push(spacer(8.0));

    match value {
        ColorOrGradient::Color(color) => {
            let on_color = on_change;
            content = content.push(color_picker_row(
                "Color",
                "Solid color value",
                color,
                move |hex| on_color(GradientPickerMessage::SetFromColor(hex)),
            ));
        }
        ColorOrGradient::Gradient(gradient) => {
            content = content.push(gradient_controls(gradient, on_change));
        }
    }

    content.into()
}

/// Creates the full gradient control panel
fn gradient_controls<'a, Message: Clone + 'a>(
    gradient: &Gradient,
    on_change: impl Fn(GradientPickerMessage) -> Message + 'a + Clone,
) -> Element<'a, Message> {
    let mut controls = column![].spacing(8);

    // Gradient preview (simple two-color bar)
    let preview = gradient_preview(&gradient.from, &gradient.to);
    controls = controls.push(container(preview).padding(8));

    let on_from = on_change.clone();
    controls = controls.push(color_picker_row(
        "From color",
        "Starting color of the gradient",
        &gradient.from,
        move |hex| on_from(GradientPickerMessage::SetFromColor(hex)),
    ));

    let on_to = on_change.clone();
    controls = controls.push(color_picker_row(
        "To color",
        "Ending color of the gradient",
        &gradient.to,
        move |hex| on_to(GradientPickerMessage::SetToColor(hex)),
    ));

    let on_angle = on_change.clone();
    controls = controls.push(slider_row_int(
        "Angle",
        "Gradient angle in degrees (0=right, 90=down, 180=left, 270=up)",
        gradient.angle,
        0,
        360,
        "°",
        move |value| on_angle(GradientPickerMessage::SetAngle(value)),
    ));

    let on_space = on_change.clone();
    controls = controls.push(picker_row(
        "Color space",
        "Color interpolation space for the gradient",
        ColorSpace::all(),
        Some(gradient.color_space),
        move |value| on_space(GradientPickerMessage::SetColorSpace(value)),
    ));

    if gradient.color_space == ColorSpace::Oklch {
        let hue_interp = gradient
            .hue_interpolation
            .unwrap_or(HueInterpolation::Shorter);
        let on_hue = on_change.clone();
        controls = controls.push(picker_row(
            "Hue interpolation",
            "How hue values are interpolated in Oklch space",
            HueInterpolation::all(),
            Some(hue_interp),
            move |value| on_hue(GradientPickerMessage::SetHueInterpolation(value)),
        ));
    }

    controls = controls.push(picker_row(
        "Relative to",
        "Whether gradient position is relative to window or workspace view",
        GradientRelativeTo::all(),
        Some(gradient.relative_to),
        move |value| on_change(GradientPickerMessage::SetRelativeTo(value)),
    ));

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
            let border_color = IcedColor {
                r: bg.r + 0.15,
                g: bg.g + 0.15,
                b: bg.b + 0.15,
                a: 1.0,
            };
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
            let border_color = IcedColor {
                r: bg.r + 0.15,
                g: bg.g + 0.15,
                b: bg.b + 0.15,
                a: 1.0,
            };
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

fn picker_card_style(theme: &iced::Theme) -> container::Style {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(hex: &str) -> Color {
        Color::from_hex(hex).expect("valid hex")
    }

    fn sample_gradient() -> Gradient {
        Gradient {
            from: color("#112233"),
            to: color("#445566"),
            angle: 90,
            relative_to: GradientRelativeTo::Window,
            color_space: ColorSpace::Srgb,
            hue_interpolation: None,
        }
    }

    #[test]
    fn editing_from_color_preserves_gradient() {
        let mut value = ColorOrGradient::Gradient(sample_gradient());
        apply_gradient_message(
            &mut value,
            GradientPickerMessage::SetFromColor("#ff0000".into()),
        );
        match value {
            ColorOrGradient::Gradient(g) => {
                assert_eq!(g.from, color("#ff0000"));
                assert_eq!(g.to, color("#445566"));
                assert_eq!(g.angle, 90);
            }
            ColorOrGradient::Color(_) => panic!("gradient flattened to solid color"),
        }
    }

    #[test]
    fn editing_solid_color_stays_solid() {
        let mut value = ColorOrGradient::Color(color("#abcdef"));
        apply_gradient_message(
            &mut value,
            GradientPickerMessage::SetFromColor("#123456".into()),
        );
        assert_eq!(value, ColorOrGradient::Color(color("#123456")));
    }

    #[test]
    fn toggle_to_solid_uses_from_color() {
        let mut value = ColorOrGradient::Gradient(sample_gradient());
        apply_gradient_message(
            &mut value,
            GradientPickerMessage::ToggleSolidGradient(false),
        );
        assert_eq!(value, ColorOrGradient::Color(color("#112233")));
    }

    #[test]
    fn toggle_to_gradient_keeps_solid_as_both_stops() {
        let mut value = ColorOrGradient::Color(color("#7fc8ff"));
        apply_gradient_message(&mut value, GradientPickerMessage::ToggleSolidGradient(true));
        match value {
            ColorOrGradient::Gradient(g) => {
                assert_eq!(g.from, color("#7fc8ff"));
                assert_eq!(g.to, color("#7fc8ff"));
            }
            ColorOrGradient::Color(_) => panic!("expected gradient"),
        }
    }

    #[test]
    fn optional_edit_does_not_flatten() {
        let mut value = Some(ColorOrGradient::Gradient(sample_gradient()));
        apply_optional_gradient_message(
            &mut value,
            GradientPickerMessage::SetToColor("#00ff00".into()),
        );
        match value {
            Some(ColorOrGradient::Gradient(g)) => {
                assert_eq!(g.from, color("#112233"));
                assert_eq!(g.to, color("#00ff00"));
            }
            other => panic!("expected gradient override, got {other:?}"),
        }
    }

    #[test]
    fn optional_edit_enables_none_as_solid() {
        let mut value = None;
        apply_optional_gradient_message(
            &mut value,
            GradientPickerMessage::SetFromColor("#aabbcc".into()),
        );
        assert_eq!(value, Some(ColorOrGradient::Color(color("#aabbcc"))));
    }
}
