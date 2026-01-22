//! Mouse settings view

use iced::widget::{column, container, scrollable};
use iced::Element;

use super::widgets::*;
use crate::config::models::MouseSettings;
use crate::messages::{Message, MouseMessage};
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
        section_header("Mouse Settings"),
        info_text(
            "Configure mouse behavior, acceleration, and scrolling."
        ),
        toggle_row(
            "Disable when touchpad active",
            "Automatically disable mouse when touchpad is in use",
            off,
            |value| Message::Mouse(MouseMessage::ToggleOffOnTouchpad(value)),
        ),
        spacer(16.0),

        section_header("Scrolling"),
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
        picker_row(
            "Scroll method",
            "How scrolling is performed (button press, wheel, etc.)",
            ScrollMethod::all(),
            Some(scroll_method),
            |value| Message::Mouse(MouseMessage::SetScrollMethod(value)),
        ),
        spacer(16.0),

        section_header("Pointer Acceleration"),
        info_text(
            "Control how mouse movement speed relates to physical movement. Acceleration speed ranges from -1 (slower) to 1 (faster)."
        ),
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
        spacer(16.0),

        section_header("Button Configuration"),
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
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
