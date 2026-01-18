//! Footer component with status message and close button

use freya::prelude::*;

use crate::ui::theme::*;

/// Create the footer with status indicator and close button
pub fn footer() -> impl IntoElement {
    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .padding((SPACING_LG, SPACING_2XL, SPACING_LG, SPACING_2XL))
        .background(MANTLE)
        .child(
            // Status indicator
            rect()
                .direction(Direction::Horizontal)
                .spacing(SPACING_SM)
                .child(label().text("✓").color(SUCCESS).font_size(FONT_SIZE_SM).max_lines(1))
                .child(label().text("Changes saved automatically").color(TEXT_GHOST).font_size(FONT_SIZE_SM).max_lines(1)),
        )
        .child(
            // Spacer
            rect().width(Size::flex(1.0)),
        )
        .child(
            // Close button
            Button::new()
                .on_press(|_| {
                    std::process::exit(0);
                })
                .child(label().text("Close").max_lines(1)),
        )
}
