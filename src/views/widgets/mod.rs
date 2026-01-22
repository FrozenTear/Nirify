//! Reusable widget components
//!
//! Helper functions for creating common UI patterns:
//! - Setting rows (toggle, slider, text input)
//! - Section headers
//! - Color pickers
//! - Expandable sections
//! - List items
//! - etc.

pub mod calibration_matrix;
pub mod color_picker;
pub mod expandable_section;
pub mod file_path;
pub mod gradient_picker;
pub mod key_capture;
pub mod list_item;
pub mod optional_picker;
pub mod setting_row;

// Re-export commonly used helpers
pub use calibration_matrix::{calibration_matrix, CalibrationMatrixMessage};
pub use color_picker::{color_picker_row, color_picker_with_swatches};
pub use expandable_section::expandable_section;
pub use file_path::{browse_task, file_path_picker, FilePathMessage, FilePickerType};
pub use gradient_picker::{gradient_picker, GradientPickerMessage};
pub use key_capture::{format_key_combination, is_modifier_only, key_capture_row, KeyCaptureMessage, KeyCaptureState};
pub use list_item::list_item;
pub use optional_picker::{optional_bool_picker, OptionalBool};
pub use setting_row::{
    info_text, picker_row, section_header, slider_row, slider_row_int, slider_row_int_with_state,
    slider_row_with_state, spacer, subsection_header, text_input_row, toggle_row,
};
