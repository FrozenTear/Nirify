//! Startup commands settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::StartupCommand;
use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, BG_ELEVATED, BORDER_SUBTLE, ERROR,
    FONT_SIZE_SM, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, TEXT_MUTED,
    TEXT_SECONDARY,
};

/// Create the startup settings page
pub fn startup_page(state: AppState) -> impl IntoView {
    // Create signals for the list of commands
    let commands = RwSignal::new(state.get_settings().startup.commands.clone());
    let next_id = RwSignal::new(state.get_settings().startup.next_id);

    Stack::vertical((
        section(
            "Startup Commands",
            Stack::vertical((
                // List of existing commands
                startup_command_list(state.clone(), commands, next_id),
                // Add button
                add_startup_command_button(state.clone(), commands, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
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

/// List of startup command rows
fn startup_command_list(
    state: AppState,
    commands: RwSignal<Vec<StartupCommand>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    floem::views::dyn_container(
        move || commands.get(),
        move |cmds| {
            if cmds.is_empty() {
                Label::derived(|| "No startup commands configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else {
                Stack::vertical(
                    cmds.into_iter()
                        .map(|cmd| startup_command_row(state.clone(), cmd, commands, next_id))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_SM))
                .into_any()
            }
        },
    )
}

/// Single startup command row
fn startup_command_row(
    state: AppState,
    cmd: StartupCommand,
    commands: RwSignal<Vec<StartupCommand>>,
    _next_id: RwSignal<u32>,
) -> impl IntoView {
    let cmd_id = cmd.id;
    // Join the command parts into a single string for editing
    let command_str = RwSignal::new(cmd.command.join(" "));

    // Callback to save when command changes
    let state_cmd = state.clone();
    let on_command_change = Rc::new(move || {
        let new_cmd_str = command_str.get();
        // Parse the command string - simple space-split for now
        // (a more robust solution would handle quoted strings)
        let parts: Vec<String> = shell_words::split(&new_cmd_str)
            .unwrap_or_else(|_| new_cmd_str.split_whitespace().map(String::from).collect());

        commands.update(|cmds| {
            if let Some(c) = cmds.iter_mut().find(|c| c.id == cmd_id) {
                c.command = parts;
            }
        });
        state_cmd.update_settings(|s| {
            s.startup.commands = commands.get();
        });
        state_cmd.mark_dirty_and_save(SettingsCategory::Startup);
    });

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        commands.update(|cmds| {
            cmds.retain(|c| c.id != cmd_id);
        });
        state_delete.update_settings(|s| {
            s.startup.commands = commands.get();
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Startup);
    };

    let on_command_change_blur = on_command_change.clone();

    Stack::horizontal((
        // Command input
        text_input(command_str)
            .placeholder("command --with-args")
            .on_event_stop(EventListener::FocusLost, move |_| {
                on_command_change_blur();
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

/// Add new startup command button
fn add_startup_command_button(
    state: AppState,
    commands: RwSignal<Vec<StartupCommand>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let new_cmd = StartupCommand {
            id,
            command: vec![],
        };

        commands.update(|cmds| {
            cmds.push(new_cmd);
        });

        state.update_settings(|s| {
            s.startup.commands = commands.get();
            s.startup.next_id = next_id.get();
        });
        state.mark_dirty_and_save(SettingsCategory::Startup);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Command".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}
