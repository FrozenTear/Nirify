//! Footer with status message and close button

use floem::prelude::*;
use floem::views::{Button, Label, Stack};

use crate::ui::theme::{
    footer_style, BG_ELEVATED, BORDER, BORDER_RADIUS_SM, SPACING_MD, SPACING_SM, TEXT_MUTED,
    TEXT_SECONDARY,
};

/// Create the footer with status and close button
pub fn footer() -> impl IntoView {
    Stack::horizontal((
        // Status message
        Label::derived(|| "Changes saved automatically".to_string())
            .style(|s| s.color(TEXT_MUTED).font_size(12.0)),
        // Spacer
        Stack::horizontal(()).style(|s| s.flex_grow(1.0)),
        // Close button
        Button::new("Close").style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .border_radius(BORDER_RADIUS_SM)
                .background(BG_ELEVATED)
                .color(TEXT_SECONDARY)
                .border(1.0)
                .border_color(BORDER)
        }),
    ))
    .style(footer_style)
}
