# Future Improvements Roadmap

**Last Updated**: January 11, 2026

Based on comprehensive code review (6 parallel agents analyzing Rust, Slint, architecture, security, tests, and NIRI coverage).

---

## Executive Summary

| Metric | Value |
|--------|-------|
| NIRI Coverage | ~85-90% |
| Test Count | 206 (all pass) |
| Code Quality | A-/B+ |
| Security | Good |

---

## Phase 10: High Priority Improvements

### 10.1 Architecture: Extract Shared Rules Logic
**Priority**: HIGH | **Complexity**: Medium | **Impact**: Maintainability

**Problem**: `window_rules.rs` (865 lines) and `layer_rules.rs` (898 lines) have significant duplication:
- Add/remove rule callbacks
- Select/deselect callbacks
- Multi-match handling
- Property sync functions

**Solution**: Create `src/ui/bridge/callbacks/rules_common.rs`:
```rust
pub trait RuleEditor<R, M> {
    fn add_rule(&mut self) -> &mut R;
    fn remove_rule(&mut self, index: usize);
    fn add_match(&mut self, rule_index: usize) -> &mut M;
    fn remove_match(&mut self, rule_index: usize, match_index: usize);
}
```

**Files to modify**:
- Create `src/ui/bridge/callbacks/rules_common.rs`
- Refactor `window_rules.rs` to use shared traits
- Refactor `layer_rules.rs` to use shared traits

---

### 10.2 Accessibility: Page Container Roles
**Priority**: HIGH | **Complexity**: Simple | **Impact**: Screen reader support

**Problem**: Multiple page containers missing `accessible-role`:
- `ui/pages/appearance.slint`
- `ui/pages/behavior.slint`
- `ui/pages/keyboard.slint`
- (and others)

**Solution**: Add to each page's root Rectangle:
```slint
Rectangle {
    accessible-role: region;
    accessible-label: "Appearance settings";
    // ...
}
```

---

### 10.3 Missing Feature: Tab Indicator Urgent Color
**Priority**: HIGH | **Complexity**: Simple | **Impact**: Feature completeness

**Problem**: Tab indicator missing `urgent-color` property.

**Files to modify**:
1. `src/config/models/layout.rs` - Add `urgent_color: ColorOrGradient`
2. `src/config/loader/layout.rs` - Parse `urgent-color`
3. `src/config/storage/layout.rs` - Generate `urgent-color`
4. `ui/pages/layout_extras.slint` - Add color picker
5. `ui/main.slint` - Add property/callback
6. `src/ui/bridge/callbacks/layout_extras.rs` - Add callback
7. `src/ui/bridge/sync.rs` - Add sync

---

## Phase 11: Medium Priority Improvements

### 11.1 Window Rule Per-Rule Overrides
**Priority**: MEDIUM | **Complexity**: Medium | **Impact**: Feature completeness

**Missing properties**:
- `shadow` (per-rule shadow settings)
- `tab-indicator` (per-rule tab indicator)
- `clip-to-geometry` (v0.1.6+)
- `tiled-state` (v25.05+)

**Implementation approach**: Similar to existing per-rule focus-ring/border overrides.

---

### 11.2 Slint: Theme Consistency
**Priority**: MEDIUM | **Complexity**: Simple | **Impact**: Visual polish

**Problem**: Hardcoded colors in dialogs:
- `ui/dialogs/first_run_wizard.slint:26` - `#000000d0`
- `ui/dialogs/error_dialog.slint:36` - `#00000080`
- `ui/pages/window_rules.slint:311` - `#ffffff`

**Solution**: Add to `ui/styles.slint`:
```slint
export global Theme {
    // ... existing ...
    out property <color> overlay-background: #000000cc;
    out property <color> text-on-overlay: #ffffff;
}
```

---

### 11.3 Architecture: Split main.slint
**Priority**: MEDIUM | **Complexity**: Medium | **Impact**: Maintainability

**Problem**: `ui/main.slint` is 2010 lines - hard to navigate.

**Solution**: Split into:
```
ui/
├── main.slint           # Layout and imports only (~200 lines)
├── properties.slint     # Property declarations (~800 lines)
├── callbacks.slint      # Callback declarations (~400 lines)
└── pages/               # (existing)
```

