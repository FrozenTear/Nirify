# Niri DE Design System

**Codename**: Phosphor
**Aesthetic Direction**: Terminal Elegance — the efficiency and honesty of terminal interfaces elevated with deliberate refinement and warmth.

---

## Design Philosophy

### Core Principles

1. **Information Density with Breathing Room**
   Pack useful information tightly, but use precise spacing to prevent claustrophobia. Every pixel of padding is intentional.

2. **Depth Through Layering, Not Decoration**
   Create hierarchy with subtle elevation and transparency, not borders or dividers. Surfaces float above surfaces.

3. **Typography as Interface**
   Text isn't decoration—it's the primary interaction medium. Make it readable, scannable, and beautiful.

4. **Warmth in the Machine**
   Catppuccin's warmth should feel like a cozy terminal session at 2am, not cold productivity software.

5. **Motion with Purpose**
   Animations communicate state changes, not showcase effects. Fast, precise, informative.

---

## Color System

### Catppuccin Mocha Palette (Canonical)

```
Background Layers
─────────────────────────────────────────────────
crust       #11111b   ████  Deepest background, panel base
mantle      #181825   ████  Recessed areas, input fields
base        #1e1e2e   ████  Primary surface, windows
surface0    #313244   ████  Elevated cards, hover states
surface1    #45475a   ████  Active selections, pressed
surface2    #585b70   ████  Borders, subtle dividers

Text Hierarchy
─────────────────────────────────────────────────
text        #cdd6f4   ████  Primary text, headings
subtext1    #bac2de   ████  Secondary text, labels
subtext0    #a6adc8   ████  Tertiary, timestamps, hints
overlay2    #9399b2   ████  Disabled text, placeholders
overlay1    #7f849c   ████  Very subtle text
overlay0    #6c7086   ████  Barely visible hints

Accent Spectrum
─────────────────────────────────────────────────
rosewater   #f5e0dc   ████  Soft highlight, selection bg
flamingo    #f2cdcd   ████  Warm notifications
pink        #f5c2e7   ████  Playful accents
mauve       #cba6f7   ████  PRIMARY ACCENT - focused elements
red         #f38ba8   ████  Errors, destructive actions
maroon      #eba0ac   ████  Warning states
peach       #fab387   ████  Attention, badges
yellow      #f9e2af   ████  Caution, pending states
green       #a6e3a1   ████  Success, positive actions
teal        #94e2d5   ████  Links, interactive elements
sky         #89dceb   ████  Information, neutral actions
sapphire    #74c7ec   ████  Cool accent alternative
blue        #89b4fa   ████  Secondary accent, selections
lavender    #b4befe   ████  Soft focus rings, gentle emphasis
```

### Semantic Color Mapping

```
┌─────────────────┬────────────┬─────────────────────────────────┐
│ Token           │ Color      │ Usage                           │
├─────────────────┼────────────┼─────────────────────────────────┤
│ bg-deep         │ crust      │ Panel, launcher backdrop        │
│ bg-base         │ base       │ Window backgrounds              │
│ bg-elevated     │ surface0   │ Cards, dropdowns, popovers      │
│ bg-active       │ surface1   │ Selected items, pressed states  │
│ bg-input        │ mantle     │ Text inputs, search fields      │
│                 │            │                                 │
│ text-primary    │ text       │ Headings, important content     │
│ text-secondary  │ subtext1   │ Body text, descriptions         │
│ text-muted      │ subtext0   │ Hints, timestamps, metadata     │
│ text-disabled   │ overlay1   │ Disabled controls               │
│                 │            │                                 │
│ accent-primary  │ mauve      │ Focus rings, primary buttons    │
│ accent-success  │ green      │ Confirmations, online status    │
│ accent-warning  │ peach      │ Warnings, pending states        │
│ accent-error    │ red        │ Errors, destructive actions     │
│ accent-info     │ sapphire   │ Information, neutral notices    │
│                 │            │                                 │
│ border-subtle   │ surface0   │ Dividers when needed            │
│ border-visible  │ surface1   │ Input borders, card edges       │
│ border-focus    │ mauve      │ Focused element rings           │
└─────────────────┴────────────┴─────────────────────────────────┘
```

### The Phosphor Glow

A signature effect: subtle glow on interactive elements suggesting warmth emanating from the screen.

