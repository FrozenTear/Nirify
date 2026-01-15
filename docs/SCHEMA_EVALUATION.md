# Schema vs Macros: Evaluation Results

## Summary

After implementing a schema-driven code generation prototype for the `appearance` category, we've decided to **continue with the macro-based approach** rather than adopt schema-driven generation.

## What We Built

1. **Schema file** (`schema/appearance.toml`) - 165 lines defining all appearance fields
2. **Generator binary** (`src/bin/generate-schema.rs`) - ~450 lines
3. **Generated code**:
   - `appearance_callbacks.rs` - 41 lines
   - `appearance_sync.rs` - 44 lines
   - `appearance_loader.rs` - 78 lines (template)
   - `appearance_storage.rs` - 36 lines (template)
   - `appearance.slint` - 36 lines

## Evaluation Results

### Criteria from Plan

| Criterion | Result | Notes |
|-----------|--------|-------|
| Schema simpler than generated? | **FAIL** | Schema: 165 lines, Generated callbacks+sync: 85 lines |
| Debuggable? | PASS | Code in src/generated/, normal debugging |
| IDE navigation works? | PASS | Regular Rust/Slint code |
| Generates valid code? | PARTIAL | Callbacks/sync good, loader/storage need manual fixes |
| Handles complex types? | PASS | Uses existing macros for ColorOrGradient |

### Line Count Comparison

| Component | Hand-written | Generated | Schema Overhead |
|-----------|-------------|-----------|-----------------|
| callbacks | 40 | 41 | - |
| sync | 30 | 44 | - |
| loader | 187 | 78 (template) | Needs ~50% manual work |
| storage | 132 | 36 (template) | Needs ~50% manual work |
| **Subtotal** | 389 | 199 | |
| Schema file | - | - | +165 |
| Generator | - | - | +450 |
| **True Total** | 389 | 199 | +615 |

## Why Schema Doesn't Work Well Here

### 1. Callbacks/Sync Already Use Macros Effectively

The existing macro system (`register_bool_callbacks!`, `register_clamped_callbacks!`, etc.) already reduces callbacks to ~40 lines. The schema just specifies which macros to call - it's an indirection layer that adds complexity without reducing code.

**Hand-written (40 lines):**
```rust
register_bool_callbacks!(ui, settings, save_manager, Appearance, appearance, [
    (on_focus_ring_toggled, focus_ring_enabled, "Focus ring enabled"),
    (on_border_toggled, border_enabled, "Border enabled"),
]);
```

**Schema equivalent (20+ lines in TOML):**
```toml
[[fields]]
name = "focus_ring_enabled"
type = "bool"
ui_callback = "on_focus_ring_toggled"
ui_setter = "set_focus_ring_enabled"
description = "Focus ring enabled"
# ... more fields
```

The schema is **more verbose** than the macro invocation it generates.

### 2. KDL Has Domain-Specific Conventions

The niri KDL format has patterns that are hard to express in a generic schema:

- **Dual syntax**: `gaps 16` vs `gaps inner=16 outer=8`
- **Presence semantics**: `focus-ring { off }` vs no `off` child
- **Nested hierarchies**: `layout { focus-ring { active { color "..." } } }`
- **Cross-category mixing**: `appearance.kdl` contains behavior settings

A schema expressive enough to capture these would be as complex as the code it generates.

### 3. Generator Maintenance Cost

The generator is 450+ lines of Rust code that:
- Must be maintained alongside the main codebase
- Has its own bugs and edge cases
- Requires running a separate command to regenerate
- Creates a "two places to look" problem when debugging

### 4. Six Places Still Required

Even with the schema, adding a new setting requires changes in:
1. Schema TOML file
2. `models.rs` (struct field)
3. Loader (complex KDL patterns need manual handling)
4. Storage (complex KDL patterns need manual handling)
5. Slint UI component
6. Re-run generator

This isn't meaningfully better than the current 6 places.

## Why Macros Work Better

### 1. Zero Indirection

Macros expand directly at the call site. What you see is what you get.

### 2. IDE-Friendly

Go-to-definition works. Refactoring tools work. No generated files to track.

### 3. Already Effective

The macro system has already reduced boilerplate significantly:
- `indices.rs`: 522 → 71 lines (with `SlintIndex` derive)
- Input device callbacks: 419 → 311 lines (26% reduction)
- Callback files use 3-5 macro calls instead of 20+ manual callbacks

### 4. Composable

Macros can be combined and nested. New patterns can be added incrementally.

### 5. Type-Safe

Rust macros are checked at compile time. Schema parsing errors only show at generation time.

## Recommendations

### Keep Using

1. **`SlintIndex` derive macro** - Excellent for enum↔index conversions
2. **Batch callback macros** - `register_bool_callbacks!`, `register_clamped_callbacks!`, etc.
3. **Sync macros** - `sync_bool_props!`, `sync_f32_props!`, etc.
4. **Dirty tracking** - Already integrated into macros

### Don't Pursue

1. Schema-driven generation for callbacks/sync
2. Schema-driven generation for loader/storage
3. Any approach requiring a separate generator binary

### Future Improvements

If boilerplate becomes problematic again, consider:

1. **More specialized macros** for common patterns (e.g., `register_optional_toggle!` for `Option<T>` fields)
2. **Proc macros on structs** to generate loader/storage (like serde, but for KDL)
3. **Better IDE tooling** rather than code generation

## Files to Remove

The schema experiment files can be removed:
- `schema/` directory
- `src/bin/generate-schema.rs`
- `src/generated/` directory

Or kept as documentation of the evaluation.

## Conclusion

The schema approach solves a problem we don't have. The macro system already provides excellent boilerplate reduction for callbacks and sync. The loader/storage have too much domain-specific logic to benefit from generic generation.

**Decision: Continue with macros. Delete schema infrastructure.**
