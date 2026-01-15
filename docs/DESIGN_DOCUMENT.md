# Niri Settings - Design Document

## Executive Summary

This document outlines the design principles, patterns, and consistency guidelines for the Niri Settings application, based on UX audit findings and research into modern settings applications (GNOME, KDE, macOS, Windows 11, elementary OS, Android).

---

## Current State Assessment

### What's Working Well
- Excellent use of Theme constants (no magic values)
- Strong accessibility foundation with accessible-role, accessible-label attributes
- Consistent use of SettingsSection wrapper for logical grouping
- Clean separation of concerns with widget components
- Consistent text color scheme (text-primary, text-secondary, text-muted)
- All pages implement ScrollView for content overflow
- Consistent padding-right treatment for scrollbar accommodation

### Issues Identified (By Severity)

#### HIGH PRIORITY
1. **Section Title Capitalization** - Mixed UPPERCASE vs Title Case
2. **Text Helper Placement** - Inconsistent position (before vs after controls)
3. **Conditional Layout Wrapping** - Inconsistent VerticalLayout usage
4. **Window Rules Page Structure** - Different from all other pages (split-pane)

#### MEDIUM PRIORITY
5. **Bottom Spacer Formatting** - Single-line vs multi-line inconsistency
6. **ColorPicker Wrapper** - Sometimes in SettingRow, sometimes standalone
7. **Conditional Content Spacing** - Varies when toggling features
8. **Accessible-Role Coverage** - Some widgets missing roles

#### LOW PRIORITY
9. **Information Sections** - Not all complex pages have them
10. **ComboBox Accessibility** - Missing accessible-descriptions on some

---

## Design Principles (Adopted from Research)

### 1. Context Over Separation
Keep settings close to where they're used. Avoid forcing users to navigate away from their current context.

### 2. Progressive Disclosure
Show essential settings first. Hide advanced options in expandable sections or subscreens.

### 3. Immediate Feedback
Toggle switches take effect immediately. No "Save" button required. Visual feedback confirms changes.

### 4. Consistent Hierarchy
- **2-3 levels maximum** for navigation depth
- **Hub-and-spoke** for main categories
- **List-detail** for collection-based settings (Window Rules)

### 5. Accessibility First
- All interactive elements keyboard navigable
- Screen reader support with proper ARIA roles
- Minimum 44px tap targets
- Color not the only indicator of state

---

## Component Patterns

### Page Structure (Standard)
```
ScrollView {
    VerticalLayout {
        spacing: Theme.spacing-lg;
        padding-right: Theme.spacing-md;

        SettingsSection { title: "SECTION NAME"; ... }
        SettingsSection { title: "SECTION NAME"; ... }

        // Bottom spacer
        Rectangle { height: Theme.spacing-lg; }
    }
}
```

### Section Title Convention
**Decision: ALL UPPERCASE** (matches majority of existing pages)
```
SettingsSection {
    title: "FOCUS RING";  // Correct
    title: "Focus Ring";  // Incorrect
}
```

### Description Text Placement
**Decision: At TOP of section, before any controls**
```
SettingsSection {
    title: "SECTION NAME";

    VerticalLayout {
        spacing: Theme.spacing-sm;

        // Description first (when needed)
        Text {
            text: "Explanation of what this section controls";
            color: Theme.text-muted;
            font-size: Theme.font-size-sm;
            wrap: word-wrap;
        }

        // Then controls
        ToggleRow { ... }
        SliderRow { ... }
    }
}
```

### Conditional Content Pattern
**Decision: Always wrap in VerticalLayout for consistent spacing**
```
ToggleRow {
    label: "Enable feature";
    checked <=> root.feature-enabled;
    toggled(val) => { ... }
}

if root.feature-enabled: VerticalLayout {
    spacing: Theme.spacing-sm;

    SliderRow { ... }
    SettingRow { ... }
}
```

### Toggle Placement
- **Right-aligned** within row (standard Western pattern)
- **Immediate effect** (no save button)
- **Label on left**, toggle on right

### List-Based Settings (Window Rules Pattern)
```
HorizontalLayout {
    // Left: List panel (fixed width)
    Rectangle {
        width: 250px;
        // Header with Add button
        // ScrollView with selectable items
        // Empty state message
    }

    // Right: Detail panel (flexible)
    if selected-index >= 0: Rectangle {
        horizontal-stretch: 1;
        ScrollView {
            // SettingsSection groups
        }
    }

    // No selection state
    if selected-index < 0: Rectangle {
        // Placeholder message
    }
}
```

---

## Spacing System

Based on Theme constants (8pt base):

| Token | Value | Usage |
|-------|-------|-------|
| spacing-xs | 4px | Within compact elements |
| spacing-sm | 8px | Between related items |
| spacing-md | 16px | Section padding, moderate gaps |
| spacing-lg | 24px | Between sections |
| spacing-xl | 32px | Major content separation |

### Row Heights
- **Compact** (no description): 48px
- **Expanded** (with description): 72px

---

## Typography Hierarchy

| Element | Size | Weight | Color |
|---------|------|--------|-------|
| Page Title | font-size-xl | 600 | text-primary |
| Section Title | font-size-md | 600 | text-muted |
| Setting Label | font-size-md | 400 | text-primary |
| Description | font-size-sm | 400 | text-muted |
| Helper Text | font-size-sm | 400 | text-muted |

---

## Accessibility Requirements

### All Interactive Elements
- `accessible-role` defined
- `accessible-label` with clear description
- `accessible-description` for complex controls (optional)

### Keyboard Navigation
- Tab moves between controls
- Space/Enter activates toggles/buttons
- Arrow keys for sliders and lists

### Screen Reader
- Section titles announced as headings
- Toggle states clearly communicated
- Error/success states announced

---

## Search Integration

Current: Category 0-12 with searchable items in `src/ui/search.rs`

Each searchable item includes:
- Category index (for navigation)
- Label (display name)
- Description (search context)
- Keywords (search terms)

---

## Recommended Fixes

### Immediate (Before Release)
1. [ ] Standardize all section titles to UPPERCASE
2. [ ] Move all section descriptions to top (before controls)
3. [ ] Ensure all conditional content uses VerticalLayout wrapper
4. [ ] Add `visible:` property to overlay components (DONE)

### Short-term
5. [ ] Add accessible-descriptions to all ComboBox controls
6. [ ] Standardize ColorPicker usage (standalone like ToggleRow)
7. [ ] Add INFORMATION sections to complex pages (Keyboard, Window Rules)

### Long-term
8. [ ] Create component style guide
9. [ ] Add visual regression testing
10. [ ] Consider responsive breakpoints for sidebar

---

## Platform Comparison Summary

| Feature | GNOME | KDE | macOS | Windows 11 | Our App |
|---------|-------|-----|-------|------------|---------|
| Sidebar Nav | Hub-spoke | 3-level | Sidebar | Sidebar | Sidebar |
| Search | Overview | Yes | Spotlight | Yes | Yes |
| Toggles | Right | Right | Right | Right | Right |
| List-Detail | No | Yes | Yes | Yes | Yes (Rules) |
| Sections | Cards | Cards | Cards | Cards | Cards |
| Responsive | No | Partial | Yes | Yes | No |

---

## Version History

- **v0.1** - Initial design document based on UX audit
- Created: 2024-12-05
