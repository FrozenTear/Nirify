//! UI synchronization from settings
//!
//! This module handles populating the UI with values from the Settings struct.

use crate::config::Settings;
use crate::ipc;
use crate::MainWindow;
use log::debug;

use super::sync_macros::{
    sync_bool_props, sync_color_hex_props, sync_color_or_gradient_hex_props,
    sync_color_or_gradient_props, sync_color_props, sync_enum_index_props, sync_f32_props,
    sync_f64_as_f32_props, sync_i32_props, sync_string_props,
};

use super::callbacks::layer_rules::{
    build_matches_model as build_layer_rule_matches_model,
    build_rules_list_model as build_layer_rules_list_model,
};
use super::callbacks::outputs::{format_mode_for_display, format_mode_for_storage};
use super::callbacks::window_rules::{
    build_matches_model, build_rules_list_model, get_open_behavior_index,
};
use super::callbacks::workspaces::build_workspaces_list_model;
use super::converters::{color_to_slint_color, key_parts_to_model, parse_key_combo_parts};
use super::indices::{
    center_focused_is_enabled, warp_mouse_is_enabled, TRACK_LAYOUT_GLOBAL, TRACK_LAYOUT_WINDOW,
};

/// Sync UI state from settings
///
/// This function populates all UI properties with values from the Settings struct.
/// It should be called after loading settings and before showing the UI.
pub fn sync_ui_from_settings(ui: &MainWindow, settings: &Settings) {
    sync_appearance(ui, settings);
    sync_behavior(ui, settings);
    sync_cursor(ui, settings);
    sync_animations(ui, settings);
    sync_overview(ui, settings);
    sync_keyboard(ui, settings);
    sync_mouse(ui, settings);
    sync_touchpad(ui, settings);
    sync_outputs(ui, settings);
    sync_layout_extras(ui, settings);
    sync_gestures(ui, settings);
    sync_miscellaneous(ui, settings);
    sync_window_rules(ui, settings);
    sync_input_devices(ui, settings);
    sync_workspaces(ui, settings);
    sync_layer_rules(ui, settings);
    sync_keybindings(ui, settings);
    sync_startup(ui, settings);
    sync_environment(ui, settings);
    sync_debug(ui, settings);
    sync_switch_events(ui, settings);
    sync_recent_windows(ui, settings);
}

fn sync_appearance(ui: &MainWindow, settings: &Settings) {
    let a = &settings.appearance;

    sync_bool_props!(
        ui,
        a,
        [
            (focus_ring_enabled, set_focus_ring_enabled),
            (border_enabled, set_border_enabled),
        ]
    );

    sync_f32_props!(
        ui,
        a,
        [
            (focus_ring_width, set_focus_ring_width),
            (border_thickness, set_border_thickness),
            (gaps_inner, set_gaps_inner),
            (gaps_outer, set_gaps_outer),
            (corner_radius, set_corner_radius),
        ]
    );

    sync_color_or_gradient_props!(
        ui,
        a,
        [
            (focus_ring_active, set_focus_ring_active_color),
            (focus_ring_inactive, set_focus_ring_inactive_color),
            (border_active, set_border_active_color),
            (border_inactive, set_border_inactive_color),
        ]
    );

    sync_color_or_gradient_hex_props!(
        ui,
        a,
        [
            (focus_ring_active, set_focus_ring_active_hex),
            (focus_ring_inactive, set_focus_ring_inactive_hex),
            (border_active, set_border_active_hex),
            (border_inactive, set_border_inactive_hex),
        ]
    );

    // Urgent colors (ColorOrGradient type - use primary_color for UI display)
    // Note: UI currently only shows color, but the model supports gradients
    ui.set_focus_ring_urgent_color(color_to_slint_color(a.focus_ring_urgent.primary_color()));
    ui.set_border_urgent_color(color_to_slint_color(a.border_urgent.primary_color()));

    // Urgent color hex values
    ui.set_focus_ring_urgent_hex(a.focus_ring_urgent.to_hex().into());
    ui.set_border_urgent_hex(a.border_urgent.to_hex().into());

    // Background color (optional)
    if let Some(ref bg) = a.background_color {
        ui.set_background_color(crate::ui::bridge::converters::color_to_slint_color(bg));
        ui.set_background_color_hex(bg.to_hex().into());
    }
}

fn sync_behavior(ui: &MainWindow, settings: &Settings) {
    let b = &settings.behavior;

    sync_bool_props!(
        ui,
        b,
        [
            (focus_follows_mouse, set_focus_follows_mouse),
            (always_center_single_column, set_always_center_single_column),
            (disable_power_key_handling, set_disable_power_key_handling),
        ]
    );

    // Custom helper functions for warp/center modes
    ui.set_warp_mouse_to_focus(warp_mouse_is_enabled(&b.warp_mouse_to_focus));
    ui.set_center_focused_column(center_focused_is_enabled(&b.center_focused_column));

    // Enum indices using SlintIndex derive
    sync_enum_index_props!(
        ui,
        b,
        [
            (default_column_width_type, set_default_column_width_index),
            (mod_key, set_mod_key_index),
        ]
    );

    sync_f32_props!(
        ui,
        b,
        [
            (
                default_column_width_proportion,
                set_default_column_width_proportion
            ),
            (default_column_width_fixed, set_default_column_width_fixed),
            (strut_left, set_strut_left),
            (strut_right, set_strut_right),
            (strut_top, set_strut_top),
            (strut_bottom, set_strut_bottom),
        ]
    );

    // Optional modifier key
    ui.set_mod_key_nested_enabled(b.mod_key_nested.is_some());
    ui.set_mod_key_nested_index(b.mod_key_nested.map(|k| k.to_index()).unwrap_or(0));
}

