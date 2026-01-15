# Progress Document

Last updated: 2026-01-14

## Current Status: Code Review Fixes Complete

All HIGH-PRIORITY bugs from the GPT-5.2, GPT-5.2-codex, GPT-5.2-bugs, and Gemini-3 code reviews have been fixed. Performance and documentation improvements have also been applied.

---

## Completed This Session

### Bug Fixes (from code reviews)

1. **Toast auto-dismiss timer lifetime** (`save_manager.rs`)
   - Fixed: Timer was being dropped immediately, preventing auto-dismiss
   - Solution: Use `Timer::single_shot()` which doesn't require keeping handle alive

2. **DirtyTracker poisoned mutex** (`dirty.rs`)
   - Fixed: Panic on mutex poison in `mark()`, `mark_many()`, `mark_all()`
   - Solution: Added poison recovery with `poisoned.into_inner()`

3. **Keybindings UI stale after edits** (`callbacks/keybindings.rs`)
   - Fixed: UI not refreshing after add/edit/delete/reorder operations
   - Solution: Added `sync_keybindings()` calls + selection index restore after all mutations

4. **Workspace name parsing silent fallback** (`loader/workspaces.rs`, `loader/import.rs`)
   - Fixed: Non-string workspace names silently ignored
   - Solution: Added `warn!()` logging for invalid types

5. **Arc<Mutex> poison swallowed in selection state** (8 callback files)
   - Fixed: Selection index using `Arc<Mutex<i32>>` which can panic on poison
   - Solution: Refactored to `Rc<Cell<i32>>` - simpler, no poison possible, correct for UI thread

6. **`expect()` in gradient_to_kdl** (`storage/gradient.rs`)
   - Fixed: Could panic on write failure (theoretical)
   - Solution: Changed to `let _ = write!()`

7. **Fragile string slicing in input.rs** (`storage/input.rs`)
   - Fixed: Hardcoded string slicing for device titles
   - Solution: Added `device_title()` method to `MappedInputDevice` trait

### Documentation Fixes

1. **Keybindings "read-only" drift** - Removed outdated comments in:
   - `config/loader/keybindings.rs`
   - `config/models/keybindings.rs`
   - `config/models/mod.rs`
   - `ui/bridge/sync.rs`
   - `config/registry.rs`
   - `config/replace.rs`

2. **Smart replace formatting clarity** (`config/replace.rs`)
   - Added module-level note about semantic vs formatting preservation
   - Clarified `Unmanaged` variant documentation

### Performance Improvements

1. **ConfigFile registry docs** (`handlers.rs`)
   - Added comment explaining intentional UI ordering difference from `ConfigFile::ALL`

2. **Keybindings double-read eliminated** (`loader/mod.rs`)
   - Was: Reading keybindings.kdl twice (once for load, once for status)
   - Now: Derives `FileLoadStatus` from what `load_keybindings` already determined

3. **Future optimization notes** - Added performance notes to:
   - `ipc/mod.rs` - IPC calls currently synchronous on UI thread
   - `save_manager.rs` - File I/O runs on UI thread via timer

---

## Previously Completed (before this session)

- Bezier curve animation controls in AnimationsPage
- Full keybindings editor implementation (add, edit, delete, reorder, import)
- Window rules and layer rules editors
- Startup commands and environment variables editors
- All core settings pages (appearance, behavior, input devices, etc.)
- Smart config replacement with backup
- First-run wizard
- Debounced auto-save with dirty tracking
- IPC integration (config reload, output detection, window list)

---

## Architecture Notes

### Selection State Pattern (updated)
```rust
// OLD - could panic on mutex poison
let selected_idx = Arc::new(Mutex::new(-1i32));
let idx = selected_idx.lock().map(|i| *i).unwrap_or(-1);

// NEW - simpler, no poison possible
let selected_idx = Rc::new(Cell::new(-1i32));
let idx = selected_idx.get();
selected_idx.set(new_value);
```

### Files Modified This Session
- `src/ui/bridge/save_manager.rs`
- `src/config/dirty.rs`
- `src/ui/bridge/callbacks/keybindings.rs`
- `src/ui/bridge/callbacks/workspaces.rs`
- `src/ui/bridge/callbacks/outputs.rs`
- `src/ui/bridge/callbacks/startup.rs`
- `src/ui/bridge/callbacks/environment.rs`
- `src/ui/bridge/callbacks/layer_rules.rs`
- `src/ui/bridge/callbacks/window_rules.rs`
- `src/ui/bridge/callbacks/rules_common.rs`
- `src/config/loader/workspaces.rs`
- `src/config/loader/import.rs`
- `src/config/loader/keybindings.rs`
- `src/config/loader/mod.rs`
- `src/config/models/keybindings.rs`
- `src/config/models/mod.rs`
- `src/config/storage/input.rs`
- `src/config/storage/gradient.rs`
- `src/config/registry.rs`
- `src/config/replace.rs`
- `src/ui/bridge/sync.rs`
- `src/handlers.rs`
- `src/ipc/mod.rs`
- `ui/main.slint` (bezier callbacks)

---

## Future Considerations (not blocking)

1. **Move I/O off UI thread** - File saves and IPC could use background threads with `invoke_from_event_loop` for UI updates

2. **ConfigFile registry usage** - Could further reduce hardcoded file lists, but current approach with comments is acceptable

3. **Async IPC** - Could prevent UI freezes on slow niri responses

---

## Build Notes

If you get SIGKILL during build (OOM):
```bash
cargo build -j 2   # or -j 1 for minimal memory
```

Slint compilation is memory-intensive, especially with `cosmic-dark` style.
