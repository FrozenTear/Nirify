//! Trackball settings view
//!
//! Configure trackball device behavior.

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::TrackballSettings;
use crate::messages::{Message, TrackballMessage};
use crate::types::{AccelProfile, ScrollMethod};

/// Creates the trackball settings view
pub fn view(settings: &TrackballSettings) -> Element<'_, Message> {
    let off = settings.off;
    let natural_scroll = settings.natural_scroll;
    let accel_speed = settings.accel_speed;
    let accel_profile = settings.accel_profile;
    let scroll_method = settings.scroll_method;
    let left_handed = settings.left_handed;
    let middle_emulation = settings.middle_emulation;
    let scroll_button_lock = settings.scroll_button_lock;

    let content = column![
        section_header("Trackball Settings"),
        info_text(
            "Configure trackball device behavior, acceleration, and scrolling."
        ),
        toggle_row(
            "Disable trackball",
            "Completely disable this trackball device",
            off,
            |value| Message::Trackball(TrackballMessage::SetOff(value)),
        ),
        spacer(16.0),

        section_header("Scrolling"),
        toggle_row(
            "Natural scroll",
            "Reverse scroll direction",
            natural_scroll,
            |value| Message::Trackball(TrackballMessage::SetNaturalScroll(value)),
        ),
        picker_row(
            "Scroll method",
            "How scrolling is performed",
            ScrollMethod::all(),
            Some(scroll_method),
            |value| Message::Trackball(TrackballMessage::SetScrollMethod(value)),
        ),
        toggle_row(
            "Scroll button lock",
            "Lock scroll button state (don't need to hold)",
            scroll_button_lock,
            |value| Message::Trackball(TrackballMessage::SetScrollButtonLock(value)),
        ),
        spacer(16.0),

        section_header("Pointer Acceleration"),
        info_text(
            "Control how trackball movement speed relates to physical rotation."
        ),
        slider_row(
            "Acceleration speed",
            "Speed from -1.0 (slow) to 1.0 (fast)",
            accel_speed as f32,
            -1.0,
            1.0,
            "",
            |value| Message::Trackball(TrackballMessage::SetAccelSpeed(value)),
        ),
        picker_row(
            "Acceleration profile",
            "Adaptive varies with speed, flat maintains constant ratio",
            AccelProfile::all(),
            Some(accel_profile),
            |value| Message::Trackball(TrackballMessage::SetAccelProfile(value)),
        ),
        spacer(16.0),

        section_header("Button Configuration"),
        toggle_row(
            "Left-handed mode",
            "Swap left and right buttons",
            left_handed,
            |value| Message::Trackball(TrackballMessage::SetLeftHanded(value)),
        ),
        toggle_row(
            "Middle button emulation",
            "Emulate middle click by pressing left+right simultaneously",
            middle_emulation,
            |value| Message::Trackball(TrackballMessage::SetMiddleEmulation(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