fn sync_cursor(ui: &MainWindow, settings: &Settings) {
    let c = &settings.cursor;

    sync_string_props!(ui, c, [(theme, set_cursor_theme)]);
    sync_i32_props!(ui, c, [(size, set_cursor_size)]);
    sync_bool_props!(ui, c, [(hide_when_typing, set_hide_when_typing)]);

    // Optional hide-after-inactive (custom logic)
    ui.set_hide_after_inactive_enabled(c.hide_after_inactive_ms.is_some());
    ui.set_hide_after_inactive_ms(c.hide_after_inactive_ms.unwrap_or(1000));
}

fn sync_animations(ui: &MainWindow, settings: &Settings) {
    use crate::config::models::AnimationId;
    use slint::{ModelRc, SharedString, VecModel};

    // Global animation settings
    ui.set_animations_enabled(settings.animations.enabled);
    ui.set_animations_slowdown(settings.animations.slowdown as f32);

    // Build animation model using AnimationId enum for type safety
    let anims = &settings.animations.per_animation;

    // Animation descriptions (could move to AnimationId impl if needed elsewhere)
    const DESCRIPTIONS: &[&str] = &[
        "Animation when switching workspaces",
        "Animation for overview toggle",
        "Animation when windows appear",
        "Animation when windows close",
        "Animation when moving windows",
        "Animation when resizing windows",
        "Animation when panning the view",
        "Config reload notification animation",
        "Exit confirmation dialog animation",
        "Screenshot interface animation",
        "Recent windows panel close animation",
    ];

    // Build model using AnimationId enum - ensures IDs match callbacks
    let animation_configs: Vec<crate::AnimationConfig> = [
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
    ]
    .iter()
    .enumerate()
    .map(|(idx, anim_id)| {
        let anim = anim_id.get(anims);
        let (bezier_x1, bezier_y1, bezier_x2, bezier_y2) =
            anim.easing.curve.bezier_points().unwrap_or((0.25, 0.1, 0.25, 1.0));
        crate::AnimationConfig {
            id: anim_id.to_index(),
            name: SharedString::from(anim_id.name()),
            description: SharedString::from(DESCRIPTIONS[idx]),
            anim_type: anim.animation_type.to_index(),
            damping: anim.spring.damping_ratio as f32,
            stiffness: anim.spring.stiffness,
            epsilon: anim.spring.epsilon as f32,
            duration: anim.easing.duration_ms,
            curve: anim.easing.curve.to_index(),
            bezier_x1: bezier_x1 as f32,
            bezier_y1: bezier_y1 as f32,
            bezier_x2: bezier_x2 as f32,
            bezier_y2: bezier_y2 as f32,
        }
    })
    .collect();

    ui.set_animations_model(ModelRc::new(VecModel::from(animation_configs)));
}

fn sync_overview(ui: &MainWindow, settings: &Settings) {
    ui.set_overview_zoom(settings.overview.zoom as f32);
    ui.set_overview_backdrop_enabled(settings.overview.backdrop_color.is_some());
    if let Some(ref color) = settings.overview.backdrop_color {
        ui.set_overview_backdrop_color(color_to_slint_color(color));
        ui.set_overview_backdrop_hex(color.to_hex().into());
    }

    // Phase 8: Workspace shadow
    ui.set_overview_workspace_shadow_enabled(settings.overview.workspace_shadow.is_some());
    if let Some(ref ws) = settings.overview.workspace_shadow {
        ui.set_overview_workspace_shadow_softness(ws.softness as f32);
        ui.set_overview_workspace_shadow_spread(ws.spread as f32);
        ui.set_overview_workspace_shadow_offset_x(ws.offset_x as f32);
        ui.set_overview_workspace_shadow_offset_y(ws.offset_y as f32);
        ui.set_overview_workspace_shadow_color(color_to_slint_color(&ws.color));
        ui.set_overview_workspace_shadow_color_hex(ws.color.to_hex().into());
    }
}

fn sync_keyboard(ui: &MainWindow, settings: &Settings) {
    let k = &settings.keyboard;

    // Device enable/disable
    ui.set_keyboard_off(k.off);

    sync_string_props!(
        ui,
        k,
        [
            (xkb_layout, set_xkb_layout),
            (xkb_variant, set_xkb_variant),
            (xkb_model, set_xkb_model),
            (xkb_rules, set_xkb_rules),
            (xkb_options, set_xkb_options),
            (xkb_file, set_xkb_file),
        ]
    );

    sync_i32_props!(
        ui,
        k,
        [
            (repeat_delay, set_repeat_delay),
            (repeat_rate, set_repeat_rate),
        ]
    );

    ui.set_numlock(k.numlock);
    ui.set_track_layout_index(if k.track_layout == "window" {
        TRACK_LAYOUT_WINDOW
    } else {
        TRACK_LAYOUT_GLOBAL
    });
}

