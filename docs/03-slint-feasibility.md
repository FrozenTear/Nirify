# Slint Feasibility Analysis

## What is Slint?

Slint is a declarative GUI toolkit written 100% in Rust. It uses a custom DSL (`.slint` files) that compiles to native Rust code at compile time.

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Runtime memory | < 300 KiB |
| Compilation | Native code (no JIT) |
| Rendering | Software, OpenGL, or Skia |
| Property system | Lazy & reactive |

**Key advantages:**
- No garbage collection pauses
- Compile-time optimization of UI
- Bindings only re-evaluate when dependencies change

## Available Widgets

Perfect for a settings application:

| Widget | Use Case |
|--------|----------|
| `LineEdit` | Text input |
| `TextEdit` | Multi-line text |
| `CheckBox` | Boolean toggles |
| `Switch` | On/off semantics |
| `Button` | Actions |
| `Slider` | Numeric values |
| `ComboBox` | Dropdown selections |
| `SpinBox` | Numeric input with +/- |
| `TabWidget` | Tabbed interface |

## Data Binding

Slint's reactive property system is ideal for settings apps:

```slint
export component SettingsPage {
    in-out property <int> gap-size: 16;

    Slider {
        value <=> gap-size;
        minimum: 0;
        maximum: 64;
    }
}
```

Two-way binding (`<=>`) automatically syncs UI with Rust state.

## Development Experience

### IDE Support
- VS Code extension with live preview
- LSP support for most editors
- Auto-complete and diagnostics

### Live Preview
```bash
slint-viewer --auto-reload myui.slint
```

See changes instantly without recompilation.

### Clear Separation
- UI in `.slint` files
- Logic in Rust
- Type-safe bindings between them

## Rendering Backends

| Backend | Description |
|---------|-------------|
| Software | Pure CPU, no GPU dependencies |
| femtovg | OpenGL ES 2.0 |
| Skia | Advanced graphics library |

For a settings app, software renderer is sufficient and simplest.

## Built-in Styles

| Style | Description |
|-------|-------------|
| Fluent | Microsoft Fluent Design (default) |
| Material | Google Material Design |
| Native | Uses Qt's QStyle if available |

All look professional out of the box.

## Limitations

1. **Non-native widgets** - Slint draws its own (but they look good)
2. **Smaller ecosystem** - Fewer community widgets than Qt/GTK
3. **New framework** - Less Stack Overflow coverage
4. **Color picker** - Not built-in (needs custom implementation)

## Licensing

| License | Terms |
|---------|-------|
| GPLv3 | Open source, copyleft |
| Royalty-Free | Attribution only |
| Commercial | Paid, more flexibility |

For an open-source niri-settings app, royalty-free license is appropriate.

## Comparison with Alternatives

| Framework | Pros | Cons |
|-----------|------|------|
| **Slint** | Rust-native, lightweight, fast | Newer, smaller ecosystem |
| gtk-rs | Mature, native on Linux | Complex bindings, larger |
| iced | Pure Rust, Elm-like | Less widgets, retained mode |
| egui | Immediate mode, simple | Not for traditional apps |

## Verdict: EXCELLENT FIT

Slint is an excellent choice for niri-settings because:

1. **Right-sized** - Has exactly the widgets a settings app needs
2. **Lightweight** - Matches niri's minimal philosophy
3. **Rust-native** - No FFI complexity
4. **Reactive** - Perfect for settings that need two-way binding
5. **Fast iteration** - Live preview speeds development
6. **Professional look** - Built-in styles look good
7. **Active development** - Regular releases, responsive maintainers
