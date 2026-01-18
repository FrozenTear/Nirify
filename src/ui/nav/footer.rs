//! Footer component with status message and close button
//!
//! Shows auto-save status and provides app-level actions

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::theme::{button_secondary_style, footer_style, FONT_SIZE_SM, SUCCESS, TEXT_GHOST};

/// Create the footer with status indicator and close button
pub fn footer() -> impl IntoView {
    Stack::horizontal((
        // Status indicator with checkmark
        Stack::horizontal((
            Label::derived(|| "✓".to_string()).style(|s| s.color(SUCCESS).font_size(FONT_SIZE_SM)),
            Label::derived(|| "Changes saved automatically".to_string())
                .style(|s| s.color(TEXT_GHOST).font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.gap(8.0).items_center()),
        // Spacer
        Stack::horizontal(()).style(|s| s.flex_grow(1.0)),
        // Close button
        Label::derived(|| "Close".to_string())
            .style(button_secondary_style)
            .on_click_stop(|_| {
                // Use Floem's proper quit mechanism for graceful shutdown
                floem::quit_app();
            }),
    ))
    .style(footer_style)
}
