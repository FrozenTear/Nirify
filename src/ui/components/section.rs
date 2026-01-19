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

/// Create a collapsible section with clickable header
pub fn collapsible_section(
    title: &str,
    expanded: bool,
    mut on_toggle: impl FnMut() + 'static,
    content: impl IntoElement,
) -> Element {
    let title_upper = title.to_uppercase();
    let arrow = if expanded { "▼" } else { "▶" };

    let mut container = rect()
        .width(Size::fill())
        .background(BG_SURFACE)
        .corner_radius(RADIUS_LG)
        .padding((SPACING_XL, SPACING_2XL, SPACING_2XL, SPACING_2XL))
        .spacing(SPACING_XL)
        .child(
            // Clickable header
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .spacing(SPACING_SM)
                .on_pointer_down(move |_| {
                    on_toggle();
                })
                .child(
                    // Arrow indicator
                    label()
                        .text(arrow)
                        .color(ACCENT_MUTED)
                        .font_size(FONT_SIZE_XS),
                )
                .child(
                    // Section title
                    rect()
                        .spacing(SPACING_SM)
                        .child(
                            label()
                                .text(title_upper)
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
                ),
        );

    // Add content only if expanded
    if expanded {
        container = container.child(content);
    }

    container.into()
}

/// Create a collapsible section that starts collapsed by default (no background)
pub fn collapsible_section_minimal(
    title: &str,
    expanded: bool,
    mut on_toggle: impl FnMut() + 'static,
    content: impl IntoElement,
) -> Element {
    let title = title.to_string();
    let arrow = if expanded { "▼" } else { "▶" };

    let mut container = rect()
        .width(Size::fill())
        .spacing(SPACING_MD)
        .child(
            // Clickable header
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .width(Size::fill())
                .cross_align(Alignment::Center)
                .spacing(SPACING_SM)
                .padding((SPACING_SM, 0.0, SPACING_SM, 0.0))
                .on_pointer_down(move |_| {
                    on_toggle();
                })
                .child(
                    // Arrow indicator
                    label()
                        .text(arrow)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_XS),
                )
                .child(
                    // Section title
                    label()
                        .text(title)
                        .color(TEXT_SOFT)
                        .font_size(FONT_SIZE_SM)
                        .font_weight(FontWeight::MEDIUM)
                        .max_lines(1),
                ),
        );

    // Add content only if expanded
    if expanded {
        container = container.child(
            rect()
                .padding((0.0, 0.0, 0.0, SPACING_LG))
                .child(content),
        );
    }

    container.into()
}
