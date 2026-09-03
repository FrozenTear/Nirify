//! Shared `LayoutOverride` editor
//!
//! One widget used by the Displays output modal and Named Workspaces so both
//! surfaces expose the same fields that storage already round-trips.
//!
//! Global Layout extras / Appearance sections bind `LayoutExtrasMessage` /
//! `AppearanceMessage` and cannot be remounted here. This editor reuses the
//! same building blocks (`optional_*` rows, `gradient_picker`, `preset_entry_row`,
//! `TabIndicatorSettings`, `InsertHintSettings`) instead.
//!
//! # Field coverage vs `LayoutOverride`
//!
//! | Field | Widget |
//! |---|---|
//! | `gaps`, `strut_*` | optional sliders |
//! | `center_focused_column` | optional picker |
//! | `always_center_single_column` | optional bool |
//! | `empty_workspace_above_first` | optional bool |
//! | `default_column_display` | optional picker |
//! | `background_color` | optional color |
//! | `default_column_width_proportion` / `_fixed` / `_auto` | optional sliders + bool (mutually exclusive) |
//! | `preset_column_widths` / `preset_window_heights` | enable + `preset_entry_row` list |
//! | `focus_ring_*` including urgent | optional bool / slider / `optional_gradient_picker` |
//! | `border_*` including urgent | optional bool / slider / `optional_gradient_picker` |
//! | `shadow_*` including inactive + draw-behind | optional bool / slider / color |
//! | `tab_indicator` | enable block + Layout-extras-equivalent fields |
//! | `insert_hint` | enable block + Layout-extras-equivalent fields |
//!
//! Remaining (intentionally none): every model field has a control. Adding a
//! field to `LayoutOverride` must update [`layout_override_field_coverage`].

use iced::widget::{button, column, text};
use iced::Element;

use super::{
    apply_gradient_message, card, delete_button_style, gradient_picker, info_text,
    optional_bool_picker, optional_color_row, optional_gradient_picker, optional_picker_row,
    optional_slider_row, picker_row, preset_entry_row, slider_row_int, subsection_header,
    toggle_row,
};
use crate::config::models::{
    DefaultColumnDisplay, InsertHintSettings, LayoutExtrasSettings, LayoutOverride, PresetHeight,
    PresetWidth, TabIndicatorPosition, TabIndicatorSettings,
};
use crate::types::CenterFocusedColumn;

const COLUMN_DISPLAY: [DefaultColumnDisplay; 2] =
    [DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed];

const TAB_POSITIONS: [TabIndicatorPosition; 4] = [
    TabIndicatorPosition::Left,
    TabIndicatorPosition::Right,
    TabIndicatorPosition::Top,
    TabIndicatorPosition::Bottom,
];

/// Enable / remove chrome plus the full override editor.
///
/// `enable_hint` is shown when no override is present (output vs workspace copy).
pub fn layout_override_content<'a, Message: Clone + 'a>(
    lo: Option<&'a LayoutOverride>,
    on_set: impl Fn(Option<LayoutOverride>) -> Message + Clone + 'a,
    enable_hint: &'a str,
) -> Element<'a, Message> {
    if let Some(lo) = lo {
        let on_remove = on_set.clone();
        let on_edit = move |next| on_set(Some(next));
        column![
            info_text(
                "Override global layout settings. Fields set to Use Global inherit from the global layout. Remove All Overrides discards this block."
            ),
            button(text("Remove All Overrides").size(14))
                .on_press(on_remove(None))
                .padding([8, 16])
                .style(delete_button_style),
            layout_override_editor(lo, on_edit),
        ]
        .spacing(8)
        .into()
    } else {
        column![
            info_text(enable_hint),
            button(text("Enable Layout Override").size(14))
                .on_press(on_set(Some(LayoutOverride::default())))
                .padding([8, 16]),
        ]
        .spacing(8)
        .into()
    }
}

/// Full field editor for an existing `LayoutOverride`.
pub fn layout_override_editor<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    column![
        gaps_struts_card(lo, on_change.clone()),
        column_behavior_card(lo, on_change.clone()),
        sizing_card(lo, on_change.clone()),
        focus_ring_card(lo, on_change.clone()),
        border_card(lo, on_change.clone()),
        shadow_card(lo, on_change.clone()),
        tab_indicator_card(lo, on_change.clone()),
        insert_hint_card(lo, on_change),
    ]
    .spacing(8)
    .into()
}

