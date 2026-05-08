//! Animations settings view — neon modal style

use iced::widget::{button, column, container, pick_list, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use super::widgets::toggle_row;
use crate::config::models::{AnimationId, AnimationSettings, AnimationType, SingleAnimationConfig};
use crate::messages::{AnimationsMessage, Message};
use crate::theme::{fonts, neon};

/// Animation type options for the dropdown
const ANIMATION_TYPES: [&str; 5] = ["Default", "Off", "Spring", "Easing", "Custom Shader"];
const ANIMATION_TYPES_NO_SHADER: [&str; 4] = ["Default", "Off", "Spring", "Easing"];

/// Creates the animations settings view
pub fn view(settings: &AnimationSettings) -> Element<'_, Message> {
    let slowdown_enabled = (settings.slowdown - 1.0).abs() > 0.01;

    let content = column![
        // ── GLOBAL SETTINGS ──
        modal_section("\u{26A1}", "GLOBAL SETTINGS", neon::PRIMARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Enable slowdown",
                "Slow down all animations for debugging or effect",
                slowdown_enabled,
                |enabled| Message::Animations(AnimationsMessage::ToggleSlowdown(enabled)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        if slowdown_enabled {
            styled_slider(
                "SLOWDOWN FACTOR",
                &format!("{:.1}x", settings.slowdown),
                1.0..=10.0,
                settings.slowdown as f32,
                0.1,
                |v| Message::Animations(AnimationsMessage::SetSlowdownFactor(v)),
            )
        } else {
            column![].into()
        },
        Space::new().height(12),
        // ── 2-COLUMN: WINDOW ANIMS (SHADER) | OTHER ANIMS ──
        row![
            // Left: Window Animations (support custom shaders)
            column![
                modal_section("\u{25A3}", "WINDOW ANIMATIONS", neon::SECONDARY),
                Space::new().height(4),
                animation_card(
                    "window-open",
                    AnimationId::WindowOpen,
                    &settings.per_animation.window_open,
                    true,
                ),
                animation_card(
                    "window-close",
                    AnimationId::WindowClose,
                    &settings.per_animation.window_close,
                    true,
                ),
                animation_card(
                    "window-resize",
                    AnimationId::WindowResize,
                    &settings.per_animation.window_resize,
                    true,
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Other Animations
            column![
                modal_section("\u{25CE}", "OTHER ANIMATIONS", neon::TERTIARY),
                Space::new().height(4),
                animation_card(
                    "workspace-switch",
                    AnimationId::WorkspaceSwitch,
                    &settings.per_animation.workspace_switch,
                    false,
                ),
                animation_card(
                    "window-movement",
                    AnimationId::WindowMovement,
                    &settings.per_animation.window_movement,
                    false,
                ),
                animation_card(
                    "horizontal-view",
                    AnimationId::HorizontalViewMovement,
                    &settings.per_animation.horizontal_view_movement,
                    false,
                ),
                animation_card(
                    "overview",
                    AnimationId::Overview,
                    &settings.per_animation.overview_open_close,
                    false,
                ),
                animation_card(
                    "config-notification",
                    AnimationId::ConfigNotification,
                    &settings.per_animation.config_notification_open_close,
                    false,
                ),
                animation_card(
                    "exit-confirmation",
                    AnimationId::ExitConfirmation,
                    &settings.per_animation.exit_confirmation_open_close,
                    false,
                ),
                animation_card(
                    "screenshot-ui",
                    AnimationId::ScreenshotUi,
                    &settings.per_animation.screenshot_ui_open,
                    false,
                ),
                animation_card(
                    "recent-windows",
                    AnimationId::RecentWindows,
                    &settings.per_animation.recent_windows_close,
                    false,
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0)
    .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Creates a card for a single animation with type selector and parameters
fn animation_card<'a>(
    name: &'static str,
    id: AnimationId,
    config: &'a SingleAnimationConfig,
    supports_shader: bool,
) -> Element<'a, Message> {
    let type_index = match config.animation_type {
        AnimationType::Default => 0,
        AnimationType::Off => 1,
        AnimationType::Spring => 2,
        AnimationType::Easing => 3,
        AnimationType::CustomShader => 4,
    };

    let type_options: Vec<&str> = if supports_shader {
        ANIMATION_TYPES.to_vec()
    } else {
        ANIMATION_TYPES_NO_SHADER.to_vec()
    };

    let selected_type = type_options.get(type_index).copied();

    let name_owned = name.to_string();
    let type_selector = pick_list(type_options, selected_type, move |selected: &str| {
        let idx = match selected {
            "Default" => 0,
            "Off" => 1,
            "Spring" => 2,
            "Easing" => 3,
            "Custom Shader" => 4,
            _ => 0,
        };
        Message::Animations(AnimationsMessage::SetAnimationType(name_owned.clone(), idx))
    })
    .width(Length::Fixed(140.0));

    let mut card_content = column![row![
        text(id.name())
            .size(12)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::ON_SURFACE),
        Space::new().width(Length::Fill),
        type_selector,
    ]
    .align_y(Alignment::Center),]
    .spacing(6);

    // Show parameters based on animation type
    match config.animation_type {
        AnimationType::Spring => {
            let name_dr = name.to_string();
            let name_ep = name.to_string();
            card_content = card_content
                .push(
                    column![
                        row![
                            text("DAMPING RATIO")
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().width(Length::Fill),
                            text(format!("{:.2}", config.spring.damping_ratio))
                                .size(11)
                                .font(fonts::MONO_FONT)
                                .color(neon::SECONDARY),
                        ]
                        .align_y(Alignment::Center),
                        iced::widget::slider(
                            0.1..=2.0,
                            config.spring.damping_ratio as f32,
                            move |v| {
                                Message::Animations(
                                    AnimationsMessage::SetAnimationSpringDampingRatio(
                                        name_dr.clone(),
                                        v,
                                    ),
                                )
                            }
                        )
                        .width(Length::Fill),
                    ]
                    .spacing(4),
                )
                .push(
                    column![
                        row![
                            text("EPSILON")
                                .size(10)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::OUTLINE_VARIANT),
                            Space::new().width(Length::Fill),
                            text(format!("{:.4}", config.spring.epsilon))
                                .size(11)
                                .font(fonts::MONO_FONT)
                                .color(neon::SECONDARY),
                        ]
                        .align_y(Alignment::Center),
                        iced::widget::slider(
                            0.0001..=0.01,
                            config.spring.epsilon as f32,
                            move |v| {
                                Message::Animations(AnimationsMessage::SetAnimationSpringEpsilon(
                                    name_ep.clone(),
                                    v,
                                ))
                            }
                        )
                        .width(Length::Fill),
                    ]
                    .spacing(4),
                );
        }
        AnimationType::Easing => {
            let name_dur = name.to_string();
            card_content = card_content.push(
                column![
                    row![
                        text("DURATION")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().width(Length::Fill),
                        text(format!("{} ms", config.easing.duration_ms))
                            .size(11)
                            .font(fonts::MONO_FONT)
                            .color(neon::SECONDARY),
                    ]
                    .align_y(Alignment::Center),
                    iced::widget::slider(
                        50.0..=2000.0,
                        config.easing.duration_ms as f32,
                        move |v| {
                            Message::Animations(AnimationsMessage::SetAnimationDuration(
                                name_dur.clone(),
                                v as i32,
                            ))
                        }
                    )
                    .width(Length::Fill),
                ]
                .spacing(4),
            );
        }
        AnimationType::CustomShader if supports_shader => {
            let shader_code = config.custom_shader.clone().unwrap_or_default();
            let name_shader = name.to_string();
            let name_template = name.to_string();
            let name_clear = name.to_string();

            card_content = card_content
                .push(
                    row![
                        text("GLSL CODE")
                            .size(10)
                            .font(fonts::UI_FONT_SEMIBOLD)
                            .color(neon::OUTLINE_VARIANT),
                        Space::new().width(Length::Fill),
                        button(text("Template").size(10).font(fonts::UI_FONT_SEMIBOLD))
                            .on_press(Message::Animations(
                                AnimationsMessage::InsertShaderTemplate(name_template),
                            ))
                            .padding([3, 8]),
                        button(text("Clear").size(10).font(fonts::UI_FONT_SEMIBOLD))
                            .on_press(Message::Animations(AnimationsMessage::ClearCustomShader(
                                name_clear
                            ),))
                            .padding([3, 8]),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .push(
                    container(
                        iced::widget::text_input("Enter GLSL shader code...", &shader_code)
                            .on_input(move |code| {
                                Message::Animations(AnimationsMessage::SetCustomShader(
                                    name_shader.clone(),
                                    code,
                                ))
                            })
                            .font(fonts::MONO_FONT)
                            .size(11)
                            .padding(8)
                            .width(Length::Fill),
                    )
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.1, 0.1, 0.12,
                        ))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.3, 0.3, 0.35),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }),
                )
                .push(
                    text("Note: Custom shaders have no backwards compatibility guarantee.")
                        .size(10)
                        .color(neon::OUTLINE_VARIANT),
                );
        }
        _ => {}
    }

    container(card_content)
        .padding(12)
        .width(Length::Fill)
        .style(crate::theme::card_style)
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn modal_section<'a>(icon: &'a str, label: &'a str, accent: iced::Color) -> Element<'a, Message> {
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

fn styled_slider<'a>(
    label: &'a str,
    display: &str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_slide: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(d)
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
