//! Animation-related UI callbacks
//!
//! Handles animation enable/disable, slowdown factor, and per-animation configuration.
//! Uses indexed callbacks (6 total) instead of per-animation callbacks (66 total).

use crate::config::models::{AnimationId, AnimationType, EasingCurve};
use crate::config::{Settings, SettingsCategory};
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{register_bool_callback, SaveManager};

/// Register an indexed animation numeric field callback with clamping
///
/// Handles callbacks that take (id, value) and update a numeric field on the animation.
/// Takes pre-cloned references to avoid additional Arc clones.
macro_rules! register_anim_clamped_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr,
     $min:expr, $max:expr, |$anim:ident| $field:expr, $msg:expr, f64) => {{
        let settings = Arc::clone(&$settings);
        let save_manager = Rc::clone(&$save_manager);
        $ui.$callback(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = (val as f64).clamp($min, $max);
            match settings.lock() {
                Ok(mut s) => {
                    let $anim = anim_id.get_mut(&mut s.animations.per_animation);
                    $field = clamped;
                    debug!("{} {}: {}", anim_id.name(), $msg, clamped);
                    save_manager.mark_dirty(SettingsCategory::Animations);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }};
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr,
     $min:expr, $max:expr, |$anim:ident| $field:expr, $msg:expr, i32) => {{
        let settings = Arc::clone(&$settings);
        let save_manager = Rc::clone(&$save_manager);
        $ui.$callback(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = val.clamp($min, $max);
            match settings.lock() {
                Ok(mut s) => {
                    let $anim = anim_id.get_mut(&mut s.animations.per_animation);
                    $field = clamped;
                    debug!("{} {}: {}", anim_id.name(), $msg, clamped);
                    save_manager.mark_dirty(SettingsCategory::Animations);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }};
}

/// Register an indexed animation enum/index field callback
///
/// Handles callbacks that take (id, index) and update an enum field using from_index.
/// Takes pre-cloned references to avoid additional Arc clones.
macro_rules! register_anim_enum_callback {
    ($ui:expr, $callback:ident, $settings:expr, $save_manager:expr,
     |$anim:ident| $field:expr, $enum_type:ty, $msg:expr) => {{
        let settings = Arc::clone(&$settings);
        let save_manager = Rc::clone(&$save_manager);
        $ui.$callback(move |id, idx| {
            let anim_id = AnimationId::from_index(id);
            match settings.lock() {
                Ok(mut s) => {
                    let $anim = anim_id.get_mut(&mut s.animations.per_animation);
                    $field = <$enum_type>::from_index(idx);
                    debug!("{} {}: {:?}", anim_id.name(), $msg, $field);
                    save_manager.mark_dirty(SettingsCategory::Animations);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }};
}

/// Set up animation-related callbacks
///
/// Uses 6 indexed callbacks instead of 66 individual callbacks.
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Clone once for all callbacks in this module
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Animations toggle
    register_bool_callback!(
        ui,
        on_animations_toggled,
        s,
        sm,
        SettingsCategory::Animations,
        |s| s.animations.enabled,
        "Animations enabled"
    );

    // Animation slowdown
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        ui.on_animations_slowdown_changed(move |slowdown| {
            let clamped = (slowdown as f64).clamp(ANIMATION_SLOWDOWN_MIN, ANIMATION_SLOWDOWN_MAX);
            match settings.lock() {
                Ok(mut s) => {
                    s.animations.slowdown = clamped;
                    debug!("Animation slowdown: {:.2}x", clamped);
                    save_manager.mark_dirty(SettingsCategory::Animations);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // =========================================================================
    // Indexed animation callbacks (6 total instead of 66)
    // Uses AnimationId enum for type safety instead of magic constants
    // =========================================================================

    // Animation type changed (enum)
    register_anim_enum_callback!(
        ui,
        on_animation_type_changed,
        s,
        sm,
        |anim| anim.animation_type,
        AnimationType,
        "type"
    );

    // Animation curve changed (enum)
    register_anim_enum_callback!(
        ui,
        on_animation_curve_changed,
        s,
        sm,
        |anim| anim.easing.curve,
        EasingCurve,
        "curve"
    );

    // Animation damping changed (f64)
    register_anim_clamped_callback!(
        ui,
        on_animation_damping_changed,
        s,
        sm,
        DAMPING_RATIO_MIN,
        DAMPING_RATIO_MAX,
        |anim| anim.spring.damping_ratio,
        "damping",
        f64
    );

    // Animation epsilon changed (f64)
    register_anim_clamped_callback!(
        ui,
        on_animation_epsilon_changed,
        s,
        sm,
        EPSILON_MIN,
        EPSILON_MAX,
        |anim| anim.spring.epsilon,
        "epsilon",
        f64
    );

    // Animation stiffness changed (i32)
    register_anim_clamped_callback!(
        ui,
        on_animation_stiffness_changed,
        s,
        sm,
        STIFFNESS_MIN,
        STIFFNESS_MAX,
        |anim| anim.spring.stiffness,
        "stiffness",
        i32
    );

    // Animation duration changed (i32)
    register_anim_clamped_callback!(
        ui,
        on_animation_duration_changed,
        s,
        sm,
        EASING_DURATION_MIN,
        EASING_DURATION_MAX,
        |anim| anim.easing.duration_ms,
        "duration",
        i32
    );

    // =========================================================================
    // Cubic-bezier control point callbacks
    // =========================================================================

    // Bezier X1 changed
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        ui.on_animation_bezier_x1_changed(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = (val as f64).clamp(0.0, 1.0);
            match settings.lock() {
                Ok(mut s) => {
                    let anim = anim_id.get_mut(&mut s.animations.per_animation);
                    if let EasingCurve::CubicBezier { x1, .. } = &mut anim.easing.curve {
                        *x1 = clamped;
                        debug!("{} bezier x1: {}", anim_id.name(), clamped);
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Bezier Y1 changed
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        ui.on_animation_bezier_y1_changed(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = (val as f64).clamp(-1.0, 2.0);
            match settings.lock() {
                Ok(mut s) => {
                    let anim = anim_id.get_mut(&mut s.animations.per_animation);
                    if let EasingCurve::CubicBezier { y1, .. } = &mut anim.easing.curve {
                        *y1 = clamped;
                        debug!("{} bezier y1: {}", anim_id.name(), clamped);
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Bezier X2 changed
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        ui.on_animation_bezier_x2_changed(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = (val as f64).clamp(0.0, 1.0);
            match settings.lock() {
                Ok(mut s) => {
                    let anim = anim_id.get_mut(&mut s.animations.per_animation);
                    if let EasingCurve::CubicBezier { x2, .. } = &mut anim.easing.curve {
                        *x2 = clamped;
                        debug!("{} bezier x2: {}", anim_id.name(), clamped);
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Bezier Y2 changed
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        ui.on_animation_bezier_y2_changed(move |id, val| {
            let anim_id = AnimationId::from_index(id);
            let clamped = (val as f64).clamp(-1.0, 2.0);
            match settings.lock() {
                Ok(mut s) => {
                    let anim = anim_id.get_mut(&mut s.animations.per_animation);
                    if let EasingCurve::CubicBezier { y2, .. } = &mut anim.easing.curve {
                        *y2 = clamped;
                        debug!("{} bezier y2: {}", anim_id.name(), clamped);
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }
}
