//! List-Detail View Components
//!
//! Reusable components for list-detail pattern views (window_rules, layer_rules, keybindings).
//! Provides consistent styling and layout across all list-detail views.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

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
pub fn list_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.1, 0.1, 0.1, 0.5,
        ))),
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
pub fn add_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
        _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
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
pub fn action_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.5),
        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.5, 0.6, 0.5),
        _ => iced::Color::from_rgba(0.25, 0.3, 0.35, 0.4),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
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
pub fn delete_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.7, 0.2, 0.2, 0.6),
        button::Status::Pressed => iced::Color::from_rgba(0.8, 0.3, 0.3, 0.7),
        _ => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
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
pub fn remove_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
        ..Default::default()
    }
}

/// Style function for list item buttons (selection-aware).
pub fn list_item_style(is_selected: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let background = match (is_selected, status) {
            (true, button::Status::Hovered) => iced::Color::from_rgba(0.3, 0.4, 0.6, 0.5),
            (true, button::Status::Pressed) => iced::Color::from_rgba(0.4, 0.5, 0.7, 0.5),
            (true, _) => iced::Color::from_rgba(0.2, 0.3, 0.5, 0.4),
            (false, button::Status::Hovered) => iced::Color::from_rgba(0.25, 0.25, 0.25, 0.5),
            (false, button::Status::Pressed) => iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
            (false, _) => iced::Color::TRANSPARENT,
        };

        button::Style {
            background: Some(iced::Background::Color(background)),
            border: iced::Border::default(),
            text_color: iced::Color::WHITE,
            ..Default::default()
        }
    }
}

/// Creates an empty list placeholder message.
pub fn empty_list_placeholder<'a, M: 'a>(
    message: &'a str,
) -> Element<'a, M> {
    container(
        text(message)
            .size(13)
            .color([0.75, 0.75, 0.75])
            .center(),
    )
    .padding(20)
    .center(Length::Fill)
    .into()
}

/// Creates an empty detail view placeholder.
pub fn empty_detail_placeholder<'a, M: 'a>(
    title: &'a str,
    subtitle: &'a str,
) -> Element<'a, M> {
    container(
        column![
            text(title)
                .size(16)
                .color([0.75, 0.75, 0.75]),
            super::spacer(8.0),
            text(subtitle)
                .size(13)
                .color([0.5, 0.5, 0.5]),
        ]
        .spacing(4)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

/// Creates a selection indicator (● or ○).
pub fn selection_indicator<'a, M: 'a>(is_selected: bool) -> Element<'a, M> {
    text(if is_selected { "●" } else { "○" })
        .size(12)
        .width(Length::Fixed(20.0))
        .color(if is_selected {
            [0.5, 0.7, 1.0]
        } else {
            [0.5, 0.5, 0.5]
        })
        .into()
}

/// Creates a small badge container (for status indicators like "max", "float", etc.).
pub fn badge<'a, M: 'a>(
    label: &'a str,
    color: iced::Color,
) -> Element<'a, M> {
    container(text(label).size(10).color([0.9, 0.9, 0.9]))
        .padding([2, 6])
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(color)),
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
pub fn match_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.15, 0.15, 0.15, 0.4,
        ))),
        border: iced::Border {
            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
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
pub fn add_item_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.4),
        _ => iced::Color::from_rgba(0.2, 0.25, 0.3, 0.3),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::from_rgb(0.7, 0.8, 0.9),
        border: iced::Border {
            color: iced::Color::from_rgba(0.4, 0.5, 0.6, 0.3),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
