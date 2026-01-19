//! Window rules settings page
//!
//! Two-panel layout with list on left and editor on right.
//! Supports multiple match criteria and collapsible sections.

use freya::prelude::*;

use crate::config::models::{OpenBehavior, WindowRule, WindowRuleMatch};
use crate::config::SettingsCategory;
use crate::ui::app::{ReactiveState, DROPDOWN_RULES_BEHAVIOR};
use crate::ui::components::{
    collapsible_section_minimal, section, select_row_with_state, slider_row, text_row, toggle_row,
};
use crate::ui::theme::*;

/// OpenBehavior options
const OPEN_BEHAVIOR_OPTIONS: &[&str] = &["Normal", "Maximized", "Fullscreen", "Floating"];

/// Convert OpenBehavior to index
fn open_behavior_to_index(b: OpenBehavior) -> usize {
    match b {
        OpenBehavior::Normal => 0,
        OpenBehavior::Maximized => 1,
        OpenBehavior::Fullscreen => 2,
        OpenBehavior::Floating => 3,
    }
}

/// Convert index to OpenBehavior
fn index_to_open_behavior(i: usize) -> OpenBehavior {
    match i {
        0 => OpenBehavior::Normal,
        1 => OpenBehavior::Maximized,
        2 => OpenBehavior::Fullscreen,
        3 => OpenBehavior::Floating,
        _ => OpenBehavior::Normal,
    }
}

/// Create the window rules settings page
pub fn window_rules_page(state: ReactiveState) -> impl IntoElement {
    // Get UI state from ReactiveState (hooks called in app_view)
    let selected_index = state.rules_selected;
    let opening_expanded = state.rules_opening_expanded;
    let visual_expanded = state.rules_visual_expanded;
    let size_expanded = state.rules_size_expanded;

    let settings = state.get_settings();
    let rules = settings.window_rules.rules.clone();
    let sel_idx = *selected_index.read();

    let selected_rule = if sel_idx >= 0 && (sel_idx as usize) < rules.len() {
        Some(rules[sel_idx as usize].clone())
    } else {
        None
    };

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::fill())
        .spacing(SPACING_LG)
        .child(rule_list_panel(state.clone(), rules.clone(), selected_index))
        .child(rule_editor_panel(
            state,
            selected_rule,
            selected_index,
            opening_expanded,
            visual_expanded,
            size_expanded,
        ))
}

/// Left panel with list of rules
fn rule_list_panel(
    state: ReactiveState,
    rules: Vec<WindowRule>,
    selected_index: State<i32>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read();

    rect()
        .content(Content::flex())
        .width(Size::px(280.0))
        .height(Size::fill())
        .spacing(SPACING_MD)
        .child(section(
            "Window Rules",
            rect()
                .width(Size::fill())
                .spacing(SPACING_MD)
                .child(add_rule_button(state.clone(), selected_index.clone()))
                .child(rule_list(rules.clone(), selected_index.clone(), sel_idx))
                .child(remove_rule_button(state, rules, selected_index, sel_idx)),
        ))
}

/// Add rule button
fn add_rule_button(state: ReactiveState, mut selected_index: State<i32>) -> impl IntoElement {
    let mut refresh = state.refresh.clone();

    rect()
        .content(Content::flex())
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .main_align(Alignment::Center)
        .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
        .corner_radius(RADIUS_MD)
        .background(ACCENT_VIVID)
        .on_pointer_down(move |_| {
            state.update_and_save(SettingsCategory::WindowRules, |s| {
                let new_id = s
                    .window_rules
                    .rules
                    .iter()
                    .map(|r| r.id)
                    .max()
                    .unwrap_or(0)
                    + 1;

                let new_rule = WindowRule {
                    id: new_id,
                    name: format!("Rule {}", new_id),
                    matches: vec![WindowRuleMatch::default()],
                    ..Default::default()
                };
                let new_idx = s.window_rules.rules.len();
                s.window_rules.rules.push(new_rule);
                *selected_index.write() = new_idx as i32;
            });
            *refresh.write() += 1;
        })
        .child(
            label()
                .text("+ Add Rule")
                .color(BG_DEEP)
                .font_size(FONT_SIZE_SM)
                .font_weight(FontWeight::MEDIUM),
        )
}

