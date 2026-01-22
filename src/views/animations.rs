//! Animations settings view with custom GLSL shader support

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::{AnimationId, AnimationSettings, AnimationType, SingleAnimationConfig};
use crate::messages::{AnimationsMessage, Message};
use crate::theme::fonts;

/// Animation type options for the dropdown
const ANIMATION_TYPES: [&str; 5] = ["Default", "Off", "Spring", "Easing", "Custom Shader"];
const ANIMATION_TYPES_NO_SHADER: [&str; 4] = ["Default", "Off", "Spring", "Easing"];

/// Creates the animations settings view
pub fn view(settings: &AnimationSettings) -> Element<'_, Message> {
    let slowdown_enabled = (settings.slowdown - 1.0).abs() > 0.01;

    let content = column![
        section_header("Animations"),
        info_text(
            "Configure niri's window and workspace animations. Each animation can use spring physics, \
             easing curves, or custom GLSL shaders."
        ),
        spacer(8.0),

        // Global settings
        subsection_header("Global Settings"),
        toggle_row(
            "Enable slowdown",
            "Slow down all animations for debugging or effect",
            slowdown_enabled,
            |enabled| Message::Animations(AnimationsMessage::ToggleSlowdown(enabled)),
        ),
        if slowdown_enabled {
            slider_row(
                "Slowdown factor",
                "How much to slow down animations",
                settings.slowdown as f32,
                1.0,
                10.0,
                "x",
                |value| Message::Animations(AnimationsMessage::SetSlowdownFactor(value)),
            )
        } else {
            column![].into()
        },
        spacer(16.0),

        // Shader-compatible animations with special UI
        subsection_header("Window Animations (Shader Support)"),
        info_text(
            "These animations support custom GLSL shaders for advanced visual effects."
        ),
        animation_card("window-open", AnimationId::WindowOpen, &settings.per_animation.window_open, true),
        animation_card("window-close", AnimationId::WindowClose, &settings.per_animation.window_close, true),
        animation_card("window-resize", AnimationId::WindowResize, &settings.per_animation.window_resize, true),
        spacer(16.0),

        // Other animations
        subsection_header("Other Animations"),
        animation_card("workspace-switch", AnimationId::WorkspaceSwitch, &settings.per_animation.workspace_switch, false),
        animation_card("window-movement", AnimationId::WindowMovement, &settings.per_animation.window_movement, false),
        animation_card("horizontal-view", AnimationId::HorizontalViewMovement, &settings.per_animation.horizontal_view_movement, false),
        animation_card("overview", AnimationId::Overview, &settings.per_animation.overview_open_close, false),
        animation_card("config-notification", AnimationId::ConfigNotification, &settings.per_animation.config_notification_open_close, false),
        animation_card("exit-confirmation", AnimationId::ExitConfirmation, &settings.per_animation.exit_confirmation_open_close, false),
        animation_card("screenshot-ui", AnimationId::ScreenshotUi, &settings.per_animation.screenshot_ui_open, false),
        animation_card("recent-windows", AnimationId::RecentWindows, &settings.per_animation.recent_windows_close, false),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
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
    let type_selector = pick_list(
        type_options,
        selected_type,
        move |selected: &str| {
            let idx = match selected {
                "Default" => 0,
                "Off" => 1,
                "Spring" => 2,
                "Easing" => 3,
                "Custom Shader" => 4,
                _ => 0,
            };
            Message::Animations(AnimationsMessage::SetAnimationType(name_owned.clone(), idx))
        },
    )
    .width(Length::Fixed(150.0));

    let mut card_content = column![
        row![
            text(id.name()).size(14).font(fonts::UI_FONT_SEMIBOLD),
            container(type_selector).width(Length::Fill).align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(8);

    // Show parameters based on animation type
    match config.animation_type {
        AnimationType::Spring => {
            let name_dr = name.to_string();
            let name_ep = name.to_string();
            card_content = card_content
                .push(
                    row![
                        text("Damping ratio:").size(12),
                        text(format!("{:.2}", config.spring.damping_ratio)).size(12).font(fonts::MONO_FONT),
                    ]
                    .spacing(8),
                )
                .push(
                    iced::widget::slider(0.1..=2.0, config.spring.damping_ratio as f32, move |v| {
                        Message::Animations(AnimationsMessage::SetAnimationSpringDampingRatio(
                            name_dr.clone(),
                            v,
                        ))
                    })
                    .width(Length::Fill),
                )
                .push(
                    row![
                        text("Epsilon:").size(12),
                        text(format!("{:.4}", config.spring.epsilon)).size(12).font(fonts::MONO_FONT),
                    ]
                    .spacing(8),
                )
                .push(
                    iced::widget::slider(0.0001..=0.01, config.spring.epsilon as f32, move |v| {
                        Message::Animations(AnimationsMessage::SetAnimationSpringEpsilon(
                            name_ep.clone(),
                            v,
                        ))
                    })
                    .width(Length::Fill),
                );
        }
        AnimationType::Easing => {
            let name_dur = name.to_string();
            card_content = card_content
                .push(
                    row![
                        text("Duration:").size(12),
                        text(format!("{} ms", config.easing.duration_ms)).size(12).font(fonts::MONO_FONT),
                    ]
                    .spacing(8),
                )
                .push(
                    iced::widget::slider(50.0..=2000.0, config.easing.duration_ms as f32, move |v| {
                        Message::Animations(AnimationsMessage::SetAnimationDuration(
                            name_dur.clone(),
                            v as i32,
                        ))
                    })
                    .width(Length::Fill),
                );
        }
        AnimationType::CustomShader if supports_shader => {
            let shader_code = config.custom_shader.clone().unwrap_or_default();
            let name_shader = name.to_string();
            let name_template = name.to_string();
            let name_clear = name.to_string();

            card_content = card_content
                .push(spacer(8.0))
                .push(
                    row![
                        text("GLSL Code:").size(12).font(fonts::UI_FONT_SEMIBOLD),
                        container(
                            row![
                                button(text("Insert Template").size(11))
                                    .on_press(Message::Animations(AnimationsMessage::InsertShaderTemplate(name_template)))
                                    .padding([4, 8]),
                                button(text("Clear").size(11))
                                    .on_press(Message::Animations(AnimationsMessage::ClearCustomShader(name_clear)))
                                    .padding([4, 8]),
                            ]
                            .spacing(4)
                        )
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Right),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .push(
                    container(
                        iced::widget::text_input("Enter GLSL shader code...", &shader_code)
                            .on_input(move |code| {
                                Message::Animations(AnimationsMessage::SetCustomShader(name_shader.clone(), code))
                            })
                            .font(fonts::MONO_FONT)
                            .size(12)
                            .padding(8)
                            .width(Length::Fill)
                    )
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.12))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.3, 0.3, 0.35),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }),
                )
                .push(
                    text("Note: Custom shaders have no backwards compatibility guarantee from niri.")
                        .size(11)
                        .color([0.6, 0.5, 0.4]),
                );
        }
        _ => {}
    }

    container(card_content)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.14))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.2, 0.2, 0.25),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}
