# Iced 0.14 API Reference

**IMPORTANT**: This document reflects iced 0.14 (released December 7, 2025), the final experimental release before 1.0. This is currently the latest stable version.

## Quick Links

- [Official Website](https://iced.rs/)
- [Official Book](https://book.iced.rs/)
- [API Documentation](https://docs.rs/iced/)
- [GitHub Repository](https://github.com/iced-rs/iced)
- [awesome-iced](https://github.com/iced-rs/awesome-iced)

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Getting Started](#2-getting-started)
3. [The Elm Architecture Pattern](#3-the-elm-architecture-pattern)
4. [Core Widgets](#4-core-widgets)
5. [Layout System](#5-layout-system)
6. [Theming and Styling](#6-theming-and-styling)
7. [Tasks and Async Operations](#7-tasks-and-async-operations)
8. [Subscriptions](#8-subscriptions)
9. [Custom Widgets](#9-custom-widgets)
10. [Best Practices](#10-best-practices)
11. [Performance Optimization](#11-performance-optimization)
12. [Testing](#12-testing)
13. [What's New in 0.14](#13-whats-new-in-014)
14. [Migration from 0.13](#14-migration-from-013)

---

## 1. Architecture Overview

### The Elm Architecture (TEA)

iced implements The Elm Architecture, organizing applications around four core concepts:

1. **State** - Your application's data model
2. **Messages** - Events representing user interactions
3. **Update** - Functions that handle messages and transform state
4. **View** - Functions that describe how state dictates displayed widgets

### The Feedback Loop

```
User Interaction → Messages → Update State → View Renders → User Interaction
```

This pattern aligns naturally with Rust's ownership model and emphasizes immutability.

### Key Principle: Single Source of Truth

**Critical**: Input elements never change their own state. They send messages to `update()`, which changes state, then `view()` re-renders.

---

## 2. Getting Started

### Installation

```toml
[dependencies]
iced = "0.14"

# With additional features
iced = { version = "0.14", features = ["tokio", "debug"] }
```

### Common Features

- **`tokio`** - Async runtime support (recommended)
- **`debug`** - Development tools and debugging
- **`time-travel`** - Time-travel debugging (requires debug)
- **`image`** - Image format support (PNG, JPG)
- **`svg`** - SVG rendering
- **`canvas`** - Custom 2D drawing
- **`qr_code`** - QR code generation
- **`lazy`** - Lazy widget support

### Minimal Application (0.14)

```rust
use iced::{Element, Task};
use iced::widget::{button, column, text};

fn main() -> iced::Result {
    iced::run("Counter", update, view)
}

#[derive(Default)]
struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

fn update(counter: &mut Counter, message: Message) -> Task<Message> {
    match message {
        Message::Increment => counter.value += 1,
        Message::Decrement => counter.value -= 1,
    }
    Task::none()
}

fn view(counter: &Counter) -> Element<Message> {
    column![
        button("+").on_press(Message::Increment),
        text(counter.value),
        button("-").on_press(Message::Decrement),
    ]
    .padding(20)
    .into()
}
```

### Application with Builder Pattern

For more complex applications with subscriptions, themes, etc.:

```rust
use iced::{Element, Task, Theme, Subscription};

struct App {
    value: i32,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (App { value: 0 }, Task::none())
    }

    fn title(&self) -> String {
        "My App".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        button(format!("Count: {}", self.value))
            .on_press(Message::Increment)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn main() -> iced::Result {
    iced::application("My App", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .antialiased(true)
        .run_with(|| App::new())
}
```

---

## 3. The Elm Architecture Pattern

### State Definition

All application state lives in one struct:

```rust
struct MyApp {
    counter: i32,
    text_input: String,
    selected_option: Option<String>,
    is_loading: bool,
    // All state here
}
```

### Message Definition

Messages are events modeled as enums:

```rust
#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    TextChanged(String),
    ButtonPressed,
    DataLoaded(Result<Data, Error>),
}
```

**Required derives**: `Clone` and `Debug` are essential.

### Update Logic

Handle state transitions:

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Increment => {
            self.counter += 1;
            Task::none()
        }
        Message::TextChanged(text) => {
            self.text_input = text;
            Task::none()
        }
        Message::ButtonPressed => {
            Task::perform(
                async { fetch_data().await },
                Message::DataLoaded
            )
        }
        Message::DataLoaded(Ok(data)) => {
            self.data = Some(data);
            self.is_loading = false;
            Task::none()
        }
        Message::DataLoaded(Err(e)) => {
            self.error = Some(e.to_string());
            self.is_loading = false;
            Task::none()
        }
    }
}
```

### View Logic

Construct UI declaratively:

```rust
fn view(&self) -> Element<Message> {
    column![
        text("Counter Example"),
        button("+").on_press(Message::Increment),
        text(self.counter),
        button("-").on_press(Message::Decrement),
    ]
    .padding(20)
    .spacing(10)
    .into()
}
```

### Composing Views

Break views into smaller functions:

```rust
fn view(&self) -> Element<Message> {
    container(
        column![
            self.view_header(),
            self.view_content(),
            self.view_footer(),
        ]
    )
    .into()
}

fn view_header(&self) -> Element<Message> {
    row![
        button("Home").on_press(Message::NavigateHome),
        button("Settings").on_press(Message::NavigateSettings),
    ]
    .spacing(10)
    .into()
}
```

---

## 4. Core Widgets

### Button

```rust
use iced::widget::button;

// Basic button
button("Click Me").on_press(Message::ButtonClicked)

// Styled button
button("Primary")
    .on_press(Message::Action)
    .style(button::primary)
    .padding(10)

// Built-in styles:
// button::primary, button::secondary, button::success,
// button::danger, button::warning, button::text, button::subtle
```

**Methods**:
- `.on_press(message)` - Required for button to be clickable
- `.style(style_fn)` - Apply styling
- `.padding(pixels)` - Internal padding
- `.width(length)` - Widget width
- `.height(length)` - Widget height

### Text

```rust
use iced::widget::text;

// Basic text
text("Hello World")

// Styled text
text("Large Text")
    .size(24)
    .color(Color::from_rgb(0.5, 0.5, 0.5))
    .align_x(Alignment::Center)
    .width(Length::Fill)
```

**Methods**:
- `.size(pixels)` - Font size
- `.color(color)` - Text color
- `.font(font)` - Font family
- `.align_x(alignment)` - Horizontal alignment
- `.align_y(alignment)` - Vertical alignment
- `.line_height(height)` - Line height (Relative or Absolute)
- `.width(length)` - Widget width

### TextInput

```rust
use iced::widget::text_input;

// Basic input
text_input("Placeholder...", &self.input_value)
    .on_input(Message::InputChanged)
    .padding(10)

// Password input
text_input("Password", &self.password)
    .on_input(Message::PasswordChanged)
    .password()
    .on_submit(Message::SubmitForm)

// With icon
text_input("Search...", &self.search)
    .on_input(Message::SearchChanged)
    .icon(text_input::Icon {
        font: Font::DEFAULT,
        code_point: '🔍',
        size: Some(16.0.into()),
        spacing: 10.0,
        side: text_input::Side::Left,
    })
```

**Methods**:
- `.on_input(|String| message)` - **Required** for editability
- `.on_submit(message)` - Fired when Enter is pressed
- `.password()` - Hide input characters
- `.icon(icon)` - Add icon to input
- `.padding(pixels)` - Internal padding
- `.size(pixels)` - Font size
- `.line_height(height)` - Line height

### TextEditor

Multi-line text editing:

```rust
use iced::widget::text_editor;

text_editor(&self.editor_content)
    .on_action(Message::EditorAction)
    .placeholder("Enter text...")
    .height(300)
```

**Actions**: `text_editor::Action` includes Edit, Move, Select, etc.

### Checkbox

```rust
use iced::widget::checkbox;

checkbox("Enable feature", self.is_enabled)
    .on_toggle(Message::ToggleFeature)
    .size(20)
    .text_size(16)
```

**Methods**:
- `.on_toggle(|bool| message)` - Handle state changes
- `.size(pixels)` - Checkbox size
- `.text_size(pixels)` - Label text size
- `.spacing(pixels)` - Space between checkbox and label

### Toggler

Switch-style boolean input:

```rust
use iced::widget::toggler;

toggler(self.is_active)
    .label("Active")
    .on_toggle(Message::Toggle)
    .size(20)
    .text_color(Color::WHITE)
```

**Methods**:
- `.label(text)` - Optional label
- `.on_toggle(|bool| message)` - Handle changes
- `.size(pixels)` - Toggler size
- `.text_color(color)` - Label color

### Radio

Single selection from options:

```rust
use iced::widget::radio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Option1,
    Option2,
    Option3,
}

// Create radio group
column![
    radio("Option 1", Choice::Option1, self.selected, Message::ChoiceSelected),
    radio("Option 2", Choice::Option2, self.selected, Message::ChoiceSelected),
    radio("Option 3", Choice::Option3, self.selected, Message::ChoiceSelected),
]
```

### Slider

```rust
use iced::widget::slider;

slider(0..=100, self.value, Message::SliderChanged)
    .step(1u8)
    .shift_step(5u8)
```

**Methods**:
- `.step(value)` - Normal step increment
- `.shift_step(value)` - Step when Shift key held
- `.width(length)` - Slider width
- `.height(length)` - Slider height

### PickList

Dropdown selection:

```rust
use iced::widget::pick_list;

pick_list(
    vec!["Option 1", "Option 2", "Option 3"],
    self.selected.as_ref(),
    Message::OptionSelected
)
.placeholder("Choose an option")
.menu_height(200)
```

**Methods**:
- `.placeholder(text)` - Text when nothing selected
- `.menu_height(pixels)` - Maximum menu height
- `.text_size(pixels)` - Font size
- `.padding(pixels)` - Internal padding

### ComboBox

Searchable dropdown:

```rust
use iced::widget::combo_box;

combo_box(
    &self.combo_state,
    "Search...",
    self.selected.as_ref(),
    Message::ComboSelected
)
.menu_height(200)
```

**State management**:
```rust
struct MyApp {
    combo_state: combo_box::State<String>,
}

impl MyApp {
    fn new() -> Self {
        Self {
            combo_state: combo_box::State::new(vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
            ]),
        }
    }
}
```

### ProgressBar

```rust
use iced::widget::progress_bar;

progress_bar(0.0..=100.0, self.progress)
    .height(20)
```

### Image

```rust
use iced::widget::image;

// From handle
image(image::Handle::from_path("path/to/image.png"))
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(150.0))

// From bytes
image(image::Handle::from_bytes(bytes))
```

### Svg

```rust
use iced::widget::svg;

svg(svg::Handle::from_path("icon.svg"))
    .width(Length::Fixed(50.0))
    .height(Length::Fixed(50.0))
    .style(|theme, status| svg::Style {
        color: Some(theme.palette().text),
    })
```

### QRCode

```rust
use iced::widget::qr_code;

qr_code(&self.qr_data)
    .cell_size(10)
```

### Tooltip

```rust
use iced::widget::tooltip;

tooltip(
    button("Hover me").on_press(Message::Action),
    "This is a tooltip",
    tooltip::Position::Bottom
)
.gap(10)
.delay(500) // New in 0.14: delay before showing
```

### Scrollable

```rust
use iced::widget::scrollable;

scrollable(
    column![
        // Long content here
    ]
)
.width(Length::Fill)
.height(Length::Fill)
.direction(scrollable::Direction::Vertical)
```

**Methods** (0.14 enhancements):
- `.auto_scroll()` - Automatically scroll to bottom on content changes
- `.hidden()` - Hide scrollbars
- `.direction()` - Vertical, Horizontal, or Both

### Canvas

Custom 2D drawing:

```rust
use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size};

Canvas::new(MyCanvasProgram::new())
    .width(Length::Fill)
    .height(Length::Fill)

// Implement canvas::Program trait
struct MyCanvasProgram;

impl canvas::Program<Message> for MyCanvasProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Draw circle
        let circle = Path::circle(Point::new(100.0, 100.0), 50.0);
        frame.fill(&circle, Color::from_rgb(0.3, 0.5, 0.8));

        // Draw line
        let path = Path::line(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
        frame.stroke(&path, Stroke::default().with_width(2.0));

        vec![frame.into_geometry()]
    }
}
```

---

## 5. Layout System

### Column

Vertical layout:

```rust
use iced::widget::column;
use iced::{Alignment, Length};

column![
    text("Header"),
    text("Body"),
    text("Footer"),
]
.spacing(10)
.padding(20)
.align_x(Alignment::Center)
.width(Length::Fill)
```

**Methods**:
- `.spacing(pixels)` - Vertical spacing between elements
- `.padding(pixels)` - Padding around column
- `.align_x(alignment)` - Horizontal alignment (Start, Center, End)
- `.width(length)` - Column width
- `.height(length)` - Column height

**New in 0.14**: `.wrap()` and `.wrapped()` methods with customizable spacing

### Row

Horizontal layout:

```rust
use iced::widget::row;

row![
    button("First"),
    button("Second"),
    button("Third"),
]
.spacing(10)
.padding(20)
.align_y(Alignment::Center)
.height(Length::Shrink)
```

**Methods**:
- `.spacing(pixels)` - Horizontal spacing between elements
- `.padding(pixels)` - Padding around row
- `.align_y(alignment)` - Vertical alignment (Start, Center, End)
- `.width(length)` - Row width
- `.height(length)` - Row height

**New in 0.14**: `.wrap()` and `.wrapped()` methods

### Container

Wrapper for positioning and styling a single widget:

```rust
use iced::widget::container;
use iced::{Alignment, Length};

container(text("Centered"))
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .padding(20)
    .style(container::rounded_box)
```

**Methods**:
- `.center_x(length)` - Center horizontally (requires explicit Length in 0.14)
- `.center_y(length)` - Center vertically
- `.padding(pixels)` - Internal padding
- `.width(length)` - Container width
- `.height(length)` - Container height
- `.style(style_fn)` - Apply styling

### Stack

Layer widgets on top of each other:

```rust
use iced::widget::stack;

stack![
    container(text("Background"))
        .width(Length::Fill)
        .height(Length::Fill),
    container(text("Foreground"))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
]
```

### Space

Empty space for layout:

```rust
use iced::widget::{Space, column};

column![
    text("Top"),
    Space::with_height(50), // Vertical space
    text("Bottom"),
]
```

### Grid

Grid layout (new in 0.14):

```rust
use iced::widget::grid;

grid![
    [button("1"), button("2"), button("3")],
    [button("4"), button("5"), button("6")],
    [button("7"), button("8"), button("9")],
]
.column_spacing(5)
.row_spacing(5)
```

### Length Types

Control widget sizing:

```rust
use iced::Length;

// Fill - Take all available space
.width(Length::Fill)

// FillPortion - Proportional fill
.width(Length::FillPortion(2)) // Takes 2/3 if sibling has FillPortion(1)

// Shrink - Use intrinsic size (minimum needed)
.width(Length::Shrink)

// Fixed - Exact pixel size
.width(Length::Fixed(200.0))
```

**Note**: In 0.14, Shrink is prioritized over Fill in layout logic.

### Responsive

Adaptive layouts:

```rust
use iced::widget::responsive;

responsive(|size| {
    if size.width > 800.0 {
        // Desktop layout
        row![sidebar(), content()].into()
    } else {
        // Mobile layout
        column![header(), content()].into()
    }
})
```

---

## 6. Theming and Styling

### Built-in Themes (23 total)

```rust
use iced::Theme;

fn theme(&self) -> Theme {
    match self.theme_choice {
        // Core themes
        ThemeChoice::Light => Theme::Light,
        ThemeChoice::Dark => Theme::Dark,

        // Popular themes
        ThemeChoice::Dracula => Theme::Dracula,
        ThemeChoice::Nord => Theme::Nord,

        // Solarized
        ThemeChoice::SolarizedLight => Theme::SolarizedLight,
        ThemeChoice::SolarizedDark => Theme::SolarizedDark,

        // Gruvbox
        ThemeChoice::GruvboxLight => Theme::GruvboxLight,
        ThemeChoice::GruvboxDark => Theme::GruvboxDark,

        // Catppuccin
        ThemeChoice::CatppuccinLatte => Theme::CatppuccinLatte,
        ThemeChoice::CatppuccinFrappe => Theme::CatppuccinFrappe,
        ThemeChoice::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
        ThemeChoice::CatppuccinMocha => Theme::CatppuccinMocha,

        // Tokyo Night
        ThemeChoice::TokyoNight => Theme::TokyoNight,
        ThemeChoice::TokyoNightStorm => Theme::TokyoNightStorm,
        ThemeChoice::TokyoNightLight => Theme::TokyoNightLight,

        // Kanagawa
        ThemeChoice::KanagawaWave => Theme::KanagawaWave,
        ThemeChoice::KanagawaDragon => Theme::KanagawaDragon,
        ThemeChoice::KanagawaLotus => Theme::KanagawaLotus,

        // Others
        ThemeChoice::Moonfly => Theme::Moonfly,
        ThemeChoice::Nightfly => Theme::Nightfly,
        ThemeChoice::Oxocarbon => Theme::Oxocarbon,
        ThemeChoice::Ferra => Theme::Ferra,
    }
}
```

### Theme Palette

Every theme has a 6-color palette:

```rust
use iced::Color;

let palette = theme.palette();

palette.background  // Background color
palette.text        // Text color
palette.primary     // Primary/accent color
palette.success     // Success actions
palette.danger      // Destructive actions
palette.warning     // New in 0.14: warning color
```

### Extended Palette

More semantic colors:

```rust
let extended = theme.extended_palette();

// Each has base, weak, and strong variants
extended.background.base
extended.background.weak
extended.background.strong

extended.primary.base
extended.primary.weak
extended.primary.strong

extended.success.base
// ... and so on
```

### Custom Themes

Create custom themes:

```rust
use iced::{Theme, theme::{Palette, Custom}};
use iced::Color;
use std::sync::Arc;

fn custom_theme() -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "My Theme".to_string(),
        Palette {
            background: Color::from_rgb(0.1, 0.1, 0.1),
            text: Color::from_rgb(0.9, 0.9, 0.9),
            primary: Color::from_rgb(0.3, 0.6, 0.9),
            success: Color::from_rgb(0.2, 0.8, 0.3),
            danger: Color::from_rgb(0.9, 0.2, 0.2),
            warning: Color::from_rgb(0.9, 0.7, 0.2),
        }
    )))
}
```

### Widget Styling

Style widgets using closures:

```rust
use iced::widget::{button, container, text};
use iced::{Border, Background, Color};

// Button styling
button("Styled")
    .on_press(Message::Action)
    .style(|theme: &Theme, status| {
        button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 0.8))),
            text_color: Color::WHITE,
            border: Border {
                radius: [5.0; 4].into(),
                width: 2.0,
                color: Color::BLACK,
            },
            shadow: Shadow::default(),
        }
    })

// Container styling
container(content)
    .style(|theme| {
        container::Style {
            background: Some(Background::Color(theme.palette().background)),
            border: Border {
                radius: [10.0; 4].into(),
                width: 1.0,
                color: theme.palette().primary,
            },
            text_color: Some(theme.palette().text),
            shadow: Shadow::default(),
        }
    })

// Text styling
text("Colored")
    .style(|theme| {
        Color::from_rgb(0.5, 0.5, 0.5)
    })
```

### Built-in Style Functions

```rust
// Buttons
button::primary    // Primary action
button::secondary  // Secondary action
button::success    // Positive action
button::danger     // Destructive action
button::warning    // Warning action (new in 0.14)
button::text       // Text-only button
button::subtle     // Subtle button

// Containers
container::rounded_box  // Rounded corners
container::bordered_box // With border
container::dark         // Dark background

// Text
text::default  // Default text color
text::primary  // Primary color
text::success  // Success color
text::danger   // Danger color
text::warning  // Warning color (new in 0.14)
```

---

## 7. Tasks and Async Operations

### What are Tasks?

**Tasks** (renamed from Commands in 0.13) represent one-shot async operations that may produce messages.

### Creating Tasks

```rust
use iced::Task;

// No-op
Task::none()

// Immediate value
Task::done(value)

// Run a Future
Task::future(my_async_function())

// Run a Future and map result to message
Task::perform(
    async { fetch_data().await },
    |result| Message::DataLoaded(result)
)

// Batch multiple tasks
Task::batch(vec![
    Task::perform(load_config(), Message::ConfigLoaded),
    Task::perform(load_cache(), Message::CacheLoaded),
])
```

### Task Transformations

```rust
// Map output
task.map(|value| Message::Transform(value))

// Chain tasks sequentially
task.chain(other_task)

// Discard output
task.discard()

// Collect results from multiple tasks
Task::batch(tasks).collect() // produces Vec<T>

// Abortable task
let (task, handle) = task.abortable();
// Later: handle.abort()
```

### Example: HTTP Request

```rust
use iced::Task;

#[derive(Debug, Clone)]
enum Message {
    FetchData,
    DataLoaded(Result<Data, String>),
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::FetchData => {
            self.is_loading = true;
            Task::perform(
                async {
                    // Perform HTTP request
                    match reqwest::get("https://api.example.com/data").await {
                        Ok(response) => response.json::<Data>().await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(e.to_string()),
                    }
                },
                Message::DataLoaded
            )
        }
        Message::DataLoaded(Ok(data)) => {
            self.data = Some(data);
            self.is_loading = false;
            Task::none()
        }
        Message::DataLoaded(Err(e)) => {
            self.error = Some(e);
            self.is_loading = false;
            Task::none()
        }
    }
}
```

### Background Threading

Offload CPU-intensive work:

```rust
Task::perform(
    async move {
        tokio::task::spawn_blocking(move || {
            // CPU-intensive operation
            expensive_computation()
        })
        .await
        .unwrap()
    },
    Message::ComputationComplete
)
```

---

## 8. Subscriptions

### What are Subscriptions?

**Subscriptions** are continuous streams of events that produce messages. They're active as long as your `subscription()` method returns them.

### Timer Subscription

```rust
use iced::{Subscription, time};
use std::time::Duration;

fn subscription(&self) -> Subscription<Message> {
    if self.timer_active {
        time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}
```

### Keyboard Events

```rust
use iced::keyboard;

fn subscription(&self) -> Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| {
        match key {
            keyboard::Key::Character(c) if c == "s" && modifiers.command() => {
                Some(Message::Save)
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                Some(Message::Cancel)
            }
            _ => None,
        }
    })
}
```

**Note**: In 0.14, keyboard subscriptions were unified into `keyboard::listen`.

### Window Events

```rust
use iced::event;

