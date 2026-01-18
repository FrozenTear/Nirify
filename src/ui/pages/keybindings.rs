//! Keybindings settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the keybindings settings page
pub fn keybindings_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let keybindings = settings.keybindings;

    let binding_count = keybindings.bindings.len();
    let loaded = keybindings.loaded;
    let error = keybindings.error.clone();

    Stack::vertical((
        section(
            "Keyboard Shortcuts",
            Stack::vertical((
                Label::derived(move || {
                    if !loaded {
                        "Keybindings could not be loaded.".to_string()
                    } else if binding_count == 0 {
                        "No keybindings configured.".to_string()
                    } else {
                        format!("{} keybinding(s) configured.", binding_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
                if let Some(err) = error {
                    Label::derived(move || format!("Error: {}", err.clone()))
                        .style(|s| s.color(crate::ui::theme::ERROR))
                        .into_any()
                } else {
                    floem::views::Empty::new().into_any()
                },
            )),
        ),
        section(
            "About Keybindings",
            Stack::vertical((Label::derived(|| {
                "Keybindings let you assign keyboard shortcuts to actions like \
                 opening applications, switching workspaces, or controlling windows. \
                 Use Mod+key combinations where Mod is typically the Super key."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
