# niri-settings-rust Feasibility Summary

## Executive Summary

**VERDICT: HIGHLY FEASIBLE**

Building a niri-settings application in Rust with Slint is not only possible but offers significant advantages over the existing Python/PyQt6 implementation.

## Key Findings

### 1. Niri Configuration System

| Aspect | Status | Notes |
|--------|--------|-------|
| Config format | KDL | Rust has excellent `kdl` crate |
| Multi-file support | Yes | `include` directive in niri 25.11+ |
| Hot reload | Partial | Some settings need restart |
| IPC available | Yes | `niri msg` or socket at `$NIRI_SOCKET` |

### 2. Slint UI Framework

| Aspect | Status | Notes |
|--------|--------|-------|
| Widget coverage | Excellent | All needed widgets available |
| Performance | Excellent | < 300 KiB runtime |
| Development experience | Good | Live preview, LSP support |
| Styling | Good | Fluent/Material built-in |
| Licensing | OK | Royalty-free option available |

### 3. Takeover Strategy

| Aspect | Status | Notes |
|--------|--------|-------|
| Feasibility | Confirmed | Via niri's `include` directive |
| Safety | High | Non-destructive to user config |
| Complexity | Low | Simple file management |

## Advantages Over Python Version

| Aspect | Python (PyQt6) | Rust (Slint) |
|--------|----------------|--------------|
| Binary size | Large (Qt deps) | Small (~few MB) |
| Memory usage | Higher | < 300 KiB runtime |
| Startup time | Slower (interpreter) | Fast (native) |
| Dependencies | Python + Qt | Single binary |
| Type safety | Runtime errors | Compile-time |
| Distribution | pip/package managers | Single binary |

## Recommended Architecture

```
niri-settings-rust/
├── src/
│   ├── main.rs              # Entry point
│   ├── config/
│   │   ├── mod.rs           # Config module
│   │   ├── kdl_parser.rs    # KDL reading/writing
│   │   ├── settings.rs      # Settings data structures
│   │   └── paths.rs         # XDG path handling
│   ├── ui/
│   │   ├── mod.rs           # UI module
│   │   └── bridge.rs        # Slint-Rust bindings
│   └── ipc/
│       └── mod.rs           # Niri IPC communication
├── ui/
│   ├── main.slint           # Main window
│   ├── appearance.slint     # Appearance tab
│   ├── behavior.slint       # Behavior tab
│   ├── input.slint          # Input tabs
│   └── widgets/
│       └── color_picker.slint  # Custom widgets
├── Cargo.toml
└── build.rs                 # Slint compilation
```

## Required Crates

```toml
[dependencies]
slint = "1.7"
kdl = "4.0"
dirs = "5.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt", "net"] }  # For IPC
anyhow = "1.0"

[build-dependencies]
slint-build = "1.7"
```

## Development Phases

### Phase 1: Foundation
- Project setup with Slint
- Basic window with tabs
- KDL parsing infrastructure
- XDG path handling

### Phase 2: Core Settings
- Appearance settings (focus ring, border, gaps)
- Read from and write to managed config file
- Basic save/apply functionality

### Phase 3: Input Devices
- Keyboard settings
- Mouse settings
- Touchpad settings

### Phase 4: Integration
- Niri IPC for reloading
- First-run setup flow
- Backup system

### Phase 5: Polish
- Color picker widget
- Error handling UI
- i18n support (optional)

## Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Slint missing widget | Low | Custom widgets are easy to build |
| KDL format changes | Low | Use official kdl crate |
| Niri API changes | Medium | Abstract IPC layer |
| Color picker complexity | Medium | Use external crate or simple RGB input |

## Conclusion

This project is well-suited for Rust + Slint:

1. **Right tool for the job** - Settings apps are Slint's sweet spot
2. **Better user experience** - Faster, lighter than Python version
3. **Cleaner architecture** - Takeover approach via includes
4. **Type safety** - Compile-time guarantees for config handling
5. **Single binary** - Easy distribution

**Recommendation: Proceed with development.**