---

### 11.4 Security: Multi-File Transaction Safety
**Priority**: MEDIUM | **Complexity**: Medium | **Impact**: Data integrity

**Problem**: `write_all_settings()` writes 25 files sequentially. Partial failure leaves inconsistent state.

**Solution options**:
1. Write all to temp dir, then atomic swap
2. Version stamping with rollback support
3. Transaction log file

---

### 11.5 Testing: UI Callback Coverage
**Priority**: MEDIUM | **Complexity**: Medium | **Impact**: Reliability

**Problem**: UI bridge callbacks lack tests (require Slint runtime).

**Solution**: Extract pure logic into testable functions:
```rust
// Instead of testing the callback directly:
pub fn calculate_clamped_gap(value: f32) -> f32 {
    value.clamp(GAP_SIZE_MIN, GAP_SIZE_MAX)
}

#[test]
fn test_gap_clamping() {
    assert_eq!(calculate_clamped_gap(-5.0), GAP_SIZE_MIN);
}
```

---

## Phase 12: Low Priority Improvements

### 12.1 Keybinding Editing
**Priority**: LOW | **Complexity**: HIGH | **Impact**: Feature completeness

Currently keybindings are read-only. Full editing would require:
- Conflict detection algorithm
- Key capture UI
- Full KDL editing of binds section
- Safe-mode reset mechanism

**Recommendation**: Defer unless user demand is high.

---

### 12.2 Custom Cubic-Bezier Animation Curves
**Priority**: LOW | **Complexity**: Low | **Impact**: Power users

**Missing**: `CubicBezier(f64, f64, f64, f64)` variant for `EasingCurve`.

**Files to modify**:
1. `src/config/models/animation.rs` - Add variant
2. `src/config/loader/display.rs` - Parse cubic-bezier
3. `src/config/storage/mod.rs` - Generate cubic-bezier
4. `ui/pages/animations.slint` - Add 4 number inputs

---

### 12.3 Rust: String Length Limits
**Priority**: LOW | **Complexity**: Simple | **Impact**: Robustness

**Problem**: String inputs not length-limited.

**Solution**: Add to validation:
```rust
const MAX_STRING_LENGTH: usize = 1024;

fn validate_string(s: &str) -> String {
    if s.len() > MAX_STRING_LENGTH {
        s[..MAX_STRING_LENGTH].to_string()
    } else {
        s.to_string()
    }
}
```

---

### 12.4 Slint: Loading States
**Priority**: LOW | **Complexity**: Simple | **Impact**: UX polish

**Problem**: No loading indicator when keybindings load.

**Solution**: Add loading spinner component and state.

---

## Code Quality Checklist

Before any PR, verify:

- [ ] `cargo check` passes with no warnings
- [ ] `cargo test` passes (206 tests)
- [ ] `cargo clippy` passes
- [ ] New settings have loader + storage + UI + callback + sync
- [ ] Accessibility labels on interactive elements
- [ ] Constants used (not magic numbers)
- [ ] Error handling with context

---

## Architecture Decision Records

### ADR-001: CategorySection Pattern
**Decision**: Use sealed trait pattern for category-to-section mapping.
**Rationale**: Compile-time safety, single source of truth.
**Status**: Implemented, working well.

### ADR-002: Dirty Tracking
**Decision**: Track modified categories, only save changed files.
**Rationale**: Reduce disk I/O during slider adjustments.
**Status**: Implemented, significant performance benefit.

### ADR-003: Atomic Writes
**Decision**: Use temp file + rename for all config writes.
**Rationale**: Prevent corruption from interrupted writes.
**Status**: Implemented throughout.

### ADR-004: 300ms Save Debounce
**Decision**: Debounce saves by 300ms.
**Rationale**: Balance responsiveness with not spamming compositor reloads.
**Status**: Implemented, documented in constants.rs.

---

## References

- `docs/PROGRESS.md` - Implementation status
- `docs/A_GRADE_GUIDE.md` - Code quality targets
- `docs/CODE_REVIEW.md` - Full review findings
- `docs/MISSING_FEATURES_IMPLEMENTATION_PLAN.md` - Original feature plan
