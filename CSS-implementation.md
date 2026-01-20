# CSS Theming Implementation Analysis for niri-settings

## Executive Summary

This document analyzes the potential adoption of `floem-css` for theming in niri-settings-floem, comparing it against the current Rust-native styling approach.

**Recommendation:** **Do NOT adopt floem-css at this time.** The current Rust-native approach is superior for this project's needs.

---

## Current Styling Architecture

### Overview

niri-settings uses a well-architected **Rust-native design system** in `src/ui/theme.rs`:

```
┌─────────────────────────────────────────────────────────────────┐
│                    DESIGN SYSTEM LAYERS                         │
├─────────────────────────────────────────────────────────────────┤
│  1. COLOR PALETTE         │  Catppuccin Mocha (18 colors)       │
│  2. SEMANTIC TOKENS       │  BG_*, TEXT_*, ACCENT_*, STATUS_*   │
│  3. SPACING SYSTEM        │  4px base unit (2XS → 3XL)          │
│  4. TYPOGRAPHY SCALE      │  XS → 2XL (10px → 24px)             │
│  5. BORDER RADIUS         │  XS → FULL (2px → 9999px)           │
│  6. COMPONENT STYLES      │  30+ reusable style functions       │
└─────────────────────────────────────────────────────────────────┘
```

### Current Strengths

| Feature | Current Implementation |
|---------|----------------------|
| **Type Safety** | Compile-time color/spacing validation |
| **IDE Support** | Full autocomplete, go-to-definition |
| **Refactoring** | Rename symbols across entire codebase |
| **Performance** | Zero runtime overhead, no file watching |
| **Conditional Logic** | Native Rust: `if is_on() { accent } else { surface }` |
| **Design Tokens** | Semantic aliases (BG_SURFACE, TEXT_PRIMARY) |

### Example: Current Component Style

```rust
/// Toggle track (on state)
pub fn toggle_track_on_style(s: Style) -> Style {
    toggle_track_style(s)
        .background(ACCENT)
        .border_color(ACCENT)
}

// Usage in component:
Container::new(Empty::new()).style(move |s| {
    if is_on() {
        toggle_track_on_style(s)
    } else {
        toggle_track_style(s)
    }
})
```

---

## floem-css Analysis

### What floem-css Offers

```
┌─────────────────────────────────────────────────────────────────┐
│                     FLOEM-CSS FEATURES                          │
├─────────────────────────────────────────────────────────────────┤
│  ✓ Hot reloading       │  Edit CSS, see changes instantly      │
│  ✓ Familiar syntax     │  CSS-like property names              │
│  ✓ Separate concerns   │  Styles in .css files, not Rust       │
│  ✗ No combinators      │  Can't do `.parent .child { }`        │
│  ✗ Experimental        │  Breaking changes expected            │
│  ✗ Limited docs        │  33% documentation coverage           │
└─────────────────────────────────────────────────────────────────┘
```

### floem-css Example

**style.css:**
```css
button-primary {
    padding: 8px 16px;
    border-radius: 8px;
    background: #cba6f7;
    color: #11111b;
    font-size: 12px;
    font-weight: 600;
}
```

**Rust:**
```rust
use floem_css::{theme_provider, ProviderOptions, StyleCss};

fn my_button() -> impl IntoView {
    button("Click me").css("button-primary")
}
```

---

## Comparative Analysis

### Feature Comparison Matrix

| Capability | Current (Rust) | floem-css | Winner |
|------------|---------------|-----------|--------|
| **Compile-time validation** | ✅ Full | ❌ None | Rust |
| **IDE autocomplete** | ✅ Full | ❌ Limited | Rust |
| **Conditional styling** | ✅ Native | ⚠️ Multiple classes | Rust |
| **Hot reload** | ❌ Requires rebuild | ✅ Instant | floem-css |
| **Designer handoff** | ❌ Rust knowledge | ✅ CSS familiar | floem-css |
| **Stability** | ✅ Mature | ⚠️ Experimental | Rust |
| **Performance** | ✅ Zero overhead | ⚠️ File watching | Rust |
| **Semantic tokens** | ✅ Constants | ⚠️ CSS variables | Rust |
| **Nested selectors** | N/A | ❌ Not supported | Neither |

