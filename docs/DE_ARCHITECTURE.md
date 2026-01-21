# Niri Desktop Environment Architecture

Design document for a cohesive desktop environment built on niri compositor using Spell framework and Slint UI.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        niri (compositor)                        │
│                    Wayland protocol + IPC socket                │
└─────────────────────────────────────────────────────────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│    Panel      │   │   Launcher    │   │ niri-settings │
│  (Spell/Slint)│   │ (Spell/Slint) │   │    (Slint)    │
└───────────────┘   └───────────────┘   └───────────────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             ▼
                 ┌───────────────────────┐
                 │   Shared Theme Spec   │
                 │   (Catppuccin-based)  │
                 └───────────────────────┘
```

## Components

### Core Applications

| Component | Framework | Purpose | Config Location |
|-----------|-----------|---------|-----------------|
| niri | - | Wayland compositor | `~/.config/niri/config.kdl` |
| niri-settings | Slint | Compositor configuration GUI | `~/.config/niri/niri-settings/` |
| niri-panel | Spell/Slint | Top/bottom bar, system tray | `~/.config/niri-panel/` |
| niri-launcher | Spell/Slint | Application launcher | `~/.config/niri-launcher/` |

### Supporting Components (External)

| Component | Purpose | Notes |
|-----------|---------|-------|
| Polkit agent | Authentication dialogs | User's choice (already selected) |
| Notification daemon | Desktop notifications | Could be custom Spell widget later |
| Screen locker | Session lock | swaylock or custom |

## Configuration Strategy

### Separate Config Directories

Each component maintains its own configuration for isolation and independent development:

```
~/.config/
├── niri/
│   ├── config.kdl                    # User's main niri config
│   └── niri-settings/                # Managed by niri-settings
│       ├── main.kdl
│       ├── appearance.kdl
│       └── ...
├── niri-panel/
│   ├── config.kdl                    # Panel layout, modules, position
│   └── theme.kdl                     # Panel-specific overrides (optional)
├── niri-launcher/
│   ├── config.kdl                    # Search providers, shortcuts
│   └── theme.kdl                     # Launcher-specific overrides (optional)
└── niri-de/
    └── theme.kdl                     # Shared theme definition (source of truth)
```

### Shared Theme Specification

All components read from a shared theme file for visual consistency:

```kdl
// ~/.config/niri-de/theme.kdl

palette {
    // Catppuccin Mocha base
    base        "#1e1e2e"
    mantle      "#181825"
    crust       "#11111b"
    surface0    "#313244"
    surface1    "#45475a"
    surface2    "#585b70"
    overlay0    "#6c7086"
    overlay1    "#7f849c"
    overlay2    "#9399b2"
    text        "#cdd6f4"
    subtext0    "#a6adc8"
    subtext1    "#bac2de"

    // Accent colors
    rosewater   "#f5e0dc"
    flamingo    "#f2cdcd"
    pink        "#f5c2e7"
    mauve       "#cba6f7"
    red         "#f38ba8"
    maroon      "#eba0ac"
    peach       "#fab387"
    yellow      "#f9e2af"
    green       "#a6e3a1"
    teal        "#94e2d5"
    sky         "#89dceb"
    sapphire    "#74c7ec"
    blue        "#89b4fa"
    lavender    "#b4befe"
}

accent "mauve"  // User-selectable primary accent

typography {
    font-family "Inter"
    font-family-mono "JetBrains Mono"

    // Scale
    size-xs     10
    size-sm     12
    size-base   14
    size-lg     16
    size-xl     20
    size-2xl    24
}

spacing {
    unit        4       // Base unit in px
    xs          4       // 1 unit
    sm          8       // 2 units
    md          12      // 3 units
    lg          16      // 4 units
    xl          24      // 6 units
}

radii {
    none        0
    sm          4
    md          8
    lg          12
    full        9999
}
```

### Theme Loading

Each component includes a shared Rust crate for theme parsing:

```rust
// niri-de-theme crate (shared)

pub struct Theme {
    pub palette: Palette,
    pub accent: String,
    pub typography: Typography,
    pub spacing: Spacing,
    pub radii: Radii,
}

impl Theme {
    pub fn load() -> Result<Self> {
        let path = dirs::config_dir()
            .unwrap()
            .join("niri-de/theme.kdl");
        // Parse and return
    }