fn edit<Message>(
    lo: &LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message,
    mutate: impl FnOnce(&mut LayoutOverride),
) -> Message {
    let mut next = lo.clone();
    mutate(&mut next);
    on_change(next)
}

fn gaps_struts_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    card(
        column![
            subsection_header("Gaps & Struts"),
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Gaps",
                    "Space between windows (px)",
                    lo.gaps,
                    0.0,
                    64.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.gaps = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Strut Left",
                    "Reserved space on the left edge (px)",
                    lo.strut_left,
                    0.0,
                    500.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.strut_left = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Strut Right",
                    "Reserved space on the right edge (px)",
                    lo.strut_right,
                    0.0,
                    500.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.strut_right = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Strut Top",
                    "Reserved space on the top edge (px)",
                    lo.strut_top,
                    0.0,
                    500.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.strut_top = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change;
                optional_slider_row(
                    "Strut Bottom",
                    "Reserved space on the bottom edge (px)",
                    lo.strut_bottom,
                    0.0,
                    500.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.strut_bottom = v),
                )
            },
        ]
        .spacing(4),
    )
}

fn column_behavior_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    card(
        column![
            subsection_header("Column Behavior"),
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_picker_row(
                    "Center Focused Column",
                    "When to auto-center the focused column",
                    CenterFocusedColumn::all(),
                    lo.center_focused_column,
                    move |v| edit(&lo_c, on.clone(), |o| o.center_focused_column = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Always Center Single Column",
                    "Center a single column even when it fits",
                    lo.always_center_single_column,
                    move |v| edit(&lo_c, on.clone(), |o| o.always_center_single_column = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Empty Workspace Above First",
                    "Keep an empty workspace above the first one",
                    lo.empty_workspace_above_first,
                    move |v| edit(&lo_c, on.clone(), |o| o.empty_workspace_above_first = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_picker_row(
                    "Default Column Display",
                    "How new columns are displayed",
                    &COLUMN_DISPLAY,
                    lo.default_column_display,
                    move |v| edit(&lo_c, on.clone(), |o| o.default_column_display = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change;
                optional_color_row(
                    "Background Color",
                    "Workspace background color for this override",
                    lo.background_color.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.background_color = v),
                )
            },
        ]
        .spacing(4),
    )
}

fn sizing_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut content = column![
        subsection_header("Default Sizing"),
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            optional_bool_picker(
                "Column Width Auto",
                "Empty default-column-width { }: windows pick their own width",
                lo.default_column_width_auto,
                move |v| edit(&lo_c, on.clone(), |o| set_default_column_width_auto(o, v)),
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            optional_slider_row(
                "Column Width (Proportion)",
                "Default column width as a fraction of screen width",
                lo.default_column_width_proportion,
                0.1,
                1.0,
                "",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        set_default_column_width_proportion(o, v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            optional_slider_row(
                "Column Width (Fixed)",
                "Default column width in pixels",
                lo.default_column_width_fixed.map(|v| v as f32),
                200.0,
                4000.0,
                "px",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        set_default_column_width_fixed(o, v.map(|f| f as i32))
                    })
                },
            )
        },
        subsection_header("Preset Sizes"),
        info_text(
            "Widths and heights cycled by switch-preset-column-width / switch-preset-window-height. Off inherits the global lists."
        ),
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            let enabled = lo.preset_column_widths.is_some();
            toggle_row(
                "Override preset column widths",
                "Replace the global switch-preset-column-width list",
                enabled,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        o.preset_column_widths = if v {
                            Some(LayoutExtrasSettings::default().preset_column_widths)
                        } else {
                            None
                        };
                    })
                },
            )
        },
    ]
    .spacing(4);

    if let Some(presets) = lo.preset_column_widths.as_ref() {
        content = content.push(preset_widths_editor(lo, presets, on_change.clone()));
    }

    content = content.push({
        let lo_c = lo.clone();
        let on = on_change.clone();
        let enabled = lo.preset_window_heights.is_some();
        toggle_row(
            "Override preset window heights",
            "Replace the global switch-preset-window-height list",
            enabled,
            move |v| {
                edit(&lo_c, on.clone(), |o| {
                    o.preset_window_heights = if v {
                        Some(LayoutExtrasSettings::default().preset_window_heights)
                    } else {
                        None
                    };
                })
            },
        )
    });

    if let Some(presets) = lo.preset_window_heights.as_ref() {
        content = content.push(preset_heights_editor(lo, presets, on_change));
    }

    card(content)
}

