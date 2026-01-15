//! Helper macros for reducing callback boilerplate
//!
//! This module provides macros that standardize callback registration patterns,
//! reducing code duplication and ensuring consistent error handling.
//!
//! All macros accept a `save_manager` parameter (Rc<SaveManager>) which is
//! captured by closures to trigger debounced saves, and a `category` parameter
//! (SettingsCategory) to enable dirty tracking.
//!
//! # Individual vs Batch Macros
//!
//! There are two types of macros in this module:
//!
//! - **Individual macros** (`register_bool_callback!`, etc.): Register a single
//!   callback with full control over the settings path expression.
//!
//! - **Batch macros** (`register_bool_callbacks!`, etc.): Register multiple
//!   callbacks at once when they follow the same pattern.
//!
//! # Type-Safe Batch Macros
//!
//! Batch macros use the `CategorySection` trait for compile-time safety.
//! Instead of passing separate category and section identifiers (error-prone),
//! you pass a single marker type that encodes both:
//!
//! ```ignore
//! use crate::config::category_section::Appearance;
//!
//! // Type-safe: the Appearance marker ensures category and section match
//! register_bool_callbacks!(ui, settings, save_manager, Appearance, [
//!     (on_focus_ring_toggled, focus_ring_enabled, "Focus ring enabled"),
//!     (on_border_toggled, border_enabled, "Border enabled"),
//! ]);
//! ```
//!
//! Available marker types (from `config::category_section`):
//! - `Appearance`, `Behavior`, `Keyboard`, `Mouse`, `Touchpad`, `Trackpoint`
//! - `Trackball`, `Tablet`, `Touch`, `Outputs`, `Animations`, `Cursor`
//! - `Overview`, `Workspaces`, `Keybindings`, `LayoutExtras`, `Gestures`
//! - `LayerRules`, `WindowRules`, `Miscellaneous`, `Startup`, `Environment`
//! - `Debug`, `SwitchEvents`, `RecentWindows`
//!
//! # When to Use Batch vs Individual Macros
//!
//! Use **batch macros** when:
//! - Multiple callbacks follow the same pattern
//! - The field is directly on the section struct (e.g., `s.appearance.field`)
//! - No custom logic is needed
//!
//! Use **individual macros** (or hand-written callbacks) when:
//! - The callback has custom logic (e.g., setting defaults on enable)
//! - The field path is complex (e.g., `s.overview.workspace_shadow.as_mut()`)
//! - Multiple fields need to be updated
//! - Type conversion is needed beyond what the macro provides

/// Register a boolean toggle callback that updates a settings field
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name (e.g., `on_feature_toggled`)
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable
/// * `$msg` - Debug message prefix
///
/// # Example
/// ```ignore
/// register_bool_callback!(
///     ui, on_border_toggled, settings, save_manager,
///     SettingsCategory::Appearance,
///     |s| s.appearance.border_enabled,
///     "Border enabled"
/// );
/// ```
macro_rules! register_bool_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |enabled| match settings.lock() {
            Ok(mut $s) => {
                $field = enabled;
                log::debug!("{}: {}", $msg, enabled);
                save_manager.mark_dirty(category);
                save_manager.request_save();
            }
            Err(e) => log::error!(
                "Settings lock error in {}: {} (updating {})",
                stringify!($callback),
                e,
                stringify!($field)
            ),
        });
    }};
}

/// Register a numeric callback with clamping
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `$min` - Minimum value
/// * `$max` - Maximum value
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable
/// * `$msg` - Debug message with format placeholder for value
///
/// # Example
/// ```ignore
/// register_clamped_callback!(
///     ui, on_width_changed, settings, save_manager,
///     SettingsCategory::Appearance,
///     FOCUS_RING_WIDTH_MIN, FOCUS_RING_WIDTH_MAX,
///     |s| s.appearance.focus_ring_width,
///     "Focus ring width: {}px"
/// );
/// ```
macro_rules! register_clamped_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, $min:expr, $max:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |val| {
            let clamped = val.clamp($min, $max);
            match settings.lock() {
                Ok(mut $s) => {
                    $field = clamped;
                    log::debug!($msg, clamped);
                    save_manager.mark_dirty(category);
                    save_manager.request_save();
                }
                Err(e) => log::error!(
                    "Settings lock error in {}: {} (updating {})",
                    stringify!($callback),
                    e,
                    stringify!($field)
                ),
            }
        });
    }};
}

/// Register a numeric callback with clamping and type conversion to f64
///
/// Similar to `register_clamped_callback!` but converts to f64 before clamping
macro_rules! register_clamped_f64_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, $min:expr, $max:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |val| {
            let clamped = (val as f64).clamp($min, $max);
            match settings.lock() {
                Ok(mut $s) => {
                    $field = clamped;
                    log::debug!($msg, clamped);
                    save_manager.mark_dirty(category);
                    save_manager.request_save();
                }
                Err(e) => log::error!(
                    "Settings lock error in {}: {} (updating {})",
                    stringify!($callback),
                    e,
                    stringify!($field)
                ),
            }
        });
    }};
}

