# Research Recommendations: File Structure & UX

Based on comprehensive research into Rust/Slint best practices and settings app UX guidelines (GNOME HIG, KDE HIG, industry standards).

---

## Part 1: File Structure Recommendations

### Current vs Recommended Structure

```
CURRENT PLAN                          RECOMMENDED
─────────────────────────────────────────────────────────────────
src/                                  src/
├── main.rs                           ├── main.rs
│                                     ├── lib.rs              ← NEW (better testing)
│                                     ├── constants.rs        ← NEW
│                                     ├── types.rs            ← NEW (shared enums)
├── config/                           ├── config/
│   ├── mod.rs                        │   ├── mod.rs
│   ├── kdl_parser.rs                 │   ├── parser.rs       ← RENAMED
│   ├── settings.rs                   │   ├── models.rs       ← RENAMED
│   └── paths.rs                      │   ├── storage.rs      ← NEW (load/save)
│                                     │   ├── paths.rs
│                                     │   └── error.rs        ← NEW
├── ui/                               ├── ui/
│   ├── mod.rs                        │   ├── mod.rs
│   └── bridge.rs                     │   ├── bridge.rs
│                                     │   └── window.rs       ← NEW (state mgmt)
│                                     ├── models.rs           ← NEW (UI data models)
└── ipc/                              ├── ipc/
    └── mod.rs                        │   ├── mod.rs
                                      │   ├── client.rs       ← NEW
                                      │   └── types.rs        ← NEW
                                      └── utils/              ← NEW
                                          └── mod.rs

ui/                                   ui/
├── main.slint                        ├── main.slint
│                                     ├── styles.slint        ← NEW (shared styles)
├── appearance.slint                  ├── appearance.slint
├── ...                               ├── ...
└── widgets/                          └── widgets/
    └── ...                               └── ...

                                      tests/                  ← NEW
                                      ├── common/
                                      │   └── mod.rs
                                      ├── config_test.rs
                                      └── ui_test.rs
```

### Key Changes Explained

| Change | Why |
|--------|-----|
| Add `lib.rs` | Enables integration testing, re-exports public API |
| Add `constants.rs` | Centralize app constants (APP_NAME, VERSION, defaults) |
| Add `types.rs` | Shared enums used across modules (Theme, InputDevice, etc.) |
| Rename to `parser.rs` | Clearer naming (matches storage.rs pattern) |
| Rename to `models.rs` | Industry standard term for data structures |
| Add `storage.rs` | Separate load/save logic from data models |
| Add `error.rs` | Custom error types with `thiserror` |
| Add `window.rs` | Window state management |
| Add `ui/models.rs` | Slint VecModel bindings for UI data |
| Add `styles.slint` | Shared UI styles/theming |
| Add `tests/` | Integration tests directory |

### Slint Model Pattern (Important!)

For efficient UI updates, use Slint's Model system:

```rust
// src/models.rs
use slint::{Model, VecModel, ModelRc};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SettingItem {
    pub label: slint::SharedString,
    pub value: slint::SharedString,
    pub category: slint::SharedString,
}

// Create model for settings list
pub fn create_settings_model() -> ModelRc<SettingItem> {
    let model = Rc::new(VecModel::default());
    ModelRc::from(model)
}
```

### Test Organization

```rust
// tests/common/mod.rs - shared test utilities
pub fn create_temp_config_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

pub fn sample_kdl_content() -> &'static str {
    r#"
    layout {
        gaps 16
    }
    "#
}

// tests/config_test.rs
use niri_settings::config::{ConfigManager, models::Settings};

#[test]
fn test_config_roundtrip() {
    let temp = common::create_temp_config_dir();
    // ...
}
```

---

## Part 2: UX Recommendations

### Major UX Changes Recommended

| Aspect | Current Plan | Recommended | Impact |
|--------|--------------|-------------|--------|
| **Navigation** | Tabs | **Sidebar** | High |
| **Window Size** | Fixed 650x700 | **Resizable** | Medium |
| **Apply Model** | Apply/Close | **Live preview** | High |
| **Search** | None | **Add search bar** | High |
| **Advanced Settings** | Flat list | **Progressive disclosure** | Medium |

### 1. Sidebar Navigation (Instead of Tabs)

