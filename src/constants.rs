/// Application name
pub const APP_NAME: &str = "Nirify";

/// Application version from Cargo.toml
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Directory name for our managed config files
pub const CONFIG_DIR_NAME: &str = "nirify";

/// Main include file name
pub const MAIN_KDL_NAME: &str = "main.kdl";

/// Default gap size in pixels
pub const DEFAULT_GAP_SIZE: i32 = 16;

/// Default focus ring width
pub const DEFAULT_FOCUS_RING_WIDTH: i32 = 4;

/// Default border width
pub const DEFAULT_BORDER_WIDTH: i32 = 2;

/// Default focus ring color (blue)
pub const DEFAULT_FOCUS_RING_COLOR: &str = "#7fc8ff";

/// Default border color (orange)
pub const DEFAULT_BORDER_COLOR: &str = "#ffc87f";

// Validation ranges for settings values

/// Focus ring width range (pixels)
pub const FOCUS_RING_WIDTH_MIN: f32 = 1.0;
pub const FOCUS_RING_WIDTH_MAX: f32 = 16.0;

/// Border thickness range (pixels)
pub const BORDER_THICKNESS_MIN: f32 = 1.0;
pub const BORDER_THICKNESS_MAX: f32 = 8.0;

/// Gap size range (pixels)
pub const GAP_SIZE_MIN: f32 = 0.0;
pub const GAP_SIZE_MAX: f32 = 64.0;

/// Corner radius range (pixels)
pub const CORNER_RADIUS_MIN: f32 = 0.0;
pub const CORNER_RADIUS_MAX: f32 = 32.0;

/// Column width proportion range (0.1 = 10%, 1.0 = 100%)
pub const COLUMN_PROPORTION_MIN: f32 = 0.1;
pub const COLUMN_PROPORTION_MAX: f32 = 1.0;

/// Column fixed width range (pixels)
pub const COLUMN_FIXED_MIN: f32 = 200.0;
pub const COLUMN_FIXED_MAX: f32 = 4000.0;

/// Strut size range (pixels)
pub const STRUT_SIZE_MIN: f32 = 0.0;
pub const STRUT_SIZE_MAX: f32 = 500.0;

// Default values for settings

/// Default inactive focus ring color (gray)
pub const DEFAULT_FOCUS_RING_INACTIVE_COLOR: &str = "#505050";

/// Default inactive border color (gray)
pub const DEFAULT_BORDER_INACTIVE_COLOR: &str = "#808080";

/// Default corner radius (pixels)
pub const DEFAULT_CORNER_RADIUS: f32 = 12.0;

/// Default column width proportion (50%)
pub const DEFAULT_COLUMN_PROPORTION: f32 = 0.5;

/// Default column width fixed (pixels)
pub const DEFAULT_COLUMN_FIXED: f32 = 800.0;

/// Default keyboard repeat delay (ms)
pub const DEFAULT_REPEAT_DELAY: i32 = 600;

/// Default keyboard repeat rate (chars/sec)
pub const DEFAULT_REPEAT_RATE: i32 = 25;

/// Default cursor size
pub const DEFAULT_CURSOR_SIZE: i32 = 24;

/// Cursor size range (pixels)
pub const CURSOR_SIZE_MIN: i32 = 16;
pub const CURSOR_SIZE_MAX: i32 = 64;

/// Hide after inactive range (milliseconds)
pub const HIDE_INACTIVE_MIN: i32 = 100;
pub const HIDE_INACTIVE_MAX: i32 = 10000;

/// Default overview zoom
pub const DEFAULT_OVERVIEW_ZOOM: f64 = 0.5;

/// Overview zoom range (0.1 = 10%, 1.0 = 100%)
pub const OVERVIEW_ZOOM_MIN: f64 = 0.1;
pub const OVERVIEW_ZOOM_MAX: f64 = 1.0;

/// Animation slowdown range
pub const ANIMATION_SLOWDOWN_MIN: f64 = 0.1;
pub const ANIMATION_SLOWDOWN_MAX: f64 = 10.0;