fn subscription(&self) -> Subscription<Message> {
    event::listen().map(|event| {
        match event {
            Event::Window(window::Event::Resized { width, height }) => {
                Message::WindowResized(width, height)
            }
            Event::Window(window::Event::CloseRequested) => {
                Message::CloseRequested
            }
            _ => Message::Ignore,
        }
    })
}
```

### Custom Subscriptions

```rust
use iced::subscription;

fn subscription(&self) -> Subscription<Message> {
    subscription::run_with_id(
        "my_worker",
        async_stream_generator()
    )
    .map(Message::WorkerEvent)
}

async fn async_stream_generator() -> impl Stream<Item = WorkerOutput> {
    // Create async stream
    futures::stream::repeat_with(|| {
        // Generate events
    })
}
```

### Batching Subscriptions

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch(vec![
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        keyboard::on_key_press(handle_key),
        event::listen().map(Message::Event),
    ])
}
```

---

## 9. Custom Widgets

### Function-Based Composition (Recommended)

The recommended approach is to use regular Rust functions:

```rust
fn settings_section<'a>(
    title: &str,
    items: Vec<Element<'a, Message>>
) -> Element<'a, Message> {
    column![
        text(title).size(20),
        column(items).spacing(10)
    ]
    .padding(20)
    .into()
}

// Usage
fn view(&self) -> Element<Message> {
    settings_section("Appearance", vec![
        checkbox("Dark mode", self.dark_mode)
            .on_toggle(Message::ToggleDarkMode)
            .into(),
        text("Font size:").into(),
        slider(10..=24, self.font_size, Message::FontSizeChanged).into(),
    ])
}
```

