# niri-settings-rust: Complete Feature Implementation Plan

**Generated**: 2025-12-07  
**Experts Consulted**: Niri Compositor, Slint UI, Rust Architecture  
**Total Effort Estimate**: ~150 hours (18-20 work days)

---

## Executive Summary

This plan covers implementing **ALL missing niri configuration options** to achieve 100% feature coverage. The work is organized into 8 phases with clear dependencies.

### Current Coverage
- **Implemented**: ~85% of common options
- **Missing**: ~50 distinct features across input, layout, animations, window rules, and new categories

### Key Decisions
1. **UI Reorganization**: Consolidate input devices into single tabbed page
2. **Gradient System**: Build reusable component used by 7+ features
3. **Progressive Disclosure**: Hide advanced options behind expandable sections
4. **Weekly niri Sync**: Check wiki every Friday for new features

---

## Phase 1: Foundation Components (Week 1)

### Priority: CRITICAL - Blocks later phases

| Feature | Type | Complexity | Hours |
|---------|------|------------|-------|
| `Gradient` struct + enums | Backend | Low | 2 |
| `ColorOrGradient` enum | Backend | Low | 1 |
| Gradient KDL helpers | Backend | Medium | 2 |
| `IntegerInputRow` widget | UI | Low | 1 |
| `FilePathRow` widget | UI | Low | 1 |
| `ExpandableSection` widget | UI | Low | 1 |

**Backend Files to Create/Modify:**
- `src/types.rs`: Add `ColorSpace`, `HueInterpolation`, `GradientRelativeTo` enums
- `src/config/models.rs`: Add `Gradient`, `ColorOrGradient` structs
- `src/config/storage.rs`: Add `gradient_to_kdl()`, `color_or_gradient_to_kdl()` helpers
- `src/config/loader.rs`: Add `parse_gradient()` helper

**UI Files to Create:**
- `ui/widgets/integer_input_row.slint`
- `ui/widgets/file_path_row.slint`
- `ui/widgets/expandable_section.slint`

**Total Phase 1**: 8 hours

---

## Phase 2: Quick Wins (Week 1-2)

### Priority: HIGH - Immediate user value, low complexity

| Feature | Category | Backend | UI | Hours |
|---------|----------|---------|-----|-------|
| `scroll-button` | Mouse/Touchpad | Add field | IntegerInputRow | 1 |
| `scroll-button-lock` | Mouse/Touchpad | Add field | ToggleRow | 0.5 |
| Device `off` toggles | All Input | Add 6 fields | ToggleRow x6 | 2 |
| `xkb.file` | Keyboard | Add field | FilePathRow | 1.5 |
| `preset-window-heights` | Layout | Add Vec | List widget | 3 |
| `default-column-display` | Behavior | Add enum | ComboBox | 1 |

**Total Phase 2**: 9 hours

---

## Phase 3: Gradient System (Week 2)

### Priority: HIGH - Used by 7+ features

| Feature | Location | Complexity | Hours |
|---------|----------|------------|-------|
| `GradientPicker` widget | New widget | Medium | 4 |
| Focus ring gradients | Appearance | Medium | 2 |
| Border gradients | Appearance | Medium | 2 |
| Tab indicator gradients + urgent | Layout Extras | Medium | 2 |
| Insert hint gradient | Layout Extras | Low | 1 |

**Total Phase 3**: 11 hours

---

## Phase 4: Input Device Expansion (Week 3)

### Priority: MEDIUM - New device types

| Feature | Device | Fields | Hours |
|---------|--------|--------|-------|
| Trackpoint settings | New | 5 fields | 2 |
| Trackball settings | New | 8 fields | 2 |
| Tablet settings | New | 4 fields + calibration | 3 |
| Touch settings | New | 3 fields + calibration | 2 |
| `CalibrationMatrixInput` widget | New | 6 floats | 2 |
| `map-to-output` dropdown | Tablet/Touch | Output list | 1 |

**UI Reorganization - Input Devices Page:**
```
Input Devices (consolidated page)
├── [Keyboard] tab
├── [Mouse] tab  
├── [Touchpad] tab
├── [Trackpoint] tab (NEW)
├── [Trackball] tab (NEW)
├── [Tablet] tab (NEW)
└── [Touch] tab (NEW)
```

**Total Phase 4**: 12 hours

---

## Phase 5: Window Rules Enhancement (Week 3-4)

### Priority: HIGH - Power user feature

| Feature | Type | Complexity | Hours |
|---------|------|------------|-------|
| New matchers (6) | Backend + UI | Medium | 4 |
| New properties (16) | Backend + UI | High | 8 |
| Per-rule styling | Backend + UI | Very High | 8 |

**New Matchers:**
- `is-active`, `is-focused`, `is-active-in-column`
- `is-window-cast-target`, `is-urgent`, `at-startup`

