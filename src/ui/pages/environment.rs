//! Environment variables settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the environment settings page
pub fn environment_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let environment = settings.environment;

    let var_count = environment.variables.len();

    Stack::vertical((
        section(
            "Environment Variables",
            Stack::vertical((
                Label::derived(move || {
                    if var_count == 0 {
                        "No environment variables configured.".to_string()
                    } else {
                        format!("{} environment variable(s) configured.", var_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "About Environment Variables",
            Stack::vertical((Label::derived(|| {
                "Environment variables set here are available to all applications \
                 launched within niri. Common uses include setting XDG paths, \
                 GPU configurations, or toolkit preferences."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