### Custom Widget with Widget Trait

For truly custom rendering and behavior:

```rust
use iced::advanced::{Layout, Widget, widget};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer;
use iced::{Element, Length, Size, Theme};

struct Circle {
    radius: f32,
}

impl Circle {
    fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Circle
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &Limits,
    ) -> Node {
        Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // Custom drawing logic
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            Color::from_rgb(0.3, 0.5, 0.8),
        );
    }
}

// Helper function to create widget
fn circle<'a, Message>(radius: f32) -> Element<'a, Message> {
    Element::new(Circle::new(radius))
}
```

**Note**: The `Component` trait is deprecated as of 0.13.0. Use function composition or the Widget trait instead.

---

## 10. Best Practices

### 1. Embrace The Elm Architecture

- All state in one place
- State changes only in `update()`
- Widgets are stateless and emit messages
- Keep update logic pure and testable

### 2. State Design

```rust
// Good: Explicit states using enums
enum AppState {
    Loading,
    Loaded(Data),
    Error(String),
}

// Bad: Boolean soup
struct AppState {
    is_loading: bool,
    has_error: bool,
    has_data: bool,
    // Makes impossible states possible
}
```

### 3. Message Organization

```rust
// For complex apps, organize messages by feature
#[derive(Debug, Clone)]
enum Message {
    // Nested enums for features
    Settings(SettingsMessage),
    Editor(EditorMessage),
    Network(NetworkMessage),

    // Top-level navigation
    NavigateTo(Screen),
}

#[derive(Debug, Clone)]
enum SettingsMessage {
    ToggleDarkMode,
    FontSizeChanged(u16),
    // ...
}
```

