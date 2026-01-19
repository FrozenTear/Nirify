//! Behavior settings page

use freya::prelude::*;

use crate::config::{ColumnWidthType, SettingsCategory};
use crate::types::{CenterFocusedColumn, ModKey, WarpMouseMode};
use crate::ui::app::{
    ReactiveState, DROPDOWN_BEHAVIOR_CENTER_COLUMN, DROPDOWN_BEHAVIOR_COLUMN_WIDTH_TYPE,
    DROPDOWN_BEHAVIOR_MOD_KEY, DROPDOWN_BEHAVIOR_WARP_MOUSE,
};
use crate::ui::components::{section, select_row_with_state, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Warp mouse to focus options
const WARP_MOUSE_OPTIONS: &[&str] = &["Off", "Center XY", "Center XY (Always)"];

/// Center focused column options
const CENTER_COLUMN_OPTIONS: &[&str] = &["Never", "On Overflow", "Always"];

/// Column width type options
const COLUMN_WIDTH_TYPE_OPTIONS: &[&str] = &["Proportion", "Fixed"];

/// Modifier key options
const MOD_KEY_OPTIONS: &[&str] = &["Super", "Alt", "Ctrl", "Shift", "Mod3", "Mod5"];

fn warp_mouse_to_index(m: WarpMouseMode) -> usize {
    match m {
        WarpMouseMode::Off => 0,
        WarpMouseMode::CenterXY => 1,
        WarpMouseMode::CenterXYAlways => 2,
    }
}

fn index_to_warp_mouse(i: usize) -> WarpMouseMode {
    match i {
        0 => WarpMouseMode::Off,
        1 => WarpMouseMode::CenterXY,
        2 => WarpMouseMode::CenterXYAlways,
        _ => WarpMouseMode::Off,
    }
}

fn center_column_to_index(c: CenterFocusedColumn) -> usize {
    match c {
        CenterFocusedColumn::Never => 0,
        CenterFocusedColumn::OnOverflow => 1,
        CenterFocusedColumn::Always => 2,
    }
}

fn index_to_center_column(i: usize) -> CenterFocusedColumn {
    match i {
        0 => CenterFocusedColumn::Never,
        1 => CenterFocusedColumn::OnOverflow,
        2 => CenterFocusedColumn::Always,
        _ => CenterFocusedColumn::Never,
    }
}

fn column_width_type_to_index(t: ColumnWidthType) -> usize {
    match t {
        ColumnWidthType::Proportion => 0,
        ColumnWidthType::Fixed => 1,
    }
}

fn index_to_column_width_type(i: usize) -> ColumnWidthType {
    match i {
        0 => ColumnWidthType::Proportion,
        1 => ColumnWidthType::Fixed,
        _ => ColumnWidthType::Proportion,
    }
}

fn mod_key_to_index(m: ModKey) -> usize {
    match m {
        ModKey::Super => 0,
        ModKey::Alt => 1,
        ModKey::Ctrl => 2,
        ModKey::Shift => 3,
        ModKey::Mod3 => 4,
        ModKey::Mod5 => 5,
    }
}

fn index_to_mod_key(i: usize) -> ModKey {
    match i {
        0 => ModKey::Super,
        1 => ModKey::Alt,
        2 => ModKey::Ctrl,
        3 => ModKey::Shift,
        4 => ModKey::Mod3,
        5 => ModKey::Mod5,
        _ => ModKey::Super,
    }
}

/// Create the behavior settings page
pub fn behavior_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let behavior = &settings.behavior;

    let warp_mouse = behavior.warp_mouse_to_focus;
    let center_column = behavior.center_focused_column;
    let column_width_type = behavior.default_column_width_type;
    let mod_key = behavior.mod_key;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();
    let state6 = state.clone();
    let mut refresh6 = state.refresh.clone();
    let state7 = state.clone();
    let mut refresh7 = state.refresh.clone();
    let state8 = state.clone();
    let mut refresh8 = state.refresh.clone();
    let state9 = state.clone();
    let mut refresh9 = state.refresh.clone();
    let state10 = state.clone();
    let mut refresh10 = state.refresh.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Focus section
        .child(section(
            "Focus",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Focus follows mouse",
                    "Automatically focus window under cursor",
                    behavior.focus_follows_mouse,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.focus_follows_mouse = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Warp mouse to focus",
                        "Move cursor when focus changes",
                        WARP_MOUSE_OPTIONS,
                        warp_mouse_to_index(warp_mouse),
                        DROPDOWN_BEHAVIOR_WARP_MOUSE,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Behavior, |s| {
                                s.behavior.warp_mouse_to_focus = index_to_warp_mouse(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(toggle_row(
                    "Workspace back and forth",
                    "Switch to previous workspace with same key",
                    behavior.workspace_auto_back_and_forth,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.workspace_auto_back_and_forth = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Layout section
        .child(section(
            "Layout",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Center focused column",
                        "When to center the focused column",
                        CENTER_COLUMN_OPTIONS,
                        center_column_to_index(center_column),
                        DROPDOWN_BEHAVIOR_CENTER_COLUMN,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Behavior, |s| {
                                s.behavior.center_focused_column = index_to_center_column(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(toggle_row(
                    "Center single column",
                    "Always center when only one column exists",
                    behavior.always_center_single_column,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.always_center_single_column = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Empty workspace above first",
                    "Add empty workspace above the first one",
                    behavior.empty_workspace_above_first,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.empty_workspace_above_first = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Default column width type",
                        "How default column width is specified",
                        COLUMN_WIDTH_TYPE_OPTIONS,
                        column_width_type_to_index(column_width_type),
                        DROPDOWN_BEHAVIOR_COLUMN_WIDTH_TYPE,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Behavior, |s| {
                                s.behavior.default_column_width_type = index_to_column_width_type(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(slider_row(
                    "Default column width",
                    "Width proportion for new columns (0.5 = half)",
                    behavior.default_column_width_proportion as f64,
                    0.1,
                    2.0,
                    "",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.default_column_width_proportion = val as f32
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Screen Margins section
        .child(section(
            "Screen Margins (Struts)",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Left margin",
                    "Reserved space on left edge",
                    behavior.strut_left as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state6.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_left = val as f32
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Right margin",
                    "Reserved space on right edge",
                    behavior.strut_right as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state7.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_right = val as f32
                        });
                        refresh7.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Top margin",
                    "Reserved space on top edge",
                    behavior.strut_top as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state8.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_top = val as f32
                        });
                        refresh8.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Bottom margin",
                    "Reserved space on bottom edge",
                    behavior.strut_bottom as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state9.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.strut_bottom = val as f32
                        });
                        refresh9.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // System section
        .child(section(
            "System",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Modifier key",
                        "Primary modifier for compositor shortcuts",
                        MOD_KEY_OPTIONS,
                        mod_key_to_index(mod_key),
                        DROPDOWN_BEHAVIOR_MOD_KEY,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Behavior, |s| {
                                s.behavior.mod_key = index_to_mod_key(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(toggle_row(
                    "Disable power key handling",
                    "Let the system handle the power button",
                    behavior.disable_power_key_handling,
                    move |val| {
                        state10.update_and_save(SettingsCategory::Behavior, |s| {
                            s.behavior.disable_power_key_handling = val
                        });
                        refresh10.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