### Risk Assessment

```
┌─────────────────────────────────────────────────────────────────┐
│                    ADOPTION RISKS                               │
├─────────────────────────────────────────────────────────────────┤
│  HIGH    │  Breaking changes in experimental library            │
│  HIGH    │  Loss of compile-time type safety                    │
│  MEDIUM  │  Migration effort for 30+ style functions            │
│  MEDIUM  │  Debugging CSS parsing errors at runtime             │
│  LOW     │  Learning curve for existing patterns                │
└─────────────────────────────────────────────────────────────────┘
```

---

## Design Recommendation

### Why NOT to Adopt floem-css

1. **Type Safety Loss**
   - Current: Misspell `ACENT` → compile error
   - floem-css: Misspell `button-primry` → silent failure

2. **Conditional Styling Complexity**
   - Current: `if condition { style_a(s) } else { style_b(s) }`
   - floem-css: Manage multiple class strings, toggle manually

3. **No Combinator Support**
   - Can't express `.card .header` relationships
   - Must define flat class names like `card-header`

4. **Experimental Status**
   - Library warns of breaking changes
   - Only 33% documented

5. **Existing Investment**
   - 400+ lines of well-organized design system
   - 30+ component style functions
   - Semantic token architecture already complete

### When floem-css WOULD Make Sense

- Rapid prototyping with non-Rust designers
- Applications requiring user-customizable themes
- Projects without existing design system investment

---

## Alternative: Enhancing Current System

Instead of adopting floem-css, consider these enhancements to the current system:

### 1. Theme Switching Support

```rust
// theme.rs
pub struct Theme {
    pub bg_base: Color,
    pub bg_surface: Color,
    pub accent: Color,
    pub text_primary: Color,
    // ... other tokens
}

impl Theme {
    pub fn catppuccin_mocha() -> Self { /* current colors */ }
    pub fn catppuccin_latte() -> Self { /* light theme */ }
    pub fn nord() -> Self { /* nord colors */ }
}

// Global reactive theme signal
pub static THEME: Lazy<RwSignal<Theme>> = Lazy::new(|| {
    RwSignal::new(Theme::catppuccin_mocha())
});
```

### 2. Dynamic Color Resolution

```rust
// Instead of: .background(BG_SURFACE)
// Use: .background(theme_color(|t| t.bg_surface))

pub fn theme_color(f: impl Fn(&Theme) -> Color) -> Color {
    f(&THEME.get())
}
```

### 3. CSS Export for Documentation

```rust
// Generate CSS documentation of design tokens
pub fn export_theme_as_css(theme: &Theme) -> String {
    format!(r#"
:root {{
    --bg-base: {};
    --bg-surface: {};
    --accent: {};
    --text-primary: {};
}}
"#,
        color_to_hex(theme.bg_base),
        color_to_hex(theme.bg_surface),
        // ...
    )
}
```

---

## Implementation Plan (If Proceeding)

If the decision is made to adopt floem-css despite recommendations:

### Phase 1: Parallel System (Low Risk)
1. Add `floem-css` dependency
2. Create `styles/theme.css` with duplicate definitions
3. Wrap app with `theme_provider`
4. Test hot-reload on new components only

### Phase 2: Gradual Migration (Medium Risk)
1. Convert simple, static components first
2. Maintain Rust styles for conditional logic
3. Document CSS class naming conventions

### Phase 3: Full Migration (High Risk)
1. Convert all components to CSS classes
2. Remove Rust style functions
3. Implement class-toggling for conditional styles

**Estimated Effort:** 40-60 hours for full migration

---

## Detailed Implementation Plan: Theme Switching

Since floem-css isn't recommended, here's a detailed plan for adding **theme switching** to the existing Rust-native system:

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    THEME SYSTEM ARCHITECTURE                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │ Theme       │    │ Theme       │    │ Theme       │        │
│   │ Catppuccin  │    │ Catppuccin  │    │ Nord        │        │
│   │ Mocha       │    │ Latte       │    │             │        │
│   │ (Dark)      │    │ (Light)     │    │ (Dark)      │        │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘        │
│          │                  │                  │                │
│          └──────────────────┼──────────────────┘                │
│                             ▼                                   │
│                    ┌─────────────────┐                          │
│                    │  ACTIVE_THEME   │                          │
│                    │  RwSignal<Theme>│                          │
│                    └────────┬────────┘                          │
│                             │                                   │
│          ┌──────────────────┼──────────────────┐               │
│          ▼                  ▼                  ▼               │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │ bg()        │    │ text()      │    │ accent()    │        │
│   │ Helper      │    │ Helper      │    │ Helper      │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Step 1: Create Theme Struct

