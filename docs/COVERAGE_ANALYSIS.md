# Niri Configuration Coverage Analysis

**Last Updated**: January 16, 2026
**Niri Version Analyzed**: v25.11 (latest wiki)

---

## Executive Summary

| Metric | Value |
|--------|-------|
| **Overall Coverage** | ~98% |
| **Fully Implemented** | 49 sections |
| **Partially Implemented** | 1 section |
| **Not Implemented** | 1 feature |

The PROGRESS.md stated ~85-90% coverage, but after detailed analysis and recent additions, the actual coverage is closer to **98%**.

---

## Detailed Coverage by Section

### Input (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| keyboard.xkb (layout, variant, options, model, rules) | ✅ | |
| keyboard.xkb.file | ✅ | Custom keymap file |
| keyboard.repeat-delay | ✅ | |
| keyboard.repeat-rate | ✅ | |
| keyboard.track-layout | ✅ | global/window |
| keyboard.numlock | ✅ | |
| keyboard.off | ✅ | Device disable |
| mouse.* (all properties) | ✅ | Including scroll-button, scroll-button-lock |
| touchpad.* (all properties) | ✅ | tap, dwt, dwtp, drag, drag-lock, etc. |
| trackpoint.* | ✅ | |
| trackball.* | ✅ | |
| tablet.* | ✅ | map-to-output, left-handed, calibration |
| touch.* | ✅ | |
| warp-mouse-to-focus | ✅ | |
| focus-follows-mouse | ✅ | Including max-scroll-amount |
| workspace-auto-back-and-forth | ✅ | |
| disable-power-key-handling | ✅ | |

### Outputs (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| off | ✅ | Disable output |
| mode | ✅ | Resolution@refresh |
| mode custom=true | ✅ | v25.11 |
| modeline | ✅ | v25.11 - Expert mode |
| scale | ✅ | Fractional supported |
| transform | ✅ | All 8 options |
| position | ✅ | x, y coordinates |
| variable-refresh-rate | ✅ | on-demand supported |
| focus-at-startup | ✅ | |
| backdrop-color | ✅ | |
| hot-corners (per-output) | ✅ | v25.11 |
| layout (per-output) | ✅ | v25.11 |

### Layout (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| gaps | ✅ | |
| center-focused-column | ✅ | never/always/on-overflow |
| always-center-single-column | ✅ | |
| empty-workspace-above-first | ✅ | |
| default-column-display | ✅ | normal/tabbed |
| background-color | ✅ | |
| preset-column-widths | ✅ | proportion/fixed |
| preset-window-heights | ✅ | |
| default-column-width | ✅ | |
| focus-ring.* | ✅ | width, active, inactive, urgent (color & gradient) |
| border.* | ✅ | width, active, inactive, urgent (color & gradient) |
| shadow.* | ✅ | All properties |
| struts.* | ✅ | left/right/top/bottom |
| tab-indicator.* | ✅ | All properties including urgent-color |
| insert-hint.* | ✅ | |

### Animations (98%)

| Feature | Status | Notes |
|---------|--------|-------|
| off | ✅ | |
| slowdown | ✅ | |
| workspace-switch | ✅ | Spring |
| window-open | ✅ | Easing |
| window-close | ✅ | Easing |
| horizontal-view-movement | ✅ | Spring |
| window-movement | ✅ | Spring |
| window-resize | ✅ | Spring |
| config-notification-open-close | ✅ | Spring |
| exit-confirmation-open-close | ✅ | Spring |
| screenshot-ui-open | ✅ | Easing |
| overview-open-close | ✅ | Spring |
| recent-windows-close | ✅ | Spring |
| Spring params (damping, stiffness, epsilon) | ✅ | |
| Easing params (duration-ms, curve) | ✅ | |
| cubic-bezier curves | ✅ | |
| Custom shaders | ❌ | **Missing** - Low priority |

