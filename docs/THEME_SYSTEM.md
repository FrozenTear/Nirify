# Theme System Implementation

**Status**: ✅ Complete
**Date**: 2026-01-22

## Overview

Implemented a complete theme system with two themes and a UI selector for runtime switching.

## Features

### 1. Theme Architecture

**Location**: `src/theme.rs`

```rust
pub enum AppTheme {
    NiriAmber,          // Custom warm amber/teal theme (default)
    CatppuccinMocha,    // Popular dark theme
}
```

**Key Functions**:
- `to_iced_theme()` - Converts AppTheme to iced Theme
- `all()` - Returns array of available themes
- `name()` - Returns display name

### 2. NiriAmber Custom Theme

**Implementation**: `build_niri_amber_theme()` function

**Color Mapping**:
```rust
Palette {
    background: #1a1d23,  // Deep charcoal base
    text:       #e6e8eb,  // High contrast text
    primary:    #f59e42,  // Warm amber accent
    success:    #10b981,  // Green
    warning:    #f59e0b,  // Amber warning
    danger:     #ef4444,  // Red error
}
```

**Extended Palette**:
- NiriColors provides additional colors for custom widgets:
  - Surface variations: `bg_surface`, `bg_surface_hover`, `bg_input`
  - Text hierarchy: `text_secondary`, `text_tertiary`
  - Additional accents: `accent_secondary` (teal), `accent_tertiary` (purple)
  - Borders: `border_subtle`, `border_strong`
  - Effects: `glow_accent`, `shadow_color`

**Usage**:
- Base iced palette handles core widgets
- Custom style functions (nav_tab_style, etc.) use full NiriColors for detailed control

### 3. Theme Selector UI

**Location**: `src/views/status_bar.rs`

**Design**:
```
[Status] • [Message]     [niri settings v0.1.0] • [◐ Niri Amber]
                                                      ^^^^^^^^^^^^^
                                                      Click to cycle
```

**Features**:
- **Icon**: Moon symbol (◐) for visual identity
- **Current theme name**: Displays active theme
- **Click interaction**: Cycles through all available themes
- **Hover feedback**: Subtle background and border
- **Press feedback**: Darker background with stronger border
- **Position**: Right side of status bar

**Behavior**:
```rust
// Click cycles to next theme
fn next_theme(current: AppTheme) -> AppTheme {
    let themes = AppTheme::all();
    let next_idx = (current_idx + 1) % themes.len();
    themes[next_idx]
}
```

### 4. Integration

**App State**: `src/app.rs`
```rust
pub struct App {
    current_theme: AppTheme,  // Default: AppTheme::NiriAmber
}
```

**Message Handling**:
```rust
Message::ChangeTheme(theme) => {
    self.current_theme = theme;
    Task::none()
}
```

**Application Configuration**:
```rust
iced::application(App::new, App::update, App::view)
    .theme(|app: &App| app.current_theme.to_iced_theme())
```

## Theme Comparison

| Feature | NiriAmber | Catppuccin Mocha |
|---------|-----------|------------------|
| Base | Deep charcoal (#1a1d23) | Catppuccin dark |
| Primary accent | Warm amber (#f59e42) | Catppuccin mauve |
| Secondary accent | Teal cyan (#4fd1c5) | Catppuccin blue |
| Visual style | Professional, warm | Popular, soft pastels |
| Use case | Default, branded | User preference |

## User Experience

1. **Default Theme**: NiriAmber (warm amber/teal aesthetic)
2. **Theme Switching**: Click theme selector in status bar
3. **Instant Feedback**: Theme changes immediately across entire UI
4. **Persistence**: Current theme stored in App state (not yet persisted to disk)

## Extensibility

### Adding a New Theme

1. **Add to enum** (`src/theme.rs`):
```rust
pub enum AppTheme {
    NiriAmber,
    CatppuccinMocha,
    NewTheme,  // ← Add here
}
```

2. **Implement conversion**:
```rust
pub fn to_iced_theme(self) -> Theme {
    match self {
        AppTheme::NiriAmber => build_niri_amber_theme(),
        AppTheme::CatppuccinMocha => Theme::CatppuccinMocha,
        AppTheme::NewTheme => Theme::CatppuccinFrappe,  // ← Add here
    }
}
```

3. **Add display name**:
```rust
pub fn name(self) -> &'static str {
    match self {
        AppTheme::NiriAmber => "Niri Amber",
        AppTheme::CatppuccinMocha => "Catppuccin Mocha",
        AppTheme::NewTheme => "New Theme",  // ← Add here
    }
}
```

4. **Done!** - Theme selector automatically shows the new theme

### Creating Custom Themes

To create a custom theme like NiriAmber:

```rust
fn build_custom_theme() -> Theme {
    let palette = Palette {
        background: Color::from_rgb(...),
        text: Color::from_rgb(...),
        primary: Color::from_rgb(...),
        success: Color::from_rgb(...),
        warning: Color::from_rgb(...),
        danger: Color::from_rgb(...),
    };

    Theme::custom("Theme Name".to_string(), palette)
}
```

## Typography System

**Location**: `src/theme.rs`

```rust
pub mod fonts {
    pub const UI_FONT: Font           // Normal weight
    pub const UI_FONT_MEDIUM: Font    // Medium weight
    pub const UI_FONT_SEMIBOLD: Font  // Semibold
    pub const MONO_FONT: Font         // Monospace
    pub const MONO_FONT_MEDIUM: Font  // Monospace medium
}
```

**Current Implementation**: System defaults (SansSerif, Monospace)

**Usage**:
- Navigation tabs: `UI_FONT_SEMIBOLD` (active), `UI_FONT_MEDIUM` (inactive)
- Sub-navigation: `UI_FONT_MEDIUM` (active), `UI_FONT` (inactive)
- Future: Use `MONO_FONT` for technical values (numbers, paths, hex codes)

**Customization**: Change `Family::SansSerif` to `Family::Name("Geist Sans")` etc.

## Future Enhancements

- [ ] Persist theme choice to config file
- [ ] Add more built-in themes (Catppuccin variants, Gruvbox, etc.)
- [ ] Theme preview before switching
- [ ] Custom theme editor
- [ ] Per-category theme overrides
- [ ] Light theme variants
- [ ] High contrast accessibility themes

## Testing

✅ **Compilation**: Successful with 0 errors
✅ **Runtime**: Theme switching works correctly
✅ **NiriAmber**: Custom theme renders properly
✅ **UI Integration**: Theme selector appears in status bar
✅ **Cycling**: Clicking cycles through both themes

## Summary

The theme system provides:

1. **Two themes** out of the box (NiriAmber default, Catppuccin Mocha)
2. **Custom NiriAmber theme** with warm amber/teal palette
3. **Professional UI selector** integrated into status bar
4. **Easy extensibility** for adding new themes
5. **Typography constants** for consistent font usage
6. **Zero memory leaks** with proper lifetime management

Users can now switch between themes with a single click while the app is running.
