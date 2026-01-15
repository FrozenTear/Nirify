# Missing Features Implementation Plan
## niri-settings-rust Complete Configuration Coverage

This document provides a structured implementation plan for all missing niri configuration features, organized by priority, complexity, and dependencies.

---

## Version Requirements Summary

| Version | Notable Features Added |
|---------|----------------------|
| 0.1.5 | VRR support |
| 0.1.6 | `is-active-in-column`, `at-startup`, `open-on-workspace`, named workspaces |
| 0.1.7 | Fractional values for gaps/struts |
| 0.1.8 | Gradient color spaces, `focus-follows-mouse.max-scroll-amount` |
| 0.1.9 | `preset-window-heights`, monitor serial/model matching, `always-center-single-column` |
| 0.1.10 | `scroll-button`, insert-hint |
| 25.01 | `empty-workspace-above-first`, `is-floating`, `default-window-height` |
| 25.02 | Shadows, tab-indicator, `default-column-display`, `scroll-factor`, `drag-lock`, `is-window-cast-target`, tablet calibration, clipboard disable-primary |
| 25.05 | `mod-key`, `mod-key-nested`, `numlock`, `drag`, `is-urgent`, `tiled-state`, background-color, focus-at-startup, backdrop-color |
| 25.08 | `scroll-button-lock`, `spawn-sh-at-startup`, `hotkey-overlay.hide-not-bound`, `config-notification.disable-failed` |
| 25.11 | `open-maximized-to-edges`, custom modes, modelines, per-output hot-corners, per-output layout, touch calibration-matrix |

**Current niri version: 25.05** (May 2025)
**Cutting-edge features require: 25.08 or 25.11**

---

## Implementation Phases

### PHASE 1: Essential Input Extensions
**Priority: HIGH** | **Complexity: Simple-Medium** | **Dependencies: None**

These are commonly requested features that enhance basic input device functionality.

#### 1.1 Mouse/Touchpad Scroll Button (v0.1.10+)
**Complexity: Simple** | **File: input/mouse.kdl, input/touchpad.kdl**

- `scroll-button` - Button code for on-button-down scrolling
- `scroll-button-lock` (v25.08+) - Toggle scrolling without holding button

**Data Model:**
```rust
// Add to MouseSettings and TouchpadSettings
pub scroll_button: Option<u32>,      // Button code (e.g., BTN_SIDE = 275)
pub scroll_button_lock: bool,        // Since 25.08
```

**UI Design:**
- Mouse/Touchpad page → "Scrolling" section
- Checkbox: "Use button for scrolling" (enables scroll-button)
- Number input: "Button code" (with helper: "Middle button = 274, Side = 275")
- Checkbox: "Lock scroll mode" (toggle without holding) - requires v25.08

**Warnings:**
- Show version warning if `scroll-button-lock` enabled on niri < 25.08
- Common button codes reference in tooltip

---

