//! Dynamic animations UI callbacks
//!
//! Handles animation configuration using model-driven dynamic UI.
//! Uses generic callbacks that dispatch based on setting ID.
//!
//! # Integration Guide
//!
//! To integrate this module, add the following to main.slint:
//!
//! 1. Import the page and struct:
//! ```slint
//! import { AnimationsDynamicPage, AnimationSettingModel } from "pages/animations_dynamic.slint";
//! ```
//!
//! 2. Add properties for models:
//! ```slint
//! in-out property <[AnimationSettingModel]> anim-global-settings: [];
//! in-out property <bool> anim-animations-enabled: true;
//! in-out property <[AnimationSettingModel]> anim-workspace-nav-animations: [];
//! in-out property <[AnimationSettingModel]> anim-window-animations: [];
//! in-out property <[AnimationSettingModel]> anim-ui-animations: [];
//! // Spring params for animations 0-10
//! in-out property <[AnimationSettingModel]> anim-spring-params-0: [];
//! // ... through anim-spring-params-10
//! // Easing params for animations 0-10
//! in-out property <[AnimationSettingModel]> anim-easing-params-0: [];
//! // ... through anim-easing-params-10
//! ```
//!
//! 3. Add callbacks:
//! ```slint
//! callback anim-setting-toggle-changed(string, bool);
//! callback anim-setting-slider-int-changed(string, int);
//! callback anim-setting-slider-float-changed(string, float);
//! callback anim-setting-combo-changed(string, int);
//! ```
//!
//! 4. Wire up the page in the content area:
//! ```slint
//! AnimationsDynamicPage {
//!     global-settings <=> root.anim-global-settings;
//!     animations-enabled <=> root.anim-animations-enabled;
//!     workspace-nav-animations <=> root.anim-workspace-nav-animations;
//!     // ... etc
//!     setting-toggle-changed(id, val) => { root.anim-setting-toggle-changed(id, val); }
//!     // ... etc
//! }
//! ```
//!
//! 5. Add to mod.rs:
//! ```rust,ignore
//! pub mod animations_dynamic;
//! ```
//!
//! 6. Call setup() in bridge/mod.rs setup_callbacks()
//!
//! 7. Call sync_all_animation_models() in sync.rs
//!
//! Property naming convention (Slint kebab-case -> Rust snake_case):
//! - `anim-global-settings` -> `anim_global_settings`
//! - `anim-setting-toggle-changed` -> `on_anim_setting_toggle_changed`

use crate::config::models::{
    AnimationId, AnimationSettings, AnimationType, EasingCurve, PerAnimationSettings,
    SingleAnimationConfig,
};
use crate::config::{Settings, SettingsCategory};
use crate::constants::{
    ANIMATION_SLOWDOWN_MAX, ANIMATION_SLOWDOWN_MIN, DAMPING_RATIO_MAX, DAMPING_RATIO_MIN,
    EASING_DURATION_MAX, EASING_DURATION_MIN, EPSILON_MAX, EPSILON_MIN, STIFFNESS_MAX,
    STIFFNESS_MIN,
};
use crate::{AnimationSettingModel, MainWindow};
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

