//! Setting model builder macros
//!
//! Provides macros to generate helper functions for creating setting models.
//! These macros eliminate code duplication across callback modules by generating
//! type-specific builder functions (make_toggle, make_slider_int, etc.) for any
//! Slint model type that follows the SettingModel pattern.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::impl_setting_builders;
//!
//! // Generate all builder functions for MouseSettingModel
//! impl_setting_builders!(MouseSettingModel);
//!
//! // Now use the functions
//! let toggle = make_toggle("id", "Label", "Description", true, true);
//! let slider = make_slider_float("id", "Label", "Desc", 50.0, 0.0, 100.0, "%", true);
//! ```

/// Generate setting model builder functions for a given model type.
///
/// This macro generates the following functions:
/// - `make_toggle` - Creates a toggle/switch setting (type 0)
/// - `make_slider_int` - Creates an integer slider setting (type 1, use_float=false)
/// - `make_slider_float` - Creates a float slider setting (type 1, use_float=true)
/// - `make_combo` - Creates a combobox/dropdown setting (type 2)
/// - `make_text` - Creates a text input setting (type 3)
/// - `make_color` - Creates a color picker setting (type 4)
///
/// All functions take common parameters (id, label, description, visible) plus
/// type-specific parameters (value, min/max for sliders, options for combo, etc.)
#[macro_export]
macro_rules! impl_setting_builders {
    ($model_type:ident) => {
        /// Create a toggle/switch setting model
        #[allow(dead_code)]
        fn make_toggle(
            id: &str,
            label: &str,
            desc: &str,
            value: bool,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 0,
                bool_value: value,
                visible,
                ..Default::default()
            }
        }

        /// Create an integer slider setting model
        #[allow(dead_code)]
        fn make_slider_int(
            id: &str,
            label: &str,
            desc: &str,
            value: i32,
            min: f32,
            max: f32,
            suffix: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 1,
                int_value: value,
                min_value: min,
                max_value: max,
                suffix: suffix.into(),
                use_float: false,
                visible,
                ..Default::default()
            }
        }

        /// Create a float slider setting model
        #[allow(dead_code)]
        fn make_slider_float(
            id: &str,
            label: &str,
            desc: &str,
            value: f32,
            min: f32,
            max: f32,
            suffix: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
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

        /// Create a combobox/dropdown setting model
        #[allow(dead_code)]
        fn make_combo(
            id: &str,
            label: &str,
            desc: &str,
            index: i32,
            options: &[&str],
            visible: bool,
        ) -> $model_type {
            let opts: Vec<slint::SharedString> =
                options.iter().map(|s| (*s).into()).collect();
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 2,
                combo_index: index,
                combo_options: slint::ModelRc::new(slint::VecModel::from(opts)),
                visible,
                ..Default::default()
            }
        }

        /// Create a text input setting model
        #[allow(dead_code)]
        fn make_text(
            id: &str,
            label: &str,
            desc: &str,
            value: &str,
            placeholder: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 3,
                text_value: value.into(),
                placeholder: placeholder.into(),
                visible,
                ..Default::default()
            }
        }

        /// Create a color picker setting model
        #[allow(dead_code)]
        fn make_color(
            id: &str,
            label: &str,
            desc: &str,
            hex_value: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 4,
                text_value: hex_value.into(),
                placeholder: "#RRGGBB".into(),
                visible,
                ..Default::default()
            }
        }
    };
}

/// Generate setting model builder functions with `_visible` suffix variants.
///
/// This variant generates functions with `_visible` suffix that take
/// integer min/max parameters for sliders (useful for window_rules, layer_rules).
#[macro_export]
macro_rules! impl_setting_builders_visible {
    ($model_type:ident) => {
        /// Create a toggle/switch setting model with explicit visibility
        #[allow(dead_code)]
        fn make_toggle_visible(
            id: &str,
            label: &str,
            desc: &str,
            value: bool,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 0,
                bool_value: value,
                visible,
                ..Default::default()
            }
        }

        /// Create an integer slider setting model with explicit visibility
        #[allow(dead_code)]
        fn make_slider_int_visible(
            id: &str,
            label: &str,
            desc: &str,
            value: i32,
            min: i32,
            max: i32,
            suffix: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
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

        /// Create a float slider setting model with explicit visibility
        #[allow(dead_code)]
        fn make_slider_float_visible(
            id: &str,
            label: &str,
            desc: &str,
            value: f32,
            min: f32,
            max: f32,
            suffix: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
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

        /// Create a combobox/dropdown setting model with explicit visibility
        #[allow(dead_code)]
        fn make_combo_visible(
            id: &str,
            label: &str,
            desc: &str,
            index: i32,
            options: &[&str],
            visible: bool,
        ) -> $model_type {
            let opts: Vec<slint::SharedString> =
                options.iter().map(|s| (*s).into()).collect();
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 2,
                combo_index: index,
                combo_options: slint::ModelRc::new(slint::VecModel::from(opts)),
                visible,
                ..Default::default()
            }
        }

        /// Create a text input setting model with explicit visibility
        #[allow(dead_code)]
        fn make_text_visible(
            id: &str,
            label: &str,
            desc: &str,
            value: &str,
            placeholder: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 3,
                text_value: value.into(),
                placeholder: placeholder.into(),
                visible,
                ..Default::default()
            }
        }

        /// Create a color picker setting model with explicit visibility
        #[allow(dead_code)]
        fn make_color_visible(
            id: &str,
            label: &str,
            desc: &str,
            hex_value: &str,
            visible: bool,
        ) -> $model_type {
            $model_type {
                id: id.into(),
                label: label.into(),
                description: desc.into(),
                setting_type: 4,
                text_value: hex_value.into(),
                placeholder: "#RRGGBB".into(),
                visible,
                ..Default::default()
            }
        }
    };
}
