//! Environment variables settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::EnvironmentVariable;
use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, BG_ELEVATED, BORDER_SUBTLE, ERROR,
    FONT_SIZE_SM, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, TEXT_MUTED,
    TEXT_SECONDARY, TEXT_TERTIARY,
};

/// Create the environment settings page
pub fn environment_page(state: AppState) -> impl IntoView {
    // Create a signal to hold the list of variables (for reactivity)
    let variables = RwSignal::new(state.get_settings().environment.variables.clone());
    let next_id = RwSignal::new(state.get_settings().environment.next_id);

    Stack::vertical((
        section(
            "Environment Variables",
            Stack::vertical((
                // List of existing variables
                env_var_list(state.clone(), variables, next_id),
                // Add button
                add_env_var_button(state.clone(), variables, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
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

/// List of environment variable rows
fn env_var_list(
    state: AppState,
    variables: RwSignal<Vec<EnvironmentVariable>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    floem::views::dyn_container(
        move || variables.get(),
        move |vars| {
            if vars.is_empty() {
                Label::derived(|| "No environment variables configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else {
                Stack::vertical(
                    vars.into_iter()
                        .map(|var| env_var_row(state.clone(), var, variables, next_id))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_SM))
                .into_any()
            }
        },
    )
}

/// Single environment variable row with name=value editing
fn env_var_row(
    state: AppState,
    var: EnvironmentVariable,
    variables: RwSignal<Vec<EnvironmentVariable>>,
    _next_id: RwSignal<u32>,
) -> impl IntoView {
    let var_id = var.id;
    let name_signal = RwSignal::new(var.name.clone());
    let value_signal = RwSignal::new(var.value.clone());

    // Callback to save when name changes
    let state_name = state.clone();
    let on_name_change = Rc::new(move || {
        let new_name = name_signal.get();
        variables.update(|vars| {
            if let Some(v) = vars.iter_mut().find(|v| v.id == var_id) {
                v.name = new_name.clone();
            }
        });
        state_name.update_settings(|s| {
            s.environment.variables = variables.get();
        });
        state_name.mark_dirty_and_save(SettingsCategory::Environment);
    });

    // Callback to save when value changes
    let state_value = state.clone();
    let on_value_change = Rc::new(move || {
        let new_value = value_signal.get();
        variables.update(|vars| {
            if let Some(v) = vars.iter_mut().find(|v| v.id == var_id) {
                v.value = new_value.clone();
            }
        });
        state_value.update_settings(|s| {
            s.environment.variables = variables.get();
        });
        state_value.mark_dirty_and_save(SettingsCategory::Environment);
    });

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        variables.update(|vars| {
            vars.retain(|v| v.id != var_id);
        });
        state_delete.update_settings(|s| {
            s.environment.variables = variables.get();
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Environment);
    };

    let on_name_change_blur = on_name_change.clone();
    let on_value_change_blur = on_value_change.clone();

    Stack::horizontal((
        // Name input
        text_input(name_signal)
            .placeholder("VAR_NAME")
            .on_event_stop(EventListener::FocusLost, move |_| {
                on_name_change_blur();
            })
            .style(|s| {
                text_input_style(s)
                    .width(150.0)
                    .font_family("monospace".to_string())
            }),
        // Equals sign
        Label::derived(|| "=".to_string())
            .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
        // Value input
        text_input(value_signal)
            .placeholder("value")
            .on_event_stop(EventListener::FocusLost, move |_| {
                on_value_change_blur();
            })
            .style(|s| {
                text_input_style(s)
                    .flex_grow(1.0)
                    .font_family("monospace".to_string())
            }),
        // Delete button
        Container::new(
            Label::derived(|| "✕".to_string())
                .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
        )
        .style(|s| icon_button_style(s).hover(|s| s.background(ERROR.with_alpha(0.2)).color(ERROR)))
        .on_click_stop(move |_| on_delete()),
    ))
    .style(|s| {
        s.width_full()
            .items_center()
            .gap(SPACING_SM)
            .padding(SPACING_SM)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_SM)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// Add new environment variable button
fn add_env_var_button(
    state: AppState,
    variables: RwSignal<Vec<EnvironmentVariable>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let new_var = EnvironmentVariable {
            id,
            name: String::new(),
            value: String::new(),
        };

        variables.update(|vars| {
            vars.push(new_var);
        });

        state.update_settings(|s| {
            s.environment.variables = variables.get();
            s.environment.next_id = next_id.get();
        });
        state.mark_dirty_and_save(SettingsCategory::Environment);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Variable".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}
