# Phase 3: Appearance Page ✅ COMPLETE

**Duration**: ~30 minutes
**Goal**: Implement complete Appearance page as template for other pages

## Achievements

### ✅ Functional Appearance Page
- Focus ring settings (enable/disable, width slider)
- Border settings (enable/disable, thickness slider)
- Layout settings (gaps, corner radius sliders)
- Conditional rendering (settings only shown when enabled)
- All changes update the real `Settings` struct
- Settings are marked as dirty for Phase 4 auto-save

### ✅ Message Handling
- Expanded `AppearanceMessage` enum with 11 variants
- Implemented `update_appearance()` handler in `app.rs`
- Proper value clamping (1-20px for width, 0-64px for gaps, etc.)
- Settings changes trigger dirty tracking

### ✅ View Architecture
- Clean separation: view takes ownership of `AppearanceSettings` (cheap to clone)
- Uses all Phase 2 widget helpers (`toggle_row`, `slider_row`, `section_header`, etc.)
- Conditional rendering based on toggle state
- Scrollable content with proper padding

## Technical Details

### Files Created/Modified
- ✅ `src/views/appearance.rs` (126 lines)
- ✅ `src/messages.rs` (updated AppearanceMessage enum)
- ✅ `src/app.rs` (added `update_appearance()` method)

### Widget Usage
The appearance page demonstrates all key widget types:
- **toggle_row**: Enable/disable focus ring and border
- **slider_row**: Width, thickness, gaps, corner radius
- **section_header**: Focus Ring, Window Border, Layout sections
- **info_text**: Helpful descriptions for each section
- **spacer**: Visual spacing between sections

### State Management
```rust
fn update_appearance(&mut self, msg: AppearanceMessage) -> Task<Message> {
    let mut settings = self.settings.lock().unwrap();

    match msg {
        AppearanceMessage::SetGaps(value) => {
            settings.appearance.gaps = value.clamp(0.0, 64.0);
        }
        // ... other fields
    }

    drop(settings); // Release lock
    self.dirty_tracker.mark(SettingsCategory::Appearance);
    Task::none()
}
```

### Conditional Rendering
```rust
if focus_ring_enabled {
    slider_row(
        "Ring width",
        "Thickness of the focus ring in pixels",
        focus_ring_width,
        1.0,
        20.0,
        " px",
        |value| Message::Appearance(AppearanceMessage::SetFocusRingWidth(value)),
    )
} else {
    spacer(0.0)
}
```

## Working Features

### ✅ Focus Ring
- Toggle enable/disable
- Adjust width (1-20px) with live preview
- Width slider only shows when enabled

### ✅ Border
- Toggle enable/disable
- Adjust thickness (1-20px) with live preview
- Thickness slider only shows when enabled

### ✅ Layout
- Window gaps slider (0-64px)
- Corner radius slider (0-32px)
- Always visible (not conditional)

### ✅ State Updates
- All changes immediately update `Arc<Mutex<Settings>>`
- Changes marked as dirty via `DirtyTracker`
- Ready for Phase 4 auto-save integration

## Deferred to Phase 7

**Color Pickers**: Color editing requires custom widgets
- Focus ring colors (active, inactive, urgent)
- Border colors (active, inactive, urgent)
- Background color

For now, a placeholder message indicates colors will be editable in Phase 7.

## Performance

- **Settings cloning**: Cheap (AppearanceSettings is ~200 bytes)
- **Lock duration**: Minimal (only during update, not during render)
- **Reactive rendering**: iced only redraws when state actually changes

## Lessons Learned

### Lifetime Management
- Views that borrow from local variables cause lifetime issues
- **Solution**: Take ownership (`AppearanceSettings`) instead of borrowing (`&AppearanceSettings`)
- Since `AppearanceSettings` implements `Clone`, cloning is cheap

### Conditional Rendering
- iced's `column![]` macro requires all items to have the same type
- Can't mix `slider_row()` (returns `Element`) with nothing
- **Solution**: Use `spacer(0.0)` for the `else` branch

## Template for Future Pages

The Appearance page serves as a **template** for implementing the remaining 24 pages:

1. **Clone the settings category** in `page_content()`
2. **Pass ownership** to the view function
3. **Use widget helpers** (`toggle_row`, `slider_row`, etc.)
4. **Implement conditional rendering** with matching types
5. **Update handler** that locks, modifies, marks dirty, unlocks
6. **Return `Element<'static, Message>`** from view

## Next: Phase 4 - SaveManager

Now that settings can be modified, we need to implement:
- Debounced auto-save (300ms)
- Subscription-based periodic checking
- Async KDL file writing
- Toast notifications
- Window close handler (final save)
- IPC niri config reload

**Phase 3 Complete!** The first real settings page is working. 🚀
