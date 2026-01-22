# Implementation Status & Fix Plan
## Post-Review Code Audit

**Date:** 2026-01-22
**Review Document:** `docs/ICED_IMPLEMENTATION_REVIEW.md`

---

## Status Summary

After comprehensive review, the following critical issues remain **UNFIXED**:

| Issue | Severity | Status | Location |
|-------|----------|--------|----------|
| Window close race condition | 🔥 CRITICAL | ❌ Not Fixed | `src/app.rs:310-318` |
| SaveManager race condition | ⚠️ HIGH | ❌ Not Fixed | `src/save_manager.rs:87,140` |
| Missing message handlers (4 categories) | ⚠️ HIGH | ❌ Not Fixed | `src/app.rs:172-191` |
| Missing message enums (15 categories) | ⚠️ HIGH | ❌ Not Fixed | `src/messages.rs:36` |
| Box::leak memory leaks | ⚠️ HIGH | ❌ Not Fixed | 10 files |
| Dead code warnings | ℹ️ LOW | ❌ Not Fixed | `src/ipc/mod.rs`, `src/app.rs` |
| Theming inconsistency | ⚠️ MEDIUM | 🔄 In Progress | Multiple files (other agent) |

**Good News:** The storage layer (save_dirty) supports ALL 25 categories! The issue is just wiring up the UI → update handlers.

---

## Critical Issues Detail

### 1. Window Close Race Condition 🔥 CRITICAL

**Current Code:** `src/app.rs:310-318`
```rust
Message::WindowCloseRequested => {
    // Perform final save before exiting
    if self.dirty_tracker.is_dirty() {
        log::info!("Window closing with unsaved changes, performing final save...");
        self.save_manager.save_task().map(|_| Message::None)  // ❌ WRONG
    } else {
        std::process::exit(0)
    }
}
```

**Problem:**
- `save_task()` returns a `Task` that runs asynchronously
- `Message::None` is returned immediately
- App may exit before save completes, **losing user data**

**Fix Strategy:**

**Option A: Block on Save (Simple)**
```rust
Message::WindowCloseRequested => {
    if self.dirty_tracker.is_dirty() {
        log::info!("Window closing with unsaved changes, performing blocking save...");

        // Blocking save before exit
        let settings = self.settings.lock().unwrap().clone();
        let dirty = self.dirty_tracker.take();

        match crate::config::save_dirty(&self.paths, &settings, &dirty) {
            Ok(count) => {
                log::info!("Saved {} files before exit", count);
            }
            Err(e) => {
                log::error!("Failed to save on exit: {}", e);
            }
        }
    }

    std::process::exit(0);
    Task::none()
}
```

**Option B: Show Saving Dialog (Better UX)**
```rust
Message::WindowCloseRequested => {
    if self.dirty_tracker.is_dirty() {
        // Show "Saving..." dialog
        self.dialog_state = DialogState::Info {
            title: "Saving".to_string(),
            message: "Please wait while settings are saved...".to_string(),
        };

        // Trigger save, then quit
        self.save_manager.save_task()
            .map(|_| Message::QuitAfterSave)
    } else {
        std::process::exit(0);
        Task::none()
    }
}

// Add new message handler:
Message::QuitAfterSave => {
    log::info!("Save completed, exiting");
    std::process::exit(0);
    Task::none()
}
```

**Recommendation:** Use Option A for simplicity. The save is typically <100ms so blocking is acceptable.

---

### 2. SaveManager Race Condition ⚠️ HIGH

**Current Code:** `src/save_manager.rs:80-144`
```rust
pub fn save_task(&self) -> Task<SaveResult> {
    // ...

    // Mark save as in progress
    *save_in_progress.lock().unwrap() = true;  // Line 87

    Task::future(async move {
        // ... save work ...

        // If this panics or returns early, flag never cleared!
        *save_in_progress.lock().unwrap() = false;  // Line 140

        save_result
    })
}
```

**Problem:**
- If the async task panics, the flag stays `true` forever
- Blocks ALL future saves permanently
- No recovery mechanism

**Fix: RAII Guard Pattern**

