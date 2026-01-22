//! Calibration matrix widget for tablet and touch screen configuration
//!
//! Displays a 2x3 matrix of input fields for libinput transformation matrix.
//! The matrix transforms touch coordinates: [x', y'] = [a b c; d e f] * [x, y, 1]

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

/// Messages for calibration matrix interactions
#[derive(Debug, Clone)]
pub enum CalibrationMatrixMessage {
    SetValue(usize, String), // (index 0-5, value as string)
    Clear,
    Reset,
}

/// Creates a calibration matrix editor widget
///
/// Shows a 2x3 grid of number inputs for the 6 calibration values.
/// Displays as:
/// ```text
/// [ a  b  c ]
/// [ d  e  f ]
/// ```
pub fn calibration_matrix<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    matrix: Option<[f64; 6]>,
    on_change: impl Fn(CalibrationMatrixMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let mut content = column![
        text(label).size(16),
        text(description).size(12).color([0.7, 0.7, 0.7]),
    ]
    .spacing(8);

    // Get matrix values or use identity matrix as default
    let values = matrix.unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

    // Create 2x3 grid
    let matrix_grid = column![
        // First row: a, b, c
        row![
            matrix_input("a", values[0], 0, on_change),
            matrix_input("b", values[1], 1, on_change),
            matrix_input("c", values[2], 2, on_change),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        // Second row: d, e, f
        row![
            matrix_input("d", values[3], 3, on_change),
            matrix_input("e", values[4], 4, on_change),
            matrix_input("f", values[5], 5, on_change),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8);

    content = content.push(
        container(matrix_grid)
            .padding(12)
            .style(|_theme| container::Style {
                border: Border {
                    color: IcedColor::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    );

    // Action buttons
    let buttons = row![
        button(text("Reset to Identity"))
            .on_press(on_change(CalibrationMatrixMessage::Reset))
            .padding([6, 12]),
        button(text("Clear"))
            .on_press(on_change(CalibrationMatrixMessage::Clear))
            .padding([6, 12])
            .style(|_theme, _status| button::Style {
                text_color: IcedColor::from_rgb(0.9, 0.4, 0.4),
                ..Default::default()
            }),
    ]
    .spacing(8);

    content = content.push(buttons);

    content = content.push(
        text("Note: Identity matrix [1 0 0; 0 1 0] means no transformation")
            .size(11)
            .color([0.6, 0.6, 0.6])
    );

    container(content)
        .padding(12)
        .into()
}

/// Creates a single matrix value input field
fn matrix_input<'a, Message: Clone + 'a>(
    label: &'a str,
    value: f64,
    index: usize,
    on_change: impl Fn(CalibrationMatrixMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    // Format value with 4 decimal places
    let value_str = format!("{:.4}", value);
    let value_static: &'static str = Box::leak(value_str.into_boxed_str());

    column![
        text(label).size(12).color([0.7, 0.7, 0.7]),
        text_input("0.0", value_static)
            .on_input(move |input| on_change(CalibrationMatrixMessage::SetValue(index, input)))
            .padding(8)
            .width(Length::Fixed(100.0)),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .into()
}