fn make_toggle(
    id: &str,
    label: &str,
    desc: &str,
    value: bool,
    visible: bool,
) -> AnimationSettingModel {
    AnimationSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

fn make_slider_int(
    id: &str,
    label: &str,
    desc: &str,
    value: i32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> AnimationSettingModel {
    AnimationSettingModel {
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

fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> AnimationSettingModel {
    AnimationSettingModel {
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

fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> AnimationSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    AnimationSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(opts)),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// MODEL POPULATION FUNCTIONS
// ============================================================================

/// Animation type options for combo box
const ANIMATION_TYPES: &[&str] = &["Default", "Off", "Spring", "Easing"];

/// Easing curve options for combo box
const EASING_CURVES: &[&str] = &[
    "Ease Out Quad",
    "Ease Out Cubic",
    "Ease Out Expo",
    "Linear",
    "Custom (Cubic Bezier)",
];

/// Populate global animation settings model
fn populate_global_settings(settings: &AnimationSettings) -> ModelRc<AnimationSettingModel> {
    let items = vec![
        make_toggle(
            "animations_enabled",
            "Enable animations",
            "Animate window movements, workspace switches, and other transitions",
            settings.enabled,
            true,
        ),
        make_slider_float(
            "slowdown",
            "Animation speed",
            "Slowdown factor (1.0 = normal, higher = slower)",
            settings.slowdown as f32,
            ANIMATION_SLOWDOWN_MIN as f32,
            ANIMATION_SLOWDOWN_MAX as f32,
            "x",
            settings.enabled,
        ),
    ];
    ModelRc::new(VecModel::from(items))
}

/// Animation info for creating models
struct AnimationInfo {
    id: AnimationId,
    name: &'static str,
    description: &'static str,
}

/// All animations organized by group
const WORKSPACE_NAV_ANIMATIONS: &[AnimationInfo] = &[
    AnimationInfo {
        id: AnimationId::WorkspaceSwitch,
        name: "Workspace Switch",
        description: "Switching between workspaces",
    },
    AnimationInfo {
        id: AnimationId::Overview,
        name: "Overview",
        description: "Opening/closing the workspace overview",
    },
];

const WINDOW_ANIMATIONS: &[AnimationInfo] = &[
    AnimationInfo {
        id: AnimationId::WindowOpen,
        name: "Window Open",
        description: "New window appearing",
    },
    AnimationInfo {
        id: AnimationId::WindowClose,
        name: "Window Close",
        description: "Window closing",
    },
    AnimationInfo {
        id: AnimationId::WindowMovement,
        name: "Window Movement",
        description: "Moving windows between columns",
    },
    AnimationInfo {
        id: AnimationId::WindowResize,
        name: "Window Resize",
        description: "Resizing windows",
    },
    AnimationInfo {
        id: AnimationId::HorizontalViewMovement,
        name: "Horizontal View",
        description: "Scrolling through columns",
    },
];

const UI_ANIMATIONS: &[AnimationInfo] = &[
    AnimationInfo {
        id: AnimationId::ConfigNotification,
        name: "Config Notification",
        description: "Config reload notification popup",
    },
    AnimationInfo {
        id: AnimationId::ExitConfirmation,
        name: "Exit Confirmation",
        description: "Exit confirmation dialog",
    },
    AnimationInfo {
        id: AnimationId::ScreenshotUi,
        name: "Screenshot UI",
        description: "Screenshot interface opening",
    },
    AnimationInfo {
        id: AnimationId::RecentWindows,
        name: "Recent Windows",
        description: "Recent windows switcher closing",
    },
];

/// Create animation type selector model for a single animation
fn create_animation_type_model(
    anim_id: AnimationId,
    config: &SingleAnimationConfig,
) -> AnimationSettingModel {
    let id_str = format!("anim_type_{}", anim_id.to_index());
    let info = get_animation_info(anim_id);

    make_combo(
        &id_str,
        info.name,
        info.description,
        config.animation_type.to_index(),
        ANIMATION_TYPES,
        true,
    )
}

/// Get animation info by ID
fn get_animation_info(id: AnimationId) -> &'static AnimationInfo {
    // Search through all animation groups
    for info in WORKSPACE_NAV_ANIMATIONS {
        if info.id == id {
            return info;
        }
    }
    for info in WINDOW_ANIMATIONS {
        if info.id == id {
            return info;
        }
    }
    for info in UI_ANIMATIONS {
        if info.id == id {
            return info;
        }
    }
    // Fallback (should never happen)
    &WORKSPACE_NAV_ANIMATIONS[0]
}

/// Create spring parameters model for a single animation
fn create_spring_params_model(
    anim_id: AnimationId,
    config: &SingleAnimationConfig,
) -> ModelRc<AnimationSettingModel> {
    let prefix = format!("spring_{}_{}", anim_id.to_index(), "");

    let items = vec![
        make_slider_float(
            &format!("{}damping", prefix),
            "Damping ratio",
            "1.0 = smooth stop, <1.0 = bouncy",
            config.spring.damping_ratio as f32,
            DAMPING_RATIO_MIN as f32,
            DAMPING_RATIO_MAX as f32,
            "",
            true,
        ),
        make_slider_int(
            &format!("{}stiffness", prefix),
            "Stiffness",
            "Higher = faster/stiffer",
            config.spring.stiffness,
            STIFFNESS_MIN as f32,
            STIFFNESS_MAX as f32,
            "",
            true,
        ),
        make_slider_float(
            &format!("{}epsilon", prefix),
            "Epsilon",
            "Animation end threshold",
            config.spring.epsilon as f32,
            EPSILON_MIN as f32,
            EPSILON_MAX as f32,
            "",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(items))
}

/// Create easing parameters model for a single animation
fn create_easing_params_model(
    anim_id: AnimationId,
    config: &SingleAnimationConfig,
) -> ModelRc<AnimationSettingModel> {
    let prefix = format!("easing_{}_{}", anim_id.to_index(), "");
    let curve_index = config.easing.curve.to_index();
    let is_bezier = curve_index == 4;

    // Get bezier points if applicable
    let (x1, y1, x2, y2) = config
        .easing
        .curve
        .bezier_points()
        .unwrap_or((0.25, 0.1, 0.25, 1.0));

    let mut items = vec![
        make_slider_int(
            &format!("{}duration", prefix),
            "Duration",
            "Animation duration in milliseconds",
            config.easing.duration_ms,
            EASING_DURATION_MIN as f32,
            EASING_DURATION_MAX as f32,
            "ms",
            true,
        ),
        make_combo(
            &format!("{}curve", prefix),
            "Easing curve",
            "Type of easing function",
            curve_index,
            EASING_CURVES,
            true,
        ),
    ];

    // Add bezier control points if custom curve is selected
    if is_bezier {
        items.push(make_slider_float(
            &format!("{}bezier_x1", prefix),
            "X1",
            "First control point X (0-1)",
            x1 as f32,
            0.0,
            1.0,
            "",
            true,
        ));
        items.push(make_slider_float(
            &format!("{}bezier_y1", prefix),
            "Y1",
            "First control point Y (-1 to 2)",
            y1 as f32,
            -1.0,
            2.0,
            "",
            true,
        ));
        items.push(make_slider_float(
            &format!("{}bezier_x2", prefix),
            "X2",
            "Second control point X (0-1)",
            x2 as f32,
            0.0,
            1.0,
            "",
            true,
        ));
        items.push(make_slider_float(
            &format!("{}bezier_y2", prefix),
            "Y2",
            "Second control point Y (-1 to 2)",
            y2 as f32,
            -1.0,
            2.0,
            "",
            true,
        ));
    }

    ModelRc::new(VecModel::from(items))
}

/// Populate animation group models
fn populate_animation_group(
    animations: &[AnimationInfo],
    per_anim: &PerAnimationSettings,
) -> ModelRc<AnimationSettingModel> {
    let items: Vec<AnimationSettingModel> = animations
        .iter()
        .map(|info| {
            let config = info.id.get(per_anim);
            create_animation_type_model(info.id, config)
        })
        .collect();
    ModelRc::new(VecModel::from(items))
}

// ============================================================================
// UI SYNC FUNCTION
// ============================================================================

/// Sync all animation models to the UI
fn sync_animation_models(ui: &MainWindow, settings: &AnimationSettings) {
    // Global settings
    ui.set_anim_global_settings(populate_global_settings(settings));
    ui.set_anim_animations_enabled(settings.enabled);

    // Per-animation type selectors by group
    ui.set_anim_workspace_nav_animations(populate_animation_group(
        WORKSPACE_NAV_ANIMATIONS,
        &settings.per_animation,
    ));
    ui.set_anim_window_animations(populate_animation_group(
        WINDOW_ANIMATIONS,
        &settings.per_animation,
    ));
    ui.set_anim_ui_animations(populate_animation_group(
        UI_ANIMATIONS,
        &settings.per_animation,
    ));

    // Spring and easing parameters for each animation (0-10)
    let all_ids = [
        AnimationId::WorkspaceSwitch,
        AnimationId::Overview,
        AnimationId::WindowOpen,
        AnimationId::WindowClose,
        AnimationId::WindowMovement,
        AnimationId::WindowResize,
        AnimationId::HorizontalViewMovement,
        AnimationId::ConfigNotification,
        AnimationId::ExitConfirmation,
        AnimationId::ScreenshotUi,
        AnimationId::RecentWindows,
    ];

    for (i, anim_id) in all_ids.iter().enumerate() {
        let config = anim_id.get(&settings.per_animation);
        let spring_model = create_spring_params_model(*anim_id, config);
        let easing_model = create_easing_params_model(*anim_id, config);

        // Set the appropriate property based on index
        match i {
            0 => {
                ui.set_anim_spring_params_0(spring_model);
                ui.set_anim_easing_params_0(easing_model);
            }
            1 => {
                ui.set_anim_spring_params_1(spring_model);
                ui.set_anim_easing_params_1(easing_model);
            }
            2 => {
                ui.set_anim_spring_params_2(spring_model);
                ui.set_anim_easing_params_2(easing_model);
            }
            3 => {
                ui.set_anim_spring_params_3(spring_model);
                ui.set_anim_easing_params_3(easing_model);
            }
            4 => {
                ui.set_anim_spring_params_4(spring_model);
                ui.set_anim_easing_params_4(easing_model);
            }
            5 => {
                ui.set_anim_spring_params_5(spring_model);
                ui.set_anim_easing_params_5(easing_model);
            }
            6 => {
                ui.set_anim_spring_params_6(spring_model);
                ui.set_anim_easing_params_6(easing_model);
            }
            7 => {
                ui.set_anim_spring_params_7(spring_model);
                ui.set_anim_easing_params_7(easing_model);
            }
            8 => {
                ui.set_anim_spring_params_8(spring_model);
                ui.set_anim_easing_params_8(easing_model);
            }
            9 => {
                ui.set_anim_spring_params_9(spring_model);
                ui.set_anim_easing_params_9(easing_model);
            }
            10 => {
                ui.set_anim_spring_params_10(spring_model);
                ui.set_anim_easing_params_10(easing_model);
            }
            _ => {}
        }
    }
}

// ============================================================================
// CALLBACK HANDLERS
// ============================================================================

/// Parse animation ID from setting ID string (e.g., "anim_type_0" -> Some(0))
fn parse_anim_index(id: &str) -> Option<i32> {
    // Handle anim_type_N format
    if let Some(suffix) = id.strip_prefix("anim_type_") {
        return suffix.parse().ok();
    }
    // Handle spring_N_ or easing_N_ format
    let parts: Vec<&str> = id.split('_').collect();
    if parts.len() >= 2 {
        if parts[0] == "spring" || parts[0] == "easing" {
            return parts[1].parse().ok();
        }
    }
    None
}

/// Handle toggle setting changes
/// Returns (changed, needs_ui_refresh)
fn handle_toggle(id: &str, value: bool, settings: &mut Settings) -> (bool, bool) {
    match id {
        "animations_enabled" => {
            settings.animations.enabled = value;
            debug!("Animations enabled: {}", value);
            (true, true) // needs UI refresh to show/hide slowdown slider
        }
        _ => {
            debug!("Unknown toggle setting: {}", id);
            (false, false)
        }
    }
}

/// Handle float slider setting changes
fn handle_slider_float(
    id: &str,
    value: f32,
    settings: &mut Settings,
    _ui_weak: &slint::Weak<MainWindow>,
) -> bool {
    let id_str = id;

    // Global slowdown
    if id_str == "slowdown" {
        let clamped = (value as f64).clamp(ANIMATION_SLOWDOWN_MIN, ANIMATION_SLOWDOWN_MAX);
        settings.animations.slowdown = clamped;
        debug!("Animation slowdown: {:.2}x", clamped);
        return true;
    }

    // Per-animation spring/easing parameters
    if let Some(anim_index) = parse_anim_index(id_str) {
        let anim_id = AnimationId::from_index(anim_index);
        let anim = anim_id.get_mut(&mut settings.animations.per_animation);

        // Spring parameters
        if id_str.contains("_damping") {
            let clamped = (value as f64).clamp(DAMPING_RATIO_MIN, DAMPING_RATIO_MAX);
            anim.spring.damping_ratio = clamped;
            debug!("{} damping: {:.2}", anim_id.name(), clamped);
            return true;
        }
        if id_str.contains("_epsilon") {
            let clamped = (value as f64).clamp(EPSILON_MIN, EPSILON_MAX);
            anim.spring.epsilon = clamped;
            debug!("{} epsilon: {:.6}", anim_id.name(), clamped);
            return true;
        }

        // Bezier control points
        if id_str.contains("_bezier_x1") {
            let clamped = (value as f64).clamp(0.0, 1.0);
            if let EasingCurve::CubicBezier { x1, .. } = &mut anim.easing.curve {
                *x1 = clamped;
                debug!("{} bezier x1: {:.2}", anim_id.name(), clamped);
                return true;
            }
        }
        if id_str.contains("_bezier_y1") {
            let clamped = (value as f64).clamp(-1.0, 2.0);
            if let EasingCurve::CubicBezier { y1, .. } = &mut anim.easing.curve {
                *y1 = clamped;
                debug!("{} bezier y1: {:.2}", anim_id.name(), clamped);
                return true;
            }
        }
        if id_str.contains("_bezier_x2") {
            let clamped = (value as f64).clamp(0.0, 1.0);
            if let EasingCurve::CubicBezier { x2, .. } = &mut anim.easing.curve {
                *x2 = clamped;
                debug!("{} bezier x2: {:.2}", anim_id.name(), clamped);
                return true;
            }
        }
        if id_str.contains("_bezier_y2") {
            let clamped = (value as f64).clamp(-1.0, 2.0);
            if let EasingCurve::CubicBezier { y2, .. } = &mut anim.easing.curve {
                *y2 = clamped;
                debug!("{} bezier y2: {:.2}", anim_id.name(), clamped);
                return true;
            }
        }
    }

    debug!("Unknown float slider setting: {}", id_str);
    false
}

/// Handle int slider setting changes
fn handle_slider_int(
    id: &str,
    value: i32,
    settings: &mut Settings,
    _ui_weak: &slint::Weak<MainWindow>,
) -> bool {
    let id_str = id;

    // Per-animation spring/easing parameters
    if let Some(anim_index) = parse_anim_index(id_str) {
        let anim_id = AnimationId::from_index(anim_index);
        let anim = anim_id.get_mut(&mut settings.animations.per_animation);

        // Spring stiffness
        if id_str.contains("_stiffness") {
            let clamped = value.clamp(STIFFNESS_MIN, STIFFNESS_MAX);
            anim.spring.stiffness = clamped;
            debug!("{} stiffness: {}", anim_id.name(), clamped);
            return true;
        }

        // Easing duration
        if id_str.contains("_duration") {
            let clamped = value.clamp(EASING_DURATION_MIN, EASING_DURATION_MAX);
            anim.easing.duration_ms = clamped;
            debug!("{} duration: {}ms", anim_id.name(), clamped);
            return true;
        }
    }

    debug!("Unknown int slider setting: {}", id_str);
    false
}

/// Handle combo box setting changes
/// Returns (changed, needs_ui_refresh)
fn handle_combo(id: &str, index: i32, settings: &mut Settings) -> (bool, bool) {
    let id_str = id;

    // Animation type selector
    if id_str.starts_with("anim_type_") {
        if let Some(anim_index) = parse_anim_index(id_str) {
            let anim_id = AnimationId::from_index(anim_index);
            let anim = anim_id.get_mut(&mut settings.animations.per_animation);
            anim.animation_type = AnimationType::from_index(index);
            debug!("{} type: {:?}", anim_id.name(), anim.animation_type);
            return (true, true); // needs UI refresh to show/hide spring/easing panels
        }
    }

    // Easing curve selector
    if id_str.contains("_curve") {
        if let Some(anim_index) = parse_anim_index(id_str) {
            let anim_id = AnimationId::from_index(anim_index);
            let anim = anim_id.get_mut(&mut settings.animations.per_animation);
            anim.easing.curve = EasingCurve::from_index(index);
            debug!("{} curve: {:?}", anim_id.name(), anim.easing.curve);
            return (true, true); // needs UI refresh to show/hide bezier controls
        }
    }

    debug!("Unknown combo setting: {}", id_str);
    (false, false)
}

// ============================================================================
// SETUP FUNCTION
// ============================================================================

/// Set up dynamic animation callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Toggle callback
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        let ui_weak = ui.as_weak();
        ui.on_anim_setting_toggle_changed(move |id, value| {
            let id_str = id.as_str();
            let result = match settings.lock() {
                Ok(mut s) => {
                    let (changed, needs_refresh) = handle_toggle(&id_str, value, &mut s);
                    if changed {
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                    if needs_refresh {
                        Some(s.animations.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    None
                }
            };

            // UI updates happen after lock is released
            if let Some(anim_settings) = result {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_animation_models(&ui, &anim_settings);
                }
            }
        });
    }

    // Float slider callback
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        let ui_weak = ui.as_weak();
        ui.on_anim_setting_slider_float_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    if handle_slider_float(&id_str, value, &mut s, &ui_weak) {
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Int slider callback
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        let ui_weak = ui.as_weak();
        ui.on_anim_setting_slider_int_changed(move |id, value| {
            let id_str = id.as_str();
            match settings.lock() {
                Ok(mut s) => {
                    if handle_slider_int(&id_str, value, &mut s, &ui_weak) {
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Combo callback
    {
        let settings = Arc::clone(&s);
        let save_manager = Rc::clone(&sm);
        let ui_weak = ui.as_weak();
        ui.on_anim_setting_combo_changed(move |id, index| {
            let id_str = id.as_str();
            let result = match settings.lock() {
                Ok(mut s) => {
                    let (changed, needs_refresh) = handle_combo(&id_str, index, &mut s);
                    if changed {
                        save_manager.mark_dirty(SettingsCategory::Animations);
                        save_manager.request_save();
                    }
                    if needs_refresh {
                        Some(s.animations.clone())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    error!("Settings lock error: {}", e);
                    None
                }
            };

            // UI updates happen after lock is released
            if let Some(anim_settings) = result {
                if let Some(ui) = ui_weak.upgrade() {
                    sync_animation_models(&ui, &anim_settings);
                }
            }
        });
    }
}

// ============================================================================
// PUBLIC SYNC FUNCTION
// ============================================================================

/// Public function to sync all animation models for sync.rs
pub fn sync_all_animation_models(ui: &MainWindow, settings: &AnimationSettings) {
    sync_animation_models(ui, settings);
}
