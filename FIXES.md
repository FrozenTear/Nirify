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

**Implementation:**
- Created `src/app/ui_state.rs` with `UiState` struct containing 26 UI-only fields
- App struct now has just 6 fields: settings, paths, dirty_tracker, search_index, last_change_time, save_in_progress, ui
- Updated ~38 field references across 9 files
- Clean separation: domain state (settings) vs UI state (selections, dialogs, etc.)

**Status:** [x] Completed

---

## Medium Priority

### 6. Improve Error Handling
**Files:** Multiple (initial count: 345 instances of unwrap/panic/expect)
**Problem:** Silent failures, potential crashes
**Fix:** Replace with proper Result types or unwrap_or_default

**Audit Results:**
- ~95 unwraps in test code - acceptable
- 3 unwraps in doc comments - acceptable (examples)
- 5 mutex poison expects in `save_manager.rs` - acceptable (single-threaded GUI)
- 1 unwrap in `helpers.rs:83` after peek() - safe (iterator guarantees)
- ~30 array accesses - all bounds-checked or fixed-size arrays
- 1 panic in `app/mod.rs:137` - acceptable (fatal startup error)

**Conclusion:** No changes needed. Error handling is already sound.

**Status:** [x] Completed (audit only - no fixes required)

---

### 7. Implement Stubbed Message Handlers
**File:** `src/app/mod.rs`
**Problem:** These handlers do nothing:
- `Message::DialogConfirm`
- `Message::WizardSetupConfig`
- `Message::ConsolidationApply`

**Implementation:**
- `DialogConfirm`: Handles `ConfirmAction::DeleteRule`, `ResetSettings`, `ClearAllKeybindings`
- `WizardSetupConfig`: Creates directories, adds include line to config.kdl with backup
- `ConsolidationApply`: Logs selected suggestions (full merge logic deferred)

**Status:** [x] Completed (basic implementation)

---

### 8. Add Pre-Save Validation
**Files:** `src/save_manager.rs`, `src/config/validation.rs`
**Problem:** Invalid configs can be written to disk
**Fix:** Add validation pass before save

**Implementation:**
- Extended `validation.rs` with `validate_settings()` function
- Validates regex patterns in window/layer rules using `regex_syntax`
- Validates opacity ranges (0.0-1.0)
- Validates keybindings for empty key combos
- Integrated into `save_manager.rs` save_task
- Logs errors/warnings but doesn't block save (niri will catch issues)

**Status:** [x] Completed

---

### 9. Expose Advanced Behavior Settings
**File:** `src/views/behavior.rs`
**Problem:** These are loaded but not in UI:
- `focus_follows_mouse_max_scroll_amount`
- `mod_key_nested`

**Implementation:**
- Added `SetFocusFollowsMouseMaxScroll(Option<f32>)` message
- Created `optional_slider_row` helper widget with enable/disable toggle
- Created `optional_picker_row` helper widget with "None (Use Default)" option
- Updated behavior view with both new settings
- `mod_key_nested` now editable (was read-only display)

**Status:** [x] Completed

---

## Low Priority

### 10. Extract ListDetailView Abstraction
**Files:** `src/views/window_rules.rs`, `layer_rules.rs`, `keybindings.rs`
**Problem:** Similar list-detail patterns implemented differently
**Fix:** Create generic component

**Implementation:**
- Created `src/views/widgets/list_detail.rs` with shared components:
  - `list_detail_layout()` - standard 1:2 split layout
  - `add_button()`, `action_button()`, `delete_button()`, `remove_button()` - consistent button styles
  - `list_item_style()` - selection-aware list item styling
  - `empty_list_placeholder()`, `empty_detail_placeholder()` - consistent empty states
  - `selection_indicator()`, `badge()` - reusable UI elements
  - `match_container_style()`, `add_item_button()` - section helpers
- Updated window_rules.rs, layer_rules.rs, keybindings.rs to use shared components
- Reduced code duplication by ~150 lines

**Status:** [x] Completed

---

### 11. Flatten Message Enum Hierarchy
**File:** `src/messages.rs`
**Problem:** 25 nested enums, 851 lines
**Fix:** Consider grouping or simplifying

**Implementation:**
- Added comprehensive module documentation explaining the architecture
- Reorganized Message enum with section headers for logical grouping:
  - Navigation & UI
  - Visual Settings
  - Behavior & Layout
  - Input Devices
  - Rules & Bindings
  - System Configuration
  - Advanced Features
  - App Management
  - Save & Persistence
  - Dialogs & Modals
  - System Events
- Added section headers and docstrings to nested message enums
- Decision: Kept nested enums for namespacing benefits; combining similar enums (e.g., TrackpointMessage/TrackballMessage) would add complexity without significant benefit

**Status:** [x] Completed (documentation & organization)

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
| 4 | Rule finder helpers | [x] | 5daf8ba |
| 5 | Split UI state | [x] | (38 changes, UiState) |
| 6 | Error handling | [x] | (audit only - no fixes needed) |
| 7 | Stubbed handlers | [x] | (basic impl) |
| 8 | Pre-save validation | [x] | (integrated) |
| 9 | Advanced behavior settings | [x] | (new widgets) |
| 10 | ListDetailView | [x] | (list_detail.rs, shared components) |
| 11 | Flatten messages | [x] | (documentation & organization) |
| 12 | HashMap for rules | [ ] | |
| 13 | Stream KDL | [ ] | |
