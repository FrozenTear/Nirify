use niri_settings_macros::SlintIndex;

/// Represents a color in RGBA format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Parse a single hex digit (0-9, a-f, A-F) to its numeric value
#[inline]
fn parse_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Parse two hex digits into a single byte value
#[inline]
fn parse_hex_pair(high: u8, low: u8) -> Option<u8> {
    let h = parse_hex_digit(high)?;
    let l = parse_hex_digit(low)?;
    Some(h * 16 + l)
}

impl Color {
    /// Parses a hex color string into a Color struct.
    ///
    /// Accepts hex strings with or without the leading '#' character.
    /// Supports:
    /// - 3-character shorthand (#RGB) - each digit is doubled (e.g., #ABC -> #AABBCC)
    /// - 4-character shorthand (#RGBA) - each digit is doubled
    /// - 6-character full (#RRGGBB)
    /// - 8-character full with alpha (#RRGGBBAA)
    ///
    /// # Arguments
    /// * `hex` - A hex color string like "#7fc8ff", "#abc", or "7fc8ff80"
    ///
    /// # Returns
    /// * `Some(Color)` - Successfully parsed color
    /// * `None` - Invalid hex format or length
    ///
    /// # Examples
    /// ```ignore
    /// let color = Color::from_hex("#7fc8ff").unwrap();
    /// let shorthand = Color::from_hex("#abc").unwrap(); // Same as #aabbcc
    /// let with_alpha = Color::from_hex("#7fc8ff80").unwrap();
    /// ```
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#').as_bytes();
        match hex.len() {
            // 3-digit shorthand: #RGB -> #RRGGBB
            3 => {
                let r = parse_hex_digit(hex[0])? * 17; // 0xF * 17 = 0xFF
                let g = parse_hex_digit(hex[1])? * 17;
                let b = parse_hex_digit(hex[2])? * 17;
                Some(Color { r, g, b, a: 255 })
            }
            // 4-digit shorthand: #RGBA -> #RRGGBBAA
            4 => {
                let r = parse_hex_digit(hex[0])? * 17;
                let g = parse_hex_digit(hex[1])? * 17;
                let b = parse_hex_digit(hex[2])? * 17;
                let a = parse_hex_digit(hex[3])? * 17;
                Some(Color { r, g, b, a })
            }
            // 6-digit full: #RRGGBB
            6 => {
                let r = parse_hex_pair(hex[0], hex[1])?;
                let g = parse_hex_pair(hex[2], hex[3])?;
                let b = parse_hex_pair(hex[4], hex[5])?;
                Some(Color { r, g, b, a: 255 })
            }
            // 8-digit full with alpha: #RRGGBBAA
            8 => {
                let r = parse_hex_pair(hex[0], hex[1])?;
                let g = parse_hex_pair(hex[2], hex[3])?;
                let b = parse_hex_pair(hex[4], hex[5])?;
                let a = parse_hex_pair(hex[6], hex[7])?;
                Some(Color { r, g, b, a })
            }
            _ => None,
        }
    }

    /// Converts the Color to a lowercase hex string.
    ///
    /// Returns a 7-character string (#RRGGBB) for fully opaque colors (alpha = 255),
    /// or a 9-character string (#RRGGBBAA) when alpha is not 255.
    ///
    /// # Examples
    /// ```ignore
    /// let color = Color { r: 127, g: 200, b: 255, a: 255 };
    /// assert_eq!(color.to_hex(), "#7fc8ff");
    /// ```
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

/// Modifier key used for compositor shortcuts and bindings.
///
/// This determines which key acts as the primary modifier for window
/// management operations in niri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum ModKey {
    /// Super/Windows/Meta key (default)
    #[default]
    #[slint_index(default)]
    Super,
    /// Alt key
    Alt,
    /// Control key
    Ctrl,
    /// Shift key
    Shift,
    /// Mod3 key (typically unused, can be remapped)
    Mod3,
    /// Mod5 key (typically unused, can be remapped)
    Mod5,
}

impl std::fmt::Display for ModKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_kdl())
    }
}