**Why**: GNOME and KDE both use sidebar navigation for settings. Tabs don't scale well beyond 5-6 categories.

```
┌────────────────────────────────────────────────────┐
│ [🔍 Search settings...]                            │
├──────────────┬─────────────────────────────────────┤
│              │                                     │
│  Appearance  │  WINDOW GAPS                        │
│  Behavior    │  ┌─────────────────────────────┐   │
│  Keyboard    │  │ Gap Size        [====●===] 16│   │
│  Mouse       │  └─────────────────────────────┘   │
│  Touchpad    │                                     │
│  ──────────  │  FOCUS INDICATOR                    │
│  Outputs     │  ┌─────────────────────────────┐   │
│  Animations  │  │ Ring Width      [==●=====]  4│   │
│  Cursor      │  │ Color           [■] #7fc8ff │   │
│  Overview    │  └─────────────────────────────┘   │
│  ──────────  │                                     │
│  ▸ Advanced  │  WINDOW BORDER                      │
│              │  ┌─────────────────────────────┐   │
│              │  │ Border Width    [●========]  2│   │
│              │  │ Color           [■] #ffc87f │   │
│              │  └─────────────────────────────┘   │
│              │                                     │
├──────────────┴─────────────────────────────────────┤
│                                        [Close]     │
└────────────────────────────────────────────────────┘
```

**Benefits**:
- Shows all categories at once
- Expandable for "Advanced" section
- Scales better with more settings
- Matches user expectations from GNOME/KDE

### 2. Resizable Window (Instead of Fixed)

**Current**: Fixed 650x700
**Recommended**: Minimum 600x500, resizable

**Implementation**:
```slint
export component MainWindow inherits Window {
    min-width: 600px;
    min-height: 500px;
    preferred-width: 800px;
    preferred-height: 600px;
    // No max constraints - let users resize freely
}
```

**Responsive breakpoints**:
- **< 600px wide**: Collapse sidebar to hamburger menu
- **600-900px**: Normal sidebar + content
- **> 900px**: Can show additional info/previews

### 3. Live Preview (Instead of Apply Button)

**Current**: Apply button saves all changes
**Recommended**: Changes apply immediately as user adjusts them

**Why**:
- Modern UX trend (macOS, most mobile apps)
- Reduces cognitive load ("did I save?")
- Allows safe experimentation
- Users see immediate feedback

**Implementation**:
```rust
// In bridge.rs - save on every change
ui.on_gap_size_changed(move |new_value| {
    let mut settings = settings.lock().unwrap();
    settings.appearance.gap_size = new_value;
    settings.save().ok(); // Save immediately
    reload_niri_config().ok(); // Apply to niri
});
```

**UI change**: Remove "Apply" button, keep only "Close"

**Exception**: For destructive actions (reset to defaults), use confirmation dialog.

### 4. Add Search Functionality

**Why**: Essential for settings apps with 15+ options. Users often don't know which category contains what they need.

```
┌────────────────────────────────────────┐
│ [🔍 Search settings...              ]  │
└────────────────────────────────────────┘
     ↓ User types "repeat"
┌────────────────────────────────────────┐
│ Results:                               │
│  ├─ Keyboard → Repeat Delay           │
│  └─ Keyboard → Repeat Rate            │
└────────────────────────────────────────┘
```

**Implementation**:
- Search bar at top of window (always visible)
- Real-time filtering (< 500ms response)
- Search labels AND descriptions
- Show category path in results

### 5. Progressive Disclosure for Advanced Settings

**Current**: All settings visible
**Recommended**: Hide advanced settings by default

**Pattern**:
```
WINDOW GAPS
┌─────────────────────────────────────┐
│ Gap Size              [====●===] 16 │
└─────────────────────────────────────┘

▸ Advanced Gap Options (click to expand)
  ┌─────────────────────────────────────┐
  │ Inner Gap            [===●====] 12 │
  │ Outer Gap            [====●===] 16 │
  │ Smart Gaps           [✓]           │
  └─────────────────────────────────────┘
```

**Benefits**:
- Non-technical users see simple interface
- Power users can access everything
- Reduces initial cognitive load

### 6. Card-Based Grouping

Group related settings in visual cards with clear headers:

