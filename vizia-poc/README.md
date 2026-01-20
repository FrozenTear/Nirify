# Niri Settings - Vizia Proof of Concept

**Phase 0** of the Slint → Vizia migration.

## Goal

Validate that Vizia provides significant compile time improvements over Slint before committing to full migration.

## What's Included

This PoC implements:
- ✅ AppState with Lens pattern (reactive data binding)
- ✅ 3 settings pages: Keyboard, Mouse, Touchpad
- ✅ Sidebar navigation with panel switching
- ✅ Sample settings: toggles, sliders, dropdowns
- ✅ Dark theme (Catppuccin Mocha inspired)
- ✅ Event-driven architecture (Model::event pattern)

## Compile Time Measurement

### Step 1: Measure Slint Baseline

From the main niri-tweaks directory:

```bash
cd /home/user/niri-tweaks

# Clean build
cargo clean
time cargo build

# Record the time (e.g., "real 3m45s")

# Incremental build (touch a UI file)
touch ui/pages/keyboard.slint
time cargo build

# Record the time (e.g., "real 0m12s")
```

### Step 2: Measure Vizia PoC

From the vizia-poc directory:

```bash
cd /home/user/niri-tweaks/vizia-poc

# Clean build
cargo clean
time cargo build

# Record the time (e.g., "real 1m30s")

# Incremental build (touch a UI file)
touch src/ui/keyboard_page.rs
time cargo build

# Record the time (e.g., "real 0m5s")
```

### Step 3: Compare Results

**Decision Matrix:**

| Improvement | Recommendation |
|-------------|---------------|
| < 1 minute saved | **ABORT** - Not worth migration effort |
| 1-2 minutes saved | **CONSIDER** - Marginal benefit, depends on priorities |
| 2-3 minutes saved | **PROCEED** - Good ROI for 6-9 week migration |
| > 3 minutes saved | **STRONGLY RECOMMEND** - Excellent long-term productivity gain |

## Running the PoC

```bash
cd vizia-poc
cargo run
```

Expected behavior:
- Window opens with sidebar + keyboard settings page
- Click "Mouse" or "Touchpad" in sidebar to switch pages
- Change settings (sliders, toggles, dropdowns)
- Click "Save Settings" to mark as saved (not actually persisted in PoC)
- Status message appears at bottom of sidebar

## What to Test

### Functionality Checklist

- [ ] Window launches without errors
- [ ] Sidebar navigation works (clicking switches pages)
- [ ] Sliders respond to drag (values update)
- [ ] Toggles work (checkboxes change state)
- [ ] Dropdowns work (can select options)
- [ ] Save button enables when changes made
- [ ] Save button shows status message when clicked
- [ ] Dark mode toggle works (visual change)

### UI Quality Checklist

- [ ] Text is readable (no font rendering issues)
- [ ] Layout is clean (no overlapping elements)
- [ ] Colors match dark theme expectations
- [ ] Hover states work (buttons darken on hover)
- [ ] No performance issues (smooth 60fps)

## Known Limitations (PoC Only)

- ❌ Settings are NOT saved to disk (stub implementation)
- ❌ No IPC integration with niri
- ❌ Only 3 pages implemented (out of 27)
- ❌ No complex widgets (ColorPicker, GradientPicker, KeyCapture)
- ❌ No dialogs or modals
- ❌ No search functionality
- ❌ No dynamic lists (keybindings, window rules)
- ⚠️ Theme is embedded CSS (not switchable light/dark)

## Architecture Highlights

### Lens Pattern (Reactive Bindings)

```rust
// In AppState
#[derive(Lens)]
pub struct AppState {
    pub keyboard: KeyboardSettings,
}

// In UI
Label::new(cx, AppState::keyboard.then(|k| format!("{}", k.repeat_rate)))
```

When `keyboard.repeat_rate` changes, the label automatically updates. No manual sync needed!

### Event Pattern

```rust
// Emit event
cx.emit(AppEvent::SetKeyboardRepeatRate(50));

// Handle in Model::event
AppEvent::SetKeyboardRepeatRate(val) => {
    self.keyboard.repeat_rate = *val;
}
```

All state mutations go through events, making data flow explicit and debuggable.

### Page Switching

```rust
Binding::new(cx, AppState::current_panel, |cx, panel| {
    match panel.get(cx) {
        Panel::Keyboard => build_keyboard_page(cx),
        Panel::Mouse => build_mouse_page(cx),
        Panel::Touchpad => build_touchpad_page(cx),
    }
});
```

When `current_panel` changes, Vizia automatically rebuilds the content area with the new page.

## Next Steps

### If Compile Time Improvement is Acceptable

1. Review `VIZIA_MIGRATION_ANALYSIS.md` in parent directory
2. Proceed with Phase 1: Core Infrastructure
3. Port DynamicSettingsSection pattern
4. Implement 10+ simple settings pages

### If Compile Time Improvement is Insufficient

1. Stay with Slint
2. Investigate other optimizations:
   - Split into multiple crates
   - Use `sccache` for compilation caching
   - Profile compile times with `cargo build --timings`
   - Consider simpler UI toolkit (e.g., egui)

## Code Structure

```
vizia-poc/
├── Cargo.toml              # Dependencies (Vizia 0.2)
├── README.md               # This file
└── src/
    ├── main.rs             # Application entry point + theme
    ├── app_state.rs        # AppState + events + Lens
    ├── types.rs            # Shared enums (AccelProfile, etc.)
    ├── constants.rs        # Value bounds (MIN/MAX)
    └── ui/
        ├── mod.rs          # UI module exports
        ├── sidebar.rs      # Navigation sidebar
        ├── keyboard_page.rs
        ├── mouse_page.rs
        └── touchpad_page.rs
```

## Troubleshooting

### "error: no matching package named vizia found"

Vizia 0.2 might not be published yet. Check crates.io and update Cargo.toml to latest version:

```bash
cargo search vizia
# Update Cargo.toml with latest version
```

### "cannot find macro `Lens` in this scope"

Add `use vizia::prelude::*;` at top of file.

### Compilation errors about trait bounds

Make sure all state structs have:
- `#[derive(Debug, Clone, Lens)]` for structs
- `#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]` for enums

### Window doesn't open

Check logs in terminal for error messages. Run with:

```bash
RUST_LOG=debug cargo run
```

## Questions?

See `VIZIA_MIGRATION_ANALYSIS.md` in parent directory for:
- Full migration plan (Phases 1-5)
- Risk assessment
- Effort estimates (6-9 weeks)
- Code reuse analysis (37% reusable as-is)

---

**Remember:** This is just a proof of concept. The goal is to measure compile times and validate the Vizia approach, not to build a production-ready app.
