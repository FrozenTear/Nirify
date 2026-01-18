//! Reusable UI components for niri-settings
//!
//! These components provide consistent styling and behavior across the app.

mod section;
mod setting_rows;

pub use section::section;
pub use setting_rows::{
    color_row, color_row_with_callback, dropdown_row, slider_row, slider_row_with_callback,
    text_row, toggle_row, toggle_row_with_callback,
};