/// Register a string conversion callback
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable
/// * `$msg` - Debug message prefix
///
/// # Example
/// ```ignore
/// register_string_callback!(
///     ui, on_theme_changed, settings, save_manager,
///     SettingsCategory::Cursor,
///     |s| s.cursor.theme,
///     "Cursor theme"
/// );
/// ```
macro_rules! register_string_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |str_val| {
            let str_string: String = str_val.into();
            match settings.lock() {
                Ok(mut $s) => {
                    $field = str_string.clone();
                    log::debug!(
                        "{}: {}",
                        $msg,
                        if str_string.is_empty() {
                            "(none)"
                        } else {
                            &str_string
                        }
                    );
                    save_manager.mark_dirty(category);
                    save_manager.request_save();
                }
                Err(e) => log::error!(
                    "Settings lock error in {}: {} (updating {})",
                    stringify!($callback),
                    e,
                    stringify!($field)
                ),
            }
        });
    }};
}

/// Register a color change callback
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable
/// * `$msg` - Debug message
macro_rules! register_color_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |color| match settings.lock() {
            Ok(mut $s) => {
                $field = crate::ui::bridge::converters::slint_color_to_color(color);
                log::debug!("{}", $msg);
                save_manager.mark_dirty(category);
                save_manager.request_save();
            }
            Err(e) => log::error!(
                "Settings lock error in {}: {} (updating {})",
                stringify!($callback),
                e,
                stringify!($field)
            ),
        });
    }};
}

/// Register a color change callback for ColorOrGradient fields
///
/// Similar to `register_color_callback!` but calls `set_color()` on a
/// ColorOrGradient field, converting any gradient to a solid color.
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable (must be ColorOrGradient)
/// * `$msg` - Debug message
macro_rules! register_color_or_gradient_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |color| match settings.lock() {
            Ok(mut $s) => {
                $field.set_color(crate::ui::bridge::converters::slint_color_to_color(color));
                log::debug!("{}", $msg);
                save_manager.mark_dirty(category);
                save_manager.request_save();
            }
            Err(e) => log::error!(
                "Settings lock error in {}: {} (updating {})",
                stringify!($callback),
                e,
                stringify!($field)
            ),
        });
    }};
}

/// Register an optional i32 callback that converts positive values to Some, zero/negative to None
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable (must be Option<i32>)
/// * `$msg` - Debug message prefix
///
/// # Example
/// ```ignore
/// register_option_i32_callback!(
///     ui, on_mouse_scroll_button_changed, settings, save_manager,
///     SettingsCategory::Mouse,
///     |s| s.mouse.scroll_button,
///     "Mouse scroll button"
/// );
/// ```
macro_rules! register_option_i32_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |val| match settings.lock() {
            Ok(mut $s) => {
                $field = if val > 0 { Some(val) } else { None };
                log::debug!("{}: {:?}", $msg, $field);
                save_manager.mark_dirty(category);
                save_manager.request_save();
            }
            Err(e) => log::error!(
                "Settings lock error in {}: {} (updating {})",
                stringify!($callback),
                e,
                stringify!($field)
            ),
        });
    }};
}

/// Register an enum index callback that converts from i32 index to enum
///
/// # Arguments
/// * `$ui` - The MainWindow reference
/// * `$callback` - The callback method name
/// * `$settings` - Arc<Mutex<Settings>> to clone
/// * `$save_manager` - Rc<SaveManager> for debounced saving
/// * `$category` - SettingsCategory for dirty tracking
/// * `|$s|` - Closure pattern binding the settings guard
/// * `$field` - Field expression using the bound variable
/// * `$enum_type` - The enum type that implements `from_index(i32)`
/// * `$msg` - Debug message prefix
///
/// # Example
/// ```ignore
/// register_enum_callback!(
///     ui, on_mouse_accel_profile_changed, settings, save_manager,
///     SettingsCategory::Mouse,
///     |s| s.mouse.accel_profile,
///     AccelProfile,
///     "Mouse accel profile"
/// );
/// ```
macro_rules! register_enum_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr, $category:expr, |$s:ident| $field:expr, $enum_type:ty, $msg:expr) => {{
        let settings = $settings.clone();
        let save_manager = Rc::clone(&$save_manager);
        let category = $category;
        $ui.$callback(move |idx| match settings.lock() {
            Ok(mut $s) => {
                $field = <$enum_type>::from_index(idx);
                log::debug!("{}: {:?}", $msg, $field);
                save_manager.mark_dirty(category);
                save_manager.request_save();
            }
            Err(e) => log::error!(
                "Settings lock error in {}: {} (updating {})",
                stringify!($callback),
                e,
                stringify!($field)
            ),
        });
    }};
}

// ============================================================================
// BATCH CALLBACK MACROS
// ============================================================================
// These macros allow registering multiple callbacks of the same type at once,
// reducing boilerplate while remaining fully debuggable and IDE-friendly.

