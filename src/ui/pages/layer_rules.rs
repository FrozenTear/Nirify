//! Layer rules settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the layer rules settings page
pub fn layer_rules_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let rules = settings.layer_rules;

    let rule_count = rules.rules.len();

    Stack::vertical((
        section(
            "Layer Rules",
            Stack::vertical((
                Label::derived(move || {
                    if rule_count == 0 {
                        "No layer rules configured.".to_string()
                    } else {
                        format!("{} layer rule(s) configured.", rule_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
            )),
        ),
        section(
            "About Layer Rules",
            Stack::vertical((Label::derived(|| {
                "Layer rules customize behavior for layer surfaces like panels, \
                 notifications, and overlays. You can control opacity, shadows, \
                 and whether they appear in screen captures."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