fn preset_widths_editor<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    presets: &'a [PresetWidth],
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut col = column![].spacing(6);

    for (idx, preset) in presets.iter().enumerate() {
        let (kind, value_str) = match preset {
            PresetWidth::Proportion(p) => ("Proportion", format!("{}", p)),
            PresetWidth::Fixed(f) => ("Fixed", format!("{}", f)),
        };
        let is_prop = matches!(preset, PresetWidth::Proportion(_));
        let make = move |kind: &str, raw: &str| -> PresetWidth {
            if kind == "Fixed" {
                PresetWidth::Fixed(raw.trim().parse::<i32>().unwrap_or(0))
            } else {
                PresetWidth::Proportion(raw.trim().parse::<f32>().unwrap_or(0.0))
            }
        };
        let value_for_kind = value_str.clone();
        let lo_kind = lo.clone();
        let on_kind = on_change.clone();
        let lo_val = lo.clone();
        let on_val = on_change.clone();
        let lo_rm = lo.clone();
        let on_rm = on_change.clone();
        col = col.push(preset_entry_row(
            kind,
            &value_str,
            is_prop,
            move |new_kind: &str| {
                let made = make(new_kind, &value_for_kind);
                edit(&lo_kind, on_kind.clone(), |o| {
                    if let Some(list) = o.preset_column_widths.as_mut() {
                        if let Some(slot) = list.get_mut(idx) {
                            *slot = made;
                        }
                    }
                })
            },
            move |kind: String, raw: String| {
                let made = make(&kind, &raw);
                edit(&lo_val, on_val.clone(), |o| {
                    if let Some(list) = o.preset_column_widths.as_mut() {
                        if let Some(slot) = list.get_mut(idx) {
                            *slot = made;
                        }
                    }
                })
            },
            edit(&lo_rm, on_rm, |o| {
                if let Some(list) = o.preset_column_widths.as_mut() {
                    if idx < list.len() {
                        list.remove(idx);
                    }
                }
            }),
        ));
    }

    let lo_add = lo.clone();
    let on_add = on_change;
    col = col.push(
        button(text("+ Add width").size(12))
            .on_press(edit(&lo_add, on_add, |o| {
                o.preset_column_widths
                    .get_or_insert_with(Vec::new)
                    .push(PresetWidth::Proportion(0.5));
            }))
            .padding([6, 12]),
    );
    col.into()
}

fn preset_heights_editor<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    presets: &'a [PresetHeight],
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut col = column![].spacing(6);

    for (idx, preset) in presets.iter().enumerate() {
        let (kind, value_str) = match preset {
            PresetHeight::Proportion(p) => ("Proportion", format!("{}", p)),
            PresetHeight::Fixed(f) => ("Fixed", format!("{}", f)),
        };
        let is_prop = matches!(preset, PresetHeight::Proportion(_));
        let make = move |kind: &str, raw: &str| -> PresetHeight {
            if kind == "Fixed" {
                PresetHeight::Fixed(raw.trim().parse::<i32>().unwrap_or(0))
            } else {
                PresetHeight::Proportion(raw.trim().parse::<f32>().unwrap_or(0.0))
            }
        };
        let value_for_kind = value_str.clone();
        let lo_kind = lo.clone();
        let on_kind = on_change.clone();
        let lo_val = lo.clone();
        let on_val = on_change.clone();
        let lo_rm = lo.clone();
        let on_rm = on_change.clone();
        col = col.push(preset_entry_row(
            kind,
            &value_str,
            is_prop,
            move |new_kind: &str| {
                let made = make(new_kind, &value_for_kind);
                edit(&lo_kind, on_kind.clone(), |o| {
                    if let Some(list) = o.preset_window_heights.as_mut() {
                        if let Some(slot) = list.get_mut(idx) {
                            *slot = made;
                        }
                    }
                })
            },
            move |kind: String, raw: String| {
                let made = make(&kind, &raw);
                edit(&lo_val, on_val.clone(), |o| {
                    if let Some(list) = o.preset_window_heights.as_mut() {
                        if let Some(slot) = list.get_mut(idx) {
                            *slot = made;
                        }
                    }
                })
            },
            edit(&lo_rm, on_rm, |o| {
                if let Some(list) = o.preset_window_heights.as_mut() {
                    if idx < list.len() {
                        list.remove(idx);
                    }
                }
            }),
        ));
    }

    let lo_add = lo.clone();
    let on_add = on_change;
    col = col.push(
        button(text("+ Add height").size(12))
            .on_press(edit(&lo_add, on_add, |o| {
                o.preset_window_heights
                    .get_or_insert_with(Vec::new)
                    .push(PresetHeight::Proportion(0.5));
            }))
            .padding([6, 12]),
    );
    col.into()
}