fn sync_mouse(ui: &MainWindow, settings: &Settings) {
    let m = &settings.mouse;

    // Device enable/disable
    ui.set_mouse_off(m.off);

    sync_bool_props!(
        ui,
        m,
        [
            (natural_scroll, set_mouse_natural_scroll),
            (left_handed, set_mouse_left_handed),
            (middle_emulation, set_mouse_middle_emulation),
            (scroll_button_lock, set_mouse_scroll_button_lock),
        ]
    );

    sync_f64_as_f32_props!(
        ui,
        m,
        [
            (accel_speed, set_mouse_accel_speed),
            (scroll_factor, set_mouse_scroll_factor),
        ]
    );

    sync_enum_index_props!(
        ui,
        m,
        [
            (accel_profile, set_mouse_accel_profile_index),
            (scroll_method, set_mouse_scroll_method_index),
        ]
    );

    // Scroll button - Option<i32> (0 means unset)
    ui.set_mouse_scroll_button(m.scroll_button.unwrap_or(0));
}

fn sync_touchpad(ui: &MainWindow, settings: &Settings) {
    let t = &settings.touchpad;

    // Device enable/disable
    ui.set_touchpad_off(t.off);

    sync_bool_props!(
        ui,
        t,
        [
            (tap, set_touchpad_tap),
            (natural_scroll, set_touchpad_natural_scroll),
            (middle_emulation, set_touchpad_middle_emulation),
            (left_handed, set_touchpad_left_handed),
            (dwt, set_touchpad_dwt),
            (dwtp, set_touchpad_dwtp),
            (drag, set_touchpad_drag),
            (drag_lock, set_touchpad_drag_lock),
            (
                disabled_on_external_mouse,
                set_touchpad_disabled_on_external_mouse
            ),
            (scroll_button_lock, set_touchpad_scroll_button_lock),
        ]
    );

    sync_f64_as_f32_props!(
        ui,
        t,
        [
            (scroll_factor, set_touchpad_scroll_factor),
            (accel_speed, set_touchpad_accel_speed),
        ]
    );

    sync_enum_index_props!(
        ui,
        t,
        [
            (tap_button_map, set_touchpad_tap_button_map_index),
            (scroll_method, set_touchpad_scroll_method_index),
            (click_method, set_touchpad_click_method_index),
            (accel_profile, set_touchpad_accel_profile_index),
        ]
    );

    // Scroll button - Option<i32> (0 means unset)
    ui.set_touchpad_scroll_button(t.scroll_button.unwrap_or(0));
}

fn sync_outputs(ui: &MainWindow, settings: &Settings) {
    // Build semicolon-separated list of output names
    let output_names: String = settings
        .outputs
        .outputs
        .iter()
        .map(|o| o.name.as_str())
        .collect::<Vec<_>>()
        .join(";");
    ui.set_output_names(output_names.into());

    // Build output list model for clickable list
    let output_list: Vec<slint::SharedString> = settings
        .outputs
        .outputs
        .iter()
        .map(|o| slint::SharedString::from(o.name.as_str()))
        .collect();
    ui.set_output_list(slint::ModelRc::new(slint::VecModel::from(output_list)));

    // If we have outputs, select the first one and load its settings
    if let Some(first_output) = settings.outputs.outputs.first() {
        ui.set_selected_output_index(0);
        ui.set_current_output_name(first_output.name.as_str().into());
        ui.set_current_output_enabled(first_output.enabled);
        ui.set_current_output_scale(first_output.scale as f32);
        ui.set_current_output_mode(first_output.mode.as_str().into());
        ui.set_current_output_pos_x(first_output.position_x);
        ui.set_current_output_pos_y(first_output.position_y);
        ui.set_current_output_transform_index(first_output.transform.to_index());
        ui.set_current_output_vrr_index(first_output.vrr.to_index());

        // Load available modes from niri IPC for the first output
        if let Ok(outputs) = ipc::get_full_outputs() {
            if let Some(output_info) = outputs.iter().find(|o| o.name == first_output.name) {
                let display_modes: Vec<slint::SharedString> = output_info
                    .modes
                    .iter()
                    .map(|m| slint::SharedString::from(format_mode_for_display(m, m.is_preferred)))
                    .collect();

                let storage_modes: Vec<String> = output_info
                    .modes
                    .iter()
                    .map(format_mode_for_storage)
                    .collect();

                ui.set_available_modes(slint::ModelRc::new(slint::VecModel::from(display_modes)));

                // Find selected mode index
                let selected_idx = if !first_output.mode.is_empty() {
                    storage_modes
                        .iter()
                        .position(|m| m == &first_output.mode)
                        .map(|i| i as i32)
                        .unwrap_or_else(|| output_info.current_mode.map(|i| i as i32).unwrap_or(-1))
                } else {
                    output_info.current_mode.map(|i| i as i32).unwrap_or(-1)
                };
                ui.set_selected_mode_index(selected_idx);

                debug!(
                    "Loaded {} modes for output {} on startup",
                    output_info.modes.len(),
                    first_output.name
                );
            }
        }
    } else {
        ui.set_selected_output_index(-1);
        ui.set_current_output_name("".into());
    }
}