### 4. Error Handling

```rust
use anyhow::{Context, Result};

// Use anyhow for application errors
fn load_config() -> Result<Config> {
    let data = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    parse_config(&data)
        .context("Failed to parse config")
}

// In update(), convert to message
Message::LoadConfig => {
    Task::perform(
        async { load_config().await },
        |result| Message::ConfigLoaded(result.map_err(|e| e.to_string()))
    )
}
```

### 5. Performance Tips

- Use `Lazy` widget for expensive views
- Cache computed values in state
- Use Canvas caching for 2D graphics
- Batch tasks when possible
- Prefer `Length::Shrink` over `Length::Fill` when appropriate (0.14 optimization)

### 6. Project Structure

```
src/
├── main.rs           # Entry point
├── app.rs            # Root app struct
├── message.rs        # Root message enum
├── state.rs          # Application state
├── screens/          # Feature modules
│   ├── home.rs
│   ├── settings.rs
│   └── editor.rs
├── widgets/          # Custom widgets
│   └── custom_button.rs
└── services/         # Business logic
    ├── api.rs
    └── storage.rs
```

### 7. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_logic() {
        let mut app = MyApp::default();

        app.update(Message::Increment);
        assert_eq!(app.counter, 1);

        app.update(Message::Decrement);
        assert_eq!(app.counter, 0);
    }
}
```

---

## 11. Performance Optimization

### Reactive Rendering (0.14)

**Automatic improvement**: Reactive rendering is enabled by default in 0.14. The runtime only redraws when state changes.

**Benefits**:
- 20% faster rendering on WebGPU
- 60-80% CPU reduction for static UIs
- Lower GPU usage
- No code changes needed

### Lazy Widget

Cache expensive view computations:

```rust
use iced::widget::Lazy;