impl ModKey {
    /// Returns all possible values for UI pickers
    pub fn all() -> &'static [Self] {
        &[Self::Super, Self::Alt, Self::Ctrl, Self::Shift, Self::Mod3, Self::Mod5]
    }

    /// Convert to KDL string representation
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Super => "Super",
            Self::Alt => "Alt",
            Self::Ctrl => "Ctrl",
            Self::Shift => "Shift",
            Self::Mod3 => "Mod3",
            Self::Mod5 => "Mod5",
        }
    }

    /// Parse from KDL string
    pub fn from_kdl(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "super" => Some(Self::Super),
            "alt" => Some(Self::Alt),
            "ctrl" => Some(Self::Ctrl),
            "shift" => Some(Self::Shift),
            "mod3" => Some(Self::Mod3),
            "mod5" => Some(Self::Mod5),
            _ => None,
        }
    }
}

/// Acceleration profile for pointer input devices (mouse, touchpad).
///
/// Controls how pointer movement speed relates to physical device movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum AccelProfile {
    /// Adaptive acceleration that adjusts based on movement speed (default)
    #[default]
    #[slint_index(default)]
    Adaptive,
    /// Flat/linear acceleration with constant ratio
    Flat,
}

impl std::fmt::Display for AccelProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Adaptive => write!(f, "Adaptive"),
            Self::Flat => write!(f, "Flat"),
        }
    }
}

impl AccelProfile {
    pub fn all() -> &'static [Self] {
        &[Self::Adaptive, Self::Flat]
    }
}

/// Scroll method for touchpad input devices.
///
/// Determines how scrolling gestures are interpreted on the touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum ScrollMethod {
    /// Two-finger scroll gesture (default, most common)
    #[default]
    #[slint_index(default)]
    TwoFinger,
    /// Edge scrolling along touchpad border
    Edge,
    /// Scroll while holding a button down
    OnButtonDown,
    /// Disable touchpad scrolling
    NoScroll,
}

impl std::fmt::Display for ScrollMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TwoFinger => write!(f, "Two Finger"),
            Self::Edge => write!(f, "Edge"),
            Self::OnButtonDown => write!(f, "On Button Down"),
            Self::NoScroll => write!(f, "No Scroll"),
        }
    }
}

impl ScrollMethod {
    pub fn all() -> &'static [Self] {
        &[Self::TwoFinger, Self::Edge, Self::OnButtonDown, Self::NoScroll]
    }
}

/// Click method for touchpad tap-to-click behavior.
///
/// Determines how multi-finger taps are interpreted as mouse button clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum ClickMethod {
    /// Click buttons based on tap location (bottom-left=left, bottom-right=right)
    #[default]
    #[slint_index(default)]
    ButtonAreas,
    /// Click buttons based on finger count (1=left, 2=right, 3=middle)
    Clickfinger,
}

impl std::fmt::Display for ClickMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ButtonAreas => write!(f, "Button Areas"),
            Self::Clickfinger => write!(f, "Clickfinger"),
        }
    }
}

impl ClickMethod {
    pub fn all() -> &'static [Self] {
        &[Self::ButtonAreas, Self::Clickfinger]
    }
}

/// Mouse warping behavior when focus changes between windows.
///
/// Controls whether the mouse pointer automatically moves to newly focused windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WarpMouseMode {
    /// Never warp the mouse (default)
    #[default]
    Off,
    /// Warp to window center when focus changes via keyboard
    CenterXY,
    /// Always warp to window center on any focus change
    CenterXYAlways,
}

impl std::fmt::Display for WarpMouseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::CenterXY => write!(f, "Center (Keyboard Only)"),
            Self::CenterXYAlways => write!(f, "Center (Always)"),
        }
    }
}

impl WarpMouseMode {
    pub fn all() -> &'static [Self] {
        &[Self::Off, Self::CenterXY, Self::CenterXYAlways]
    }
}