fn focus_ring_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    card(
        column![
            subsection_header("Focus Ring"),
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Enabled",
                    "Show focus ring around focused window",
                    lo.focus_ring_enabled,
                    move |v| edit(&lo_c, on.clone(), |o| o.focus_ring_enabled = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Width",
                    "Focus ring thickness (px)",
                    lo.focus_ring_width.map(|v| v as f32),
                    1.0,
                    16.0,
                    "px",
                    move |v| {
                        edit(&lo_c, on.clone(), |o| {
                            o.focus_ring_width = v.map(|f| f as i32)
                        })
                    },
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_gradient_picker(
                    "Active Color",
                    "Color or gradient of the focus ring on the focused window",
                    lo.focus_ring_active.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.focus_ring_active = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_gradient_picker(
                    "Inactive Color",
                    "Color or gradient of the focus ring on unfocused windows",
                    lo.focus_ring_inactive.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.focus_ring_inactive = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change;
                optional_gradient_picker(
                    "Urgent Color",
                    "Color or gradient of the focus ring on urgent windows",
                    lo.focus_ring_urgent.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.focus_ring_urgent = v),
                )
            },
        ]
        .spacing(4),
    )
}

fn border_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    card(
        column![
            subsection_header("Border"),
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Enabled",
                    "Show border around windows",
                    lo.border_enabled,
                    move |v| edit(&lo_c, on.clone(), |o| o.border_enabled = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Width",
                    "Border thickness (px)",
                    lo.border_width.map(|v| v as f32),
                    1.0,
                    8.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.border_width = v.map(|f| f as i32)),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_gradient_picker(
                    "Active Color",
                    "Color or gradient of the border on the focused window",
                    lo.border_active.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.border_active = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_gradient_picker(
                    "Inactive Color",
                    "Color or gradient of the border on unfocused windows",
                    lo.border_inactive.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.border_inactive = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change;
                optional_gradient_picker(
                    "Urgent Color",
                    "Color or gradient of the border on urgent windows",
                    lo.border_urgent.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.border_urgent = v),
                )
            },
        ]
        .spacing(4),
    )
}

