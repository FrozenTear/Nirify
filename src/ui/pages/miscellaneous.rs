//! Miscellaneous settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the miscellaneous settings page
pub fn miscellaneous_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let misc = settings.miscellaneous;

    let state_csd = state.clone();
    let state_path = state.clone();
    let state_clipboard = state.clone();
    let state_hotkey = state.clone();

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
                        state_csd.update_settings(|s| s.miscellaneous.prefer_no_csd = val);
                        state_csd.mark_dirty_and_save(SettingsCategory::Miscellaneous);
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
                        state_path.update_settings(|s| s.miscellaneous.screenshot_path = val);
                        state_path.mark_dirty_and_save(SettingsCategory::Miscellaneous);
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
                        state_clipboard
                            .update_settings(|s| s.miscellaneous.disable_primary_clipboard = val);
                        state_clipboard.mark_dirty_and_save(SettingsCategory::Miscellaneous);
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
                        state_hotkey
                            .update_settings(|s| s.miscellaneous.hotkey_overlay_skip_at_startup = val);
                        state_hotkey.mark_dirty_and_save(SettingsCategory::Miscellaneous);
                    },
                )),
        ))
}
