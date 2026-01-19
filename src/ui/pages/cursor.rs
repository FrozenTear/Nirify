//! Cursor settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the cursor settings page
pub fn cursor_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let cursor = &settings.cursor;

    let auto_hide = cursor.hide_after_inactive_ms.is_some();
    let hide_delay = cursor.hide_after_inactive_ms.unwrap_or(3000) as f64;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Appearance section
        .child(section(
            "Appearance",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(text_row(
                    "Cursor theme",
                    "Theme name (empty = system default)",
                    &cursor.theme,
                    "Adwaita",
                    move |val| {
                        state1.update_and_save(SettingsCategory::Cursor, |s| {
                            s.cursor.theme = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Cursor size",
                    "Size in pixels",
                    cursor.size as f64,
                    16.0,
                    96.0,
                    "px",
                    move |val| {
                        state2.update_and_save(SettingsCategory::Cursor, |s| {
                            s.cursor.size = val as i32
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Behavior section
        .child(section(
            "Behavior",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Hide while typing",
                    "Hide cursor when using keyboard",
                    cursor.hide_when_typing,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Cursor, |s| {
                            s.cursor.hide_when_typing = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Auto-hide when idle",
                    "Hide cursor after inactivity",
                    auto_hide,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Cursor, |s| {
                            s.cursor.hide_after_inactive_ms = if val { Some(3000) } else { None };
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Auto-hide delay",
                    "Milliseconds before hiding",
                    hide_delay,
                    500.0,
                    10000.0,
                    "ms",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Cursor, |s| {
                            if s.cursor.hide_after_inactive_ms.is_some() {
                                s.cursor.hide_after_inactive_ms = Some(val as i32);
                            }
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