```
Glow Specifications
─────────────────────────────────────────────────
Subtle:   0 0 8px  rgba(203, 166, 247, 0.15)   // Hover states
Medium:   0 0 12px rgba(203, 166, 247, 0.25)   // Focus states
Strong:   0 0 20px rgba(203, 166, 247, 0.35)   // Active/pressed

Applied sparingly:
- Search field when focused
- Active workspace indicator
- Primary action buttons
- Toggle switches when ON
```

---

## Typography

### Font Stack

**Display & UI**: JetBrains Mono
A technical choice that feels intentional, not default. Excellent legibility at small sizes, distinctive character.

**Fallback**: IBM Plex Mono → SF Mono → Consolas → monospace

```
Why Monospace Everywhere?
─────────────────────────────────────────────────
1. Honest - acknowledges the technical audience
2. Scannable - aligned columns, predictable widths
3. Distinctive - most DEs use proportional fonts
4. Practical - code snippets, paths, commands blend naturally
```

### Type Scale

Based on a 1.2 ratio (minor third) from 14px base:

```
Token          Size    Weight    Line Height    Usage
─────────────────────────────────────────────────────────────────
text-xs        10px    400       1.4            Timestamps, badges
text-sm        12px    400       1.4            Secondary labels, hints
text-base      14px    400       1.5            Body text, default
text-lg        16px    500       1.4            Section headers
text-xl        20px    600       1.3            Page titles
text-2xl       24px    600       1.2            Hero text (rare)

Letter Spacing
─────────────────────────────────────────────────────────────────
text-xs/sm     +0.5px  (open up for legibility)
text-base      +0      (natural)
text-lg+       -0.5px  (tighten for display)
```

### Text Treatments

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  SECTION HEADER                          ← text-xs, subtext0    │
│  ─────────────────────────────              uppercase, +1px     │
│                                             letter-spacing      │
│  Primary Setting Label                   ← text-base, text      │
│  Secondary description text that         ← text-sm, subtext1    │
│  explains what this setting does.                               │
│                                                                 │
│  Shortcut                          ⌘K    ← text-sm, subtext0    │
│                                             right-aligned       │
└─────────────────────────────────────────────────────────────────┘
```

---

## Spacing System

### Base Unit: 4px

All spacing derives from multiples of 4px for consistent rhythm.

```
Token     Pixels    Usage
───────────────────────────────────────────────────
space-0   0px       Collapse spacing
space-1   4px       Tight: icon-to-label gaps
space-2   8px       Compact: related items, input padding
space-3   12px      Default: list item padding, card padding
space-4   16px      Comfortable: section gaps
space-5   20px      Relaxed: major section dividers
space-6   24px      Spacious: page margins
space-8   32px      Dramatic: hero spacing
space-12  48px      Expansive: major layout gaps
```

### Component Spacing Patterns

```
Panel (32px height)
─────────────────────────────────────────────────
┌──────────────────────────────────────────────────────────────┐
│ 8px │ CONTENT │ 12px gap │ CONTENT │ 12px gap │ CONTENT │ 8px│
│     │         │          │         │          │         │    │
│  ↑  │         │          │         │          │         │    │
│ 8px vertical padding (centers content in 32px)              │
└──────────────────────────────────────────────────────────────┘

Settings List
─────────────────────────────────────────────────
┌─────────────────────────────────────────────────┐
│ 16px padding                                    │
│ ┌─────────────────────────────────────────────┐ │
│ │ 12px │ Icon │ 12px │ Label ──────── │ 12px │ │ ← 48px row
│ └─────────────────────────────────────────────┘ │
│ 4px gap                                         │
│ ┌─────────────────────────────────────────────┐ │
│ │ 12px │ Icon │ 12px │ Label ──────── │ 12px │ │
│ └─────────────────────────────────────────────┘ │
│ 16px padding                                    │
└─────────────────────────────────────────────────┘

Launcher
─────────────────────────────────────────────────
┌─────────────────────────────────────────────────┐
│ 16px                                            │
│ ┌─────────────────────────────────────────────┐ │
│ │  🔍  │ 12px │ Search...                     │ │ ← 48px
│ └─────────────────────────────────────────────┘ │
│ 12px                                            │
│ ┌─────────────────────────────────────────────┐ │
│ │ Icon │ 12px │ Firefox ──────────── │ badge │ │ ← 44px
│ └─────────────────────────────────────────────┘ │
│ 4px                                             │
│ ┌─────────────────────────────────────────────┐ │
│ │ Icon │ 12px │ Files ───────────────         │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