fn shadow_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    card(
        column![
            subsection_header("Shadow"),
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Enabled",
                    "Show shadow behind windows",
                    lo.shadow_enabled,
                    move |v| edit(&lo_c, on.clone(), |o| o.shadow_enabled = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_bool_picker(
                    "Draw Behind Window",
                    "Draw shadow underneath (for transparency)",
                    lo.shadow_draw_behind_window,
                    move |v| edit(&lo_c, on.clone(), |o| o.shadow_draw_behind_window = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Softness",
                    "Shadow blur radius (px)",
                    lo.shadow_softness.map(|v| v as f32),
                    0.0,
                    100.0,
                    "px",
                    move |v| {
                        edit(&lo_c, on.clone(), |o| {
                            o.shadow_softness = v.map(|f| f as i32)
                        })
                    },
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Spread",
                    "Shadow expansion (px)",
                    lo.shadow_spread.map(|v| v as f32),
                    0.0,
                    100.0,
                    "px",
                    move |v| edit(&lo_c, on.clone(), |o| o.shadow_spread = v.map(|f| f as i32)),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Offset X",
                    "Horizontal shadow offset (px)",
                    lo.shadow_offset_x.map(|v| v as f32),
                    -100.0,
                    100.0,
                    "px",
                    move |v| {
                        edit(&lo_c, on.clone(), |o| {
                            o.shadow_offset_x = v.map(|f| f as i32)
                        })
                    },
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_slider_row(
                    "Offset Y",
                    "Vertical shadow offset (px)",
                    lo.shadow_offset_y.map(|v| v as f32),
                    -100.0,
                    100.0,
                    "px",
                    move |v| {
                        edit(&lo_c, on.clone(), |o| {
                            o.shadow_offset_y = v.map(|f| f as i32)
                        })
                    },
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change.clone();
                optional_color_row(
                    "Color",
                    "Shadow color",
                    lo.shadow_color.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.shadow_color = v),
                )
            },
            {
                let lo_c = lo.clone();
                let on = on_change;
                optional_color_row(
                    "Inactive Color",
                    "Shadow color on unfocused windows",
                    lo.shadow_inactive_color.as_ref(),
                    move |v| edit(&lo_c, on.clone(), |o| o.shadow_inactive_color = v),
                )
            },
        ]
        .spacing(4),
    )
}

fn tab_indicator_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let enabled = lo.tab_indicator.is_some();
    let lo_toggle = lo.clone();
    let on_toggle = on_change.clone();
    let mut content = column![
        subsection_header("Tab Indicator"),
        info_text(
            "Some writes a tab-indicator { } block that must round-trip. Off inherits the global indicator."
        ),
        toggle_row(
            "Override tab indicator",
            "Replace the global tab-indicator block for this output or workspace",
            enabled,
            move |v| {
                edit(&lo_toggle, on_toggle.clone(), |o| {
                    o.tab_indicator = if v {
                        Some(TabIndicatorSettings::default())
                    } else {
                        None
                    };
                })
            },
        ),
    ]
    .spacing(4);

    if let Some(tab) = lo.tab_indicator.as_ref() {
        content = content.push(tab_indicator_fields(lo, tab, on_change));
    }

    card(content)
}

/// Field editors for an enabled tab-indicator override (mirrors Layout extras).
fn tab_indicator_fields<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    tab: &'a TabIndicatorSettings,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let tab_length = (tab.length_proportion * 100.0) as i32;

    column![
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Enable tab indicator",
                "Show indicator for tabbed windows",
                tab.enabled,
                move |v| edit(&lo_c, on.clone(), |o| map_tab(o, |t| t.enabled = v)),
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Hide when single tab",
                "Don't show when only one tab",
                tab.hide_when_single_tab,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.hide_when_single_tab = v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Place within column",
                "Position inside the column",
                tab.place_within_column,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.place_within_column = v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Custom active color",
                "Override the focus-ring active color",
                tab.use_active_color,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.use_active_color = v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Custom inactive color",
                "Override the focus-ring inactive color",
                tab.use_inactive_color,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.use_inactive_color = v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            toggle_row(
                "Custom urgent color",
                "Override the urgent color",
                tab.use_urgent_color,
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.use_urgent_color = v)
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            slider_row_int(
                "Gap",
                "Space between the indicator and the window",
                tab.gap,
                0,
                50,
                "px",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.gap = v.clamp(0, 50))
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            slider_row_int(
                "Width",
                "Indicator thickness",
                tab.width,
                1,
                50,
                "px",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.width = v.clamp(1, 50))
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            slider_row_int(
                "Length",
                "Indicator length as a percent of the column",
                tab_length,
                10,
                200,
                "%",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| {
                            t.length_proportion = (v as f32 / 100.0).clamp(0.1, 2.0)
                        })
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            slider_row_int(
                "Corner Radius",
                "Indicator corner rounding",
                tab.corner_radius,
                0,
                50,
                "px",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.corner_radius = v.clamp(0, 50))
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            slider_row_int(
                "Gaps Between Tabs",
                "Space between tab indicators",
                tab.gaps_between_tabs,
                0,
                50,
                "px",
                move |v| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| t.gaps_between_tabs = v.clamp(0, 50))
                    })
                },
            )
        },
        {
            let lo_c = lo.clone();
            let on = on_change.clone();
            picker_row(
                "Position",
                "Which edge of the column shows the indicator",
                &TAB_POSITIONS,
                Some(tab.position),
                move |v| edit(&lo_c, on.clone(), |o| map_tab(o, |t| t.position = v)),
            )
        },
        if tab.use_active_color {
            let lo_c = lo.clone();
            let on = on_change.clone();
            gradient_picker(
                "Active color",
                "Color or gradient for the active tab indicator",
                &tab.active,
                move |msg| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| {
                            apply_gradient_message(&mut t.active, msg);
                            t.use_active_color = true;
                        })
                    })
                },
            )
        } else {
            info_text("Active follows focus ring colors when off.")
        },
        if tab.use_inactive_color {
            let lo_c = lo.clone();
            let on = on_change.clone();
            gradient_picker(
                "Inactive color",
                "Color or gradient for the inactive tab indicator",
                &tab.inactive,
                move |msg| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| {
                            apply_gradient_message(&mut t.inactive, msg);
                            t.use_inactive_color = true;
                        })
                    })
                },
            )
        } else {
            info_text("Inactive follows focus ring colors when off.")
        },
        if tab.use_urgent_color {
            let lo_c = lo.clone();
            let on = on_change;
            gradient_picker(
                "Urgent color",
                "Color or gradient for urgent tab indicators",
                &tab.urgent,
                move |msg| {
                    edit(&lo_c, on.clone(), |o| {
                        map_tab(o, |t| {
                            apply_gradient_message(&mut t.urgent, msg);
                            t.use_urgent_color = true;
                        })
                    })
                },
            )
        } else {
            info_text("niri default #9b0000 when off.")
        },
    ]
    .spacing(4)
    .into()
}

