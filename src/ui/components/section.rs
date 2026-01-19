//! Section component - refined card with accent header
//!
//! Clean geometric cards with subtle depth

use freya::prelude::*;

use crate::ui::theme::*;

/// Create a section card with accent-colored header label
pub fn section(title: &str, content: impl IntoElement) -> impl IntoElement {
    let title = title.to_uppercase();

    rect()
        .width(Size::fill())
        .background(BG_SURFACE)
        .corner_radius(RADIUS_LG)
        .padding((SPACING_XL, SPACING_2XL, SPACING_2XL, SPACING_2XL))
        .spacing(SPACING_XL)
        .child(
            // Section header with accent underline
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                .child(
                    label()
                        .text(title)
                        .color(ACCENT_VIVID)
                        .font_size(FONT_SIZE_2XS)
                        .font_weight(FontWeight::BOLD)
                        .max_lines(1),
                )
                .child(
                    // Accent underline
                    rect()
                        .width(Size::px(32.0))
                        .height(Size::px(2.0))
                        .corner_radius(RADIUS_FULL)
                        .background(ACCENT_MUTED),
                ),
        )
        .child(content)
}

/// Create a section with just a subtle header (no accent line)
pub fn section_minimal(title: &str, content: impl IntoElement) -> impl IntoElement {
    let title = title.to_string();

    rect()
        .width(Size::fill())
        .background(BG_SURFACE)
        .corner_radius(RADIUS_LG)
        .padding((SPACING_XL, SPACING_2XL, SPACING_2XL, SPACING_2XL))
        .spacing(SPACING_LG)
        .child(
            label()
                .text(title)
                .color(TEXT_SOFT)
                .font_size(FONT_SIZE_SM)
                .font_weight(FontWeight::MEDIUM)
                .max_lines(1),
        )
        .child(content)
}
