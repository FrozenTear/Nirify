//! Populate functions for window rules UI models
//!
//! Functions that create and populate UI model data from WindowRule settings.

use crate::config::models::{
    DefaultColumnDisplay, OpenBehavior, TabIndicatorPosition, WindowRule, WindowRuleMatch,
};
use crate::MainWindow;
use crate::WindowRuleSettingModel;
use slint::{ModelRc, SharedString, VecModel};

use super::helpers::{
    make_color_setting_visible, make_combo_setting, make_combo_setting_visible,
    make_slider_float_visible, make_slider_int_visible, option_bool_to_index,
};

/// Build rule list model for UI display
pub fn build_rule_list_model(rules: &[WindowRule]) -> ModelRc<SharedString> {
    let names: Vec<SharedString> = rules.iter().map(|r| r.name.as_str().into()).collect();
    ModelRc::new(VecModel::from(names))
}

/// Build matches list model for UI display
pub fn build_matches_list_model(matches: &[WindowRuleMatch]) -> ModelRc<SharedString> {
    let labels: Vec<SharedString> = matches
        .iter()
        .enumerate()
        .map(|(i, _)| format!("Match {}", i + 1).into())
        .collect();
    ModelRc::new(VecModel::from(labels))
}

/// Populate all models for a rule
pub fn populate_rule_models(ui: &MainWindow, rule: &WindowRule, match_idx: i32) {
    // Matches list
    ui.set_window_rules_matches_list_dynamic(build_matches_list_model(&rule.matches));
    ui.set_selected_match_index_dynamic(match_idx);

    // Rule settings
    populate_rule_settings(ui, rule);
    populate_match_settings(ui, rule, match_idx);
    populate_opening_settings(ui, rule);
    populate_visual_settings(ui, rule);
    populate_shadow_settings(ui, rule);
    populate_tab_settings(ui, rule);
}

pub fn populate_rule_settings(ui: &MainWindow, rule: &WindowRule) {
    let model = vec![WindowRuleSettingModel {
        id: "rule_name".into(),
        label: "Rule name".into(),
        description: "Display name for this rule".into(),
        setting_type: 3, // text
        text_value: rule.name.as_str().into(),
        placeholder: "Rule name".into(),
        visible: true,
        ..Default::default()
    }];
    ui.set_window_rules_rule_settings(ModelRc::new(VecModel::from(model)));
}

pub fn populate_match_settings(ui: &MainWindow, rule: &WindowRule, match_idx: i32) {
    let m = rule
        .matches
        .get(match_idx as usize)
        .cloned()
        .unwrap_or_default();

    let model = vec![
        WindowRuleSettingModel {
            id: "match_app_id".into(),
            label: "App ID".into(),
            description: "Application identifier (regex)".into(),
            setting_type: 3,
            text_value: m.app_id.unwrap_or_default().into(),
            placeholder: "e.g., firefox".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "match_title".into(),
            label: "Window title".into(),
            description: "Window title pattern (regex)".into(),
            setting_type: 3,
            text_value: m.title.unwrap_or_default().into(),
            placeholder: "e.g., .*YouTube.*".into(),
            visible: true,
            ..Default::default()
        },
        make_combo_setting(
            "match_is_floating",
            "Is floating",
            "Match floating windows",
            option_bool_to_index(m.is_floating),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_active",
            "Is active",
            "Match active windows",
            option_bool_to_index(m.is_active),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_focused",
            "Is focused",
            "Match focused windows",
            option_bool_to_index(m.is_focused),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_active_in_column",
            "Is active in column",
            "Match windows active in column",
            option_bool_to_index(m.is_active_in_column),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_window_cast_target",
            "Is window cast target",
            "Match screen-casted windows",
            option_bool_to_index(m.is_window_cast_target),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_is_urgent",
            "Is urgent",
            "Match urgent windows",
            option_bool_to_index(m.is_urgent),
            &["Any", "Yes", "No"],
        ),
        make_combo_setting(
            "match_at_startup",
            "At startup",
            "Match only at window creation",
            option_bool_to_index(m.at_startup),
            &["Any", "Yes", "No"],
        ),
    ];
    ui.set_window_rules_match_settings(ModelRc::new(VecModel::from(model)));
}