Lazy::new(dependency, |state| {
    // Expensive view computation
    expensive_view(state)
})
```

Only rebuilds when `dependency` changes.

### Canvas Caching

```rust
use iced::widget::canvas::Cache;

struct MyCanvas {
    cache: Cache,
}

impl canvas::Program<Message> for MyCanvas {
    fn draw(&self, ...) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Drawing only happens when cache is invalidated
            draw_complex_shapes(frame);
        });
        vec![geometry]
    }
}

// Clear cache when data changes
self.canvas.cache.clear();
```

### Background Processing

```rust
// CPU-intensive work
Task::perform(
    async move {
        tokio::task::spawn_blocking(move || {
            process_large_file()
        })
        .await
        .unwrap()
    },
    Message::ProcessingComplete
)
```

### Image Optimization

- Load images on background threads
- Cache image handles in state
- Use appropriate image formats (WebP, AVIF)
- Resize images before loading

---

## 12. Testing

### Unit Testing State Logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_increment() {
        let mut app = Counter { value: 0 };
        app.update(Message::Increment);
        assert_eq!(app.value, 1);
    }

    #[test]
    fn test_state_transitions() {
        let mut state = AppState::Loading;
        // Test state machine transitions
        state.handle_event(Event::DataArrived);
        assert!(matches!(state, AppState::Loaded(_)));
    }
}
```