    pub fn accent_color(&self) -> &str {
        self.palette.get(&self.accent)
    }
}
```

## Inter-Process Communication

### niri IPC

All components communicate with niri via its existing IPC socket:

```rust
// Request window list, workspace info, etc.
let socket_path = std::env::var("NIRI_SOCKET")?;
let mut stream = UnixStream::connect(socket_path)?;
```

Used for:
- Workspace switching (panel)
- Window focusing (launcher)
- Config reload (niri-settings)

### Component-to-Component Communication

For cases where DE components need to talk to each other:

**Option A: D-Bus (Recommended)**
- Standard Linux IPC
- Well-supported in Rust (zbus crate)
- Allows external tools to integrate

```rust
// Panel requests launcher to open
conn.call_method(
    Some("org.niri.Launcher"),
    "/org/niri/Launcher",
    Some("org.niri.Launcher"),
    "Toggle",
    &(),
)?;
```

**Option B: Simple Unix Sockets**
- Lighter weight
- Each component exposes a socket in `$XDG_RUNTIME_DIR`

### Communication Use Cases

| From | To | Purpose | Method |
|------|----|---------|--------|
| Panel | Launcher | Toggle visibility | D-Bus |
| Panel | niri | Switch workspace | niri IPC |
| niri-settings | niri | Reload config | niri IPC |
| niri-settings | Panel | Theme changed signal | D-Bus (optional) |
| Keybind | Launcher | Open launcher | D-Bus |

## Panel Architecture

### Modules

The panel is composed of pluggable modules:

```kdl
// ~/.config/niri-panel/config.kdl

position "top"
height 32

left {
    workspaces {
        show-names false
        show-numbers true
    }
    window-title {
        max-length 50
    }
}

center {
    clock {
        format "%a %b %d  %H:%M"
    }
}

right {
    systray {}
    network {}
    audio {
        show-icon true
        show-percentage false
    }
    battery {}
}
```

### Module Interface

```rust
pub trait PanelModule {
    fn name(&self) -> &str;
    fn render(&self, theme: &Theme) -> slint::Component;
    fn update(&mut self) -> Result<()>;
    fn on_click(&mut self, button: MouseButton);
}
```

## Launcher Architecture

### Search Providers

```kdl
// ~/.config/niri-launcher/config.kdl

providers {
    applications {
        priority 1
        show-actions true
    }

    calculator {
        priority 2
        prefix "="
    }

    files {
        priority 3
        prefix "/"
        max-results 5
    }

    web {
        priority 4
        prefix "?"
        engine "duckduckgo"
    }
}

appearance {
    width 600
    max-results 8
}

shortcuts {
    terminal "foot"
    browser "firefox"
    files "nautilus"
}
```

## Development Workflow

### Repository Structure

Two options:

**Monorepo (Recommended for tight integration)**
```
niri-de/
├── crates/
│   ├── niri-de-theme/      # Shared theme parsing
│   ├── niri-de-ipc/        # Shared IPC utilities
│   ├── niri-panel/         # Panel application
│   ├── niri-launcher/      # Launcher application
│   └── niri-settings/      # Settings application (existing)
├── ui/
│   ├── shared/             # Shared Slint components
│   ├── panel/
│   ├── launcher/
│   └── settings/
└── Cargo.toml              # Workspace
```

**Separate repos (Current approach)**
```
niri-settings-rust/         # Existing
niri-panel/                 # New repo
niri-launcher/              # New repo
niri-de-theme/              # Shared crate, published to crates.io
```

### Build & Release

Each component builds independently but shares:
- Theme crate (`niri-de-theme`)
- Slint widget library (optional)
- CI/CD patterns

## Migration Path

### Phase 1: Foundation
- [ ] Create `niri-de-theme` crate with Catppuccin palette
- [ ] Update niri-settings to use shared theme
- [ ] Define theme.kdl specification

### Phase 2: Panel
- [ ] Scaffold niri-panel with Spell
- [ ] Implement core modules: workspaces, clock, systray
- [ ] Add niri IPC integration

### Phase 3: Launcher
- [ ] Scaffold niri-launcher with Spell
- [ ] Implement application search
- [ ] Add D-Bus interface for panel integration

### Phase 4: Polish
- [ ] Unified installer/setup wizard
- [ ] Documentation and user guide
- [ ] Theme editor in niri-settings

## Open Questions

1. **Notification daemon**: Build custom with Spell or use existing (mako, dunst)?
2. **Session management**: How to handle login/logout/lock coordination?
3. **Portals**: Which xdg-desktop-portal backend to use/build?
4. **OSD**: On-screen display for volume/brightness - panel module or separate?

---

*Document version: 0.1 - Draft*