/// List of rules
fn rule_list(
    rules: Vec<WindowRule>,
    selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let mut container = rect()
        .width(Size::fill())
        .spacing(SPACING_XS)
        .max_height(Size::px(400.0));

    for (idx, rule) in rules.iter().enumerate() {
        let is_selected = idx as i32 == sel_idx;
        let name = rule.name.clone();
        let match_desc = if rule.matches.is_empty() {
            "No match criteria".to_string()
        } else {
            let m = &rule.matches[0];
            m.app_id
                .clone()
                .or_else(|| m.title.clone())
                .unwrap_or_else(|| "All windows".to_string())
        };
        let mut selected_index = selected_index.clone();

        let bg_color = if is_selected {
            SELECTED_BG
        } else {
            (0, 0, 0, 0)
        };
        let text_color = if is_selected { ACCENT_VIVID } else { TEXT_BRIGHT };

        container = container.child(
            rect()
                .content(Content::flex())
                .direction(Direction::Vertical)
                .width(Size::fill())
                .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
                .corner_radius(RADIUS_MD)
                .background(bg_color)
                .spacing(SPACING_2XS)
                .on_pointer_down(move |_| {
                    *selected_index.write() = idx as i32;
                })
                .child(
                    label()
                        .text(name)
                        .color(text_color)
                        .font_size(FONT_SIZE_BASE)
                        .font_weight(FontWeight::MEDIUM)
                        .max_lines(1),
                )
                .child(
                    label()
                        .text(match_desc)
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_SM)
                        .max_lines(1),
                ),
        );
    }

    if rules.is_empty() {
        container = container.child(
            label()
                .text("No window rules configured")
                .color(TEXT_DIM)
                .font_size(FONT_SIZE_SM),
        );
    }

    container
}

/// Remove rule button
fn remove_rule_button(
    state: ReactiveState,
    rules: Vec<WindowRule>,
    mut selected_index: State<i32>,
    sel_idx: i32,
) -> impl IntoElement {
    let can_remove = sel_idx >= 0 && (sel_idx as usize) < rules.len();
    let mut refresh = state.refresh.clone();

    if can_remove {
        rect()
            .content(Content::flex())
            .width(Size::fill())
            .cross_align(Alignment::Center)
            .main_align(Alignment::Center)
            .padding((SPACING_SM, SPACING_MD, SPACING_SM, SPACING_MD))
            .corner_radius(RADIUS_MD)
            .background(ERROR)
            .on_pointer_down(move |_| {
                state.update_and_save(SettingsCategory::WindowRules, |s| {
                    if sel_idx >= 0 && (sel_idx as usize) < s.window_rules.rules.len() {
                        s.window_rules.rules.remove(sel_idx as usize);
                        let new_len = s.window_rules.rules.len() as i32;
                        if new_len == 0 {
                            *selected_index.write() = -1;
                        } else if sel_idx >= new_len {
                            *selected_index.write() = new_len - 1;
                        }
                    }
                });
                *refresh.write() += 1;
            })
            .child(
                label()
                    .text("Remove Rule")
                    .color(TEXT_BRIGHT)
                    .font_size(FONT_SIZE_SM)
                    .font_weight(FontWeight::MEDIUM),
            )
            .into_element()
    } else {
        rect().into_element()
    }
}

/// Right panel with rule editor
fn rule_editor_panel(
    state: ReactiveState,
    selected_rule: Option<WindowRule>,
    selected_index: State<i32>,
    opening_expanded: State<bool>,
    visual_expanded: State<bool>,
    size_expanded: State<bool>,
) -> impl IntoElement {
    match selected_rule {
        Some(rule) => rule_editor(
            state,
            rule,
            selected_index,
            opening_expanded,
            visual_expanded,
            size_expanded,
        )
        .into_element(),
        None => no_selection_panel().into_element(),
    }
}

/// Panel shown when no rule is selected
fn no_selection_panel() -> impl IntoElement {
    rect()
        .content(Content::flex())
        .width(Size::flex(1.0))
        .height(Size::fill())
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .spacing(SPACING_MD)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text("No Rule Selected")
                        .color(TEXT_DIM)
                        .font_size(FONT_SIZE_LG)
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    label()
                        .text("Select a rule from the list or add a new one")
                        .color(TEXT_GHOST)
                        .font_size(FONT_SIZE_SM),
                ),
        )
}

