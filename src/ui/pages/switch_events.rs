//! Switch events settings page (lid close/open, tablet mode)

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the switch events settings page
pub fn switch_events_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let events = settings.switch_events;

    let has_lid_close = !events.lid_close.spawn.is_empty();
    let has_lid_open = !events.lid_open.spawn.is_empty();
    let has_tablet_on = !events.tablet_mode_on.spawn.is_empty();
    let has_tablet_off = !events.tablet_mode_off.spawn.is_empty();

    Stack::vertical((
        section(
            "Lid Events",
            Stack::vertical((
                Label::derived(move || {
                    if has_lid_close {
                        "Lid close: Command configured".to_string()
                    } else {
                        "Lid close: No action".to_string()
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
                Label::derived(move || {
                    if has_lid_open {
                        "Lid open: Command configured".to_string()
                    } else {
                        "Lid open: No action".to_string()
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "Tablet Mode Events",
            Stack::vertical((
                Label::derived(move || {
                    if has_tablet_on {
                        "Tablet mode on: Command configured".to_string()
                    } else {
                        "Tablet mode on: No action".to_string()
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
                Label::derived(move || {
                    if has_tablet_off {
                        "Tablet mode off: Command configured".to_string()
                    } else {
                        "Tablet mode off: No action".to_string()
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "About Switch Events",
            Stack::vertical((Label::derived(|| {
                "Switch events let you run commands when hardware switches change state. \
                 For example, lock the screen when closing the laptop lid, or change \
                 layout when entering tablet mode."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