/// Behavior for centering the focused column in the viewport.
///
/// Controls when niri automatically scrolls to keep the focused column centered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum CenterFocusedColumn {
    /// Never auto-center (default)
    #[default]
    #[slint_index(default)]
    Never,
    /// Center only when column would be partially off-screen
    OnOverflow,
    /// Always keep focused column centered
    Always,
}

impl std::fmt::Display for CenterFocusedColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "Never"),
            Self::OnOverflow => write!(f, "On Overflow"),
            Self::Always => write!(f, "Always"),
        }
    }
}

impl CenterFocusedColumn {
    pub fn all() -> &'static [Self] {
        &[Self::Never, Self::OnOverflow, Self::Always]
    }

    /// Convert to KDL string representation
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Always => "always",
            Self::OnOverflow => "on-overflow",
        }
    }

    /// Parse from KDL string
    pub fn parse_kdl(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "never" => Some(Self::Never),
            "always" => Some(Self::Always),
            "on-overflow" | "on_overflow" => Some(Self::OnOverflow),
            _ => None,
        }
    }
}

/// Mapping of multi-finger taps to mouse buttons.
///
/// Defines which mouse button is triggered by 2-finger and 3-finger taps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum TapButtonMap {
    /// 1-finger=left, 2-finger=right, 3-finger=middle (default)
    #[default]
    #[slint_index(default)]
    LeftRightMiddle,
    /// 1-finger=left, 2-finger=middle, 3-finger=right
    LeftMiddleRight,
}

impl std::fmt::Display for TapButtonMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftRightMiddle => write!(f, "Left-Right-Middle"),
            Self::LeftMiddleRight => write!(f, "Left-Middle-Right"),
        }
    }
}

impl TapButtonMap {
    pub fn all() -> &'static [Self] {
        &[Self::LeftRightMiddle, Self::LeftMiddleRight]
    }
}

/// Monitor output transform (rotation and flip).
///
/// Controls the orientation and mirroring of display output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum Transform {
    /// No transformation (default)
    #[default]
    #[slint_index(default)]
    Normal,
    /// Rotate 90 degrees clockwise
    Rotate90,
    /// Rotate 180 degrees (upside down)
    Rotate180,
    /// Rotate 270 degrees clockwise (90 counter-clockwise)
    Rotate270,
    /// Flip horizontally (mirror)
    Flipped,
    /// Flip horizontally then rotate 90 degrees
    Flipped90,
    /// Flip horizontally then rotate 180 degrees
    Flipped180,
    /// Flip horizontally then rotate 270 degrees
    Flipped270,
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Rotate90 => write!(f, "90°"),
            Self::Rotate180 => write!(f, "180°"),
            Self::Rotate270 => write!(f, "270°"),
            Self::Flipped => write!(f, "Flipped"),
            Self::Flipped90 => write!(f, "Flipped + 90°"),
            Self::Flipped180 => write!(f, "Flipped + 180°"),
            Self::Flipped270 => write!(f, "Flipped + 270°"),
        }
    }
}

impl Transform {
    pub fn all() -> &'static [Self] {
        &[
            Self::Normal,
            Self::Rotate90,
            Self::Rotate180,
            Self::Rotate270,
            Self::Flipped,
            Self::Flipped90,
            Self::Flipped180,
            Self::Flipped270,
        ]
    }
}

/// Variable Refresh Rate (VRR/FreeSync/G-Sync) mode for monitors.
///
/// Controls adaptive sync behavior to reduce screen tearing and stuttering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, SlintIndex)]
pub enum VrrMode {
    /// VRR disabled (default)
    #[default]
    #[slint_index(default)]
    Off,
    /// VRR always enabled
    On,
    /// VRR enabled only when an application requests it
    OnDemand,
}

impl std::fmt::Display for VrrMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::On => write!(f, "On"),
            Self::OnDemand => write!(f, "On Demand"),
        }
    }
}

impl VrrMode {
    pub fn all() -> &'static [Self] {
        &[Self::Off, Self::On, Self::OnDemand]
    }
}

// ============================================================================
// GRADIENT TYPES
// ============================================================================

