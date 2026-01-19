//! Reusable UI components for niri-settings

mod section;
mod setting_rows;

pub use section::{collapsible_section, collapsible_section_minimal, section, section_minimal};
pub use setting_rows::{
    select_row, select_row_with_state, slider_row, slider_row_display, text_row, text_row_display,
    toggle_row, toggle_row_display, value_row,
};