### Testing View Functions

```rust
#[test]
fn test_view_creation() {
    let state = MyApp::default();
    let element = state.view();
    // Element created without panicking
}
```

### Integration Tests

```rust
// tests/integration_test.rs
#[test]
fn test_full_workflow() {
    let mut app = MyApp::new().0;

    // Simulate user actions
    app.update(Message::LoadData);
    app.update(Message::ProcessData);
    app.update(Message::SaveResults);

    // Verify final state
    assert!(app.results.is_some());
}
```

---

## 13. What's New in 0.14

### Major Features

#### 1. Reactive Rendering

- Enabled by default
- Only redraws changed UI portions
- 20% rendering improvements
- 60-80% CPU reductions

#### 2. Time Travel Debugging

Enable with features:
```toml
iced = { version = "0.14", features = ["debug", "time-travel"] }
```

- Press F12 to open Comet debugger
- Step through message history
- Replay application states
- Requires `Clone` on Message types

#### 3. Animation API

Direct control over animations in application code.

#### 4. Headless Testing

First-class end-to-end testing support for CI/CD.

#### 5. Input Method Support

Full IME support for international text input.

### New Widgets

- `table` - Structured data display
- `grid` - Grid layouts
- `sensor` - Reactive input detection
- `float` - Floating elements
- `pin` - Fixed positioning