Add to `src/save_manager.rs`:
```rust
/// RAII guard that automatically clears save_in_progress flag on drop
struct SaveGuard(Arc<Mutex<bool>>);

impl SaveGuard {
    fn new(flag: Arc<Mutex<bool>>) -> Option<Self> {
        let mut lock = flag.lock().unwrap();
        if *lock {
            return None;  // Already saving
        }
        *lock = true;
        Some(SaveGuard(flag))
    }
}

impl Drop for SaveGuard {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.0.lock() {
            *lock = false;
        }
    }
}
```

Modify `save_task()`:
```rust
pub fn save_task(&self) -> Task<SaveResult> {
    let save_in_progress = self.save_in_progress.clone();

    // Try to acquire guard
    let guard = match SaveGuard::new(save_in_progress) {
        Some(g) => g,
        None => {
            log::debug!("Save already in progress, skipping");
            return Task::none();
        }
    };

    // ... rest of setup ...

    Task::future(async move {
        let _guard = guard;  // Held until task completes (auto-clears on drop)

        // ... existing save logic ...

        save_result  // Guard drops here, clearing flag
    })
}
```

---

### 3. Missing Message Handlers (4 Categories) ⚠️ HIGH

**Status:** Message enums exist but update handlers are stubs

**Affected Categories:**
1. Animations (has AnimationsMessage enum)
2. Workspaces (has WorkspacesMessage enum)
3. WindowRules (has WindowRulesMessage enum)
4. Keybindings (has KeybindingsMessage enum)

**Current Code:** `src/app.rs:172-191`
```rust
Message::Animations(_msg) => {
    // TODO: Phase 5 - Implement animations updates
    self.dirty_tracker.mark(SettingsCategory::Animations);
    Task::none()
}

Message::Cursor(msg) => self.update_cursor(msg),

Message::Workspaces(_msg) => {
    // TODO: Phase 6 - Implement workspaces updates
    self.dirty_tracker.mark(SettingsCategory::Workspaces);
    Task::none()
}

Message::WindowRules(_msg) => {
    // TODO: Phase 7 - Implement window rules updates
    self.dirty_tracker.mark(SettingsCategory::WindowRules);
    Task::none()
}

Message::Keybindings(_msg) => {
    // TODO: Phase 7 - Implement keybindings updates
    self.dirty_tracker.mark(SettingsCategory::Keybindings);
    Task::none()
}
```

**Fix Plan:**

#### 3.1 Implement update_animations()

**Template:**
```rust
fn update_animations(&mut self, msg: AnimationsMessage) -> Task<Message> {
    let mut settings = self.settings.lock().unwrap();

    match msg {
        AnimationsMessage::ToggleSlowdown(value) => {
            settings.animations.slowdown = value;
        }
        AnimationsMessage::SetSlowdownFactor(value) => {
            settings.animations.slowdown_factor = value.clamp(0.1, 10.0);
        }
        AnimationsMessage::SetAnimationEnabled(name, enabled) => {
            if let Some(anim) = settings.animations.animations.iter_mut()
                .find(|a| a.name == name) {
                anim.enabled = enabled;
            }
        }
        AnimationsMessage::SetAnimationDuration(name, duration) => {
            if let Some(anim) = settings.animations.animations.iter_mut()
                .find(|a| a.name == name) {
                anim.duration_ms = duration.clamp(0, 5000);
            }
        }
        AnimationsMessage::SetAnimationCurve(name, curve) => {
            if let Some(anim) = settings.animations.animations.iter_mut()
                .find(|a| a.name == name) {
                anim.curve = curve;
            }
        }
        AnimationsMessage::SetAnimationSpringDampingRatio(name, ratio) => {
            if let Some(anim) = settings.animations.animations.iter_mut()
                .find(|a| a.name == name) {
                anim.spring_damping_ratio = Some(ratio.clamp(0.1, 2.0));
            }
        }
        AnimationsMessage::SetAnimationSpringEpsilon(name, epsilon) => {
            if let Some(anim) = settings.animations.animations.iter_mut()
                .find(|a| a.name == name) {
                anim.spring_epsilon = Some(epsilon.clamp(0.0001, 1.0));
            }
        }
    }

    drop(settings);
    self.dirty_tracker.mark(SettingsCategory::Animations);
    self.save_manager.mark_changed();
    Task::none()
}
```

Then change line 172-175 to:
```rust
Message::Animations(msg) => self.update_animations(msg),
```

#### 3.2 Implement update_workspaces()

