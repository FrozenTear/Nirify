# Slint to Vizia Migration Analysis: Niri Settings App

**Date:** 2026-01-20
**Status:** Phase 1 Complete - Architecture Analysis
**Goal:** Port niri-settings from Slint to Vizia to reduce compile times

---

## Executive Summary

The niri-settings application is a comprehensive Wayland compositor settings manager built with **Slint 1.14.1** as its UI framework. After thorough analysis, this report provides a complete inventory of the current architecture and identifies critical migration considerations for porting to Vizia.

### Key Metrics
- **Total Slint LOC:** ~19,055 lines across 46 files
- **Total Rust LOC:** ~36,051 lines
- **Settings Categories:** 21 distinct categories
- **UI Pages:** 27 settings pages + 4 dialogs
- **Custom Widgets:** 21 reusable components
- **Complexity Level:** **High** (dynamic models, custom rendering, IPC integration)

---

## 1. Phase 1.1: Current Slint Architecture Inventory

### 1.1 Slint File Organization

#### **Pages (27 files - ~12,000 LOC)**
Location: `ui/pages/`

**Dynamic Model-Driven Pages (Most pages):**
- ✅ `animations.slint` - Per-animation spring/easing configs
- ✅ `appearance.slint` - Focus ring, borders, colors, gradients
- ✅ `behavior.slint` - Window behavior, struts, column widths
- ✅ `cursor.slint` - Cursor theme, size, hide-after-inactive
- ✅ `debug.slint` - Debug overlay, damage tracking, FPS counter
- ✅ `gestures.slint` - Touchpad gestures configuration
- ✅ `keyboard.slint` - Repeat rate/delay, track-layout
- ✅ `layout_extras.slint` - Default column widths, center-focused-column
- ✅ `miscellaneous.slint` - Screenshot path, disable-cursor-plane
- ✅ `mouse.slint` - Accel speed/profile, scroll factor, etc.
- ✅ `overview.slint` - Overview zoom settings
- ✅ `switch_events.slint` - Lid close, tablet mode toggle
- ✅ `touchpad.slint` - Touchpad input settings
- ✅ `trackpoint.slint` - TrackPoint input settings
- ✅ `trackball.slint` - Trackball input settings
- ✅ `tablet.slint` - Tablet input settings
- ✅ `touch.slint` - Touch input settings
- ✅ `window_rules.slint` - Dynamic window rule editor
- ✅ `layer_rules.slint` - Layer rule configuration
- ✅ `workspaces.slint` - Workspace settings
- ✅ `startup.slint` - Startup commands
- ✅ `environment.slint` - Environment variables
- ✅ `recent_windows.slint` - Recent windows switcher settings (v25.05+)

**Static/Complex Pages (Remaining):**
- ⚠️ `displays.slint` - Output/monitor configuration (mode, scale, position, transform, VRR)
- ⚠️ `keybindings.slint` - **Most complex:** Key capture, action editor, dynamic list
- ⚠️ `config_editor.slint` - Raw KDL editor (fallback for unsupported features)
- ⚠️ `backups.slint` - Backup management UI
- ⚠️ `tools.slint` - Import/export utilities

#### **Widgets (21 files - ~5,000 LOC)**
Location: `ui/widgets/`

**Layout & Structure:**
- `sidebar.slint` - Navigation sidebar with category selection
- `category_nav.slint` - Sub-category navigation tabs
- `section.slint` - Settings section container
- `expandable_section.slint` - Collapsible sections
- `floating_panel.slint` - Overlay panels
- `search.slint` - Search bar with fuzzy matching
- `search_results_panel.slint` - Search results overlay

**Custom Input Components (CRITICAL - Complex Rendering):**
- 🔴 `color_picker.slint` - Color picker with hex input, expandable swatches, presets
- 🔴 `gradient_picker.slint` - Advanced gradient editor (from/to colors, angle, color-space, hue-interpolation, relative-to)
- 🔴 `key_capture.slint` - Keyboard shortcut capture widget with modifier detection
- 🔴 `key_badges.slint` - Visual key combination display (Mod+Shift+Q → badge pills)
- 🔴 `calibration_matrix_row.slint` - 6-value matrix input for display calibration

