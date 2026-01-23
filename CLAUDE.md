# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Native Rust settings application for the [niri](https://github.com/YaLTeR/niri) Wayland compositor. Uses **iced 0.14** for UI and manages KDL config files without modifying the user's main config directly.

**Target users**: Non-technical users who don't want to edit KDL config files manually.

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
├── main.rs                    # App entry point
├── lib.rs                     # Library exports
├── app/
│   ├── mod.rs                 # App struct, update(), view()
│   ├── ui_state.rs            # UI-only state (selections, dialogs, etc.)
│   └── handlers/              # Message handlers (one file per settings category)
├── views/                     # View functions (pure UI construction)
│   ├── mod.rs                 # View exports
│   ├── widgets/               # Reusable widget components
│   └── <category>.rs          # One file per settings page
├── messages.rs                # All Message enums
├── config/
│   ├── models/                # Settings struct hierarchy
│   ├── loader/                # Load settings from KDL files
│   ├── storage/               # Save settings to KDL files
│   ├── paths.rs               # XDG path resolution
│   └── validation.rs          # Pre-save validation
├── ipc/
│   ├── mod.rs                 # Niri socket communication (sync)
│   └── tasks.rs               # Async IPC helpers returning iced::Task
├── save_manager.rs            # Debounced auto-save
├── search.rs                  # Search indexing
├── theme.rs                   # Theme definitions
├── types.rs                   # Shared enums (AccelProfile, ModKey, Color, etc.)
└── constants.rs               # Value bounds (MIN/MAX), defaults
```

### Key Patterns

**Settings flow**: User interaction → Message → `update()` handler → modify `Settings` → mark dirty → `SaveManager` debounces → `config/storage/` writes KDL

**Adding a new setting**:
1. Add field to appropriate struct in `config/models/<category>.rs`
2. Add loader in `config/loader/<category>.rs`
3. Add storage in `config/storage/<category>.rs`
4. Add message variant in `messages.rs`
5. Add handler in `app/handlers/<category>.rs`
6. Add UI in `views/<category>.rs`

**Async IPC**: Use helpers from `ipc::tasks` module:
```rust
Message::CheckNiri => crate::ipc::tasks::check_niri_running(Message::NiriStatusChecked)
```

## Conventions

### iced Code Style

```rust
// State: All application data
struct App {
    settings: Settings,
    ui: UiState,
    // ...
}

// Messages: All possible events
#[derive(Debug, Clone)]
enum Message { /* variants */ }

// Update: Handle state changes, return Task for async work
fn update(&mut self, message: Message) -> Task<Message> { /* logic */ }

// View: Construct UI from state (pure function)
fn view(&self) -> Element<Message> { /* widgets */ }
```

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
| tokio | Async runtime for iced Tasks |

## Links

- [Niri](https://github.com/YaLTeR/niri)
- [iced Website](https://iced.rs/)
- [iced Documentation](https://book.iced.rs/)
- [iced GitHub](https://github.com/iced-rs/iced)
- [KDL Spec](https://kdl.dev)
- **API Reference**: `docs/ICED_API.md` (comprehensive iced 0.14 guide)
