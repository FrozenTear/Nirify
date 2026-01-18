//! Workspaces settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::NamedWorkspace;
use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, BG_ELEVATED, BORDER_SUBTLE, ERROR,
    FONT_SIZE_SM, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, TEXT_MUTED,
    TEXT_SECONDARY, TEXT_TERTIARY,
};

/// Create the workspaces settings page
pub fn workspaces_page(state: AppState) -> impl IntoView {
    // Create signals for workspaces list
    let workspaces = RwSignal::new(state.get_settings().workspaces.workspaces.clone());
    let next_id = RwSignal::new(state.get_settings().workspaces.next_id);

    Stack::vertical((
        section(
            "Named Workspaces",
            Stack::vertical((
                // List of existing workspaces
                workspace_list(state.clone(), workspaces, next_id),
                // Add button
                add_workspace_button(state.clone(), workspaces, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "Info",
            Stack::vertical((Label::derived(|| {
                "Named workspaces can be pinned to specific outputs \
                 and accessed by name in keybindings. Leave 'Output' empty \
                 to allow the workspace on any monitor."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// List of workspace rows
fn workspace_list(
    state: AppState,
    workspaces: RwSignal<Vec<NamedWorkspace>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    floem::views::dyn_container(
        move || workspaces.get(),
        move |ws_list| {
            if ws_list.is_empty() {
                Label::derived(|| "No named workspaces configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else {
                Stack::vertical(
                    ws_list
                        .into_iter()
                        .map(|ws| workspace_row(state.clone(), ws, workspaces, next_id))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_SM))
                .into_any()
            }
        },
    )
}

/// Single workspace row
fn workspace_row(
    state: AppState,
    ws: NamedWorkspace,
    workspaces: RwSignal<Vec<NamedWorkspace>>,
    _next_id: RwSignal<u32>,
) -> impl IntoView {
    let ws_id = ws.id;
    let name_signal = RwSignal::new(ws.name.clone());
    let output_signal = RwSignal::new(ws.open_on_output.clone().unwrap_or_default());

    // Save helper
    let save = {
        let state = state.clone();
        Rc::new(move || {
            state.update_settings(|s| {
                s.workspaces.workspaces = workspaces.get();
            });
            state.mark_dirty_and_save(SettingsCategory::Workspaces);
        })
    };

    // Name change callback
    let save_name = save.clone();
    let on_name_change = move || {
        workspaces.update(|ws_list| {
            if let Some(w) = ws_list.iter_mut().find(|w| w.id == ws_id) {
                w.name = name_signal.get();
            }
        });
        save_name();
    };

    // Output change callback
    let save_output = save.clone();
    let on_output_change = move || {
        let output = output_signal.get();
        workspaces.update(|ws_list| {
            if let Some(w) = ws_list.iter_mut().find(|w| w.id == ws_id) {
                w.open_on_output = if output.is_empty() {
                    None
                } else {
                    Some(output.clone())
                };
            }
        });
        save_output();
    };

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        workspaces.update(|ws_list| {
            ws_list.retain(|w| w.id != ws_id);
        });
        state_delete.update_settings(|s| {
            s.workspaces.workspaces = workspaces.get();
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Workspaces);
    };

    Stack::horizontal((
        // Name input
        Stack::horizontal((
            Label::derived(|| "Name".to_string())
                .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            text_input(name_signal)
                .placeholder("Workspace Name")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_name_change();
                })
                .style(|s| text_input_style(s).width(150.0)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
        // Output input
        Stack::horizontal((
            Label::derived(|| "Output".to_string())
                .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            text_input(output_signal)
                .placeholder("Any (eDP-1, HDMI-A-1, etc.)")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_output_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .flex_grow(1.0)
                        .font_family("monospace".to_string())
                }),
        ))
        .style(|s| s.items_center().gap(SPACING_XS).flex_grow(1.0)),
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
            .gap(SPACING_MD)
            .padding(SPACING_SM)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_SM)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// Add new workspace button
fn add_workspace_button(
    state: AppState,
    workspaces: RwSignal<Vec<NamedWorkspace>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let new_ws = NamedWorkspace {
            id,
            name: format!("Workspace {}", id + 1),
            open_on_output: None,
            layout_override: None,
        };

        workspaces.update(|ws_list| {
            ws_list.push(new_ws);
        });

        state.update_settings(|s| {
            s.workspaces.workspaces = workspaces.get();
            s.workspaces.next_id = next_id.get();
        });
        state.mark_dirty_and_save(SettingsCategory::Workspaces);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Workspace".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}