/// Rule editor with all fields organized in sections
fn rule_editor(
    state: ReactiveState,
    rule: WindowRule,
    selected_index: State<i32>,
    opening_expanded: State<bool>,
    visual_expanded: State<bool>,
    size_expanded: State<bool>,
) -> impl IntoElement {
    let sel_idx = *selected_index.read() as usize;
    let mut refresh = state.refresh.clone();

    // Get first match for editing (simplified - only edit first match)
    let first_match = rule.matches.first().cloned().unwrap_or_default();

    rect()
        .content(Content::flex())
        .width(Size::flex(1.0))
        .height(Size::fill())
        .spacing(SPACING_LG)
        // Basic info section
        .child(section(
            &format!("Rule: {}", rule.name),
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                .child({
                    let state = state.clone();
                    let name = rule.name.clone();
                    let mut refresh = refresh.clone();
                    text_row("Name", "Display name for this rule", &name, "My Rule", move |v| {
                        state.update_and_save(SettingsCategory::WindowRules, |s| {
                            if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                r.name = v;
                            }
                        });
                        *refresh.write() += 1;
                    })
                }),
        ))
        // Match criteria section
        .child(section(
            "Match Criteria",
            rect()
                .width(Size::fill())
                .spacing(SPACING_SM)
                // App ID
                .child({
                    let state = state.clone();
                    let app_id = first_match.app_id.clone().unwrap_or_default();
                    let mut refresh = refresh.clone();
                    text_row(
                        "App ID",
                        "Match by application ID (regex)",
                        &app_id,
                        "firefox.*",
                        move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    if let Some(m) = r.matches.first_mut() {
                                        m.app_id = if v.is_empty() { None } else { Some(v) };
                                    }
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Title
                .child({
                    let state = state.clone();
                    let title = first_match.title.clone().unwrap_or_default();
                    let mut refresh = refresh.clone();
                    text_row(
                        "Title",
                        "Match by window title (regex)",
                        &title,
                        ".*YouTube.*",
                        move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    if let Some(m) = r.matches.first_mut() {
                                        m.title = if v.is_empty() { None } else { Some(v) };
                                    }
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                // Is floating
                .child({
                    let state = state.clone();
                    let is_floating = first_match.is_floating.unwrap_or(false);
                    let mut refresh = refresh.clone();
                    toggle_row(
                        "Match Floating",
                        "Only match floating windows",
                        is_floating,
                        move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    if let Some(m) = r.matches.first_mut() {
                                        m.is_floating = if v { Some(true) } else { None };
                                    }
                                }
                            });
                            *refresh.write() += 1;
                        },
                    )
                }),
        ))
        // Opening behavior section (collapsible)
        .child({
            let opening_exp = *opening_expanded.read();
            let mut opening_expanded = opening_expanded.clone();

            collapsible_section_minimal(
                "Opening Behavior",
                opening_exp,
                move || {
                    *opening_expanded.write() = !opening_exp;
                },
                rect()
                    .width(Size::fill())
                    .spacing(SPACING_SM)
                    // Open behavior select
                    .child({
                        let state_clone = state.clone();
                        let behavior = rule.open_behavior;
                        let mut refresh = refresh.clone();
                        select_row_with_state(
                            "Behavior",
                            "How the window opens",
                            OPEN_BEHAVIOR_OPTIONS,
                            open_behavior_to_index(behavior),
                            DROPDOWN_RULES_BEHAVIOR,
                            &state,
                            move |i| {
                                state_clone.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.open_behavior = index_to_open_behavior(i);
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Open focused
                    .child({
                        let state = state.clone();
                        let focused = rule.open_focused.unwrap_or(true);
                        let mut refresh = refresh.clone();
                        toggle_row(
                            "Open Focused",
                            "Focus window when it opens",
                            focused,
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.open_focused = Some(v);
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Open on output
                    .child({
                        let state = state.clone();
                        let output = rule.open_on_output.clone().unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row(
                            "Open on Output",
                            "Specific output to open on",
                            &output,
                            "eDP-1",
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.open_on_output = if v.is_empty() { None } else { Some(v) };
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Open on workspace
                    .child({
                        let state = state.clone();
                        let workspace = rule.open_on_workspace.clone().unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row(
                            "Open on Workspace",
                            "Specific workspace to open on",
                            &workspace,
                            "browser",
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.open_on_workspace = if v.is_empty() { None } else { Some(v) };
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    }),
            )
        })
        // Visual section (collapsible)
        .child({
            let visual_exp = *visual_expanded.read();
            let mut visual_expanded = visual_expanded.clone();

            collapsible_section_minimal(
                "Visual Settings",
                visual_exp,
                move || {
                    *visual_expanded.write() = !visual_exp;
                },
                rect()
                    .width(Size::fill())
                    .spacing(SPACING_SM)
                    // Opacity
                    .child({
                        let state = state.clone();
                        let opacity = rule.opacity.unwrap_or(1.0) as f64;
                        let mut refresh = refresh.clone();
                        slider_row(
                            "Opacity",
                            "Window transparency (0-1)",
                            opacity,
                            0.0,
                            1.0,
                            "",
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.opacity = if (v - 1.0).abs() < 0.01 { None } else { Some(v as f32) };
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Corner radius
                    .child({
                        let state = state.clone();
                        let radius = rule.corner_radius.map(|r| r.to_string()).unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row(
                            "Corner Radius",
                            "Custom corner radius (px)",
                            &radius,
                            "12",
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.corner_radius = v.parse::<i32>().ok();
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Block from screencast
                    .child({
                        let state = state.clone();
                        let block = rule.block_out_from_screencast;
                        let mut refresh = refresh.clone();
                        toggle_row(
                            "Block from Screencast",
                            "Hide window in screen recordings",
                            block,
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.block_out_from_screencast = v;
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    })
                    // Clip to geometry
                    .child({
                        let state = state.clone();
                        let clip = rule.clip_to_geometry.unwrap_or(false);
                        let mut refresh = refresh.clone();
                        toggle_row(
                            "Clip to Geometry",
                            "Clip window to visual geometry",
                            clip,
                            move |v| {
                                state.update_and_save(SettingsCategory::WindowRules, |s| {
                                    if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                        r.clip_to_geometry = if v { Some(true) } else { None };
                                    }
                                });
                                *refresh.write() += 1;
                            },
                        )
                    }),
            )
        })
        // Size constraints section (collapsible)
        .child({
            let size_exp = *size_expanded.read();
            let mut size_expanded = size_expanded.clone();

            collapsible_section_minimal(
                "Size Constraints",
                size_exp,
                move || {
                    *size_expanded.write() = !size_exp;
                },
                rect()
                    .width(Size::fill())
                    .spacing(SPACING_SM)
                    // Min width
                    .child({
                        let state = state.clone();
                        let min_w = rule.min_width.map(|w| w.to_string()).unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row("Min Width", "Minimum width (px)", &min_w, "400", move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    r.min_width = v.parse::<i32>().ok();
                                }
                            });
                            *refresh.write() += 1;
                        })
                    })
                    // Max width
                    .child({
                        let state = state.clone();
                        let max_w = rule.max_width.map(|w| w.to_string()).unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row("Max Width", "Maximum width (px)", &max_w, "1920", move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    r.max_width = v.parse::<i32>().ok();
                                }
                            });
                            *refresh.write() += 1;
                        })
                    })
                    // Min height
                    .child({
                        let state = state.clone();
                        let min_h = rule.min_height.map(|h| h.to_string()).unwrap_or_default();
                        let mut refresh = refresh.clone();
                        text_row("Min Height", "Minimum height (px)", &min_h, "300", move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    r.min_height = v.parse::<i32>().ok();
                                }
                            });
                            *refresh.write() += 1;
                        })
                    })
                    // Max height
                    .child({
                        let state = state.clone();
                        let max_h = rule.max_height.map(|h| h.to_string()).unwrap_or_default();
                        text_row("Max Height", "Maximum height (px)", &max_h, "1080", move |v| {
                            state.update_and_save(SettingsCategory::WindowRules, |s| {
                                if let Some(r) = s.window_rules.rules.get_mut(sel_idx) {
                                    r.max_height = v.parse::<i32>().ok();
                                }
                            });
                            *refresh.write() += 1;
                        })
                    }),
            )
        })
}
