# Iced 0.13 Widget Catalog

Comprehensive documentation of all available widgets in iced 0.13 (compatible with iced 0.14) with properties, methods, and practical examples.

## Table of Contents

1. [Core Built-in Widgets](#core-built-in-widgets)
   - [Input Widgets](#input-widgets)
   - [Layout Widgets](#layout-widgets)
   - [Display Widgets](#display-widgets)
   - [Advanced Widgets](#advanced-widgets)
2. [Additional Widgets (iced_aw)](#additional-widgets-iced_aw)
3. [Custom Widget Creation](#custom-widget-creation)
4. [Widget Styling and Theming](#widget-styling-and-theming)

---

## Core Built-in Widgets

### Input Widgets

#### Button

A generic widget that produces a message when pressed.

**Properties & Methods:**
- `new(content)` - Creates button with any content implementing `Into<Element>`
- `width(length)` - Sets button width
- `height(length)` - Sets button height
- `padding(padding)` - Sets padding around content
- `on_press(message)` - Message produced when pressed (required for enabled state)
- `on_press_with(closure)` - Uses closure to produce message (reduces overhead)
- `on_press_maybe(option)` - Conditionally enables button (`None` disables it)
- `style(closure)` - Custom styling function taking `(&Theme, Status)` returning `Style`
- `class(class)` - Sets style class (requires `advanced` feature)
- `clip(bool)` - Controls whether content clips on overflow

**Built-in Styles:**
- `button::primary` - Main action (emphasized)
- `button::secondary` - Complementary action
- `button::success` - Positive outcome
- `button::danger` - Destructive action
- `button::warning` - Risky action
- `button::text` - Text-only (useful for links)
- `button::subtle` - Weak background

**Example:**
```rust
use iced::widget::button;

#[derive(Clone)]
enum Message {
    ButtonPressed,
    Submit,
}

// Basic button
button("Press me!")
    .on_press(Message::ButtonPressed)

// Styled button
button("Submit")
    .on_press(Message::Submit)
    .style(button::primary)
    .width(150)
    .padding(10)

// Disabled button (no on_press)
button("I am disabled!")

// Conditional button
button("Maybe Active")
    .on_press_maybe(if enabled { Some(Message::Submit) } else { None })
```

---

#### TextInput

A single-line text input field.

**Properties & Methods:**
- `new(placeholder, value)` - Creates text input with placeholder and current value
- `id(id)` - Sets unique widget ID
- `on_input(message)` - Message produced when text is typed (required for input)
- `on_input_maybe(option)` - Conditionally enables input
- `on_submit(message)` - Message when Enter is pressed
- `secure(bool)` - Converts to password input (shows dots)
- `icon(config)` - Adds icon decoration
- `width(length)` - Sets input width
- `padding(padding)` - Sets padding
- `size(pixels)` - Sets font size
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::text_input;

#[derive(Clone)]
enum Message {
    InputChanged(String),
    SubmitPressed,
}

// Basic text input
text_input("Type something...", &self.value)
    .on_input(Message::InputChanged)

// Password input
text_input("Password", &self.password)
    .on_input(Message::InputChanged)
    .secure(true)
    .on_submit(Message::SubmitPressed)

// Styled input with icon
text_input("Search...", &self.search)
    .on_input(Message::InputChanged)
    .icon(text_input::Icon {
        font: Font::default(),
        code_point: '🔍',
        size: None,
        spacing: 10.0,
        side: text_input::Side::Left,
    })
    .width(300)
```

---

#### TextEditor

A multi-line text editor with advanced editing capabilities.

**Properties & Methods:**
- `new(content)` - Creates editor with `text_editor::Content`
- `id(id)` - Sets unique widget ID
- `on_action(message)` - Message when editor actions are performed
- `placeholder(text)` - Sets placeholder text
- `width(length)` - Sets editor width
- `height(length)` - Sets editor height
- `min_height(pixels)` - Sets minimum height
- `max_height(pixels)` - Sets maximum height
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::text_editor;

#[derive(Clone)]
enum Message {
    EditorAction(text_editor::Action),
}

struct State {
    content: text_editor::Content,
}

// Text editor
text_editor(&self.content)
    .on_action(Message::EditorAction)
    .placeholder("Write your text here...")
    .height(300)
```

---

#### Slider

A horizontal bar with a handle for selecting a value from a range.

**Properties & Methods:**
- `new(range, value, on_change)` - Creates slider with range, current value, and callback
- `width(length)` - Sets slider width
- `height(length)` - Sets slider height (default: 16px)
- `step(value)` - Sets increment size between values
- `shift_step(value)` - Alternative step when Shift is pressed
- `default(value)` - Reset value (Ctrl/Cmd+click to reset)
- `on_release(message)` - Message sent when mouse is released
- `style(closure)` - Custom styling
- `class(class)` - Style class (requires `advanced` feature)

**Type Requirements:** Generic `T` must implement `Copy`, `From<u8>`, and `PartialOrd`

**Example:**
```rust
use iced::widget::slider;

#[derive(Clone)]
enum Message {
    SliderChanged(f32),
    SliderReleased,
}

// Basic slider
slider(0.0..=100.0, self.value, Message::SliderChanged)

// Advanced slider with all options
slider(0..=255, self.brightness, Message::SliderChanged)
    .step(5)
    .shift_step(10)
    .default(128)
    .on_release(Message::SliderReleased)
    .width(300)
```

---

#### VerticalSlider

A vertical bar with a handle for selecting values.

**Properties & Methods:**
Same as Slider but oriented vertically.

**Example:**
```rust
use iced::widget::vertical_slider;

vertical_slider(0.0..=100.0, self.volume, Message::VolumeChanged)
    .height(200)
```

---

#### Checkbox

A box that can be checked for binary choices.

**Properties & Methods:**
- `new(label, is_checked)` - Creates checkbox with label and state
- `on_toggle(message)` - Message when toggled (takes `bool` parameter)
- `on_toggle_maybe(option)` - Conditionally enables toggling
- `size(pixels)` - Sets checkbox size
- `width(length)` - Sets total width
- `spacing(pixels)` - Space between box and label
- `text_size(pixels)` - Sets label font size
- `icon(icon)` - Custom checkmark icon
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::checkbox;

#[derive(Clone)]
enum Message {
    CheckboxToggled(bool),
}

// Basic checkbox
checkbox("Enable notifications", self.notifications_enabled)
    .on_toggle(Message::CheckboxToggled)

// Styled checkbox
checkbox("I agree to terms", self.agreed)
    .on_toggle(Message::CheckboxToggled)
    .size(20)
    .spacing(10)
    .text_size(14)
```

---

#### Radio

A circular button for selecting one option from multiple choices.

**Properties & Methods:**
- `new(value, option_label, selected, on_select)` - Creates radio button
  - `value` - The value this radio represents
  - `option_label` - Display label
  - `selected` - Currently selected value (optional)
  - `on_select` - Message constructor taking the value
- `size(pixels)` - Sets radio button size
- `width(length)` - Sets total width
- `spacing(pixels)` - Space between button and label
- `text_size(pixels)` - Sets label font size
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::{radio, column};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Rust,
    Python,
    JavaScript,
}

#[derive(Clone)]
enum Message {
    LanguageSelected(Language),
}

// Radio button group
column![
    radio(
        Language::Rust,
        "Rust",
        Some(self.selected_language),
        Message::LanguageSelected,
    ),
    radio(
        Language::Python,
        "Python",
        Some(self.selected_language),
        Message::LanguageSelected,
    ),
    radio(
        Language::JavaScript,
        "JavaScript",
        Some(self.selected_language),
        Message::LanguageSelected,
    ),
]
.spacing(10)
```

---

#### Toggler

A switch for binary choices (on/off).

**Properties & Methods:**
- `new(is_toggled)` - Creates toggler with current state
- `label(text)` - Optional text label
- `on_toggle(message)` - Message when toggled (takes `bool` parameter)
- `on_toggle_maybe(option)` - Conditionally enables toggling
- `width(length)` - Sets width
- `size(pixels)` - Sets toggler size
- `text_size(pixels)` - Sets label font size
- `spacing(pixels)` - Space between switch and label
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::toggler;

#[derive(Clone)]
enum Message {
    TogglerChanged(bool),
}

// Basic toggler
toggler(self.enabled)
    .label("Enable feature")
    .on_toggle(Message::TogglerChanged)

// Without label
toggler(self.dark_mode)
    .on_toggle(Message::TogglerChanged)
    .size(20)
```

---

#### PickList

A dropdown list for selecting a single value from options.

**Properties & Methods:**
- `new(options, selected, on_select)` - Creates pick list
  - `options` - Collection of selectable options
  - `selected` - Currently selected value (optional)
  - `on_select` - Message constructor taking selected value
- `placeholder(text)` - Text shown when nothing selected
- `width(length)` - Sets width
- `padding(padding)` - Sets padding
- `text_size(pixels)` - Sets font size
- `style(closure)` - Custom styling

**Requirements:** Options must implement `Display`, `Clone`, and `Eq`

**Example:**
```rust
use iced::widget::pick_list;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
    Auto,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
enum Message {
    ThemeSelected(Theme),
}

// Pick list
pick_list(
    &[Theme::Light, Theme::Dark, Theme::Auto][..],
    Some(self.theme),
    Message::ThemeSelected,
)
.placeholder("Choose a theme...")
.width(200)
```

---

#### ComboBox

A searchable dropdown for selecting a single value.

**Properties & Methods:**
- `new(state, options, selected, on_select)` - Creates combo box
- `on_input(message)` - Message when text is typed
- `on_option_hovered(message)` - Message when option is hovered
- `on_open(message)` - Message when dropdown opens
- `on_close(message)` - Message when dropdown closes
- `width(length)` - Sets width
- `padding(padding)` - Sets padding
- `size(pixels)` - Sets font size
- `style(closure)` - Custom styling for input
- `menu_style(closure)` - Custom styling for dropdown menu

**Example:**
```rust
use iced::widget::{combo_box, ComboBox};

#[derive(Clone)]
enum Message {
    SearchChanged(String),
    OptionSelected(String),
}

struct State {
    combo_state: combo_box::State<String>,
    selected: Option<String>,
}

// Combo box
combo_box(
    &self.combo_state,
    "Search...",
    self.selected.as_ref(),
    Message::OptionSelected,
)
.on_input(Message::SearchChanged)
.width(300)
```

---

### Layout Widgets

#### Column

A container that distributes contents vertically.

**Properties & Methods:**
- `column![element1, element2, ...]` - Macro for creating columns
- `new()` - Creates empty column
- `push(element)` - Adds element to column
- `spacing(pixels)` - Space between elements
- `padding(padding)` - Padding around contents
- `width(length)` - Sets width
- `height(length)` - Sets height
- `max_width(pixels)` - Sets maximum width
- `align_items(alignment)` - Horizontal alignment of items
- `clip(bool)` - Clips overflowing content

**Alignment Options:**
- `Alignment::Start` - Align to left
- `Alignment::Center` - Center items
- `Alignment::End` - Align to right
- `Alignment::Fill` - Stretch to fill width

**Example:**
```rust
use iced::widget::{column, text, button};
use iced::Alignment;

// Using macro
column![
    text("Header"),
    button("Click me"),
    text("Footer"),
]
.spacing(20)
.padding(10)
.align_items(Alignment::Center)

// Using builder
column::new()
    .push(text("Item 1"))
    .push(text("Item 2"))
    .spacing(10)
```

---

#### Row

A container that distributes contents horizontally.

**Properties & Methods:**
- `row![element1, element2, ...]` - Macro for creating rows
- `new()` - Creates empty row
- `push(element)` - Adds element to row
- `spacing(pixels)` - Space between elements
- `padding(padding)` - Padding around contents
- `width(length)` - Sets width
- `height(length)` - Sets height
- `max_height(pixels)` - Sets maximum height
- `align_items(alignment)` - Vertical alignment of items
- `clip(bool)` - Clips overflowing content

**Alignment Options:**
- `Alignment::Start` - Align to top
- `Alignment::Center` - Center items
- `Alignment::End` - Align to bottom
- `Alignment::Fill` - Stretch to fill height

**Example:**
```rust
use iced::widget::{row, text, button};
use iced::Alignment;

// Using macro
row![
    text("Left"),
    button("Middle"),
    text("Right"),
]
.spacing(20)
.padding(10)
.align_items(Alignment::Center)

// Using builder
row::new()
    .push(text("Item 1"))
    .push(text("Item 2"))
    .spacing(10)
```

---

#### Container

A widget that aligns and styles its contents within boundaries.

**Properties & Methods:**
- `container(element)` - Creates container with content
- `width(length)` - Sets width
- `height(length)` - Sets height
- `max_width(pixels)` - Sets maximum width
- `max_height(pixels)` - Sets maximum height
- `padding(padding)` - Padding around content
- `center_x()` - Centers content horizontally
- `center_y()` - Centers content vertically
- `center()` - Centers content both directions
- `align_x(alignment)` - Horizontal alignment
- `align_y(alignment)` - Vertical alignment
- `clip(bool)` - Clips overflowing content
- `style(closure)` - Custom styling (background, borders, etc.)

**Helper Functions:**
- `center(element)` - Creates centered container
- `center_x(element)` - Centers horizontally
- `center_y(element)` - Centers vertically

**Example:**
```rust
use iced::widget::{container, text};
use iced::{Color, Border, Shadow};

// Basic container
container(text("Centered content"))
    .center()
    .width(300)
    .height(200)

// Styled container
container(text("Styled box"))
    .padding(20)
    .style(|theme| {
        container::Style {
            background: Some(Color::from_rgb(0.2, 0.2, 0.8).into()),
            border: Border {
                color: Color::WHITE,
                width: 2.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    })
```

---

#### Grid

A container that distributes contents on a responsive grid.

**Properties & Methods:**
- `grid![row![...], row![...]]` - Macro for creating grids
- `new()` - Creates empty grid
- `push_row(row)` - Adds a row to the grid
- `spacing(pixels)` - Space between cells
- `padding(padding)` - Padding around grid
- `width(length)` - Sets width
- `height(length)` - Sets height

**Example:**
```rust
use iced::widget::{grid, row, text, button};

grid![
    row![text("A1"), text("A2"), text("A3")],
    row![button("B1"), button("B2"), button("B3")],
    row![text("C1"), text("C2"), text("C3")],
]
.spacing(10)
.padding(20)
```

---

#### Stack

Displays children layered on top of each other.

**Properties & Methods:**
- `stack![element1, element2, ...]` - Macro for creating stacks
- `new()` - Creates empty stack
- `push(element)` - Adds element to top of stack
- `width(length)` - Sets width
- `height(length)` - Sets height

**Example:**
```rust
use iced::widget::{stack, container, text};

// Background with overlay
stack![
    container(text("Background"))
        .width(300)
        .height(200)
        .center()
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.8, 0.8, 0.8).into()),
            ..Default::default()
        }),
    container(text("Overlay"))
        .center()
        .width(100)
        .height(50)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.2, 0.2, 0.8, 0.7).into()),
            ..Default::default()
        }),
]
```

---

#### Scrollable

A widget for displaying scrollable content.

**Properties & Methods:**
- `scrollable(content)` - Creates scrollable area
- `width(length)` - Sets width
- `height(length)` - Sets height
- `direction(direction)` - Sets scroll direction (Vertical, Horizontal, Both)
- `on_scroll(message)` - Message when scrolled
- `id(id)` - Sets unique widget ID
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::{scrollable, column, text};

// Vertical scrollable list
scrollable(
    column![
        text("Item 1"),
        text("Item 2"),
        text("Item 3"),
        // ... many more items
    ]
    .spacing(10)
)
.height(300)

// Horizontal scrollable
scrollable(content)
    .direction(scrollable::Direction::Horizontal)
```

---

#### PaneGrid

A grid of panes that can be split, resized, and reorganized.

**Properties & Methods:**
- `pane_grid(state, view_function)` - Creates pane grid
  - `state` - Grid state tracking pane layout
  - `view_function` - Function creating content for each pane
- `width(length)` - Sets width
- `height(length)` - Sets height
- `spacing(pixels)` - Space between panes
- `on_click(message)` - Message when pane is clicked
- `on_drag(message)` - Message when pane is dragged
- `on_resize(message)` - Message when pane is resized

**Example:**
```rust
use iced::widget::{pane_grid, text, container};

#[derive(Clone)]
enum Message {
    PaneClicked(pane_grid::Pane),
    PaneResized(pane_grid::ResizeEvent),
}

struct State {
    panes: pane_grid::State<String>,
}

pane_grid(&self.panes, |pane, content, _is_maximized| {
    container(text(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center()
        .into()
})
.on_click(Message::PaneClicked)
.on_resize(10, Message::PaneResized)
.spacing(5)
```

---

### Display Widgets

#### Text

A widget for displaying text.

**Properties & Methods:**
- `text(content)` - Creates text widget
- `text!(format_args)` - Macro for formatted text
- `size(pixels)` - Sets font size
- `line_height(height)` - Sets line height
- `font(font)` - Sets font
- `width(length)` - Sets width
- `height(length)` - Sets height
- `horizontal_alignment(alignment)` - Text alignment (Left, Center, Right)
- `vertical_alignment(alignment)` - Vertical alignment
- `shaping(shaping)` - Text shaping strategy
- `style(closure)` - Custom styling (color)

**Example:**
```rust
use iced::widget::text;
use iced::{Color, Font, Alignment};

// Basic text
text("Hello, world!")

// Styled text
text("Large Red Text")
    .size(50)
    .color(Color::from_rgb(1.0, 0.0, 0.0))

// Formatted text using macro
let count = 42;
text!("Count: {}", count)
    .size(20)

// Aligned text
text("Centered")
    .horizontal_alignment(Alignment::Center)
    .width(Length::Fill)
```

---

#### Image

A widget for displaying raster images (PNG, JPG, etc.).

**Properties & Methods:**
- `image(handle)` - Creates image widget
  - `handle` - Can be path string or `image::Handle`
- `width(length)` - Sets width
- `height(length)` - Sets height
- `content_fit(fit)` - How image fits container (Cover, Contain, Fill, ScaleDown, None)
- `filter_method(method)` - Scaling filter (Nearest, Linear)

**Example:**
```rust
use iced::widget::image;

// From file path
image("path/to/image.png")
    .width(300)
    .height(200)
    .content_fit(image::ContentFit::Cover)

// From bytes
use iced::widget::image::Handle;

let handle = Handle::from_bytes(image_bytes);
image(handle)
    .filter_method(image::FilterMethod::Linear)
```

---

#### Svg

A widget for displaying vector graphics (SVG).

**Properties & Methods:**
- `svg(handle)` - Creates SVG widget
  - `handle` - `svg::Handle` from path or bytes
- `width(length)` - Sets width
- `height(length)` - Sets height
- `content_fit(fit)` - How SVG fits container
- `style(closure)` - Custom styling (color override)

**Example:**
```rust
use iced::widget::{svg, Svg};

// From file path
let handle = svg::Handle::from_path("path/to/image.svg");
svg(handle)
    .width(100)
    .height(100)

// From bytes
let handle = svg::Handle::from_memory(svg_bytes);
svg(handle)
    .content_fit(svg::ContentFit::Contain)
```

---

#### ProgressBar

A bar that displays progress.

**Properties & Methods:**
- `progress_bar(range, value)` - Creates progress bar
- `width(length)` - Sets width
- `height(length)` - Sets height
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::progress_bar;

// Basic progress bar
progress_bar(0.0..=100.0, self.progress)
    .width(Length::Fill)
    .height(20)
```

---

#### QRCode

A widget for displaying QR codes.

**Properties & Methods:**
- `qr_code(data)` - Creates QR code from data
- `cell_size(pixels)` - Sets size of each cell

**Requirements:** Enable `qr_code` feature

**Example:**
```rust
use iced::widget::qr_code;

// Create QR code
qr_code("https://example.com")
    .cell_size(4)
```

---

#### Rule

A horizontal or vertical line for dividing content.

**Properties & Methods:**
- `horizontal_rule(height)` - Creates horizontal line
- `vertical_rule(width)` - Creates vertical line
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::{horizontal_rule, vertical_rule};

// Horizontal divider
horizontal_rule(2)

// Vertical divider
vertical_rule(2)
```

---

#### Space

An amount of empty space.

**Properties & Methods:**
- `space()` - Creates empty space filling available space
- `width(length)` - Sets width
- `height(length)` - Sets height

**Example:**
```rust
use iced::widget::{space, row, text};

// Flexible spacing
row![
    text("Left"),
    space().width(Length::Fill),
    text("Right"),
]

// Fixed spacing
space()
    .width(50)
    .height(20)
```

---

### Advanced Widgets

#### Canvas

A widget for drawing 2D graphics.

**Properties & Methods:**
- `canvas(program)` - Creates canvas with draw program
- `width(length)` - Sets width
- `height(length)` - Sets height

**Requirements:** Implement `canvas::Program` trait

**Example:**
```rust
use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size};

struct Circle;

impl canvas::Program<Message> for Circle {
    fn draw(&self, _state: &(), renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 4.0;

        let circle = Path::circle(center, radius);
        frame.fill(&circle, Color::from_rgb(0.3, 0.6, 0.9));

        vec![frame.into_geometry()]
    }
}

// Usage
canvas(Circle)
    .width(200)
    .height(200)
```

---

#### MouseArea

A widget that emits messages on mouse events.

**Properties & Methods:**
- `mouse_area(content)` - Creates mouse-aware area
- `on_press(message)` - Left button press
- `on_release(message)` - Left button release
- `on_right_press(message)` - Right button press
- `on_right_release(message)` - Right button release
- `on_middle_press(message)` - Middle button press
- `on_middle_release(message)` - Middle button release
- `on_enter(message)` - Mouse enters area
- `on_move(message)` - Mouse moves within area
- `on_exit(message)` - Mouse exits area
- `interaction(interaction)` - Sets cursor interaction type

**Example:**
```rust
use iced::widget::{mouse_area, container, text};

#[derive(Clone)]
enum Message {
    Clicked,
    Hovered,
    Exited,
}

mouse_area(
    container(text("Hover or click me"))
        .padding(20)
)
.on_press(Message::Clicked)
.on_enter(Message::Hovered)
.on_exit(Message::Exited)
```

---

#### Tooltip

A widget that displays a tooltip on hover.

**Properties & Methods:**
- `tooltip(content, tooltip_text, position)` - Creates tooltip
  - `content` - The widget to add tooltip to
  - `tooltip_text` - Tooltip content
  - `position` - Tooltip position (Top, Bottom, Left, Right, FollowCursor)
- `gap(pixels)` - Space between content and tooltip
- `padding(padding)` - Tooltip padding
- `style(closure)` - Custom styling

**Example:**
```rust
use iced::widget::{tooltip, button, text};
use iced::widget::tooltip::Position;

// Basic tooltip
tooltip(
    button("Hover me"),
    "This is a tooltip",
    Position::Top,
)

// Styled tooltip
tooltip(
    text("Info"),
    "Detailed information here",
    Position::FollowCursor,
)
.gap(5)
.padding(10)
.style(|theme| {
    tooltip::Style {
        background: Color::from_rgb(0.2, 0.2, 0.2).into(),
        border: Border {
            color: Color::WHITE,
            width: 1.0,
            radius: 5.0.into(),
        },
        text_color: Color::WHITE,
    }
})
```

---

#### Responsive

A widget that adapts based on available space.

**Properties & Methods:**
- `responsive(closure)` - Creates responsive widget
  - `closure` - Function taking `Size` and returning `Element`

**Example:**
```rust
use iced::widget::{responsive, text, column};
use iced::Size;

responsive(|size: Size| {
    if size.width > 600.0 {
        column![
            text("Wide layout").size(30),
            text("More space available"),
        ]
        .spacing(20)
        .into()
    } else {
        column![
            text("Narrow layout").size(20),
            text("Compact view"),
        ]
        .spacing(10)
        .into()
    }
})
```

---

## Additional Widgets (iced_aw)

The `iced_aw` crate provides additional widgets for iced. Version 0.13 is compatible with iced 0.14. Each widget is feature-gated for selective inclusion.

**Installation:**
```toml
[dependencies]
iced = "0.13"
iced_aw = { version = "0.13", features = ["badge", "card", "color_picker"] }
```

### Badge

Visual indicator widget for highlighting or labeling.

**Feature:** `badge`

**Example:**
```rust
use iced_aw::badge;

badge("New")
    .style(badge::primary)
```

---

### Card

Container widget for organizing content in a card layout.

**Feature:** `card`

**Example:**
```rust
use iced_aw::Card;

Card::new(
    text("Card Title"),
    text("Card content goes here..."),
)
.foot(button("Action"))
.max_width(400)
```

---

### ColorPicker

Interactive color selection widget.

**Feature:** `color_picker`

**Example:**
```rust
use iced_aw::ColorPicker;

#[derive(Clone)]
enum Message {
    ColorChanged(Color),
    PickerClosed,
}

ColorPicker::new(
    self.show_picker,
    self.color,
    button("Pick Color"),
    Message::PickerClosed,
    Message::ColorChanged,
)
```

---

### DatePicker

Calendar interface for date selection.

**Feature:** `date_picker`

**Example:**
```rust
use iced_aw::DatePicker;
use time::Date;

#[derive(Clone)]
enum Message {
    DateSelected(Date),
    PickerClosed,
}

DatePicker::new(
    self.show_picker,
    self.selected_date,
    button("Select Date"),
    Message::PickerClosed,
    Message::DateSelected,
)
```

---

### TimePicker

Interface for time selection.

**Feature:** `time_picker`

**Example:**
```rust
use iced_aw::TimePicker;
use time::Time;

#[derive(Clone)]
enum Message {
    TimeSelected(Time),
    PickerClosed,
}

TimePicker::new(
    self.show_picker,
    self.selected_time,
    button("Select Time"),
    Message::PickerClosed,
    Message::TimeSelected,
)
```

---

### NumberInput

Specialized text input that accepts only numeric values.

**Feature:** `number_input`

**Note:** Not available on web platform

**Example:**
```rust
use iced_aw::NumberInput;

#[derive(Clone)]
enum Message {
    NumberChanged(f32),
}

NumberInput::new(
    self.value,
    100.0,
    Message::NumberChanged,
)
.min(0.0)
.max(100.0)
.step(1.0)
```

---

### TabBar & Tabs

Navigation interface with tabbed sections.

**Features:** `tab_bar` and `tabs`

**Example:**
```rust
use iced_aw::{TabBar, TabLabel};

#[derive(Clone)]
enum Message {
    TabSelected(usize),
}

TabBar::new(Message::TabSelected)
    .push(TabLabel::Text("Tab 1".to_string()))
    .push(TabLabel::Text("Tab 2".to_string()))
    .push(TabLabel::Text("Tab 3".to_string()))
    .set_active_tab(&self.active_tab)
```

---

### SelectionList

List of selectable options.

**Feature:** `selection_list`

**Example:**
```rust
use iced_aw::SelectionList;

#[derive(Clone)]
enum Message {
    ItemSelected(usize),
}

SelectionList::new(
    &self.options,
    Message::ItemSelected,
)
.selected(self.selected_index)
```

---

### Menu & ContextMenu

Menu system for navigation and context actions.

**Features:** `menu`, `quad` (for separators)

**Example:**
```rust
use iced_aw::{menu, menu_bar, menu_tree};

menu_bar(vec![
    menu_tree!(
        "File",
        menu_tree!("New", Message::New),
        menu_tree!("Open", Message::Open),
        menu_tree!("Save", Message::Save)
    ),
    menu_tree!(
        "Edit",
        menu_tree!("Cut", Message::Cut),
        menu_tree!("Copy", Message::Copy),
        menu_tree!("Paste", Message::Paste)
    ),
])
```

---

### Sidebar

Layout component for side navigation.

**Feature:** `sidebar`

**Variants:** Standard, FlushColumn, FlushRow

---

### SlideBar

Range selection widget (alternative to slider).

**Feature:** `slide_bar`

---

## Custom Widget Creation

### Using the Widget Trait

To create custom widgets, implement the `iced::advanced::widget::Widget` trait.

**Required Methods:**
1. `size()` - Returns widget dimensions
2. `layout()` - Computes layout node
3. `draw()` - Renders the widget

**Optional Methods:**
- `tag()` & `state()` - Widget identity and state
- `children()` & `diff()` - Child management
- `update()` - Process events
- `operate()` - Apply operations
- `mouse_interaction()` - Cursor behavior
- `overlay()` - Render overlays

**Example: Simple Circle Widget**
```rust
use iced::advanced::{
    layout, renderer,
    widget::{self, Widget},
    Layout, Size, Rectangle,
};
use iced::{Color, Element, Length, Point};
use iced::mouse;

struct Circle {
    radius: f32,
    color: Color,
}

impl Circle {
    fn new(radius: f32, color: Color) -> Self {
        Self { radius, color }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Circle
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.radius * 2.0),
            height: Length::Fixed(self.radius * 2.0),
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(
            Length::Fixed(self.radius * 2.0),
            Length::Fixed(self.radius * 2.0),
            Size::new(self.radius * 2.0, self.radius * 2.0),
        );

        layout::Node::new(size)
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
        use iced::advanced::graphics::geometry::{Path, fill};

        let bounds = layout.bounds();
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::rounded(self.radius),
                ..Default::default()
            },
            self.color,
        );
    }
}

// Helper function to use the widget
fn circle<'a, Message, Theme, Renderer>(radius: f32, color: Color) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Element::new(Circle::new(radius, color))
}

// Usage
circle(50.0, Color::from_rgb(0.3, 0.6, 0.9))
```

---

### Component Pattern (Deprecated)

Note: The `Component` trait was deprecated in iced 0.13 because it introduced encapsulated state and hampered the use of a single source of truth.

**Current Recommendation:** Use the Widget trait directly or leverage the Elm Architecture within your application.

---

## Widget Styling and Theming

### Built-in Theme System

Iced provides a built-in theme system with `Light`, `Dark`, and custom themes.

**Using Built-in Themes:**
```rust
use iced::{Application, Theme};

impl Application for MyApp {
    // ...

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
```

---

### Custom Styling

Each widget accepts a `style()` method that takes a closure returning the widget's style.

**Button Styling Example:**
```rust
use iced::widget::button;
use iced::{Color, Border, Theme};

button("Styled Button")
    .style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        match status {
            button::Status::Active => button::Style {
                background: Some(Color::from_rgb(0.2, 0.6, 0.9).into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                text_color: Color::WHITE,
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Color::from_rgb(0.3, 0.7, 1.0).into()),
                ..button::primary(theme, status)
            },
            button::Status::Pressed => button::Style {
                background: Some(Color::from_rgb(0.1, 0.5, 0.8).into()),
                ..button::primary(theme, status)
            },
            button::Status::Disabled => button::Style {
                background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
                text_color: Color::from_rgb(0.7, 0.7, 0.7),
                ..Default::default()
            },
        }
    })
```

---

### Custom Theme Implementation

Create a fully custom theme by defining style functions for each widget.

**Example:**
```rust
use iced::{application, color, Color};
use iced::widget::{button, container, text};

#[derive(Debug, Clone, Copy, Default)]
pub struct CustomTheme;

// Application theme
impl application::StyleSheet for CustomTheme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: Color::from_rgb(0.1, 0.1, 0.15),
            text_color: Color::from_rgb(0.9, 0.9, 0.95),
        }
    }
}

// Button theme
impl button::StyleSheet for CustomTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.3, 0.6, 0.9).into()),
                border: Border::rounded(4),
                text_color: Color::WHITE,
                ..Default::default()
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.4, 0.4, 0.5).into()),
                border: Border::rounded(4),
                text_color: Color::WHITE,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: Vector::new(0.0, 2.0),
            ..active
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Secondary,
}
```

---

### Using Custom Themes

**Application Level:**
```rust
use iced::Application;

impl Application for MyApp {
    type Theme = CustomTheme;

    fn theme(&self) -> Self::Theme {
        CustomTheme
    }
}
```

**Widget Level:**
```rust
button("Click Me")
    .style(ButtonStyle::Primary)
```

---

## Widget Examples Repository

For more examples, explore the official iced repository:

**GitHub Examples:**
- Tour: Comprehensive widget showcase
- Todos: Dynamic UI with multiple widgets
- Styling: Custom theme examples
- Canvas: Custom drawing
- PaneGrid: Complex layout management
- Custom Widget: Building widgets from scratch

**Run Examples:**
```bash
git clone https://github.com/iced-rs/iced
cd iced/examples
cargo run --package tour
cargo run --package todos
cargo run --package styling
```

---

## Summary

Iced 0.13 provides a comprehensive set of widgets for building cross-platform GUIs:

**Input Widgets:** Button, TextInput, TextEditor, Slider, VerticalSlider, Checkbox, Radio, Toggler, PickList, ComboBox

**Layout Widgets:** Column, Row, Container, Grid, Stack, Scrollable, PaneGrid

**Display Widgets:** Text, Image, Svg, ProgressBar, QRCode, Rule, Space

**Advanced Widgets:** Canvas, MouseArea, Tooltip, Responsive

**iced_aw Widgets:** Badge, Card, ColorPicker, DatePicker, TimePicker, NumberInput, TabBar, SelectionList, Menu, Sidebar, SlideBar

Each widget is highly customizable through styling, supports the Elm Architecture for state management, and can be extended through custom widget creation using the Widget trait.

---

## Sources

- [iced Widget Documentation](https://docs.rs/iced/latest/iced/widget/index.html)
- [iced Official Website](https://iced.rs/)
- [iced GitHub Repository](https://github.com/iced-rs/iced)
- [iced Examples](https://github.com/iced-rs/iced/tree/master/examples)
- [iced Examples README](https://github.com/iced-rs/iced/blob/master/examples/README.md)
- [iced_aw Repository](https://github.com/iced-rs/iced_aw)
- [iced_aw Documentation](https://docs.rs/crate/iced_aw/latest)
- [iced Book](https://book.iced.rs/)
- [Awesome iced](https://github.com/iced-rs/awesome-iced)
- [iced Discourse](https://discourse.iced.rs/)
- [iced Styling Reference](https://austinmreppert.github.io/iced-reference/chapter_3.html)
