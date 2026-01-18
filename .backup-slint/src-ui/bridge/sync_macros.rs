//! Sync macros for reducing UI synchronization boilerplate
//!
//! These macros reduce the repetitive `ui.set_*()` calls in sync.rs by providing
//! batch operations for common patterns.

/// Sync multiple boolean properties from settings to UI
///
/// # Example
/// ```ignore
/// sync_bool_props!(ui, settings.appearance, [
///     (focus_ring_enabled, set_focus_ring_enabled),
///     (border_enabled, set_border_enabled),
/// ]);
/// ```
macro_rules! sync_bool_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field);
        )*
    };
}

/// Sync multiple i32 properties from settings to UI
///
/// # Example
/// ```ignore
/// sync_i32_props!(ui, settings.keyboard, [
///     (repeat_delay, set_repeat_delay),
///     (repeat_rate, set_repeat_rate),
/// ]);
/// ```
macro_rules! sync_i32_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field);
        )*
    };
}

/// Sync multiple f32 properties from settings to UI
///
/// For fields already stored as f32 (no casting needed)
///
/// # Example
/// ```ignore
/// sync_f32_props!(ui, settings.appearance, [
///     (focus_ring_width, set_focus_ring_width),
///     (corner_radius, set_corner_radius),
/// ]);
/// ```
macro_rules! sync_f32_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field);
        )*
    };
}

/// Sync multiple enum-to-index properties from settings to UI
///
/// Uses `.to_index()` on the enum value (requires SlintIndex derive)
///
/// # Example
/// ```ignore
/// sync_enum_index_props!(ui, settings.mouse, [
///     (accel_profile, set_mouse_accel_profile_index),
///     (scroll_method, set_mouse_scroll_method_index),
/// ]);
/// ```
macro_rules! sync_enum_index_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field.to_index());
        )*
    };
}

/// Sync multiple f64-to-f32 properties from settings to UI
///
/// Casts f64 fields to f32 for Slint compatibility
///
/// # Example
/// ```ignore
/// sync_f64_as_f32_props!(ui, settings.mouse, [
///     (accel_speed, set_mouse_accel_speed),
///     (scroll_factor, set_mouse_scroll_factor),
/// ]);
/// ```
macro_rules! sync_f64_as_f32_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field as f32);
        )*
    };
}

/// Sync multiple string properties from settings to UI
///
/// Converts String to slint::SharedString via `.into()`
///
/// # Example
/// ```ignore
/// sync_string_props!(ui, settings.keyboard, [
///     (xkb_layout, set_xkb_layout),
///     (xkb_variant, set_xkb_variant),
/// ]);
/// ```
macro_rules! sync_string_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field.as_str().into());
        )*
    };
}

/// Sync multiple Color properties from settings to UI
///
/// Converts crate::types::Color to slint::Color
///
/// # Example
/// ```ignore
/// sync_color_props!(ui, settings.layout_extras.shadow, [
///     (color, set_shadow_color),
///     (inactive_color, set_shadow_inactive_color),
/// ]);
/// ```
macro_rules! sync_color_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($crate::ui::bridge::converters::color_to_slint_color(&$section.$field));
        )*
    };
}

/// Sync multiple Color hex string properties from settings to UI
///
/// Converts crate::types::Color to hex string SharedString
///
/// # Example
/// ```ignore
/// sync_color_hex_props!(ui, settings.layout_extras.shadow, [
///     (color, set_shadow_color_hex),
///     (inactive_color, set_shadow_inactive_color_hex),
/// ]);
/// ```
macro_rules! sync_color_hex_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field.to_hex().into());
        )*
    };
}

/// Sync multiple ColorOrGradient properties from settings to UI
///
/// Uses `.primary_color()` to get the Color, then converts to slint::Color
///
/// # Example
/// ```ignore
/// sync_color_or_gradient_props!(ui, settings.appearance, [
///     (focus_ring_active, set_focus_ring_active_color),
///     (focus_ring_inactive, set_focus_ring_inactive_color),
/// ]);
/// ```
macro_rules! sync_color_or_gradient_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($crate::ui::bridge::converters::color_to_slint_color($section.$field.primary_color()));
        )*
    };
}

/// Sync multiple ColorOrGradient hex string properties from settings to UI
///
/// Uses `.to_hex()` on the ColorOrGradient (which returns primary color hex)
///
/// # Example
/// ```ignore
/// sync_color_or_gradient_hex_props!(ui, settings.appearance, [
///     (focus_ring_active, set_focus_ring_active_hex),
///     (focus_ring_inactive, set_focus_ring_inactive_hex),
/// ]);
/// ```
macro_rules! sync_color_or_gradient_hex_props {
    ($ui:expr, $section:expr, [$(($field:ident, $setter:ident)),* $(,)?]) => {
        $(
            $ui.$setter($section.$field.to_hex().into());
        )*
    };
}

// Export macros for use in sync.rs
pub(crate) use sync_bool_props;
pub(crate) use sync_color_hex_props;
pub(crate) use sync_color_or_gradient_hex_props;
pub(crate) use sync_color_or_gradient_props;
pub(crate) use sync_color_props;
pub(crate) use sync_enum_index_props;
pub(crate) use sync_f32_props;
pub(crate) use sync_f64_as_f32_props;
pub(crate) use sync_i32_props;
pub(crate) use sync_string_props;
