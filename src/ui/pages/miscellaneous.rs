//! Miscellaneous settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the miscellaneous settings page
pub fn miscellaneous_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let misc = &settings.miscellaneous;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Window Decorations section
        .child(section(
            "Window Decorations",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Prefer server-side decorations",
                    "Request apps to not draw their own title bars",
                    misc.prefer_no_csd,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Miscellaneous, |s| {
                            s.miscellaneous.prefer_no_csd = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Screenshots section
        .child(section(
            "Screenshots",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(text_row(
                    "Screenshot path",
                    "Path template for saved screenshots",
                    &misc.screenshot_path,
                    "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png",
                    move |val| {
                        state2.update_and_save(SettingsCategory::Miscellaneous, |s| {
                            s.miscellaneous.screenshot_path = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Clipboard section
        .child(section(
            "Clipboard",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable primary clipboard",
                    "Turn off middle-click paste selection",
                    misc.disable_primary_clipboard,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Miscellaneous, |s| {
                            s.miscellaneous.disable_primary_clipboard = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Hotkey Overlay section
        .child(section(
            "Hotkey Overlay",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Skip at startup",
                    "Don't show hotkey overlay when niri starts",
                    misc.hotkey_overlay_skip_at_startup,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Miscellaneous, |s| {
                            s.miscellaneous.hotkey_overlay_skip_at_startup = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
