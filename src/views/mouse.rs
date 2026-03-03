//! Mouse settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::MouseSettings;
use crate::messages::{Message, MouseMessage};
use crate::theme::muted_text_container;
use crate::types::{AccelProfile, ScrollMethod};

/// Creates the mouse settings view
pub fn view(settings: &MouseSettings) -> Element<'_, Message> {
    let off = settings.off;
    let natural_scroll = settings.natural_scroll;
    let accel_speed = settings.accel_speed;
    let accel_profile = settings.accel_profile;
    let scroll_factor = settings.scroll_factor;
    let scroll_method = settings.scroll_method;
    let left_handed = settings.left_handed;
    let middle_emulation = settings.middle_emulation;
    let scroll_button_lock = settings.scroll_button_lock;

    let content = column![
        page_title("Mouse Settings"),
        info_text(
            "Configure mouse behavior, acceleration, and scrolling."
        ),
        card(column![
            toggle_row(
                "Disable when touchpad active",
                "Automatically disable mouse when touchpad is in use",
                off,
                |value| Message::Mouse(MouseMessage::ToggleOffOnTouchpad(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Scrolling"),
        card(column![
            toggle_row(
                "Natural scroll",
                "Reverse scroll direction (like on macOS/touchpads)",
                natural_scroll,
                |value| Message::Mouse(MouseMessage::ToggleNaturalScroll(value)),
            ),
            slider_row(
                "Scroll factor",
                "Multiplier for scroll speed (1.0 = default)",
                scroll_factor as f32,
                0.1,
                10.0,
                "x",
                |value| Message::Mouse(MouseMessage::SetScrollFactor(value)),
            ),
            optional_scroll_factor(
                settings.scroll_factor_horizontal,
                |v| Message::Mouse(MouseMessage::SetScrollFactorHorizontal(v)),
            ),
            picker_row(
                "Scroll method",
                "How scrolling is performed (button press, wheel, etc.)",
                ScrollMethod::all(),
                Some(scroll_method),
                |value| Message::Mouse(MouseMessage::SetScrollMethod(value)),
            ),
            scroll_button_input(settings.scroll_button, |v| Message::Mouse(MouseMessage::SetScrollButton(v))),
        ].spacing(0).width(Length::Fill)),
        section_header("Pointer Acceleration"),
        info_text(
            "Control how mouse movement speed relates to physical movement. Acceleration speed ranges from -1 (slower) to 1 (faster)."
        ),
        card(column![
            slider_row(
                "Acceleration speed",
                "Pointer acceleration from -1.0 (slow) to 1.0 (fast)",
                accel_speed as f32,
                -1.0,
                1.0,
                "",
                |value| Message::Mouse(MouseMessage::SetAccelSpeed(value)),
            ),
            picker_row(
                "Acceleration profile",
                "Choose between adaptive (varies with speed) or flat (constant ratio)",
                AccelProfile::all(),
                Some(accel_profile),
                |value| Message::Mouse(MouseMessage::SetAccelProfile(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Button Configuration"),
        card(column![
            toggle_row(
                "Left-handed mode",
                "Swap left and right mouse buttons",
                left_handed,
                |value| Message::Mouse(MouseMessage::ToggleLeftHanded(value)),
            ),
            toggle_row(
                "Middle button emulation",
                "Emulate middle click by pressing left+right simultaneously",
                middle_emulation,
                |value| Message::Mouse(MouseMessage::ToggleMiddleEmulation(value)),
            ),
            toggle_row(
                "Scroll button lock",
                "Lock scroll button state when using on-button-down scrolling",
                scroll_button_lock,
                |value| Message::Mouse(MouseMessage::ToggleScrollButtonLock(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Optional horizontal scroll factor (toggler + slider)
fn optional_scroll_factor<'a>(
    value: Option<f64>,
    on_change: impl Fn(Option<f32>) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    use iced::widget::{row, slider, toggler};
    use iced::Alignment;
    let is_enabled = value.is_some();
    let current_value = value.unwrap_or(1.0) as f32;

    let mut col = column![
        row![
            column![
                text("Horizontal scroll factor").size(15),
                container(text("Override scroll speed for horizontal scrolling (leave disabled to match vertical)").size(11)).style(muted_text_container),
            ]
            .width(Length::Fill),
            toggler(is_enabled)
                .on_toggle(move |enabled| {
                    if enabled {
                        on_change(Some(1.0))
                    } else {
                        on_change(None)
                    }
                }),
        ]
        .spacing(20)
        .align_y(Alignment::Center),
    ]
    .spacing(8)
    .padding(12);

    if is_enabled {
        col = col.push(
            row![
                slider(0.1..=10.0, current_value, move |v| on_change(Some(v)))
                    .width(Length::Fill),
                text(format!("{:.1}x", current_value))
                    .size(13)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        );
    }

    col.into()
}

/// Scroll button input (optional integer for on-button-down scrolling)
fn scroll_button_input<'a>(
    value: Option<i32>,
    on_change: impl Fn(Option<i32>) -> Message + 'a,
) -> Element<'a, Message> {
    let display_value = value.map(|v| v.to_string()).unwrap_or_default();
    column![
        text("Scroll button").size(15),
        container(text("Linux button code for on-button-down scrolling (e.g., 274 for middle button). Leave empty for default.").size(11)).style(muted_text_container),
        text_input("e.g., 274", &display_value)
            .on_input(move |s| {
                if s.is_empty() {
                    on_change(None)
                } else if let Ok(v) = s.parse::<i32>() {
                    on_change(Some(v))
                } else {
                    on_change(value)
                }
            })
            .padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}
