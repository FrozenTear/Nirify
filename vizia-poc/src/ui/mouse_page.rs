//! Mouse settings page

use vizia::prelude::*;
use crate::app_state::{AppEvent, AppState};
use crate::constants::*;
use crate::types::AccelProfile;

pub fn build_mouse_page(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Mouse Settings")
            .font_size(24.0)
            .font_weight(FontWeightKeyword::Bold)
            .class("page-title");

        // Natural Scroll
        setting_row(cx, "Natural Scroll", |cx| {
            Checkbox::new(cx, AppState::mouse.then(|m| m.natural_scroll))
                .on_toggle(|cx| {
                    cx.emit(AppEvent::SetMouseNaturalScroll(!cx.data::<AppState>().unwrap().mouse.natural_scroll));
                });
        });

        // Acceleration Speed
        setting_row(cx, "Acceleration Speed", |cx| {
            HStack::new(cx, |cx| {
                Slider::new(cx, AppState::mouse.then(|m| m.accel_speed as f32))
                    .range(MOUSE_ACCEL_SPEED_MIN as f32..MOUSE_ACCEL_SPEED_MAX as f32)
                    .on_changing(|cx, val| {
                        cx.emit(AppEvent::SetMouseAccelSpeed(val as f64));
                    });

                Label::new(cx, AppState::mouse.then(|m| format!("{:.2}", m.accel_speed)))
                    .width(Pixels(50.0));
            });
        });

        // Acceleration Profile
        setting_row(cx, "Acceleration Profile", |cx| {
            Dropdown::new(
                cx,
                AppState::mouse.then(|m| m.accel_profile),
                |cx, item| {
                    Label::new(cx, match item {
                        AccelProfile::Flat => "Flat",
                        AccelProfile::Adaptive => "Adaptive",
                    })
                },
                |cx, item| {
                    cx.emit(AppEvent::SetMouseAccelProfile(*item));
                },
            );
        });

        // Scroll Factor
        setting_row(cx, "Scroll Factor", |cx| {
            HStack::new(cx, |cx| {
                Slider::new(cx, AppState::mouse.then(|m| m.scroll_factor as f32))
                    .range(MOUSE_SCROLL_FACTOR_MIN as f32..MOUSE_SCROLL_FACTOR_MAX as f32)
                    .on_changing(|cx, val| {
                        cx.emit(AppEvent::SetMouseScrollFactor(val as f64));
                    });

                Label::new(cx, AppState::mouse.then(|m| format!("{:.2}", m.scroll_factor)))
                    .width(Pixels(50.0));
            });
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
