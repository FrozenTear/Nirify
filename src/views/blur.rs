//! Top-level background blur settings view (niri 26.04+) — neon modal style

use iced::widget::{column, container, row, scrollable, text, toggler, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::BlurSettings;
use crate::messages::{BlurMessage, Message};
use crate::theme::{fonts, neon};

pub fn view(settings: &BlurSettings, niri_supports_blur: bool) -> Element<'static, Message> {
    let enabled = settings.enabled;
    let passes = settings.passes;
    let offset = settings.offset;
    let noise = settings.noise;
    let saturation = settings.saturation;

    let mut content = column![].spacing(4);

    if !niri_supports_blur {
        content = content.push(notice(
            "Requires niri 26.04 or newer. Your niri version does not support background blur — \
             settings are kept here but blur.kdl is not written.",
        ));
        content = content.push(Space::new().height(12));
    }

    content = content.push(modal_section("◍", "STATUS", neon::SECONDARY));
    content = content.push(
        container(column![toggle_row(
            "Enable blur",
            "Blurs the background behind windows and panels that request it, or that have blur \
             forced via window/layer rules.",
            enabled,
            |v| Message::Blur(BlurMessage::SetEnabled(v)),
        )])
        .padding(8)
        .style(crate::theme::card_style),
    );

    content = content.push(Space::new().height(12));
    content = content.push(modal_section("▤", "QUALITY", neon::PRIMARY));
    content = content.push(Space::new().height(4));
    content = content.push(styled_slider_int(
        "PASSES",
        &format!("{}", passes),
        0..=8,
        passes,
        |v| Message::Blur(BlurMessage::SetPasses(v)),
    ));
    content = content.push(super::widgets::info_text(
        "More passes = smoother, heavier on GPU. Increase Offset first.",
    ));
    content = content.push(styled_slider_f64(
        "OFFSET",
        &format!("{:.1}", offset),
        0.0..=30.0,
        0.5,
        offset,
        |v| Message::Blur(BlurMessage::SetOffset(v)),
    ));

    content = content.push(Space::new().height(12));
    content = content.push(modal_section("◉", "APPEARANCE", neon::TERTIARY));
    content = content.push(Space::new().height(4));
    content = content.push(styled_slider_f64(
        "NOISE",
        &format!("{:.2}", noise),
        0.0..=1.0,
        0.01,
        noise,
        |v| Message::Blur(BlurMessage::SetNoise(v)),
    ));
    content = content.push(super::widgets::info_text("Reduces color banding."));
    content = content.push(styled_slider_f64(
        "SATURATION",
        &format!("{:.2}", saturation),
        0.0..=3.0,
        0.05,
        saturation,
        |v| Message::Blur(BlurMessage::SetSaturation(v)),
    ));

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

// ── Helpers ──

fn notice(msg: &str) -> Element<'static, Message> {
    let owned = msg.to_string();
    container(
        row![
            text("⚠").size(16).color(neon::ERROR),
            Space::new().width(10),
            text(owned).size(12).color(neon::ON_SURFACE_VARIANT),
        ]
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color {
            a: 0.10,
            ..neon::ERROR
        })),
        border: iced::Border {
            radius: 8.0.into(),
            color: iced::Color {
                a: 0.35,
                ..neon::ERROR
            },
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn modal_section(icon: &str, label: &str, accent: iced::Color) -> Element<'static, Message> {
    let icon = icon.to_string();
    let label = label.to_string();
    row![
        text(icon).size(14).color(accent),
        Space::new().width(6),
        text(label)
            .size(11)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(accent),
        Space::new().width(12),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.25, ..accent })),
                ..Default::default()
            }),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding([14, 0])
    .into()
}

fn toggle_row(
    label: &str,
    desc: &str,
    value: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<'static, Message> {
    let label = label.to_string();
    let desc = desc.to_string();
    row![
        column![
            text(label).size(14).font(fonts::UI_FONT_MEDIUM),
            text(desc).size(11).color(neon::ON_SURFACE_VARIANT),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value).on_toggle(on_toggle),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}

fn styled_slider_int(
    label: &str,
    display: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'static,
) -> Element<'static, Message> {
    let label = label.to_string();
    let display = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(display)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide).width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn styled_slider_f64(
    label: &str,
    display: &str,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    value: f64,
    on_slide: impl Fn(f64) -> Message + 'static,
) -> Element<'static, Message> {
    let label = label.to_string();
    let display = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(display)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
