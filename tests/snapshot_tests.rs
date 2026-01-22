//! Snapshot tests for KDL generation
//!
//! These tests capture the exact output of KDL generation and alert you
//! when it changes. This prevents accidental regressions in config output.
//!
//! Run with: cargo test --test snapshot_tests
//! Review changes with: cargo insta review
//! Accept changes with: cargo insta accept

use insta::assert_snapshot;
use niri_settings::config::models::*;
use niri_settings::config::storage::*;
use niri_settings::types::*;

// ============================================================================
// APPEARANCE SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_appearance_default() {
    let appearance = AppearanceSettings::default();
    let behavior = BehaviorSettings::default();
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_default", kdl);
}

#[test]
fn snapshot_appearance_custom_gaps() {
    let appearance = AppearanceSettings {
        gaps: 24.0,
        ..Default::default()
    };
    let behavior = BehaviorSettings::default();
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_custom_gaps", kdl);
}

#[test]
fn snapshot_appearance_focus_ring_disabled() {
    let appearance = AppearanceSettings {
        focus_ring_enabled: false,
        ..Default::default()
    };
    let behavior = BehaviorSettings::default();
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_focus_ring_disabled", kdl);
}

#[test]
fn snapshot_appearance_border_enabled() {
    let appearance = AppearanceSettings {
        border_enabled: true,
        border_thickness: 3.0,
        border_active: ColorOrGradient::Color(Color::from_hex("#ff5500").unwrap()),
        border_inactive: ColorOrGradient::Color(Color::from_hex("#333333").unwrap()),
        ..Default::default()
    };
    let behavior = BehaviorSettings::default();
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_border_enabled", kdl);
}

#[test]
fn snapshot_appearance_with_struts() {
    let appearance = AppearanceSettings::default();
    let behavior = BehaviorSettings {
        strut_left: 50.0,
        strut_right: 50.0,
        strut_top: 30.0,
        strut_bottom: 0.0,
        ..Default::default()
    };
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_with_struts", kdl);
}

#[test]
fn snapshot_appearance_corner_radius() {
    let appearance = AppearanceSettings {
        corner_radius: 16.0,
        ..Default::default()
    };
    let behavior = BehaviorSettings::default();
    let kdl = generate_appearance_kdl(&appearance, &behavior);
    assert_snapshot!("appearance_corner_radius", kdl);
}

// ============================================================================
// BEHAVIOR SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_behavior_default() {
    let behavior = BehaviorSettings::default();
    let kdl = generate_behavior_kdl(&behavior);
    assert_snapshot!("behavior_default", kdl);
}

#[test]
fn snapshot_behavior_focus_follows_mouse() {
    let behavior = BehaviorSettings {
        focus_follows_mouse: true,
        ..Default::default()
    };
    let kdl = generate_behavior_kdl(&behavior);
    assert_snapshot!("behavior_focus_follows_mouse", kdl);
}

#[test]
fn snapshot_behavior_warp_mouse() {
    let behavior = BehaviorSettings {
        warp_mouse_to_focus: WarpMouseMode::CenterXY,
        ..Default::default()
    };
    let kdl = generate_behavior_kdl(&behavior);
    assert_snapshot!("behavior_warp_mouse", kdl);
}

#[test]
fn snapshot_behavior_center_focused() {
    let behavior = BehaviorSettings {
        center_focused_column: CenterFocusedColumn::Always,
        ..Default::default()
    };
    let kdl = generate_behavior_kdl(&behavior);
    assert_snapshot!("behavior_center_focused", kdl);
}

// ============================================================================
// INPUT SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_keyboard_default() {
    let keyboard = KeyboardSettings::default();
    let kdl = generate_keyboard_kdl(&keyboard);
    assert_snapshot!("keyboard_default", kdl);
}

#[test]
fn snapshot_keyboard_custom_layout() {
    let keyboard = KeyboardSettings {
        xkb_layout: "de".to_string(),
        xkb_variant: "nodeadkeys".to_string(),
        xkb_options: "ctrl:nocaps".to_string(),
        repeat_delay: 400,
        repeat_rate: 30,
        ..Default::default()
    };
    let kdl = generate_keyboard_kdl(&keyboard);
    assert_snapshot!("keyboard_custom_layout", kdl);
}

#[test]
fn snapshot_mouse_default() {
    let mouse = MouseSettings::default();
    let kdl = generate_mouse_kdl(&mouse);
    assert_snapshot!("mouse_default", kdl);
}

#[test]
fn snapshot_mouse_custom() {
    let mouse = MouseSettings {
        accel_speed: 0.5,
        accel_profile: AccelProfile::Flat,
        natural_scroll: true,
        scroll_factor: 1.5,
        ..Default::default()
    };
    let kdl = generate_mouse_kdl(&mouse);
    assert_snapshot!("mouse_custom", kdl);
}

