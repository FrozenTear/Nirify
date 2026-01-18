//! Section component - elevated glass-like card with header
//!
//! Creates visual grouping for related settings with
//! uppercase accent-colored headers

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::theme::{section_header_style, section_style};

/// Create a section card with an uppercase title header
///
/// # Arguments
/// * `title` - The section title (will be displayed in uppercase)
/// * `content` - The section content (typically a stack of setting rows)
///
/// # Example
/// ```ignore
/// section(
///     "Focus Ring",
///     Stack::vertical((
///         toggle_row(...),
///         slider_row(...),
///     ))
/// )
/// ```
pub fn section<V: IntoView + 'static>(title: &'static str, content: V) -> impl IntoView {
    Stack::vertical((
        // Section header - uppercase, accent colored, small caps style
        Label::derived(move || title.to_uppercase()).style(section_header_style),
        // Section content
        content,
    ))
    .style(section_style)
}
