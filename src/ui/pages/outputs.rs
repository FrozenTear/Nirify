//! Outputs (displays) settings page

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the outputs settings page
pub fn outputs_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let outputs = settings.outputs;

    let output_count = outputs.outputs.len();

    Stack::vertical((
        section(
            "Connected Displays",
            Stack::vertical((
                Label::derived(move || {
                    if output_count == 0 {
                        "No displays configured.".to_string()
                    } else {
                        format!("{} display(s) configured.", output_count)
                    }
                })
                .style(|s| s.color(TEXT_SECONDARY)),
                // List configured outputs
                if output_count > 0 {
                    let output_names: Vec<String> = outputs
                        .outputs
                        .iter()
                        .map(|o| {
                            let mode = if o.mode.is_empty() { "auto".to_string() } else { o.mode.clone() };
                            let scale = o.scale;
                            format!("{}: {} @ {}x", o.name, mode, scale)
                        })
                        .collect();

                    Stack::vertical(
                        output_names
                            .into_iter()
                            .map(|name| {
                                Label::derived(move || name.clone())
                                    .style(|s| s.color(TEXT_SECONDARY).padding_vert(4.0))
                            })
                            .collect::<Vec<_>>(),
                    )
                    .into_any()
                } else {
                    floem::views::Empty::new().into_any()
                },
            )),
        ),
        section(
            "About Display Configuration",
            Stack::vertical((Label::derived(|| {
                "Display settings control resolution, refresh rate, scaling, \
                 rotation, and position for each monitor. Changes take effect \
                 immediately when saved."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