fn sync_layout_extras(ui: &MainWindow, settings: &Settings) {
    // Shadow settings
    let s = &settings.layout_extras.shadow;
    sync_bool_props!(ui, s, [(enabled, set_shadow_enabled)]);
    sync_i32_props!(
        ui,
        s,
        [
            (softness, set_shadow_softness),
            (spread, set_shadow_spread),
            (offset_x, set_shadow_offset_x),
            (offset_y, set_shadow_offset_y),
        ]
    );
    ui.set_shadow_draw_behind(s.draw_behind_window);
    sync_color_props!(
        ui,
        s,
        [
            (color, set_shadow_color),
            (inactive_color, set_shadow_inactive_color),
        ]
    );
    sync_color_hex_props!(
        ui,
        s,
        [
            (color, set_shadow_color_hex),
            (inactive_color, set_shadow_inactive_color_hex),
        ]
    );

    // Tab indicator settings
    let ti = &settings.layout_extras.tab_indicator;
    sync_bool_props!(
        ui,
        ti,
        [
            (enabled, set_tab_indicator_enabled),
            (hide_when_single_tab, set_tab_indicator_hide_single),
            (place_within_column, set_tab_indicator_within_column),
        ]
    );
    sync_i32_props!(
        ui,
        ti,
        [
            (gap, set_tab_indicator_gap),
            (width, set_tab_indicator_width),
            (corner_radius, set_tab_indicator_corner_radius),
            (gaps_between_tabs, set_tab_indicator_gaps_between),
        ]
    );
    sync_enum_index_props!(ui, ti, [(position, set_tab_indicator_position_index)]);
    sync_color_or_gradient_props!(
        ui,
        ti,
        [
            (active, set_tab_indicator_active_color),
            (inactive, set_tab_indicator_inactive_color),
            (urgent, set_tab_indicator_urgent_color),
        ]
    );
    sync_color_or_gradient_hex_props!(
        ui,
        ti,
        [
            (active, set_tab_indicator_active_hex),
            (inactive, set_tab_indicator_inactive_hex),
            (urgent, set_tab_indicator_urgent_hex),
        ]
    );

    // Insert hint settings
    let ih = &settings.layout_extras.insert_hint;
    ui.set_insert_hint_enabled(ih.enabled);
    ui.set_insert_hint_color(color_to_slint_color(ih.color.primary_color()));
    ui.set_insert_hint_color_hex(ih.color.to_hex().into());
}

fn sync_gestures(ui: &MainWindow, settings: &Settings) {
    // Hot corners
    let hc = &settings.gestures.hot_corners;
    sync_bool_props!(
        ui,
        hc,
        [
            (enabled, set_hot_corners_enabled),
            (top_left, set_hot_corner_top_left),
            (top_right, set_hot_corner_top_right),
            (bottom_left, set_hot_corner_bottom_left),
            (bottom_right, set_hot_corner_bottom_right),
        ]
    );

    // DND edge scroll
    let es = &settings.gestures.dnd_edge_view_scroll;
    sync_bool_props!(ui, es, [(enabled, set_dnd_edge_scroll_enabled)]);
    sync_i32_props!(
        ui,
        es,
        [
            (trigger_size, set_dnd_edge_scroll_trigger_width),
            (delay_ms, set_dnd_edge_scroll_delay),
            (max_speed, set_dnd_edge_scroll_max_speed),
        ]
    );

    // DND workspace switch
    let ws = &settings.gestures.dnd_edge_workspace_switch;
    sync_bool_props!(ui, ws, [(enabled, set_dnd_workspace_switch_enabled)]);
    sync_i32_props!(
        ui,
        ws,
        [
            (trigger_size, set_dnd_workspace_switch_trigger_height),
            (delay_ms, set_dnd_workspace_switch_delay),
            (max_speed, set_dnd_workspace_switch_max_speed),
        ]
    );
}

fn sync_miscellaneous(ui: &MainWindow, settings: &Settings) {
    let m = &settings.miscellaneous;

    sync_bool_props!(
        ui,
        m,
        [
            (prefer_no_csd, set_prefer_no_csd),
            (disable_primary_clipboard, set_disable_primary_clipboard),
            (hotkey_overlay_skip_at_startup, set_hotkey_overlay_skip),
            (
                hotkey_overlay_hide_not_bound,
                set_hotkey_overlay_hide_not_bound
            ),
            (
                config_notification_disable_failed,
                set_config_notification_disable_failed
            ),
            (spawn_sh_at_startup, set_spawn_sh_at_startup),
        ]
    );
    sync_string_props!(ui, m, [(screenshot_path, set_screenshot_path)]);

    // XWayland satellite (custom logic)
    use crate::config::models::XWaylandSatelliteConfig;
    let (idx, path) = match &m.xwayland_satellite {
        XWaylandSatelliteConfig::Default => (0, String::new()),
        XWaylandSatelliteConfig::Off => (1, String::new()),
        XWaylandSatelliteConfig::CustomPath(p) => (2, p.clone()),
    };
    ui.set_xwayland_satellite_index(idx);
    ui.set_xwayland_satellite_path(path.as_str().into());
}