```slint
component SettingsSection inherits Rectangle {
    in property <string> title;

    background: #f5f5f5;
    border-radius: 8px;
    padding: 16px;

    VerticalLayout {
        Text {
            text: title;
            font-weight: 600;
            font-size: 14px;
            color: #666;
        }

        @children
    }
}
```

### 7. Plain-Language Labels with Descriptions

**Bad**:
```
DWT: [✓]
Accel Profile: [Adaptive ▼]
```

**Good**:
```
Disable While Typing                    [✓]
Pause touchpad when using keyboard

Pointer Acceleration                    [Adaptive ▼]
How the cursor speeds up as you move faster
  • Flat: Consistent speed, good for precision
  • Adaptive: Speeds up with faster movement
```

### 8. Keyboard Accessibility

**Required features**:
- Tab navigates through all controls
- Arrow keys navigate within groups
- Enter/Space activates buttons
- Escape closes dialogs
- Visible focus indicator

```slint
// Ensure focus rectangle is visible
component FocusableButton inherits Button {
    states [
        focused when self.has-focus : {
            border-width: 2px;
            border-color: #0066cc;
        }
    ]
}
```

---

## Summary: Priority Changes

### Must Have (Phase 1-2)
1. ✅ Sidebar navigation instead of tabs
2. ✅ Resizable window
3. ✅ Plain-language labels with descriptions
4. ✅ Card-based grouping
5. ✅ Keyboard accessibility

### Should Have (Phase 3-4)
1. Search functionality
2. Live preview (remove Apply button)
3. Progressive disclosure for advanced settings

### Nice to Have (Phase 5-6)
1. Responsive breakpoints
2. Animations/transitions
3. Theme support (light/dark)

---

## Updated UI Mockup

```
┌─────────────────────────────────────────────────────────────┐
│  Niri Settings                                    [─][□][×] │
├─────────────────────────────────────────────────────────────┤
│  [🔍 Search settings...]                                    │
├───────────────┬─────────────────────────────────────────────┤
│               │                                             │
│   Appearance  │  WINDOW SPACING                             │
│ ● Behavior    │  ┌───────────────────────────────────────┐ │
│   Keyboard    │  │ Space Between Windows    [====●==] 16 │ │
│   Mouse       │  │ Pixels of gap between tiled windows   │ │
│   Touchpad    │  └───────────────────────────────────────┘ │
│   ───────────  │                                             │
│   Displays    │  FOCUS INDICATOR                            │
│   Animations  │  ┌───────────────────────────────────────┐ │
│   Cursor      │  │ Ring Width              [==●=====]  4 │ │
│   Overview    │  │ Thickness of the focus highlight      │ │
│   ───────────  │  │                                       │ │
│ ▸ Advanced    │  │ Active Color            [■] #7fc8ff   │ │
│               │  │ Color when window is focused          │ │
│               │  └───────────────────────────────────────┘ │
│               │                                             │
│               │  ▸ More Border Options                     │
│               │                                             │
├───────────────┴─────────────────────────────────────────────┤
│  Changes are saved automatically                   [Close]  │
└─────────────────────────────────────────────────────────────┘
```

---

## File Structure Final Recommendation

```
niri-settings-rust/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   ├── constants.rs            # App constants
│   ├── types.rs                # Shared enums
│   ├── config/
│   │   ├── mod.rs              # Config API
│   │   ├── models.rs           # Data structures
│   │   ├── parser.rs           # KDL parsing
│   │   ├── storage.rs          # Load/save
│   │   ├── paths.rs            # File paths
│   │   └── error.rs            # Error types
│   ├── models.rs               # UI data models
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── window.rs           # Window state
│   │   └── bridge.rs           # Slint callbacks
│   └── ipc/
│       ├── mod.rs
│       ├── client.rs
│       └── types.rs
├── ui/
│   ├── main.slint              # Main window + sidebar
│   ├── styles.slint            # Shared styles
│   ├── pages/                  # Category pages
│   │   ├── appearance.slint
│   │   ├── behavior.slint
│   │   └── ...
│   └── widgets/
│       ├── sidebar.slint
│       ├── search.slint
│       ├── section.slint
│       ├── color_picker.slint
│       └── ...
├── tests/
│   ├── common/mod.rs
│   └── ...
├── Cargo.toml
└── build.rs
```
