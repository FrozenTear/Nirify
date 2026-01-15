# Niri Configuration System

## Config File Locations

| Priority | Location | Purpose |
|----------|----------|---------|
| Primary | `~/.config/niri/config.kdl` | User configuration |
| Fallback | `/etc/niri/config.kdl` | System-wide defaults |

Environment variable `$XDG_CONFIG_HOME` can override the default `~/.config` location.

## Configuration Format: KDL

Niri uses **KDL (KDL Document Language)** for configuration. This is a structured, human-readable format.

Example structure:
```kdl
input {
    keyboard {
        xkb {
            layout "us"
        }
    }
    touchpad {
        tap
        natural-scroll
    }
}

layout {
    gaps 16
    focus-ring {
        width 4
    }
}
```

## Configuration Sections

| Section | Description |
|---------|-------------|
| `input` | Keyboards, mice, touchpads |
| `outputs` | Display/monitor configuration |
| `binds` | Key bindings |
| `switch-events` | Event-triggered actions |
| `layout` | Workspace and window arrangement |
| `named-workspaces` | Custom workspace definitions |
| `spawn-at-startup` | Programs to start with niri |
| `window-rules` | Per-application behaviors |
| `layer-rules` | Layer-shell component handling |
| `animations` | Visual effect timing |
| `gestures` | Touchpad and mouse gestures |

## Multi-File Configuration (CRITICAL)

**Niri supports the `include` directive** to split configuration across multiple files.

This feature was added in niri 25.11 and is exactly what enables a "takeover" approach:

```kdl
// In main config.kdl
include "~/.config/niri/settings.kdl"
include "~/.config/niri/keybinds.kdl"
include "~/.config/niri/appearance.kdl"
```

### Implications for niri-settings-rust

1. **Non-destructive editing** - We can manage our own file without touching user's main config
2. **Clean separation** - User's custom bindings stay in their file, our settings in ours
3. **Easy rollback** - User can remove include line to disable our settings

## Applying Configuration Changes

### IPC Communication
- Use `niri msg` command to communicate with running niri
- JSON responses available with `--json` flag
- Direct socket at `$NIRI_SOCKET`

### Reload Behavior
- **Hot reload**: Some settings apply immediately via IPC
- **Session restart**: Some changes require session restart
- Application restarts may be needed for per-app rules

## Rust Ecosystem Support

| Need | Crate |
|------|-------|
| KDL Parsing | `kdl` (official KDL parser) |
| XDG Paths | `dirs` or `xdg` |
| IPC/Socket | `tokio` or standard library |
| Process Communication | `niri-ipc` (if available) |
