//! Window rules settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Empty, Label, Stack};
use std::collections::HashSet;
use std::rc::Rc;

use crate::config::models::{OpenBehavior, WindowRule, WindowRuleMatch};
use crate::config::SettingsCategory;
use crate::ipc::get_focused_window;
use crate::ui::components::{
    section, slider_row_with_callback, toggle_row_with_callback, window_picker_modal,
    WindowPickerState,
};
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, theme, ACCENT, BG_ELEVATED,
    BG_SURFACE, BORDER_SUBTLE, ERROR, FONT_SIZE_BASE, FONT_SIZE_SM, RADIUS_MD, RADIUS_SM,
    SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
    TEXT_TERTIARY,
};

/// Create the window rules settings page
pub fn window_rules_page(state: AppState) -> impl IntoView {
    // Create signals for rules list
    let rules = RwSignal::new(state.get_settings().window_rules.rules.clone());
    let next_id = RwSignal::new(state.get_settings().window_rules.next_id);

    // Signal to store the callback for when a window is picked
    // Each row sets this before opening the picker
    let picker_on_select: RwSignal<Option<Rc<dyn Fn(String, String)>>> = RwSignal::new(None);

    // Create window picker state with callback that invokes the stored callback
    let picker_state = WindowPickerState::new(move |app_id, title| {
        if let Some(callback) = picker_on_select.get() {
            callback(app_id, title);
        }
    });

    // Page content
    let page_content = Stack::vertical((
        section(
            "Window Rules",
            Stack::vertical((
                // List of existing rules
                window_rule_list(
                    state.clone(),
                    rules,
                    next_id,
                    picker_state.clone(),
                    picker_on_select,
                ),
                // Add button
                add_window_rule_button(state.clone(), rules, next_id),
            ))
            .style(|s| s.width_full().gap(SPACING_MD)),
        ),
        section(
            "About Window Rules",
            Stack::vertical((Label::derived(|| {
                "Window rules let you customize behavior for specific windows. \
                 Each rule has match criteria (app-id, title) and actions to apply. \
                 Rules are evaluated in order - later rules can override earlier ones."
                    .to_string()
            })
            .style(|s| s.color(TEXT_SECONDARY)),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG));

    // Stack with relative positioning so modal can overlay
    // Position::Absolute is applied inside window_picker_modal only when visible
    Stack::new((
        page_content,
        window_picker_modal(picker_state.clone()),
    ))
    .style(|s| {
        s.width_full()
            .height_full()
            .position(floem::style::Position::Relative)
    })
}

/// List of window rule cards
fn window_rule_list(
    state: AppState,
    rules: RwSignal<Vec<WindowRule>>,
    next_id: RwSignal<u32>,
    picker_state: WindowPickerState,
    picker_on_select: RwSignal<Option<Rc<dyn Fn(String, String)>>>,
) -> impl IntoView {
    // Track expanded state at the list level so it persists across rule changes
    let expanded_rules: RwSignal<HashSet<u32>> = RwSignal::new(HashSet::new());

    floem::views::dyn_container(
        move || rules.get(),
        move |rule_list| {
            if rule_list.is_empty() {
                Label::derived(|| "No window rules configured.".to_string())
                    .style(|s| s.color(TEXT_MUTED).padding_vert(SPACING_MD))
                    .into_any()
            } else {
                Stack::vertical(
                    rule_list
                        .into_iter()
                        .enumerate()
                        .map(|(idx, rule)| {
                            window_rule_card(
                                state.clone(),
                                idx,
                                rule,
                                rules,
                                next_id,
                                expanded_rules,
                                picker_state.clone(),
                                picker_on_select,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_MD))
                .into_any()
            }
        },
    )
}

/// Single window rule card - rule-centric design with multiple matches support
fn window_rule_card(
    state: AppState,
    index: usize,
    rule: WindowRule,
    rules: RwSignal<Vec<WindowRule>>,
    _next_id: RwSignal<u32>,
    expanded_rules: RwSignal<HashSet<u32>>,
    picker_state: WindowPickerState,
    picker_on_select: RwSignal<Option<Rc<dyn Fn(String, String)>>>,
) -> impl IntoView {
    let rule_id = rule.id;

    // Rule name signal
    let name_signal = RwSignal::new(rule.name.clone());

    // Store all matches in a signal for reactivity
    let matches_signal = RwSignal::new(rule.matches.clone());

    // Settings signals
    let open_behavior_idx = RwSignal::new(open_behavior_to_index(rule.open_behavior));
    let opacity = RwSignal::new(rule.opacity.unwrap_or(1.0) as f64);
    let has_opacity = RwSignal::new(rule.opacity.is_some());
    let block_screencast = RwSignal::new(rule.block_out_from_screencast);
    let workspace = RwSignal::new(rule.open_on_workspace.clone().unwrap_or_default());
    let output = RwSignal::new(rule.open_on_output.clone().unwrap_or_default());

    // Helper to toggle expanded state for this rule
    let toggle_expanded = move || {
        expanded_rules.update(|set| {
            if set.contains(&rule_id) {
                set.remove(&rule_id);
            } else {
                set.insert(rule_id);
            }
        });
    };

    // Check if this rule is expanded (using the shared expanded_rules set)
    let check_expanded = move || expanded_rules.get().contains(&rule_id);

    // Save helper
    let save = {
        let state = state.clone();
        Rc::new(move || {
            state.update_settings(|s| {
                s.window_rules.rules = rules.get();
            });
            state.mark_dirty_and_save(SettingsCategory::WindowRules);
        })
    };

    // Name change
    let save_name = save.clone();
    let on_name_change = move || {
        rules.update(|r_list| {
            if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                r.name = name_signal.get();
            }
        });
        save_name();
    };

    // Delete rule callback
    let state_delete = state.clone();
    let on_delete = move || {
        rules.update(|r_list| {
            r_list.retain(|r| r.id != rule_id);
        });
        state_delete.update_settings(|s| {
            s.window_rules.rules = rules.get();
        });
        state_delete.mark_dirty_and_save(SettingsCategory::WindowRules);
    };

    // Build summary of what the rule matches
    let match_count = rule.matches.len();
    let match_summary = if match_count == 0 {
        "all windows".to_string()
    } else if match_count == 1 {
        let m = &rule.matches[0];
        match (&m.app_id, &m.title) {
            (Some(a), Some(t)) => format!("app:{} title:{}", a, t),
            (Some(a), None) => format!("app:{}", a),
            (None, Some(t)) => format!("title:{}", t),
            (None, None) => "all windows".to_string(),
        }
    } else {
        format!("{} match criteria", match_count)
    };

    Stack::vertical((
        // Header row - Rule number, name, summary
        Stack::horizontal((
            // Rule number badge
            Container::new(
                Label::derived(move || format!("{}", index + 1))
                    .style(|s| s.color(ACCENT).font_size(FONT_SIZE_SM).font_bold()),
            )
            .style(|s| {
                s.width(24.0)
                    .height(24.0)
                    .border_radius(RADIUS_SM)
                    .background(ACCENT.with_alpha(0.15))
                    .items_center()
                    .justify_center()
            }),
            // Rule name (editable)
            text_input(name_signal)
                .placeholder("Rule name")
                .on_event_stop(EventListener::FocusLost, move |_| {
                    on_name_change();
                })
                .style(|s| {
                    s.width(150.0)
                        .padding_vert(SPACING_XS)
                        .padding_horiz(SPACING_SM)
                        .background(BG_ELEVATED)
                        .border_radius(RADIUS_SM)
                        .border(1.0)
                        .border_color(BORDER_SUBTLE)
                        .color(TEXT_PRIMARY)
                        .font_size(FONT_SIZE_BASE)
                }),
            // Match summary
            Label::derived(move || match_summary.clone()).style(|s| {
                s.color(TEXT_TERTIARY)
                    .font_size(FONT_SIZE_SM)
                    .flex_grow(1.0)
            }),
            // Expand button
            Container::new(
                Label::derived(move || if check_expanded() { "▼" } else { "▶" }.to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(ACCENT)))
            .on_click_stop(move |_| toggle_expanded()),
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
                move || check_expanded(),
                move |rule_expanded| {
                    let save = save.clone();

                    if rule_expanded {
                        let save_behavior = save.clone();
                        let save_workspace = save.clone();
                        let save_output = save.clone();
                        let save_opacity = save.clone();
                        let save_has_opacity = save.clone();
                        let save_screencast = save.clone();
                        let save_matches = save.clone();

                        Stack::vertical((
                            // Match Criteria section
                            Stack::horizontal((
                                Label::derived(|| "Match Criteria".to_string())
                                    .style(|s| s.color(ACCENT).font_size(FONT_SIZE_SM).font_bold()),
                                Label::derived(|| "(rule applies if ANY match)".to_string())
                                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
                            ))
                            .style(|s| s.gap(SPACING_SM).margin_bottom(SPACING_XS)),
                            // List of all matches
                            match_criteria_list(
                                rule_id,
                                matches_signal,
                                rules,
                                save_matches.clone(),
                                picker_state.clone(),
                                picker_on_select,
                            ),
                            // Add match button
                            {
                                let save = save_matches.clone();
                                Container::new(
                                    Label::derived(|| "+ Add Match".to_string())
                                        .style(|s| s.color(ACCENT).font_size(FONT_SIZE_SM)),
                                )
                                .style(|s| {
                                    s.padding_horiz(SPACING_SM)
                                        .padding_vert(SPACING_XS)
                                        .border_radius(RADIUS_SM)
                                        .hover(|s| s.background(ACCENT.with_alpha(0.1)))
                                })
                                .on_click_stop(move |_| {
                                    matches_signal.update(|m| {
                                        m.push(WindowRuleMatch::default());
                                    });
                                    rules.update(|r_list| {
                                        if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id)
                                        {
                                            r.matches = matches_signal.get();
                                        }
                                    });
                                    save();
                                })
                            },
                            // Actions section
                            Label::derived(|| "Actions".to_string()).style(|s| {
                                s.color(ACCENT)
                                    .font_size(FONT_SIZE_SM)
                                    .font_bold()
                                    .margin_top(SPACING_MD)
                                    .margin_bottom(SPACING_XS)
                            }),
                            // Open behavior
                            Stack::horizontal((
                                Label::derived(|| "Open as".to_string()).style(|s| {
                                    s.color(TEXT_SECONDARY)
                                        .font_size(FONT_SIZE_SM)
                                        .min_width(80.0)
                                }),
                                behavior_selector(open_behavior_idx, rules, rule_id, save_behavior),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                            // Open on workspace
                            workspace_selector(
                                state.clone(),
                                workspace,
                                rules,
                                rule_id,
                                save_workspace,
                            ),
                            // Open on output
                            output_selector(
                                state.clone(),
                                output,
                                rules,
                                rule_id,
                                save_output,
                            ),
                            // Opacity
                            Stack::horizontal((
                                option_chip("Custom Opacity", has_opacity, move |val| {
                                    rules.update(|r_list| {
                                        if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id)
                                        {
                                            r.opacity = if val {
                                                Some(opacity.get() as f32)
                                            } else {
                                                None
                                            };
                                        }
                                    });
                                    save_has_opacity();
                                }),
                                floem::views::dyn_container(
                                    move || has_opacity.get(),
                                    move |show_opacity| {
                                        if show_opacity {
                                            let save = save_opacity.clone();
                                            slider_row_with_callback(
                                                "",
                                                None,
                                                opacity,
                                                0.0,
                                                1.0,
                                                0.05,
                                                "",
                                                Some(Rc::new(move |val: f64| {
                                                    rules.update(|r_list| {
                                                        if let Some(r) = r_list
                                                            .iter_mut()
                                                            .find(|r| r.id == rule_id)
                                                        {
                                                            r.opacity = Some(val as f32);
                                                        }
                                                    });
                                                    save();
                                                })),
                                            )
                                            .into_any()
                                        } else {
                                            floem::views::Empty::new().into_any()
                                        }
                                    },
                                ),
                            ))
                            .style(|s| s.width_full().items_center().gap(SPACING_SM)),
                            // Block screencast
                            toggle_row_with_callback(
                                "Block from screencast",
                                Some("Hide this window from screen recordings"),
                                block_screencast,
                                Some(Rc::new(move |val: bool| {
                                    rules.update(|r_list| {
                                        if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id)
                                        {
                                            r.block_out_from_screencast = val;
                                        }
                                    });
                                    save_screencast();
                                })),
                            ),
                        ))
                        .style(|s| {
                            s.width_full()
                                .gap(SPACING_SM)
                                .padding_top(SPACING_MD)
                                .border_top(1.0)
                                .border_color(BORDER_SUBTLE)
                                .margin_top(SPACING_SM)
                        })
                        .into_any()
                    } else {
                        floem::views::Empty::new().into_any()
                    }
                },
            )
        },
    ))
    .style(|s| {
        s.width_full()
            .padding(SPACING_MD)
            .background(BG_ELEVATED)
            .border_radius(RADIUS_MD)
            .border(1.0)
            .border_color(BORDER_SUBTLE)
    })
}

/// List of match criteria for a rule
fn match_criteria_list(
    rule_id: u32,
    matches_signal: RwSignal<Vec<WindowRuleMatch>>,
    rules: RwSignal<Vec<WindowRule>>,
    save: Rc<dyn Fn()>,
    picker_state: WindowPickerState,
    picker_on_select: RwSignal<Option<Rc<dyn Fn(String, String)>>>,
) -> impl IntoView {
    floem::views::dyn_container(
        move || matches_signal.get(),
        move |matches| {
            let save = save.clone();
            if matches.is_empty() {
                Label::derived(|| "No match criteria (matches all windows)".to_string())
                    .style(|s| {
                        s.color(TEXT_MUTED)
                            .font_size(FONT_SIZE_SM)
                            .padding_vert(SPACING_XS)
                    })
                    .into_any()
            } else {
                Stack::vertical(
                    matches
                        .into_iter()
                        .enumerate()
                        .map(|(idx, m)| {
                            match_criteria_row(
                                rule_id,
                                idx,
                                m,
                                matches_signal,
                                rules,
                                save.clone(),
                                picker_state.clone(),
                                picker_on_select,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .style(|s| s.width_full().gap(SPACING_XS))
                .into_any()
            }
        },
    )
}

/// Single match criteria row
fn match_criteria_row(
    rule_id: u32,
    match_idx: usize,
    m: WindowRuleMatch,
    matches_signal: RwSignal<Vec<WindowRuleMatch>>,
    rules: RwSignal<Vec<WindowRule>>,
    save: Rc<dyn Fn()>,
    picker_state: WindowPickerState,
    picker_on_select: RwSignal<Option<Rc<dyn Fn(String, String)>>>,
) -> impl IntoView {
    let app_id = RwSignal::new(m.app_id.clone().unwrap_or_default());
    let title = RwSignal::new(m.title.clone().unwrap_or_default());

    let save_app = save.clone();
    let save_title = save.clone();
    let save_delete = save.clone();

    // Helper to update signals and save - used by Detect and Browse buttons
    let update_and_save: Rc<dyn Fn(String, String)> = {
        let save = save.clone();
        Rc::new(move |app: String, ttl: String| {
            app_id.set(app.clone());
            title.set(ttl.clone());
            matches_signal.update(|matches| {
                if let Some(m) = matches.get_mut(match_idx) {
                    m.app_id = if app.is_empty() { None } else { Some(app) };
                    m.title = if ttl.is_empty() { None } else { Some(ttl) };
                }
            });
            rules.update(|r_list| {
                if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                    r.matches = matches_signal.get();
                }
            });
            save();
        })
    };

    Stack::vertical((
        // First row: inputs
        Stack::horizontal((
            // Match number
            Label::derived(move || format!("{}.", match_idx + 1))
                .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM).min_width(20.0)),
            // App ID
            Stack::horizontal((
                Label::derived(|| "app:".to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
                text_input(app_id)
                    .placeholder("(any)")
                    .on_event_stop(EventListener::FocusLost, move |_| {
                        let app = app_id.get();
                        matches_signal.update(|matches| {
                            if let Some(m) = matches.get_mut(match_idx) {
                                m.app_id = if app.is_empty() {
                                    None
                                } else {
                                    Some(app.clone())
                                };
                            }
                        });
                        rules.update(|r_list| {
                            if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                                r.matches = matches_signal.get();
                            }
                        });
                        save_app();
                    })
                    .style(|s| {
                        text_input_style(s)
                            .width(120.0)
                            .font_family("monospace".to_string())
                    }),
            ))
            .style(|s| s.items_center().gap(SPACING_XS)),
            // Title
            Stack::horizontal((
                Label::derived(|| "title:".to_string())
                    .style(|s| s.color(TEXT_TERTIARY).font_size(FONT_SIZE_SM)),
                text_input(title)
                    .placeholder("(any)")
                    .on_event_stop(EventListener::FocusLost, move |_| {
                        let t = title.get();
                        matches_signal.update(|matches| {
                            if let Some(m) = matches.get_mut(match_idx) {
                                m.title = if t.is_empty() { None } else { Some(t.clone()) };
                            }
                        });
                        rules.update(|r_list| {
                            if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                                r.matches = matches_signal.get();
                            }
                        });
                        save_title();
                    })
                    .style(|s| {
                        text_input_style(s)
                            .flex_grow(1.0)
                            .font_family("monospace".to_string())
                    }),
            ))
            .style(|s| s.items_center().gap(SPACING_XS).flex_grow(1.0)),
            // Delete match button
            Container::new(
                Label::derived(|| "✕".to_string())
                    .style(|s| s.color(TEXT_MUTED).font_size(FONT_SIZE_SM)),
            )
            .style(|s| icon_button_style(s).hover(|s| s.color(ERROR)))
            .on_click_stop(move |_| {
                matches_signal.update(|matches| {
                    if match_idx < matches.len() {
                        matches.remove(match_idx);
                    }
                });
                rules.update(|r_list| {
                    if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                        r.matches = matches_signal.get();
                    }
                });
                save_delete();
            }),
        ))
        .style(|s| s.width_full().items_center().gap(SPACING_SM)),
        // Second row: Detect and Browse buttons
        Stack::horizontal((
            Empty::new().style(|s| s.min_width(20.0)), // Spacer to align with inputs
            // Detect button - grabs focused window
            {
                let update_and_save = update_and_save.clone();
                Container::new(
                    Stack::horizontal((
                        Label::derived(|| "◎".to_string())
                            .style(|s| s.font_size(FONT_SIZE_SM)),
                        Label::derived(|| "Detect".to_string())
                            .style(|s| s.font_size(FONT_SIZE_SM)),
                    ))
                    .style(|s| s.gap(SPACING_XS).items_center()),
                )
                .style(move |s| {
                    let t = theme();
                    s.padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS)
                        .border_radius(RADIUS_SM)
                        .color(t.text_muted)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(|s| s.background(t.hover_bg).color(t.text_secondary))
                })
                .on_click_stop(move |_| {
                    // Fetch focused window synchronously
                    match get_focused_window() {
                        Ok(Some(window)) => {
                            update_and_save(window.app_id.clone(), window.title.clone());
                        }
                        Ok(None) => {
                            // No focused window - could show feedback but keep simple
                        }
                        Err(_) => {
                            // Error - niri not running - could show feedback
                        }
                    }
                })
            },
            // Browse button - opens window picker
            {
                let update_and_save = update_and_save.clone();
                Container::new(
                    Stack::horizontal((
                        Label::derived(|| "📋".to_string())
                            .style(|s| s.font_size(FONT_SIZE_SM)),
                        Label::derived(|| "Browse...".to_string())
                            .style(|s| s.font_size(FONT_SIZE_SM)),
                    ))
                    .style(|s| s.gap(SPACING_XS).items_center()),
                )
                .style(move |s| {
                    let t = theme();
                    s.padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS)
                        .border_radius(RADIUS_SM)
                        .color(t.text_muted)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(|s| s.background(t.hover_bg).color(t.text_secondary))
                })
                .on_click_stop(move |_| {
                    // Set the callback so the picker knows how to update this row
                    picker_on_select.set(Some(update_and_save.clone()));
                    // Open the picker
                    picker_state.open();
                })
            },
            Empty::new().style(|s| s.flex_grow(1.0)), // Spacer
        ))
        .style(|s| s.gap(SPACING_SM).margin_left(20.0).margin_top(SPACING_XS)),
    ))
    .style(|s| {
        s.width_full()
            .padding(SPACING_XS)
            .background(BG_SURFACE)
            .border_radius(RADIUS_SM)
    })
}