/// Register multiple boolean toggle callbacks at once (type-safe version)
///
/// Uses the CategorySection trait for compile-time verification that the
/// category and section match correctly.
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Appearance;
///
/// register_bool_callbacks!(ui, settings, save_manager, Appearance, [
///     (focus_ring_toggled, focus_ring_enabled, "Focus ring enabled"),
///     (border_toggled, border_enabled, "Border enabled"),
/// ]);
/// ```
macro_rules! register_bool_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_bool_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

/// Register multiple clamped numeric callbacks at once (type-safe version)
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Appearance;
///
/// register_clamped_callbacks!(ui, settings, save_manager, Appearance, [
///     (focus_ring_width_changed, focus_ring_width, FOCUS_RING_WIDTH_MIN, FOCUS_RING_WIDTH_MAX, "Focus ring width: {}px"),
///     (gaps_inner_changed, gaps_inner, GAP_SIZE_MIN, GAP_SIZE_MAX, "Inner gaps: {}px"),
/// ]);
/// ```
macro_rules! register_clamped_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $min:expr, $max:expr, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_clamped_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                $min,
                $max,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

/// Register multiple color or gradient callbacks at once (type-safe version)
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Appearance;
///
/// register_color_or_gradient_callbacks!(ui, settings, save_manager, Appearance, [
///     (focus_ring_active_color_changed, focus_ring_active, "Focus ring active color changed"),
///     (focus_ring_inactive_color_changed, focus_ring_inactive, "Focus ring inactive color changed"),
/// ]);
/// ```
macro_rules! register_color_or_gradient_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_color_or_gradient_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

/// Register multiple clamped f64 callbacks at once (type-safe version)
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Mouse;
///
/// register_clamped_f64_callbacks!(ui, settings, save_manager, Mouse, [
///     (mouse_accel_speed_changed, accel_speed, ACCEL_SPEED_MIN, ACCEL_SPEED_MAX, "Accel speed: {:.2}"),
/// ]);
/// ```
macro_rules! register_clamped_f64_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $min:expr, $max:expr, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_clamped_f64_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                $min,
                $max,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

/// Register multiple string callbacks at once (type-safe version)
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Keyboard;
///
/// register_string_callbacks!(ui, settings, save_manager, Keyboard, [
///     (xkb_layout_changed, xkb_layout, "XKB layout"),
///     (xkb_variant_changed, xkb_variant, "XKB variant"),
/// ]);
/// ```
macro_rules! register_string_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_string_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

/// Register multiple enum index callbacks at once (type-safe version)
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Mouse;
///
/// register_enum_callbacks!(ui, settings, save_manager, Mouse, [
///     (on_mouse_accel_profile_changed, accel_profile, AccelProfile, "Mouse accel profile"),
///     (on_mouse_scroll_method_changed, scroll_method, ScrollMethod, "Mouse scroll method"),
/// ]);
/// ```
macro_rules! register_enum_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $enum_type:ty, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_enum_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $enum_type,
                $msg
            );
        )*
    }};
}

/// Register multiple optional i32 callbacks at once (type-safe version)
///
/// Converts positive values to Some(val), zero/negative to None.
/// Useful for scroll button settings.
///
/// # Example
/// ```ignore
/// use crate::config::category_section::Mouse;
///
/// register_option_i32_callbacks!(ui, settings, save_manager, Mouse, [
///     (on_mouse_scroll_button_changed, scroll_button, "Mouse scroll button"),
/// ]);
/// ```
macro_rules! register_option_i32_callbacks {
    ($ui:expr, $settings:expr, $save_manager:expr, $marker:ty, [
        $(($callback:ident, $field:ident, $msg:expr)),* $(,)?
    ]) => {{
        $(
            $crate::ui::bridge::macros::register_option_i32_callback!(
                $ui,
                $callback,
                $settings,
                $save_manager,
                <$marker as $crate::config::CategorySection>::CATEGORY,
                |s| <$marker as $crate::config::CategorySection>::section_mut(&mut s).$field,
                $msg
            );
        )*
    }};
}

// Export macros for use in submodules
pub(crate) use register_bool_callback;
pub(crate) use register_bool_callbacks;
pub(crate) use register_clamped_callback;
pub(crate) use register_clamped_callbacks;
pub(crate) use register_clamped_f64_callback;
pub(crate) use register_clamped_f64_callbacks;
pub(crate) use register_color_callback;
pub(crate) use register_color_or_gradient_callback;
pub(crate) use register_color_or_gradient_callbacks;
pub(crate) use register_enum_callback;
pub(crate) use register_enum_callbacks;
pub(crate) use register_option_i32_callback;
pub(crate) use register_option_i32_callbacks;
pub(crate) use register_string_callback;
pub(crate) use register_string_callbacks;

// Re-export types needed by macros
pub(crate) use super::save_manager::SaveManager;
