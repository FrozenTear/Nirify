# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Native Rust settings application for the [niri](https://github.com/YaLTeR/niri) Wayland compositor. Uses Floem for UI and manages KDL config files without modifying the user's main config directly.

**Target users**: Non-technical users who don't want to edit KDL config files manually.

## Build Commands

```bash
cargo check              # Fast validation (prefer over cargo build)
cargo run                # Run in debug mode
cargo test               # Run all tests
cargo test <test_name>   # Run single test
cargo clippy             # Lint
cargo fmt                # Format code
cargo build --release    # Release binary
```

## Architecture

### Takeover Strategy

The app doesn't edit `config.kdl` directly. Instead:
1. Manages config files in `~/.config/niri/niri-settings/`
2. User adds one line to their config: `include "~/.config/niri/niri-settings/main.kdl"`
3. Each settings category = one `.kdl` file (appearance.kdl, behavior.kdl, input/keyboard.kdl, etc.)

### Code Structure

```
src/
├── main.rs                    # App initialization, Floem launch
├── lib.rs                     # Library exports
├── constants.rs               # Value bounds (MIN/MAX), defaults
├── types.rs                   # Shared enums (AccelProfile, ModKey, Color, etc.)
├── config/
│   ├── models/                # Settings struct hierarchy (Settings -> AppearanceSettings, etc.)
│   ├── paths.rs               # XDG path resolution, include line detection
│   ├── parser.rs              # KDL parsing utilities
│   ├── loader/                # Load settings from KDL files (one file per category)
│   └── storage/               # Save settings to KDL files (one file per category)
├── ui/
│   ├── app.rs                 # Main app composition, page routing
│   ├── state.rs               # AppState with RwSignal-based reactivity
│   ├── theme.rs               # Theme colors (Catppuccin Mocha), styling helpers
│   ├── components/            # Reusable UI components
│   │   ├── setting_rows.rs    # toggle_row, slider_row, color_row, text_row
│   │   └── section.rs         # Section container with header
│   ├── pages/                 # One .rs file per settings page
│   └── nav/                   # Navigation components
│       ├── sidebar.rs         # Category sidebar navigation
│       ├── header.rs          # App title header
│       ├── footer.rs          # Status bar and close button
│       └── search_bar.rs      # Search input
└── ipc/
    └── mod.rs                 # Niri socket communication (reload_config)
```

### Key Patterns

**Settings flow**: `config/loader/` reads KDL → `Settings` struct → `AppState` wraps in `Arc<Mutex<>>` → UI pages read via `state.get_settings()` → user changes trigger callbacks → `state.update_settings()` modifies Settings → `state.mark_dirty_and_save()` triggers auto-save → `config/storage/` writes KDL

**Reactive UI** (Floem signals):
- `RwSignal<T>` for local UI state that needs reactivity
- `AppState` provides `get_settings()` and `update_settings()` for config access
- Use `*_with_callback` variants (e.g., `toggle_row_with_callback`) to wire auto-save

**Adding a new setting**:
1. Add field to appropriate struct in `config/models/<category>.rs`
2. Add loader in `config/loader/<category>.rs`
3. Add storage in `config/storage/<category>.rs`
4. Add UI in `src/ui/pages/<category>.rs` using setting row components
5. Wire callback to `state.update_settings()` and `state.mark_dirty_and_save()`

## Conventions

### Floem

- Uses custom Catppuccin Mocha theme (defined in `ui/theme.rs`)
- Reactive state via `RwSignal<T>` from `floem::reactive`
- Components return `impl IntoView`
- Styling via `.style()` method chains
- Event handling via `.on_click_stop()`, `.on_event_stop()`, etc.

### State Management

- Settings stored in `Arc<Mutex<Settings>>` inside `AppState`
- `AppState` is cloned into each page (cheap - uses `Arc` internally)
- Use `Rc<dyn Fn()>` for callbacks (Floem is single-threaded)

### KDL

- Generate readable, indented output
- Parse generated KDL to validate before saving
- Backup before overwriting

### Paths

- Use `dirs::config_dir()` for XDG compliance
- Use `fs::create_dir_all()` before writing

## UX Guidelines

- Live preview: Changes apply immediately (no Apply button)
- Auto-save with 300ms debounce
- Human-readable labels ("Window Spacing" not "Gaps")
- Sidebar navigation, not tabs
- Search bar routes to appropriate category

## Dependencies

| Crate | Purpose |
|-------|---------|
| floem | UI framework (reactive, GPU-accelerated) |
| kdl 6.5 | KDL config parsing |
| dirs 6.0 | XDG paths |
| anyhow | Error handling |
| thiserror | Custom error types |
| log/env_logger | Logging |
| chrono | Timestamps for backups |

## Links

- [Niri](https://github.com/YaLTeR/niri)
- [Floem](https://github.com/lapce/floem) - UI framework
- [KDL Spec](https://kdl.dev)
