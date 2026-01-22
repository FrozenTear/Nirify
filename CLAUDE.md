# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Native Rust settings application for the [niri](https://github.com/YaLTeR/niri) Wayland compositor. Uses **iced 0.14** for UI and manages KDL config files without modifying the user's main config directly.

**Target users**: Non-technical users who don't want to edit KDL config files manually.

**Important**: This project is currently in migration from Slint to iced. The old Slint code structure is documented below but is being replaced with iced's Elm Architecture pattern.

## Iced UI Framework

This project uses **iced 0.14**, a cross-platform GUI library for Rust inspired by Elm.

**Comprehensive iced documentation**: See `docs/ICED_API.md` for complete API reference, including:
- The Elm Architecture pattern (State, Messages, Update, View)
- Complete widget catalog with examples
- Theming and styling guide
- Tasks and async operations
- Subscriptions for continuous events
- Best practices and performance optimization
- What's new in iced 0.14

### Key iced Concepts

- **The Elm Architecture**: State → Messages → Update → View
- **Single source of truth**: All state in one struct
- **Pure functions**: Update logic is testable without UI
- **Reactive rendering**: Only redraws when state changes (iced 0.14)
- **Type safety**: Rust's type system prevents impossible states

### Quick Reference

```rust
// Minimal iced 0.14 application
use iced::widget::{button, column, text};

pub fn run() -> iced::Result {
    iced::run(App::update, App::view)
}

#[derive(Default)]
struct App { counter: i32 }

#[derive(Debug, Clone, Copy)]
enum Message { Increment }

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.counter += 1,
        }
    }

    fn view(&self) -> iced::Element<Message> {
        column![
            text(self.counter),
            button("Increment").on_press(Message::Increment),
        ]
        .into()
    }
}
```

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
├── main.rs                    # App initialization, wizard, close handling
├── lib.rs                     # Library exports (re-exports MainWindow from Slint)
├── constants.rs               # Value bounds (MIN/MAX), defaults
├── types.rs                   # Shared enums (AccelProfile, ModKey, Color, etc.)
├── config/
│   ├── models.rs              # Settings struct hierarchy (Settings -> AppearanceSettings, etc.)
│   ├── paths.rs               # XDG path resolution, include line detection
│   ├── parser.rs              # KDL parsing utilities
│   ├── loader/                # Load settings from KDL files (one file per category)
│   └── storage/               # Save settings to KDL files (one file per category)
├── ui/
│   ├── window.rs              # Window state management
│   ├── search.rs              # Search keyword -> category mapping
│   └── bridge/
│       ├── mod.rs             # setup_callbacks() entry point
│       ├── callbacks/         # UI event handlers (one file per settings page)
│       ├── sync.rs            # sync_ui_from_settings() - populate UI from Settings
│       ├── converters.rs      # Slint <-> Rust type conversions
│       ├── indices.rs         # Enum <-> combobox index mappings
│       ├── macros.rs          # Callback registration helpers
│       └── save_manager.rs    # 300ms debounced auto-save
└── ipc/
    └── mod.rs                 # Niri socket communication (reload_config)

ui/                            # Slint UI files
├── main.slint                 # Main window, sidebar navigation
├── styles.slint               # Theme colors (Catppuccin Mocha)
├── pages/                     # One .slint file per settings page
├── widgets/                   # Reusable components (ToggleRow, SliderRow, etc.)
└── dialogs/                   # First-run wizard, error dialog
```

### Key Patterns

**Settings flow**: `config/loader/` reads KDL → `Settings` struct → `bridge/sync.rs` populates UI → user changes trigger callbacks → `bridge/callbacks/` updates Settings → `SaveManager` debounces → `config/storage/` writes KDL

**Callback macros** (in `bridge/macros.rs`): Use these to reduce boilerplate:
- `register_bool_callback!` - Toggle switches
- `register_clamped_callback!` - Sliders with min/max
- `register_string_callback!` - Text inputs
- `register_color_callback!` - Color pickers

**Adding a new setting**:
1. Add field to appropriate struct in `config/models.rs`
2. Add loader in `config/loader/<category>.rs`
3. Add storage in `config/storage/<category>.rs`
4. Add UI in `ui/pages/<category>.slint`
5. Add callback in `ui/bridge/callbacks/<category>.rs`
6. Add sync in `ui/bridge/sync.rs`

## Conventions

### iced (New Architecture)

- **The Elm Architecture**: State, Messages, Update, View
- **Pure functions**: Update logic separated from UI
- **Immutable patterns**: State changes only in `update()` method
- **Type-safe messages**: Enums with `#[derive(Debug, Clone)]`
- **Widgets are values**: Created in `view()`, produce messages via callbacks
- **Reactive rendering**: UI only redraws when state changes

### iced Code Style

```rust
// State: All application data
#[derive(Default)]
struct App { /* fields */ }

// Messages: All possible events
#[derive(Debug, Clone)]
enum Message { /* variants */ }

// Update: Handle state changes
fn update(&mut self, message: Message) { /* logic */ }

// View: Construct UI from state
fn view(&self) -> Element<Message> { /* widgets */ }
```

### Old Slint Architecture (Being Replaced)

- Settings stored in `Arc<Mutex<Settings>>`
- Callbacks clone Arc and lock when needed
- Use `Rc<SaveManager>` for debounced saves

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
| iced 0.14 | UI framework (pure Rust, GPU-accelerated) |
| kdl 6.5 | KDL config parsing |
| dirs 6.0 | XDG paths |
| anyhow | Error handling |
| thiserror | Custom error types |
| log/env_logger | Logging |
| chrono | Timestamps for backups |

## Links

- [Niri](https://github.com/YaLTeR/niri)
- [iced Website](https://iced.rs/)
- [iced Documentation](https://book.iced.rs/)
- [iced GitHub](https://github.com/iced-rs/iced)
- [KDL Spec](https://kdl.dev)
- **API Reference**: `docs/ICED_API.md` (comprehensive iced 0.14 guide)