```rust
fn update_workspaces(&mut self, msg: WorkspacesMessage) -> Task<Message> {
    let mut settings = self.settings.lock().unwrap();

    match msg {
        WorkspacesMessage::AddWorkspace => {
            let new_index = settings.workspaces.workspaces.len() + 1;
            settings.workspaces.workspaces.push(
                format!("Workspace {}", new_index)
            );
        }
        WorkspacesMessage::RemoveWorkspace(index) => {
            if index < settings.workspaces.workspaces.len() {
                settings.workspaces.workspaces.remove(index);
            }
        }
        WorkspacesMessage::UpdateWorkspaceName(index, name) => {
            if let Some(workspace) = settings.workspaces.workspaces.get_mut(index) {
                *workspace = name;
            }
        }
        WorkspacesMessage::MoveWorkspaceUp(index) => {
            if index > 0 && index < settings.workspaces.workspaces.len() {
                settings.workspaces.workspaces.swap(index - 1, index);
            }
        }
        WorkspacesMessage::MoveWorkspaceDown(index) => {
            if index < settings.workspaces.workspaces.len() - 1 {
                settings.workspaces.workspaces.swap(index, index + 1);
            }
        }
    }

    drop(settings);
    self.dirty_tracker.mark(SettingsCategory::Workspaces);
    self.save_manager.mark_changed();
    Task::none()
}
```

#### 3.3 Implement update_window_rules() & update_keybindings()

These are more complex (list editing with detail views). Will need:
- State to track selected rule/keybinding
- Add/remove operations
- Field updates for selected item

**Defer to Phase 7** (after basic categories work).

---

### 4. Missing Message Enums (15 Categories) ⚠️ HIGH

**Current:** Line 36 in `src/messages.rs` says "TODO: Add remaining 15 categories"

**Missing Categories:**
1. Trackpoint
2. Trackball
3. Tablet
4. Touch
5. LayoutExtras
6. Gestures
7. LayerRules
8. Outputs
9. Miscellaneous
10. Startup
11. Environment
12. Debug
13. SwitchEvents
14. RecentWindows
15. (Actually only 14 missing since Cursor is implemented)

**Fix Plan:** Add message enum for each, then implement update handlers

**Template for Simple Categories:**
```rust
// In messages.rs:
#[derive(Debug, Clone)]
pub enum TrackpointMessage {
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    // ... other fields from TrackpointSettings
}

// In Message enum:
Trackpoint(TrackpointMessage),

// In app.rs:
Message::Trackpoint(msg) => self.update_trackpoint(msg),

// Implement handler:
fn update_trackpoint(&mut self, msg: TrackpointMessage) -> Task<Message> {
    let mut settings = self.settings.lock().unwrap();

    match msg {
        TrackpointMessage::SetAccelSpeed(value) => {
            settings.trackpoint.accel_speed = value.clamp(-1.0, 1.0);
        }
        TrackpointMessage::SetAccelProfile(profile) => {
            settings.trackpoint.accel_profile = profile;
        }
        // ... handle other fields
    }

    drop(settings);
    self.dirty_tracker.mark(SettingsCategory::Trackpoint);
    self.save_manager.mark_changed();
    Task::none()
}
```

**Complexity Estimate:**
- Simple categories (8): 15 min each = 2 hours
- Medium categories (4): 30 min each = 2 hours
- Complex categories (2): 1 hour each = 2 hours
- **Total: ~6 hours**

---

### 5. Box::leak Memory Leaks ⚠️ HIGH

**Affected Files (10):**
1. `src/views/widgets/color_picker.rs:28`
2. `src/views/behavior.rs:144-148`
3. `src/views/keyboard.rs` (multiple locations)
4. `src/views/cursor.rs`
5. `src/views/widgets/file_path.rs:32`
6. `src/views/widgets/calibration_matrix.rs`
7. `src/views/widgets/key_capture.rs`
8. `src/views/search_results.rs`
9. `docs/PHASE_7_COMPLETE.md` (documentation, ignore)
10. `docs/PHASE_8_COMPLETE.md` (documentation, ignore)

**Example Issue:** `src/views/widgets/color_picker.rs:28`
```rust
pub fn color_picker_row<'a, Message: Clone + 'a>(
    label: &'static str,
    description: &'static str,
    color: &crate::types::Color,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let hex_value = color.to_hex();
    let hex_static: &'static str = Box::leak(hex_value.clone().into_boxed_str());  // ❌ LEAK

    // ... use hex_static in text_input
}
```

