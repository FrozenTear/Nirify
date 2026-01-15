//! Settings models for niri configuration
//!
//! This module contains all the data structures that represent niri's configuration.
//! Each submodule focuses on a specific category of settings.

mod animation;
mod appearance;
mod behavior;
mod debug;
mod gestures;
mod input;
mod keybindings;
mod layout;
mod misc;
mod output;
mod recent_windows;
mod rules;
mod startup;
mod switch_events;
mod workspaces;

// Re-export all types for convenient access
pub use animation::*;
pub use appearance::*;
pub use behavior::*;
pub use debug::*;
pub use gestures::*;
pub use input::*;
pub use keybindings::*;
pub use layout::*;
pub use misc::*;
pub use output::*;
pub use recent_windows::*;
pub use rules::*;
pub use startup::*;
pub use switch_events::*;
pub use workspaces::*;

use crate::constants::{
    ACCEL_SPEED_MAX, ACCEL_SPEED_MIN, ANIMATION_SLOWDOWN_MAX, ANIMATION_SLOWDOWN_MIN,
    BORDER_THICKNESS_MAX, BORDER_THICKNESS_MIN, COLUMN_FIXED_MAX, COLUMN_FIXED_MIN,
    COLUMN_PROPORTION_MAX, COLUMN_PROPORTION_MIN, CORNER_RADIUS_MAX, CORNER_RADIUS_MIN,
    CURSOR_SIZE_MAX, CURSOR_SIZE_MIN, DAMPING_RATIO_MAX, DAMPING_RATIO_MIN, EASING_DURATION_MAX,
    EASING_DURATION_MIN, EPSILON_MAX, EPSILON_MIN, FOCUS_RING_WIDTH_MAX, FOCUS_RING_WIDTH_MIN,
    GAP_SIZE_MAX, GAP_SIZE_MIN, HIDE_INACTIVE_MAX, HIDE_INACTIVE_MIN, OVERVIEW_ZOOM_MAX,
    OVERVIEW_ZOOM_MIN, REPEAT_DELAY_MAX, REPEAT_DELAY_MIN, REPEAT_RATE_MAX, REPEAT_RATE_MIN,
    SCROLL_FACTOR_MAX, SCROLL_FACTOR_MIN, STIFFNESS_MAX, STIFFNESS_MIN, STRUT_SIZE_MAX,
    STRUT_SIZE_MIN,
};

/// Root settings structure containing all configuration
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Settings {
    pub appearance: AppearanceSettings,
    pub behavior: BehaviorSettings,
    pub keyboard: KeyboardSettings,
    pub mouse: MouseSettings,
    pub touchpad: TouchpadSettings,
    pub trackpoint: TrackpointSettings,
    pub trackball: TrackballSettings,
    pub tablet: TabletSettings,
    pub touch: TouchSettings,
    pub animations: AnimationSettings,
    pub cursor: CursorSettings,
    pub overview: OverviewSettings,
    pub outputs: OutputSettings,
    pub layout_extras: LayoutExtrasSettings,
    pub gestures: GestureSettings,
    pub miscellaneous: MiscSettings,
    pub workspaces: WorkspacesSettings,
    pub layer_rules: LayerRulesSettings,
    pub window_rules: WindowRulesSettings,
    /// Keybindings (keyboard shortcuts)
    pub keybindings: KeybindingsSettings,
    /// Startup commands (spawn-at-startup)
    pub startup: StartupSettings,
    /// Environment variables
    pub environment: EnvironmentSettings,
    /// Debug settings
    pub debug: DebugSettings,
    /// Switch events (lid close, tablet mode, etc.)
    pub switch_events: SwitchEventsSettings,
    /// Recent windows switcher settings (v25.05+)
    pub recent_windows: RecentWindowsSettings,
}

