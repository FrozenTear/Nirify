//! Helper functions for window rules
//!
//! Contains conversion utilities and setting model factory functions.

use crate::WindowRuleSettingModel;
use slint::{ModelRc, SharedString, VecModel};

/// Convert combo index to Option<bool>: 0=None, 1=Some(true), 2=Some(false)
pub fn index_to_option_bool(index: i32) -> Option<bool> {
    match index {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

/// Convert Option<bool> to combo index
pub fn option_bool_to_index(opt: Option<bool>) -> i32 {
    match opt {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    }
}

// Helper functions to create setting models

pub fn make_combo_setting(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
) -> WindowRuleSettingModel {
    make_combo_setting_visible(id, label, desc, index, options, true)
}

pub fn make_combo_setting_visible(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(
            options
                .iter()
                .map(|s| SharedString::from(*s))
                .collect::<Vec<_>>(),
        )),
        visible,
        ..Default::default()
    }
}

pub fn make_slider_int_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: i32,
    max: i32,
    suffix: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        int_value: value,
        min_value: min as f32,
        max_value: max as f32,
        suffix: suffix.into(),
        use_float: false,
        visible,
        ..Default::default()
    }
}

pub fn make_slider_float_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        float_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: true,
        visible,
        ..Default::default()
    }
}

pub fn make_color_setting_visible(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    visible: bool,
) -> WindowRuleSettingModel {
    WindowRuleSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 4,
        text_value: value.into(),
        visible,
        ..Default::default()
    }
}