**Standard Setting Rows:**
- `base_setting_row.slint` - Base template for setting rows
- `toggle_row.slint` - Label + Switch
- `slider_row.slint` - Label + Slider + value display
- `percentage_slider_row.slint` - Slider with percentage formatting
- `combobox_row.slint` - Label + ComboBox
- `text_input_row.slint` - Label + LineEdit
- `integer_input_row.slint` - Label + SpinBox
- `file_path_row.slint` - Label + file picker button

**Dynamic Rendering:**
- `dynamic_settings.slint` - Universal model-driven row renderer (SettingModel struct)

#### **Dialogs (4 files - ~1,500 LOC)**
Location: `ui/dialogs/`

- `first_run_wizard.slint` - Onboarding wizard for new users
- `error_dialog.slint` - Error/success/info message dialog + toast notifications
- `confirm_dialog.slint` - Confirmation dialogs (delete, reset, etc.)
- `import_summary.slint` - Import results summary
- `diff_view.slint` - Config diff viewer (shows changes before applying)
- `consolidation_dialog.slint` - Rule consolidation suggestions

#### **Core Files (2 files - ~500 LOC)**
- `main.slint` - Root window, layout, page routing
- `styles.slint` - Theme definitions (Catppuccin Mocha), shared structs

---

### 1.2 Widget Types in Use

| Widget Type | Count | Complexity | Notes |
|-------------|-------|------------|-------|
| **Switch/Toggle** | ~150 | Low | Simple boolean controls |
| **Slider** | ~80 | Low-Med | Float/int ranges with value display |
| **ComboBox** | ~60 | Low | Enum selections |
| **LineEdit** | ~40 | Low | Text inputs |
| **Button** | ~30 | Low | Actions (add, delete, import, etc.) |
| **ColorPicker** | ~15 | **High** | Custom component with swatches, hex input |
| **GradientPicker** | ~8 | **Very High** | Multi-color with angle/space/interpolation |
| **KeyCapture** | ~2 | **Very High** | Keyboard event capture with modifier parsing |
| **ScrollView** | ~25 | Med | Lists and scrollable content |
| **ListView/for-loop** | ~15 | Med | Dynamic lists (keybindings, rules, outputs) |

---

### 1.3 Custom Slint Components (Non-Trivial)

#### 🔴 **ColorPicker** (High Priority)
**File:** `ui/widgets/color_picker.slint`
**Complexity:** High
**Features:**
- Color preview swatch (clickable to expand)
- Hex input field (`#RRGGBB` or `#RGB`)
- Expandable palette with 24 preset colors
- Two-way binding between `selected-color` and `hex-value`
- Smooth animations for expand/collapse
- Material Design + Catppuccin color presets

**Vizia Migration Notes:**
- Vizia has basic `Color` type but no built-in color picker
- Will need custom view implementing:
  - Color swatch rendering (rectangles with rounded corners)
  - Hex text input with validation
  - Grid layout for swatches
  - Click handlers for swatch selection
  - State management for expanded/collapsed

---

#### 🔴 **GradientPicker** (Very High Priority - Most Complex Widget)
**File:** `ui/widgets/gradient_picker.slint`
**Complexity:** Very High
**Features:**
- Toggle between solid color and gradient modes
- **Gradient mode includes:**
  - From/To color pickers (each with hex input + swatch)
  - Angle slider (0-360°) with visual preview
  - Color space dropdown: `srgb`, `srgb-linear`, `oklab`, `oklch`
  - Relative-to dropdown: `window`, `workspace-view`
  - Hue interpolation (for `oklch`): `shorter`, `longer`, `increasing`, `decreasing`
- Gradient preview rectangle showing from→to transition
- Expandable panel (~200px height in gradient mode, ~50px in solid mode)

**Vizia Migration Notes:**
- **CRITICAL:** This is the most complex widget in the app
- Gradient preview requires custom drawing (two rectangles side-by-side)
- Will need to implement all dropdowns and sliders manually
- State management for visibility (solid vs gradient mode)
- Consider phased migration: solid color first, gradient later

---

#### 🔴 **KeyCaptureInput** (Very High Priority)
**File:** `ui/widgets/key_capture.slint`
**Complexity:** Very High
**Features:**
- Captures raw keyboard events when active
- Displays "Press keys..." prompt when capturing
- Shows captured key combo as badge pills (Mod+Shift+Q)
- Parses modifiers: Meta, Ctrl, Alt, Shift
- Escape key cancels capture
- Focus-based activation/deactivation
- Clear button to reset

**Rust Integration:**
```rust
callback raw-key-pressed(text: string, meta: bool, ctrl: bool, alt: bool, shift: bool)
```

