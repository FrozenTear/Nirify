//! Section component - elevated glass-like card with header

use freya::prelude::*;

use crate::ui::theme::*;

/// Create a section card with an uppercase title header
pub fn section(title: &str, content: impl IntoElement) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .background(BG_SURFACE)
        .corner_radius(RADIUS_LG)
        .padding((SPACING_XL, SPACING_XL, SPACING_XL, SPACING_XL))
        .spacing(SPACING_LG)
        .child(
            // Section header
            rect()
                .color(ACCENT)
                .font_size(FONT_SIZE_XS)
                .font_weight(FontWeight::BOLD)
                .child(title.to_uppercase()),
        )
        .child(content)
}