/// Open behavior selector
fn behavior_selector(
    behavior_idx: RwSignal<usize>,
    rules: RwSignal<Vec<WindowRule>>,
    rule_id: u32,
    save: Rc<dyn Fn()>,
) -> impl IntoView {
    static BEHAVIORS: &[&str] = &["Normal", "Maximized", "Fullscreen", "Floating"];

    Stack::horizontal(
        BEHAVIORS
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let save = save.clone();
                let name = *name;
                let is_selected = move || behavior_idx.get() == idx;

                Container::new(Label::derived(move || name.to_string()).style(move |s| {
                    let base = s
                        .font_size(FONT_SIZE_SM)
                        .padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS);
                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_MUTED)
                    }
                }))
                .style(move |s| {
                    let base = s.border_radius(RADIUS_SM);
                    if is_selected() {
                        base.background(ACCENT.with_alpha(0.15))
                    } else {
                        base.hover(|s| s.background(BG_SURFACE))
                    }
                })
                .on_click_stop(move |_| {
                    behavior_idx.set(idx);
                    rules.update(|r_list| {
                        if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                            r.open_behavior = index_to_open_behavior(idx);
                        }
                    });
                    save();
                })
            })
            .collect::<Vec<_>>(),
    )
    .style(|s| s.gap(SPACING_XS))
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

