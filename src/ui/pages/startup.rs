//! Startup commands settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the startup settings page
pub fn startup_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let startup = settings.startup;

    let command_count = startup.commands.len();

    Stack::vertical((
        section(
            "Startup Commands",
            Stack::vertical((
                Label::derived(move || {
                    if command_count == 0 {
                        "No startup commands configured.".to_string()
                    } else {
                        format!("{} startup command(s) configured.", command_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "About Startup Commands",
            Stack::vertical((Label::derived(|| {
                "Startup commands are executed when niri starts. \
                 Use them to launch background services, status bars, \
                 notification daemons, or other applications."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
