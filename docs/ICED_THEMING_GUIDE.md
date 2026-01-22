# Iced 0.13 Theming and Styling Guide

A comprehensive guide to creating beautiful, custom-styled applications with iced, a cross-platform GUI library for Rust.

## Table of Contents

- [Overview](#overview)
- [Theme System](#theme-system)
- [Creating Custom Themes](#creating-custom-themes)
- [Widget Styling](#widget-styling)
- [Color Management](#color-management)
- [Typography and Fonts](#typography-and-fonts)
- [Layout and Spacing](#layout-and-spacing)
- [StyleSheet Pattern](#stylesheet-pattern)
- [Best Practices](#best-practices)
- [Real-World Examples](#real-world-examples)

---

## Overview

Iced's styling system is based on a **theme-first approach** where styling changes the appearance of widgets without affecting layout. The framework provides:

- **23 built-in themes** (Light, Dark, Dracula, Nord, Tokyo Night, Catppuccin variants, etc.)
- **Custom theme support** with flexible palette customization
- **Per-widget styling** through closures and style functions
- **Theme-aware components** that adapt to the active theme

**Key Principle**: Iced does not have a unified styling system. Instead, all built-in widgets follow a consistent styling approach where the `style` method takes a closure that receives the current `Theme` and returns the widget's appearance.

---

## Theme System

### Built-in Themes

Iced 0.13 includes 23 pre-configured themes:

**Light Variants:**
- `Theme::Light`
- `Theme::SolarizedLight`
- `Theme::GruvboxLight`
- `Theme::CatppuccinLatte`
- `Theme::TokyoNightLight`
- `Theme::KanagawaLotus`

**Dark Variants:**
- `Theme::Dark`
- `Theme::Dracula`
- `Theme::Nord`
- `Theme::SolarizedDark`
- `Theme::GruvboxDark`
- `Theme::CatppuccinFrappe`
- `Theme::CatppuccinMacchiato`
- `Theme::CatppuccinMocha`
- `Theme::TokyoNight`
- `Theme::TokyoNightStorm`
- `Theme::KanagawaWave`
- `Theme::KanagawaDragon`
- `Theme::Moonfly`
- `Theme::Nightfly`
- `Theme::Oxocarbon`
- `Theme::Ferra`

**Custom:**
- `Theme::Custom(Arc<Custom>)`

### Using Themes in Applications

Set the theme by implementing the `theme()` method on your application:

```rust
use iced::{Application, Theme};

struct MyApp {
    theme: Theme,
}

impl Application for MyApp {
    // ... other trait methods

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
```

### Dynamic Theme Switching

Make themes dynamic based on application state:

```rust
enum Message {
    NextTheme,
    PreviousTheme,
    ClearTheme,
}

impl MyApp {
    fn update(&mut self, message: Message) {
        match message {
            Message::NextTheme => {
                self.theme = self.next_theme();
            }
            Message::PreviousTheme => {
                self.theme = self.previous_theme();
            }
            Message::ClearTheme => {
                self.theme = Theme::default();
            }
        }
    }
}
```

---

## Creating Custom Themes

### Method 1: Custom Palette

Create a custom theme from a `Palette`:

```rust
use iced::{Color, Theme};
use iced::theme::{Custom, Palette};

// Define custom colors
let custom_palette = Palette {
    background: Color::from_rgb(0.1, 0.1, 0.15),
    text: Color::from_rgb(0.9, 0.9, 0.95),
    primary: Color::from_rgb(0.4, 0.6, 1.0),
    success: Color::from_rgb(0.3, 0.8, 0.4),
    warning: Color::from_rgb(1.0, 0.7, 0.2),
    danger: Color::from_rgb(0.9, 0.3, 0.3),
};

// Create custom theme
let custom_theme = Theme::custom(
    "My Custom Theme".to_string(),
    custom_palette
);
```

### Method 2: Custom with Extended Palette Generator

For more control over color variations:

```rust
use iced::theme::palette::Extended;

let custom_theme = Theme::custom_with_fn(
    "Advanced Custom".to_string(),
    custom_palette,
    |palette| Extended::generate(palette)
);
```

### Palette Structure

The `Palette` struct contains six core colors:

```rust
pub struct Palette {
    pub background: Color,  // Backdrop color
    pub text: Color,        // Text rendering color
    pub primary: Color,     // Main accent color
    pub success: Color,     // Successful states
    pub warning: Color,     // Cautionary conditions
    pub danger: Color,      // Error or critical states
}
```

### Extended Palette

The `Extended` palette provides semantic color sets:

```rust
pub struct Extended {
    pub background: Background,  // Background color variations
    pub primary: Primary,        // Primary color scheme
    pub secondary: Secondary,    // Alternative color scheme
    pub success: Success,        // Positive/successful states
    pub warning: Warning,        // Caution/alert states
    pub danger: Danger,          // Critical/error states
    pub is_dark: bool,           // Dark mode flag
}
```

Each color set contains variations like `base`, `weak`, and `strong` for different UI states.

---

## Widget Styling

### Basic Styling Pattern

All iced widgets follow this pattern:

```rust
widget_name(content)
    .style(|theme: &Theme, status| {
        // Return widget-specific Style struct
    })
```

### Button Styling

#### Using Built-in Styles

```rust
use iced::widget::button;

let styles = [
    ("Primary", button::primary as fn(&Theme, _) -> _),
    ("Secondary", button::secondary),
    ("Success", button::success),
    ("Warning", button::warning),
    ("Danger", button::danger),
    ("Text", button::text),
];

// Apply style
button("Click me")
    .on_press(Message::ButtonPressed)
    .style(button::primary)
```

#### Custom Button Style

```rust
use iced::widget::button;
use iced::{Border, Color, Theme};
use iced::border::Radius;

button("Custom Button")
    .on_press(Message::ButtonPressed)
    .style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        match status {
            button::Status::Active => button::Style {
                background: Some(palette.primary.base.color.into()),
                text_color: palette.primary.base.text,
                border: Border {
                    color: palette.primary.strong.color,
                    width: 2.0,
                    radius: Radius::from(8.0),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
            },
            button::Status::Hovered => button::Style {
                background: Some(palette.primary.strong.color.into()),
                ..button::primary(theme, status)
            },
            button::Status::Pressed => button::Style {
                background: Some(palette.primary.weak.color.into()),
                ..button::primary(theme, status)
            },
            button::Status::Disabled => {
                let mut style = button::primary(theme, status);
                style.background = style.background.map(|bg| {
                    Color { a: 0.5, ..bg.into() }.into()
                });
                style
            }
        }
    })
```

### Container Styling

```rust
use iced::widget::container;
use iced::Border;
use iced::border::Radius;

// Using built-in styles
container(content)
    .style(container::rounded_box)
    .padding(20);

// Custom container style
container(content)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();

        container::Style {
            text_color: Some(palette.background.base.text),
            background: Some(palette.background.weak.color.into()),
            border: Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: Radius::from(12.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
        }
    })
    .padding(20)
```

### Text Input Styling

```rust
use iced::widget::text_input;

text_input("Enter text...", &self.input_value)
    .on_input(Message::InputChanged)
    .style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        match status {
            text_input::Status::Active => text_input::Style {
                background: palette.background.base.color.into(),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: Radius::from(4.0),
                },
                icon: palette.background.weak.text,
                placeholder: palette.background.strong.text,
                value: palette.background.base.text,
                selection: palette.primary.weak.color,
            },
            text_input::Status::Focused => text_input::Style {
                border: Border {
                    color: palette.primary.strong.color,
                    width: 2.0,
                    radius: Radius::from(4.0),
                },
                ..text_input::default(theme, status)
            },
            text_input::Status::Disabled => {
                let mut style = text_input::default(theme, status);
                style.background = Color { a: 0.5, ..style.background.into() }.into();
                style
            }
        }
    })
```

### Slider Styling

```rust
use iced::widget::slider;

slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
    .style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        slider::Style {
            rail: slider::Rail {
                colors: (
                    palette.primary.base.color,
                    palette.background.strong.color,
                ),
                width: 4.0,
                border_radius: Radius::from(2.0),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                color: palette.primary.strong.color,
                border_width: 2.0,
                border_color: palette.background.base.color,
            },
        }
    })
```

### Common Widget Style Properties

Most widgets support these style properties:

```rust
pub struct Appearance {
    pub background: Option<Background>,  // Widget background
    pub text_color: Color,               // Text color
    pub border: Border,                  // Border styling
    pub shadow: Shadow,                  // Shadow effect
}

pub struct Border {
    pub color: Color,     // Border color
    pub width: f32,       // Border thickness
    pub radius: Radius,   // Corner rounding
}

pub struct Shadow {
    pub color: Color,         // Shadow color (use alpha)
    pub offset: Vector,       // X/Y offset
    pub blur_radius: f32,     // Blur amount
}
```

---

## Color Management

### Color Definition

Iced uses RGBA colors with values from 0.0 to 1.0:

```rust
use iced::Color;

// From RGB (0.0 - 1.0)
let color1 = Color::from_rgb(0.5, 0.7, 1.0);

// From RGBA (0.0 - 1.0)
let color2 = Color::from_rgba(0.5, 0.7, 1.0, 0.8);

// From RGB8 (0 - 255)
let color3 = Color::from_rgb8(128, 178, 255);

// From RGBA8 (0 - 255)
let color4 = Color::from_rgba8(128, 178, 255, 204);

// Named colors
let black = Color::BLACK;
let white = Color::WHITE;
let transparent = Color::TRANSPARENT;
```

### Accessing Theme Colors

Extract colors from the current theme:

```rust
// Basic palette
let palette = theme.palette();
let bg_color = palette.background;
let text_color = palette.text;
let primary_color = palette.primary;

// Extended palette (more variations)
let extended = theme.extended_palette();
let primary_base = extended.primary.base.color;
let primary_weak = extended.primary.weak.color;
let primary_strong = extended.primary.strong.color;
let primary_text = extended.primary.base.text;

// Background variations
let bg_base = extended.background.base.color;
let bg_weak = extended.background.weak.color;
let bg_strong = extended.background.strong.color;

// Semantic colors
let success_color = extended.success.base.color;
let warning_color = extended.warning.base.color;
let danger_color = extended.danger.base.color;
```

### Color Manipulation

```rust
// Adjust alpha (transparency)
let semi_transparent = Color { a: 0.5, ..base_color };

// Lighten/darken (manual)
let lighter = Color {
    r: (base_color.r * 1.2).min(1.0),
    g: (base_color.g * 1.2).min(1.0),
    b: (base_color.b * 1.2).min(1.0),
    ..base_color
};

let darker = Color {
    r: base_color.r * 0.8,
    g: base_color.g * 0.8,
    b: base_color.b * 0.8,
    ..base_color
};
```

### Palette Integration

Use the `palette` crate for advanced color operations:

```rust
use iced::Color;

// Convert to HSL for manipulation
fn lighten(color: Color, amount: f32) -> Color {
    // Use palette crate for HSL conversion
    // This requires adding palette as a dependency
    color
}
```

---

## Typography and Fonts

### Text Widget

```rust
use iced::widget::text;
use iced::Font;

text("Hello, World!")
    .size(24)                    // Font size in logical pixels
    .font(Font::MONOSPACE)       // Font family
    .line_height(1.5)            // Relative to size (150%)
    .width(Length::Fill)         // Width constraint
    .height(Length::Shrink)      // Height constraint
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        text::Style {
            color: Some(palette.primary.strong.color),
        }
    })
```

### Font Types

```rust
// Built-in fonts
Font::DEFAULT      // System default font
Font::MONOSPACE    // Monospace font

// Custom fonts
Font {
    family: Family::Name("My Custom Font"),
    weight: Weight::Bold,
    stretch: Stretch::Normal,
    style: Style::Normal,
}
```

### Loading Custom Fonts

Custom fonts must be loaded before use (implementation varies by platform).

### Text Styling

```rust
use iced::widget::text;

// Built-in text styles
text("Success!").style(text::success);
text("Warning!").style(text::warning);
text("Danger!").style(text::danger);

// Custom text style
text("Custom")
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        text::Style {
            color: Some(palette.primary.strong.color),
        }
    })
```

### Line Height Options

```rust
use iced::widget::text::LineHeight;

// Relative to font size
text("Text").line_height(LineHeight::Relative(1.5));  // 150%

// Absolute height in pixels
text("Text").line_height(LineHeight::Absolute(24.0));
```

**Typography Best Practices:**
- Body text: 14-16px
- Line height: 145-150% of font size
- Fonts with large x-heights: 14-15px
- Fonts with small x-heights: 15-16px

---

## Layout and Spacing

### Column Layout

Vertical container that distributes contents vertically:

```rust
use iced::widget::{column, text, button};
use iced::{Length, Alignment};

column![
    text("Title").size(24),
    text("Subtitle").size(16),
    button("Action").on_press(Message::Action),
]
.spacing(20)                              // Space between elements
.padding(30)                              // Padding around content
.width(Length::Fill)                      // Take all available width
.height(Length::Shrink)                   // Use intrinsic height
.align_x(Alignment::Center)               // Horizontal alignment
```

### Row Layout

Horizontal container that distributes contents horizontally:

```rust
use iced::widget::{row, button};

row![
    button("Cancel").on_press(Message::Cancel),
    button("OK").on_press(Message::Ok),
]
.spacing(10)                              // Space between buttons
.padding(15)                              // Padding around content
.width(Length::Shrink)                    // Use intrinsic width
.height(Length::Shrink)                   // Use intrinsic height
.align_y(Alignment::Center)               // Vertical alignment
```

### Space Widget

Create empty space between elements:

```rust
use iced::widget::{Space, row};
use iced::Length;

row![
    button("Left"),
    Space::with_width(Length::Fill),  // Push elements apart
    button("Right"),
]
```

### Container Widget

Wrapper for positioning and styling a single widget:

```rust
use iced::widget::container;
use iced::{Length, Alignment};

container(
    text("Centered Content")
)
.width(Length::Fill)
.height(Length::Fill)
.center_x(Length::Fill)
.center_y(Length::Fill)
.padding(20)
.style(container::rounded_box)
```

### Responsive Widget

Create responsive layouts that adapt to available space:

```rust
use iced::widget::Responsive;

Responsive::new(|size| {
    // size: Size - available space

    if size.width > 800.0 {
        // Desktop layout
        row![
            sidebar(),
            content(),
        ].into()
    } else {
        // Mobile layout
        column![
            hamburger_menu(),
            content(),
        ].into()
    }
})
```

### Length Types

Control widget sizing:

```rust
use iced::Length;

// Fill all available space
Length::Fill

// Fill specific portion (for multiple Fill widgets)
Length::FillPortion(2)  // Takes 2x space compared to FillPortion(1)

// Use intrinsic size (minimum needed)
Length::Shrink

// Fixed size in pixels
Length::Fixed(200.0)
```

### Alignment

```rust
use iced::Alignment;

// Horizontal alignment
Alignment::Start    // Left
Alignment::Center   // Center
Alignment::End      // Right

// Vertical alignment (same values)
```

### Padding

```rust
use iced::Padding;

// Uniform padding
.padding(20)                          // All sides

// Individual sides
.padding(Padding {
    top: 10.0,
    right: 20.0,
    bottom: 10.0,
    left: 20.0,
})

// Shorthand methods
.padding([10, 20])                    // Vertical, Horizontal
.padding([10, 20, 15, 25])            // Top, Right, Bottom, Left
```

---

## StyleSheet Pattern

### Understanding StyleSheet

Iced delegates widget styling to **StyleSheet traits** - traits with functions that return style structs for different widget states.

### StyleSheet Trait Structure

```rust
pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> Appearance;
    fn hovered(&self, style: &Self::Style) -> Appearance;
    fn pressed(&self, style: &Self::Style) -> Appearance;
    fn disabled(&self, style: &Self::Style) -> Appearance;
}
```

**Key Points:**
- **Type Style**: Associated type storing additional style information
- **State methods**: Different appearance for each interaction state
- **Appearance**: Returned struct with visual properties

### Widget-Specific StyleSheets

Different widgets have their own StyleSheet traits:

**Button StyleSheet:**
```rust
pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> button::Style;
    fn hovered(&self, style: &Self::Style) -> button::Style;
    fn pressed(&self, style: &Self::Style) -> button::Style;
    fn disabled(&self, style: &Self::Style) -> button::Style;
}
```

**Text Input StyleSheet:**
```rust
pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> text_input::Style;
    fn focused(&self, style: &Self::Style) -> text_input::Style;
    fn disabled(&self, style: &Self::Style) -> text_input::Style;
    fn placeholder_color(&self, style: &Self::Style) -> Color;
    fn value_color(&self, style: &Self::Style) -> Color;
    fn selection_color(&self, style: &Self::Style) -> Color;
}
```

**Container StyleSheet:**
```rust
pub trait StyleSheet {
    type Style: Default;

    fn style(&self, style: &Self::Style) -> container::Style;
}
```

### Custom StyleSheet Implementation

#### Approach 1: Closure-Based (Modern)

The modern iced approach uses closures for inline styling:

```rust
button("Click me")
    .style(|theme: &Theme, status| {
        // Custom styling logic
        match status {
            button::Status::Active => { /* ... */ },
            button::Status::Hovered => { /* ... */ },
            // ...
        }
    })
```

#### Approach 2: Custom Theme Type (Advanced)

For complete control, implement StyleSheet traits on your own type:

```rust
use iced::widget::button;

pub struct MyTheme {
    palette: Palette,
}

impl button::StyleSheet for MyTheme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> button::Style {
        button::Style {
            background: Some(self.palette.primary.into()),
            text_color: Color::WHITE,
            border: Border {
                color: self.palette.primary,
                width: 2.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Style {
        let active = self.active(style);
        button::Style {
            background: Some(lighten(self.palette.primary, 0.1).into()),
            ..active
        }
    }

    // ... implement other methods
}

// Use in application
impl Application for MyApp {
    type Theme = MyTheme;

    // ...
}
```

### Built-in Style Functions

Iced provides convenience functions for common styles:

```rust
// Buttons
button::primary(theme, status)
button::secondary(theme, status)
button::success(theme, status)
button::warning(theme, status)
button::danger(theme, status)
button::text(theme, status)

// Containers
container::bordered_box(theme)
container::rounded_box(theme)
container::transparent(theme)

// Text
text::default(theme)
text::success(theme)
text::warning(theme)
text::danger(theme)
```

---

## Best Practices

### 1. Use Theme-Aware Styling

Always extract colors from the current theme rather than hardcoding:

```rust
// ❌ Bad: Hardcoded colors
button("Click")
    .style(|_theme, _status| button::Style {
        background: Some(Color::from_rgb(0.2, 0.4, 0.8).into()),
        ..Default::default()
    })

// ✅ Good: Theme-aware
button("Click")
    .style(|theme, status| {
        let palette = theme.extended_palette();
        button::Style {
            background: Some(palette.primary.base.color.into()),
            ..button::primary(theme, status)
        }
    })
```

### 2. Leverage Built-in Styles

Use built-in style functions as a base:

```rust
button("Submit")
    .style(|theme, status| {
        // Start with built-in style
        let mut style = button::success(theme, status);

        // Customize specific properties
        style.border.radius = Radius::from(12.0);

        style
    })
```

### 3. Extract Reusable Styles

Create helper functions for commonly used styles:

```rust
fn card_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: Radius::from(12.0),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

// Use in views
container(content)
    .style(card_style)
    .padding(20)
```

### 4. Consistent Spacing

Define spacing constants:

```rust
mod spacing {
    pub const SMALL: u16 = 8;
    pub const MEDIUM: u16 = 16;
    pub const LARGE: u16 = 24;
    pub const XLARGE: u16 = 32;
}

column![
    // ...
]
.spacing(spacing::MEDIUM)
.padding(spacing::LARGE)
```

### 5. Semantic Color Usage

Use semantic colors for meaningful UI states:

```rust
// ✅ Good: Semantic meaning
text("Operation successful!")
    .style(text::success);

button("Delete")
    .style(button::danger);

// ❌ Bad: Generic colors
text("Operation successful!")
    .style(|theme, _| text::Style {
        color: Some(Color::from_rgb(0.3, 0.8, 0.4)),
    });
```

### 6. Dark Mode Support

Always test both light and dark themes:

```rust
// Extended palette handles light/dark automatically
let palette = theme.extended_palette();

// Use semantic colors that adapt
container(content)
    .style(|theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.base.color.into()),
            text_color: Some(palette.background.base.text),
            ..Default::default()
        }
    })
```

### 7. Responsive Design Patterns

Adapt layouts to available space:

```rust
Responsive::new(|size| {
    let is_mobile = size.width < 600.0;
    let spacing = if is_mobile { 10 } else { 20 };

    if is_mobile {
        column![
            // Mobile layout
        ]
        .spacing(spacing)
        .into()
    } else {
        row![
            // Desktop layout
        ]
        .spacing(spacing)
        .into()
    }
})
```

### 8. Accessible Design

Ensure sufficient contrast and touch targets:

```rust
// Minimum touch target size (44x44 logical pixels)
button("Action")
    .width(Length::Fixed(44.0))
    .height(Length::Fixed(44.0))
```

---

## Real-World Examples

### Applications with Custom Themes

Based on the [awesome-iced](https://github.com/iced-rs/awesome-iced) repository, here are notable projects:

#### **COSMIC Desktop Environment** (Pop!_OS)
The entire COSMIC desktop uses iced with a cohesive design system:
- cosmic-settings: Settings application
- cosmic-text-editor: Text editor
- cosmic-launcher: Application launcher
- cosmic-applets: System applets

**Key Features:**
- System-wide consistent theme
- Dark/light mode support
- Custom widget library
- Professional design language

#### **Halloy**
An open-source IRC client with custom chat interface styling.

[Project Link](https://github.com/squidowl/halloy)

**Styling Highlights:**
- Custom message bubbles
- Channel list theming
- User-defined color schemes
- Responsive layout

#### **Frostbyte Terminal**
Yakuake-inspired dropdown terminal with custom styling.

**Features:**
- Custom terminal themes
- Transparency support
- Configurable appearance

#### **Veloren**
Multiplayer voxel RPG using iced for its launcher frontend.

**UI Features:**
- Game-themed styling
- Complex layout system
- Custom graphics integration

### Theme Libraries

#### **iced_modern_theme**
A comprehensive Modern-inspired theme for iced 0.13.1.

[Crates.io](https://crates.io/crates/iced_modern_theme)

Features:
- Modern design aesthetic
- Complete widget coverage
- Easy integration

#### **marcel**
A theme loading and management system for iced.

Features:
- Dynamic theme loading
- User-distributable themes
- Theme development tools

### Example: Complete Custom Theme

```rust
use iced::{Application, Color, Element, Theme};
use iced::theme::{Custom, Palette};
use iced::widget::{button, column, container, text};

struct MyApp {
    theme: Theme,
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    ToggleTheme,
}

impl Application for MyApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        let custom_palette = Palette {
            background: Color::from_rgb(0.95, 0.95, 0.97),
            text: Color::from_rgb(0.15, 0.15, 0.20),
            primary: Color::from_rgb(0.4, 0.6, 1.0),
            success: Color::from_rgb(0.3, 0.8, 0.4),
            warning: Color::from_rgb(1.0, 0.7, 0.2),
            danger: Color::from_rgb(0.9, 0.3, 0.3),
        };

        (
            Self {
                theme: Theme::custom("Custom".to_string(), custom_palette),
                counter: 0,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        "Custom Theme Demo".to_string()
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::Increment => self.counter += 1,
            Message::Decrement => self.counter -= 1,
            Message::ToggleTheme => {
                self.theme = if matches!(self.theme, Theme::Dark) {
                    Theme::Light
                } else {
                    Theme::Dark
                };
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("Counter Demo")
                .size(32)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.strong.color),
                    }
                }),

            text(format!("Count: {}", self.counter))
                .size(48),

            row![
                button("Decrement")
                    .on_press(Message::Decrement)
                    .style(button::danger),

                button("Increment")
                    .on_press(Message::Increment)
                    .style(button::success),
            ]
            .spacing(20),

            button("Toggle Theme")
                .on_press(Message::ToggleTheme)
                .style(button::secondary),
        ]
        .spacing(30)
        .padding(40)
        .align_x(iced::Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.base.color.into()),
                    ..Default::default()
                }
            })
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
```

### Learning Resources

**Official Documentation:**
- [Iced Docs](https://docs.rs/iced/)
- [Iced Book](https://book.iced.rs/)
- [Official Examples](https://github.com/iced-rs/iced/tree/master/examples)

**Community Resources:**
- [Awesome Iced](https://github.com/iced-rs/awesome-iced) - Curated list of projects and resources
- [Iced Reference Guide](https://austinmreppert.github.io/iced-reference/) - Comprehensive styling guide
- [Video: How to use custom themes in iced](https://github.com/iced-rs/awesome-iced)

**Key Examples:**
- [Styling Example](https://github.com/iced-rs/iced/tree/master/examples/styling) - Official example with theme switching
- [GitHub Discussion: Custom Themes](https://github.com/iced-rs/iced/discussions/1431)

---

## Summary

Iced 0.13 provides a powerful, flexible theming system:

**Strengths:**
- 23 built-in themes covering popular color schemes
- Type-safe styling through closures and traits
- Theme-aware widget styling
- Extensive color palette system
- Clean separation of styling and layout

**Key Takeaways:**
1. Use closures for inline widget styling
2. Extract colors from theme palettes for consistency
3. Leverage built-in style functions as a base
4. Create reusable style helpers for common patterns
5. Test both light and dark themes
6. Use semantic colors for meaningful UI states

**Next Steps:**
- Explore the [official styling example](https://github.com/iced-rs/iced/tree/master/examples/styling)
- Read the [styling reference](https://austinmreppert.github.io/iced-reference/chapter_3.html)
- Study real-world applications like COSMIC and Halloy
- Experiment with custom palettes and themes

---

## Sources

- [iced::theme - Rust](https://docs.rs/iced/latest/iced/theme/index.html)
- [Theme in iced - Rust](https://docs.rs/iced/latest/iced/enum.Theme.html)
- [Styling - Iced Reference](https://austinmreppert.github.io/iced-reference/chapter_3.html)
- [iced GitHub Examples](https://github.com/iced-rs/iced/tree/master/examples/styling)
- [Awesome Iced](https://github.com/iced-rs/awesome-iced)
- [Palette in iced::theme - Rust](https://docs.iced.rs/iced/theme/struct.Palette.html)
- [Extended Palette - Rust](https://docs.rs/iced/latest/iced/theme/palette/struct.Extended.html)
- [Text Widget - iced Book](https://book.iced.rs/text.html)
- [Row Widget - Rust](https://docs.iced.rs/iced/widget/struct.Row.html)
- [Column Widget - Rust](https://docs.rs/iced/latest/iced/widget/struct.Column.html)
- [Container Style - Rust](https://docs.iced.rs/iced/widget/container/struct.Style.html)
