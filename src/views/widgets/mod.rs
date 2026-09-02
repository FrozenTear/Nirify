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
pub mod error_banner;
pub mod expandable_section;
pub mod file_path;
pub mod gradient_picker;
pub mod key_capture;
pub mod layout_override_editor;
pub mod list_detail;
pub mod list_item;
pub mod optional_picker;
pub mod preset_list;
pub mod setting_row;

// Re-export commonly used helpers
pub use calibration_matrix::{calibration_matrix, format_matrix_values, CalibrationMatrixMessage};
pub use color_picker::{color_picker_row, color_picker_with_swatches, optional_color_row};
pub use expandable_section::expandable_section;
pub use file_path::{browse_task, file_path_picker, FilePathMessage, FilePickerType};
pub use gradient_picker::{
    apply_gradient_message, apply_optional_gradient_message, gradient_picker,
    optional_gradient_picker, GradientPickerMessage,
};
pub use key_capture::{
    format_key_combination, is_modifier_only, key_capture_row, KeyCaptureMessage, KeyCaptureState,
};
pub use layout_override_editor::{layout_override_content, layout_override_editor};
pub use list_detail::{
    action_button, action_button_style, add_button, add_button_style, add_item_button, badge,
    delete_button, delete_button_style, empty_detail_placeholder, empty_list_placeholder,
    list_detail_layout, list_item_style, list_panel_style, match_container_style, remove_button,
    selection_indicator, BADGE_BEHAVIOR, BADGE_VISIBILITY,
};
pub use list_item::list_item;
pub use optional_picker::{optional_bool_picker, OptionalBool};
pub use preset_list::preset_entry_row;
pub use setting_row::{
    card, card_group, info_text, optional_picker_row, optional_slider_row, page_title, picker_row,
    section_header, slider_row, slider_row_int, slider_row_int_with_state, slider_row_with_state,
    spacer, status_pill, subsection_header, text_input_row, text_input_with_suggestions,
    toggle_row,
};