**File:** `src/ui/theme.rs` (modify existing)

```rust
use floem::reactive::RwSignal;
use once_cell::sync::Lazy;

/// Complete theme definition with all semantic colors
#[derive(Clone, Copy)]
pub struct Theme {
    // Background layers
    pub bg_deep: Color,
    pub bg_base: Color,
    pub bg_surface: Color,
    pub bg_elevated: Color,
    pub bg_floating: Color,

    // Text hierarchy
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_muted: Color,

    // Accent colors
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_muted: Color,
    pub secondary: Color,

    // Borders
    pub border: Color,
    pub border_subtle: Color,
    pub border_accent: Color,

    // Status
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Interactive
    pub hover_bg: Color,
    pub active_bg: Color,
}
```

### Step 2: Define Theme Presets

```rust
impl Theme {
    /// Catppuccin Mocha (default dark theme)
    pub fn catppuccin_mocha() -> Self {
        Self {
            bg_deep: Color::from_rgb8(0x11, 0x11, 0x1b),
            bg_base: Color::from_rgb8(0x1e, 0x1e, 0x2e),
            bg_surface: Color::from_rgb8(0x31, 0x32, 0x44),
            bg_elevated: Color::from_rgb8(0x45, 0x47, 0x5a),
            bg_floating: Color::from_rgb8(0x58, 0x5b, 0x70),
            text_primary: Color::from_rgb8(0xcd, 0xd6, 0xf4),
            text_secondary: Color::from_rgb8(0xba, 0xc2, 0xde),
            text_tertiary: Color::from_rgb8(0xa6, 0xad, 0xc8),
            text_muted: Color::from_rgb8(0x7f, 0x84, 0x9c),
            accent: Color::from_rgb8(0xcb, 0xa6, 0xf7),
            accent_hover: Color::from_rgb8(0xb4, 0xbe, 0xfe),
            // ... rest of colors
        }
    }

    /// Catppuccin Latte (light theme)
    pub fn catppuccin_latte() -> Self {
        Self {
            bg_deep: Color::from_rgb8(0xdc, 0xe0, 0xe8),
            bg_base: Color::from_rgb8(0xef, 0xf1, 0xf5),
            bg_surface: Color::from_rgb8(0xe6, 0xe9, 0xef),
            bg_elevated: Color::from_rgb8(0xcc, 0xd0, 0xda),
            bg_floating: Color::from_rgb8(0xbc, 0xc0, 0xcc),
            text_primary: Color::from_rgb8(0x4c, 0x4f, 0x69),
            text_secondary: Color::from_rgb8(0x5c, 0x5f, 0x77),
            text_tertiary: Color::from_rgb8(0x6c, 0x6f, 0x85),
            text_muted: Color::from_rgb8(0x8c, 0x8f, 0xa1),
            accent: Color::from_rgb8(0x88, 0x39, 0xef),
            accent_hover: Color::from_rgb8(0x72, 0x87, 0xfd),
            // ... rest of colors
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            bg_deep: Color::from_rgb8(0x2e, 0x34, 0x40),
            bg_base: Color::from_rgb8(0x3b, 0x42, 0x52),
            bg_surface: Color::from_rgb8(0x43, 0x4c, 0x5e),
            // ... nord colors
        }
    }
}
```

### Step 3: Global Theme Signal

