# Progress Notes - Gaps UI Bug Investigation

## Date: 2026-01-20

## Issue
The Gaps page in the Appearance section shows a **toggle switch** instead of a **slider** for adjusting gap size. The value "19px" is displayed but users cannot adjust it.

## Screenshot Analysis
- Shows "WINDOW GAPS" section
- "Gaps" label with "Space between windows and screen edges" description
- Right side shows what appears to be a toggle (blue oval with white dot) + "19px" text
- Should show a slider with the value text

## Investigation Findings

### 1. Niri Configuration (Confirmed Correct)
- Niri uses a **single `gaps` value**, NOT separate inner/outer gaps
- To emulate outer gaps, use negative struts
- Current model (`AppearanceSettings.gaps: f32`) is correct
- Sources: https://github.com/YaLTeR/niri/wiki/Configuration:-Layout

### 2. Code Analysis
The code path looks correct:

**Rust side** (`src/ui/bridge/callbacks/appearance.rs`):
```rust
fn make_slider_float(...) -> AppearanceSettingModel {
    AppearanceSettingModel {
        setting_type: 1,  // 1 = slider
        float_value: value,
        min_value: min,
        max_value: max,
        use_float: true,
        ...
    }
}

fn populate_gaps_settings(appearance: &AppearanceSettings) -> ModelRc<AppearanceSettingModel> {
    // Creates slider with setting_type: 1
    make_slider_float("gaps", "Gaps", ..., appearance.gaps, GAP_SIZE_MIN, GAP_SIZE_MAX, "px", true)
}
```

**Slint side** (`ui/pages/appearance.slint`):
```slint
// Control column renders based on setting-type:
if setting.setting-type == 0: Switch { ... }      // Toggle
if setting.setting-type == 1: Slider { ... }      // Slider
if setting.setting-type == 2: ComboBox { ... }    // Dropdown
```

### 3. Suspected Issues
1. **Field mapping**: Slint uses `setting-type` (kebab-case), Rust uses `setting_type` (snake_case). Should auto-convert but might have issue.
2. **Default value**: If `setting_type` isn't properly set, it defaults to 0 (toggle)
3. **Model sync**: The model might not be getting populated correctly on startup

### 4. Debug Code Added
Added debug logging to `populate_gaps_settings()` to verify values:
```rust
debug!(
    "Gaps model: setting_type={}, float_value={}, min={}, max={}, use_float={}",
    model.setting_type, model.float_value, model.min_value, model.max_value, model.use_float
);
```

## Files Modified
- `src/ui/bridge/callbacks/appearance.rs` - Added debug logging

## Next Steps
1. Run with `RUST_LOG=niri_settings=debug` to see the debug output
2. Verify `setting_type` value is 1 (slider) not 0 (toggle)
3. If value is correct in Rust, check Slint binding
4. Compare with Corners page (uses same `make_slider_float`) - does it work?
5. Check if the cosmic-dark Slider widget renders correctly

## Related Files
- `ui/pages/appearance.slint` - DynamicRow component (lines 75-276)
- `src/ui/bridge/callbacks/appearance.rs` - Model creation
- `src/ui/bridge/sync.rs` - Initial sync
- `src/config/models/appearance.rs` - AppearanceSettings struct

## Constants
- `GAP_SIZE_MIN = 0.0`
- `GAP_SIZE_MAX = 64.0`
- `DEFAULT_GAP_SIZE = 16`
