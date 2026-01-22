//! Animation settings and types

use niri_settings_macros::SlintIndex;

use crate::constants::{
    DAMPING_RATIO_DEFAULT, EASING_DURATION_DEFAULT, EPSILON_DEFAULT, STIFFNESS_DEFAULT,
};

/// Type of animation curve
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum AnimationType {
    /// Use niri's default for this animation
    #[default]
    #[slint_index(default)]
    Default,
    /// Animation disabled
    Off,
    /// Spring physics animation
    Spring,
    /// Easing curve animation
    Easing,
    /// Custom GLSL shader (only for window-open, window-close, window-resize)
    CustomShader,
}

/// Animation identifier for indexed callbacks
///
/// Maps to fields in `PerAnimationSettings`. Used instead of magic constants
/// to provide type safety and derive `get_animation_mut`/`get_animation_name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum AnimationId {
    #[default]
    #[slint_index(default)]
    WorkspaceSwitch,
    Overview,
    WindowOpen,
    WindowClose,
    WindowMovement,
    WindowResize,
    HorizontalViewMovement,
    ConfigNotification,
    ExitConfirmation,
    ScreenshotUi,
    RecentWindows,
}

impl AnimationId {
    /// Get the human-readable name for logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::WorkspaceSwitch => "Workspace switch",
            Self::Overview => "Overview",
            Self::WindowOpen => "Window open",
            Self::WindowClose => "Window close",
            Self::WindowMovement => "Window movement",
            Self::WindowResize => "Window resize",
            Self::HorizontalViewMovement => "Horizontal view",
            Self::ConfigNotification => "Config notification",
            Self::ExitConfirmation => "Exit confirmation",
            Self::ScreenshotUi => "Screenshot UI",
            Self::RecentWindows => "Recent windows",
        }
    }

    /// Get mutable reference to the corresponding animation settings
    pub fn get_mut<'a>(
        &self,
        per_anim: &'a mut PerAnimationSettings,
    ) -> &'a mut SingleAnimationConfig {
        match self {
            Self::WorkspaceSwitch => &mut per_anim.workspace_switch,
            Self::Overview => &mut per_anim.overview_open_close,
            Self::WindowOpen => &mut per_anim.window_open,
            Self::WindowClose => &mut per_anim.window_close,
            Self::WindowMovement => &mut per_anim.window_movement,
            Self::WindowResize => &mut per_anim.window_resize,
            Self::HorizontalViewMovement => &mut per_anim.horizontal_view_movement,
            Self::ConfigNotification => &mut per_anim.config_notification_open_close,
            Self::ExitConfirmation => &mut per_anim.exit_confirmation_open_close,
            Self::ScreenshotUi => &mut per_anim.screenshot_ui_open,
            Self::RecentWindows => &mut per_anim.recent_windows_close,
        }
    }

    /// Get immutable reference to the corresponding animation settings
    pub fn get<'a>(&self, per_anim: &'a PerAnimationSettings) -> &'a SingleAnimationConfig {
        match self {
            Self::WorkspaceSwitch => &per_anim.workspace_switch,
            Self::Overview => &per_anim.overview_open_close,
            Self::WindowOpen => &per_anim.window_open,
            Self::WindowClose => &per_anim.window_close,
            Self::WindowMovement => &per_anim.window_movement,
            Self::WindowResize => &per_anim.window_resize,
            Self::HorizontalViewMovement => &per_anim.horizontal_view_movement,
            Self::ConfigNotification => &per_anim.config_notification_open_close,
            Self::ExitConfirmation => &per_anim.exit_confirmation_open_close,
            Self::ScreenshotUi => &per_anim.screenshot_ui_open,
            Self::RecentWindows => &per_anim.recent_windows_close,
        }
    }

    /// Check if this animation supports custom GLSL shaders
    /// Only window-open, window-close, and window-resize support custom shaders
    pub fn supports_custom_shader(&self) -> bool {
        matches!(
            self,
            Self::WindowOpen | Self::WindowClose | Self::WindowResize
        )
    }

    /// Get the required GLSL function name for custom shaders
    pub fn shader_function_name(&self) -> Option<&'static str> {
        match self {
            Self::WindowOpen => Some("open_color"),
            Self::WindowClose => Some("close_color"),
            Self::WindowResize => Some("resize_color"),
            _ => None,
        }
    }
}

/// Easing curve type
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EasingCurve {
    #[default]
    EaseOutQuad,
    EaseOutCubic,
    EaseOutExpo,
    Linear,
    /// Custom cubic-bezier curve with control points (x1, y1, x2, y2)
    CubicBezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
}

