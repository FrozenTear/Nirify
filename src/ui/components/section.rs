//! Section component - a card-like container for grouping settings

use floem::prelude::*;
use floem::views::{empty, Label, Stack};

use crate::ui::theme::{section_style, SPACING_LG, SPACING_SM, TEXT_PRIMARY, TEXT_SECONDARY};

/// Create a section with a title and content
pub fn section<V: IntoView + 'static>(
    title: impl Into<String>,
    description: Option<&'static str>,
    content: V,
) -> impl IntoView {
    let title = title.into();

    let header = Stack::vertical((
        Label::derived(move || title.clone())
            .style(|s| s.font_size(16.0).font_bold().color(TEXT_PRIMARY)),
        match description {
            Some(desc) => Label::derived(move || desc.to_string())
                .style(|s| {
                    s.font_size(12.0)
                        .color(TEXT_SECONDARY)
                        .margin_top(SPACING_SM / 2.0)
                })
                .into_any(),
            None => empty().into_any(),
        },
    ))
    .style(|s| s.margin_bottom(SPACING_LG));

    Stack::vertical((header, content)).style(section_style)
}

/// Create a section with just a title (no description)
pub fn section_simple<V: IntoView + 'static>(title: impl Into<String>, content: V) -> impl IntoView {
    section(title, None, content)
}
