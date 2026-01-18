//! Section component with uppercase header

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::theme::{section_style, ACCENT, SPACING_MD};

/// Create a section with an uppercase title
pub fn section<V: IntoView + 'static>(title: &'static str, content: V) -> impl IntoView {
    Stack::vertical((
        // Section header (uppercase, accent color)
        Label::derived(move || title.to_uppercase()).style(|s| {
            s.font_size(11.0)
                .font_bold()
                .color(ACCENT)
                .margin_bottom(SPACING_MD)
        }),
        // Content
        content,
    ))
    .style(section_style)
}