fn sync_window_rules(ui: &MainWindow, settings: &Settings) {
    // Build rule list model
    let rule_list = build_rules_list_model(&settings.window_rules.rules);
    ui.set_window_rules_list(rule_list);

    // If we have rules, select the first one and load its settings
    if let Some(first_rule) = settings.window_rules.rules.first() {
        ui.set_selected_window_rule_index(0);
        ui.set_current_rule_name(first_rule.name.as_str().into());

        // Multi-match support
        ui.set_current_matches_list(build_matches_model(&first_rule.matches));
        ui.set_selected_match_index(0);
        ui.set_current_matches_count(first_rule.matches.len() as i32);

        // Set current match properties (first match)
        if let Some(first_match) = first_rule.matches.first() {
            ui.set_current_match_app_id(first_match.app_id.as_deref().unwrap_or_default().into());
            ui.set_current_match_title(first_match.title.as_deref().unwrap_or_default().into());
        } else {
            ui.set_current_match_app_id("".into());
            ui.set_current_match_title("".into());
        }

        ui.set_current_open_behavior_index(get_open_behavior_index(first_rule.open_behavior));
        ui.set_current_has_opacity(first_rule.opacity.is_some());
        ui.set_current_opacity(first_rule.opacity.unwrap_or(1.0));
        ui.set_current_block_screencast(first_rule.block_out_from_screencast);
        ui.set_current_has_corner_radius(first_rule.corner_radius.is_some());
        ui.set_current_corner_radius(first_rule.corner_radius.unwrap_or(12));
        ui.set_current_open_on_output(
            first_rule
                .open_on_output
                .as_deref()
                .unwrap_or_default()
                .into(),
        );
        ui.set_current_open_on_workspace(
            first_rule
                .open_on_workspace
                .as_deref()
                .unwrap_or_default()
                .into(),
        );
    } else {
        ui.set_selected_window_rule_index(-1);
    }
}

fn sync_input_devices(ui: &MainWindow, settings: &Settings) {
    // Trackpoint
    let tp = &settings.trackpoint;
    sync_bool_props!(
        ui,
        tp,
        [
            (off, set_trackpoint_off),
            (natural_scroll, set_trackpoint_natural_scroll),
            (left_handed, set_trackpoint_left_handed),
            (scroll_button_lock, set_trackpoint_scroll_button_lock),
            (middle_emulation, set_trackpoint_middle_emulation),
        ]
    );
    sync_f64_as_f32_props!(ui, tp, [(accel_speed, set_trackpoint_accel_speed)]);
    sync_enum_index_props!(
        ui,
        tp,
        [
            (accel_profile, set_trackpoint_accel_profile_index),
            (scroll_method, set_trackpoint_scroll_method_index),
        ]
    );
    ui.set_trackpoint_scroll_button(tp.scroll_button.unwrap_or(0));

    // Trackball
    let tb = &settings.trackball;
    sync_bool_props!(
        ui,
        tb,
        [
            (off, set_trackball_off),
            (natural_scroll, set_trackball_natural_scroll),
            (left_handed, set_trackball_left_handed),
            (scroll_button_lock, set_trackball_scroll_button_lock),
            (middle_emulation, set_trackball_middle_emulation),
        ]
    );
    sync_f64_as_f32_props!(ui, tb, [(accel_speed, set_trackball_accel_speed)]);
    sync_enum_index_props!(
        ui,
        tb,
        [
            (accel_profile, set_trackball_accel_profile_index),
            (scroll_method, set_trackball_scroll_method_index),
        ]
    );
    ui.set_trackball_scroll_button(tb.scroll_button.unwrap_or(0));

    // Tablet
    let tab = &settings.tablet;
    sync_bool_props!(
        ui,
        tab,
        [(off, set_tablet_off), (left_handed, set_tablet_left_handed),]
    );
    sync_string_props!(ui, tab, [(map_to_output, set_tablet_map_to_output)]);
    ui.set_tablet_has_calibration(tab.calibration_matrix.is_some());
    if let Some(ref matrix) = tab.calibration_matrix {
        ui.set_tablet_cal_m0(matrix[0] as f32);
        ui.set_tablet_cal_m1(matrix[1] as f32);
        ui.set_tablet_cal_m2(matrix[2] as f32);
        ui.set_tablet_cal_m3(matrix[3] as f32);
        ui.set_tablet_cal_m4(matrix[4] as f32);
        ui.set_tablet_cal_m5(matrix[5] as f32);
    }

    // Touch
    let tch = &settings.touch;
    sync_bool_props!(ui, tch, [(off, set_touch_off)]);
    sync_string_props!(ui, tch, [(map_to_output, set_touch_map_to_output)]);
    ui.set_touch_has_calibration(tch.calibration_matrix.is_some());
    if let Some(ref matrix) = tch.calibration_matrix {
        ui.set_touch_cal_m0(matrix[0] as f32);
        ui.set_touch_cal_m1(matrix[1] as f32);
        ui.set_touch_cal_m2(matrix[2] as f32);
        ui.set_touch_cal_m3(matrix[3] as f32);
        ui.set_touch_cal_m4(matrix[4] as f32);
        ui.set_touch_cal_m5(matrix[5] as f32);
    }
}