/// Color space for gradient interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSpace {
    #[default]
    Srgb,
    SrgbLinear,
    Oklab,
    Oklch,
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Srgb => write!(f, "sRGB"),
            Self::SrgbLinear => write!(f, "sRGB Linear"),
            Self::Oklab => write!(f, "Oklab"),
            Self::Oklch => write!(f, "Oklch"),
        }
    }
}

impl ColorSpace {
    pub fn all() -> &'static [Self] {
        &[Self::Srgb, Self::SrgbLinear, Self::Oklab, Self::Oklch]
    }

    /// Convert to KDL string representation
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Srgb => "srgb",
            Self::SrgbLinear => "srgb-linear",
            Self::Oklab => "oklab",
            Self::Oklch => "oklch",
        }
    }

    /// Parse from KDL string (base part, without hue modifier)
    pub fn from_kdl(s: &str) -> Option<Self> {
        match s.split_whitespace().next()? {
            "srgb" => Some(Self::Srgb),
            "srgb-linear" => Some(Self::SrgbLinear),
            "oklab" => Some(Self::Oklab),
            "oklch" => Some(Self::Oklch),
            _ => None,
        }
    }
}

/// Hue interpolation method for oklch color space gradients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HueInterpolation {
    #[default]
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

impl std::fmt::Display for HueInterpolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shorter => write!(f, "Shorter Hue"),
            Self::Longer => write!(f, "Longer Hue"),
            Self::Increasing => write!(f, "Increasing Hue"),
            Self::Decreasing => write!(f, "Decreasing Hue"),
        }
    }
}

impl HueInterpolation {
    pub fn all() -> &'static [Self] {
        &[Self::Shorter, Self::Longer, Self::Increasing, Self::Decreasing]
    }

    /// Convert to KDL string representation
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Shorter => "shorter hue",
            Self::Longer => "longer hue",
            Self::Increasing => "increasing hue",
            Self::Decreasing => "decreasing hue",
        }
    }

    /// Parse from KDL string
    pub fn from_kdl(s: &str) -> Option<Self> {
        if s.contains("shorter") {
            Some(Self::Shorter)
        } else if s.contains("longer") {
            Some(Self::Longer)
        } else if s.contains("increasing") {
            Some(Self::Increasing)
        } else if s.contains("decreasing") {
            Some(Self::Decreasing)
        } else {
            None
        }
    }
}

/// What the gradient position is relative to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientRelativeTo {
    #[default]
    Window,
    WorkspaceView,
}

impl std::fmt::Display for GradientRelativeTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Window => write!(f, "Window"),
            Self::WorkspaceView => write!(f, "Workspace View"),
        }
    }
}

impl GradientRelativeTo {
    pub fn all() -> &'static [Self] {
        &[Self::Window, Self::WorkspaceView]
    }

    /// Convert to KDL string representation
    pub fn to_kdl(&self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::WorkspaceView => "workspace-view",
        }
    }

    /// Parse from KDL string
    pub fn from_kdl(s: &str) -> Option<Self> {
        match s {
            "window" => Some(Self::Window),
            "workspace-view" => Some(Self::WorkspaceView),
            _ => None,
        }
    }
}

/// A gradient color definition for niri visual elements
///
/// Supports CSS-style linear gradients with color space interpolation options.
/// Used for focus-ring, border, tab-indicator, and insert-hint.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    /// Starting color
    pub from: Color,
    /// Ending color
    pub to: Color,
    /// Angle in degrees (0-360, default 180 = top to bottom)
    pub angle: i32,
    /// What the gradient is relative to
    pub relative_to: GradientRelativeTo,
    /// Color space for interpolation
    pub color_space: ColorSpace,
    /// Hue interpolation (only used with Oklch color space)
    pub hue_interpolation: Option<HueInterpolation>,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            from: Color::from_hex("#7fc8ff").unwrap_or_default(),
            to: Color::from_hex("#505050").unwrap_or_default(),
            angle: 180,
            relative_to: GradientRelativeTo::Window,
            color_space: ColorSpace::Srgb,
            hue_interpolation: None,
        }
    }
}