```rust
/// Global reactive theme - changes propagate to all components
pub static ACTIVE_THEME: Lazy<RwSignal<Theme>> = Lazy::new(|| {
    RwSignal::new(Theme::catppuccin_mocha())
});

/// Available theme options
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    CatppuccinMocha,
    CatppuccinLatte,
    Nord,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "Catppuccin Mocha (Dark)",
            Self::CatppuccinLatte => "Catppuccin Latte (Light)",
            Self::Nord => "Nord",
        }
    }

    pub fn to_theme(&self) -> Theme {
        match self {
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
            Self::CatppuccinLatte => Theme::catppuccin_latte(),
            Self::Nord => Theme::nord(),
        }
    }
}

/// Switch to a different theme
pub fn set_theme(preset: ThemePreset) {
    ACTIVE_THEME.set(preset.to_theme());
}
```

### Step 4: Dynamic Color Helpers

```rust
/// Get current theme's background base color (reactive)
pub fn bg_base() -> Color {
    ACTIVE_THEME.get().bg_base
}

/// Get current theme's accent color (reactive)
pub fn accent() -> Color {
    ACTIVE_THEME.get().accent
}

// ... similar helpers for all semantic colors
```

### Step 5: Update Style Functions

**Before:**
```rust
pub fn section_style(s: Style) -> Style {
    s.width_full()
        .background(BG_SURFACE)  // Static constant
        .border_color(BORDER_SUBTLE)
}
```

**After:**
```rust
pub fn section_style(s: Style) -> Style {
    let theme = ACTIVE_THEME.get();
    s.width_full()
        .background(theme.bg_surface)  // Dynamic from theme
        .border_color(theme.border_subtle)
}
```

### Step 6: Theme Selector UI

**File:** `src/ui/pages/appearance.rs`

```rust
fn theme_selector() -> impl IntoView {
    let current = RwSignal::new(ThemePreset::CatppuccinMocha);

    let options = vec![
        ThemePreset::CatppuccinMocha,
        ThemePreset::CatppuccinLatte,
        ThemePreset::Nord,
    ];

    Stack::vertical((
        Label::new("Theme"),
        // Dropdown or radio buttons for theme selection
        Stack::horizontal(
            options.into_iter().map(|preset| {
                theme_option_button(preset, current)
            })
        ),
    ))
}

fn theme_option_button(
    preset: ThemePreset,
    current: RwSignal<ThemePreset>
) -> impl IntoView {
    let is_selected = move || current.get() == preset;

    Container::new(Label::new(preset.name()))
        .style(move |s| {
            if is_selected() {
                secondary_tab_selected_style(s)
            } else {
                secondary_tab_style(s)
            }
        })
        .on_click_stop(move |_| {
            current.set(preset);
            set_theme(preset);
        })
}
```

### Step 7: Persist Theme Choice

```rust
// In AppSettings
pub struct AppSettings {
    pub theme: ThemePreset,
    // ... other settings
}

// Load on startup
pub fn load_theme_preference() {
    if let Some(settings) = load_app_settings() {
        set_theme(settings.theme);
    }
}
```

### Migration Checklist

- [ ] Create `Theme` struct with all color fields
- [ ] Implement theme presets (Mocha, Latte, Nord)
- [ ] Add `ACTIVE_THEME` global signal
- [ ] Create color helper functions
- [ ] Update all style functions to use dynamic colors
- [ ] Add theme selector in Appearance page
- [ ] Persist theme preference to settings
- [ ] Test all components with each theme
- [ ] Add theme preview in selector

### Estimated Effort

| Task | Hours |
|------|-------|
| Theme struct & presets | 2 |
| Global signal setup | 1 |
| Update style functions | 4 |
| Theme selector UI | 2 |
| Settings persistence | 1 |
| Testing & polish | 2 |
| **Total** | **~12 hours** |

---

## Conclusion

The current Rust-native styling system in niri-settings is **well-designed and appropriate** for the project's needs. The benefits of floem-css (hot reloading, CSS familiarity) do not outweigh the costs (type safety loss, stability risks, migration effort).

**Recommended Action:** Continue with current architecture, consider adding theme-switching capability using reactive signals for multiple color schemes as detailed above.

---

## Sources

- [Floem Official Site](https://lap.dev/floem/)
- [Floem GitHub](https://github.com/lapce/floem)
- [floem-css GitHub](https://github.com/aalhitennf/floem-css)
- [floem-css Documentation](https://docs.rs/floem-css/latest/floem_css/)
- [2025 Survey of Rust GUI Libraries](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)