/// Keyboard repeat delay range (ms)
pub const REPEAT_DELAY_MIN: i32 = 100;
pub const REPEAT_DELAY_MAX: i32 = 2000;

/// Keyboard repeat rate range (chars/sec)
pub const REPEAT_RATE_MIN: i32 = 1;
pub const REPEAT_RATE_MAX: i32 = 100;

/// Acceleration speed range (-1 to 1)
pub const ACCEL_SPEED_MIN: f64 = -1.0;
pub const ACCEL_SPEED_MAX: f64 = 1.0;

/// Scroll factor range
pub const SCROLL_FACTOR_MIN: f64 = 0.1;
pub const SCROLL_FACTOR_MAX: f64 = 10.0;

// UI index constants for ComboBox selections

/// Column width type index: Proportion
pub const COLUMN_WIDTH_TYPE_PROPORTION: i32 = 0;

/// Column width type index: Fixed
pub const COLUMN_WIDTH_TYPE_FIXED: i32 = 1;

// ============================================================================
// ANIMATION PARAMETERS (Phase 7)
// ============================================================================

/// Spring damping ratio range (0.1-3.0 for UI, but niri supports higher)
/// 1.0 = critically damped (no bounce), <1.0 = underdamped (bouncy)
pub const DAMPING_RATIO_MIN: f64 = 0.1;
pub const DAMPING_RATIO_MAX: f64 = 3.0;
pub const DAMPING_RATIO_DEFAULT: f64 = 1.0;

/// Spring stiffness range (higher = faster/stiffer)
pub const STIFFNESS_MIN: i32 = 50;
pub const STIFFNESS_MAX: i32 = 2000;
pub const STIFFNESS_DEFAULT: i32 = 800;

/// Spring epsilon range (animation end threshold, lower = smoother ending)
pub const EPSILON_MIN: f64 = 0.00001;
pub const EPSILON_MAX: f64 = 0.1;
pub const EPSILON_DEFAULT: f64 = 0.0001;

/// Easing duration range (milliseconds)
pub const EASING_DURATION_MIN: i32 = 50;
pub const EASING_DURATION_MAX: i32 = 1000;
pub const EASING_DURATION_DEFAULT: i32 = 150;

// Animation type and easing curve indices are now derived via SlintIndex
// on AnimationType and EasingCurve enums in models.rs

// ============================================================================
// UI TIMING CONSTANTS
// ============================================================================

/// Debounce delay for auto-save (milliseconds)
///
/// 300ms balances responsiveness with avoiding excessive IPC reloads:
/// - Short enough that users see their changes apply quickly
/// - Long enough to batch rapid slider drags into a single save/reload
/// - Prevents spamming the niri compositor with reload requests
pub const SAVE_DEBOUNCE_MS: u64 = 300;

/// Debounce delay for search input (milliseconds)
pub const SEARCH_DEBOUNCE_MS: u64 = 200;

/// Toast notification auto-dismiss delay (milliseconds)
pub const TOAST_DISMISS_MS: u64 = 3000;

/// Status message auto-hide delay (seconds)
pub const STATUS_AUTO_HIDE_SECS: u64 = 3;

// ============================================================================
// STRING AND COLLECTION SIZE LIMITS (Task 12.1 & 12.2)
// ============================================================================

/// Maximum length for user-input strings (prevents memory issues)
pub const MAX_STRING_LENGTH: usize = 1024;

/// Maximum length for rule patterns (app-id, title, namespace)
/// Shorter than general strings since patterns rarely need to be very long
pub const MAX_PATTERN_LENGTH: usize = 512;

/// Maximum number of window rules
pub const MAX_WINDOW_RULES: usize = 100;

/// Maximum number of layer rules
pub const MAX_LAYER_RULES: usize = 100;

/// Maximum number of named workspaces
pub const MAX_WORKSPACES: usize = 50;

/// Maximum number of match criteria per rule
pub const MAX_MATCHES_PER_RULE: usize = 20;

/// Maximum number of environment variables
pub const MAX_ENVIRONMENT_VARS: usize = 100;

/// Maximum number of startup commands
pub const MAX_STARTUP_COMMANDS: usize = 50;
