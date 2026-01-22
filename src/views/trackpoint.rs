//! Trackpoint settings view
//!
//! Configure trackpoint (pointing stick / nipple mouse) behavior.

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::TrackpointSettings;
use crate::messages::{Message, TrackpointMessage};
use crate::types::{AccelProfile, ScrollMethod};

/// Creates the trackpoint settings view
pub fn view(settings: TrackpointSettings) -> Element<'static, Message> {
    let off = settings.off;
    let natural_scroll = settings.natural_scroll;
    let accel_speed = settings.accel_speed;
    let accel_profile = settings.accel_profile;
    let scroll_method = settings.scroll_method;
    let left_handed = settings.left_handed;
    let middle_emulation = settings.middle_emulation;
    let scroll_button_lock = settings.scroll_button_lock;

    let content = column![
        section_header("Trackpoint Settings"),
        info_text(
            "Configure trackpoint (pointing stick) behavior, acceleration, and scrolling."
        ),
        toggle_row(
            "Disable trackpoint",
            "Completely disable this trackpoint device",
            off,
            |value| Message::Trackpoint(TrackpointMessage::SetOff(value)),
        ),
        spacer(16.0),

        section_header("Scrolling"),
        toggle_row(
            "Natural scroll",
            "Reverse scroll direction",
            natural_scroll,
            |value| Message::Trackpoint(TrackpointMessage::SetNaturalScroll(value)),
        ),
        picker_row(
            "Scroll method",
            "How scrolling is performed (on-button-down is typical for trackpoints)",
            ScrollMethod::all(),
            Some(scroll_method),
            |value| Message::Trackpoint(TrackpointMessage::SetScrollMethod(value)),
        ),
        toggle_row(
            "Scroll button lock",
            "Lock scroll button state (don't need to hold)",
            scroll_button_lock,
            |value| Message::Trackpoint(TrackpointMessage::SetScrollButtonLock(value)),
        ),
        spacer(16.0),

        section_header("Pointer Acceleration"),
        info_text(
            "Control how trackpoint movement speed relates to physical pressure."
        ),
        slider_row(
            "Acceleration speed",
            "Speed from -1.0 (slow) to 1.0 (fast)",
            accel_speed as f32,
            -1.0,
            1.0,
            "",
            |value| Message::Trackpoint(TrackpointMessage::SetAccelSpeed(value)),
        ),
        picker_row(
            "Acceleration profile",
            "Adaptive varies with speed, flat maintains constant ratio",
            AccelProfile::all(),
            Some(accel_profile),
            |value| Message::Trackpoint(TrackpointMessage::SetAccelProfile(value)),
        ),
        spacer(16.0),

        section_header("Button Configuration"),
        toggle_row(
            "Left-handed mode",
            "Swap left and right buttons",
            left_handed,
            |value| Message::Trackpoint(TrackpointMessage::SetLeftHanded(value)),
        ),
        toggle_row(
            "Middle button emulation",
            "Emulate middle click by pressing left+right simultaneously",
            middle_emulation,
            |value| Message::Trackpoint(TrackpointMessage::SetMiddleEmulation(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
