# Niri Settings - Fix & Improvement Plan

## High Priority

### 1. Remove `.clone()` from View Calls
**File:** `src/app/mod.rs` (lines 598-637)
**Problem:** Every page render deep-clones settings structs unnecessarily
**Fix:** Change view function signatures to accept `&` references instead of owned values

**Status:** [ ] Not Started

---

### 2. Complete Layer Rules View
**File:** `src/views/layer_rules.rs`
**Problem:** View is a stub/placeholder despite full model and handler support
**Fix:** Build out the UI similar to window_rules.rs

**Status:** [ ] Not Started

---

### 3. Expose Missing Window Rule Match Types
**File:** `src/views/window_rules.rs`
**Problem:** These match types are in the model but not exposed in UI:
- `is_active`
- `is_active_in_column` (v0.1.6+)
- `is_window_cast_target` (v25.02+)
- `is_urgent` (v25.05+)
- `at_startup` (v0.1.6+)

**Status:** [ ] Not Started

---

### 4. Add Helper Methods for Rule Finding
**Files:** `src/app/handlers/window_rules.rs`, `layer_rules.rs`, `outputs.rs`
**Problem:** `iter_mut().find(|r| r.id == id)` pattern repeated 30+ times
**Fix:** Add helper methods to Settings or individual rule structs

**Status:** [ ] Not Started

---

### 5. Split UI State from App Struct
**File:** `src/app/mod.rs`
**Problem:** App struct has 129 fields mixing domain and UI state
**Fix:** Extract UI-only state into separate `UiState` struct

**Status:** [ ] Not Started

---

## Medium Priority

### 6. Improve Error Handling
**Files:** Multiple (345 instances of unwrap/panic/expect)
**Problem:** Silent failures, potential crashes
**Fix:** Replace with proper Result types or unwrap_or_default

**Status:** [ ] Not Started

---

### 7. Implement Stubbed Message Handlers
**File:** `src/app/mod.rs`
**Problem:** These handlers do nothing:
- `Message::DialogConfirm`
- `Message::WizardSetupConfig`
- `Message::ConsolidationApply`

**Status:** [ ] Not Started

---

### 8. Add Pre-Save Validation
**Files:** `src/save_manager.rs`, handlers
**Problem:** Invalid configs can be written to disk
**Fix:** Add validation pass before save

**Status:** [ ] Not Started

---

### 9. Expose Advanced Behavior Settings
**File:** `src/views/behavior.rs`
**Problem:** These are loaded but not in UI:
- `focus_follows_mouse_max_scroll_amount`
- `mod_key_nested`

**Status:** [ ] Not Started

---

## Low Priority

### 10. Extract ListDetailView Abstraction
**Files:** `src/views/window_rules.rs`, `layer_rules.rs`, `keybindings.rs`
**Problem:** Similar list-detail patterns implemented differently
**Fix:** Create generic component

**Status:** [ ] Not Started

---

### 11. Flatten Message Enum Hierarchy
**File:** `src/messages.rs`
**Problem:** 25 nested enums, 851 lines
**Fix:** Consider grouping or simplifying

**Status:** [ ] Not Started

---

### 12. Use HashMap for Rules (O(1) Lookup)
**Files:** `src/config/models/rules.rs`
**Problem:** Vec with iter().find() is O(n)
**Fix:** Use HashMap<u32, Rule> for large collections

**Status:** [ ] Not Started

---

### 13. Stream KDL Parsing
**Files:** `src/config/loader/*.rs`
**Problem:** Large configs cause 1-2s startup delay
**Fix:** Lazy-load or stream parse

**Status:** [ ] Not Started

---

## Progress Tracking

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Remove view clones | [x] | 9743f6c |
| 2 | Layer rules view | [x] | ab464c4 |
| 3 | Window rule match types | [x] | 656eac2 |
| 4 | Rule finder helpers | [ ] | |
| 5 | Split UI state | [ ] | |
| 6 | Error handling | [ ] | |
| 7 | Stubbed handlers | [ ] | |
| 8 | Pre-save validation | [ ] | |
| 9 | Advanced behavior settings | [ ] | |
| 10 | ListDetailView | [ ] | |
| 11 | Flatten messages | [ ] | |
| 12 | HashMap for rules | [ ] | |
| 13 | Stream KDL | [ ] | |
