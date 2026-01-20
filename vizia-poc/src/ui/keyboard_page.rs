//! Keyboard settings page

use vizia::prelude::*;
use crate::app_state::{AppEvent, AppState, TrackLayout};
use crate::constants::*;

pub fn build_keyboard_page(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Page title
        Label::new(cx, "Keyboard Settings")
            .font_size(24.0)
            .font_weight(FontWeightKeyword::Bold)
            .class("page-title");

        // Repeat Rate
        setting_row(cx, "Repeat Rate", |cx| {
            HStack::new(cx, |cx| {
                Slider::new(cx, AppState::keyboard.then(|k| k.repeat_rate as f32))
                    .range(KEYBOARD_REPEAT_RATE_MIN as f32..KEYBOARD_REPEAT_RATE_MAX as f32)
                    .on_changing(|cx, val| {
                        cx.emit(AppEvent::SetKeyboardRepeatRate(val as i32));
                    });

                Label::new(cx, AppState::keyboard.then(|k| format!("{}", k.repeat_rate)))
                    .width(Pixels(40.0));
            });
        });

        // Repeat Delay
        setting_row(cx, "Repeat Delay (ms)", |cx| {
            HStack::new(cx, |cx| {
                Slider::new(cx, AppState::keyboard.then(|k| k.repeat_delay as f32))
                    .range(KEYBOARD_REPEAT_DELAY_MIN as f32..KEYBOARD_REPEAT_DELAY_MAX as f32)
                    .on_changing(|cx, val| {
                        cx.emit(AppEvent::SetKeyboardRepeatDelay(val as i32));
                    });

                Label::new(cx, AppState::keyboard.then(|k| format!("{} ms", k.repeat_delay)))
                    .width(Pixels(60.0));
            });
        });

        // Track Layout
        setting_row(cx, "Track Layout", |cx| {
            Dropdown::new(
                cx,
                AppState::keyboard.then(|k| k.track_layout),
                |cx, item| {
                    Label::new(cx, match item {
                        TrackLayout::Global => "Global",
                        TrackLayout::Window => "Per Window",
                    })
                },
                |cx, item| {
                    cx.emit(AppEvent::SetKeyboardTrackLayout(*item));
                },
            );
        });

        // Save button
        Button::new(
            cx,
            |cx| cx.emit(AppEvent::SaveConfig),
            |cx| Label::new(cx, "Save Settings"),
        )
        .class("save-button")
        .disabled(AppState::unsaved_changes.map(|u| !*u));
    })
    .class("page-content")
    .child_space(Stretch(1.0));
}

/// Reusable setting row with label and control
fn setting_row(cx: &mut Context, label: &'static str, content: impl FnOnce(&mut Context)) {
    HStack::new(cx, |cx| {
        Label::new(cx, label)
            .class("setting-label")
            .width(Pixels(180.0));

        content(cx);
    })
    .class("setting-row")
    .height(Pixels(40.0));
}