/// Either a solid color or a gradient
#[derive(Debug, Clone, PartialEq)]
pub enum ColorOrGradient {
    Color(Color),
    Gradient(Gradient),
}

impl Default for ColorOrGradient {
    fn default() -> Self {
        Self::Color(Color::default())
    }
}

impl From<Color> for ColorOrGradient {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl From<Gradient> for ColorOrGradient {
    fn from(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
}

impl ColorOrGradient {
    /// Returns the primary color for UI display.
    /// For gradients, returns the "from" color.
    pub fn primary_color(&self) -> &Color {
        match self {
            ColorOrGradient::Color(c) => c,
            ColorOrGradient::Gradient(g) => &g.from,
        }
    }

    /// Returns a mutable reference to the primary color.
    /// For gradients, returns a mutable reference to the "from" color.
    pub fn primary_color_mut(&mut self) -> &mut Color {
        match self {
            ColorOrGradient::Color(c) => c,
            ColorOrGradient::Gradient(g) => &mut g.from,
        }
    }

    /// Sets this to a solid color (converts gradient to color).
    pub fn set_color(&mut self, color: Color) {
        *self = ColorOrGradient::Color(color);
    }

    /// Returns the hex representation of the primary color.
    pub fn to_hex(&self) -> String {
        self.primary_color().to_hex()
    }

    /// Returns true if this is a gradient, false if solid color.
    pub fn is_gradient(&self) -> bool {
        matches!(self, ColorOrGradient::Gradient(_))
    }
}

// ============================================================================
// POINTER DEVICE TRAIT
// ============================================================================

/// Trait for pointer input devices that share common settings.
///
/// This trait abstracts the common fields shared between mouse, touchpad,
/// trackpoint, and trackball settings, enabling generic parsing code.
///
/// All pointer devices support these core settings:
/// - `off`: Disable the device entirely
/// - `natural_scroll`: Invert scroll direction
/// - `left_handed`: Swap left/right buttons
/// - `middle_emulation`: Emulate middle button with left+right click
/// - `scroll_button_lock`: Lock scroll button state
/// - `accel_speed`: Pointer acceleration speed (-1.0 to 1.0)
/// - `accel_profile`: Pointer acceleration profile (adaptive/flat)
/// - `scroll_method`: How scrolling works (two-finger, edge, on-button-down, none)
/// - `scroll_button`: Which button triggers on-button-down scrolling
pub trait PointerDeviceSettings {
    fn off(&self) -> bool;
    fn set_off(&mut self, value: bool);

    fn natural_scroll(&self) -> bool;
    fn set_natural_scroll(&mut self, value: bool);

    fn left_handed(&self) -> bool;
    fn set_left_handed(&mut self, value: bool);

    fn middle_emulation(&self) -> bool;
    fn set_middle_emulation(&mut self, value: bool);

    fn scroll_button_lock(&self) -> bool;
    fn set_scroll_button_lock(&mut self, value: bool);

    fn accel_speed(&self) -> f64;
    fn set_accel_speed(&mut self, value: f64);

    fn accel_profile(&self) -> AccelProfile;
    fn set_accel_profile(&mut self, value: AccelProfile);

    fn scroll_method(&self) -> ScrollMethod;
    fn set_scroll_method(&mut self, value: ScrollMethod);