/// Add new window rule button
fn add_window_rule_button(
    state: AppState,
    rules: RwSignal<Vec<WindowRule>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_add = move || {
        let id = next_id.get();
        next_id.set(id + 1);

        let mut new_rule = WindowRule::default();
        new_rule.id = id;
        new_rule.name = format!("Rule {}", id + 1);

        rules.update(|r_list| {
            r_list.push(new_rule);
        });

        state.update_settings(|s| {
            s.window_rules.rules = rules.get();
            s.window_rules.next_id = next_id.get();
        });
        state.mark_dirty_and_save(SettingsCategory::WindowRules);
    };

    Container::new(
        Stack::horizontal((
            Label::derived(|| "+".to_string()).style(|s| s.font_size(FONT_SIZE_SM).font_bold()),
            Label::derived(|| "Add Rule".to_string()).style(|s| s.font_size(FONT_SIZE_SM)),
        ))
        .style(|s| s.items_center().gap(SPACING_XS)),
    )
    .style(|s| button_primary_style(s).margin_top(SPACING_SM))
    .on_click_stop(move |_| on_add())
}

/// Workspace selector - pick from named workspaces or enter custom
fn workspace_selector(
    state: AppState,
    workspace: RwSignal<String>,
    rules: RwSignal<Vec<WindowRule>>,
    rule_id: u32,
    save: Rc<dyn Fn()>,
) -> impl IntoView {
    // Get named workspaces from state
    let named_workspaces: Vec<String> = state
        .get_settings()
        .workspaces
        .workspaces
        .iter()
        .map(|ws| ws.name.clone())
        .collect();

    let has_named = !named_workspaces.is_empty();
    let current_val = workspace.get();
    let is_custom =
        !current_val.is_empty() && has_named && !named_workspaces.contains(&current_val);

    let show_custom_input = RwSignal::new(is_custom);

    Stack::horizontal((
        Label::derived(|| "Workspace".to_string()).style(|s| {
            s.color(TEXT_SECONDARY)
                .font_size(FONT_SIZE_SM)
                .min_width(80.0)
        }),
        // "None" option
        {
            let save = save.clone();
            let is_selected = move || workspace.get().is_empty();
            Container::new(
                Label::derived(|| "None".to_string()).style(move |s| {
                    let base = s
                        .font_size(FONT_SIZE_SM)
                        .padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS);
                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_MUTED)
                    }
                }),
            )
            .style(move |s| {
                let base = s.border_radius(RADIUS_SM);
                if is_selected() {
                    base.background(ACCENT.with_alpha(0.15))
                } else {
                    base.hover(|s| s.background(BG_SURFACE))
                }
            })
            .on_click_stop(move |_| {
                workspace.set(String::new());
                show_custom_input.set(false);
                rules.update(|r_list| {
                    if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                        r.open_on_workspace = None;
                    }
                });
                save();
            })
        },
        // Named workspace chips - built statically from the list
        Stack::horizontal(
            named_workspaces
                .iter()
                .map(|ws_name| {
                    let save = save.clone();
                    let name = ws_name.clone();
                    let name_display = ws_name.clone();
                    let name_check = ws_name.clone();
                    let name_click = ws_name.clone();

                    Container::new(
                        Label::derived(move || name_display.clone()).style(move |s| {
                            let is_selected = workspace.get() == name_check;
                            let base = s
                                .font_size(FONT_SIZE_SM)
                                .padding_horiz(SPACING_SM)
                                .padding_vert(SPACING_XS);
                            if is_selected {
                                base.color(ACCENT)
                            } else {
                                base.color(TEXT_MUTED)
                            }
                        }),
                    )
                    .style(move |s| {
                        let is_selected = workspace.get() == name;
                        let base = s.border_radius(RADIUS_SM);
                        if is_selected {
                            base.background(ACCENT.with_alpha(0.15))
                        } else {
                            base.hover(|s| s.background(BG_SURFACE))
                        }
                    })
                    .on_click_stop(move |_| {
                        workspace.set(name_click.clone());
                        show_custom_input.set(false);
                        rules.update(|r_list| {
                            if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                                r.open_on_workspace = Some(name_click.clone());
                            }
                        });
                        save();
                    })
                })
                .collect::<Vec<_>>(),
        )
        .style(|s| s.gap(SPACING_XS)),
        // Custom input toggle/field
        {
            let save = save.clone();
            floem::views::dyn_container(
                move || show_custom_input.get(),
                move |show_input| {
                    let save = save.clone();
                    if show_input {
                        // Show text input for custom workspace name
                        text_input(workspace)
                            .placeholder("workspace name")
                            .on_event_stop(EventListener::FocusLost, {
                                let save = save.clone();
                                move |_| {
                                    let val = workspace.get();
                                    rules.update(|r_list| {
                                        if let Some(r) =
                                            r_list.iter_mut().find(|r| r.id == rule_id)
                                        {
                                            r.open_on_workspace = if val.is_empty() {
                                                None
                                            } else {
                                                Some(val)
                                            };
                                        }
                                    });
                                    save();
                                }
                            })
                            .style(|s| {
                                text_input_style(s)
                                    .width(120.0)
                                    .font_family("monospace".to_string())
                            })
                            .into_any()
                    } else {
                        // Show "Custom..." button
                        Container::new(
                            Label::derived(|| "Custom...".to_string()).style(|s| {
                                s.font_size(FONT_SIZE_SM)
                                    .color(TEXT_TERTIARY)
                                    .padding_horiz(SPACING_SM)
                                    .padding_vert(SPACING_XS)
                            }),
                        )
                        .style(|s| {
                            s.border_radius(RADIUS_SM)
                                .hover(|s| s.background(BG_SURFACE).color(TEXT_SECONDARY))
                        })
                        .on_click_stop(move |_| {
                            show_custom_input.set(true);
                        })
                        .into_any()
                    }
                },
            )
        },
    ))
    .style(|s| s.width_full().items_center().gap(SPACING_XS))
}

