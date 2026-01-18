//! Miscellaneous settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, text_row, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the miscellaneous settings page
pub fn miscellaneous_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let misc = settings.miscellaneous;

    let prefer_no_csd = RwSignal::new(misc.prefer_no_csd);
    let screenshot_path = RwSignal::new(misc.screenshot_path);
    let disable_primary_clipboard = RwSignal::new(misc.disable_primary_clipboard);
    let hotkey_overlay_skip = RwSignal::new(misc.hotkey_overlay_skip_at_startup);

    // Callbacks
    let on_prefer_no_csd = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.miscellaneous.prefer_no_csd = val);
        state.mark_dirty_and_save(SettingsCategory::Miscellaneous);
    })};

    let on_disable_primary_clipboard = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.miscellaneous.disable_primary_clipboard = val);
        state.mark_dirty_and_save(SettingsCategory::Miscellaneous);
    })};

    let on_hotkey_overlay_skip = { Rc::new(move |val: bool| {
        state.update_settings(|s| s.miscellaneous.hotkey_overlay_skip_at_startup = val);
        state.mark_dirty_and_save(SettingsCategory::Miscellaneous);
    })};

    Stack::vertical((
        section(
            "Window Decorations",
            Stack::vertical((
                toggle_row_with_callback("Prefer server-side decorations", Some("Request apps to not draw their own title bars"), prefer_no_csd, Some(on_prefer_no_csd)),
            )),
        ),
        section(
            "Screenshots",
            Stack::vertical((text_row(
                "Screenshot path",
                Some("Path template for saved screenshots"),
                screenshot_path,
                "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png",
            ),)),
        ),
        section(
            "Clipboard",
            Stack::vertical((
                toggle_row_with_callback("Disable primary clipboard", Some("Turn off middle-click paste selection"), disable_primary_clipboard, Some(on_disable_primary_clipboard)),
            )),
        ),
        section(
            "Hotkey Overlay",
            Stack::vertical((
                toggle_row_with_callback("Skip at startup", Some("Don't show hotkey overlay when niri starts"), hotkey_overlay_skip, Some(on_hotkey_overlay_skip)),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
