//! Keyboard settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the keyboard settings page
pub fn keyboard_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let keyboard = settings.keyboard;

    let off = RwSignal::new(keyboard.off);
    let xkb_layout = RwSignal::new(keyboard.xkb_layout);
    let xkb_variant = RwSignal::new(keyboard.xkb_variant);
    let xkb_options = RwSignal::new(keyboard.xkb_options);
    let repeat_delay = RwSignal::new(keyboard.repeat_delay as f64);
    let repeat_rate = RwSignal::new(keyboard.repeat_rate as f64);
    let numlock = RwSignal::new(keyboard.numlock);

    Stack::vertical((
        // XKB Settings section
        section(
            "Layout",
            Stack::vertical((
                text_row(
                    "Keyboard layout",
                    Some("XKB layout code (e.g., us, de, fr)"),
                    xkb_layout,
                    "us",
                ),
                text_row(
                    "Layout variant",
                    Some("Layout variant (e.g., dvorak, colemak)"),
                    xkb_variant,
                    "",
                ),
                text_row(
                    "XKB options",
                    Some("Additional options (e.g., compose:ralt)"),
                    xkb_options,
                    "",
                ),
            )),
        ),
        // Repeat settings
        section(
            "Key Repeat",
            Stack::vertical((
                slider_row(
                    "Repeat delay",
                    Some("Delay before key repeat starts"),
                    repeat_delay,
                    25.0,
                    1000.0,
                    25.0,
                    "ms",
                ),
                slider_row(
                    "Repeat rate",
                    Some("Characters per second when held"),
                    repeat_rate,
                    7.0,
                    300.0,
                    5.0,
                    "/s",
                ),
            )),
        ),
        // Other settings
        section(
            "Options",
            Stack::vertical((
                toggle_row("Enable NumLock", Some("Turn on NumLock at startup"), numlock),
                toggle_row(
                    "Disable keyboard",
                    Some("WARNING: This can lock you out!"),
                    off,
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
