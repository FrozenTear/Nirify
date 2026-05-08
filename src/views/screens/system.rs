//! System screen — summary cards + modal editors

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::{
    DebugSettings, EnvironmentSettings, MiscSettings, RecentWindowsSettings, StartupSettings,
    SwitchEventsSettings,
};
use crate::messages::{EditableSection, Message};
use crate::theme::{fonts, neon};

pub fn view<'a>(
    startup: &'a StartupSettings,
    environment: &'a EnvironmentSettings,
    miscellaneous: &'a MiscSettings,
    switch_events: &'a SwitchEventsSettings,
    debug: &'a DebugSettings,
    recent_windows: &'a RecentWindowsSettings,
) -> Element<'a, Message> {
    let content = column![
        super::hero_header(
            "SYSTEM CORE",
            "Under the Hood",
            "Startup programs, environment variables, switch events, and advanced debugging options for the compositor.",
            neon::TERTIARY,
        ),
        Space::new().height(24),
        // Row 1
        row![
            summary_card(EditableSection::StartupPrograms, vec![
                ("Commands", format!("{}", startup.commands.len())),
                ("Status", if startup.commands.is_empty() { "None" } else { "Active" }.to_string()),
            ]),
            summary_card(EditableSection::EnvironmentVars, vec![
                ("Variables", format!("{}", environment.variables.len())),
                ("Status", if environment.variables.is_empty() { "None" } else { "Active" }.to_string()),
            ]),
            summary_card(EditableSection::Miscellaneous, vec![
                ("CSD", if miscellaneous.prefer_no_csd { "Off" } else { "On" }.to_string()),
                ("Clipboard", if miscellaneous.disable_primary_clipboard { "Off" } else { "On" }.to_string()),
            ]),
        ].spacing(12).align_y(Alignment::Start),
        Space::new().height(12),
        // Row 2
        row![
            summary_card(EditableSection::SwitchEvents, vec![
                ("Lid Close", if switch_events.lid_close.has_action() { "Set" } else { "None" }.to_string()),
                ("Tablet Mode", if switch_events.tablet_mode_on.has_action() { "Set" } else { "None" }.to_string()),
            ]),
            summary_card(EditableSection::Debug, vec![
                ("Expert", if debug.expert_mode { "On" } else { "Off" }.to_string()),
                ("Render", if debug.disable_direct_scanout { "Software" } else { "Direct" }.to_string()),
            ]),
            summary_card(EditableSection::RecentWindows, vec![
                ("Enabled", if recent_windows.off { "Off" } else { "On" }.to_string()),
                ("Delay", format!("{}ms", recent_windows.open_delay_ms)),
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
