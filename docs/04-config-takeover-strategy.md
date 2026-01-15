# Config Takeover Strategy

## What is "Takeover"?

A takeover approach means niri-settings-rust would **own and control** specific configuration files rather than just editing the user's main config.

## Feasibility: CONFIRMED POSSIBLE

Niri's `include` directive (added in v25.11) makes this completely feasible.

## Architecture: Multi-File Approach

We use a **multi-file architecture** where each settings category has its own file. This makes debugging easier and allows users to selectively disable categories.

### File Structure

```
~/.config/niri/
├── config.kdl                      # User's main config (we add ONE include line)
└── niri-settings/                  # Our managed directory
    ├── main.kdl                    # Master file (includes all category files)
    │
    │   # CORE SETTINGS (Phase 2-3)
    ├── appearance.kdl              # Layout: gaps, focus-ring, border, struts
    ├── behavior.kdl                # Focus follows mouse, mouse warp, mod key
    ├── input/
    │   ├── keyboard.kdl            # XKB layout, repeat rate, numlock
    │   ├── mouse.kdl               # Acceleration, scroll, left-handed
    │   └── touchpad.kdl            # Tap, gestures, scroll method, DWT
    │
    │   # DISPLAY & VISUAL (Phase 5)
    ├── outputs.kdl                 # Monitor: scale, resolution, position, VRR
    ├── animations.kdl              # Animation speeds, curves, enable/disable
    ├── cursor.kdl                  # Cursor theme, size, hide when typing
    ├── overview.kdl                # Overview zoom, backdrop color
    │
    │   # ADVANCED SETTINGS (Phase 6)
    ├── advanced/
    │   ├── layout-extras.kdl       # Shadow, tab-indicator, insert-hint, presets
    │   ├── gestures.kdl            # Hot corners, DND edge scroll
    │   ├── window-rules.kdl        # Per-app rules (floating, opacity, size)
    │   └── misc.kdl                # Screenshot path, CSD, clipboard, startup
    │
    └── .backup/                    # Timestamped backups
        └── ...
```

### Integration with User Config

We add ONE line to user's `config.kdl`:

```kdl
// niri-settings-rust managed settings
include "~/.config/niri/niri-settings/main.kdl"
```

### Our main.kdl Structure

```kdl
// niri-settings-rust managed configuration
// Do not edit manually - changes will be overwritten

// Core settings
include "appearance.kdl"
include "behavior.kdl"
include "input/keyboard.kdl"
include "input/mouse.kdl"
include "input/touchpad.kdl"

// Display & visual
include "outputs.kdl"
include "animations.kdl"
include "cursor.kdl"
include "overview.kdl"

// Advanced (commented out by default, user can enable)
// include "advanced/layout-extras.kdl"
// include "advanced/gestures.kdl"
// include "advanced/window-rules.kdl"
// include "advanced/misc.kdl"
```

### Example Category Files

**appearance.kdl:**
```kdl
// Appearance settings - managed by niri-settings-rust

layout {
    gaps 16

    focus-ring {
        width 4
        active-color "#7fc8ff"
        inactive-color "#505050"
    }

    border {
        width 2
        active-color "#ffc87f"
        inactive-color "#303030"
    }

    struts {
        left 0
        right 0
        top 0
        bottom 0
    }
}
```

**input/touchpad.kdl:**
```kdl
// Touchpad settings - managed by niri-settings-rust

input {
    touchpad {
        tap
        natural-scroll
        accel-speed 0.2
        accel-profile "adaptive"
    }
}
```

## Benefits of Multi-File Approach

| Benefit | Description |
|---------|-------------|
| **Clean separation** | User's custom config untouched |
| **Easy debugging** | "Touchpad broken? Check input/touchpad.kdl" |
| **Selective disable** | Comment out one include to disable a category |
| **Small files** | Each file is readable at a glance |
| **Matches UI** | One tab = one file, clear mental model |
| **Better diffs** | Smaller changes when updating |
| **No merge conflicts** | We own our files completely |
| **Backup safety** | Per-file backups |

## Implementation Strategy

### First Run
1. Check if `~/.config/niri/niri-settings/` exists
2. If not, create directory structure including `input/` subdirectory
3. Check if `config.kdl` has our include line for `main.kdl`
4. If not, prompt user to add it (or offer to add automatically)
5. Create all initial `.kdl` files with defaults
6. Generate `main.kdl` with includes for all category files

### Normal Operation
1. Read all managed files
2. Display current settings in UI (each tab loads its file)
3. On "Apply", write changes to relevant category files only
4. Trigger niri reload via IPC

### Conflict Handling

If user has same setting in both their config and ours:
- **Niri behavior**: Later includes override earlier ones
- **Our strategy**: Place our include at the END of their config
- **Alternative**: Detect conflicts and warn user

## Safety Measures

### Backup System (Per-File)
```rust
fn save_category(&self, category: Category) -> Result<()> {
    let file_path = self.paths.get_path(category);
    let backup_name = format!(
        "{}.{}.bak",
        category.filename(),
        chrono::Local::now().format("%Y-%m-%dT%H-%M-%S")
    );
    let backup_path = self.paths.backup_dir.join(backup_name);

    // 1. Create timestamped backup
    if file_path.exists() {
        fs::copy(&file_path, &backup_path)?;
    }

    // 2. Generate and validate KDL
    let kdl_content = category.to_kdl();
    let _: KdlDocument = kdl_content.parse()
        .context("Generated invalid KDL")?;

    // 3. Write new settings
    fs::write(&file_path, kdl_content)?;

    // 4. Clean old backups (keep last 5)
    self.cleanup_old_backups(category)?;

    Ok(())
}
```

### Validation
- Parse our generated KDL before saving
- Optionally use `niri validate` if available
- Keep backup if validation fails

### Rollback
```rust
fn rollback(&self, category: Category) -> Result<()> {
    let backups = self.list_backups(category)?;
    if let Some(latest) = backups.first() {
        fs::copy(&latest.path, &self.paths.get_path(category))?;
    }
    Ok(())
}
```

## User Consent Flow

```
┌─────────────────────────────────────────────────┐
│  niri-settings-rust - First Run Setup           │
├─────────────────────────────────────────────────┤
│                                                 │
│  Welcome! This app helps you configure niri     │
│  without editing config files manually.         │
│                                                 │
│  Settings will be saved in:                     │
│  ~/.config/niri/niri-settings/                  │
│                                                 │
│  To enable, add this line to your config.kdl:   │
│                                                 │
│  include "~/.config/niri/niri-settings/main.kdl"│
│                                                 │
│  [Add Automatically]  [I'll Do It Manually]     │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Disabling Categories

Users can disable specific categories by editing `main.kdl`:

```kdl
// main.kdl
include "appearance.kdl"
include "behavior.kdl"
// include "animations.kdl"        // Disabled
include "input/keyboard.kdl"
include "input/mouse.kdl"
// include "input/touchpad.kdl"    // Disabled - using external mouse only
```

Or by removing the entire integration:

```kdl
// In user's config.kdl, comment out:
// include "~/.config/niri/niri-settings/main.kdl"
```

## Conclusion

The multi-file takeover approach is:
- **Feasible**: Niri's include directive supports nested includes
- **Safe**: Non-destructive to user's main config
- **Clean**: Clear ownership boundaries, one file per category
- **Debuggable**: Easy to identify which file caused issues
- **Flexible**: Users can disable individual categories
- **Recommended**: Best approach for a settings management app