#[test]
fn snapshot_touchpad_default() {
    let touchpad = TouchpadSettings::default();
    let kdl = generate_touchpad_kdl(&touchpad);
    assert_snapshot!("touchpad_default", kdl);
}

#[test]
fn snapshot_touchpad_custom() {
    let touchpad = TouchpadSettings {
        tap: true,
        natural_scroll: true,
        dwt: true,
        dwtp: true,
        accel_speed: 0.3,
        scroll_method: ScrollMethod::TwoFinger,
        click_method: ClickMethod::Clickfinger,
        ..Default::default()
    };
    let kdl = generate_touchpad_kdl(&touchpad);
    assert_snapshot!("touchpad_custom", kdl);
}

// ============================================================================
// DISPLAY SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_cursor_default() {
    let cursor = CursorSettings::default();
    let kdl = generate_cursor_kdl(&cursor);
    assert_snapshot!("cursor_default", kdl);
}

#[test]
fn snapshot_cursor_custom() {
    let cursor = CursorSettings {
        theme: "Adwaita".to_string(),
        size: 32,
        hide_when_typing: true,
        hide_after_inactive_ms: Some(3000),
    };
    let kdl = generate_cursor_kdl(&cursor);
    assert_snapshot!("cursor_custom", kdl);
}

#[test]
fn snapshot_animations_default() {
    let animations = AnimationSettings::default();
    let kdl = generate_animations_kdl(&animations);
    assert_snapshot!("animations_default", kdl);
}

#[test]
fn snapshot_animations_disabled() {
    let animations = AnimationSettings {
        enabled: false,
        ..Default::default()
    };
    let kdl = generate_animations_kdl(&animations);
    assert_snapshot!("animations_disabled", kdl);
}

#[test]
fn snapshot_animations_slowdown() {
    let animations = AnimationSettings {
        slowdown: 2.5,
        ..Default::default()
    };
    let kdl = generate_animations_kdl(&animations);
    assert_snapshot!("animations_slowdown", kdl);
}

#[test]
fn snapshot_overview_default() {
    let overview = OverviewSettings::default();
    let kdl = generate_overview_kdl(&overview);
    assert_snapshot!("overview_default", kdl);
}

#[test]
fn snapshot_overview_with_backdrop() {
    let overview = OverviewSettings {
        zoom: 0.75,
        backdrop_color: Some(Color::from_hex("#000000cc").unwrap()),
        workspace_shadow: None,
    };
    let kdl = generate_overview_kdl(&overview);
    assert_snapshot!("overview_with_backdrop", kdl);
}

// ============================================================================
// OUTPUTS SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_outputs_empty() {
    let outputs = OutputSettings::default();
    let kdl = generate_outputs_kdl(&outputs);
    assert_snapshot!("outputs_empty", kdl);
}

#[test]
fn snapshot_outputs_single() {
    let outputs = OutputSettings {
        outputs: vec![OutputConfig {
            name: "DP-1".to_string(),
            enabled: true,
            scale: 1.5,
            mode: "2560x1440@144".to_string(),
            position_x: 0,
            position_y: 0,
            transform: Transform::Normal,
            vrr: VrrMode::On,
            ..Default::default()
        }],
    };
    let kdl = generate_outputs_kdl(&outputs);
    assert_snapshot!("outputs_single", kdl);
}

#[test]
fn snapshot_outputs_multiple() {
    let outputs = OutputSettings {
        outputs: vec![
            OutputConfig {
                name: "DP-1".to_string(),
                enabled: true,
                scale: 2.0,
                mode: "3840x2160@60".to_string(),
                position_x: 0,
                position_y: 0,
                transform: Transform::Normal,
                vrr: VrrMode::Off,
                ..Default::default()
            },
            OutputConfig {
                name: "HDMI-A-1".to_string(),
                enabled: true,
                scale: 1.0,
                mode: "1920x1080@60".to_string(),
                position_x: 3840,
                position_y: 0,
                transform: Transform::Rotate90,
                vrr: VrrMode::Off,
                ..Default::default()
            },
        ],
    };
    let kdl = generate_outputs_kdl(&outputs);
    assert_snapshot!("outputs_multiple", kdl);
}

#[test]
fn snapshot_outputs_disabled() {
    let outputs = OutputSettings {
        outputs: vec![OutputConfig {
            name: "HDMI-A-1".to_string(),
            enabled: false,
            ..Default::default()
        }],
    };
    let kdl = generate_outputs_kdl(&outputs);
    assert_snapshot!("outputs_disabled", kdl);
}

// ============================================================================
// WORKSPACES SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_workspaces_empty() {
    let workspaces = WorkspacesSettings::default();
    let kdl = generate_workspaces_kdl(&workspaces);
    assert_snapshot!("workspaces_empty", kdl);
}