### Window Rules (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| **Match Criteria** | | |
| title | ✅ | Regex |
| app-id | ✅ | Regex |
| is-active | ✅ | |
| is-focused | ✅ | |
| is-active-in-column | ✅ | |
| is-floating | ✅ | |
| is-window-cast-target | ✅ | |
| is-urgent | ✅ | |
| at-startup | ✅ | |
| **Opening Properties** | | |
| default-column-width | ✅ | |
| default-window-height | ✅ | |
| open-on-output | ✅ | |
| open-on-workspace | ✅ | |
| open-maximized | ✅ | |
| open-maximized-to-edges | ✅ | v25.11 |
| open-fullscreen | ✅ | |
| open-floating | ✅ | |
| open-focused | ✅ | |
| **Dynamic Properties** | | |
| block-out-from | ✅ | |
| opacity | ✅ | |
| variable-refresh-rate | ✅ | |
| default-column-display | ✅ | |
| default-floating-position | ✅ | |
| scroll-factor | ✅ | |
| draw-border-with-background | ✅ | |
| focus-ring (per-rule) | ✅ | |
| border (per-rule) | ✅ | |
| shadow (per-rule) | ✅ | |
| tab-indicator (per-rule) | ✅ | |
| geometry-corner-radius | ✅ | |
| clip-to-geometry | ✅ | |
| tiled-state | ✅ | |
| baba-is-float | ✅ | |
| min/max-width/height | ✅ | |
| exclude | ✅ | Exclude matcher (January 2026) |

### Layer Rules (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| namespace (match) | ✅ | Regex |
| at-startup (match) | ✅ | |
| opacity | ✅ | |
| shadow | ✅ | |
| geometry-corner-radius | ✅ | |
| block-out-from | ✅ | |
| place-within-backdrop | ✅ | v25.05 |
| baba-is-float | ✅ | v25.05 |

### Named Workspaces (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| workspace "name" | ✅ | |
| open-on-output | ✅ | |
| layout (per-workspace) | ✅ | v25.11 |

### Miscellaneous (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| spawn-at-startup | ✅ | |
| spawn-sh-at-startup | ✅ | v25.08 |
| prefer-no-csd | ✅ | |
| screenshot-path | ✅ | |
| environment | ✅ | |
| cursor.* | ✅ | theme, size, hide settings |
| overview.* | ✅ | zoom, backdrop, workspace-shadow |
| xwayland-satellite | ✅ | v25.08 |
| clipboard.disable-primary | ✅ | v25.02 |
| hotkey-overlay.skip-at-startup | ✅ | |
| hotkey-overlay.hide-not-bound | ✅ | |
| config-notification.disable-failed | ✅ | v25.08 |

### Gestures (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| hot-corners | ✅ | All corners |
| dnd-edge-view-scroll | ✅ | v25.02 |
| dnd-edge-workspace-switch | ✅ | v25.05 |

### Switch Events (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| lid-close | ✅ | |
| lid-open | ✅ | |
| tablet-mode-on | ✅ | |
| tablet-mode-off | ✅ | |

### Recent Windows (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| off | ✅ | |
| debounce-ms | ✅ | |
| open-delay-ms | ✅ | |
| highlight.* | ✅ | active-color, urgent-color, padding, corner-radius |
| previews.* | ✅ | max-height, max-scale |
| binds (filter, scope) | ✅ | Model + loader/storage (January 2026) |

### Debug Options (100%)

All 20 debug options implemented.

### Keybindings (100%)

| Feature | Status | Notes |
|---------|--------|-------|
| Read keybindings | ✅ | |
| Add keybindings | ✅ | |
| Edit keybindings | ✅ | |
| Delete keybindings | ✅ | |
| Key capture UI | ✅ | |

---

## Missing Features Summary

### Recently Implemented (January 2026)

| Feature | Status |
|---------|--------|
| focus-ring/border urgent-gradient | ✅ Implemented |
| Window rule `exclude` matcher | ✅ Implemented |
| Recent windows binds filter/scope | ✅ Implemented |

### Remaining (Low Priority)

| Feature | Effort | Impact |
|---------|--------|--------|
| Custom animation shaders | High | Niche |

---

## Comparison with PROGRESS.md

The PROGRESS.md file listed these as missing, but they are **actually implemented**:

| Feature | Actual Status |
|---------|---------------|
| Tab indicator urgent-color | ✅ Implemented |
| Custom cubic-bezier curves | ✅ Implemented |
| Window rule clip-to-geometry | ✅ Implemented |
| Window rule tiled-state | ✅ Implemented |
| Window rule per-rule shadow | ✅ Implemented |
| Window rule per-rule tab-indicator | ✅ Implemented |
| Keybinding editing | ✅ Implemented |
| Urgent gradients | ✅ Implemented (January 2026) |
| Window rule exclude | ✅ Implemented (January 2026) |
| Recent windows binds | ✅ Implemented (January 2026) |

**Recommendation**: Update PROGRESS.md to reflect actual ~98% coverage.

---

## Remaining Gap

The only remaining unimplemented feature is:

| Feature | Effort | Reason |
|---------|--------|--------|
| Custom animation shaders | High | Very niche use case, requires GLSL editor |

Coverage is now **~98%** of all niri configuration options.