/// Output selector - pick from configured outputs or enter custom
fn output_selector(
    state: AppState,
    output: RwSignal<String>,
    rules: RwSignal<Vec<WindowRule>>,
    rule_id: u32,
    save: Rc<dyn Fn()>,
) -> impl IntoView {
    // Get configured outputs from state
    let configured_outputs: Vec<String> = state
        .get_settings()
        .outputs
        .outputs
        .iter()
        .map(|o| o.name.clone())
        .collect();

    // Also get outputs from workspaces (in case they reference outputs not in outputs.kdl)
    let workspace_outputs: Vec<String> = state
        .get_settings()
        .workspaces
        .workspaces
        .iter()
        .filter_map(|ws| ws.open_on_output.clone())
        .collect();

    // Combine and deduplicate
    let mut all_outputs: Vec<String> = configured_outputs;
    for wo in workspace_outputs {
        if !all_outputs.contains(&wo) {
            all_outputs.push(wo);
        }
    }

    let has_outputs = !all_outputs.is_empty();
    let current_val = output.get();
    let is_custom = !current_val.is_empty() && has_outputs && !all_outputs.contains(&current_val);

    let show_custom_input = RwSignal::new(is_custom);

    Stack::horizontal((
        Label::derived(|| "Output".to_string()).style(|s| {
            s.color(TEXT_SECONDARY)
                .font_size(FONT_SIZE_SM)
                .min_width(80.0)
        }),
        // "None" option
        {
            let save = save.clone();
            let is_selected = move || output.get().is_empty();
            Container::new(
                Label::derived(|| "None".to_string()).style(move |s| {
                    let base = s
                        .font_size(FONT_SIZE_SM)
                        .padding_horiz(SPACING_SM)
                        .padding_vert(SPACING_XS);
                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_MUTED)
                    }
                }),
            )
            .style(move |s| {
                let base = s.border_radius(RADIUS_SM);
                if is_selected() {
                    base.background(ACCENT.with_alpha(0.15))
                } else {
                    base.hover(|s| s.background(BG_SURFACE))
                }
            })
            .on_click_stop(move |_| {
                output.set(String::new());
                show_custom_input.set(false);
                rules.update(|r_list| {
                    if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                        r.open_on_output = None;
                    }
                });
                save();
            })
        },
        // Output chips - built statically from the list
        Stack::horizontal(
            all_outputs
                .iter()
                .map(|output_name| {
                    let save = save.clone();
                    let name = output_name.clone();
                    let name_display = output_name.clone();
                    let name_check = output_name.clone();
                    let name_click = output_name.clone();

                    Container::new(
                        Label::derived(move || name_display.clone()).style(move |s| {
                            let is_selected = output.get() == name_check;
                            let base = s
                                .font_size(FONT_SIZE_SM)
                                .padding_horiz(SPACING_SM)
                                .padding_vert(SPACING_XS);
                            if is_selected {
                                base.color(ACCENT)
                            } else {
                                base.color(TEXT_MUTED)
                            }
                        }),
                    )
                    .style(move |s| {
                        let is_selected = output.get() == name;
                        let base = s.border_radius(RADIUS_SM);
                        if is_selected {
                            base.background(ACCENT.with_alpha(0.15))
                        } else {
                            base.hover(|s| s.background(BG_SURFACE))
                        }
                    })
                    .on_click_stop(move |_| {
                        output.set(name_click.clone());
                        show_custom_input.set(false);
                        rules.update(|r_list| {
                            if let Some(r) = r_list.iter_mut().find(|r| r.id == rule_id) {
                                r.open_on_output = Some(name_click.clone());
                            }
                        });
                        save();
                    })
                })
                .collect::<Vec<_>>(),
        )
        .style(|s| s.gap(SPACING_XS)),
        // Custom input toggle/field
        {
            let save = save.clone();
            floem::views::dyn_container(
                move || show_custom_input.get(),
                move |show_input| {
                    let save = save.clone();
                    if show_input {
                        // Show text input for custom output name
                        text_input(output)
                            .placeholder("DP-1, HDMI-A-1, etc.")
                            .on_event_stop(EventListener::FocusLost, {
                                let save = save.clone();
                                move |_| {
                                    let val = output.get();
                                    rules.update(|r_list| {
                                        if let Some(r) =
                                            r_list.iter_mut().find(|r| r.id == rule_id)
                                        {
                                            r.open_on_output = if val.is_empty() {
                                                None
                                            } else {
                                                Some(val)
                                            };
                                        }
                                    });
                                    save();
                                }
                            })
                            .style(|s| {
                                text_input_style(s)
                                    .width(140.0)
                                    .font_family("monospace".to_string())
                            })
                            .into_any()
                    } else {
                        // Show "Custom..." button
                        Container::new(
                            Label::derived(|| "Custom...".to_string()).style(|s| {
                                s.font_size(FONT_SIZE_SM)
                                    .color(TEXT_TERTIARY)
                                    .padding_horiz(SPACING_SM)
                                    .padding_vert(SPACING_XS)
                            }),
                        )
                        .style(|s| {
                            s.border_radius(RADIUS_SM)
                                .hover(|s| s.background(BG_SURFACE).color(TEXT_SECONDARY))
                        })
                        .on_click_stop(move |_| {
                            show_custom_input.set(true);
                        })
                        .into_any()
                    }
                },
            )
        },
    ))
    .style(|s| s.width_full().items_center().gap(SPACING_XS))
}

// OpenBehavior helpers
fn open_behavior_to_index(b: OpenBehavior) -> usize {
    match b {
        OpenBehavior::Normal => 0,
        OpenBehavior::Maximized => 1,
        OpenBehavior::Fullscreen => 2,
        OpenBehavior::Floating => 3,
    }
}

fn index_to_open_behavior(idx: usize) -> OpenBehavior {
    match idx {
        0 => OpenBehavior::Normal,
        1 => OpenBehavior::Maximized,
        2 => OpenBehavior::Fullscreen,
        3 => OpenBehavior::Floating,
        _ => OpenBehavior::Normal,
    }
}
