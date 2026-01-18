//! Keybindings settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::{KeybindAction, Keybinding};
use crate::config::SettingsCategory;
use crate::ui::components::section;
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, ACCENT, BG_ELEVATED, BG_SURFACE,
    BORDER_SUBTLE, ERROR, FONT_SIZE_SM, RADIUS_MD, RADIUS_SM, SPACING_LG, SPACING_MD, SPACING_SM,
    SPACING_XS, TEXT_MUTED, TEXT_SECONDARY, TEXT_TERTIARY, WARNING,
};

/// Create the keybindings settings page
pub fn keybindings_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let error = settings.keybindings.error.clone();

    // Create signals for keybindings list
    let bindings = RwSignal::new(settings.keybindings.bindings.clone());
    let next_id = RwSignal::new(
        settings
            .keybindings
            .bindings
            .iter()
            .map(|b| b.id)
            .max()
            .unwrap_or(0)
            + 1,
    );

    Stack::vertical((
        // Error section if loading failed
        if let Some(err) = error {
            section(
                "Error",
                Stack::vertical((Label::derived(move || {
                    format!("Failed to load keybindings: {}", err.clone())
                })
                .style(|s| s.color(WARNING)),)),
            )
            .into_any()
        } else {
            floem::views::Empty::new().into_any()
        },
        section(
            "Keyboard Shortcuts",
            Stack::vertical((
                // List of existing keybindings
                keybinding_list(state.clone(), bindings, next_id),
                // Add button
                add_keybinding_button(state.clone(), bindings, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "About Keybindings",
            Stack::vertical((Label::derived(|| {
                "Use Mod+Key format for shortcuts (e.g., 'Mod+Space', 'Mod+Shift+Q'). \
                 Actions can be 'spawn <command>' to run programs, or built-in niri actions \
                 like 'close-window', 'toggle-overview', 'focus-workspace browser', etc."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// List of keybinding rows
fn keybinding_list(
    state: AppState,
    bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    floem::views::dyn_container(
        move || bindings.get(),
        move |bind_list| {
            if bind_list.is_empty() {
                Label::derived(|| "No keybindings configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else {
                Stack::vertical(
                    bind_list
                        .into_iter()
                        .map(|bind| keybinding_row(state.clone(), bind, bindings, next_id))
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_SM))
                .into_any()
            }
        },
    )
}

/// Single keybinding row with expandable details
fn keybinding_row(
    state: AppState,
    bind: Keybinding,
    bindings: RwSignal<Vec<Keybinding>>,
    _next_id: RwSignal<u32>,
) -> impl IntoView {
    let bind_id = bind.id;
    let key_combo_signal = RwSignal::new(bind.key_combo.clone());
    let action_signal = RwSignal::new(action_to_string(&bind.action));
    let expanded = RwSignal::new(false);

    // Save helper
    let save = {
        let state = state.clone();
        Rc::new(move || {
            state.update_settings(|s| {
                s.keybindings.bindings = bindings.get();
                s.keybindings.loaded = true;
            });
            state.mark_dirty_and_save(SettingsCategory::Keybindings);
        })
    };

    // Key combo change
    let save_key = save.clone();
    let on_key_change = move || {
        bindings.update(|b_list| {
            if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                b.key_combo = key_combo_signal.get();
            }
        });
        save_key();
    };

    // Action change
    let save_action = save.clone();
    let on_action_change = move || {
        let action_str = action_signal.get();
        let action = string_to_action(&action_str);
        bindings.update(|b_list| {
            if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                b.action = action;
            }
        });
        save_action();
    };

    // Delete callback
    let state_delete = state.clone();
    let on_delete = move || {
        bindings.update(|b_list| {
            b_list.retain(|b| b.id != bind_id);
        });
        state_delete.update_settings(|s| {
            s.keybindings.bindings = bindings.get();
            s.keybindings.loaded = true;
        });
        state_delete.mark_dirty_and_save(SettingsCategory::Keybindings);
    };

    Stack::vertical((
        // Header row
        Stack::horizontal((
            // Key combo input
            text_input(key_combo_signal)
                .placeholder("Mod+Key")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_key_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .width(150.0)
                        .font_family("monospace".to_string())
                }),
            // Action input
            text_input(action_signal)
                .placeholder("close-window or spawn <app>")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_action_change();
                })
                .style(|s| {
                    text_input_style(s)
                        .flex_grow(1.0)
                        .font_family("monospace".to_string())
                }),
            // Expand button
            Container::new(
                Label::derived(move || if expanded.get() { "▼" } else { "▶" }.to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(ACCENT)))
            .on_click_stop(move |_| expanded.set(!expanded.get())),
            // Delete button
            Container::new(
                Label::derived(|| "✕".to_string())
                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
            )
            .style(|s| {
                icon_button_style(s).hover(|s| s.background(ERROR.with_alpha(0.2)).color(ERROR))
            })
            .on_click_stop(move |_| on_delete()),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Expanded settings
        {
            let save = save.clone();

            floem::views::dyn_container(
                move || expanded.get(),
                move |is_expanded| {
                    if is_expanded {
                        keybinding_details(bind_id, bind.clone(), bindings, save.clone()).into_any()
                    } else {
                        floem::views::Empty::new().into_any()
                    }
                },
            )
        },
    ))
    .style(|s| {
        s.width_full()
            .padding(SPACING_SM)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_MD)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// Expanded keybinding details
fn keybinding_details(
    bind_id: u32,
    bind: Keybinding,
    bindings: RwSignal<Vec<Keybinding>>,
    save: Rc<dyn Fn()>,
) -> impl IntoView {
    let allow_locked = RwSignal::new(bind.allow_when_locked);
    let repeat = RwSignal::new(bind.repeat);
    let title_signal = RwSignal::new(bind.hotkey_overlay_title.clone().unwrap_or_default());

    let save_locked = save.clone();
    let save_repeat = save.clone();
    let save_title = save.clone();

    Stack::vertical((
        // Hotkey overlay title
        Stack::horizontal((
            Label::derived(|| "Title".to_string()).style(|s| {
                s.color(TEXT_TERTIARY)
                    .font_size(FONT_SIZE_SM)
                    .min_width(100.0)
            }),
            text_input(title_signal)
                .placeholder("(shown in hotkey overlay)")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    let title = title_signal.get();
                    bindings.update(|b_list| {
                        if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                            b.hotkey_overlay_title = if title.is_empty() {
                                None
                            } else {
                                Some(title.clone())
                            };
                        }
                    });
                    save_title();
                })
                .style(|s| text_input_style(s).flex_grow(1.0)),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Options row
        Stack::horizontal((
            // Allow when locked toggle
            option_chip("Allow locked", allow_locked, move |val| {
                bindings.update(|b_list| {
                    if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                        b.allow_when_locked = val;
                    }
                });
                save_locked();
            }),
            // Repeat toggle
            option_chip("Repeat", repeat, move |val| {
                bindings.update(|b_list| {
                    if let Some(b) = b_list.iter_mut().find(|b| b.id == bind_id) {
                        b.repeat = val;
                    }
                });
                save_repeat();
            }),
        ))
        .style(|s| s.gap(SPACING_SM)),
    ))
    .style(|s| {
        s.width_full()
            .gap(SPACING_SM)
            .padding_top(SPACING_SM)
            .border_top(1.0)
            .border_color(BORDER_SUBTLE)
            .margin_top(SPACING_SM)
    })
}

/// Small toggle chip for options
fn option_chip<F>(label: &'static str, value: RwSignal<bool>, on_change: F) -> impl IntoView
where
    F: Fn(bool) + 'static,
{
    let is_on = move || value.get();

    Container::new(Label::derived(move || label.to_string()).style(move |s| {
        let base = s
            .font_size(FONT_SIZE_SM)
            .padding_horiz(SPACING_SM)
            .padding_vert(SPACING_XS);
        if is_on() {
            base.color(ACCENT)
        } else {
            base.color(TEXT_MUTED)
        }
    }))
    .style(move |s| {
        let base = s.border_radius(RADIUS_SM).border(1.0);
        if is_on() {
            base.background(ACCENT.with_alpha(0.15))
                .border_color(ACCENT)
        } else {
            base.background(BG_SURFACE)
                .border_color(BORDER_SUBTLE)
                .hover(|s| s.border_color(TEXT_MUTED))
        }
    })
    .on_click_stop(move |_| {
        let new_val = !value.get();
        value.set(new_val);
        on_change(new_val);
    })
}

/// Add new keybinding button
fn add_keybinding_button(
    state: AppState,
    bindings: RwSignal<Vec<Keybinding>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let new_bind = Keybinding {
            id,
            key_combo: String::new(),
            hotkey_overlay_title: None,
            allow_when_locked: false,
            cooldown_ms: None,
            repeat: false,
            action: KeybindAction::NiriAction(String::new()),
        };

        bindings.update(|b_list| {
            b_list.push(new_bind);
        });

        state.update_settings(|s| {
            s.keybindings.bindings = bindings.get();
            s.keybindings.loaded = true;
        });
        state.mark_dirty_and_save(SettingsCategory::Keybindings);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Keybinding".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}

/// Convert action to editable string
fn action_to_string(action: &KeybindAction) -> String {
    match action {
        KeybindAction::Spawn(args) => {
            if args.is_empty() {
                "spawn".to_string()
            } else {
                format!("spawn {}", shell_words::join(args))
            }
        }
        KeybindAction::NiriAction(action) => action.clone(),
        KeybindAction::NiriActionWithArgs(action, args) => {
            format!("{} {}", action, args.join(" "))
        }
    }
}

/// Parse string back to action
fn string_to_action(s: &str) -> KeybindAction {
    let trimmed = s.trim();
    if trimmed.starts_with("spawn ") {
        let cmd = trimmed.strip_prefix("spawn ").unwrap_or("");
        let args = shell_words::split(cmd).unwrap_or_else(|_| vec![cmd.to_string()]);
        KeybindAction::Spawn(args)
    } else if trimmed == "spawn" {
        KeybindAction::Spawn(vec![])
    } else if trimmed.contains(' ') {
        // Action with args
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let args: Vec<String> = parts[1].split_whitespace().map(String::from).collect();
            KeybindAction::NiriActionWithArgs(parts[0].to_string(), args)
        } else {
            KeybindAction::NiriAction(trimmed.to_string())
        }
    } else {
        KeybindAction::NiriAction(trimmed.to_string())
    }
}
