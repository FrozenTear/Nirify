# Quick Start Guide - When You Get Home

## 1. Run the PoC Application

```bash
cd ~/niri-tweaks/vizia-poc
cargo run
```

**What you should see:**
- Window opens titled "Niri Settings (Vizia PoC)"
- Dark theme with sidebar on left
- Three navigation items: Keyboard, Mouse, Touchpad
- Keyboard page shown by default with sliders and toggles

**Test it:**
- Click sidebar items to switch pages
- Drag sliders - values should update in real-time
- Toggle checkboxes - they should change state
- Click "Save Settings" - status message should appear

## 2. Measure Compile Times (Most Important!)

### Option A: Automated Benchmark

```bash
cd ~/niri-tweaks/vizia-poc
./benchmark.sh
```

This will run 3 compile time tests and display results.

### Option B: Manual Comparison

**Test Slint baseline:**
```bash
cd ~/niri-tweaks
cargo clean && time cargo build
# Note the "real" time (e.g., 3m45s)

touch ui/pages/keyboard.slint && time cargo build
# Note the incremental time (e.g., 0m12s)
```

**Test Vizia PoC:**
```bash
cd ~/niri-tweaks/vizia-poc
cargo clean && time cargo build
# Note the "real" time (e.g., 1m30s)

touch src/ui/keyboard_page.rs && time cargo build
# Note the incremental time (e.g., 0m5s)
```

## 3. Decision Time

**Calculate the improvement:**
- Clean build improvement: `Slint time - Vizia time`
- Incremental build improvement: `Slint time - Vizia time`

**Decision matrix:**

| Improvement | Action |
|-------------|--------|
| < 1 min | ❌ **DON'T MIGRATE** - Not worth 6-9 weeks |
| 1-2 min | ⚠️ **MAYBE** - Depends on priorities |
| 2-3 min | ✅ **PROCEED** - Good ROI |
| > 3 min | ✅✅ **STRONGLY RECOMMEND** - Excellent gain |

## 4. If Results Look Good

Read the full migration plan:
```bash
cat ~/niri-tweaks/VIZIA_MIGRATION_ANALYSIS.md
```

Key sections to review:
- Phase 1: Core Infrastructure (what to build first)
- Section 5: Migration Recommendations (detailed plan)
- Section 9: Appendix (important files to understand)

## Troubleshooting

### Build fails with "package vizia not found"

Update Cargo.toml with the correct Vizia version:
```bash
cargo search vizia
# Update vizia-poc/Cargo.toml with latest version
```

### "error: linker clang not found"

Edit `.cargo/config.toml` and remove the linker settings:
```toml
[target.x86_64-unknown-linux-gnu]
# Comment out or remove these lines
# linker = "clang"
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### Window doesn't open

Check for error messages:
```bash
RUST_LOG=debug cargo run
```

### Vizia version issues

Try different Vizia versions in Cargo.toml:
- Latest stable: Check crates.io
- Git version: `vizia = { git = "https://github.com/vizia/vizia" }`

## Questions?

- Full analysis: `VIZIA_MIGRATION_ANALYSIS.md` (in parent directory)
- PoC details: `README.md` (this directory)
- Code structure: Browse `src/` directory

## What's Next?

If you decide to proceed:
1. Create a new branch for Phase 1
2. Set up the full project structure (not just PoC)
3. Port the DynamicSettingsSection pattern
4. Implement 3-5 simple settings pages
5. Validate that everything works before continuing

---

**Remember:** The PoC is just to test compile times and UX. Don't expect full functionality!
