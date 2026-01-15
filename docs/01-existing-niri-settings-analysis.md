# Existing niri-settings Analysis

## Overview

The existing [niri-settings](https://github.com/stefonarch/niri-settings) is a Python/PyQt6 application that provides a GUI for configuring the niri window manager.

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Python (98.5%) |
| UI Framework | PyQt6 |
| Config Format | KDL |
| License | GPL-2.0 |
| Min niri version | 25.11+ |

## Features

### Appearance Tab
- Shadow and animation configuration
- Focus ring styling with color picker
- Border styling with color picker
- Overview zoom level adjustment
- Window margin and strut configuration

### Behavior Tab
- Hotkey overlay management
- Mouse warping control
- Focus-follows-mouse toggling
- Power key handling
- Modifier key selection (Super/Alt/Ctrl)
- Screenshot path customization
- Cursor visibility with configurable inactive timeout

### Input Tabs
- **Mouse**: Natural scroll, left-handed mode, acceleration, scroll factor
- **Touchpad**: Tap-to-click, drag lock, scroll method, tap button mapping
- **Keyboard**: XKB layout/variant/options, repeat delay/rate

## Architecture

```
Main Entry Point (main.py)
├── Application initialization
├── Locale detection and translation loading
├── SettingsWindow instantiation
└── Event loop execution

SettingsWindow (settings_window.py)
├── Tab widget container (600x750px)
├── Five configuration tabs
│   ├── AppearanceTab
│   ├── BehaviorTab
│   ├── KeyboardTab
│   ├── MouseTab
│   └── TouchpadTab
├── Button controls (Wiki, Apply, Close)
└── Configuration I/O methods
```

## Configuration Interaction

- **Default path**: `$XDG_CONFIG_HOME/lxqt/wayland/niri/basicsettings.kdl`
- Uses regex pattern matching to parse KDL
- Preserves commented-out settings (prefixed with `//`)
- Automatically creates parent directories

## Key Observations for Rust Port

1. **Modular tab design** - Easy to replicate in Slint
2. **Regex-based parsing** - Rust has excellent regex support
3. **Fixed window size** - Simple layout, no complex responsive design needed
4. **XDG compliance** - Rust has `dirs` crate for this
5. **Color pickers** - Will need custom Slint component or external crate