fn sync_workspaces(ui: &MainWindow, settings: &Settings) {
    use crate::types::CenterFocusedColumn;

    // Build workspace list model
    let workspace_list = build_workspaces_list_model(&settings.workspaces.workspaces);
    ui.set_workspaces_list(workspace_list);

    // If we have workspaces, select the first one and load its settings
    if let Some(first_ws) = settings.workspaces.workspaces.first() {
        ui.set_selected_workspace_index(0);
        ui.set_current_workspace_name(first_ws.name.as_str().into());
        ui.set_current_workspace_open_on_output(
            first_ws
                .open_on_output
                .as_deref()
                .unwrap_or_default()
                .into(),
        );

        let has_override = first_ws.layout_override.is_some();
        ui.set_current_workspace_has_layout_override(has_override);

        if let Some(ref lo) = first_ws.layout_override {
            // Gaps
            let has_gaps = lo.gaps_inner.is_some() || lo.gaps_outer.is_some();
            ui.set_current_workspace_has_gaps_override(has_gaps);
            ui.set_current_workspace_gaps_inner(lo.gaps_inner.unwrap_or(16.0));
            ui.set_current_workspace_gaps_outer(lo.gaps_outer.unwrap_or(8.0));

            // Struts
            let has_struts = lo.strut_left.is_some()
                || lo.strut_right.is_some()
                || lo.strut_top.is_some()
                || lo.strut_bottom.is_some();
            ui.set_current_workspace_has_struts_override(has_struts);
            ui.set_current_workspace_strut_left(lo.strut_left.unwrap_or(0.0));
            ui.set_current_workspace_strut_right(lo.strut_right.unwrap_or(0.0));
            ui.set_current_workspace_strut_top(lo.strut_top.unwrap_or(0.0));
            ui.set_current_workspace_strut_bottom(lo.strut_bottom.unwrap_or(0.0));

            // Center
            let has_center =
                lo.center_focused_column.is_some() || lo.always_center_single_column.is_some();
            ui.set_current_workspace_has_center_override(has_center);
            let center_idx = lo
                .center_focused_column
                .unwrap_or(CenterFocusedColumn::Never)
                .to_index();
            ui.set_current_workspace_center_focused_column_index(center_idx);
            ui.set_current_workspace_always_center_single_column(
                lo.always_center_single_column.unwrap_or(false),
            );
        } else {
            ui.set_current_workspace_has_gaps_override(false);
            ui.set_current_workspace_has_struts_override(false);
            ui.set_current_workspace_has_center_override(false);
        }
    } else {
        ui.set_selected_workspace_index(-1);
    }
}

