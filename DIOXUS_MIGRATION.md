# Dioxus Migration Status

**Branch:** `dioxus-migration`
**Last Updated:** 2026-01-18

## Overview

This branch migrates niri-settings from Slint to Dioxus with the Blitz native renderer. The app now renders natively on Wayland via wgpu/Vulkan without requiring a web runtime.

## What's Been Done

### 1. Framework Migration
- Removed Slint dependencies and all `.slint` UI files
- Added `dioxus-native` crate with Blitz renderer
- App uses reactive signals for state management
- Static globals with `LazyLock` for config paths and settings

### 2. UI Design (Matching Slint)
- Top-bar navigation with 6 primary tabs
- Secondary tabs for subcategories within each group
- Search bar in header (placeholder - filtering not implemented)
- Footer with "Changes saved automatically" status
- Dark theme with purple accents (#9580ff)

### 3. Navigation Structure

| Primary Tab | Secondary Tabs |
|-------------|----------------|
| Appearance | Windows, Cursor |
| Input | Keyboard, Mouse, Touchpad, Trackpoint, Trackball, Tablet, Touch |
| Visuals | Animations, Overview, Recent Windows |
| Layout | Gaps, Extras, Workspaces |
| Rules | Windows, Layers, Gestures |
| System | Displays, Keybindings, Startup, Environment, Switch Events, Miscellaneous, Debug |

### 4. Working Features
- Config loading from `~/.config/niri/niri-settings/*.kdl`
- Live save on change (immediate config write)
- IPC reload to niri (100ms rate-limited to prevent overwhelming)
- Toggle switches (on/off buttons)
- Slider controls (+/- buttons with value display)
- Color pickers (native HTML color input with system dialog)
- Settings sections with styled headers

### 5. Reusable Components

```rust
Section       // Settings section with title header
ToggleRow     // Boolean toggle with label and optional description
SliderRow     // Numeric value with +/- buttons, min/max/step
ColorRow      // Color picker for ColorOrGradient values
OptionalColorRow  // Optional color with enable/disable toggle
```

### 6. Page Content

**Appearance/Windows:**
- Focus ring: toggle, width, active/inactive/urgent colors
- Window border: toggle
- Background: optional window background color

**Layout/Gaps:**
- Window gaps slider
- Corner radius slider
- Focus behavior settings
- Screen edge margins (struts)

**Other pages:** Basic settings implemented, some with placeholder content

## File Structure

```
src/
├── main.rs        (2235 lines)  - App entry, all UI components and pages
├── styles.css     (600 lines)   - CSS styling matching Slint design
├── lib.rs                       - Library exports
├── config/                      - KDL config loading/saving (unchanged from master)
│   ├── models/                  - Settings structs
│   ├── loader/                  - Load from KDL files
│   └── storage/                 - Save to KDL files
├── types.rs                     - Color, ColorOrGradient, enums
├── constants.rs                 - Min/max values, defaults
└── ipc/                         - Niri socket communication
```

## What's Not Yet Implemented

- [ ] Search functionality (UI exists but doesn't filter results)
- [ ] Keyboard shortcut editor (shows list but no add/edit/delete)
- [ ] Window rule editor (shows list but no add/edit/delete)
- [ ] Layer rule editor (shows list but no add/edit/delete)
- [ ] Output/display configuration UI
- [ ] First-run wizard dialog
- [ ] Error dialog for config issues

## Dependencies

```toml
dioxus-native = { version = "0.7", features = ["prelude"] }
kdl = { version = "6.5", features = ["v1-fallback"] }
# ... rest unchanged from master
```

## Running

```bash
# Development
cargo run

# Release build
cargo build --release
```

## Commits

```
d638827 feat: make color picker functional with native color input
f4b7f16 feat: add color pickers and complete settings like Slint
5d21b22 refactor: redesign UI to match Slint top-bar navigation
fc3864f feat: add all remaining pages for feature parity with master
f2ff298 feat: add all settings pages with config loading/saving
6ea84f8 feat: add Appearance page with working controls
8b92aa1 feat: add clickable sidebar navigation
1dad83b fix: clean up debug prints, confirm Dioxus+Blitz works
bef8668 refactor: migrate from Slint to Dioxus with Blitz native renderer
```

## Notes

- Blitz renderer has CSS limitations (no ::before/::after pseudo-elements)
- Toggle buttons show "On"/"Off" text since CSS-only switches aren't possible
- Color picker uses native HTML `<input type="color">` for system dialog
- All config changes are saved immediately and trigger niri reload via IPC
