# Comprehensive UI/UX Design Guide for Wayland Compositor Settings Application

## Executive Summary

This guide synthesizes best practices from six major design systems to provide actionable recommendations for building a modern, accessible, and user-friendly settings application for a Wayland compositor. The principles outlined here draw from Material Design 3, Apple Human Interface Guidelines, Microsoft Fluent Design System, GNOME HIG, KDE HIG, and IBM Carbon Design System.

---

## Table of Contents

1. [Design System Analysis](#design-system-analysis)
2. [Core Design Principles](#core-design-principles)
3. [Settings Organization & Information Architecture](#settings-organization--information-architecture)
4. [Component Guidelines](#component-guidelines)
5. [Spacing & Layout System](#spacing--layout-system)
6. [Typography](#typography)
7. [Color & Theming](#color--theming)
8. [Accessibility Requirements](#accessibility-requirements)
9. [Best Practices Summary](#best-practices-summary)
10. [Implementation Checklist](#implementation-checklist)

---

## Design System Analysis

### 1. Material Design 3 (Material You)

**Key Principles:**
- **Dynamic Color & Personalization**: Uses HCT color space (Hue, Chroma, Tone) for color extraction and system-wide palette generation
- **Seven Foundations**: Color, Typography, Shape, Motion, Interaction, Layout, and Elevation
- **Accessibility & Inclusivity**: Built-in support for contrast ratios, scalable typography, responsive grids
- **Grid System**: Flexible 4dp baseline (refined from 8dp) for precise alignment; 12-column responsive grid
- **Meaningful Motion**: Smooth transitions, subtle feedback animations (ripple effects)

**Settings UI Patterns:**
- Settings should be reached via "Settings" label (never "Options" or "Preferences")
- Place in navigation drawer below all items except Help & Feedback
- **Organization Guidelines:**
  - 7 or fewer preferences: Don't group
  - 9-16 preferences: Use section dividers
  - 16+ preferences: Create subscreens
- Prioritize frequently used settings; group less important ones

**Components for Settings:**
- Selection components: checkboxes, radio buttons, chips, switches, sliders
- Five component categories: action, containment, navigation, selection, text input

**Material 3 Expressive (2025):**
- Enhanced animation and more colorful design
- Improved designer control over theme and motion
- Users recognize UI elements up to 4x faster
- Helps older users spot elements as fast as young users

### 2. Apple Human Interface Guidelines (HIG)

**Core Principles:**
- **Clarity**: Legible, precise, easy to understand interfaces
- **Deference**: UI helps focus on content by minimizing visual clutter
- **Depth**: Visual layers and realistic motion convey hierarchy

**2025 Update - Liquid Glass Design System:**
- Refined color palette
- Bolder left-aligned typography
- Concentricity for unified rhythm between hardware and software
- Harmonized design language across devices

**macOS Design Considerations:**
- Support in-depth productivity tasks
- Enable viewing media/content
- Allow multitasking with several apps at once
- Leverage power, spaciousness, and flexibility

**Key HIG Categories:**
- **Hierarchy**: Prioritize content so users know what's important
- **Harmony**: Align with Apple's hardware and software
- **Consistency**: Use familiar patterns across devices

**Accessibility:**
- Minimum 11pt font size
- Dynamic Type for user-adjustable text
- High contrast between text and background
- Avoid overly decorative typefaces
- Support for VoiceOver and accessibility features

### 3. Microsoft Fluent Design System

**Five Key Components:**
- Light, Depth, Motion, Material, Scale

**Design Principles:**
- Innovation
- Clarity
- Inclusiveness
- User-centeredness
- Consistency

**Fluent 2 (Current Version):**
- Seamless collaboration and creativity
- Fluid movement from design to development
- Cross-platform support (Windows, iOS, Android, macOS, Web)

**Typography:**
- Segoe UI as base font (Microsoft's signature typeface)
- Baseline alignment for consistent vertical rhythm
- Left-aligned text for LTR languages
- Avoid justified text in web pages

**Spacing & Layout:**
- Proximity creates perceived relationships
- Spacing creates logical sections without dividers
- More spacing = higher perceived importance
- Consistent spacing for familiar visual rhythm

**Design Tokens:**
- Two layers: Global tokens (raw values) and Alias tokens (semantic meaning)
- Store color, typography, border radius, stroke width, animation values
- Enable consistent designs across platforms

### 4. GNOME Human Interface Guidelines

**Core Framework:**
- Intended for GTK 4 and Libadwaita
- Design principles, Guidelines, and Patterns

**Key Spacing Guidelines:**
- Leave space in increments of 6 pixels
- 12 horizontal pixels between labels and components
- 18 pixels vertical spacing between component groups
- 18 pixels general padding between dialog content and window borders
- 12 pixels indentation for hierarchy

**Layout Principles:**
- Left-to-right, top-to-bottom placement
- First element top-left, last element bottom-right
- Exact alignment of controls
- Right-justify labels when possible (avoid large gaps)
- Use white space and indentation instead of frames

**2025 Updates:**
- Cantarell replaced by Adwaita Sans (based on Inter)
- Source Code Pro replaced by Adwaita Mono (based on Iosevka)

**Preferences Design:**
- Three style options: light, dark, follow system preference
- Use existing style classes and color variables
- Automatically adjust for light, dark, and high-contrast

**Libadwaita:**
- Responsive layouts based on available screen space
- Integrates Adwaita stylesheet
- Runtime recoloring with named colors
- Cross-desktop dark style preference support

### 5. KDE Human Interface Guidelines

**Central Principle:**
"Simple by default, powerful when needed"

**Philosophy:**
- Don't limit to small, single-purpose apps
- Appeal to basic users through experts
- Provide powerful features and extensibility
- Target users from basic technical knowledge to professionals

**Technical Framework:**
- QtQuick with Kirigami
- Hardware-accelerated rendering
- Declarative UI design
- Better animation, touch, and gesture support
- Convergent UI toolkit (adapts to device form factor)

**Spacing Guidelines:**
- 800x600 rule of thumb for configuration dialogs
- Can add ~50 pixels per dimension if needed
- Dialogs should resize proportionally with font size increases

**Accessibility Testing:**
- Test with keyboard only (mouse/touchpad disabled)
- Important text shouldn't be in hover tooltips only
- Test with screen reader (Orca)
- Disable animations and verify static displays
- Avoid blinking UI elements

**UI Spacing & Layout:**
- Maximize space for main content area
- Avoid unnecessary frames, spacing, padding around content
- Use QtQuick.ListView or QtQuick.GridView for multiple items
- Lists superior for mostly-textual content

### 6. IBM Carbon Design System

**Foundation:**
- Based on IBM Design Language
- Working code, design tools, guidelines
- Open source and community-driven

**Design Principles:**
- Purpose-driven elements
- Design with restraint
- Emphasize the essential
- Work for everyone (not just average users)
- Inclusive design

**Icon Design:**
- Unique and non-redundant
- Follow IBM Design Language
- Meet IBM accessibility standards
- Work well on all platforms/devices
- Scale correctly at different sizes
- Culturally neutral symbols

**Components:**
- 30+ component building blocks
- React, Angular, Vue, Svelte, Vanilla JS support
- Follows IBM Accessibility Checklist (WCAG AA, Section 508, European standards)

**Themes:**
- Four default themes: White, Gray 10 (light); Gray 90, Gray 100 (dark)
- Universal tokens for easy customization
- Role-based tokens with theme-specific values

**Form Inputs:**
- Three sizes: Large (48px), Medium (40px), Small (32px)
- Helper text below fields

**Accessibility:**
- WCAG AA compliance
- Section 508 and European standards
- Perceivable, operable, understandable patterns
- Screen reader compatible

**Color Accessibility:**
- Black text accessible on colors 10-50
- White text accessible on colors 60-100
- Color difference of 50+ meets WCAG AA contrast

---

## Core Design Principles

Based on analysis of all six design systems, the following principles should guide the Wayland compositor settings application:

### 1. Clarity and Simplicity
- Interfaces should be immediately understandable
- Strip away unnecessary complexity
- Focus on essential settings
- Use clear, descriptive labels

### 2. Consistency
- Follow platform conventions (Linux desktop expectations)
- Use familiar patterns and components
- Maintain visual and behavioral consistency throughout

### 3. Accessibility First
- Design for everyone, including users with disabilities
- Ensure keyboard navigation for all features
- Maintain proper contrast ratios
- Support screen readers
- Provide scalable text

### 4. Progressive Disclosure
- Show most important/frequent settings first
- Group related settings logically
- Use subscreens for complex categories
- Implement search for deep hierarchies

### 5. User Control & Flexibility
- "Simple by default, powerful when needed" (KDE principle)
- Don't hide advanced options, but organize them appropriately
- Allow customization where it matters
- Provide clear defaults with easy reset options

### 6. Visual Hierarchy
- Use spacing, typography, and color to create clear hierarchy
- Guide user attention to important elements
- Make current focus/selection obvious

### 7. Responsive & Adaptive
- Support different screen sizes and resolutions
- Adapt layout based on available space
- Consider touch and pointer input methods

---

## Settings Organization & Information Architecture

### Categorization Strategy

**For 7 or Fewer Settings:**
- Display all settings in a single view
- No grouping needed
- Use simple vertical layout

**For 8-15 Settings:**
- Group related settings under section headers
- Use spacing and typography to separate groups
- Keep all in one scrollable view

**For 16+ Settings:**
- Create category subscreens
- Implement sidebar navigation or tab system
- Add search functionality

### Recommended Layout Patterns

#### Option 1: Sidebar Navigation (Best for Desktop Wayland Compositor)

```
┌─────────────────────────────────────────┐
│ App Header / Title Bar                  │
├──────────┬──────────────────────────────┤
│          │                              │
│ General  │  General Settings            │
│ Display  │                              │
│ Input    │  ┌─────────────────────┐    │
│ Keyboard │  │ Setting Group       │    │
│ Windows  │  │ ○ Option 1          │    │
│ Effects  │  │ ○ Option 2          │    │
│          │  └─────────────────────┘    │
│          │                              │
│          │  ┌─────────────────────┐    │
│          │  │ Another Group       │    │
│          │  │ [Toggle] Feature    │    │
│          │  └─────────────────────┘    │
│          │                              │
└──────────┴──────────────────────────────┘
```

**Advantages:**
- Efficient use of space
- Clear categorization
- Easy navigation between categories
- Familiar to desktop users

**Specifications:**
- Sidebar width: 200-250px
- Highlight active category
- Support keyboard navigation (arrow keys)
- Allow collapsing sidebar for smaller screens

#### Option 2: Tabbed Interface

```
┌─────────────────────────────────────────┐
│ App Header                              │
├─────────────────────────────────────────┤
│ General | Display | Input | Windows ... │
├─────────────────────────────────────────┤
│                                         │
│  General Settings                       │
│                                         │
│  ┌─────────────────────┐               │
│  │ Setting Group       │               │
│  └─────────────────────┘               │
│                                         │
└─────────────────────────────────────────┘
```

**Advantages:**
- Compact horizontal use of space
- Clear category separation
- Familiar pattern

**Disadvantages:**
- Limited number of categories before crowding
- Less efficient vertical space usage

#### Option 3: Card-Based Layout

```
┌─────────────────────────────────────────┐
│ Settings                         [🔍]   │
├─────────────────────────────────────────┤
│                                         │
│  ┌───────────────┐  ┌───────────────┐  │
│  │ General       │  │ Display       │  │
│  │               │  │               │  │
│  │ Basic config  │  │ Monitors etc  │  │
│  └───────────────┘  └───────────────┘  │
│                                         │
│  ┌───────────────┐  ┌───────────────┐  │
│  │ Input         │  │ Windows       │  │
│  │               │  │               │  │
│  │ Mouse/touch   │  │ Management    │  │
│  └───────────────┘  └───────────────┘  │
│                                         │
└─────────────────────────────────────────┘
```

**Advantages:**
- Visual and modern
- Good for overview
- Works well with search

**Disadvantages:**
- Requires extra click to access settings
- Less space-efficient

### Recommended Approach for Wayland Compositor Settings

**Use Sidebar Navigation with the following structure:**

1. **General**
   - Application preferences
   - Startup behavior
   - Default applications

2. **Display**
   - Monitors and outputs
   - Resolution and scaling
   - Refresh rate
   - Color profiles

3. **Input**
   - Mouse and touchpad
   - Touch input
   - Tablets and styluses

4. **Keyboard**
   - Layout and variants
   - Shortcuts and hotkeys
   - Input methods

5. **Windows**
   - Window rules
   - Focus behavior
   - Tiling/floating settings
   - Animations

6. **Workspaces**
   - Number of workspaces
   - Layout and arrangement
   - Switching behavior

7. **Appearance**
   - Theme (light/dark/auto)
   - Window decorations
   - Cursor theme
   - Fonts

8. **Effects & Animations**
   - Enable/disable effects
   - Performance settings
   - Individual effect toggles

9. **Power Management**
   - Screen blanking
   - Suspend settings
   - Power button behavior

10. **Advanced**
    - Debug options
    - Experimental features
    - Backend settings

### Search Functionality

For applications with 15+ settings, implement search:

**Features:**
- Real-time filtering as user types
- Search across setting names and descriptions
- Highlight matched terms
- Show breadcrumb (category > setting)
- Support keyboard navigation (arrow keys, Enter)

**Placement:**
- Top-right corner of window
- Always visible
- Keyboard shortcut: Ctrl+F or Ctrl+K

---

## Component Guidelines

### Switches/Toggles

**Use When:**
- Binary on/off settings
- Immediate effect (no "Apply" button needed)
- Examples: "Enable animations", "Show desktop icons"

**Design Specifications:**
- Minimum target size: 48x48px (touch-friendly)
- Visual distinction between on/off states
- Color: Use accent color for "on" state
- Animation: Smooth transition (150-200ms)
- Label position: Left of switch, vertically centered

**Accessibility Requirements:**
- Keyboard: Space or Enter to toggle
- ARIA role: `switch`
- ARIA state: `aria-checked="true|false"`
- Focus indicator: 3:1 contrast ratio minimum
- Label association: `aria-labelledby` or wrapped label

**Visual Example:**
```
Enable window animations        [====○ ON ]
Show notification badges         [○==== OFF]
```

**Color Contrast:**
- On state: 4.5:1 against background
- Off state: 4.5:1 against background
- Focus indicator: 3:1 against adjacent colors

### Radio Buttons

**Use When:**
- Selecting one option from 2-7 choices
- All options should be visible simultaneously
- Examples: "Theme: Light / Dark / Auto"

**Design Specifications:**
- Minimum target size: 48x48px
- Vertical list preferred (easier to scan)
- Radio button on left, label on right
- Clear visual indication of selected state
- Group related options with spacing

**Accessibility:**
- Keyboard: Arrow keys to navigate, Space to select
- ARIA role: `radio` with `radiogroup` parent
- ARIA state: `aria-checked="true|false"`
- Group label: `aria-labelledby` on group

**Visual Example:**
```
Window focus mode:
  ○ Click to focus
  ● Focus follows mouse
  ○ Sloppy focus
```

### Checkboxes

**Use When:**
- Multiple independent options can be selected
- Each option is binary (on/off)
- Examples: "Enabled workspace features: [x] Animations [ ] Dynamic workspaces [x] Wrap around"

**Design Specifications:**
- Minimum target size: 48x48px
- Checkbox on left, label on right
- Allow for indeterminate state when needed
- Group related options

**Accessibility:**
- Keyboard: Space to toggle
- ARIA: Use native `<input type="checkbox">` or `role="checkbox"`
- ARIA state: `aria-checked="true|false|mixed"`

### Sliders

**Use When:**
- Adjusting a value within a range
- Visual feedback of value is important
- Examples: "Animation speed: 0.5x → 2.0x", "Opacity: 0% → 100%"

**Design Specifications:**
- Minimum height: 32px (for touch target)
- Show current value adjacent to slider
- Include min/max labels
- Snap to increments when appropriate
- Provide instant visual feedback

**Accessibility:**
- Keyboard: Arrow keys to adjust, Home/End for min/max
- ARIA role: `slider`
- ARIA properties: `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext`
- Focus indicator clearly visible

**Visual Example:**
```
Animation speed:  Slow [====|====----] Fast  (1.5x)
Window opacity:   0%   [========|--] 100%    (75%)
```

### Dropdown Menus / Combo Boxes

**Use When:**
- Selecting from 8+ options
- Space is limited
- Options can be categorized or searched
- Examples: "Default terminal: [Alacritty ▼]"

**Design Specifications:**
- Minimum height: 40px
- Show currently selected value
- Clear dropdown indicator (▼)
- Support keyboard navigation and search
- Highlight hovered/focused items

**Accessibility:**
- Keyboard: Arrow keys, type to search, Enter to select
- ARIA role: `combobox` with `listbox` popup
- ARIA state: `aria-expanded="true|false"`
- ARIA association: `aria-controls`, `aria-activedescendant`

### Text Inputs

**Use When:**
- Entering custom values (numbers, text, paths)
- Examples: "Terminal command: [________________]"

**Design Specifications:**
- Three sizes available:
  - Small: 32px height
  - Medium: 40px height (default)
  - Large: 48px height
- Helper text below field (not above)
- Clear label above or beside input
- Show validation errors below field
- Placeholder text for hints (don't rely on it exclusively)

**Accessibility:**
- Keyboard: Tab to navigate, type to input
- Label association: `<label for="id">` or `aria-labelledby`
- Error association: `aria-describedby` for helper text/errors
- Invalid state: `aria-invalid="true"`

### Number Inputs

**Use When:**
- Entering numeric values
- Increment/decrement buttons are useful

**Design Specifications:**
- Include +/- buttons or allow typing
- Show min/max constraints
- Display units when applicable
- Validate on blur or submit

**Visual Example:**
```
Border width:  [  2  ]  px
               [-][+]
```

### Buttons

**Use When:**
- Applying settings
- Resetting to defaults
- Opening dialogs or subscreens
- Performing actions

**Types & Hierarchy:**

1. **Primary Button** (one per view)
   - Most important action
   - Example: "Apply", "Save", "OK"
   - Style: Filled background, accent color

2. **Secondary Button**
   - Alternative actions
   - Example: "Cancel", "Reset to Defaults"
   - Style: Outlined or text-only

3. **Tertiary Button**
   - Less common actions
   - Example: "Advanced Settings..."
   - Style: Text-only

**Design Specifications:**
- Minimum height: 40px
- Minimum width: 88px
- Padding: 16px horizontal
- Clear, action-oriented labels
- Icon + text when helpful

**Accessibility:**
- Keyboard: Enter or Space to activate
- Native `<button>` element or `role="button"`
- Focus indicator: 3:1 contrast
- Disabled state: `disabled` attribute or `aria-disabled="true"`

### List Items

**Use When:**
- Displaying multiple similar settings
- Settings that can be added/removed
- Examples: Keyboard shortcuts, window rules, workspace names

**Design Specifications:**
- Minimum height: 56px for touch-friendly lists
- Icon/avatar on left (if applicable)
- Primary text and secondary text
- Action buttons on right (edit, delete)
- Hover state for entire item
- Selection state when applicable

**Accessibility:**
- Keyboard: Arrow keys to navigate, Enter to activate/edit
- ARIA role: `listbox` with `option` children, or `list` with `listitem`
- ARIA state: `aria-selected` when applicable

---

## Spacing & Layout System

### Base Unit System

Based on analysis of all design systems, use a **6px base unit** with common multiples:

- **4px**: Material Design minimum (half-unit)
- **6px**: GNOME recommended base
- **8px**: Common standard (Material Design primary)
- **12px**: GNOME label spacing, KDE indentation
- **16px**: Fluent padding
- **18px**: GNOME vertical groups, dialog padding
- **24px**: Section spacing
- **32px**: Major section divisions
- **48px**: Minimum touch target size

### Spacing Scale

Define a consistent spacing scale:

```
space-1:  4px   (tight spacing, internal padding)
space-2:  8px   (small gaps)
space-3:  12px  (label-to-control, indentation)
space-4:  16px  (default padding)
space-5:  18px  (GNOME-style group spacing)
space-6:  24px  (section headers)
space-8:  32px  (major sections)
space-12: 48px  (page-level spacing)
```

### Specific Spacing Guidelines

#### Dialog/Window Padding
- **18px** padding between window content and borders (GNOME standard)
- **16px** acceptable alternative (Fluent standard)

#### Component Spacing

**Label to Control:**
- Horizontal: **12px** between label and associated control
- Vertical: **8px** between label above and control below

**Between Controls:**
- Related controls: **12px** vertical spacing
- Unrelated controls: **18px** vertical spacing

**Section Groups:**
- Between groups: **24px** vertical spacing
- Section header to content: **12px**

**Indentation:**
- Nested/dependent settings: **12px** left indent

#### Alignment

**Text Alignment:**
- Left-aligned for LTR languages (English)
- Right-aligned labels acceptable when avoiding large gaps
- Avoid justified text (scanning difficulty)

**Control Alignment:**
- Align all controls vertically along left edge
- Align labels vertically when possible
- Exact alignment critical (eye is sensitive to misalignment)

### Layout Grid

**Desktop Application:**
- Use flexible grid system
- Sidebar: 200-250px fixed width
- Content area: Minimum 400px, scales with window
- Maximum content width: 800px (readability)
- Margins: 24px on content sides

**Responsive Behavior:**

**Compact (< 600px width):**
- Hide/collapse sidebar
- Stack elements vertically
- Full-width components
- Larger touch targets

**Medium (600-1024px width):**
- Optional sidebar visibility
- Two-column layouts where appropriate

**Expanded (> 1024px width):**
- Always show sidebar
- Multi-column layouts for dense information
- Maintain maximum content width for readability

### Touch Targets

**Minimum Interactive Element Size:**
- **48x48px** for all interactive elements (WCAG 2.5.8 Level AA)
- Applies to: buttons, switches, checkboxes, radio buttons, sliders
- Internal visible element can be smaller, but clickable/tappable area must be 48x48px

**Spacing Between Targets:**
- Minimum **8px** spacing between adjacent interactive elements
- Reduces accidental activation

---

## Typography

### Font Selection

**For Linux/Wayland Compositor Settings:**

**Option 1: System Default**
- Use system's default sans-serif font
- Ensures consistency with desktop environment
- Respects user's font choices

**Option 2: Specific Recommendations**
- **GNOME/GTK:** Adwaita Sans (new 2025 default, based on Inter)
- **KDE/Qt:** System default or Inter/Roboto
- **Fallback:** Sans-serif, Arial, Liberation Sans

**Monospace Font:**
- For code, paths, commands
- **GNOME:** Adwaita Mono (based on Iosevka)
- **KDE/General:** Cascadia Code, Fira Code, JetBrains Mono
- **Fallback:** Monospace, Courier New

### Type Scale

Define clear typographic hierarchy:

```
Heading 1 (Page Title):
  - Size: 24px
  - Weight: 600 (Semi-bold)
  - Line height: 32px
  - Use: Main settings page title

Heading 2 (Section Header):
  - Size: 20px
  - Weight: 600
  - Line height: 28px
  - Use: Major section dividers

Heading 3 (Subsection):
  - Size: 16px
  - Weight: 600
  - Line height: 24px
  - Use: Setting groups

Body (Default):
  - Size: 14px
  - Weight: 400 (Regular)
  - Line height: 20px
  - Use: Setting labels, descriptions

Body Small (Helper Text):
  - Size: 12px
  - Weight: 400
  - Line height: 16px
  - Use: Helper text, hints, secondary info

Caption:
  - Size: 11px
  - Weight: 400
  - Line height: 16px
  - Use: Metadata, footnotes
```

### Font Weight

- **Regular (400)**: Body text, labels
- **Medium (500)**: Optional emphasis
- **Semi-bold (600)**: Headings, important labels
- **Bold (700)**: Strong emphasis (use sparingly)

### Line Height

- Body text: 1.4-1.6x font size
- Headings: 1.2-1.4x font size
- Tight spaces: 1.2x minimum
- Never less than 1.2x for accessibility

### Text Styling

**Emphasis:**
- Use weight changes (medium/semi-bold) over italics
- Italics acceptable for very brief emphasis or citations
- Avoid all-caps except for short acronyms

**Color:**
- Primary text: High contrast (see Color section)
- Secondary text: Medium emphasis (60-87% opacity)
- Disabled text: Low emphasis (38% opacity)

**Line Length:**
- Optimal: 50-75 characters per line
- Maximum: 90 characters
- Improves readability and scanning

### Accessibility

**Minimum Font Size:**
- **11pt** (Apple HIG standard)
- **12px** for body text recommended
- Support Dynamic Type / font scaling

**Contrast:**
- Normal text (< 18px or < 14px bold): **4.5:1** minimum (WCAG AA)
- Large text (≥ 18px or ≥ 14px bold): **3:1** minimum (WCAG AA)

---

## Color & Theming

### Color System Architecture

Implement a two-layer token system (inspired by Fluent):

**Layer 1: Global Tokens (Raw Values)**
```
gray-0:   #000000 (pure black)
gray-10:  #1C1C1C (near black)
gray-20:  #2D2D2D
gray-30:  #3F3F3F
gray-40:  #525252
gray-50:  #6B6B6B
gray-60:  #8A8A8A
gray-70:  #A8A8A8
gray-80:  #C6C6C6
gray-90:  #E0E0E0
gray-95:  #F0F0F0
gray-100: #FFFFFF (pure white)

blue-40:  #1E40AF
blue-50:  #3B82F6 (primary/accent)
blue-60:  #60A5FA
```

**Layer 2: Semantic Tokens (Role-Based)**
```
background:          (theme-dependent)
surface:             (theme-dependent)
surface-variant:     (theme-dependent)
primary:             blue-50
on-background:       (theme-dependent)
on-surface:          (theme-dependent)
on-primary:          white / black
error:               red-50
warning:             yellow-60
success:             green-60
```

### Theme Implementation

#### Light Theme (Default)
```
background:          gray-100 (#FFFFFF)
surface:             gray-95  (#F0F0F0)
surface-variant:     gray-90  (#E0E0E0)
primary:             blue-50
on-background:       gray-10  (87% opacity for primary text)
on-surface:          gray-10  (87% opacity for primary text)
on-surface-variant:  gray-30  (60% opacity for secondary text)
disabled:            gray-30  (38% opacity)
border:              gray-80
```

#### Dark Theme
```
background:          gray-10  (#1C1C1C) - NOT pure black
surface:             gray-20  (#2D2D2D)
surface-variant:     gray-30  (#3F3F3F)
primary:             blue-60  (lighter variant for dark bg)
on-background:       gray-100 (87% white)
on-surface:          gray-100 (87% white)
on-surface-variant:  gray-80  (60% white)
disabled:            gray-70  (38% white)
border:              gray-60
```

**Why #1C1C1C Instead of #000000?**
- Reduces eye strain (pure black too harsh)
- Better for OLED power savings vs usability trade-off
- Allows for elevation/shadow effects
- Google Material Design recommendation

### Dark Theme Best Practices (2025)

1. **Background Color:**
   - Use #121212 (Material Design) or #1C1C1C
   - Never pure black (#000000)

2. **Text Colors:**
   - Avoid pure white (#FFFFFF)
   - Use off-white or light gray
   - Material recommended opacities:
     - High emphasis: 87% white
     - Medium emphasis: 60% white
     - Disabled: 38% white

3. **Color Saturation:**
   - Desaturate colors in dark mode
   - High-contrast combinations for readability
   - Avoid overly saturated colors (eye strain)

4. **Elevation/Depth:**
   - Lighter surfaces for higher elevation
   - Subtle differences (don't overdo)
   - Example: surface-1: +5% lighter, surface-2: +10% lighter

### Contrast Requirements

**WCAG 2.2 Standards:**

**Text Contrast:**
- Normal text: **4.5:1** minimum (AA), **7:1** enhanced (AAA)
- Large text (18px+ or 14px+ bold): **3:1** minimum (AA), **4.5:1** enhanced (AAA)

**UI Components (WCAG 1.4.11):**
- Buttons, inputs, borders: **3:1** minimum against adjacent colors
- Focus indicators: **3:1** minimum against both focused and unfocused states

**Material Design Dark Theme:**
- Minimum **15.8:1** contrast between text and background
- Ensures body text passes 4.5:1 at highest elevation

### Color Calculation Tool

To verify contrast ratios:
- Use online tools: WebAIM Contrast Checker, Coolors Contrast Checker
- In Carbon: Color difference of 50+ between values = accessible
- Test both light and dark themes

### Accent/Primary Color

**Guidelines:**
- Should reflect compositor branding
- Must meet 4.5:1 contrast on both light and dark backgrounds
- Adjust saturation/lightness for dark theme
- Use for:
  - Primary buttons
  - Switch "on" state
  - Selected items
  - Links
  - Focus indicators

**Example:**
- Light theme primary: blue-50 (#3B82F6)
- Dark theme primary: blue-60 (#60A5FA) - lighter variant

### Semantic Colors

**Error:** Red
- Light theme: red-50 (#EF4444)
- Dark theme: red-60 (#F87171)
- Use for: Error messages, destructive actions

**Warning:** Yellow/Orange
- Light theme: yellow-60 (#F59E0B)
- Dark theme: yellow-70 (#FCD34D)
- Use for: Warnings, caution messages

**Success:** Green
- Light theme: green-50 (#10B981)
- Dark theme: green-60 (#34D399)
- Use for: Success messages, confirmations

**Info:** Blue (can match primary)
- Use for: Informational messages, tips

### Theme Switching

**Options to Provide:**
1. Light
2. Dark
3. Auto (follow system preference)

**Implementation:**
- Respect `prefers-color-scheme` media query
- Save user preference locally
- Smooth transition between themes (200-300ms)
- Reload/persist theme on app restart

---

## Accessibility Requirements

### Keyboard Navigation

**Essential Requirements (WCAG 2.1.1):**

1. **All Features Keyboard Accessible:**
   - Every interactive element operable with keyboard only
   - No mouse-only features

2. **Standard Keyboard Controls:**
   - **Tab / Shift+Tab:** Navigate between interactive elements
   - **Arrow keys:** Navigate within groups (radio buttons, lists, dropdowns, sliders)
   - **Enter / Return:** Activate buttons, links, submit
   - **Space:** Toggle checkboxes, switches, activate buttons
   - **Home / End:** Jump to first/last item in list or slider min/max
   - **Escape:** Close dialogs, cancel operations, exit menus

3. **Tab Order:**
   - Logical and intuitive (top-to-bottom, left-to-right)
   - Match visual layout
   - Skip to main content option for complex layouts

4. **Focus Management:**
   - Focus visible at all times (WCAG 2.4.7)
   - **3:1** contrast ratio between focused and unfocused states
   - Clear, distinctive focus indicator (outline, border change, background change)
   - Focus doesn't get trapped (keyboard users can navigate away)

**Recommended Focus Styles:**
```
Outline: 2px solid accent-color
Offset: 2px from element
Alternative: 3px solid semi-transparent accent color
```

### Screen Reader Support

**ARIA Roles and Properties:**

1. **Switches:**
   - Role: `switch`
   - State: `aria-checked="true|false"`
   - Label: `aria-labelledby` or `aria-label`

2. **Checkboxes:**
   - Native `<input type="checkbox">` preferred
   - If custom: role `checkbox`, state `aria-checked="true|false|mixed"`

3. **Radio Buttons:**
   - Native `<input type="radio">` preferred
   - If custom: role `radio`, grouping with `radiogroup`, state `aria-checked`

4. **Sliders:**
   - Role: `slider`
   - Properties: `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, `aria-valuetext`
   - Optionally: `aria-orientation="horizontal|vertical"`

5. **Buttons:**
   - Native `<button>` preferred
   - If custom: role `button`
   - Disabled: `disabled` attribute or `aria-disabled="true"`

6. **Text Inputs:**
   - Associated label: `<label for="id">` or `aria-labelledby`
   - Helper text/errors: `aria-describedby`
   - Invalid state: `aria-invalid="true"`

7. **Combobox/Dropdown:**
   - Role: `combobox`
   - Popup: role `listbox` with `option` children
   - State: `aria-expanded="true|false"`
   - Association: `aria-controls`, `aria-activedescendant`

8. **Lists:**
   - Role: `list` with `listitem`, or `listbox` with `option`
   - Selection state: `aria-selected="true|false"` when applicable

**Naming Requirements:**
- Every interactive element has accessible name
- Avoid relying solely on placeholders or tooltips
- Meaningful, descriptive labels
- Context provided for icons without text

### Visual Accessibility

**Text Contrast:**
- Normal text: **4.5:1** minimum (WCAG AA)
- Large text: **3:1** minimum (WCAG AA)
- Enhanced contrast: **7:1** (WCAG AAA)

**Component Contrast:**
- UI components: **3:1** minimum (WCAG 1.4.11)
- Focus indicators: **3:1** minimum

**Color Independence:**
- Don't rely on color alone to convey information
- Use icons, labels, patterns in addition to color
- Example: Error state should have red color + icon + text, not just red border

**Text Sizing:**
- Support text scaling up to 200% without breaking layout
- Minimum 11pt font size (Apple HIG)
- No maximum line length enforced by hard wrapping

**Motion & Animation:**
- Respect `prefers-reduced-motion` media query
- Disable animations when user preference set
- Provide static alternatives for loading spinners
- Avoid blinking/flashing content (seizure risk)

### Touch Accessibility

**Target Sizes (WCAG 2.5.8):**
- Minimum **48x48px** for all interactive elements (Level AA)
- Spacing: Minimum **8px** between adjacent targets

**Touch Feedback:**
- Visual indication on touch (press state)
- Sufficient time for double-tap if used
- No reliance on hover states (not available on touch)

### Testing Checklist

**Keyboard Testing:**
1. Unplug mouse, test all features with keyboard only
2. Verify focus indicator visible at all times
3. Check tab order is logical
4. Ensure no keyboard traps
5. Test all shortcuts

**Screen Reader Testing:**
1. Turn off screen and use Orca (Linux) or NVDA (Windows)
2. Verify all elements have intelligible labels
3. Check state changes are announced (toggle on/off, selection)
4. Ensure navigation is logical

**Visual Testing:**
1. Check all text meets 4.5:1 contrast (normal) or 3:1 (large)
2. Verify UI components meet 3:1 contrast
3. Test color-blind modes (protanopia, deuteranopia, tritanopia)
4. Increase font size to 200%, verify no content cut off
5. Check dark theme meets same contrast requirements

**Motion Testing:**
1. Enable reduced motion system setting
2. Verify animations disabled or reduced
3. Check static alternatives for spinners/loaders

---

## Best Practices Summary

### Settings Organization

1. **Group Logically:**
   - Organize by user mental models, not technical architecture
   - Use clear, descriptive category names
   - 7 or fewer: no grouping; 8-15: section headers; 16+: subscreens/sidebar

2. **Prioritize:**
   - Most frequent/important settings first
   - Advanced/rarely used settings in dedicated sections
   - Provide search for 15+ settings

3. **Progressive Disclosure:**
   - Don't overwhelm users with all options at once
   - Use expandable sections for related settings
   - Clearly indicate when settings have sub-options

### Component Selection

1. **Switches for Binary Choices:**
   - Immediate effect, no apply button
   - Clear on/off states with color and position

2. **Radio Buttons for Exclusive Choices:**
   - 2-7 visible options
   - Vertical layout preferred

3. **Checkboxes for Multiple Selections:**
   - Independent options
   - Can select 0, 1, or many

4. **Sliders for Ranges:**
   - Continuous or stepped values
   - Show current value clearly
   - Min/max labels

5. **Dropdowns for Many Options:**
   - 8+ choices
   - Space-constrained
   - Support search/filter for long lists

### Layout & Spacing

1. **Use Consistent Spacing Scale:**
   - Base unit: 6-8px
   - Common values: 12px, 16px, 18px, 24px, 32px, 48px

2. **Respect Touch Targets:**
   - Minimum 48x48px for all interactive elements
   - 8px minimum between targets

3. **Create Clear Hierarchy:**
   - Use spacing to group related items
   - Larger gaps between unrelated sections
   - Consistent indentation (12px) for nested items

4. **Alignment Matters:**
   - Align controls precisely
   - Vertical alignment of labels and controls
   - Left-align text for LTR languages

### Typography

1. **Clear Hierarchy:**
   - Page title (24px), section headers (20px), subsections (16px), body (14px)
   - Use weight changes for emphasis

2. **Readability:**
   - Line height 1.4-1.6x for body text
   - Line length 50-75 characters optimal
   - Minimum 11pt font size

3. **Accessibility:**
   - 4.5:1 contrast for normal text
   - Support text scaling to 200%
   - Avoid justified text

### Color & Theming

1. **Dark Theme Support:**
   - Use #121212 or #1C1C1C, not pure black
   - Desaturate colors for dark mode
   - 87% white for high-emphasis text, 60% for medium, 38% for disabled

2. **Contrast Compliance:**
   - Text: 4.5:1 minimum (normal), 3:1 (large)
   - Components: 3:1 minimum
   - Test both themes

3. **Semantic Colors:**
   - Error (red), warning (yellow), success (green), info (blue)
   - Adjust lightness for dark theme
   - Don't rely on color alone

4. **Theme Options:**
   - Light, Dark, Auto (follow system)
   - Smooth transitions between themes
   - Persist user choice

### Accessibility

1. **Keyboard First:**
   - All features accessible via keyboard
   - Logical tab order
   - Clear focus indicators (3:1 contrast)

2. **Screen Reader Ready:**
   - Proper ARIA roles and states
   - Associated labels for all inputs
   - Meaningful element names

3. **Visual Accessibility:**
   - Sufficient contrast ratios
   - Don't rely on color alone
   - Support text scaling

4. **Motion Sensitivity:**
   - Respect `prefers-reduced-motion`
   - Disable animations when requested
   - Provide static alternatives

### User Experience

1. **Clear Labels:**
   - Descriptive, action-oriented
   - Avoid jargon
   - Explain impact of settings

2. **Helpful Descriptions:**
   - Brief helper text for complex settings
   - Below controls, not above
   - Provide examples when useful

3. **Defaults & Reset:**
   - Sensible defaults
   - Clear indication of default values
   - Easy "Reset to Defaults" option

4. **Immediate Feedback:**
   - Instant updates for switches/toggles when possible
   - Clear confirmation for destructive actions
   - Loading states for slow operations

5. **Error Prevention:**
   - Validate inputs
   - Provide clear error messages
   - Guide users to correct issues

---

## Implementation Checklist

### Phase 1: Foundation

- [ ] Define color tokens (global and semantic)
- [ ] Implement light and dark themes
- [ ] Set up spacing scale (4px, 8px, 12px, 16px, 18px, 24px, 32px, 48px)
- [ ] Define typography scale (24px, 20px, 16px, 14px, 12px, 11px)
- [ ] Choose and integrate fonts (Adwaita Sans or system default)
- [ ] Set up theme switching (light/dark/auto)
- [ ] Implement `prefers-color-scheme` detection
- [ ] Create base layout structure (sidebar + content area)

### Phase 2: Components

- [ ] Build accessible Switch component
  - [ ] 48x48px target size
  - [ ] Keyboard support (Space/Enter)
  - [ ] ARIA role and state
  - [ ] Focus indicator (3:1 contrast)
  - [ ] Smooth animation (150-200ms)

- [ ] Build accessible Radio Button component
  - [ ] 48x48px target size
  - [ ] Arrow key navigation
  - [ ] ARIA role and state
  - [ ] Vertical layout default

- [ ] Build accessible Checkbox component
  - [ ] 48x48px target size
  - [ ] Space to toggle
  - [ ] Support indeterminate state

- [ ] Build accessible Slider component
  - [ ] Arrow keys to adjust
  - [ ] Home/End for min/max
  - [ ] ARIA attributes
  - [ ] Value display

- [ ] Build accessible Dropdown/Combobox component
  - [ ] Keyboard navigation
  - [ ] Type-to-search
  - [ ] ARIA combobox pattern

- [ ] Build Text Input component
  - [ ] Label association
  - [ ] Helper text support
  - [ ] Error state styling
  - [ ] Three sizes (32px, 40px, 48px)

- [ ] Build Button components
  - [ ] Primary, secondary, tertiary variants
  - [ ] Disabled state
  - [ ] Focus indicator
  - [ ] Min 40px height, 88px width

### Phase 3: Settings Structure

- [ ] Implement sidebar navigation
  - [ ] Keyboard navigation (arrow keys)
  - [ ] Active state highlighting
  - [ ] Collapse for small screens

- [ ] Define category structure:
  - [ ] General
  - [ ] Display
  - [ ] Input
  - [ ] Keyboard
  - [ ] Windows
  - [ ] Workspaces
  - [ ] Appearance
  - [ ] Effects & Animations
  - [ ] Power Management
  - [ ] Advanced

- [ ] Implement search functionality
  - [ ] Real-time filtering
  - [ ] Keyboard shortcut (Ctrl+F or Ctrl+K)
  - [ ] Highlight matches
  - [ ] Show breadcrumbs

### Phase 4: Accessibility

- [ ] Test keyboard navigation
  - [ ] Tab order logical
  - [ ] All features accessible
  - [ ] No keyboard traps
  - [ ] Focus always visible

- [ ] Test screen reader compatibility
  - [ ] All elements labeled
  - [ ] State changes announced
  - [ ] Navigation logical

- [ ] Verify contrast ratios
  - [ ] Text: 4.5:1 (normal), 3:1 (large)
  - [ ] Components: 3:1
  - [ ] Focus indicators: 3:1
  - [ ] Both light and dark themes

- [ ] Test touch targets
  - [ ] All interactive elements 48x48px minimum
  - [ ] 8px spacing between targets

- [ ] Implement motion preferences
  - [ ] Respect `prefers-reduced-motion`
  - [ ] Disable/reduce animations accordingly
  - [ ] Static alternatives for loaders

### Phase 5: Polish

- [ ] Add loading states for slow operations
- [ ] Implement error handling and validation
- [ ] Add helpful descriptions for complex settings
- [ ] Implement "Reset to Defaults" functionality
- [ ] Test responsive behavior (compact, medium, expanded)
- [ ] Test with increased text size (200% zoom)
- [ ] Implement smooth theme transitions
- [ ] Add confirmation dialogs for destructive actions
- [ ] Test color-blind modes
- [ ] Perform final accessibility audit

### Phase 6: Documentation

- [ ] Document component usage
- [ ] Create style guide
- [ ] Write accessibility guidelines for contributors
- [ ] Document keyboard shortcuts
- [ ] Create user documentation for settings

---

## Additional Resources

### Design Systems Documentation

**Material Design 3:**
- Official Site: https://m3.material.io/
- Components: https://m3.material.io/components
- Foundations: https://m3.material.io/foundations

**Apple Human Interface Guidelines:**
- Main HIG: https://developer.apple.com/design/human-interface-guidelines/
- macOS Guidelines: https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- Accessibility: https://developer.apple.com/design/human-interface-guidelines/accessibility

**Microsoft Fluent Design System:**
- Fluent 2: https://fluent2.microsoft.design/
- Design Principles: https://fluent2.microsoft.design/design-principles
- Components: https://fluent2.microsoft.design/components

**GNOME Human Interface Guidelines:**
- Main HIG: https://developer.gnome.org/hig/
- Patterns: https://developer.gnome.org/hig/patterns
- Accessibility: https://developer.gnome.org/hig/accessibility

**KDE Human Interface Guidelines:**
- Main HIG: https://develop.kde.org/hig/
- Design Philosophy: https://develop.kde.org/hig/kde_app_design/

**IBM Carbon Design System:**
- Main Site: https://carbondesignsystem.com/
- Components: https://carbondesignsystem.com/components/overview/
- Accessibility: https://carbondesignsystem.com/guidelines/accessibility/overview/

### WCAG Guidelines

- WCAG 2.2 Overview: https://www.w3.org/WAI/WCAG22/quickref/
- ARIA Authoring Practices: https://www.w3.org/WAI/ARIA/apg/
- WebAIM Contrast Checker: https://webaim.org/resources/contrastchecker/

### Tools

**Color & Contrast:**
- WebAIM Contrast Checker: https://webaim.org/resources/contrastchecker/
- Coolors Contrast Checker: https://coolors.co/contrast-checker
- Material Color Tool: https://material.io/resources/color/

**Accessibility Testing:**
- axe DevTools (browser extension)
- WAVE (Web Accessibility Evaluation Tool)
- Orca Screen Reader (Linux)

**Design Tools:**
- Figma (Material 3, Apple, Fluent libraries available)
- Material Theme Builder
- Carbon Design Kit

---

## Conclusion

This guide synthesizes best practices from six leading design systems to provide comprehensive, actionable recommendations for building a Wayland compositor settings application. The key principles to remember:

1. **Accessibility is not optional** - Design for everyone from the start
2. **Consistency builds trust** - Use familiar patterns and maintain visual consistency
3. **Clarity over complexity** - Simple by default, powerful when needed
4. **Test with real users** - Including those using assistive technologies
5. **Iterate based on feedback** - Design systems evolve; so should your application

By following these guidelines, you'll create a settings application that is:
- Intuitive and easy to navigate
- Accessible to all users
- Visually consistent and polished
- Scalable and maintainable
- Aligned with modern Linux desktop expectations

Good luck with your Wayland compositor settings application!

---

## Document Metadata

**Version:** 1.0
**Last Updated:** 2025-12-05
**Author:** UX Design Research Compilation
**Based on Research of:** Material Design 3, Apple HIG, Microsoft Fluent, GNOME HIG, KDE HIG, IBM Carbon Design System