#[test]
fn snapshot_workspaces_named() {
    let workspaces = WorkspacesSettings {
        workspaces: vec![
            NamedWorkspace {
                id: 1,
                name: "main".to_string(),
                open_on_output: None,
                layout_override: None,
            },
            NamedWorkspace {
                id: 2,
                name: "dev".to_string(),
                open_on_output: Some("DP-1".to_string()),
                layout_override: None,
            },
            NamedWorkspace {
                id: 3,
                name: "chat".to_string(),
                open_on_output: Some("HDMI-A-1".to_string()),
                layout_override: None,
            },
        ],
        next_id: 4,
    };
    let kdl = generate_workspaces_kdl(&workspaces);
    assert_snapshot!("workspaces_named", kdl);
}

// ============================================================================
// RULES SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_window_rules_empty() {
    let rules = WindowRulesSettings::default();
    let kdl = generate_window_rules_kdl(&rules, false);
    assert_snapshot!("window_rules_empty", kdl);
}

#[test]
fn snapshot_window_rules_single() {
    let rules = WindowRulesSettings {
        rules: vec![WindowRule {
            id: 1,
            name: "Firefox Floating".to_string(),
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".to_string()),
                ..Default::default()
            }],
            open_behavior: OpenBehavior::Floating,
            opacity: Some(0.95),
            corner_radius: Some(12),
            ..Default::default()
        }],
        next_id: 2,
    };
    let kdl = generate_window_rules_kdl(&rules, false);
    assert_snapshot!("window_rules_single", kdl);
}

#[test]
fn snapshot_window_rules_complex() {
    let rules = WindowRulesSettings {
        rules: vec![
            WindowRule {
                id: 1,
                name: "Terminals floating".to_string(),
                matches: vec![
                    WindowRuleMatch {
                        app_id: Some("kitty".to_string()),
                        ..Default::default()
                    },
                    WindowRuleMatch {
                        app_id: Some("Alacritty".to_string()),
                        ..Default::default()
                    },
                ],
                open_behavior: OpenBehavior::Floating,
                ..Default::default()
            },
            WindowRule {
                id: 2,
                name: "Video players fullscreen".to_string(),
                matches: vec![WindowRuleMatch {
                    app_id: Some("mpv".to_string()),
                    ..Default::default()
                }],
                open_behavior: OpenBehavior::Fullscreen,
                block_out_from_screencast: true,
                ..Default::default()
            },
        ],
        next_id: 3,
    };
    let kdl = generate_window_rules_kdl(&rules, false);
    assert_snapshot!("window_rules_complex", kdl);
}

#[test]
fn snapshot_layer_rules_empty() {
    let rules = LayerRulesSettings::default();
    let kdl = generate_layer_rules_kdl(&rules);
    assert_snapshot!("layer_rules_empty", kdl);
}

// ============================================================================
// MISCELLANEOUS SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_misc_default() {
    let misc = MiscSettings::default();
    let kdl = generate_misc_kdl(&misc);
    assert_snapshot!("misc_default", kdl);
}

#[test]
fn snapshot_misc_custom() {
    let misc = MiscSettings {
        prefer_no_csd: true,
        screenshot_path: "~/Pictures/Screenshots".to_string(),
        hotkey_overlay_skip_at_startup: true,
        ..Default::default()
    };
    let kdl = generate_misc_kdl(&misc);
    assert_snapshot!("misc_custom", kdl);
}

// ============================================================================
// STARTUP AND ENVIRONMENT SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_startup_empty() {
    let startup = StartupSettings::default();
    let kdl = generate_startup_kdl(&startup);
    assert_snapshot!("startup_empty", kdl);
}

#[test]
fn snapshot_startup_with_commands() {
    let startup = StartupSettings {
        commands: vec![
            StartupCommand {
                id: 1,
                command: vec!["waybar".to_string()],
            },
            StartupCommand {
                id: 2,
                command: vec![
                    "swaybg".to_string(),
                    "-i".to_string(),
                    "~/wallpaper.png".to_string(),
                ],
            },
        ],
        next_id: 3,
    };
    let kdl = generate_startup_kdl(&startup);
    assert_snapshot!("startup_with_commands", kdl);
}

#[test]
fn snapshot_environment_empty() {
    let env = EnvironmentSettings::default();
    let kdl = generate_environment_kdl(&env);
    assert_snapshot!("environment_empty", kdl);
}

#[test]
fn snapshot_environment_with_vars() {
    let env = EnvironmentSettings {
        variables: vec![
            EnvironmentVariable {
                id: 1,
                name: "GTK_THEME".to_string(),
                value: "Adwaita:dark".to_string(),
            },
            EnvironmentVariable {
                id: 2,
                name: "QT_QPA_PLATFORM".to_string(),
                value: "wayland".to_string(),
            },
        ],
        next_id: 3,
    };
    let kdl = generate_environment_kdl(&env);
    assert_snapshot!("environment_with_vars", kdl);
}

// ============================================================================
// DEBUG SNAPSHOTS
// ============================================================================

#[test]
fn snapshot_debug_default() {
    let debug = DebugSettings::default();
    let kdl = generate_debug_kdl(&debug);
    assert_snapshot!("debug_default", kdl);
}