### Widget Enhancements

- Column/Row: `.wrap()` and `.wrapped()` methods
- Scrollable: `.auto_scroll()`, `.hidden()` methods
- Tooltip: `.delay()` configuration
- Text Editor: indent/unindent, line endings
- Markdown: incremental parsing, quotes, task lists

### Breaking Changes

1. **Event by reference**: Widget::update takes `&Event` instead of `Event`
2. **Keyboard subscriptions**: Unified into `keyboard::listen`
3. **Center methods**: Require explicit Length parameter
4. **Layout priority**: Shrink now prioritized over Fill

---

## 14. Migration from 0.13

### Quick Checklist

1. **Update Cargo.toml**:
   ```toml
   iced = "0.14"
   ```

2. **Event handling**: Change `Event` to `&Event` in custom widgets
   ```rust
   // Old
   fn update(&mut self, state: &mut State, event: Event) -> Status

   // New
   fn update(&mut self, state: &mut State, event: &Event) -> Status
   ```

3. **Keyboard subscriptions**: Replace with unified `keyboard::listen`
   ```rust
   // Old
   keyboard::on_key_press(handler)

   // New
   keyboard::listen().map(|event| match event {
       keyboard::Event::KeyPressed { key, .. } => handler(key),
       _ => Message::Ignore,
   })
   ```