    fn scroll_button(&self) -> Option<i32>;
    fn set_scroll_button(&mut self, value: Option<i32>);
}

/// Navigation category for UI sidebar.
///
/// This is distinct from `config::SettingsCategory` which tracks dirty state
/// for all 25 config file categories. This enum only includes the 9 main
/// navigation categories shown in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationCategory {
    Appearance,
    Behavior,
    Keyboard,
    Mouse,
    Touchpad,
    Outputs,
    Animations,
    Cursor,
    Overview,
}

impl NavigationCategory {
    /// Returns the human-readable display label for this category.
    ///
    /// Used for sidebar navigation and category headers in the UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Behavior => "Behavior",
            Self::Keyboard => "Keyboard",
            Self::Mouse => "Mouse",
            Self::Touchpad => "Touchpad",
            Self::Outputs => "Displays",
            Self::Animations => "Animations",
            Self::Cursor => "Cursor",
            Self::Overview => "Overview",
        }
    }

    /// Returns a static slice containing all navigation categories in display order.
    ///
    /// Used to populate the sidebar navigation.
    pub fn all() -> &'static [NavigationCategory] {
        &[
            Self::Appearance,
            Self::Behavior,
            Self::Keyboard,
            Self::Mouse,
            Self::Touchpad,
            Self::Outputs,
            Self::Animations,
            Self::Cursor,
            Self::Overview,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex_6digit() {
        let color = Color::from_hex("#7fc8ff").unwrap();
        assert_eq!(color.r, 0x7f);
        assert_eq!(color.g, 0xc8);
        assert_eq!(color.b, 0xff);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_from_hex_without_hash() {
        let color = Color::from_hex("7fc8ff").unwrap();
        assert_eq!(color.r, 0x7f);
        assert_eq!(color.g, 0xc8);
        assert_eq!(color.b, 0xff);
    }

    #[test]
    fn test_color_from_hex_shorthand_3digit() {
        // #abc should expand to #aabbcc
        let color = Color::from_hex("#abc").unwrap();
        assert_eq!(color.r, 0xaa);
        assert_eq!(color.g, 0xbb);
        assert_eq!(color.b, 0xcc);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_from_hex_shorthand_4digit() {
        // #abcd should expand to #aabbccdd
        let color = Color::from_hex("#abcd").unwrap();
        assert_eq!(color.r, 0xaa);
        assert_eq!(color.g, 0xbb);
        assert_eq!(color.b, 0xcc);
        assert_eq!(color.a, 0xdd);
    }

    #[test]
    fn test_color_from_hex_uppercase() {
        let color = Color::from_hex("#AABBCC").unwrap();
        assert_eq!(color.r, 0xaa);
        assert_eq!(color.g, 0xbb);
        assert_eq!(color.b, 0xcc);
    }

    #[test]
    fn test_color_from_hex_invalid_length() {
        assert!(Color::from_hex("#ab").is_none());
        assert!(Color::from_hex("#abcde").is_none());
        assert!(Color::from_hex("#abcdefgh").is_none());
    }

    #[test]
    fn test_color_from_hex_invalid_chars() {
        assert!(Color::from_hex("#gggggg").is_none());
        assert!(Color::from_hex("#12345z").is_none());
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color {
            r: 127,
            g: 200,
            b: 255,
            a: 255,
        };
        assert_eq!(color.to_hex(), "#7fc8ff");
    }

    #[test]
    fn test_color_with_alpha() {
        let color = Color::from_hex("#7fc8ff80").unwrap();
        assert_eq!(color.a, 0x80);
        assert_eq!(color.to_hex(), "#7fc8ff80");
    }

    #[test]
    fn test_color_roundtrip() {
        let original = Color {
            r: 0x12,
            g: 0x34,
            b: 0x56,
            a: 0x78,
        };
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_parse_hex_digit() {
        assert_eq!(parse_hex_digit(b'0'), Some(0));
        assert_eq!(parse_hex_digit(b'9'), Some(9));
        assert_eq!(parse_hex_digit(b'a'), Some(10));
        assert_eq!(parse_hex_digit(b'f'), Some(15));
        assert_eq!(parse_hex_digit(b'A'), Some(10));
        assert_eq!(parse_hex_digit(b'F'), Some(15));
        assert_eq!(parse_hex_digit(b'g'), None);
        assert_eq!(parse_hex_digit(b'!'), None);
    }

    #[test]
    fn test_center_focused_column_kdl_roundtrip() {
        for mode in [
            CenterFocusedColumn::Never,
            CenterFocusedColumn::Always,
            CenterFocusedColumn::OnOverflow,
        ] {
            let kdl = mode.to_kdl();
            let parsed = CenterFocusedColumn::parse_kdl(kdl).unwrap();
            assert_eq!(mode, parsed);
        }
    }
}