fn sync_layer_rules(ui: &MainWindow, settings: &Settings) {
    // Build rule list model
    let rule_list = build_layer_rules_list_model(&settings.layer_rules.rules);
    ui.set_layer_rules_list(rule_list);

    // If we have rules, select the first one and load its settings
    if let Some(first_rule) = settings.layer_rules.rules.first() {
        ui.set_selected_layer_rule_index(0);
        ui.set_current_layer_rule_name(first_rule.name.as_str().into());

        // Matches
        ui.set_current_layer_rule_matches_list(build_layer_rule_matches_model(&first_rule.matches));
        ui.set_selected_layer_rule_match_index(0);
        ui.set_current_layer_rule_matches_count(first_rule.matches.len() as i32);

        // Current match properties
        if let Some(first_match) = first_rule.matches.first() {
            ui.set_current_layer_rule_match_namespace(
                first_match.namespace.as_deref().unwrap_or_default().into(),
            );
            ui.set_current_layer_rule_match_has_at_startup(first_match.at_startup.is_some());
            ui.set_current_layer_rule_match_at_startup(first_match.at_startup.unwrap_or(true));
        }

        // Block out from - use SlintIndex derive
        ui.set_current_layer_rule_has_block_out_from(first_rule.block_out_from.is_some());
        ui.set_current_layer_rule_block_out_from_index(
            first_rule.block_out_from.map(|b| b.to_index()).unwrap_or(0),
        );

        // Opacity
        ui.set_current_layer_rule_has_opacity(first_rule.opacity.is_some());
        ui.set_current_layer_rule_opacity(first_rule.opacity.unwrap_or(1.0));

        // Corner radius
        ui.set_current_layer_rule_has_corner_radius(first_rule.geometry_corner_radius.is_some());
        ui.set_current_layer_rule_corner_radius(first_rule.geometry_corner_radius.unwrap_or(12));

        // Boolean flags
        ui.set_current_layer_rule_place_within_backdrop(first_rule.place_within_backdrop);
        ui.set_current_layer_rule_baba_is_float(first_rule.baba_is_float);

        // Shadow
        let has_shadow = first_rule.shadow.is_some();
        ui.set_current_layer_rule_has_shadow(has_shadow);
        if let Some(ref shadow) = first_rule.shadow {
            ui.set_current_layer_rule_shadow_enabled(shadow.enabled);
            ui.set_current_layer_rule_shadow_softness(shadow.softness);
            ui.set_current_layer_rule_shadow_spread(shadow.spread);
            ui.set_current_layer_rule_shadow_offset_x(shadow.offset_x);
            ui.set_current_layer_rule_shadow_offset_y(shadow.offset_y);
            ui.set_current_layer_rule_shadow_draw_behind(shadow.draw_behind_window);

            let color = slint::Color::from_argb_u8(
                shadow.color.a,
                shadow.color.r,
                shadow.color.g,
                shadow.color.b,
            );
            ui.set_current_layer_rule_shadow_color(color);
            ui.set_current_layer_rule_shadow_color_hex(shadow.color.to_hex().into());

            let inactive_color = slint::Color::from_argb_u8(
                shadow.inactive_color.a,
                shadow.inactive_color.r,
                shadow.inactive_color.g,
                shadow.inactive_color.b,
            );
            ui.set_current_layer_rule_shadow_inactive_color(inactive_color);
            ui.set_current_layer_rule_shadow_inactive_color_hex(
                shadow.inactive_color.to_hex().into(),
            );
        }
    } else {
        ui.set_selected_layer_rule_index(-1);
    }
}

/// Sync keybindings from settings to UI
pub fn sync_keybindings(ui: &MainWindow, settings: &Settings) {
    use crate::config::models::KeybindAction;
    use crate::KeybindingItem;
    use slint::ModelRc;
    use slint::VecModel;
    use std::rc::Rc;

    let keybindings = &settings.keybindings;

    // Build keybindings list model
    let items: Vec<KeybindingItem> = keybindings
        .bindings
        .iter()
        .map(|kb| {
            let (action_type, action_detail) = match &kb.action {
                KeybindAction::Spawn(args) => ("spawn".into(), args.join(" ").into()),
                KeybindAction::NiriAction(action) => ("niri".into(), action.clone().into()),
                KeybindAction::NiriActionWithArgs(action, args) => (
                    "niri-args".into(),
                    format!("{} {}", action, args.join(" ")).into(),
                ),
            };

            KeybindingItem {
                id: kb.id as i32,
                key_combo: kb.key_combo.clone().into(),
                key_parts: key_parts_to_model(parse_key_combo_parts(&kb.key_combo)),
                display_name: kb.display_name().into(),
                action_type,
                action_detail,
                allow_when_locked: kb.allow_when_locked,
                has_cooldown: kb.cooldown_ms.is_some(),
                cooldown_ms: kb.cooldown_ms.unwrap_or(0),
                repeat: kb.repeat,
            }
        })
        .collect();

    let model = Rc::new(VecModel::from(items));
    ui.set_keybindings_list(ModelRc::from(model));

    // Set metadata
    ui.set_keybindings_loaded(keybindings.loaded);
    ui.set_keybindings_source_file(keybindings.source_file.clone().unwrap_or_default().into());
    ui.set_keybindings_error(keybindings.error.clone().unwrap_or_default().into());

    // Reset selection
    ui.set_selected_keybinding_index(-1);
}

/// Sync startup commands from settings to UI
fn sync_startup(ui: &MainWindow, settings: &Settings) {
    use slint::{ModelRc, SharedString, VecModel};

    // Build command list
    let commands: Vec<SharedString> = settings
        .startup
        .commands
        .iter()
        .map(|cmd| SharedString::from(cmd.display()))
        .collect();

    ui.set_startup_command_list(ModelRc::new(VecModel::from(commands)));
    ui.set_selected_startup_index(-1);
    ui.set_current_startup_command(SharedString::default());
}

/// Sync environment variables from settings to UI
fn sync_environment(ui: &MainWindow, settings: &Settings) {
    use slint::{ModelRc, SharedString, VecModel};

    // Build variable list (display as NAME=value)
    let variables: Vec<SharedString> = settings
        .environment
        .variables
        .iter()
        .map(|var| SharedString::from(format!("{}={}", var.name, var.value)))
        .collect();

    ui.set_environment_variable_list(ModelRc::new(VecModel::from(variables)));
    ui.set_selected_environment_index(-1);
    ui.set_current_env_name(SharedString::default());
    ui.set_current_env_value(SharedString::default());
}