---

## Component Specifications

### Panel

**Dimensions**: 32px height, full width, anchored to top

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ◆ 1  ◇ 2  ◇ 3  ◇ 4  │           Tue Jan 21  14:32           │  ⚡ 🔊 ⚙  │
└──────────────────────────────────────────────────────────────────────────┘
   └─ Workspaces ─┘      └─────── Clock (center) ───────┘    └─ Systray ─┘

Background: crust (#11111b) with 95% opacity
            Subtle blur if compositor supports (optional)
```

**Workspace Indicators**:
```
States
─────────────────────────────────────────────────
Empty:      ◇  (outline only, overlay0)
Occupied:   ◆  (filled, subtext0)
Active:     ◆  (filled, mauve, with subtle glow)
Urgent:     ◆  (filled, peach, pulsing glow)

Hover:      Scale 1.1x, brighten color
Transition: 150ms ease-out
```

**Clock**:
```
Format:     "Tue Jan 21  14:32"
Style:      text-sm, text color
Spacing:    Double space between date and time for visual grouping
Hover:      Show tooltip with full date/time + calendar preview
```

**Systray Icons**:
```
Size:       18x18px icons
Gap:        8px between icons
Hover:      Surface0 circular background (24px diameter)
            Icon brightens to full white
```

### Launcher

**Dimensions**: 560px wide, dynamic height (max 480px)

```
╭──────────────────────────────────────────────────────────╮
│                                                          │
│   🔍  Search applications...                        ⌘K   │
│   ─────────────────────────────────────────────────────  │
│                                                          │
│   APPLICATIONS                                           │
│   ┌────────────────────────────────────────────────────┐ │
│   │  🦊  Firefox                              browser  │ │ ← selected
│   └────────────────────────────────────────────────────┘ │
│      📁  Files                                  files    │
│      📝  Text Editor                           editor    │
│      🖥️  Terminal                            terminal    │
│      ⚙️  Settings                            settings    │
│                                                          │
│   RECENT                                                 │
│      📄  project-notes.md                    ~/docs      │
│      📄  config.kdl                         ~/.config    │
│                                                          │
╰──────────────────────────────────────────────────────────╯

Background: base (#1e1e2e) with 98% opacity
Border:     1px surface0, 12px radius
Shadow:     0 8px 32px rgba(0,0,0,0.5)
            0 0 0 1px rgba(205,214,244,0.05) (inner glow)
```

**Search Field**:
```
Background:   mantle
Height:       48px
Icon:         20px, subtext0, left-aligned
Placeholder:  "Search applications...", overlay1
Input:        text color, text-base
Focus:        Subtle mauve glow on container
              Cursor: mauve
```

**Result Item**:
```
Height:       44px
Padding:      0 12px
Icon:         24x24px
Label:        text-base, text
Category:     text-xs, subtext0, right-aligned
Gap:          4px vertical between items

States:
- Default:    transparent background
- Hover:      surface0 background, 8px radius
- Selected:   surface1 background, text brightens to white
              Left border: 3px mauve (selection indicator)
- Keyboard:   Same as selected, no left border
```

**Section Headers**:
```
Text:         "APPLICATIONS", "RECENT", etc.
Style:        text-xs, subtext0, uppercase
Spacing:      16px top, 8px bottom
              12px left (align with content)
```

### Settings App

**Layout**: Sidebar (240px) + Content area

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ← ─ ─ 240px ─ ─ → │  ← ─ ─ ─ ─ ─ ─ ─ ─ ─ Content ─ ─ ─ ─ ─ ─ ─ ─ ─ → │
│                    │                                                    │
│   NIRI SETTINGS    │   Appearance                                       │
│   ───────────────  │   ═══════════════════════════════════════════════  │
│                    │                                                    │
│   🔍 Search...     │   GAPS                                             │
│                    │   ─────────────────────────────────────────────    │
│   ┌──────────────┐ │                                                    │
│   │ 🎨 Appearance│ │   Window Spacing                                   │
│   └──────────────┘ │   Space between windows in a workspace             │
│     🖱️ Input      │   ┌────────────────────────────────────┐  16 px    │
│     📐 Layout     │   │ ●━━━━━━━━━━━━━━━━━○───────────────│            │
│     🔧 Behavior   │   └────────────────────────────────────┘            │
│     🖥️ Outputs    │                                                    │
│     ⌨️ Bindings   │   Screen Edge Gap                                   │
│                    │   Space between windows and screen edges           │
│                    │   ┌────────────────────────────────────┐  8 px     │
│                    │   │ ●━━━━━━━━━○───────────────────────│            │
│                    │   └────────────────────────────────────┘            │
│                    │                                                    │
│                    │   COLORS                                           │
│                    │   ─────────────────────────────────────────────    │
│                    │                                                    │
│   ───────────────  │   Focus Ring                                       │
│     ℹ️ About      │   Color of the border around focused windows       │
│                    │   ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐              │
│                    │   │████│ │████│ │████│ │████│ │████│              │
│                    │   └────┘ └────┘ └────┘ └────┘ └────┘              │
│                    │    ▲                                               │
│                    │   selected                                         │
│                    │                                                    │
└─────────────────────────────────────────────────────────────────────────┘

Sidebar background: mantle
Content background: base
```

**Sidebar Navigation**:
```
Item Height:   40px
Padding:       12px horizontal
Icon:          18px, subtext1
Label:         text-sm, subtext1
Gap:           4px between items

States:
- Default:     transparent
- Hover:       surface0 background, 8px radius
- Active:      surface1 background, text/icon brighten to text color
               Left border: 3px mauve
```

**Section Headers** (in content area):
```
Style:         text-xs, subtext0, uppercase, +1px letter-spacing
Divider:       1px surface0 line below (optional, use sparingly)
Spacing:       24px top (first section: 0), 12px bottom
```

**Setting Row**:
```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  Label                                                    [ Control ]   │
│  Description text in muted color                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Height:       Variable (min 56px)
Padding:      16px horizontal, 12px vertical
Label:        text-base, text
Description:  text-sm, subtext0

Hover:        Entire row gets surface0/50% background
              Smooth transition 100ms
```

---

## Interactive Elements

### Toggle Switch

```
Dimensions:   44px × 24px
Track:        22px radius (fully rounded)
Thumb:        18px diameter, 3px inset

OFF State:
┌────────────────────────────────────────────┐
│  Track:   surface1                         │
│  Thumb:   subtext1                         │
│  ●○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○       │
└────────────────────────────────────────────┘

ON State:
┌────────────────────────────────────────────┐
│  Track:   mauve                            │
│  Thumb:   text (white)                     │
│  Glow:    0 0 12px mauve/30%               │
│  ○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○○●        │
└────────────────────────────────────────────┘

Transition:   200ms ease-out
              Thumb slides, track color fades
```

### Slider

```
Dimensions:   Full width, 24px hit area
Track:        4px height, surface1, 2px radius
Progress:     4px height, mauve
Thumb:        16px diameter

┌────────────────────────────────────────────┐
│        ━━━━━━━━━━━━━●─────────────         │
│        └─ filled ─┘ ▲ └─ empty ─┘          │
│                   thumb                     │
└────────────────────────────────────────────┘

States:
- Default:    Thumb is mauve
- Hover:      Thumb scales to 20px, subtle glow
- Dragging:   Thumb scales to 18px, strong glow
- Focus:      Focus ring around thumb (2px mauve, 2px offset)

Value Display:
- Show current value to the right of slider
- text-sm, monospace, subtext1
- Updates live during drag
```

### Button

```
Primary Button
─────────────────────────────────────────────────
Background:   mauve
Text:         crust (dark on light)
Height:       36px
Padding:      0 16px
Radius:       8px
Font:         text-sm, 500 weight

Hover:        Brighten background 10%
              Subtle glow: 0 0 12px mauve/25%
Active:       Darken background 10%, scale 0.98
Focus:        2px ring, 2px offset, mauve/50%


Secondary Button
─────────────────────────────────────────────────
Background:   surface0
Text:         text
Border:       1px surface1

Hover:        surface1 background
Active:       surface2 background


Ghost Button
─────────────────────────────────────────────────
Background:   transparent
Text:         subtext1

Hover:        surface0 background
Active:       surface1 background
```

### Text Input

```
Height:       40px
Background:   mantle
Border:       1px surface1
Radius:       8px
Padding:      0 12px

┌─────────────────────────────────────────────┐
│  │ Placeholder text...                      │
└─────────────────────────────────────────────┘

States:
- Default:    As above
- Hover:      Border brightens to surface2
- Focus:      Border becomes mauve
              Subtle glow: 0 0 8px mauve/20%
              Cursor: mauve
- Error:      Border becomes red
              Glow: 0 0 8px red/20%
- Disabled:   Background crust, text overlay1
```

### Dropdown / Combobox

```
Closed State:
┌─────────────────────────────────────────────┐
│  Selected Value                          ▼  │
└─────────────────────────────────────────────┘

Open State:
┌─────────────────────────────────────────────┐
│  Selected Value                          ▲  │
├─────────────────────────────────────────────┤
│  Option 1                                   │ ← hover: surface0
│  Option 2                               ✓   │ ← selected: surface1 + check
│  Option 3                                   │
└─────────────────────────────────────────────┘

Dropdown panel:
- Background: surface0
- Border: 1px surface1
- Shadow: 0 4px 16px rgba(0,0,0,0.3)
- Radius: 8px
- Max height: 240px (scrollable)
```

---

## Motion & Transitions

### Timing Functions

```
Standard:     cubic-bezier(0.4, 0.0, 0.2, 1)    // Material standard
Decelerate:   cubic-bezier(0.0, 0.0, 0.2, 1)    // Entering elements
Accelerate:   cubic-bezier(0.4, 0.0, 1, 1)      // Exiting elements
Sharp:        cubic-bezier(0.4, 0.0, 0.6, 1)    // Quick state changes
```

### Duration Scale

```
Instant:      0ms       // Disabled states
Fast:         100ms     // Hover states, small changes
Normal:       200ms     // Most transitions
Slow:         300ms     // Complex animations, page transitions
Slower:       500ms     // Dramatic reveals (use sparingly)
```

### Component Animations

```
Launcher Open/Close
─────────────────────────────────────────────────
Open:
  - Fade in: 0 → 1 opacity, 200ms
  - Scale: 0.95 → 1, 200ms, decelerate
  - Origin: center

Close:
  - Fade out: 1 → 0 opacity, 150ms
  - Scale: 1 → 0.95, 150ms, accelerate


Panel Popover (Calendar, Volume, etc.)
─────────────────────────────────────────────────
Open:
  - Fade in: 150ms
  - Slide: -8px → 0 (from top), 200ms, decelerate

Close:
  - Fade out: 100ms
  - Slide: 0 → -8px, 150ms, accelerate


Settings Page Transition
─────────────────────────────────────────────────
Crossfade:
  - Old page: fade out 150ms
  - New page: fade in 150ms, 50ms delay
  - No position animation (instant swap)


List Item Hover
─────────────────────────────────────────────────
Background:   100ms ease-out
Ideal:        User should barely notice the transition
              but absence would feel jarring
```

---

## Iconography

### Style Guidelines

```
Type:         Outline style, 1.5px stroke
Size:         18px default (panel, lists)
              24px large (launcher results)
              16px small (badges, inline)
Corner:       2px radius on corners
Color:        Inherit from text color
              Single color, no fills

Recommended Sets:
- Phosphor Icons (preferred - matches aesthetic name!)
- Lucide
- Tabler Icons
```

### System Status Icons

```
Battery:
  🔋 Full      ─────────  green fill
  🔋 Medium    ─────      text fill
  🔋 Low       ──         peach fill
  🔋 Critical  ─          red fill, pulsing
  🔌 Charging  ─────⚡    green + lightning

Network:
  📶 Connected      Full arcs, text color
  📶 Weak           Partial arcs, subtext0
  📶 Disconnected   X overlay, overlay1
  📶 VPN            Lock badge overlay

Audio:
  🔊 High     Three waves
  🔉 Medium   Two waves
  🔈 Low      One wave
  🔇 Muted    X overlay, subtext0
```

---

## Shadows & Elevation

### Elevation Levels

```
Level 0 - Flat
─────────────────────────────────────────────────
Shadow: none
Use:    Inline elements, backgrounds

Level 1 - Raised
─────────────────────────────────────────────────
Shadow: 0 1px 3px rgba(0,0,0,0.12),
        0 1px 2px rgba(0,0,0,0.24)
Use:    Cards, elevated surfaces

Level 2 - Floating
─────────────────────────────────────────────────
Shadow: 0 3px 6px rgba(0,0,0,0.15),
        0 2px 4px rgba(0,0,0,0.12)
Use:    Dropdowns, popovers

Level 3 - Modal
─────────────────────────────────────────────────
Shadow: 0 10px 20px rgba(0,0,0,0.2),
        0 3px 6px rgba(0,0,0,0.15)
Use:    Dialogs, launcher

Level 4 - Dramatic
─────────────────────────────────────────────────
Shadow: 0 15px 30px rgba(0,0,0,0.25),
        0 5px 15px rgba(0,0,0,0.2)
Use:    Context menus, high-priority overlays
```

---

## Accessibility

### Focus Indicators

```
All interactive elements MUST have visible focus:

Default Focus Ring:
  - 2px solid mauve
  - 2px offset from element
  - Subtle glow: 0 0 0 4px mauve/20%

High Contrast Mode:
  - 3px solid white
  - 2px offset
  - No glow (can interfere)
```

### Color Contrast

```
Catppuccin Mocha contrast ratios:
─────────────────────────────────────────────────
text (#cdd6f4) on base (#1e1e2e):     12.5:1 ✓
subtext1 (#bac2de) on base:           9.1:1  ✓
subtext0 (#a6adc8) on base:           6.8:1  ✓
overlay1 (#7f849c) on base:           4.0:1  ✓ (large text only)

mauve (#cba6f7) on crust (#11111b):   9.8:1  ✓
```

### Minimum Touch Targets

```
Desktop:    32px minimum (we use 40-48px)
Pointer:    24px minimum for icons
Gap:        4px minimum between targets
```

---

## Slint Implementation Notes

### Theme Struct

```slint
// theme.slint

export global Theme {
    // Colors - populated from KDL config
    in-out property <brush> crust: #11111b;
    in-out property <brush> mantle: #181825;
    in-out property <brush> base: #1e1e2e;
    in-out property <brush> surface0: #313244;
    in-out property <brush> surface1: #45475a;
    in-out property <brush> surface2: #585b70;
    in-out property <brush> text: #cdd6f4;
    in-out property <brush> subtext1: #bac2de;
    in-out property <brush> subtext0: #a6adc8;
    in-out property <brush> overlay1: #7f849c;
    in-out property <brush> mauve: #cba6f7;
    // ... etc

    // Semantic aliases
    in-out property <brush> bg-deep: crust;
    in-out property <brush> bg-base: base;
    in-out property <brush> accent: mauve;

    // Spacing
    in-out property <length> space-1: 4px;
    in-out property <length> space-2: 8px;
    in-out property <length> space-3: 12px;
    in-out property <length> space-4: 16px;

    // Typography
    in-out property <length> text-sm: 12px;
    in-out property <length> text-base: 14px;
    in-out property <length> text-lg: 16px;

    // Radii
    in-out property <length> radius-sm: 4px;
    in-out property <length> radius-md: 8px;
    in-out property <length> radius-lg: 12px;

    // Durations
    in-out property <duration> duration-fast: 100ms;
    in-out property <duration> duration-normal: 200ms;
}
```

### Reusable Component Pattern

```slint
// widgets/toggle.slint

import { Theme } from "../theme.slint";

export component Toggle inherits Rectangle {
    in-out property <bool> checked: false;
    callback toggled(bool);

    width: 44px;
    height: 24px;
    border-radius: 12px;
    background: checked ? Theme.mauve : Theme.surface1;

    animate background { duration: Theme.duration-normal; }

    // Thumb
    Rectangle {
        x: checked ? parent.width - self.width - 3px : 3px;
        y: 3px;
        width: 18px;
        height: 18px;
        border-radius: 9px;
        background: checked ? Theme.text : Theme.subtext1;

        animate x { duration: Theme.duration-normal; easing: ease-out; }
        animate background { duration: Theme.duration-normal; }
    }

    // Glow effect when checked
    drop-shadow-blur: checked ? 12px : 0px;
    drop-shadow-color: checked ? Theme.mauve.with-alpha(0.3) : transparent;

    TouchArea {
        clicked => {
            checked = !checked;
            toggled(checked);
        }
    }
}
```

---

## File Organization

```
ui/
├── theme.slint              # Global theme tokens
├── styles.slint             # Legacy (migrate to theme.slint)
├── widgets/
│   ├── toggle.slint
│   ├── slider.slint
│   ├── button.slint
│   ├── text-input.slint
│   ├── dropdown.slint
│   ├── setting-row.slint    # Compound: label + description + control
│   ├── section-header.slint
│   └── icon.slint           # SVG icon wrapper
├── pages/
│   └── ...                  # Settings pages
└── main.slint
```

---

*Phosphor Design System v0.1 — Niri Desktop Environment*