**Problem:**
- Every time view rerenders, a new string is leaked
- Color changes accumulate leaked memory
- Over extended use, could leak MBs

**Fix Strategy:**

**Option A: Accept Owned String (Best)**
```rust
pub fn color_picker_row<'a, Message: Clone + 'a>(
    label: &'static str,
    description: &'static str,
    color: &crate::types::Color,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let hex_value = color.to_hex();

    // text_input can accept String directly via Into<String>
    let input = text_input("", &hex_value)
        .on_input(on_change)
        .padding(8);

    // ... rest of widget
}
```

**Option B: Use Cow (If Needed)**
```rust
use std::borrow::Cow;

pub fn color_picker_row<'a, Message: Clone + 'a>(
    // ...
    value: Cow<'a, str>,  // Can be borrowed or owned
) -> Element<'a, Message> {
    let input = text_input("", value.as_ref())
        .on_input(on_change);
    // ...
}
```

**Fix for behavior.rs:144-148 (Conditional Leak)**
```rust
// ❌ BEFORE:
container(
    iced::widget::text(Box::leak(
        if let Some(key) = mod_key_nested {
            format!("Nested modifier key: {}", key).into_boxed_str()
        } else {
            "...".to_string().into_boxed_str()
        }
    ) as &'static str)
)

// ✅ AFTER:
let message = if let Some(key) = mod_key_nested {
    format!("Nested modifier key: {} (edit behavior.kdl to change)", key)
} else {
    "Nested modifier key: None (uses same as above) - edit behavior.kdl to change".to_string()
};

container(
    iced::widget::text(message)
        .size(13)
        .color([0.6, 0.7, 0.9])
)
```

iced's `text()` widget accepts `impl ToString`, so owned strings work fine!

**Complexity Estimate:** 2-3 hours to fix all 10 files

---

### 6. Dead Code Warnings ℹ️ LOW

**Diagnostics:**
```
src/ipc/mod.rs:
  ⚠ [Line 127:8] struct `NiriOkResponse` is never constructed
  ⚠ [Line 133:8] struct `NiriErrResponse` is never constructed
  ⚠ [Line 139:4] function `parse_niri_response` is never used

src/app.rs:
  ⚠ [Line 26:5] field `paths` is never read
  ⚠ [Line 837:8] method `placeholder_page` is never used
```

**Fix Plan:**

#### 6.1 IPC Structs
These were likely for parsing niri socket responses but aren't used.

**Option A: Remove**
```rust
// Delete lines 127-145 in src/ipc/mod.rs
```

**Option B: Use Them**
Implement proper IPC response parsing if needed. Currently the code just checks success/failure.

**Recommendation:** Remove for now. Add back if IPC parsing is needed later.

#### 6.2 App::paths Field
```rust
// src/app.rs:26
pub struct App {
    settings: Arc<Mutex<Settings>>,
    paths: Arc<ConfigPaths>,  // ❌ Never read
    // ...
}
```

This is stored but never accessed directly. The `SaveManager` has its own copy.

