//! Section component - elevated glass-like card with header

use freya::prelude::*;

use crate::ui::theme::*;

/// Create a section card with an uppercase title header
pub fn section(title: &str, content: impl IntoElement) -> impl IntoElement {
    let title = title.to_uppercase();

    rect()
        .width(Size::fill())
        .background(BG_SURFACE)
        .corner_radius(RADIUS_LG)
        .padding((SPACING_XL, SPACING_XL, SPACING_XL, SPACING_XL))
        .spacing(SPACING_LG)
        .child(
            // Section header
            label()
                .text(title)
                .color(ACCENT)
                .font_size(FONT_SIZE_XS)
                .font_weight(FontWeight::BOLD)
                .max_lines(1),
        )
        .child(content)
}
