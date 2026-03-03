//! Touchpad settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::TouchpadSettings;
use crate::messages::{Message, TouchpadMessage};
use crate::theme::muted_text_container;
use crate::types::{AccelProfile, ClickMethod, ScrollMethod, TapButtonMap};

/// Creates the touchpad settings view
pub fn view(settings: &TouchpadSettings) -> Element<'_, Message> {
    let tap = settings.tap;
    let dwt = settings.dwt;
    let dwtp = settings.dwtp;
    let natural_scroll = settings.natural_scroll;
    let accel_speed = settings.accel_speed;
    let accel_profile = settings.accel_profile;
    let scroll_factor = settings.scroll_factor;
    let scroll_method = settings.scroll_method;
    let click_method = settings.click_method;
    let tap_button_map = settings.tap_button_map;
    let left_handed = settings.left_handed;
    let middle_emulation = settings.middle_emulation;
    let drag = settings.drag;
    let drag_lock = settings.drag_lock;

    let content = column![
        page_title("Tap Settings"),
        info_text(
            "Configure tap-to-click behavior and multi-finger tap gestures."
        ),
        card(column![
            toggle_row(
                "Tap to click",
                "Enable tapping the touchpad to register clicks",
                tap,
                |value| Message::Touchpad(TouchpadMessage::ToggleTapToClick(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Disable While Typing"),
        info_text(
            "Prevent accidental touchpad input while typing. DWT = disable while typing, DWTP = disable while trackpoint is active."
        ),
        card(column![
            toggle_row(
                "Disable while typing (DWT)",
                "Temporarily disable touchpad when keyboard is in use",
                dwt,
                |value| Message::Touchpad(TouchpadMessage::ToggleDwt(value)),
            ),
            toggle_row(
                "Disable while trackpoint active (DWTP)",
                "Disable touchpad when trackpoint is being used",
                dwtp,
                |value| Message::Touchpad(TouchpadMessage::ToggleDwtp(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Scrolling"),
        card(column![
            toggle_row(
                "Natural scroll",
                "Reverse scroll direction (like on macOS)",
                natural_scroll,
                |value| Message::Touchpad(TouchpadMessage::ToggleNaturalScroll(value)),
            ),
            slider_row(
                "Scroll factor",
                "Multiplier for scroll speed (1.0 = default)",
                scroll_factor as f32,
                0.1,
                10.0,
                "x",
                |value| Message::Touchpad(TouchpadMessage::SetScrollFactor(value)),
            ),
            optional_scroll_factor(
                settings.scroll_factor_horizontal,
                |v| Message::Touchpad(TouchpadMessage::SetScrollFactorHorizontal(v)),
            ),
            picker_row(
                "Scroll method",
                "How scrolling gestures are interpreted (two-finger, edge, etc.)",
                ScrollMethod::all(),
                Some(scroll_method),
                |value| Message::Touchpad(TouchpadMessage::SetScrollMethod(value)),
            ),
            scroll_button_input(settings.scroll_button, |v| Message::Touchpad(TouchpadMessage::SetScrollButton(v))),
        ].spacing(0).width(Length::Fill)),
        section_header("Pointer Acceleration"),
        info_text(
            "Control how pointer movement speed relates to finger movement. Speed ranges from -1 (slower) to 1 (faster)."
        ),
        card(column![
            slider_row(
                "Acceleration speed",
                "Pointer acceleration from -1.0 (slow) to 1.0 (fast)",
                accel_speed as f32,
                -1.0,
                1.0,
                "",
                |value| Message::Touchpad(TouchpadMessage::SetAccelSpeed(value)),
            ),
            picker_row(
                "Acceleration profile",
                "Choose between adaptive (varies with speed) or flat (constant ratio)",
                AccelProfile::all(),
                Some(accel_profile),
                |value| Message::Touchpad(TouchpadMessage::SetAccelProfile(value)),
            ),
            picker_row(
                "Click method",
                "How multi-finger taps are interpreted (button areas vs clickfinger)",
                ClickMethod::all(),
                Some(click_method),
                |value| Message::Touchpad(TouchpadMessage::SetClickMethod(value)),
            ),
            picker_row(
                "Tap button map",
                "Mapping of 2/3-finger taps to mouse buttons",
                TapButtonMap::all(),
                Some(tap_button_map),
                |value| Message::Touchpad(TouchpadMessage::SetTapButtonMap(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Additional Settings"),
        card(column![
            toggle_row(
                "Left-handed mode",
                "Swap left and right button areas/gestures",
                left_handed,
                |value| Message::Touchpad(TouchpadMessage::ToggleLeftHanded(value)),
            ),
            toggle_row(
                "Middle button emulation",
                "Emulate middle click by tapping with two fingers simultaneously",
                middle_emulation,
                |value| Message::Touchpad(TouchpadMessage::ToggleMiddleEmulation(value)),
            ),
            toggle_row(
                "Drag",
                "Enable tap-and-drag gesture",
                drag,
                |value| Message::Touchpad(TouchpadMessage::ToggleDrag(value)),
            ),
            toggle_row(
                "Drag lock",
                "Lock drag state until tapped again (requires drag enabled)",
                drag_lock,
                |value| Message::Touchpad(TouchpadMessage::ToggleDragLock(value)),
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