impl Settings {
    /// Validate and clamp all settings to their valid ranges.
    ///
    /// This ensures all values are within acceptable bounds after loading
    /// from potentially corrupted or hand-edited config files.
    ///
    /// Values are clamped and logged when out of range, following
    /// the principle of being lenient in what we accept.
    pub fn validate(&mut self) {
        // Helper macro to clamp and log when value changes
        macro_rules! clamp_and_log {
            ($field:expr, $min:expr, $max:expr, $name:expr) => {{
                let original = $field;
                let clamped = original.clamp($min, $max);
                if original != clamped {
                    log::debug!(
                        "Clamped {} from {} to {} (range: {}..={})",
                        $name,
                        original,
                        clamped,
                        $min,
                        $max
                    );
                }
                $field = clamped;
            }};
        }

        // Appearance
        clamp_and_log!(
            self.appearance.focus_ring_width,
            FOCUS_RING_WIDTH_MIN,
            FOCUS_RING_WIDTH_MAX,
            "focus_ring_width"
        );
        clamp_and_log!(
            self.appearance.border_thickness,
            BORDER_THICKNESS_MIN,
            BORDER_THICKNESS_MAX,
            "border_thickness"
        );
        clamp_and_log!(
            self.appearance.gaps_inner,
            GAP_SIZE_MIN,
            GAP_SIZE_MAX,
            "gaps_inner"
        );
        clamp_and_log!(
            self.appearance.gaps_outer,
            GAP_SIZE_MIN,
            GAP_SIZE_MAX,
            "gaps_outer"
        );
        clamp_and_log!(
            self.appearance.corner_radius,
            CORNER_RADIUS_MIN,
            CORNER_RADIUS_MAX,
            "corner_radius"
        );

        // Behavior - struts
        clamp_and_log!(
            self.behavior.strut_left,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "strut_left"
        );
        clamp_and_log!(
            self.behavior.strut_right,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "strut_right"
        );
        clamp_and_log!(
            self.behavior.strut_top,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "strut_top"
        );
        clamp_and_log!(
            self.behavior.strut_bottom,
            STRUT_SIZE_MIN,
            STRUT_SIZE_MAX,
            "strut_bottom"
        );

        // Behavior - column width
        clamp_and_log!(
            self.behavior.default_column_width_proportion,
            COLUMN_PROPORTION_MIN,
            COLUMN_PROPORTION_MAX,
            "default_column_width_proportion"
        );
        clamp_and_log!(
            self.behavior.default_column_width_fixed,
            COLUMN_FIXED_MIN,
            COLUMN_FIXED_MAX,
            "default_column_width_fixed"
        );

        // Keyboard
        clamp_and_log!(
            self.keyboard.repeat_delay,
            REPEAT_DELAY_MIN,
            REPEAT_DELAY_MAX,
            "repeat_delay"
        );
        clamp_and_log!(
            self.keyboard.repeat_rate,
            REPEAT_RATE_MIN,
            REPEAT_RATE_MAX,
            "repeat_rate"
        );

        // Mouse
        clamp_and_log!(
            self.mouse.accel_speed,
            ACCEL_SPEED_MIN,
            ACCEL_SPEED_MAX,
            "mouse.accel_speed"
        );
        clamp_and_log!(
            self.mouse.scroll_factor,
            SCROLL_FACTOR_MIN,
            SCROLL_FACTOR_MAX,
            "mouse.scroll_factor"
        );

        // Touchpad
        clamp_and_log!(
            self.touchpad.accel_speed,
            ACCEL_SPEED_MIN,
            ACCEL_SPEED_MAX,
            "touchpad.accel_speed"
        );
        clamp_and_log!(
            self.touchpad.scroll_factor,
            SCROLL_FACTOR_MIN,
            SCROLL_FACTOR_MAX,
            "touchpad.scroll_factor"
        );

        // Animations
        clamp_and_log!(
            self.animations.slowdown,
            ANIMATION_SLOWDOWN_MIN,
            ANIMATION_SLOWDOWN_MAX,
            "animations.slowdown"
        );

        // Per-animation validation
        self.validate_per_animation();

        // Cursor
        clamp_and_log!(
            self.cursor.size,
            CURSOR_SIZE_MIN,
            CURSOR_SIZE_MAX,
            "cursor.size"
        );
        if let Some(ms) = self.cursor.hide_after_inactive_ms {
            let clamped = ms.clamp(HIDE_INACTIVE_MIN, HIDE_INACTIVE_MAX);
            if ms != clamped {
                log::debug!(
                    "Clamped cursor.hide_after_inactive_ms from {} to {} (range: {}..={})",
                    ms,
                    clamped,
                    HIDE_INACTIVE_MIN,
                    HIDE_INACTIVE_MAX
                );
            }
            self.cursor.hide_after_inactive_ms = Some(clamped);
        }

        // Overview
        clamp_and_log!(
            self.overview.zoom,
            OVERVIEW_ZOOM_MIN,
            OVERVIEW_ZOOM_MAX,
            "overview.zoom"
        );

        // Window rules - validate opacity and corner radius
        for (i, rule) in self.window_rules.rules.iter_mut().enumerate() {
            if let Some(opacity) = rule.opacity {
                let clamped = opacity.clamp(0.0, 1.0);
                if opacity != clamped {
                    log::debug!(
                        "Clamped window_rule[{}].opacity from {} to {} (range: 0.0..=1.0)",
                        i,
                        opacity,
                        clamped
                    );
                }
                rule.opacity = Some(clamped);
            }
            if let Some(radius) = rule.corner_radius {
                let clamped = radius.clamp(CORNER_RADIUS_MIN as i32, CORNER_RADIUS_MAX as i32);
                if radius != clamped {
                    log::debug!(
                        "Clamped window_rule[{}].corner_radius from {} to {} (range: {}..={})",
                        i,
                        radius,
                        clamped,
                        CORNER_RADIUS_MIN,
                        CORNER_RADIUS_MAX
                    );
                }
                rule.corner_radius = Some(clamped);
            }
            if let Some(width) = rule.default_column_width {
                let clamped = width.clamp(COLUMN_PROPORTION_MIN, COLUMN_PROPORTION_MAX);
                if width != clamped {
                    log::debug!(
                        "Clamped window_rule[{}].default_column_width from {} to {} (range: {}..={})",
                        i,
                        width,
                        clamped,
                        COLUMN_PROPORTION_MIN,
                        COLUMN_PROPORTION_MAX
                    );
                }
                rule.default_column_width = Some(clamped);
            }
        }
    }