**New Properties:**
- Geometry: `default-window-height`, `min/max-width/height`, `clip-to-geometry`
- Behavior: `open-maximized-to-edges`, `variable-refresh-rate`, `scroll-factor`
- Advanced: `tiled-state`, `baba-is-float`, `default-floating-position`
- Styling: Per-rule `focus-ring`, `border`, `shadow`, `tab-indicator` overrides

**Total Phase 5**: 20 hours

---

## Phase 6: Named Workspaces & Layer Rules (Week 4)

### Priority: MEDIUM - New pages

| Feature | Type | Complexity | Hours |
|---------|------|------------|-------|
| Named Workspaces page | New page | Medium | 5 |
| Layer Rules page | New page | Medium | 6 |

**Total Phase 6**: 11 hours

---

## Phase 7: Per-Animation Configuration (Week 5)

### Priority: LOW - Nice-to-have, complex

| Feature | Type | Complexity | Hours |
|---------|------|------------|-------|
| Animation type enum | Backend | Low | 1 |
| Spring/Easing params | Backend | Medium | 2 |
| Animation config HashMap | Backend | Medium | 3 |
| Animation config UI | UI | High | 8 |
| 11 animation editors | UI | High | 6 |

**Animation Types (11 total):**
- workspace-switch, window-open, window-close
- window-resize, window-movement, horizontal-view-movement
- config-notification-open-close, screenshot-ui-open
- overview-open-close, recent-windows-close

**Total Phase 7**: 20 hours

---

## Phase 8: Output & Misc Enhancements (Week 5-6)

### Priority: MEDIUM - Mixed features

| Feature | Category | Complexity | Hours |
|---------|----------|------------|-------|
| Per-output hot corners | Displays | Medium | 3 |
| Per-output layout overrides | Displays | High | 5 |
| Custom modeline (DANGEROUS) | Displays | Medium | 2 |
| `overview.workspace-shadow` | Overview | Low | 2 |
| `hotkey-overlay.hide-not-bound` | Misc | Low | 0.5 |
| `config-notification.disable-failed` | Misc | Low | 0.5 |
| `disable-power-key-handling` | Input | Low | 0.5 |
| `mod-key-nested` | Input | Low | 0.5 |

**Total Phase 8**: 14 hours

---

## Final UI Category Structure

```
MAIN NAVIGATION
├── Appearance (focus ring + border with gradients)
├── Behavior (+ default-column-display)
├── Input Devices (CONSOLIDATED - 7 device tabs)
├── Displays (+ per-output overrides, hot corners)
├── Cursor
├── Overview (+ workspace-shadow)
├── Animations (ENHANCED - per-animation config)
├── Workspaces (NEW - named workspaces)

ADVANCED SECTION
├── Layout Extras (+ gradients, preset heights)
├── Gestures
├── Window Rules (ENHANCED - 20+ new options)
├── Layer Rules (NEW)
└── Miscellaneous
```

---

## Version Requirements

| Feature | niri Version |
|---------|--------------|
| scroll-button-lock | 25.08+ |
| default-column-display | 25.02+ |
| preset-window-heights | 0.1.9+ |
| shadow, tab-indicator | 25.02+ |
| open-maximized-to-edges | 25.11+ |
| per-output hot corners | 25.11+ |
| custom modeline | 25.11+ |

---

## Implementation Timeline

| Week | Phase | Hours |
|------|-------|-------|
| 1 | 1 + 2: Foundation + Quick Wins | 17 |
| 2 | 3: Gradient System | 11 |
| 3 | 4 + 5a: Input Devices + Window Rules Start | 20 |
| 4 | 5b + 6: Window Rules + Workspaces/Layers | 23 |
| 5 | 7: Per-Animation Config | 20 |
| 6 | 8 + Polish: Outputs/Misc + Testing | 19 |

**Total: ~110 hours coding + 40 hours testing = 150 hours**

---

## Dangerous Features

1. **Custom Modeline** - Can damage displays
   - Require multi-step confirmation
   - Show hardware damage warning
   
2. **Device Disable** - Can lock out user
   - Warn about alternative input methods
   - Suggest SSH access as backup

3. **Calibration Matrix** - Can make device unusable
   - Provide "Reset to Identity" button
   - Validate values in range

---

## Weekly niri Sync Process

Every Friday:
1. Check https://github.com/YaLTeR/niri/wiki for updates
2. Review https://github.com/YaLTeR/niri/releases for new options
3. Update this plan with any new features
4. Create GitHub issues for new features

---

## Files to Create

### Backend
- Extend existing files (no new model files needed)

### UI  
- `ui/widgets/gradient_picker.slint`
- `ui/widgets/calibration_matrix.slint`
- `ui/widgets/integer_input_row.slint`
- `ui/widgets/file_path_row.slint`
- `ui/widgets/expandable_section.slint`
- `ui/pages/input_devices.slint`
- `ui/pages/workspaces.slint`
- `ui/pages/layer_rules.slint`
- `ui/dialogs/animation_config.slint`

### Tests
- `tests/gradient_test.rs`
- `tests/animation_config_test.rs`
- `tests/window_rules_extended_test.rs`