4. **Center methods**: Add explicit Length
   ```rust
   // Old
   container(content).center_x()

   // New
   container(content).center_x(Length::Fill)
   ```

5. **Review layouts**: Check Fill/Shrink behavior if layouts break

6. **Test thoroughly**: Reactive rendering may expose timing issues

### For Most Applications

Most apps will see immediate performance benefits with minimal code changes. The reactive rendering improvements are automatic.

---

## 15. Additional Resources

### Official

- [iced.rs](https://iced.rs/) - Official website
- [book.iced.rs](https://book.iced.rs/) - Official book
- [docs.rs/iced](https://docs.rs/iced/) - API documentation
- [GitHub](https://github.com/iced-rs/iced) - Source code
- [Discourse](https://discourse.iced.rs/) - Community forum

### Community

- [awesome-iced](https://github.com/iced-rs/awesome-iced) - Curated resources
- [iced_aw](https://github.com/iced-rs/iced_aw) - Additional widgets
- [Unofficial Guide](https://jl710.github.io/iced-guide/) - Community guide

### Notable Applications

- **COSMIC Desktop** - System76's desktop environment
- **Halloy** - IRC client
- **Liana** - Bitcoin wallet
- **Veloren** - Voxel RPG
- **OctaSine** - FM Synth plugin

---

## Quick Reference

### Essential Imports

```rust
use iced::{
    widget::{button, column, container, row, text, text_input},
    Alignment, Element, Length, Task, Theme, Color,
};
```

### Common Patterns

```rust
// No-op task
Task::none()

// Async operation
Task::perform(async_fn(), Message::Handler)

// Batch tasks
Task::batch(vec![task1, task2])

// Column layout
column![widgets...].spacing(10).padding(20)

// Row layout
row![widgets...].spacing(10).padding(20)

// Center container
container(widget).center_x(Length::Fill).center_y(Length::Fill)

// Button with style
button("Text").on_press(Message).style(button::primary)

// Text input
text_input("Placeholder", &value).on_input(Message)

// Timer subscription
time::every(Duration::from_secs(1)).map(|_| Message::Tick)
```

---

## Version Info

- **iced Version**: 0.14.0
- **Release Date**: December 7, 2025
- **Status**: Last experimental release before 1.0
- **Rust MSRV**: 1.82 (Rust 2024 edition)
- **Next Milestone**: 1.0 (stable API)
