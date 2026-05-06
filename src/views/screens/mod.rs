//! Screen views for the redesigned UI
//!
//! Each screen consolidates multiple old pages into a single cohesive view.

pub mod dashboard;
pub mod displays;
pub mod gear;
pub mod input;
pub mod layout;
pub mod rules;
pub mod system;
pub mod visuals;

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::theme::{fonts, neon};

/// Creates a consistent screen header: large title + subtitle
pub fn screen_header<'a, Message: 'a>(title: &'a str, subtitle: &'a str) -> Element<'a, Message> {
    column![
        text(title).size(28).font(fonts::UI_FONT_SEMIBOLD),
        text(subtitle).size(14).color(neon::ON_SURFACE_VARIANT),
    ]
    .spacing(4)
    .into()
}

/// Hero header with category label, accent line, large title, and description
pub fn hero_header<'a, Message: 'a>(
    category: &'a str,
    title: &'a str,
    description: &'a str,
    accent: iced::Color,
) -> Element<'a, Message> {
    column![
        row![
            text(category)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(accent),
            Space::new().width(12),
            container(Space::new().width(Length::Fill).height(1))
                .width(Length::Fill)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.3, ..accent })),
                    ..Default::default()
                }),
        ]
        .align_y(Alignment::Center),
        text(title).size(40).font(fonts::UI_FONT_SEMIBOLD),
        text(description).size(14).color(neon::ON_SURFACE_VARIANT),
    ]
    .spacing(8)
    .into()
}

/// Neon section card for masonry grids
pub fn neon_section<'a, Message: 'a>(
    title: &'a str,
    subtitle: &'a str,
    _accent: iced::Color,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            text(title).size(16).font(fonts::UI_FONT_SEMIBOLD),
            text(subtitle).size(11).color(neon::ON_SURFACE_VARIANT),
            Space::new().height(8),
            content,
        ]
        .spacing(4),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Shrink)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 20.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    })
    .into()
}

/// Wraps any content Element in a standard section editor modal
pub fn section_editor_modal<'a>(
    section: crate::messages::EditableSection,
    content: Element<'a, crate::messages::Message>,
) -> Element<'a, crate::messages::Message> {
    use crate::messages::Message;
    use iced::widget::{button, scrollable};

    let accent = section.accent();
    let icon = section.icon();
    let name = section.name();

    let editor = column![
        // Header
        row![
            container(text(icon).size(24).color(accent))
                .width(48)
                .height(48)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.15, ..accent })),
                    border: iced::Border {
                        radius: 14.0.into(),
                        color: iced::Color { a: 0.25, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("SECTION EDITOR")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(accent),
                text(name).size(22).font(fonts::UI_FONT_SEMIBOLD),
            ]
            .spacing(4)
            .width(Length::Fill),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::CloseSectionEditor)
                .padding([8, 12])
                .style(|_: &iced::Theme, status: iced::widget::button::Status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => iced::Color {
                            a: 0.15,
                            ..neon::ON_SURFACE
                        },
                        _ => iced::Color {
                            a: 0.08,
                            ..neon::ON_SURFACE
                        },
                    };
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: neon::ON_SURFACE,
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center),
        Space::new().height(16),
        content,
        // Footer
        Space::new().height(16),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        container(
            row![
                row![
                    text("●").size(10).color(neon::SECONDARY),
                    text("Live Configuration Sync Active")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill),
                button(text("Close").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::CloseSectionEditor)
                    .padding([10, 24])
                    .style(|_: &iced::Theme, status: iced::widget::button::Status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => neon::PRIMARY,
                            _ => iced::Color {
                                a: 0.85,
                                ..neon::PRIMARY
                            },
                        };
                        iced::widget::button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::SURFACE_LOW,
                            border: iced::Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .align_y(iced::Alignment::Center)
        )
        .padding([16, 0]),
    ];

    let modal_content = scrollable(editor.spacing(0).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(850.0))
        .max_height(700.0)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color { a: 0.3, ..accent },
                width: 2.0,
                radius: 20.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}

/// Creates a sub-tab bar for screens with multiple sections
pub fn sub_tab_bar<'a, T, Message>(
    tabs: &'a [T],
    active: T,
    on_select: impl Fn(T) -> Message + 'a + Copy,
) -> Element<'a, Message>
where
    T: Copy + PartialEq + 'a,
    T: TabLabel,
    Message: Clone + 'a,
{
    use iced::widget::{button, row, scrollable};

    let tab_row = row(tabs.iter().map(|tab| {
        let is_active = active == *tab;
        button(text(tab.label()).size(13).font(if is_active {
            fonts::UI_FONT_MEDIUM
        } else {
            fonts::UI_FONT
        }))
        .on_press(on_select(*tab))
        .padding([8, 16])
        .style(
            move |_theme: &iced::Theme, status: iced::widget::button::Status| {
                let (bg, text_color) = if is_active {
                    (
                        iced::Color {
                            a: 0.15,
                            ..neon::PRIMARY
                        },
                        neon::PRIMARY,
                    )
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => (
                            iced::Color {
                                a: 0.08,
                                ..neon::ON_SURFACE
                            },
                            neon::ON_SURFACE,
                        ),
                        _ => (iced::Color::TRANSPARENT, neon::ON_SURFACE_VARIANT),
                    }
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color,
                    border: iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            },
        )
        .into()
    }))
    .spacing(4);

    scrollable(tab_row)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default().width(0).scroller_width(0),
        ))
        .into()
}

/// Trait for tab labels
pub trait TabLabel {
    fn label(&self) -> &'static str;
}
