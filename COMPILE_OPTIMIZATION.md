# Slint Compile Time Optimization Guide

This document provides strategies to reduce compile times for niri-settings, a Slint-based Rust application.

## Current Baseline

| Build Type | Time |
|------------|------|
| Clean `cargo check` | ~2m 50s |
| Incremental Rust change | ~0.4s |
| Slint UI change | ~31s |

The Slint compiler itself accounts for most UI rebuild time (~31s). This is independent of the linker.

---

## Quick Wins (Immediate Impact)

### 1. Use mold Linker (10-20x faster linking)

mold is the fastest linker available. Install it:

```bash
# Arch Linux
sudo pacman -S mold

# Ubuntu/Debian
sudo apt install mold

# Fedora
sudo dnf install mold
```

Update `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Benchmark comparison:**
| Linker | Relative Speed |
|--------|----------------|
| GNU ld | 1x (baseline) |
| lld | ~10x faster |
| mold | ~20x faster |

### 2. Reduce Debug Info (30-40% faster incremental builds)

Add to `Cargo.toml`:

```toml
[profile.dev]
debug = "line-tables-only"  # Keep stack traces, skip variable info

[profile.dev.package."*"]
debug = false  # No debug info for dependencies
```

### 3. Split Debug Info

```toml
[profile.dev]
split-debuginfo = "unpacked"  # Faster incremental linking
```

---

## Slint-Specific Optimizations

### 4. Enable Live Preview (Eliminates UI Recompilation)

**This is the biggest win for UI development.** Changes to `.slint` files apply instantly without recompilation.

Add to `Cargo.toml`:

```toml
[dependencies]
slint = { version = "1.14", features = ["live-preview"] }
```

Run with:

```bash
SLINT_LIVE_PREVIEW=1 cargo run
```

Edit any `.slint` file and save - UI updates without restart.

### 5. Avoid renderer-skia

The Skia renderer compiles Skia from source, adding significant build time. Your project uses the defaults (`renderer-femtovg` and `renderer-software`), which is good.

If you explicitly need to disable Skia:

```toml
[dependencies]
slint = { version = "1.14", default-features = false, features = [
    "backend-winit",
    "renderer-femtovg",
    "accessibility"
] }
```

---

## Cargo Profile Optimizations

### 6. Complete Recommended Cargo.toml

```toml
[profile.dev]
opt-level = 0
debug = "line-tables-only"
split-debuginfo = "unpacked"
incremental = true

# Optimize all dependencies (faster runtime, proc-macros compile faster)
[profile.dev.package."*"]
opt-level = 2
debug = false

# Optimize build scripts and proc-macros
[profile.dev.build-override]
opt-level = 2

# Fast iteration profile (no debug info at all)
[profile.dev-fast]
inherits = "dev"
debug = false

# Full debugging when needed
[profile.debugging]
inherits = "dev"
debug = true
split-debuginfo = "off"

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
```

### 7. Complete Recommended .cargo/config.toml

```toml
[build]
jobs = 16  # Adjust to your CPU core count

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# Alternative: use lld if mold not installed
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[alias]
dev-fast = "build --profile dev-fast"
```

---

## Advanced Optimizations

### 8. Cranelift Backend (Nightly, Experimental)

Cranelift generates code faster than LLVM but produces slower binaries. Good for development.

```bash
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
```

Add to `Cargo.toml`:

```toml
# Requires: cargo-features = ["codegen-backend"] at top of file
[profile.dev]
codegen-backend = "cranelift"
```

Run with:

```bash
cargo +nightly build
```

**Results:**
- ~25% faster clean builds
- ~75% faster incremental builds
- No debugger support

### 9. sccache (Compilation Cache)

Caches compiled crates across projects:

```bash
cargo install sccache
```

Add to `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
```

**Limitations:**
- Cannot cache incrementally compiled crates
- Cannot cache proc-macro crates
- Most effective for clean builds

### 10. Parallel Rustc Frontend (Nightly)

```toml
# .cargo/config.toml
[build]
rustflags = ["-Zthreads=8"]
```

Run with `cargo +nightly build`. Can provide up to 50% improvement.

---

## Workspace Splitting (For Large Projects)

If compile times become critical, consider splitting into multiple crates:

```
niri-settings/
├── crates/
│   ├── niri-settings-core/     # Core types, models
│   ├── niri-settings-config/   # KDL loading/saving
│   ├── niri-settings-ui/       # Slint bindings
│   └── niri-settings/          # Thin main binary
```

Benefits:
- Only changed crates recompile
- Better parallelization
- Clearer architecture

---

## Development Workflow Tips

### Use cargo-watch

```bash
cargo install cargo-watch
cargo watch -x check  # Fast type-checking on save
cargo watch -x run    # Auto-restart on changes
```

### Analyze Build Times

```bash
cargo build --timings
```

Generates HTML report showing:
- Which dependencies are slow
- Parallelization efficiency
- Critical path

### Profile Proc-Macros (Nightly)

```bash
cargo +nightly rustc -- -Zmacro-stats
```

---

## Summary: Recommended Changes

### Minimum (Easy Wins)

1. Install mold: `sudo pacman -S mold`
2. Update `.cargo/config.toml` to use mold
3. Set `debug = "line-tables-only"` in dev profile
4. Set `debug = false` for dependencies

### For UI Development

5. Enable `live-preview` feature in Slint
6. Run with `SLINT_LIVE_PREVIEW=1 cargo run`

### Expected Results

| Change | Impact |
|--------|--------|
| mold linker | 10-20x faster linking |
| Reduced debug info | 30-40% faster incremental |
| Live preview | 0s for UI changes (no recompile) |
| Cranelift (nightly) | 25-75% faster compilation |

---

## Tool Comparison

| Tool | Best For | Install |
|------|----------|---------|
| mold | Fastest linking | `pacman -S mold` |
| lld | Good linking, LTO support | Usually pre-installed |
| sccache | Cross-project caching | `cargo install sccache` |
| cargo-watch | Auto-rebuild on save | `cargo install cargo-watch` |
| cargo-nextest | Faster test execution | `cargo install cargo-nextest` |

---

## References

- [Slint Build Configuration](https://docs.rs/slint-build)
- [Slint Live Preview](https://slint.dev/blog/slint-1.13-released)
- [mold Linker](https://github.com/rui314/mold)
- [Rust Performance Book - Compile Times](https://nnethercote.github.io/perf-book/compile-times.html)
- [Cargo Build Performance](https://doc.rust-lang.org/stable/cargo/guide/build-performance.html)
- [Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