fn insert_hint_card<'a, Message: Clone + 'a>(
    lo: &'a LayoutOverride,
    on_change: impl Fn(LayoutOverride) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let enabled = lo.insert_hint.is_some();
    let lo_toggle = lo.clone();
    let on_toggle = on_change.clone();
    let mut content = column![
        subsection_header("Insert Hint"),
        info_text(
            "Some writes an insert-hint { } block that must round-trip. Off inherits the global hint."
        ),
        toggle_row(
            "Override insert hint",
            "Replace the global insert-hint block for this output or workspace",
            enabled,
            move |v| {
                edit(&lo_toggle, on_toggle.clone(), |o| {
                    o.insert_hint = if v {
                        Some(InsertHintSettings::default())
                    } else {
                        None
                    };
                })
            },
        ),
    ]
    .spacing(4);

    if let Some(hint) = lo.insert_hint.as_ref() {
        let lo_en = lo.clone();
        let on_en = on_change.clone();
        let lo_color = lo.clone();
        let on_color = on_change;
        content = content.push(toggle_row(
            "Enable insert hint",
            "Show visual hint when inserting windows",
            hint.enabled,
            move |v| edit(&lo_en, on_en.clone(), |o| map_hint(o, |h| h.enabled = v)),
        ));
        content = content.push(gradient_picker(
            "Hint color",
            "Color or gradient shown when inserting a window",
            &hint.color,
            move |msg| {
                edit(&lo_color, on_color.clone(), |o| {
                    map_hint(o, |h| apply_gradient_message(&mut h.color, msg))
                })
            },
        ));
    }

    card(content)
}

fn map_tab(lo: &mut LayoutOverride, f: impl FnOnce(&mut TabIndicatorSettings)) {
    if let Some(tab) = lo.tab_indicator.as_mut() {
        f(tab);
    }
}

fn map_hint(lo: &mut LayoutOverride, f: impl FnOnce(&mut InsertHintSettings)) {
    if let Some(hint) = lo.insert_hint.as_mut() {
        f(hint);
    }
}

/// Enabling auto width clears proportion/fixed so niri emits one form.
pub fn set_default_column_width_auto(lo: &mut LayoutOverride, auto: Option<bool>) {
    lo.default_column_width_auto = auto;
    if auto == Some(true) {
        lo.default_column_width_proportion = None;
        lo.default_column_width_fixed = None;
    }
}

/// Setting a proportion width clears auto and fixed.
pub fn set_default_column_width_proportion(lo: &mut LayoutOverride, value: Option<f32>) {
    lo.default_column_width_proportion = value;
    if value.is_some() {
        lo.default_column_width_auto = None;
        lo.default_column_width_fixed = None;
    }
}

