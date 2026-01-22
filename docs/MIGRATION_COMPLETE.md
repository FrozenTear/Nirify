# iced Migration Complete

**Date:** 2026-01-22
**Framework:** iced 0.14 (from Slint)

## Summary

Successfully migrated niri-settings from Slint to iced 0.14. The app now uses pure Rust with no C++ dependencies.

## What Changed

### Architecture
- **Slint callbacks** → **iced Elm Architecture** (Message → update → view)
- **Slint properties** → **Rust state** in App struct
- **Slint timer** → **iced Subscription** for debounced saves

### New Structure
```
src/
├── app.rs           # Main application (State, Update, View)
├── messages.rs      # Message enum hierarchy (25 categories)
├── theme.rs         # 12 built-in themes with dark/light variants
├── search.rs        # Search indexing across all settings
├── save_manager.rs  # 300ms debounced auto-save
└── views/           # 25+ page views + widgets
```

### Features
- **All 25+ settings pages** implemented with actual data display
- **Theme system** with 12 themes (Catppuccin, Nord, Solarized, etc.)
- **Search** with keyword indexing
- **Auto-save** with 300ms debounce, toast notifications
- **List-detail views** for Window Rules, Layer Rules, Outputs, Keybindings

### Build Improvements
- **Compile time:** ~3-4 min (vs ~10 min with Slint)
- **Dependencies:** Pure Rust (no C++ from Slint)
- **IDE support:** Better rust-analyzer integration

## Views Status

| Page | Status | Features |
|------|--------|----------|
| Appearance | ✅ Full | Focus ring, border, gaps, colors |
| Behavior | ✅ Full | Focus, workspace, struts, modifiers |
| Keyboard | ✅ Full | XKB layout, repeat settings |
| Mouse | ✅ Full | Accel, scroll, buttons |
| Touchpad | ✅ Full | Tap, gestures, scroll |
| Window Rules | ✅ Full | List-detail with all properties |
| Layer Rules | ✅ Full | List-detail with match criteria |
| Keybindings | ✅ Full | Key capture, actions |
| Outputs | ✅ Full | Scale, mode, VRR |
| Animations | ✅ Full | Per-animation spring/easing |
| Debug | ✅ Read-only | 20+ debug flags |
| Misc | ✅ Read-only | CSD, screenshots, clipboard |
| Environment | ✅ Read-only | Variable list |
| Switch Events | ✅ Read-only | Lid/tablet actions |
| Recent Windows | ✅ Read-only | Alt-Tab settings |
| Others | ✅ Read-only | Trackpoint, Trackball, Tablet, Touch |

## Widgets

- `toggle_row` / `slider_row` / `text_input_row` - Standard inputs
- `color_picker` - Hex input with preview swatch
- `gradient_picker` - Multi-stop gradient editor
- `key_capture` - Keyboard shortcut capture
- `expandable_section` - Collapsible settings groups
- `list_item` - Selectable list items

## Configuration

The app preserves the takeover strategy:
- Manages `~/.config/niri/niri-settings/*.kdl` files
- User adds one include line to their config
- Each category = one KDL file