**Fix:** Remove from App struct (it's in SaveManager already).

#### 6.3 placeholder_page() Method
```rust
// src/app.rs:837
fn placeholder_page(&self, page_name: &str) -> Element<'_, Message> {
    // ... creates placeholder UI
}
```

This was used when pages were stubs. Now views are implemented.

**Fix:** Remove method (views handle their own placeholders).

---

## Implementation Priority

### Phase 1: Critical Fixes (Do First) - 4 hours

1. ✅ **Fix window close handler** (30 min)
   - Implement blocking save on exit
   - Test with unsaved changes

2. ✅ **Fix SaveManager race condition** (1 hour)
   - Implement SaveGuard RAII pattern
   - Add tests for panic recovery

3. ✅ **Fix Box::leak in top 3 files** (1.5 hours)
   - color_picker.rs
   - behavior.rs
   - keyboard.rs

4. ✅ **Remove dead code** (30 min)
   - IPC unused structs
   - App::paths field
   - placeholder_page() method

### Phase 2: Message Handlers (Next) - 3 hours

5. ✅ **Implement update_animations()** (1 hour)
   - Add handler in app.rs
   - Test with animations view

6. ✅ **Implement update_workspaces()** (1 hour)
   - Add handler in app.rs
   - Test add/remove/reorder

7. ✅ **Stub update_window_rules() & update_keybindings()** (1 hour)
   - Basic structure
   - Defer complex logic to Phase 3

### Phase 3: Remaining Categories (Later) - 6 hours

8. ⏳ **Add message enums for 14 categories** (3 hours)
9. ⏳ **Implement update handlers for 14 categories** (3 hours)

### Phase 4: Complete Views (Later) - 8 hours

10. ⏳ **Complete animations view** (2 hours)
11. ⏳ **Complete workspaces view** (2 hours)
12. ⏳ **Complete window_rules view** (2 hours)
13. ⏳ **Complete keybindings view** (2 hours)

### Phase 5: Polish (Final) - 4 hours

14. ⏳ **Fix remaining Box::leak (7 files)** (2 hours)
15. ⏳ **Add integration tests** (1 hour)
16. ⏳ **Performance profiling** (1 hour)

---

## Testing Checklist

After each fix, verify:

### Window Close Fix
- [ ] Start app
- [ ] Change a setting (e.g., focus ring width)
- [ ] Close window immediately
- [ ] Reopen app
- [ ] Setting should be saved

### SaveManager Fix
- [ ] Modify save code to panic during save
- [ ] Trigger save
- [ ] Remove panic
- [ ] Next save should work (not stuck)

### Message Handlers
- [ ] Navigate to page
- [ ] Change a setting
- [ ] Check dirty tracker marks category
- [ ] Wait 300ms
- [ ] Verify KDL file written
- [ ] Reload app
- [ ] Setting should persist

### Box::leak Fixes
- [ ] Run with Valgrind or similar
- [ ] Open/close color pickers repeatedly
- [ ] Memory usage should be stable
- [ ] No growing RSS over time

---

## Files to Modify Summary

| File | Changes Needed | Lines | Priority |
|------|----------------|-------|----------|
| `src/app.rs` | Window close fix, add 4 update methods, remove dead code | ~310,26,837 + new | P1 |
| `src/save_manager.rs` | Add SaveGuard, modify save_task() | 80-144 | P1 |
| `src/views/widgets/color_picker.rs` | Remove Box::leak | 28 | P1 |
| `src/views/behavior.rs` | Remove Box::leak | 144-148 | P1 |
| `src/views/keyboard.rs` | Remove Box::leak | Multiple | P1 |
| `src/ipc/mod.rs` | Remove dead code | 127-145 | P1 |
| `src/messages.rs` | Add 14 message enums | 36+ | P2 |
| `src/views/cursor.rs` | Remove Box::leak | TBD | P3 |
| ... 6 more files | Remove Box::leak | TBD | P3 |

**Total Files to Touch:** ~15 files
**Estimated Total Time:** ~25 hours (spread over phases)

---

## Key Insights

### Good News 🎉
1. **Storage layer is complete** - All 25 categories have save/load logic
2. **Config system is solid** - Atomic writes, XDG paths, graceful fallbacks
3. **Architecture is sound** - Elm pattern correctly implemented
4. **Widget library is comprehensive** - Minimal duplication

### Bad News ⚠️
1. **Critical data loss bug** - Window close doesn't wait for save
2. **19 of 25 categories don't work** - Only 6 have update handlers
3. **Memory leaks accumulate** - Box::leak in 10+ files
4. **Race condition in save** - Can permanently block saves

### Migration Status: 65% → 85% After Phase 1+2

After fixing critical issues and implementing 4 main message handlers, the app will be:
- **Safe** (no data loss on exit)
- **Stable** (no race conditions)
- **Functional** (10 of 25 categories working)
- **Production-ready for core use cases**

The remaining 15 categories can be added incrementally as needed.

---

## Next Steps

1. **Start with Phase 1** (Critical Fixes) - highest ROI
2. **Test thoroughly** after each fix
3. **Commit incrementally** with clear messages
4. **Update this document** as work progresses
5. **Create Phase 2 branch** for message handler work

---

**Document Version:** 1.0
**Author:** Multi-agent code review + audit
**Last Updated:** 2026-01-22
**Next Review:** After Phase 1 completion