#### 1.2 Device Disable (All Versions)
**Complexity: Simple** | **File: input/*.kdl**

- `off` directive - Completely disables input device

**Data Model:**
```rust
// Add to MouseSettings, TouchpadSettings, KeyboardSettings
pub device_enabled: bool,  // false = add "off" directive
```

**UI Design:**
- Each input device page → Top of page
- Prominent toggle: "Enable this device"
- When disabled, gray out all other settings
- Warning dialog: "Disabling keyboard may lock you out!"

**Warnings:**
- Critical warning for keyboard disable
- Suggest testing with another input method available

---

#### 1.3 Keyboard XKB File (v25.02+)
**Complexity: Simple** | **File: input/keyboard.kdl**

- `xkb.file` - Direct path to .xkb keymap file (overrides layout/variant/options)

**Data Model:**
```rust
// Add to KeyboardSettings
pub xkb_file: Option<String>,  // Path to .xkb file
```

**UI Design:**
- Keyboard page → "Advanced" collapsible section
- File picker: "Custom XKB keymap file"
- Help text: "Overrides layout, variant, and options"
- Browse button with .xkb file filter
- Clear button to remove custom file

**Validation:**
- Check file exists and is readable
- Warn if both xkb.file and layout/variant set (file takes precedence)

---

### PHASE 2: Advanced Input Devices
**Priority: MEDIUM** | **Complexity: Medium** | **Dependencies: IPC for device detection**

Support for specialized input devices beyond mouse/touchpad/keyboard.

#### 2.1 Trackpoint & Trackball Settings
**Complexity: Medium** | **File: input/trackpoint.kdl, input/trackball.kdl**

**Features:**
- Both support: scroll-method, accel-speed, accel-profile, natural-scroll, left-handed
- scroll-method includes "on-button-down" option

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TrackpointSettings {
    pub device_enabled: bool,
    pub scroll_method: ScrollMethod,  // Add OnButtonDown variant
    pub scroll_button: Option<u32>,
    pub scroll_button_lock: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub natural_scroll: bool,
    pub left_handed: bool,
}

// Similar for TrackballSettings
```

**UI Design:**
- New pages: "Trackpoint" and "Trackball" in Input section
- Similar layout to Mouse/Touchpad
- Auto-detect if device present (via IPC or libinput)
- Hide page if device not detected (show in "All Devices" view)

**Dependencies:**
- Device detection via niri IPC or libinput queries
- May need to list available input devices

---

#### 2.2 Tablet Settings (v25.02+)
**Complexity: Medium** | **File: input/tablet.kdl**

**Features:**
- map-to-output
- left-handed
- calibration-matrix (6 floats)

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TabletSettings {
    pub device_enabled: bool,
    pub map_to_output: Option<String>,
    pub left_handed: bool,
    pub calibration_matrix: Option<[f64; 6]>,  // Since 25.02
}
```

**UI Design:**
- New page: "Tablet" in Input section (or Advanced Input)
- Dropdown: "Map to output" (list available monitors)
- Checkbox: "Left-handed mode"
- Advanced: Calibration matrix (6 number inputs with reset button)
- Helper: "Calibration matrix format: [a b c d e f]"

**Warnings:**
- Calibration is advanced - provide "Reset to default" button
- Explain matrix format (link to libinput docs)

---

#### 2.3 Touch Device Settings (v25.11+)
**Complexity: Medium** | **File: input/touch.kdl**

**Features:**
- map-to-output
- calibration-matrix (since 25.11)

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TouchSettings {
    pub device_enabled: bool,
    pub map_to_output: Option<String>,
    pub calibration_matrix: Option<[f64; 6]>,  // Since 25.11
}
```

**UI Design:**
- Similar to Tablet settings
- May combine with Tablet in "Touch & Stylus" page

**Version Requirements:**
- Show warning if calibration-matrix used on niri < 25.11

---

#### 2.4 Power Key Handling
**Complexity: Simple** | **File: behavior.kdl or misc.kdl**

- `disable-power-key-handling` - Allows external handling

**Data Model:**
```rust
// Add to BehaviorSettings or MiscSettings
pub disable_power_key_handling: bool,
```

**UI Design:**
- Behavior page → "Power Button" section
- Checkbox: "Let system handle power button"
- Help: "Disable niri's power button handling for custom scripts"

---

### PHASE 3: Layout Enhancements
**Priority: HIGH** | **Complexity: Medium** | **Dependencies: None**

Missing layout features that improve window management.

#### 3.1 Preset Window Heights (v0.1.9+)
**Complexity: Simple** | **File: appearance.kdl or advanced/layout-extras.kdl**

- `preset-window-heights` - Like preset-column-widths but for heights

**Data Model:**
```rust
// Add to LayoutExtrasSettings
pub preset_window_heights: Vec<PresetHeight>,

#[derive(Debug, Clone, PartialEq)]
pub enum PresetHeight {
    Proportion(f32),  // Fraction of output height
    Fixed(i32),       // Logical pixels
}
```

**UI Design:**
- Appearance page → "Window Sizing" section (or Layout Extras)
- List of preset heights (editable)
- Add/Remove buttons
- Toggle between Proportion and Fixed for each
- Default: 1/3, 1/2, 2/3

**KDL Generation:**
```kdl
layout {
    preset-window-heights {
        proportion 0.33333
        proportion 0.5
        proportion 0.66667
        fixed 1000
    }
}
```

---

#### 3.2 Default Column Display (v25.02+)
**Complexity: Simple** | **File: appearance.kdl**

- `default-column-display` - "normal" or "tabbed"

**Data Model:**
```rust
// Add to BehaviorSettings or new LayoutSettings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDisplay {
    Normal,
    Tabbed,
}

pub default_column_display: ColumnDisplay,
```

**UI Design:**
- Appearance page → "Columns" section
- Radio buttons or Dropdown: "Default column display"
  - "Normal" (stacked vertically)
  - "Tabbed" (side-by-side tabs)
- Help: "How new windows appear in columns (since v25.02)"

---

#### 3.3 Gradient Support (v0.1.8+)
**Complexity: Complex** | **Files: appearance.kdl, advanced/layout-extras.kdl**

**Features:**
- Gradients for: focus-ring, border, tab-indicator (active/inactive/urgent)
- insert-hint already supports gradients
- Properties: angle, from/to colors, color-space, relative-to

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ColorValue {
    Solid(Color),
    Gradient(Gradient),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub angle: i32,                    // Degrees (0-360)
    pub from: Color,
    pub to: Color,
    pub color_space: ColorSpace,       // Since 0.1.8
    pub relative_to: RelativeTo,       // workspace-view or default
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    SrgbLinear,
    Oklab,
    Oklch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTo {
    Default,  // Per-window
    WorkspaceView,
}
```

**UI Updates:**
- Appearance page → All color pickers
- Replace simple color picker with:
  - Radio: "Solid Color" / "Gradient"
  - If gradient:
    - Slider: Angle (0-360°)
    - Color picker: From color
    - Color picker: To color
    - Dropdown: Color space (Srgb, Oklab, Oklch)
    - Checkbox: "Relative to workspace view"
- Visual preview of gradient

**Complexity Justification:**
- Need new gradient color picker widget
- Complex KDL generation with nested properties
- Preview rendering in UI

**KDL Generation:**
```kdl
layout {
    focus-ring {
        active-gradient {
            angle 45
            from "#7fc8ff"
            to "#eb6f92"
            in "oklch"
            relative-to "workspace-view"
        }
    }
}
```

---

#### 3.4 Mod Key Nested (v25.05+)
**Complexity: Simple** | **File: behavior.kdl**

- `mod-key-nested` - Modifier for nested window mode

**Data Model:**
```rust
// Add to BehaviorSettings
pub mod_key_nested: Option<ModKey>,  // Since 25.05
```

**UI Design:**
- Behavior page → "Modifier Keys" section
- Dropdown: "Nested window modifier"
- Options: None, Super, Alt, Ctrl, Shift
- Help: "Modifier key for nested window interactions"

---

### PHASE 4: Visual & Notification Settings
**Priority: MEDIUM** | **Complexity: Simple** | **Dependencies: None**

Fine-tuning for notifications and visual effects.

#### 4.1 Workspace Shadow in Overview (v25.05+)
**Complexity: Simple** | **File: overview.kdl**

- `overview.workspace-shadow` - Shadow config for workspaces in overview

**Data Model:**
```rust
// Add to OverviewSettings
pub workspace_shadow: Option<ShadowSettings>,  // Reuse existing ShadowSettings
```

**UI Design:**
- Overview page → "Workspace Shadow" section
- Checkbox: "Enable workspace shadows in overview"
- If enabled, show shadow controls (reuse from window shadows):
  - Softness, spread, offset, color

**KDL Generation:**
```kdl
overview {
    workspace-shadow {
        softness 30
        spread 10
        color "#00000050"
    }
}
```

---

#### 4.2 Hotkey Overlay Options (v25.08+)
**Complexity: Simple** | **File: misc.kdl**

- `hotkey-overlay.skip-at-startup` (existing)
- `hotkey-overlay.hide-not-bound` (v25.08+) - Hide unbound actions

**Data Model:**
```rust
// Add to MiscSettings
pub hotkey_overlay_hide_not_bound: bool,  // Since 25.08
```

**UI Design:**
- Miscellaneous page → "Hotkey Overlay" section
- Checkbox: "Skip hotkey overlay at startup" (existing)
- Checkbox: "Hide unbound actions in overlay" (v25.08+)

---

#### 4.3 Config Notification (v25.08+)
**Complexity: Simple** | **File: misc.kdl**

- `config-notification.disable-failed` - Disable config failure notifications

**Data Model:**
```rust
// Add to MiscSettings
pub config_notification_disable_failed: bool,  // Since 25.08
```

**UI Design:**
- Miscellaneous page → "Notifications" section
- Checkbox: "Disable config error notifications"
- Warning: "You won't be notified of config errors!"

---

#### 4.4 Tab Indicator Urgent Color
**Complexity: Simple** | **File: advanced/layout-extras.kdl**

- Add `urgent-color` to tab-indicator (complement to active/inactive)

**Data Model:**
```rust
// Add to TabIndicatorSettings
pub urgent_color: ColorValue,  // Supports gradient
```

**UI Design:**
- Layout Extras page → Tab Indicator section
- Color picker: "Urgent color" (for windows requesting attention)

---

### PHASE 5: Output/Display Advanced
**Priority: MEDIUM** | **Complexity: Medium-Complex** | **Dependencies: Monitor detection**

Advanced display configuration for power users.

#### 5.1 Custom Modes & Modelines (v25.11+)
**Complexity: Complex** | **File: outputs.kdl**

**Features:**
- `mode custom=true` - Non-standard modes
- `modeline` - CVT/GTF modeline specification

**Data Model:**
```rust
// Add to OutputConfig
pub mode_custom: bool,                    // Since 25.11
pub modeline: Option<String>,            // Since 25.11
```

**UI Design:**
- Outputs page → Per-monitor settings → "Advanced" section
- Checkbox: "Enable custom mode" (big warning!)
- If custom:
  - Text input: "Modeline" (for experts)
  - Help: Link to modeline calculator tools
- Warning dialog:
  ```
  ⚠️ DANGER: Custom modes may damage your monitor!
  Only use if you know what you're doing.
  Incorrect settings can permanently damage hardware.
  ```

**Safety:**
- Require confirmation checkbox: "I understand the risks"
- Show warning on every save
- Backup previous mode for quick rollback

---

#### 5.2 Per-Output Hot Corners (v25.11+)
**Complexity: Medium** | **File: outputs.kdl**

- `hot-corners` - Override global hot-corners per output

**Data Model:**
```rust
// Add to OutputConfig
pub hot_corners: Option<HotCorners>,  // None = use global
```

**UI Design:**
- Outputs page → Per-monitor settings
- Section: "Hot Corners"
- Radio: "Use global settings" / "Override for this output"
- If override: Show hot-corner checkboxes (reuse from global)

---

#### 5.3 Per-Output Layout Overrides (v25.11+)
**Complexity: Complex** | **File: outputs.kdl**

- `layout { }` block per output - Override global layout settings

**Data Model:**
```rust
// Add to OutputConfig
pub layout_override: Option<LayoutOverride>,

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutOverride {
    pub gaps_inner: Option<f32>,
    pub gaps_outer: Option<f32>,
    pub strut_left: Option<f32>,
    // ... all layout settings except empty-workspace-above-first and insert-hint
}
```

**UI Design:**
- Outputs page → Per-monitor settings → "Layout Override" section
- Expandable: "Customize layout for this monitor"
- Show subset of appearance/layout settings
- "Reset to global" button for each override

**Complexity:**
- Need to duplicate layout UI per output
- Merge logic: output-specific > global
- Clear indication which settings are overridden

---

### PHASE 6: Window Rules Extensions
**Priority: HIGH** | **Complexity: Medium** | **Dependencies: Phase 3 (gradients)**

Complete window rules coverage.

#### 6.1 New Window Rule Matchers
**Complexity: Simple-Medium** | **File: advanced/window-rules.kdl**

**Matchers to add:**
- `is-active` (all versions) - Window has active border/focus ring
- `is-focused` (all versions) - Window has keyboard focus
- `is-active-in-column` (v0.1.6+) - Last-focused window in column
- `is-window-cast-target` (v25.02+) - Window in active screencast
- `is-urgent` (v25.05+) - Window requesting attention
- `at-startup` (v0.1.6+) - First 60 seconds after launch

**Data Model:**
```rust
// Add to WindowRuleMatch
pub is_active: Option<bool>,
pub is_focused: Option<bool>,
pub is_active_in_column: Option<bool>,      // Since 0.1.6
pub is_window_cast_target: Option<bool>,    // Since 25.02
pub is_urgent: Option<bool>,                // Since 25.05
pub at_startup: Option<bool>,               // Since 0.1.6
```

**UI Design:**
- Window Rules page → Match criteria section
- Add checkboxes for boolean matchers:
  - "Active window"
  - "Focused window"
  - "Active in column"
  - "Being screen-shared"
  - "Requesting attention (urgent)"
  - "During startup (first 60s)"
- Version warnings for newer matchers

---

#### 6.2 New Window Rule Properties
**Complexity: Medium** | **File: advanced/window-rules.kdl**

**Opening Properties:**
- `default-window-height` (v25.01+) - Initial height
- `open-maximized-to-edges` (v25.11+) - Maximize to screen edges
- `default-floating-position` - Floating position with anchoring

**Dynamic Properties:**
- `scroll-factor` (v25.02+) - Per-window scroll multiplier
- `draw-border-with-background` - Border rendering override
- `min-width`, `max-width`, `min-height`, `max-height` - Size constraints
- `clip-to-geometry` (v0.1.6+) - Clip to visual geometry
- `tiled-state` (v25.05+) - Inform window it's tiled
- `baba-is-float` (v25.02+) - April Fools' float animation

**Per-Rule Styling:**
- `focus-ring` - Override with gradient support
- `border` - Override with gradient support
- `shadow` - Override shadow settings
- `tab-indicator` - Override tab indicator

**Data Model:**
```rust
// Add to WindowRule
pub default_window_height: Option<WindowHeight>,  // Since 25.01
pub open_maximized_to_edges: bool,                // Since 25.11
pub default_floating_position: Option<FloatingPosition>,

pub scroll_factor: Option<f64>,                   // Since 25.02
pub draw_border_with_background: Option<bool>,
pub min_width: Option<i32>,
pub max_width: Option<i32>,
pub min_height: Option<i32>,
pub max_height: Option<i32>,
pub clip_to_geometry: Option<bool>,               // Since 0.1.6
pub tiled_state: Option<bool>,                    // Since 25.05
pub baba_is_float: bool,                          // Since 25.02

// Styling overrides (use existing ColorValue with gradient support)
pub focus_ring_override: Option<FocusRingOverride>,
pub border_override: Option<BorderOverride>,
pub shadow_override: Option<ShadowSettings>,
pub tab_indicator_override: Option<TabIndicatorSettings>,

#[derive(Debug, Clone, PartialEq)]
pub enum WindowHeight {
    Proportion(f32),
    Fixed(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingPosition {
    pub x: FloatingCoord,
    pub y: FloatingCoord,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatingCoord {
    Pixels(i32),
    Percent(f32),
    Anchor(Anchor),  // left, center, right, top, bottom
}
```

**UI Design:**
- Window Rules page → Rule properties (collapsible sections)
- **Opening Behavior:**
  - Default window height (proportion/fixed)
  - Checkbox: "Maximize to screen edges"
  - Floating position picker (visual or coordinate input)
- **Dynamic Properties:**
  - Scroll factor slider
  - Size constraints (min/max width/height)
  - Advanced checkboxes (clip-to-geometry, tiled-state, etc.)
- **Styling Overrides:**
  - Expandable: "Override focus ring"
  - Expandable: "Override border"
  - Expandable: "Override shadow"
  - Each shows full styling controls (reuse from appearance)

**Complexity:**
- Many properties to add
- Complex UI (multiple collapsible sections)
- Gradient support for per-rule styling
- Floating position needs visual picker

---

### PHASE 7: Named Workspaces & Layer Rules
**Priority: MEDIUM** | **Complexity: Medium-Complex** | **Dependencies: None**

Complete configuration coverage with advanced workspace and layer features.

#### 7.1 Named Workspaces (v0.1.6+)
**Complexity: Medium** | **File: workspaces.kdl (new)**

**Features:**
- Declare named workspaces that always exist
- `open-on-output` - Pin to specific monitor (v0.1.9+ for serial/model)
- `layout { }` - Per-workspace layout overrides (v25.11+)

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct NamedWorkspace {
    pub name: String,
    pub open_on_output: Option<String>,
    pub layout_override: Option<LayoutOverride>,  // Since 25.11
}

// Add to Settings
pub named_workspaces: Vec<NamedWorkspace>,
```

**UI Design:**
- New page: "Workspaces" in main navigation
- List of named workspaces with add/remove
- Per workspace:
  - Text input: "Name"
  - Dropdown: "Default output" (None or monitor list)
  - Expandable: "Layout overrides" (similar to per-output)
- Drag to reorder

**KDL Generation:**
```kdl
workspace "browser"
workspace "chat" {
    open-on-output "Dell Inc. DELL U2415 Some Serial"
}
workspace "coding" {
    layout {
        gaps 8
        center-focused-column "always"
    }
}
```

**Integration:**
- Window rules can use `open-on-workspace "name"`
- Need to list named workspaces in window rule dropdown

---

#### 7.2 Layer Rules (All Versions)
**Complexity: Medium** | **File: advanced/layer-rules.kdl**

**Features:**
- Match by `namespace` (regex) and `at-startup`
- Properties:
  - `block-out-from` (screencast/screen-capture)
  - `opacity` (0.0-1.0)
  - `shadow` (v25.02+)
  - `geometry-corner-radius` (v25.02+)
  - `place-within-backdrop` (v25.05+)
  - `baba-is-float` (v25.05+)

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRuleMatch {
    pub namespace: Option<String>,      // Regex
    pub at_startup: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerRule {
    pub id: u32,
    pub name: String,
    pub matches: Vec<LayerRuleMatch>,
    pub block_out_from: Option<BlockOutFrom>,
    pub opacity: Option<f32>,
    pub shadow: Option<ShadowSettings>,              // Since 25.02
    pub geometry_corner_radius: Option<i32>,         // Since 25.02
    pub place_within_backdrop: bool,                 // Since 25.05
    pub baba_is_float: bool,                         // Since 25.05
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOutFrom {
    Screencast,
    ScreenCapture,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerRulesSettings {
    pub rules: Vec<LayerRule>,
    pub next_id: u32,
}

// Add to Settings
pub layer_rules: LayerRulesSettings,
```

**UI Design:**
- New page: "Layer Rules" in Advanced section
- Similar to Window Rules page:
  - List of layer rules with add/edit/delete
  - Per rule:
    - Name input
    - Match criteria (namespace regex, at-startup)
    - Properties (opacity, shadow, etc.)
- Helper: "Run `niri msg layers` to see layer namespaces"
- Common examples:
  - Block notifications from screencasts
  - Put wallpaper in overview backdrop

**KDL Generation:**
```kdl
layer-rule {
    match namespace="^notifications$"
    block-out-from "screencast"
}

layer-rule {
    match namespace="^wallpaper$"
    place-within-backdrop true
}
```

---

#### 7.3 Recent Windows Tracking
**Complexity: Unknown** | **File: TBD**

**Status:** Need to research if this is a configuration option or runtime state.

**Research Needed:**
- Is there a config option for recent windows?
- Or is this only accessible via IPC/keybindings?
- Check niri wiki for "recent-windows" configuration

**Action:** Defer until research complete. May not be a config option.

---

### PHASE 8: Per-Animation Configuration
**Priority: LOW** | **Complexity: Complex** | **Dependencies: None**

Fine-grained control over individual animations.

#### 8.1 Individual Animation Overrides
**Complexity: Complex** | **File: animations.kdl**

**Current:** Only global `off` and `slowdown`
**Missing:** Per-animation curves, durations, and spring parameters

**Animations to Configure:**
- `workspace-switch` (spring)
- `window-open` (easing)
- `window-close` (easing)
- `horizontal-view-movement` (spring)
- `window-movement` (spring)
- `window-resize` (spring)
- `config-notification-open-close` (spring)
- `exit-confirmation-open-close` (spring)
- `screenshot-ui-open` (easing)
- `overview-open-close` (spring)
- `recent-windows-close` (spring)

**Data Model:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationConfig {
    Off,
    Spring {
        damping_ratio: f64,  // 0.1-10.0
        stiffness: i32,
        epsilon: f64,
    },
    Easing {
        duration_ms: i32,
        curve: EasingCurve,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingCurve {
    EaseOutQuad,
    EaseOutCubic,
    EaseOutExpo,
    Linear,
    CubicBezier(f64, f64, f64, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetailedAnimationSettings {
    pub global_enabled: bool,
    pub global_slowdown: f64,

    // Per-animation overrides (None = use defaults)
    pub workspace_switch: Option<AnimationConfig>,
    pub window_open: Option<AnimationConfig>,
    pub window_close: Option<AnimationConfig>,
    pub horizontal_view_movement: Option<AnimationConfig>,
    pub window_movement: Option<AnimationConfig>,
    pub window_resize: Option<AnimationConfig>,
    pub config_notification_open_close: Option<AnimationConfig>,
    pub exit_confirmation_open_close: Option<AnimationConfig>,
    pub screenshot_ui_open: Option<AnimationConfig>,
    pub overview_open_close: Option<AnimationConfig>,
    pub recent_windows_close: Option<AnimationConfig>,
}
```

**UI Design:**
- Animations page → "Advanced" expandable section
- List of all animations with override controls
- Per animation:
  - Checkbox: "Override"
  - If override:
    - If spring: Sliders for damping-ratio, stiffness, epsilon
    - If easing: Duration input, curve dropdown
- "Reset to defaults" button
- Visual preview of animation curves (stretch goal)

**KDL Generation:**
```kdl
animations {
    slowdown 1.0

    workspace-switch {
        spring damping-ratio=1.0 stiffness=1000 epsilon=0.0001
    }

    window-open {
        easing duration-ms=150 curve="ease-out-expo"
    }

    window-close {
        off
    }
}
```

**Complexity Justification:**
- 11 different animations to configure
- Two animation types (spring vs easing) with different parameters
- Complex UI with many inputs
- Curve preview would be nice but complex

---

## Category Organization Recommendations

### Current Categories (from CLAUDE.md)
- Appearance
- Behavior
- Keyboard
- Mouse
- Touchpad
- Displays (Outputs)
- Animations
- Cursor
- Overview
- Advanced (Gestures, Window Rules, Layout Extras, Misc)

### Proposed Reorganization

#### Core Settings
1. **Appearance** (appearance.kdl)
   - Gaps, focus ring, border, corner radius, background
   - Add: default-column-display, preset-window-heights
   - Add: Gradient support for all colors

2. **Behavior** (behavior.kdl)
   - Focus, workspace layout, struts, mod-key, mod-key-nested
   - Add: disable-power-key-handling

3. **Input Devices** (Sidebar subsection)
   - Keyboard (input/keyboard.kdl)
     - Add: xkb.file, device disable
   - Mouse (input/mouse.kdl)
     - Add: scroll-button, scroll-button-lock, device disable
   - Touchpad (input/touchpad.kdl)
     - Add: scroll-button, scroll-button-lock, device disable
   - **NEW:** Trackpoint (input/trackpoint.kdl)
   - **NEW:** Trackball (input/trackball.kdl)
   - **NEW:** Tablet & Stylus (input/tablet.kdl)
   - **NEW:** Touch Screen (input/touch.kdl)

#### Display & Visual
4. **Displays** (outputs.kdl)
   - Monitor configuration
   - Add: custom modes, modelines (with warnings)
   - Add: per-output hot-corners, per-output layout

5. **Animations** (animations.kdl)
   - Global enable/slowdown
   - Add: Per-animation configuration (in Advanced section)

6. **Cursor** (cursor.kdl)
   - No changes

7. **Overview** (overview.kdl)
   - Add: workspace-shadow

#### Workspace & Windows
8. **NEW: Workspaces** (workspaces.kdl)
   - Named workspaces list
   - Per-workspace settings

9. **Window Rules** (advanced/window-rules.kdl)
   - Move from Advanced to main navigation (commonly used)
   - Add: New matchers and properties

#### Advanced
10. **Layout Extras** (advanced/layout-extras.kdl)
    - Shadow, tab-indicator (add urgent-color), insert-hint
    - Preset widths/heights

11. **Gestures** (advanced/gestures.kdl)
    - Hot corners, DND edge scroll

12. **NEW: Layer Rules** (advanced/layer-rules.kdl)
    - Layer-shell surface rules

13. **Miscellaneous** (advanced/misc.kdl)
    - Screenshot path, CSD, clipboard, hotkey overlay
    - Add: config-notification.disable-failed

### Sidebar Organization
```
Search...
─────────────
Appearance
Behavior
Workspaces       ← NEW
─────────────
▸ Input Devices  ← Expandable group
  - Keyboard
  - Mouse
  - Touchpad
  - Trackpoint
  - Trackball
  - Tablet & Stylus
  - Touch Screen
─────────────
Displays
Animations
Cursor
Overview
─────────────
Window Rules     ← Promoted from Advanced
─────────────
▸ Advanced       ← Collapsed by default
  - Layout Extras
  - Gestures
  - Layer Rules
  - Miscellaneous
```

---

## Dangerous Features & Warnings

### CRITICAL WARNINGS

#### 1. Custom Display Modes (v25.11+)
**Danger Level: CRITICAL - Hardware Damage Risk**

```
⚠️ DANGER: May permanently damage your monitor!

Custom display modes can overvoltage or overclock your monitor
beyond safe limits. Incorrect settings may cause:
- Permanent screen burn-in
- Monitor hardware failure
- Reduced monitor lifespan

Only use if you fully understand modelines and your monitor's limits.

□ I understand this may damage my monitor
□ I have a backup monitor available
[Cancel] [I Accept the Risks]
```

**UI Requirements:**
- Two-step confirmation
- Bold red warning text
- Backup previous mode automatically
- Easy rollback button
- Link to safe modeline calculators

---

#### 2. Keyboard Disable
**Danger Level: HIGH - Lockout Risk**

```
⚠️ Warning: Disabling keyboard may lock you out!

If you disable your keyboard and have no other input device,
you may be unable to control niri or re-enable the keyboard.

Ensure you have:
- Another keyboard connected, OR
- SSH access to this machine, OR
- Physical access to force-reboot

[Cancel] [I Have Another Input Method]
```

---

#### 3. Calibration Matrix
**Danger Level: MEDIUM - Usability Risk**

```
⚠️ Advanced Feature

Incorrect calibration matrix values can make your tablet
or touch screen unusable. Values should be calculated using
libinput's calibration tools.

[Reset to Default] [Learn More] [Apply]
```

---

### INFORMATIONAL WARNINGS

#### 4. Version Warnings
Show inline warnings for features requiring newer niri versions:

```
ⓘ Requires niri v25.08 or later
  Current version: 25.05 (detected)
  This setting will be ignored until you upgrade.
```

Detection:
- Query niri version via IPC: `niri msg version`
- Parse version and compare
- Show warning badges on unsupported settings

---

## Complexity Estimates

| Feature | Complexity | Effort (hours) | Dependencies |
|---------|-----------|----------------|--------------|
| **Phase 1: Essential Input** | | | |
| scroll-button/lock | Simple | 4 | None |
| Device disable | Simple | 3 | None |
| xkb.file | Simple | 3 | File picker |
| **Phase 2: Advanced Input** | | | |
| Trackpoint/Trackball | Medium | 8 | Device detection |
| Tablet settings | Medium | 6 | None |
| Touch settings | Medium | 5 | None |
| Power key handling | Simple | 2 | None |
| **Phase 3: Layout Enhancements** | | | |
| preset-window-heights | Simple | 3 | None |
| default-column-display | Simple | 2 | None |
| Gradient support | Complex | 20 | Color picker widget |
| mod-key-nested | Simple | 2 | None |
| **Phase 4: Visual & Notifications** | | | |
| workspace-shadow | Simple | 4 | Shadow UI reuse |
| hotkey-overlay options | Simple | 2 | None |
| config-notification | Simple | 2 | None |
| tab-indicator urgent | Simple | 2 | None |
| **Phase 5: Output Advanced** | | | |
| Custom modes/modelines | Complex | 12 | Warning dialogs |
| Per-output hot-corners | Medium | 6 | None |
| Per-output layout | Complex | 15 | Layout UI duplication |
| **Phase 6: Window Rules** | | | |
| New matchers (6) | Simple | 6 | None |
| New properties (15+) | Medium | 20 | Various |
| Per-rule styling | Complex | 15 | Gradient support |
| **Phase 7: Workspaces & Layers** | | | |
| Named workspaces | Medium | 10 | None |
| Layer rules | Medium | 12 | None |
| **Phase 8: Per-Animation** | | | |
| Individual animations | Complex | 25 | None |
| **TOTAL** | | **187 hours** | |

**Estimate Breakdown by Complexity:**
- Simple (16 features): ~50 hours
- Medium (10 features): ~67 hours
- Complex (6 features): ~87 hours

---

## Implementation Priority Matrix

### High Priority + Simple = Quick Wins
1. Device disable (Phase 1)
2. xkb.file (Phase 1)
3. preset-window-heights (Phase 3)
4. default-column-display (Phase 3)
5. scroll-button/lock (Phase 1)
6. Window rule matchers (Phase 6)

**Recommended First Sprint:** These 6 features, ~20 hours

### High Priority + Complex = Core Features
1. Gradient support (Phase 3) - 20 hours
2. Window rule properties (Phase 6) - 20 hours
3. Named workspaces (Phase 7) - 10 hours

**Recommended Second Sprint:** These 3 features, ~50 hours

### Medium Priority
- Advanced input devices (Phase 2)
- Per-output overrides (Phase 5)
- Layer rules (Phase 7)

**Recommended Third Sprint:** ~40 hours

### Low Priority
- Per-animation configuration (Phase 8)
- Tab indicator urgent color
- Various misc settings

**Recommended Final Sprint:** ~30 hours

---

## Feature Dependencies Graph

```
Phase 1 (Essential Input) ─┐
                           ├─> Phase 6 (Window Rules) ─┐
Phase 3 (Layout)          ─┘                           │
  ├─ Gradient Support ────────────────────────────────┘
  └─ preset-window-heights ─> Phase 6 (default-window-height)

Phase 2 (Advanced Input) ─> Standalone
Phase 4 (Visual) ─────────> Standalone (uses existing Shadow UI)
Phase 5 (Output Advanced) ─> Standalone (uses Phase 7 per-output layout)
Phase 7 (Workspaces) ─────> Standalone (used by Window Rules)
Phase 8 (Animations) ─────> Standalone
```

**Critical Path:**
1. Gradient support (needed for window rule styling)
2. Window rules extensions (high user demand)
3. Named workspaces (enables workflow automation)

---

## Testing Strategy

### Unit Tests (Per Phase)
- KDL generation/parsing roundtrip
- Validation and clamping
- Default value correctness

### Integration Tests
- Full save/load cycle for each new feature
- Multi-file includes remain valid
- Backward compatibility (features degrade gracefully)

### Manual Testing Checklist
For each feature:
- [ ] Setting appears in UI
- [ ] Default value is correct
- [ ] Changes persist after save
- [ ] Changes apply in niri (via reload)
- [ ] Validation works (rejects invalid input)
- [ ] Version warning shows (if applicable)
- [ ] Danger warning shows (if applicable)
- [ ] Help text is clear and accurate
- [ ] Search finds the setting

### Version Compatibility Testing
- Test with niri 0.1.10, 25.01, 25.02, 25.05, 25.08, 25.11
- Verify version detection works
- Verify settings degrade gracefully on older versions

---

## Documentation Requirements

### User Documentation
For each feature:
1. Plain-language description (what it does, when to use it)
2. Default value and recommended range
3. Screenshot of UI
4. Example use cases
5. Link to official niri documentation

### Developer Documentation
For each feature:
1. Data model (Rust struct)
2. KDL syntax and examples
3. Validation rules
4. Version requirements
5. Related features (dependencies)

### Migration Guide
For users upgrading:
1. What's new in each phase
2. Breaking changes (if any)
3. How to enable new features
4. Recommended settings for common use cases

---

## Risk Assessment & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Custom modes damage monitor | Medium | Critical | Multi-step warnings, backup, rollback |
| Gradient UI too complex | Medium | Medium | Progressive disclosure, presets |
| Version detection fails | Low | Medium | Graceful degradation, manual override |
| Too many settings overwhelm users | High | Medium | Search, progressive disclosure, presets |
| KDL generation breaks niri | Low | High | Validate before save, backup system |
| Per-output layout too confusing | Medium | Medium | Clear visual indicators, reset button |
| Performance with many window rules | Low | Low | Optimize regex matching, limit rules |

---

## Success Metrics

### Feature Coverage
- **Target:** 100% of niri configuration options
- **Current:** ~70% (based on existing implementation)
- **After completion:** 100%

### User Experience
- **Target:** Non-technical users can configure all features
- **Measure:** User testing with 10+ participants
- **Success:** 80% can complete common tasks without help

### Code Quality
- **Target:** All features have tests
- **Measure:** Code coverage
- **Success:** >85% coverage for new code

### Performance
- **Target:** UI remains responsive with all features
- **Measure:** Time to render settings pages
- **Success:** <100ms per page load

### Documentation
- **Target:** Every setting documented
- **Measure:** Documentation completeness
- **Success:** 100% coverage

---

## Recommended Implementation Order

### Sprint 1: Quick Wins (2 weeks)
- Device disable
- xkb.file
- scroll-button/lock
- preset-window-heights
- default-column-display
- Window rule matchers

**Deliverable:** Essential missing features that many users request

---

### Sprint 2: Core Visuals (3 weeks)
- Gradient support (focus ring, border, tab-indicator)
- workspace-shadow
- tab-indicator urgent-color
- Window rule styling overrides

**Deliverable:** Complete visual customization

---

### Sprint 3: Window Management (2 weeks)
- Window rule properties (all 15+)
- Named workspaces
- default-window-height, size constraints

**Deliverable:** Advanced window management

---

### Sprint 4: Input Devices (2 weeks)
- Trackpoint, Trackball
- Tablet, Touch
- Power key handling
- calibration-matrix

**Deliverable:** Complete input device support

---

### Sprint 5: Advanced Display (2 weeks)
- Custom modes & modelines
- Per-output hot-corners
- Per-output layout overrides

**Deliverable:** Multi-monitor power user features

---

### Sprint 6: Layer & Misc (1.5 weeks)
- Layer rules
- hotkey-overlay.hide-not-bound
- config-notification.disable-failed
- mod-key-nested

**Deliverable:** Complete niri feature parity

---

### Sprint 7 (Optional): Animations (2 weeks)
- Per-animation configuration
- Spring and easing curves
- Visual previews

**Deliverable:** Animation customization for enthusiasts

---

## Total Timeline: 14.5 weeks (3.5 months)

**Minimum Viable Completion:** Sprints 1-6 = 12.5 weeks
**Full Completion:** All sprints = 14.5 weeks

---

## Sources & References

- [Niri GitHub Repository](https://github.com/YaLTeR/niri)
- [Niri v25.02 Release](https://github.com/YaLTeR/niri/discussions/1162)
- [Niri Configuration: Input](https://github.com/YaLTeR/niri/wiki/Configuration:-Input)
- [Niri Configuration: Layout](https://github.com/YaLTeR/niri/wiki/Configuration:-Layout)
- [Niri Configuration: Window Rules](https://github.com/YaLTeR/niri/wiki/Configuration:-Window-Rules)
- [Niri Configuration: Outputs](https://github.com/YaLTeR/niri/wiki/Configuration:-Outputs)
- [Niri Configuration: Animations](https://github.com/YaLTeR/niri/wiki/Configuration:-Animations)
- [Niri Configuration: Named Workspaces](https://github.com/YaLTeR/niri/wiki/Configuration:-Named-Workspaces)
- [Niri Configuration: Layer Rules](https://github.com/YaLTeR/niri/wiki/Configuration:-Layer-Rules)
- [Niri 25.02 Release Notes - Phoronix](https://www.phoronix.com/news/Niri-25.02-Labwc-0.8.3)
- [Niri 25.02 Release Notes - Linux IAC](https://linuxiac.com/niri-25-02-wayland-compositor-released/)
