//! Footer component - minimal status bar
//!
//! Clean, understated footer with status indicator

use freya::prelude::*;

use crate::ui::theme::*;

/// Create a minimal footer with status indicator
pub fn footer() -> impl IntoElement {
    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(FOOTER_HEIGHT))
        .padding((0.0, SPACING_3XL, 0.0, SPACING_3XL))
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .background(BG_DEEP)
        .child(
            // Status indicator - left aligned
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(SPACING_SM)
                .child(
                    // Status dot
                    rect()
                        .width(Size::px(6.0))
                        .height(Size::px(6.0))
                        .corner_radius(RADIUS_FULL)
                        .background(SUCCESS),
                )
                .child(
                    label()
                        .text("Auto-save enabled")
                        .color(TEXT_GHOST)
                        .font_size(FONT_SIZE_XS)
                        .max_lines(1),
                ),
        )
        .child(
            // Close button - right aligned, minimal
            rect()
                .content(Content::flex())
                .padding((SPACING_SM, SPACING_LG, SPACING_SM, SPACING_LG))
                .corner_radius(RADIUS_MD)
                .on_pointer_down(|_| {
                    std::process::exit(0);
                })
                .child(
                    label()
                        .text("Close")
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        )
}
