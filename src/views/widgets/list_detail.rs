//! List-Detail View Components
//!
//! Reusable components for list-detail pattern views (window_rules, layer_rules, keybindings).
//! Provides consistent styling and layout across all list-detail views.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::theme::muted_text_container;

/// Creates the standard list-detail two-panel layout.
///
/// Layout: 1:2 ratio (list panel : detail panel)
pub fn list_detail_layout<'a, M: 'a>(
    list_panel: impl Into<Element<'a, M>>,
    detail_panel: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    row![
        container(list_panel)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(list_panel_style),
        container(detail_panel)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .padding(20),
    ]
    .spacing(0)
    .into()
}

/// Standard style for the list panel container (dark background).
/// Uses theme background with slight darkening for depth.
pub fn list_panel_style(theme: &iced::Theme) -> container::Style {
    let bg = theme.palette().background;
    // Darken the background slightly for the list panel
    let darkened = iced::Color {
        r: bg.r * 0.85,
        g: bg.g * 0.85,
        b: bg.b * 0.85,
        a: 0.5,
    };
    container::Style {
        background: Some(iced::Background::Color(darkened)),
        ..Default::default()
    }
}

/// Creates the list panel header with title and add button.
pub fn list_header<'a, M: Clone + 'a>(
    title: &'a str,
    on_add: M,
) -> Element<'a, M> {
    row![
        text(title).size(18),
        add_button(on_add),
    ]
    .spacing(10)
    .padding([12, 20])
    .align_y(Alignment::Center)
    .into()
}

/// Standard "+" add button with blue styling.
pub fn add_button<'a, M: Clone + 'a>(on_press: M) -> Element<'a, M> {
    button(text("+").size(18))
        .on_press(on_press)
        .padding([4, 12])
        .style(add_button_style)
        .into()
}

/// Style function for the add button.
/// Uses theme's primary color for a consistent accent.
pub fn add_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let primary = theme.palette().primary;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.6, ..primary },
        button::Status::Pressed => iced::Color { a: 0.7, ..primary },
        _ => iced::Color { a: 0.5, ..primary },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: theme.palette().text,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Creates an action button (duplicate, reorder, etc.) with neutral styling.
pub fn action_button<'a, M: Clone + 'a>(
    label: &'a str,
    on_press: M,
) -> Element<'a, M> {
    button(text(label).size(13))
        .on_press(on_press)
        .padding([8, 12])
        .style(action_button_style)
        .into()
}

/// Style function for neutral action buttons.
/// Uses theme background colors for subtle appearance.
pub fn action_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg_base = theme.palette().background;
    let bg = match status {
        button::Status::Hovered => iced::Color { r: bg_base.r + 0.15, g: bg_base.g + 0.15, b: bg_base.b + 0.15, a: 0.5 },
        button::Status::Pressed => iced::Color { r: bg_base.r + 0.20, g: bg_base.g + 0.20, b: bg_base.b + 0.20, a: 0.5 },
        _ => iced::Color { r: bg_base.r + 0.10, g: bg_base.g + 0.10, b: bg_base.b + 0.10, a: 0.4 },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: theme.palette().text,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Creates a delete button with red styling.
pub fn delete_button<'a, M: Clone + 'a>(on_press: M) -> Element<'a, M> {
    button(text("Delete").size(13))
        .on_press(on_press)
        .padding([8, 12])
        .style(delete_button_style)
        .into()
}

/// Style function for delete buttons.
/// Uses theme's danger color for warning appearance.
pub fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.6, ..danger },
        button::Status::Pressed => iced::Color { a: 0.7, ..danger },
        _ => iced::Color { a: 0.5, ..danger },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: theme.palette().text,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Creates a small remove button (×) for removing items in lists.
pub fn remove_button<'a, M: Clone + 'a>(on_press: M) -> Element<'a, M> {
    button(text("×").size(14))
        .on_press(on_press)
        .padding([2, 8])
        .style(remove_button_style)
        .into()
}

/// Style function for remove buttons.
/// Uses theme's danger color for text.
pub fn remove_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.5, ..danger },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color { a: 0.8, ..danger },
        ..Default::default()
    }
}

