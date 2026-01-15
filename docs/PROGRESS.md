# Implementation Progress - niri-settings-rust

**Last Updated**: January 11, 2026

## Current Status: Phase 9 Complete

All tests pass (206 total), builds successfully with no warnings.

**NIRI Settings Coverage: ~85-90%**

---

## Recent Completions (January 2026)

### A-Grade Code Quality Improvements
- ✅ Added `Copy`, `Eq` derives to `EasingCurve` enum
- ✅ Updated `Default` impls to use constants (`DAMPING_RATIO_DEFAULT`, etc.)
- ✅ Added parse-time validation clamping in `parse_spring_params`
- ✅ Unified 6 animation macros into 2 parameterized macros (42% reduction)
- ✅ Batched Arc clones in callbacks (animations, outputs, workspaces, layer_rules)

### IPC Hardening
- ✅ Replaced string-based JSON parsing with typed `serde_json` structs
- ✅ Added `ActionResponse` and `ActionResult` enums with proper error handling
- ✅ Added 10 unit tests for IPC response parsing
- ✅ Debounce documented at 300ms with rationale

### Missing Quick Wins
- ✅ Device disable toggles for keyboard, mouse, touchpad (`off` directive)
- ✅ Keyboard warning dialog for lockout prevention
- ✅ `scroll-button` and `scroll-button-lock` for mouse/touchpad
- ✅ `xkb.file` custom keymap support for keyboard

### Accessibility
- ✅ Added `accessible-role` and `accessible-label` to interactive elements
- ✅ Coverage across 7 pages (animations, keybindings, backups, switch_events, displays, window_rules, layer_rules)

### Backup/Restore
- ✅ Implemented backend callbacks for listing, previewing, and restoring backups
- ✅ Proper date/size formatting, atomic writes, status toasts

---

## Completed Phases

### Phase 1-5: Core Settings (COMPLETE)
- Appearance (focus ring, border, gaps, corner radius)
- Behavior (focus follows mouse, warp mouse, column width, struts)
- Keyboard (XKB layout, repeat rate, numlock, xkb.file)
- Mouse/Touchpad (acceleration, scroll, gestures, scroll-button)
- Outputs (scale, mode, position, VRR)
- Animations (enable/disable, slowdown)
- Cursor (theme, size, hide settings)
- Overview (zoom, backdrop)
- Layout Extras (shadow, tab indicator, insert hint)
- Gestures (hot corners, DND edge scroll)
- Miscellaneous (CSD, screenshot path, clipboard)
- Window Rules (match criteria, opacity, corner radius, open behavior)
- Input Devices (trackpoint, trackball, tablet, touch)

### Phase 6: Named Workspaces & Layer Rules (COMPLETE)
- Named workspaces with layout overrides
- Layer rules with namespace matching, opacity, shadow

### Phase 7: Animations Enhancement (COMPLETE)
- Per-animation configuration with Spring/Easing parameters
- 11 animations fully configurable

### Phase 8: Output/Misc Enhancements (COMPLETE)
- Custom modes/modelines (v25.11)
- Per-output hot corners and layout overrides
- Workspace shadow in overview
- Hotkey overlay and config notification options

### Phase 9: Quality & Polish (COMPLETE)
- A-Grade code improvements
- IPC hardening
- Accessibility improvements
- Backup/restore functionality

---

## Test Summary

| Category | Count |
|----------|-------|
| Unit Tests | 143 |
| Integration Tests | 63 |
| **Total** | **206** |

All tests pass.

---

## NIRI Settings Coverage

### Fully Implemented (100%)
- Input: Keyboard, Mouse, Touchpad, Trackpoint, Trackball, Tablet, Touch
- Outputs/Displays (including v25.11 features)
- Layout: gaps, struts, focus-ring, border, shadow, insert-hint
- Cursor, Overview, Gestures, Miscellaneous
- Layer Rules, Workspaces
- Environment Variables, Spawn-at-Startup, Switch Events
- Recent Windows, Debug Options

### Partially Implemented
| Feature | Coverage | Missing |
|---------|----------|---------|
| Tab Indicator | 95% | `urgent-color` |
| Animations | 95% | Custom cubic-bezier curves |
| Window Rules | 85% | Per-rule shadow/tab-indicator, clip-to-geometry, tiled-state |
| Keybindings | 50% | Editing (currently read-only) |

---

## Remaining Work

See `docs/FUTURE_IMPROVEMENTS.md` for detailed roadmap.

### High Priority
1. Extract shared rules logic (window_rules + layer_rules duplication)
2. Add missing `accessible-role: region` to page containers
3. Implement tab indicator `urgent-color`

### Medium Priority
4. Window rule per-rule shadow/tab-indicator overrides
5. Split `main.slint` (2010 lines) into smaller files
6. Add UI callback tests

### Low Priority
7. Keybinding editing (significant work)
8. Custom cubic-bezier animation curves
9. Window rule `clip-to-geometry`, `tiled-state`

---

## How to Resume

```bash
cargo check    # Verify build (should complete with no warnings)
cargo test     # Verify all 206 tests pass
cargo run      # Test the application
```

### Key Files for New Development
- `CLAUDE.md` - Development guidelines
- `docs/FUTURE_IMPROVEMENTS.md` - Detailed roadmap
- `docs/A_GRADE_GUIDE.md` - Code quality targets
- `docs/CODE_REVIEW.md` - Full review findings