**Vizia Migration Notes:**
- Vizia has `on_key_down` event handlers
- Will need to track focus state and capture mode
- Modifier parsing should work similarly
- Badge rendering (KeyBadges component) needs custom pill-shaped views
- Consider using Vizia's `FocusPolicy` for capture mode

---

#### 🟡 **CalibrationMatrixRow** (Medium Priority)
**File:** `ui/widgets/calibration_matrix_row.slint`
**Features:**
- 6 float inputs for display calibration matrix
- Label + 6 LineEdit fields in a row
- Used only in Displays page (rare feature)

**Vizia Migration Notes:**
- Low usage frequency (advanced feature)
- Can be implemented as 6 `Textbox` views in HStack
- Validate float parsing on input

---

#### 🟡 **DynamicSettingsSection** (Medium Priority - Backbone of Model-Driven Pages)
**File:** `ui/widgets/dynamic_settings.slint`
**Features:**
- Universal `SettingModel` struct for all control types
- Single component renders any setting based on `setting-type` field:
  - Type 0: Toggle (Switch)
  - Type 1: Slider (int or float)
  - Type 2: ComboBox
  - Type 3: Text input
  - Type 4: Color input
- Used by 20+ pages to reduce boilerplate

**Vizia Migration Notes:**
- This is the KEY pattern that makes the migration manageable
- Vizia equivalent: create enum `SettingType` with variants for each control
- Use pattern matching to render appropriate widget
- Consider using Vizia's `Binding` for model updates

---

## 2. Phase 1.2: State Architecture & Data Flow

### 2.1 Main Configuration Struct

**Location:** `src/config/models/mod.rs`

```rust
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
    pub keybindings: KeybindingsSettings,
    pub startup: StartupSettings,
    pub environment: EnvironmentSettings,
    pub debug: DebugSettings,
    pub switch_events: SwitchEventsSettings,
    pub recent_windows: RecentWindowsSettings,
}
```

**Total:** 21 categories, each with 5-30 fields = ~300+ settings

---

### 2.2 Data Flow (Slint → Vizia Comparison)

#### **Current Slint Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│ 1. LOAD (Startup)                                           │
└─────────────────────────────────────────────────────────────┘
   KDL Files (25 .kdl files)
        ↓ (loader/)
   Settings struct (Arc<Mutex<Settings>>)
        ↓ (sync.rs)
   Slint UI Properties (~1000 in-out properties in main.slint)