    /// Validate per-animation parameters
    fn validate_per_animation(&mut self) {
        let animations = [
            &mut self.animations.per_animation.workspace_switch,
            &mut self.animations.per_animation.window_open,
            &mut self.animations.per_animation.window_close,
            &mut self.animations.per_animation.horizontal_view_movement,
            &mut self.animations.per_animation.window_movement,
            &mut self.animations.per_animation.window_resize,
            &mut self.animations.per_animation.config_notification_open_close,
            &mut self.animations.per_animation.exit_confirmation_open_close,
            &mut self.animations.per_animation.screenshot_ui_open,
            &mut self.animations.per_animation.overview_open_close,
            &mut self.animations.per_animation.recent_windows_close,
        ];

        for anim in animations {
            // Validate spring parameters
            anim.spring.damping_ratio = anim
                .spring
                .damping_ratio
                .clamp(DAMPING_RATIO_MIN, DAMPING_RATIO_MAX);
            anim.spring.stiffness = anim.spring.stiffness.clamp(STIFFNESS_MIN, STIFFNESS_MAX);
            anim.spring.epsilon = anim.spring.epsilon.clamp(EPSILON_MIN, EPSILON_MAX);

            // Validate easing parameters
            anim.easing.duration_ms = anim
                .easing
                .duration_ms
                .clamp(EASING_DURATION_MIN, EASING_DURATION_MAX);
        }
    }
}