/// Sync debug settings from settings to UI
fn sync_debug(ui: &MainWindow, settings: &Settings) {
    use slint::SharedString;

    let d = &settings.debug;

    sync_bool_props!(
        ui,
        d,
        [
            (expert_mode, set_debug_expert_mode),
            (preview_render, set_debug_preview_render),
            (enable_overlay_planes, set_debug_enable_overlay_planes),
            (disable_cursor_plane, set_debug_disable_cursor_plane),
            (disable_direct_scanout, set_debug_disable_direct_scanout),
            (
                restrict_primary_scanout_to_matching_format,
                set_debug_restrict_primary_scanout_to_matching_format
            ),
            (
                disable_resize_throttling,
                set_debug_disable_resize_throttling
            ),
            (disable_transactions, set_debug_disable_transactions),
            (
                emulate_zero_presentation_time,
                set_debug_emulate_zero_presentation_time
            ),
            (
                skip_cursor_only_updates_during_vrr,
                set_debug_skip_cursor_only_updates_during_vrr
            ),
            (disable_monitor_names, set_debug_disable_monitor_names),
            (
                force_disable_connectors_on_resume,
                set_debug_force_disable_connectors_on_resume
            ),
            (
                honor_xdg_activation_with_invalid_serial,
                set_debug_honor_xdg_activation_with_invalid_serial
            ),
            (
                deactivate_unfocused_windows,
                set_debug_deactivate_unfocused_windows
            ),
            (
                force_pipewire_invalid_modifier,
                set_debug_force_pipewire_invalid_modifier
            ),
        ]
    );

    // Non-standard names (field doesn't match setter suffix)
    ui.set_debug_wait_for_frame_completion(d.wait_for_frame_completion_before_queueing);
    ui.set_debug_dbus_interfaces_in_non_session(d.dbus_interfaces_in_non_session_instances);
    ui.set_debug_keep_laptop_panel_on_lid_closed(d.keep_laptop_panel_on_when_lid_is_closed);
    ui.set_debug_strict_new_window_focus(d.strict_new_window_focus_policy);

    // Optional string
    ui.set_debug_render_drm_device(SharedString::from(
        d.render_drm_device.as_deref().unwrap_or(""),
    ));

    // Comma-separated list of devices
    ui.set_debug_ignore_drm_devices(SharedString::from(d.ignore_drm_devices.join(", ")));
}

/// Sync switch events settings from settings to UI
fn sync_switch_events(ui: &MainWindow, settings: &Settings) {
    use slint::{ModelRc, SharedString, VecModel};

    // Helper to convert Vec<String> to ModelRc<SharedString>
    fn to_model(commands: &[String]) -> ModelRc<SharedString> {
        ModelRc::new(VecModel::from(
            commands
                .iter()
                .map(|s| SharedString::from(s.as_str()))
                .collect::<Vec<_>>(),
        ))
    }

    // Lid close
    ui.set_switch_lid_close_commands(to_model(&settings.switch_events.lid_close.spawn));
    ui.set_switch_lid_close_selected(-1);
    ui.set_switch_lid_close_current(SharedString::default());

    // Lid open
    ui.set_switch_lid_open_commands(to_model(&settings.switch_events.lid_open.spawn));
    ui.set_switch_lid_open_selected(-1);
    ui.set_switch_lid_open_current(SharedString::default());

    // Tablet mode on
    ui.set_switch_tablet_mode_on_commands(to_model(&settings.switch_events.tablet_mode_on.spawn));
    ui.set_switch_tablet_mode_on_selected(-1);
    ui.set_switch_tablet_mode_on_current(SharedString::default());

    // Tablet mode off
    ui.set_switch_tablet_mode_off_commands(to_model(&settings.switch_events.tablet_mode_off.spawn));
    ui.set_switch_tablet_mode_off_selected(-1);
    ui.set_switch_tablet_mode_off_current(SharedString::default());
}

/// Sync recent windows settings from settings to UI (v25.05+)
fn sync_recent_windows(ui: &MainWindow, settings: &Settings) {
    let rw = &settings.recent_windows;

    // Enable/disable (inverted: off in settings = disabled in UI)
    ui.set_recent_windows_enabled(!rw.off);

    // Timing
    sync_i32_props!(
        ui,
        rw,
        [
            (debounce_ms, set_recent_windows_debounce_ms),
            (open_delay_ms, set_recent_windows_open_delay_ms),
        ]
    );

    // Highlight colors
    let hl = &rw.highlight;
    sync_color_props!(
        ui,
        hl,
        [
            (active_color, set_recent_windows_highlight_active_color),
            (urgent_color, set_recent_windows_highlight_urgent_color),
        ]
    );
    sync_color_hex_props!(
        ui,
        hl,
        [
            (active_color, set_recent_windows_highlight_active_color_hex),
            (urgent_color, set_recent_windows_highlight_urgent_color_hex),
        ]
    );

    // Highlight dimensions
    sync_i32_props!(
        ui,
        hl,
        [
            (padding, set_recent_windows_highlight_padding),
            (corner_radius, set_recent_windows_highlight_corner_radius),
        ]
    );

    // Previews
    let pv = &rw.previews;
    ui.set_recent_windows_previews_max_height(pv.max_height);
    ui.set_recent_windows_previews_max_scale(pv.max_scale as f32);
}