pub fn populate_opening_settings(ui: &MainWindow, rule: &WindowRule) {
    let is_floating = matches!(rule.open_behavior, OpenBehavior::Floating);
    let has_position = rule.default_floating_position.is_some();
    let pos = rule.default_floating_position.clone().unwrap_or_default();

    let model = vec![
        make_combo_setting(
            "open_behavior",
            "Open as",
            "How window opens",
            rule.open_behavior.to_index(),
            &["Normal", "Maximized", "Fullscreen", "Floating"],
        ),
        WindowRuleSettingModel {
            id: "open_on_output".into(),
            label: "Open on output".into(),
            description: "Force open on specific monitor".into(),
            setting_type: 3,
            text_value: rule.open_on_output.clone().unwrap_or_default().into(),
            placeholder: "e.g., eDP-1".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "open_on_workspace".into(),
            label: "Open on workspace".into(),
            description: "Force open on specific workspace".into(),
            setting_type: 3,
            text_value: rule.open_on_workspace.clone().unwrap_or_default().into(),
            placeholder: "e.g., browser".into(),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "has_floating_position".into(),
            label: "Set position".into(),
            description: "Specify floating window position".into(),
            setting_type: 0, // toggle
            bool_value: has_position,
            visible: is_floating,
            ..Default::default()
        },
        make_combo_setting_visible(
            "floating_relative_to",
            "Relative to",
            "Position reference",
            pos.relative_to.to_index(),
            &[
                "Top-Left",
                "Top-Right",
                "Bottom-Left",
                "Bottom-Right",
                "Top",
                "Bottom",
                "Left",
                "Right",
                "Center",
            ],
            is_floating && has_position,
        ),
        make_slider_int_visible(
            "floating_x",
            "X offset",
            "Horizontal offset",
            pos.x,
            -500,
            500,
            "px",
            is_floating && has_position,
        ),
        make_slider_int_visible(
            "floating_y",
            "Y offset",
            "Vertical offset",
            pos.y,
            -500,
            500,
            "px",
            is_floating && has_position,
        ),
    ];
    ui.set_window_rules_opening_settings(ModelRc::new(VecModel::from(model)));
}

pub fn populate_visual_settings(ui: &MainWindow, rule: &WindowRule) {
    let has_opacity = rule.opacity.is_some();
    let has_radius = rule.corner_radius.is_some();

    let model = vec![
        WindowRuleSettingModel {
            id: "has_opacity".into(),
            label: "Custom opacity".into(),
            description: "Set window transparency".into(),
            setting_type: 0,
            bool_value: has_opacity,
            visible: true,
            ..Default::default()
        },
        make_slider_float_visible(
            "opacity",
            "Opacity",
            "Transparency level",
            rule.opacity.unwrap_or(1.0),
            0.0,
            1.0,
            "%",
            has_opacity,
        ),
        WindowRuleSettingModel {
            id: "has_corner_radius".into(),
            label: "Geometry corner radius".into(),
            description: "Round corners".into(),
            setting_type: 0,
            bool_value: has_radius,
            visible: true,
            ..Default::default()
        },
        make_slider_int_visible(
            "corner_radius",
            "Radius",
            "Corner radius",
            rule.corner_radius.unwrap_or(12),
            0,
            32,
            "px",
            has_radius,
        ),
        WindowRuleSettingModel {
            id: "clip_to_geometry".into(),
            label: "Clip to geometry".into(),
            description: "Clip window to visual geometry".into(),
            setting_type: 0,
            bool_value: rule.clip_to_geometry.unwrap_or(false),
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "block_screencast".into(),
            label: "Block from screencast".into(),
            description: "Hide in screen recordings".into(),
            setting_type: 0,
            bool_value: rule.block_out_from_screencast,
            visible: true,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "baba_is_float".into(),
            label: "Animated floating".into(),
            description: "Enable baba-is-float effect".into(),
            setting_type: 0,
            bool_value: rule.baba_is_float.unwrap_or(false),
            visible: true,
            ..Default::default()
        },
        make_combo_setting(
            "vrr",
            "Variable Refresh Rate",
            "Enable VRR for this window",
            match rule.variable_refresh_rate {
                Some(true) => 1,
                Some(false) => 2,
                None => 0,
            },
            &["Default", "On", "Off"],
        ),
        make_combo_setting(
            "column_display",
            "Column display",
            "Window column display mode",
            match rule.default_column_display {
                Some(DefaultColumnDisplay::Tabbed) => 1,
                _ => 0,
            },
            &["Default", "Tabbed"],
        ),
        make_combo_setting(
            "tiled_state",
            "Tiled state",
            "Mark as tiled or floating",
            match rule.tiled_state {
                Some(true) => 1,
                Some(false) => 2,
                None => 0,
            },
            &["Default", "Tiled", "Floating"],
        ),
    ];
    ui.set_window_rules_visual_settings(ModelRc::new(VecModel::from(model)));
}

