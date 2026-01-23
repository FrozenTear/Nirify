//! Widget demonstration page for testing reusable components

use iced::widget::{column, scrollable};
use iced::Element;

use super::widgets::*;
use crate::messages::Message;
use crate::types::{Color, ColorOrGradient, Gradient};

/// Test message for widget demo
#[derive(Debug, Clone)]
pub enum DemoMessage {
    ToggleChanged(bool),
    SliderChanged(f32),
    IntSliderChanged(i32),
    TextChanged(String),
    ColorChanged(String),
}

/// Widget demo state
pub struct DemoState {
    pub toggle_value: bool,
    pub slider_value: f32,
    pub int_slider_value: i32,
    pub text_value: String,
    pub color_value: Color,
    pub gradient_value: ColorOrGradient,
    pub calibration_matrix: Option<[f64; 6]>,
    pub calibration_matrix_formatted: [String; 6],  // Pre-formatted for widget
    pub file_path: String,
}

impl Default for DemoState {
    fn default() -> Self {
        let matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]); // Identity matrix
        Self {
            toggle_value: true,
            slider_value: 10.0,
            int_slider_value: 50,
            text_value: "Hello World".to_string(),
            color_value: Color::from_hex("#7fc8ff").unwrap_or_default(),
            gradient_value: ColorOrGradient::Gradient(Gradient::default()),
            calibration_matrix: matrix,
            calibration_matrix_formatted: format_matrix_values(matrix),
            file_path: "/home/user/example.txt".to_string(),
        }
    }
}

/// Creates the widget demo view
pub fn view<'a>(state: &'a DemoState) -> Element<'a, Message> {
    let content = column![
        section_header("Widget Showcase - Phase 2"),
        info_text("Testing all reusable widget components"),
        spacer(8.0),
        section_header("Toggle Widgets"),
        // Note: For the demo, we'll use Message::None since we don't have actual state updates yet
        toggle_row(
            "Enable Feature",
            "This is a toggle switch with label and description",
            state.toggle_value,
            |_| Message::None,
        ),
        toggle_row(
            "Another Toggle",
            "Demonstrates multiple toggles in sequence",
            !state.toggle_value,
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("Slider Widgets"),
        slider_row(
            "Float Slider",
            "A slider for decimal values with unit display",
            state.slider_value,
            1.0,
            20.0,
            "px",
            |_| Message::None,
        ),
        slider_row_int(
            "Integer Slider",
            "A slider for whole number values",
            state.int_slider_value,
            0,
            100,
            "%",
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("Text Input Widgets"),
        text_input_row(
            "Text Field",
            "Enter any text value here",
            &state.text_value,
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("Color Picker Widgets"),
        color_picker_row(
            "Simple Color Picker",
            "Hex input with color preview",
            &state.color_value,
            |_| Message::None,
        ),
        color_picker_with_swatches(
            "Color Picker with Swatches",
            "Includes preset color swatches for quick selection",
            &state.color_value,
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("Gradient Picker Widget"),
        gradient_picker(
            "Advanced Gradient Editor",
            "Toggle between solid color and gradient with full control over angle, color space, and interpolation",
            &state.gradient_value,
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("Calibration Matrix Widget"),
        calibration_matrix(
            "Tablet Calibration Matrix",
            "2x3 transformation matrix for tablet and touch screen calibration (libinput format)",
            state.calibration_matrix,
            &state.calibration_matrix_formatted,
            |_| Message::None,
        ),
        spacer(8.0),
        section_header("File Path Picker Widget"),
        file_path_picker(
            "Select File",
            "Browse for files with native dialog integration",
            &state.file_path,
            FilePickerType::File,
            |_| Message::None,
        ),
        spacer(8.0),
        subsection_header("Subsection Example"),
        info_text("This is an info text block for providing additional context or hints to the user. It can wrap to multiple lines if needed."),
        spacer(16.0),
    ]
    .spacing(4);

    scrollable(content).into()
}
