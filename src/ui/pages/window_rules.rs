//! Window rules settings page

use floem::event::EventListener;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Container, Label, Stack};
use std::rc::Rc;

use crate::config::models::{OpenBehavior, WindowRule, WindowRuleMatch};
use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::{
    button_primary_style, icon_button_style, text_input_style, ACCENT, BG_ELEVATED, BG_SURFACE,
    BORDER_SUBTLE, ERROR, FONT_SIZE_BASE, FONT_SIZE_SM, RADIUS_MD, RADIUS_SM, SPACING_LG,
    SPACING_MD, SPACING_SM, SPACING_XS, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
};

/// Create the window rules settings page
pub fn window_rules_page(state: AppState) -> impl IntoView {
    // Create signals for rules list
    let rules = RwSignal::new(state.get_settings().window_rules.rules.clone());
    let next_id = RwSignal::new(state.get_settings().window_rules.next_id);

    Stack::vertical((
        section(
            "Window Rules",
            Stack::vertical((
                // List of existing rules
                window_rule_list(state.clone(), rules, next_id),
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
    .style(|s| s.width_full().gap(SPACING_LG))
}

/// List of window rule cards
fn window_rule_list(
    state: AppState,
    rules: RwSignal<Vec<WindowRule>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
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
                            window_rule_card(state.clone(), idx, rule, rules, next_id)
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

    let expanded = RwSignal::new(false);

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
                    let save = save.clone();

                    if is_expanded {
                        let save_behavior = save.clone();
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
                            match_criteria_row(rule_id, idx, m, matches_signal, rules, save.clone())
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
) -> impl IntoView {
    let app_id = RwSignal::new(m.app_id.clone().unwrap_or_default());
    let title = RwSignal::new(m.title.clone().unwrap_or_default());

    let save_app = save.clone();
    let save_title = save.clone();
    let save_delete = save.clone();

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
    .style(|s| {
        s.width_full()
            .items_center()
            .gap(SPACING_SM)
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
