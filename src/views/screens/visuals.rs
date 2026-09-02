//! Visuals screen — summary cards + modal editors

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::{
    AnimationSettings, AppearanceSettings, BehaviorSettings, BlurSettings, CursorSettings,
    LayoutExtrasSettings, OverviewSettings,
};
use crate::messages::{EditableSection, Message};
use crate::theme::{fonts, neon};

pub fn view<'a>(
    appearance: &'a AppearanceSettings,
    animations: &'a AnimationSettings,
    cursor: &'a CursorSettings,
    layout_extras: &'a LayoutExtrasSettings,
    behavior: &'a BehaviorSettings,
    blur: &'a BlurSettings,
    blur_supported: bool,
    overview: &'a OverviewSettings,
) -> Element<'a, Message> {
    let content = column![
        super::hero_header(
            "VISUAL ENGINE",
            "Surface & Motion",
            "Control how windows present themselves visually — focus indicators, borders, shadows, workspace background, overview, animations, and cursor behavior.",
            neon::PRIMARY,
        ),
        Space::new().height(24),
        // Row 1
        row![
            summary_card(EditableSection::FocusRing, vec![
                ("Enabled", if appearance.focus_ring_enabled { "On" } else { "Off" }.to_string()),
                ("Width", format!("{}px", appearance.focus_ring_width)),
            ]),
            summary_card(EditableSection::WindowBorder, vec![
                ("Enabled", if appearance.border_enabled { "On" } else { "Off" }.to_string()),
                ("Width", format!("{:.0}px", appearance.border_thickness)),
            ]),
            summary_card(EditableSection::WindowShadow, vec![
                ("Enabled", if layout_extras.shadow.enabled { "On" } else { "Off" }.to_string()),
                ("Softness", format!("{}px", layout_extras.shadow.softness)),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        // Row 2
        row![
            summary_card(EditableSection::ModifierKeys, vec![
                ("Mod Key", format!("{}", behavior.mod_key)),
                ("Power Key", if behavior.disable_power_key_handling { "Disabled" } else { "Enabled" }.to_string()),
            ]),
            summary_card(EditableSection::Animations, vec![
                ("Count", "11 types".to_string()),
                ("Slowdown", format!("{:.1}x", animations.slowdown)),
            ]),
            summary_card(EditableSection::Cursor, vec![
                ("Theme", cursor.theme.clone()),
                ("Size", format!("{}", cursor.size)),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        // Row 3 — blur, workspace background, overview (browse, not search-only)
        row![
            summary_card(EditableSection::Blur, vec![
                ("Enabled", if blur.enabled { "On" } else { "Off" }.to_string()),
                if blur_supported {
                    ("Passes", format!("{}", blur.passes))
                } else {
                    ("Requires", "niri 26.04+".to_string())
                },
            ]),
            summary_card(EditableSection::WorkspaceBackground, vec![
                ("Color", appearance.background_color
                    .map(|c| c.to_hex())
                    .unwrap_or_else(|| "Default".to_string())),
                ("Override", if appearance.background_color.is_some() { "On" } else { "Off" }.to_string()),
            ]),
            summary_card(EditableSection::Overview, vec![
                ("Zoom", format!("{:.2}x", overview.zoom)),
                ("Shadow", if overview.workspace_shadow.as_ref().is_some_and(|s| s.enabled) {
                    "On"
                } else {
                    "Off"
                }.to_string()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
    ]
    .spacing(0)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

fn summary_card<'a>(
    section: EditableSection,
    summary: Vec<(&'static str, String)>,
) -> Element<'a, Message> {
    let accent = section.accent();
    let icon = section.icon();
    let name = section.name();

    let mut summary_items = column![].spacing(4);
    for (label, value) in summary {
        summary_items = summary_items.push(
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(value).size(11).font(fonts::MONO_FONT).color(accent),
            ]
            .align_y(Alignment::Center),
        );
    }

    container(column![
        row![
            container(text(icon).size(16).color(accent))
                .width(36)
                .height(36)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.12, ..accent })),
                    border: iced::Border {
                        radius: 10.0.into(),
                        color: iced::Color { a: 0.2, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(10),
            text(name).size(14).font(fonts::UI_FONT_SEMIBOLD),
        ]
        .align_y(Alignment::Center),
        Space::new().height(10),
        summary_items,
        Space::new().height(10),
        iced::widget::button(
            text("CONFIGURE")
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(accent)
        )
        .on_press(Message::OpenSectionEditor(section))
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => iced::Color { a: 0.15, ..accent },
                _ => iced::Color { a: 0.08, ..accent },
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: accent,
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color { a: 0.2, ..accent },
                    width: 1.0,
                },
                ..Default::default()
            }
        }),
    ])
    .padding(16)
    .width(Length::FillPortion(1))
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
        border: iced::Border {
            color: iced::Color { a: 0.12, ..accent },
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color { a: 0.08, ..accent },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    })
    .into()
}
