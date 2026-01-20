//! Sidebar navigation component

use vizia::prelude::*;
use crate::app_state::{AppEvent, AppState, Panel};

pub fn build_sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Title
        Label::new(cx, "Niri Settings")
            .font_size(20.0)
            .font_weight(FontWeightKeyword::Bold)
            .class("sidebar-title");

        // Navigation items
        nav_item(cx, "Keyboard", Panel::Keyboard);
        nav_item(cx, "Mouse", Panel::Mouse);
        nav_item(cx, "Touchpad", Panel::Touchpad);

        // Spacer to push theme toggle to bottom
        Element::new(cx).class("spacer");

        // Theme toggle
        HStack::new(cx, |cx| {
            Label::new(cx, "Dark Mode");
            Checkbox::new(cx, AppState::dark_mode)
                .on_toggle(|cx| cx.emit(AppEvent::ToggleDarkMode));
        })
        .class("theme-toggle");

        // Status bar
        Label::new(cx, AppState::status_message)
            .class("status-bar");
    })
    .class("sidebar")
    .width(Pixels(200.0));
}

fn nav_item(cx: &mut Context, label: &'static str, panel: Panel) {
    let panel_clone = panel;
    Button::new(
        cx,
        |cx| cx.emit(AppEvent::SelectPanel(panel_clone)),
        |cx| Label::new(cx, label),
    )
    .class("nav-item")
    .checked(AppState::current_panel.map(move |p| *p == panel));
}