impl EasingCurve {
    /// Convert to UI combobox index
    /// 0=EaseOutQuad, 1=EaseOutCubic, 2=EaseOutExpo, 3=Linear, 4=CubicBezier
    pub fn to_index(&self) -> i32 {
        match self {
            Self::EaseOutQuad => 0,
            Self::EaseOutCubic => 1,
            Self::EaseOutExpo => 2,
            Self::Linear => 3,
            Self::CubicBezier { .. } => 4,
        }
    }

    /// Create from UI combobox index (CubicBezier gets default control points)
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => Self::EaseOutCubic,
            2 => Self::EaseOutExpo,
            3 => Self::Linear,
            4 => Self::CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            },
            _ => Self::EaseOutQuad,
        }
    }

    /// Check if this is a cubic-bezier curve
    pub fn is_cubic_bezier(&self) -> bool {
        matches!(self, Self::CubicBezier { .. })
    }

    /// Get cubic-bezier control points, or None if not a cubic-bezier
    pub fn bezier_points(&self) -> Option<(f64, f64, f64, f64)> {
        match self {
            Self::CubicBezier { x1, y1, x2, y2 } => Some((*x1, *y1, *x2, *y2)),
            _ => None,
        }
    }

    /// Convert to KDL string (for preset curves only)
    pub fn to_kdl(&self) -> Option<&'static str> {
        match self {
            Self::EaseOutQuad => Some("ease-out-quad"),
            Self::EaseOutCubic => Some("ease-out-cubic"),
            Self::EaseOutExpo => Some("ease-out-expo"),
            Self::Linear => Some("linear"),
            Self::CubicBezier { .. } => None, // Handled separately in storage
        }
    }

    /// Parse from KDL string (preset curves only)
    pub fn from_kdl(s: &str) -> Self {
        match s {
            "ease-out-cubic" => Self::EaseOutCubic,
            "ease-out-expo" => Self::EaseOutExpo,
            "linear" => Self::Linear,
            _ => Self::EaseOutQuad,
        }
    }
}

/// Spring animation parameters
#[derive(Debug, Clone, PartialEq)]
pub struct SpringParams {
    /// Damping ratio (1.0 = critically damped, <1.0 = bouncy)
    pub damping_ratio: f64,
    /// Stiffness (higher = faster/stiffer)
    pub stiffness: i32,
    /// Epsilon (animation end threshold)
    pub epsilon: f64,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            damping_ratio: DAMPING_RATIO_DEFAULT,
            stiffness: STIFFNESS_DEFAULT,
            epsilon: EPSILON_DEFAULT,
        }
    }
}

/// Easing animation parameters
#[derive(Debug, Clone, PartialEq)]
pub struct EasingParams {
    /// Duration in milliseconds
    pub duration_ms: i32,
    /// Easing curve type
    pub curve: EasingCurve,
}

impl Default for EasingParams {
    fn default() -> Self {
        Self {
            duration_ms: EASING_DURATION_DEFAULT,
            curve: EasingCurve::EaseOutQuad,
        }
    }
}

/// Configuration for a single animation
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SingleAnimationConfig {
    /// Animation type (Default, Off, Spring, Easing, CustomShader)
    pub animation_type: AnimationType,
    /// Spring parameters (used when animation_type == Spring)
    pub spring: SpringParams,
    /// Easing parameters (used when animation_type == Easing)
    pub easing: EasingParams,
    /// Custom GLSL shader code (used when animation_type == CustomShader)
    /// Only supported for window-open, window-close, and window-resize animations
    pub custom_shader: Option<String>,
}

/// Per-animation configuration for all niri animations
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PerAnimationSettings {
    /// Workspace switch animation (default: spring)
    pub workspace_switch: SingleAnimationConfig,
    /// Window open animation (default: easing)
    pub window_open: SingleAnimationConfig,
    /// Window close animation (default: easing)
    pub window_close: SingleAnimationConfig,
    /// Horizontal view movement animation (default: spring)
    pub horizontal_view_movement: SingleAnimationConfig,
    /// Window movement animation (default: spring)
    pub window_movement: SingleAnimationConfig,
    /// Window resize animation (default: spring)
    pub window_resize: SingleAnimationConfig,
    /// Config notification open/close animation (default: spring)
    pub config_notification_open_close: SingleAnimationConfig,
    /// Exit confirmation open/close animation (default: spring)
    pub exit_confirmation_open_close: SingleAnimationConfig,
    /// Screenshot UI open animation (default: easing)
    pub screenshot_ui_open: SingleAnimationConfig,
    /// Overview open/close animation (default: spring)
    pub overview_open_close: SingleAnimationConfig,
    /// Recent windows close animation (default: spring)
    pub recent_windows_close: SingleAnimationConfig,
}

/// Animation settings
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationSettings {
    pub enabled: bool,
    pub slowdown: f64,
    /// Per-animation configuration
    pub per_animation: PerAnimationSettings,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            slowdown: 1.0,
            per_animation: PerAnimationSettings::default(),
        }
    }
}
