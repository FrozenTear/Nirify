//! Window rules settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the window rules settings page
pub fn window_rules_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let rules = settings.window_rules;

    let rule_count = rules.rules.len();

    Stack::vertical((
        section(
            "Window Rules",
            Stack::vertical((
                Label::derived(move || {
                    if rule_count == 0 {
                        "No window rules configured.".to_string()
                    } else {
                        format!("{} window rule(s) configured.", rule_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "About Window Rules",
            Stack::vertical((Label::derived(|| {
                "Window rules let you customize behavior for specific applications. \
                 Match windows by app-id, title, or other criteria, then apply \
                 custom settings like opacity, size constraints, or opening behavior."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
