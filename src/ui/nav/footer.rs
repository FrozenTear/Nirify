//! Footer with status message and close button

use floem::prelude::*;
use floem::views::{Button, Label, Stack};

use crate::ui::theme::{
    footer_style, BORDER_RADIUS_SM, OVERLAY0, SPACING_MD, SPACING_SM, SUBTEXT1, SURFACE1,
};

/// Create the footer with status and close button
pub fn footer() -> impl IntoView {
    Stack::horizontal((
        // Status message
        Label::derived(|| "Changes saved automatically".to_string())
            .style(|s| s.color(OVERLAY0).font_size(12.0)),
        // Spacer
        Stack::horizontal(()).style(|s| s.flex_grow(1.0)),
        // Close button
        Button::new("Close").style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .border_radius(BORDER_RADIUS_SM)
                .background(SURFACE1)
                .color(SUBTEXT1)
        }),
    ))
    .style(footer_style)
}