pub fn populate_shadow_settings(ui: &MainWindow, rule: &WindowRule) {
    let mode = match &rule.shadow {
        Some(s) if s.enabled => 1,
        Some(_) => 2,
        None => 0,
    };
    let shadow = rule.shadow.clone().unwrap_or_default();
    let show_details = mode == 1;

    let model = vec![
        make_combo_setting(
            "shadow_mode",
            "Shadow",
            "Override window shadow",
            mode,
            &["Default", "Custom", "Off"],
        ),
        make_slider_int_visible(
            "shadow_softness",
            "Softness",
            "Blur radius",
            shadow.softness,
            0,
            100,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_spread",
            "Spread",
            "Spread distance",
            shadow.spread,
            -50,
            50,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_offset_x",
            "Offset X",
            "Horizontal offset",
            shadow.offset_x,
            -50,
            50,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "shadow_offset_y",
            "Offset Y",
            "Vertical offset",
            shadow.offset_y,
            -50,
            50,
            "px",
            show_details,
        ),
        make_color_setting_visible(
            "shadow_color",
            "Active color",
            "Active shadow color",
            &shadow.color.to_hex(),
            show_details,
        ),
        make_color_setting_visible(
            "shadow_inactive_color",
            "Inactive color",
            "Inactive shadow color",
            &shadow.inactive_color.to_hex(),
            show_details,
        ),
        WindowRuleSettingModel {
            id: "shadow_draw_behind".into(),
            label: "Draw behind window".into(),
            description: "Render behind opaque areas".into(),
            setting_type: 0,
            bool_value: shadow.draw_behind_window,
            visible: show_details,
            ..Default::default()
        },
    ];
    ui.set_window_rules_shadow_settings(ModelRc::new(VecModel::from(model)));
}

pub fn populate_tab_settings(ui: &MainWindow, rule: &WindowRule) {
    let mode = match &rule.tab_indicator {
        Some(ti) if ti.enabled => 1,
        Some(_) => 2,
        None => 0,
    };
    let ti = rule.tab_indicator.clone().unwrap_or_default();
    let show_details = mode == 1;

    let model = vec![
        make_combo_setting(
            "tab_mode",
            "Tab indicator",
            "Override tab indicator",
            mode,
            &["Default", "Custom", "Off"],
        ),
        WindowRuleSettingModel {
            id: "tab_hide_when_single".into(),
            label: "Hide when single tab".into(),
            description: "Hide with only one window".into(),
            setting_type: 0,
            bool_value: ti.hide_when_single_tab,
            visible: show_details,
            ..Default::default()
        },
        WindowRuleSettingModel {
            id: "tab_place_within_column".into(),
            label: "Place within column".into(),
            description: "Shrink column to fit".into(),
            setting_type: 0,
            bool_value: ti.place_within_column,
            visible: show_details,
            ..Default::default()
        },
        make_combo_setting_visible(
            "tab_position",
            "Position",
            "Tab indicator placement",
            match ti.position {
                TabIndicatorPosition::Left => 0,
                TabIndicatorPosition::Right => 1,
                TabIndicatorPosition::Top => 2,
                TabIndicatorPosition::Bottom => 3,
            },
            &["Left", "Right", "Top", "Bottom"],
            show_details,
        ),
        make_slider_int_visible(
            "tab_gap",
            "Gap",
            "Space from window",
            ti.gap,
            0,
            32,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "tab_width",
            "Width",
            "Indicator thickness",
            ti.width,
            1,
            32,
            "px",
            show_details,
        ),
        make_slider_float_visible(
            "tab_length",
            "Length",
            "Proportion of window",
            ti.length_proportion,
            0.0,
            1.0,
            "%",
            show_details,
        ),
        make_slider_int_visible(
            "tab_gaps_between",
            "Gaps between tabs",
            "Space between segments",
            ti.gaps_between_tabs,
            0,
            16,
            "px",
            show_details,
        ),
        make_slider_int_visible(
            "tab_corner_radius",
            "Corner radius",
            "Indicator rounding",
            ti.corner_radius,
            0,
            32,
            "px",
            show_details,
        ),
        make_color_setting_visible(
            "tab_active_color",
            "Active color",
            "Active tab color",
            &ti.active.to_hex(),
            show_details,
        ),
        make_color_setting_visible(
            "tab_inactive_color",
            "Inactive color",
            "Inactive tab color",
            &ti.inactive.to_hex(),
            show_details,
        ),
    ];
    ui.set_window_rules_tab_settings(ModelRc::new(VecModel::from(model)));
}
