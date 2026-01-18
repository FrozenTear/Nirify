//! Workspaces settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the workspaces settings page
pub fn workspaces_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let workspaces = settings.workspaces;

    let workspace_count = workspaces.workspaces.len();

    Stack::vertical((
        section(
            "Named Workspaces",
            Stack::vertical((
                Label::derived(move || {
                    if workspace_count == 0 {
                        "No named workspaces configured.".to_string()
                    } else {
                        format!("{} named workspace(s) configured.", workspace_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
                // Note: A full implementation would list workspaces with edit/delete buttons
                // and an "Add Workspace" button
            )),
        ),
        section(
            "Info",
            Stack::vertical((Label::derived(|| {
                "Named workspaces can be pinned to specific outputs \
                 and accessed by name in keybindings."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
