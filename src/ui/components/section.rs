//! Section component - subtle card with refined header
//!
//! Creates visual grouping for related settings with
//! understated headers that don't compete for attention

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::theme::{section_header_style, section_style};

/// Create a section card with a subtle title header
///
/// # Arguments
/// * `title` - The section title (displayed as-is, not transformed)
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
        // Section header - subtle, understated
        Label::derived(move || title.to_string()).style(section_header_style),
        // Section content
        content,
    ))
    .style(section_style)
}
