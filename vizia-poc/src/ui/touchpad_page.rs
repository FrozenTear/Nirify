//! Touchpad settings page

use vizia::prelude::*;
use crate::app_state::{AppEvent, AppState};
use crate::types::{AccelProfile, ClickMethod, ScrollMethod};

pub fn build_touchpad_page(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Touchpad Settings")
            .font_size(24.0)
            .font_weight(FontWeightKeyword::Bold)
            .class("page-title");

        // Enabled
        setting_row(cx, "Enabled", |cx| {
            Checkbox::new(cx, AppState::touchpad.then(|t| t.enabled))
                .on_toggle(|cx| {
                    let enabled = cx.data::<AppState>().unwrap().touchpad.enabled;
                    cx.emit(AppEvent::SetTouchpadEnabled(!enabled));
                });
        });

        // Tap to Click
        setting_row(cx, "Tap to Click", |cx| {
            Checkbox::new(cx, AppState::touchpad.then(|t| t.tap_to_click))
                .on_toggle(|cx| {
                    let tap = cx.data::<AppState>().unwrap().touchpad.tap_to_click;
                    cx.emit(AppEvent::SetTouchpadTapToClick(!tap));
                });
        });

        // Natural Scroll
        setting_row(cx, "Natural Scroll", |cx| {
            Checkbox::new(cx, AppState::touchpad.then(|t| t.natural_scroll))
                .on_toggle(|cx| {
                    let nat = cx.data::<AppState>().unwrap().touchpad.natural_scroll;
                    cx.emit(AppEvent::SetTouchpadNaturalScroll(!nat));
                });
        });

        // Scroll Method
        setting_row(cx, "Scroll Method", |cx| {
            Dropdown::new(
                cx,
                AppState::touchpad.then(|t| t.scroll_method),
                |cx, item| {
                    Label::new(cx, match item {
                        ScrollMethod::TwoFinger => "Two Finger",
                        ScrollMethod::Edge => "Edge",
                        ScrollMethod::OnButtonDown => "On Button Down",
                    })
                },
                |cx, item| {
                    cx.emit(AppEvent::SetTouchpadScrollMethod(*item));
                },
            );
        });

        // Click Method
        setting_row(cx, "Click Method", |cx| {
            Dropdown::new(
                cx,
                AppState::touchpad.then(|t| t.click_method),
                |cx, item| {
                    Label::new(cx, match item {
                        ClickMethod::ButtonAreas => "Button Areas",
                        ClickMethod::ClickFinger => "Clickfinger",
                    })
                },
                |cx, item| {
                    cx.emit(AppEvent::SetTouchpadClickMethod(*item));
                },
            );
        });

        // Acceleration Profile
        setting_row(cx, "Acceleration Profile", |cx| {
            Dropdown::new(
                cx,
                AppState::touchpad.then(|t| t.accel_profile),
                |cx, item| {
                    Label::new(cx, match item {
                        AccelProfile::Flat => "Flat",
                        AccelProfile::Adaptive => "Adaptive",
                    })
                },
                |cx, item| {
                    cx.emit(AppEvent::SetTouchpadAccelProfile(*item));
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
