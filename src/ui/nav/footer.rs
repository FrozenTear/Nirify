//! Footer component with status message and close button

use freya::prelude::*;

use crate::ui::theme::*;

/// Create the footer with status indicator and close button
pub fn footer() -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .padding((SPACING_LG, SPACING_2XL, SPACING_LG, SPACING_2XL))
        .background(MANTLE)
        .child(
            // Status indicator
            rect()
                .horizontal()
                .spacing(SPACING_SM)
                .child(rect().color(SUCCESS).font_size(FONT_SIZE_SM).child("✓"))
                .child(
                    rect()
                        .color(TEXT_GHOST)
                        .font_size(FONT_SIZE_SM)
                        .child("Changes saved automatically"),
                ),
        )
        .child(
            // Spacer
            rect().width(Size::fill()),
        )
        .child(
            // Close button
            Button::new()
                .on_press(|_| {
                    std::process::exit(0);
                })
                .child("Close"),
        )
}
