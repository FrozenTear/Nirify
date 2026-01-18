//! Switch events settings page (lid close/open, tablet mode)

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    icon_button_style, text_input_style, BG_ELEVATED, BORDER_SUBTLE, FONT_SIZE_BASE, FONT_SIZE_SM,
    RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
    TEXT_TERTIARY,
};

/// Create the switch events settings page
pub fn switch_events_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let events = settings.switch_events;

    // Create signals for each event's command
    let lid_close = RwSignal::new(events.lid_close.spawn.join(" "));
    let lid_open = RwSignal::new(events.lid_open.spawn.join(" "));
    let tablet_mode_on = RwSignal::new(events.tablet_mode_on.spawn.join(" "));
    let tablet_mode_off = RwSignal::new(events.tablet_mode_off.spawn.join(" "));

    Stack::vertical((
        section(
            "Lid Events",
            Stack::vertical((
                switch_event_row(
                    "Lid close",
                    "Command to run when laptop lid closes",
                    lid_close,
                    state.clone(),
                    |s, cmd| s.switch_events.lid_close.spawn = cmd,
                ),
                switch_event_row(
                    "Lid open",
                    "Command to run when laptop lid opens",
                    lid_open,
                    state.clone(),
                    |s, cmd| s.switch_events.lid_open.spawn = cmd,
                ),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "Tablet Mode Events",
            Stack::vertical((
                switch_event_row(
                    "Tablet mode on",
                    "Command to run when entering tablet mode",
                    tablet_mode_on,
                    state.clone(),
                    |s, cmd| s.switch_events.tablet_mode_on.spawn = cmd,
                ),
                switch_event_row(
                    "Tablet mode off",
                    "Command to run when exiting tablet mode",
                    tablet_mode_off,
                    state.clone(),
                    |s, cmd| s.switch_events.tablet_mode_off.spawn = cmd,
                ),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "About Switch Events",
            Stack::vertical((Label::derived(|| {
                "Switch events let you run commands when hardware switches change state. \
                 For example, lock the screen when closing the laptop lid, or change \
                 layout when entering tablet mode."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// A row for editing a switch event command
fn switch_event_row<F>(
    label: &'static str,
    description: &'static str,
    command_signal: RwSignal<String>,
    state: AppState,
    update_fn: F,
) -> impl IntoView
where
    F: Fn(&mut crate::config::Settings, Vec<String>) + Clone + 'static,
{
    let update_fn = Rc::new(update_fn);
    let update_fn_blur = update_fn.clone();
    let update_fn_clear = update_fn.clone();

    let state_blur = state.clone();
    let state_clear = state.clone();

    let on_change = move || {
        let cmd_str = command_signal.get();
        let parts: Vec<String> = if cmd_str.trim().is_empty() {
            vec![]
        } else {
            shell_words::split(&cmd_str)
                .unwrap_or_else(|_| cmd_str.split_whitespace().map(String::from).collect())
        };

        state_blur.update_settings(|s| {
            update_fn_blur(s, parts);
        });
        state_blur.mark_dirty_and_save(SettingsCategory::SwitchEvents);
    };

    let on_clear = move || {
        command_signal.set(String::new());
        state_clear.update_settings(|s| {
            update_fn_clear(s, vec![]);
        });
        state_clear.mark_dirty_and_save(SettingsCategory::SwitchEvents);
    };

    Stack::vertical((
        // Label and description
        Stack::vertical((
            Label::derived(move || label.to_string())
                .style(|s| s.color(TEXT_PRIMARY).font_size(FONT_SIZE_BASE)),
            Label::derived(move || description.to_string())
                .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.gap(SPACING_SM / 2.0)),
        // Command input row
        Stack::horizontal((
            text_input(command_signal)
                .placeholder("command --with-args (leave empty for no action)")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .flex_grow(1.0)
                        .font_family("monospace".to_string())
                }),
            // Clear button
            Container::new(
                Label::derived(|| "✕".to_string())
                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(TEXT_SECONDARY)))
            .on_click_stop(move |_| on_clear()),
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
        }),
    ))
    .style(|s| s.width_full().gap(SPACING_SM))
}