/// Style function for list item buttons (selection-aware).
/// Uses theme's primary color for selected state.
pub fn list_item_style(is_selected: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme, status| {
        let primary = theme.palette().primary;
        let bg_base = theme.palette().background;

        let background = match (is_selected, status) {
            (true, button::Status::Hovered) => iced::Color { a: 0.5, ..primary },
            (true, button::Status::Pressed) => iced::Color { a: 0.6, ..primary },
            (true, _) => iced::Color { a: 0.4, ..primary },
            (false, button::Status::Hovered) => iced::Color { r: bg_base.r + 0.1, g: bg_base.g + 0.1, b: bg_base.b + 0.1, a: 0.5 },
            (false, button::Status::Pressed) => iced::Color { r: bg_base.r + 0.15, g: bg_base.g + 0.15, b: bg_base.b + 0.15, a: 0.5 },
            (false, _) => iced::Color::TRANSPARENT,
        };

        button::Style {
            background: Some(iced::Background::Color(background)),
            border: iced::Border::default(),
            text_color: theme.palette().text,
            ..Default::default()
        }
    }
}

/// Creates an empty list placeholder message.
/// Uses muted text via container style for theme awareness.
pub fn empty_list_placeholder<'a, M: 'a>(
    message: &'a str,
) -> Element<'a, M> {
    container(
        container(text(message).size(13).center()).style(muted_text_container)
    )
    .padding(20)
    .center(Length::Fill)
    .into()
}

/// Creates an empty detail view placeholder.
/// Uses muted text via container style for theme awareness.
pub fn empty_detail_placeholder<'a, M: 'a>(
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, M> {
    container(
        column![
            container(text(title).size(16)).style(muted_text_container),
            super::spacer(8.0),
            container(text(subtitle).size(13)).style(|theme: &iced::Theme| {
                let txt = theme.palette().text;
                container::Style {
                    text_color: Some(iced::Color { a: 0.35, ..txt }),
                    ..Default::default()
                }
            }),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

/// Creates a selection indicator (● or ○).
/// Uses theme-aware colors via container styling.
pub fn selection_indicator<'a, M: 'a>(is_selected: bool) -> Element<'a, M> {
    container(
        text(if is_selected { "●" } else { "○" })
            .size(12)
            .width(Length::Fixed(20.0))
    )
    .style(move |theme: &iced::Theme| {
        let color = if is_selected {
            theme.palette().primary
        } else {
            let txt = theme.palette().text;
            iced::Color { a: 0.5, ..txt }
        };
        container::Style {
            text_color: Some(color),
            ..Default::default()
        }
    })
    .into()
}

/// Creates a small badge container (for status indicators like "max", "float", etc.).
/// Uses theme text color for badge text.
pub fn badge<'a, M: 'a>(
    label: &'a str,
    color: iced::Color,
) -> Element<'a, M> {
    container(text(label).size(10))
        .padding([2, 6])
        .style(move |theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(color)),
            text_color: Some(theme.palette().text),
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Standard badge color for behavior/status (purple-ish).
pub const BADGE_BEHAVIOR: iced::Color = iced::Color {
    r: 0.4,
    g: 0.3,
    b: 0.6,
    a: 0.4,
};

/// Standard badge color for visibility/blocking (red-ish).
pub const BADGE_VISIBILITY: iced::Color = iced::Color {
    r: 0.6,
    g: 0.3,
    b: 0.3,
    a: 0.4,
};

/// Container style for match criteria blocks.
/// Uses theme background colors for subtle appearance.
pub fn match_container_style(theme: &iced::Theme) -> container::Style {
    let bg = theme.palette().background;
    let bg_lighter = iced::Color {
        r: bg.r + 0.05,
        g: bg.g + 0.05,
        b: bg.b + 0.05,
        a: 0.4,
    };
    let border_color = iced::Color {
        r: bg.r + 0.15,
        g: bg.g + 0.15,
        b: bg.b + 0.15,
        a: 0.5,
    };
    container::Style {
        background: Some(iced::Background::Color(bg_lighter)),
        border: iced::Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Creates an "Add X" button for adding items within sections.
pub fn add_item_button<'a, M: Clone + 'a>(
    label: &'a str,
    on_press: M,
) -> Element<'a, M> {
    button(
        row![text("+").size(14), text(label).size(13),].spacing(6),
    )
    .on_press(on_press)
    .padding([8, 16])
    .style(add_item_button_style)
    .into()
}

/// Style for "Add X" buttons within sections.
/// Uses theme colors for consistent appearance.
pub fn add_item_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let primary = theme.palette().primary;
    let bg_base = theme.palette().background;
    let bg = match status {
        button::Status::Hovered => iced::Color { r: bg_base.r + 0.15, g: bg_base.g + 0.15, b: bg_base.b + 0.15, a: 0.4 },
        _ => iced::Color { r: bg_base.r + 0.10, g: bg_base.g + 0.10, b: bg_base.b + 0.10, a: 0.3 },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color { a: 0.9, ..primary },
        border: iced::Border {
            color: iced::Color { a: 0.3, ..primary },
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