/// Setting a fixed width clears auto and proportion.
pub fn set_default_column_width_fixed(lo: &mut LayoutOverride, value: Option<i32>) {
    lo.default_column_width_fixed = value;
    if value.is_some() {
        lo.default_column_width_auto = None;
        lo.default_column_width_proportion = None;
    }
}

/// Compile-time proof that every `LayoutOverride` field is accounted for.
///
/// If a field is added to the model this function fails to compile until the
/// shared editor (and the table in the module docs) is updated.
pub fn layout_override_field_coverage(lo: &LayoutOverride) {
    let LayoutOverride {
        gaps,
        strut_left,
        strut_right,
        strut_top,
        strut_bottom,
        center_focused_column,
        always_center_single_column,
        empty_workspace_above_first,
        default_column_display,
        background_color,
        default_column_width_proportion,
        default_column_width_fixed,
        preset_column_widths,
        preset_window_heights,
        focus_ring_enabled,
        focus_ring_width,
        focus_ring_active,
        focus_ring_inactive,
        focus_ring_urgent,
        border_enabled,
        border_width,
        border_active,
        border_inactive,
        border_urgent,
        shadow_enabled,
        shadow_softness,
        shadow_spread,
        shadow_offset_x,
        shadow_offset_y,
        shadow_color,
        shadow_inactive_color,
        shadow_draw_behind_window,
        tab_indicator,
        insert_hint,
        default_column_width_auto,
    } = lo;
    let _ = (
        gaps,
        strut_left,
        strut_right,
        strut_top,
        strut_bottom,
        center_focused_column,
        always_center_single_column,
        empty_workspace_above_first,
        default_column_display,
        background_color,
        default_column_width_proportion,
        default_column_width_fixed,
        preset_column_widths,
        preset_window_heights,
        focus_ring_enabled,
        focus_ring_width,
        focus_ring_active,
        focus_ring_inactive,
        focus_ring_urgent,
        border_enabled,
        border_width,
        border_active,
        border_inactive,
        border_urgent,
        shadow_enabled,
        shadow_softness,
        shadow_spread,
        shadow_offset_x,
        shadow_offset_y,
        shadow_color,
        shadow_inactive_color,
        shadow_draw_behind_window,
        tab_indicator,
        insert_hint,
        default_column_width_auto,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{Message, OutputsMessage, WorkspacesMessage};
    use crate::types::{Color, ColorOrGradient, Gradient};
    use crate::views::widgets::GradientPickerMessage;

    fn sample_override() -> LayoutOverride {
        LayoutOverride {
            gaps: Some(8.0),
            strut_left: Some(1.0),
            strut_right: Some(2.0),
            strut_top: Some(3.0),
            strut_bottom: Some(4.0),
            center_focused_column: Some(CenterFocusedColumn::Always),
            always_center_single_column: Some(true),
            empty_workspace_above_first: Some(true),
            default_column_display: Some(DefaultColumnDisplay::Tabbed),
            background_color: Some(Color::from_hex("#123456").unwrap()),
            default_column_width_proportion: Some(0.5),
            default_column_width_fixed: None,
            preset_column_widths: Some(vec![PresetWidth::Proportion(0.5)]),
            preset_window_heights: Some(vec![PresetHeight::Fixed(800)]),
            focus_ring_enabled: Some(true),
            focus_ring_width: Some(3),
            focus_ring_active: Some(ColorOrGradient::default()),
            focus_ring_inactive: Some(ColorOrGradient::default()),
            focus_ring_urgent: Some(ColorOrGradient::default()),
            border_enabled: Some(true),
            border_width: Some(2),
            border_active: Some(ColorOrGradient::default()),
            border_inactive: Some(ColorOrGradient::default()),
            border_urgent: Some(ColorOrGradient::default()),
            shadow_enabled: Some(true),
            shadow_softness: Some(20),
            shadow_spread: Some(4),
            shadow_offset_x: Some(1),
            shadow_offset_y: Some(2),
            shadow_color: Some(Color::from_hex("#00000077").unwrap()),
            shadow_inactive_color: Some(Color::from_hex("#00000050").unwrap()),
            shadow_draw_behind_window: Some(true),
            tab_indicator: Some(TabIndicatorSettings::default()),
            insert_hint: Some(InsertHintSettings::default()),
            default_column_width_auto: None,
        }
    }

    #[test]
    fn coverage_function_sees_a_fully_populated_override() {
        layout_override_field_coverage(&sample_override());
        assert!(sample_override().has_any());
    }

    #[test]
    fn auto_width_clears_proportion_and_fixed() {
        let mut lo = sample_override();
        lo.default_column_width_fixed = Some(800);
        set_default_column_width_auto(&mut lo, Some(true));
        assert_eq!(lo.default_column_width_auto, Some(true));
        assert_eq!(lo.default_column_width_proportion, None);
        assert_eq!(lo.default_column_width_fixed, None);
    }

    #[test]
    fn proportion_width_clears_auto_and_fixed() {
        let mut lo = LayoutOverride {
            default_column_width_auto: Some(true),
            default_column_width_fixed: Some(640),
            ..LayoutOverride::default()
        };
        set_default_column_width_proportion(&mut lo, Some(0.4));
        assert_eq!(lo.default_column_width_proportion, Some(0.4));
        assert_eq!(lo.default_column_width_auto, None);
        assert_eq!(lo.default_column_width_fixed, None);
    }

    #[test]
    fn fixed_width_clears_auto_and_proportion() {
        let mut lo = LayoutOverride {
            default_column_width_auto: Some(true),
            default_column_width_proportion: Some(0.3),
            ..LayoutOverride::default()
        };
        set_default_column_width_fixed(&mut lo, Some(720));
        assert_eq!(lo.default_column_width_fixed, Some(720));
        assert_eq!(lo.default_column_width_auto, None);
        assert_eq!(lo.default_column_width_proportion, None);
    }

    #[test]
    fn edit_preserves_fields_the_caller_does_not_touch() {
        let lo = sample_override();
        let urgent = lo.focus_ring_urgent.clone();
        let tab = lo.tab_indicator.clone();
        let next = {
            let mut next = lo.clone();
            next.gaps = Some(16.0);
            next
        };
        assert_eq!(next.gaps, Some(16.0));
        assert_eq!(next.focus_ring_urgent, urgent);
        assert_eq!(next.tab_indicator, tab);
        assert_eq!(next.shadow_inactive_color, lo.shadow_inactive_color);
        assert_eq!(next.insert_hint, lo.insert_hint);
    }

    #[test]
    fn tab_indicator_gradient_edit_does_not_flatten() {
        let gradient = ColorOrGradient::Gradient(Gradient {
            from: Color::from_hex("#ff0000").unwrap(),
            to: Color::from_hex("#0000ff").unwrap(),
            angle: 45,
            ..Default::default()
        });
        let mut lo = LayoutOverride {
            tab_indicator: Some(TabIndicatorSettings {
                active: gradient.clone(),
                use_active_color: true,
                ..TabIndicatorSettings::default()
            }),
            ..LayoutOverride::default()
        };
        map_tab(&mut lo, |t| {
            apply_gradient_message(&mut t.active, GradientPickerMessage::SetAngle(90));
        });
        let tab = lo.tab_indicator.as_ref().unwrap();
        assert!(tab.active.is_gradient());
        match &tab.active {
            ColorOrGradient::Gradient(g) => assert_eq!(g.angle, 90),
            ColorOrGradient::Color(_) => panic!("gradient flattened"),
        }
    }

    #[test]
    fn both_call_sites_wrap_the_same_override() {
        let lo = sample_override();
        let outputs = Message::Outputs(OutputsMessage::SetLayoutOverride(0, Some(lo.clone())));
        let workspaces = Message::Workspaces(WorkspacesMessage::SetLayoutOverride(
            0,
            Some(Box::new(lo.clone())),
        ));
        match outputs {
            Message::Outputs(OutputsMessage::SetLayoutOverride(0, Some(got))) => {
                assert_eq!(got.tab_indicator, lo.tab_indicator);
                assert_eq!(got.shadow_draw_behind_window, Some(true));
                assert_eq!(got.default_column_width_auto, None);
            }
            other => panic!("unexpected {other:?}"),
        }
        match workspaces {
            Message::Workspaces(WorkspacesMessage::SetLayoutOverride(0, Some(got))) => {
                assert_eq!(*got, lo);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