┌─────────────────────────────────────────────────────────────┐
│ 2. USER CHANGES (Runtime)                                   │
└─────────────────────────────────────────────────────────────┘
   User interacts with UI
        ↓ (Slint callback)
   Rust callback handler (callbacks/*.rs)
        ↓ (updates Settings)
   SaveManager.request_save() (300ms debounce)
        ↓ (storage/)
   Write dirty KDL files (only changed categories)
        ↓ (ipc::async_ops)
   Niri IPC: reload_config() [background thread]
```

#### **Proposed Vizia Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│ 1. LOAD (Startup)                                           │
└─────────────────────────────────────────────────────────────┘
   KDL Files (25 .kdl files)
        ↓ (loader/ - reuse as-is)
   Settings struct (add #[derive(Lens)])
        ↓ (AppState.build())
   Vizia UI Bindings (Binding::new + Lens)

┌─────────────────────────────────────────────────────────────┐
│ 2. USER CHANGES (Runtime)                                   │
└─────────────────────────────────────────────────────────────┘
   User interacts with UI
        ↓ (Vizia event handler)
   cx.emit(AppEvent::SetSomeValue(val))
        ↓ (Model::event)
   AppState.field = val
        ↓ (SaveManager - reuse logic)
   Write dirty KDL files (only changed categories)
        ↓ (ipc::async_ops - reuse as-is)
   Niri IPC: reload_config() [background thread]
```

**Key Difference:**
- **Slint:** Properties declared in `.slint` files, manually synced to Rust
- **Vizia:** Properties are Rust fields with `#[derive(Lens)]`, UI binds directly

---

### 2.3 Callback Pattern (Slint)

**Example:** Appearance focus ring width slider

```rust
// src/ui/bridge/callbacks/appearance.rs
ui.on_focus_ring_width_changed({
    let settings = settings.clone();
    let save_manager = save_manager.clone();
    move |width| {
        if let Ok(mut s) = settings.lock() {
            s.appearance.focus_ring_width = width as i32;
        }
        save_manager.request_save(SettingsCategory::Appearance);
    }
});
```

**Vizia Equivalent (Proposed):**

```rust
// In AppState Model::event
AppEvent::SetFocusRingWidth(width) => {
    self.appearance.focus_ring_width = width;
    self.dirty_tracker.mark(SettingsCategory::Appearance);
    self.save_manager.request_save();
}
```

---

### 2.4 SaveManager (Reusable in Vizia)

**Current Implementation:** `src/ui/bridge/save_manager.rs`

- **300ms debounce:** Prevents excessive disk I/O during rapid changes (e.g., slider drag)
- **Dirty tracking:** Only saves modified categories (1-3 files vs all 25)
- **Background IPC:** `reload_config_async()` runs on separate thread to avoid UI freeze
- **Toast notifications:** Shows save success/error status

**Vizia Integration:**
- SaveManager can be **reused as-is** (it's framework-agnostic)
- Replace `slint::Timer` with `std::thread::spawn` + channels or use Vizia's timer API
- Replace `Weak<MainWindow>` with Vizia's context for toast notifications

---

## 3. Phase 1.3: Complexity Hotspots

### 3.1 Most Complex Screens (Priority Order)

#### 🔴 **1. Keybindings Page** (HIGHEST COMPLEXITY)
**File:** `ui/pages/keybindings.slint`
**Lines:** ~800
**Complexity:** **10/10**

**Features:**
- Dynamic list of keybindings (add/edit/delete)
- Key capture modal with live key combo preview
- Action type selector: `spawn`, `niri`, `niri-args`
- Per-binding options: allow-when-locked, repeat, cooldown-ms
- Overlay title for `spawn` actions
- Import from file button
- Refresh button to reload from disk

**Custom Components Used:**
- KeyCaptureInput (keyboard event capture)
- KeyBadges (visual key display)
- Modal overlay editor

**Why Complex:**
- Real-time keyboard event capture and parsing
- Dynamic list with inline editing
- Multiple conditional fields based on action type
- File import with error handling

**Vizia Migration Strategy:**
- **Phase 1:** Read-only list view (no editing)
- **Phase 2:** Add modal editor with text input (no capture)
- **Phase 3:** Implement key capture with Vizia's keyboard events
- **Phase 4:** Full feature parity

---

#### 🔴 **2. Displays Page** (HIGH COMPLEXITY)
**File:** `ui/pages/displays.slint`
**Lines:** ~600
**Complexity:** **8/10**

**Features:**
- List of connected outputs (monitors)
- Per-output settings:
  - Mode selection (resolution + refresh rate)
  - Scale factor (1.0 - 4.0)
  - Position (X, Y coordinates)
  - Transform (rotation: Normal, 90°, 180°, 270°, Flipped variants)
  - VRR (variable refresh rate) toggle
- Calibration matrix (6 float values)
- Real-time IPC query to get output info from niri

**Custom Components Used:**
- CalibrationMatrixRow
- Dynamic mode dropdown (populated from IPC)

**Why Complex:**
- IPC integration for live output detection
- Conditional fields (calibration only if supported)
- Layout preview (potential future feature)

**Vizia Migration Strategy:**
- Reuse `ipc::get_full_outputs()` logic (already Rust)
- Create OutputCard component with all controls
- Use Vizia's List/VStack for multiple outputs

---

#### 🟡 **3. Window Rules Page** (MEDIUM-HIGH COMPLEXITY)
**File:** `ui/pages/window_rules.slint`
**Lines:** ~450 (after dynamic migration)
**Complexity:** **7/10**

**Features:**
- Dynamic list of rules (add/edit/delete)
- Match conditions: app-id, title (regex support)
- Actions: opacity, corner-radius, default-column-width, etc.
- Conditional visibility based on selected action

**Custom Components Used:**
- DynamicSettingsSection (model-driven rendering)
- Match editor (app-id/title selection)

**Why Complex:**
- Dynamic list with inline editing
- Conditional field visibility
- Regex validation

**Vizia Migration Strategy:**
- Port DynamicSettingsSection pattern to Vizia first
- Use pattern matching for conditional rendering
- Reuse Rust validation logic

---

#### 🟢 **4. Simple Settings Pages** (LOW COMPLEXITY)
**Examples:** Behavior, Cursor, Keyboard, Mouse, Touchpad, etc.
**Complexity:** **3/10**

**Features:**
- Mostly static layouts with 10-20 settings
- Standard controls: toggles, sliders, comboboxes
- Already using DynamicSettingsSection for rendering

**Vizia Migration Strategy:**
- Start here for learning Vizia patterns
- Port DynamicSettingsSection first, then these pages become trivial

---

### 3.2 Dynamic Lists (State Management)

| Feature | Current Implementation | Vizia Equivalent |
|---------|----------------------|------------------|
| **Keybindings list** | `ModelRc<VecModel<KeybindingItem>>` | `Vec<Keybinding>` in AppState + `List` view |
| **Window rules list** | `ModelRc<VecModel<WindowRuleSettingModel>>` | `Vec<WindowRule>` in AppState + `List` view |
| **Startup commands** | `ModelRc<VecModel<StartupCommand>>` | `Vec<StartupCommand>` in AppState + `List` view |
| **Environment vars** | `ModelRc<VecModel<EnvVar>>` | `Vec<EnvVar>` in AppState + `List` view |
| **Outputs** | IPC query (not persisted) | Same - query via `ipc::get_outputs()` |

**Migration Notes:**
- Slint uses `ModelRc<VecModel<T>>` for reactive lists
- Vizia uses regular `Vec<T>` in AppState with `#[derive(Lens)]`
- Both support add/remove/edit via callbacks/events

---

### 3.3 Real-Time IPC Features

**Feature:** Live output detection in Displays page
**Current:** `ipc::get_full_outputs()` called on page load
**Vizia:** Same - call from event handler or view init

**Feature:** Config reload after save
**Current:** `ipc::async_ops::reload_config_async()` (background thread)
**Vizia:** Same - reuse existing IPC module as-is

**No custom rendering or live preview:** All IPC is request/response, no streaming data.

---

## 4. Code Reuse Assessment

### ✅ **Can be reused AS-IS (minimal/no changes)**

| Module | LOC | Notes |
|--------|-----|-------|
| `src/config/loader/` | ~3,000 | KDL parsing - framework agnostic |
| `src/config/storage/` | ~2,500 | KDL writing - framework agnostic |
| `src/config/models/` | ~4,000 | Add `#[derive(Lens)]` to structs |
| `src/config/parser.rs` | ~500 | KDL utilities |
| `src/config/paths.rs` | ~300 | XDG path resolution |
| `src/config/validation.rs` | ~200 | String validation |
| `src/ipc/` | ~1,800 | Niri IPC - framework agnostic |
| `src/types.rs` | ~800 | Shared enums (AccelProfile, etc.) |
| `src/constants.rs` | ~300 | Value bounds (MIN/MAX) |
| **Total reusable** | **~13,400 LOC (37%)** | |

### 🔧 **Needs adaptation but logic reusable**

| Module | LOC | Changes Needed |
|--------|-----|----------------|
| `src/ui/bridge/callbacks/` | ~8,000 | Convert to Vizia events |
| `src/ui/bridge/sync.rs` | ~1,500 | Replace with Lens bindings |
| `src/ui/bridge/save_manager.rs` | ~300 | Replace Timer with Vizia equivalent |
| **Total adaptable** | **~9,800 LOC (27%)** | |

### ❌ **Must be rewritten from scratch**

| Module | LOC | Notes |
|--------|-----|-------|
| `ui/` (all .slint files) | ~19,055 | Complete rewrite in Vizia |
| `src/ui/bridge/converters.rs` | ~500 | Slint-specific type conversions |
| `src/ui/bridge/indices.rs` | ~400 | ComboBox index mappings |
| **Total rewrite** | **~19,955 LOC (36%)** | |

---

## 5. Migration Recommendations

### 5.1 Phased Migration Plan (Revised)

#### **Phase 0: Setup & Proof of Concept** (1-2 days)
- [ ] Create new `niri-settings-vizia/` project
- [ ] Set up Cargo.toml with Vizia dependencies
- [ ] Copy reusable modules (`config/`, `ipc/`, `types.rs`)
- [ ] Create basic AppState struct with 2-3 settings
- [ ] Implement hello-world window with one settings page

**Success Criteria:** App launches, displays one page, can change a setting and save

---

#### **Phase 1: Core Infrastructure** (3-5 days)
- [ ] Port DynamicSettingsSection pattern to Vizia
- [ ] Implement SaveManager with Vizia timer/threading
- [ ] Port 3 simple settings pages (e.g., Keyboard, Mouse, Cursor)
- [ ] Implement sidebar navigation
- [ ] Implement toast notifications

**Success Criteria:** 3 pages functional, settings persist to KDL, auto-save works

---

#### **Phase 2: Standard Widgets** (5-7 days)
- [ ] Port remaining simple pages (10+ pages)
- [ ] Implement ColorPicker widget (solid colors only)
- [ ] Implement search functionality
- [ ] Port Appearance page (uses ColorPicker)

**Success Criteria:** 15+ pages working, color selection functional

---

#### **Phase 3: Advanced Widgets** (7-10 days)
- [ ] Implement GradientPicker (solid mode first, gradient later)
- [ ] Port Animations page (uses GradientPicker)
- [ ] Implement CalibrationMatrixRow
- [ ] Port Displays page (without key capture)

**Success Criteria:** All standard pages working, advanced inputs functional

---

#### **Phase 4: Complex Features** (10-14 days)
- [ ] Implement KeyCaptureInput (keyboard event handling)
- [ ] Implement KeyBadges (visual key display)
- [ ] Port Keybindings page (read-only list first)
- [ ] Add keybinding editing modal
- [ ] Implement Window Rules editor
- [ ] Port Layer Rules page

**Success Criteria:** All pages functional, keybinding capture works

---

#### **Phase 5: Polish & Testing** (5-7 days)
- [ ] Port all dialogs (error, confirm, diff view, import summary)
- [ ] Implement first-run wizard
- [ ] Add keyboard navigation
- [ ] Light/dark theme support
- [ ] Accessibility improvements
- [ ] Comprehensive testing

**Success Criteria:** Feature parity with Slint version, no regressions

---

### 5.2 Critical Risks & Mitigation

#### 🔴 **Risk 1: Key Capture Complexity**
**Problem:** Keyboard event capture with modifier detection is non-trivial
**Mitigation:**
- Research Vizia's keyboard event API first
- Prototype key capture in isolation before full page port
- Consider text-based fallback if capture proves difficult

#### 🔴 **Risk 2: Gradient Rendering**
**Problem:** GradientPicker requires custom drawing for preview
**Mitigation:**
- Implement solid color first, add gradient later
- Use simple rectangle drawing for preview (Vizia supports Canvas)
- Document gradient syntax in help text if preview too complex

#### 🟡 **Risk 3: Dynamic List Performance**
**Problem:** 100+ keybindings in list, potential scroll lag
**Mitigation:**
- Use Vizia's `List` with virtualization if available
- Benchmark with 100+ items early
- Add search/filter to reduce visible items

#### 🟡 **Risk 4: Compile Time Not Improved**
**Problem:** Vizia itself may have slow compile times
**Mitigation:**
- Measure baseline Vizia compile time in Phase 0
- Use `--release` optimizations in dev (via `.cargo/config.toml`)
- Consider modular crate structure if needed

---

### 5.3 Alternative: Hybrid Approach (Not Recommended)

**Idea:** Keep complex pages in Slint, port simple ones to Vizia first

**Pros:**
- Lower initial risk
- Incremental migration

**Cons:**
- Requires maintaining two UI frameworks in parallel
- Complex inter-framework communication
- No compile time benefit until full migration
- Technical debt increases

**Recommendation:** Do NOT pursue hybrid approach. Full migration or stay with Slint.

---

## 6. Estimated Effort (Full Migration)

| Phase | Duration | % Complete | Notes |
|-------|----------|------------|-------|
| **Phase 0: PoC** | 1-2 days | 5% | Risk validation |
| **Phase 1: Core** | 3-5 days | 20% | Foundation |
| **Phase 2: Standard** | 5-7 days | 50% | Bulk of pages |
| **Phase 3: Advanced** | 7-10 days | 75% | Complex widgets |
| **Phase 4: Complex** | 10-14 days | 95% | Keybindings, rules |
| **Phase 5: Polish** | 5-7 days | 100% | QA, testing |
| **Total** | **31-45 days** | | |

**Assumptions:**
- Single developer, full-time work
- Assumes familiarity with Vizia (add 5-7 days if learning from scratch)
- Assumes no major blockers in Vizia API

---

## 7. Compile Time Analysis

### Current Slint Baseline
```bash
# From user reports and typical Slint apps
cargo clean && time cargo build
# Expected: 3-5 minutes on modern hardware
```

### Expected Vizia Improvement
```bash
# Vizia claims faster compile times than Slint
# Expected: 1-3 minutes for clean build
# Incremental: 5-20 seconds for UI changes
```

**TODO:** Measure actual compile times in Phase 0 to validate migration benefit.

---

## 8. Next Steps

### Immediate Actions (Before Migration)
1. ✅ **Read this analysis document**
2. ⏭️ **Prototype Vizia setup** (Phase 0)
   - Create minimal window with one settings page
   - Measure baseline compile time
   - Test Lens binding pattern
3. ⏭️ **Decision point:** Go/No-Go based on compile time improvement
   - If compile time < 2min improvement → **ABORT migration**
   - If compile time ≥ 2min improvement → **PROCEED with Phase 1**

### Migration Start Checklist
- [ ] Create `niri-settings-vizia/` directory
- [ ] Initialize Cargo project
- [ ] Copy `VIZIA_MIGRATION_ANALYSIS.md` to project root
- [ ] Set up Git branch: `vizia-migration`
- [ ] Document compile time baselines (Slint vs Vizia)

---

## 9. Appendix: Key Files Reference

### Most Important Files to Understand First

1. **State Management**
   - `src/config/models/mod.rs` - Root Settings struct
   - `src/config/loader/mod.rs` - KDL loading entry point
   - `src/config/storage/mod.rs` - KDL saving entry point

2. **UI Bridge**
   - `src/ui/bridge/mod.rs` - Callback registration
   - `src/ui/bridge/sync.rs` - UI synchronization
   - `src/ui/bridge/save_manager.rs` - Debounced saving

3. **Complex Widgets**
   - `ui/widgets/dynamic_settings.slint` - Model-driven rendering pattern
   - `ui/widgets/color_picker.slint` - Custom color input
   - `ui/widgets/gradient_picker.slint` - Advanced gradient editor
   - `ui/widgets/key_capture.slint` - Keyboard event capture

4. **Complex Pages**
   - `ui/pages/keybindings.slint` - Most complex page
   - `ui/pages/displays.slint` - IPC integration example
   - `ui/pages/window_rules.slint` - Dynamic list example

5. **IPC Integration**
   - `src/ipc/mod.rs` - Niri socket communication
   - `src/ipc/async_ops.rs` - Background operations

---

## 10. Questions Answered (From Porting Plan)

> **How many .slint files exist and what are they named?**

**Answer:** 46 files total (~19,055 LOC):
- 27 pages (ui/pages/)
- 21 widgets (ui/widgets/)
- 4 dialogs (ui/dialogs/)
- 2 core files (main.slint, styles.slint)

> **What's the structure of your main config struct?**

**Answer:** `Settings` struct with 21 categories (see section 2.1). Each category has 5-30 fields. Total ~300+ individual settings. Already uses Default trait, PartialEq, Clone.

> **Are you using any custom Slint components that do non-trivial rendering?**

**Answer:** YES - 4 critical components:
1. **ColorPicker** - Custom swatch grid + hex input
2. **GradientPicker** - Multi-input gradient editor with preview
3. **KeyCaptureInput** - Keyboard event capture with modifier parsing
4. **KeyBadges** - Visual pill-shaped key display

> **What's your current approach for persisting settings back to KDL?**

**Answer:**
- SaveManager with 300ms debounce
- DirtyTracker marks changed categories
- Only saves dirty files (1-3 vs all 25)
- Background IPC reload via `reload_config_async()`

> **Do you have any real-time IPC features (e.g., live preview of changes)?**

**Answer:** NO live preview. IPC is request/response only:
- Query outputs on Displays page load
- Reload config after save
- All IPC is async on background threads

---

## 11. Conclusion

The niri-settings app is a **well-architected, feature-rich application** with excellent separation between UI and business logic. **37% of the codebase can be reused as-is**, and another **27% needs only minor adaptation**.

The **most challenging aspects** of the Vizia migration are:
1. KeyCaptureInput (keyboard events)
2. GradientPicker (custom rendering)
3. Dynamic list management (keybindings, rules)

However, the **modular architecture** (especially the DynamicSettingsSection pattern) makes the migration tractable. By porting the dynamic rendering infrastructure first, the majority of pages become straightforward.

**Recommendation:** Proceed with **Phase 0 proof of concept** to validate compile time improvement. If Vizia delivers ≥2min compile time reduction, the migration is worth the 6-9 week effort for long-term productivity gains.

---

**End of Analysis**
