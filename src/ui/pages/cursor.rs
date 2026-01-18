//! Cursor settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the cursor settings page
pub fn cursor_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let cursor = settings.cursor;

    let auto_hide_enabled = cursor.hide_after_inactive_ms.is_some();
    let hide_delay = cursor.hide_after_inactive_ms.unwrap_or(3000) as f64;

    let state_theme = state.clone();
    let state_size = state.clone();
    let state_hide_typing = state.clone();
    let state_auto_hide = state.clone();
    let state_delay = state.clone();

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
                        state_theme.update_settings(|s| s.cursor.theme = val);
                        state_theme.mark_dirty_and_save(SettingsCategory::Cursor);
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
                        state_size.update_settings(|s| s.cursor.size = val as i32);
                        state_size.mark_dirty_and_save(SettingsCategory::Cursor);
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
                        state_hide_typing.update_settings(|s| s.cursor.hide_when_typing = val);
                        state_hide_typing.mark_dirty_and_save(SettingsCategory::Cursor);
                    },
                ))
                .child(toggle_row(
                    "Auto-hide when idle",
                    "Hide cursor after inactivity",
                    auto_hide_enabled,
                    move |val| {
                        state_auto_hide.update_settings(|s| {
                            s.cursor.hide_after_inactive_ms = if val { Some(3000) } else { None };
                        });
                        state_auto_hide.mark_dirty_and_save(SettingsCategory::Cursor);
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
                        state_delay.update_settings(|s| {
                            if s.cursor.hide_after_inactive_ms.is_some() {
                                s.cursor.hide_after_inactive_ms = Some(val as i32);
                            }
                        });
                        state_delay.mark_dirty_and_save(SettingsCategory::Cursor);
                    },
                )),
        ))
}
