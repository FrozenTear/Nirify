//! Message types for the Elm Architecture
//!
//! This module defines all possible events/actions in the application.
//! Messages flow: User interaction → Message → update() → State change → view()
//!
//! # Architecture
//!
//! The message system follows the Elm Architecture pattern:
//! 1. User interacts with UI → creates a `Message`
//! 2. `update()` matches on the message and modifies state
//! 3. `view()` renders the updated state
//!
//! # Organization
//!
//! Messages are organized into nested enums by settings category:
//!
//! - **Navigation & System**: `NavigateToPage`, `ToggleSidebar`, `SearchQueryChanged`, etc.
//! - **Visual Settings**: `AppearanceMessage`, `AnimationsMessage`, `CursorMessage`
//! - **Behavior Settings**: `BehaviorMessage`, `LayoutExtrasMessage`, `GesturesMessage`
//! - **Input Devices**: `KeyboardMessage`, `MouseMessage`, `TouchpadMessage`,
//!   `TrackpointMessage`, `TrackballMessage`, `TabletMessage`, `TouchMessage`
//! - **Rules & Bindings**: `WindowRulesMessage`, `LayerRulesMessage`, `KeybindingsMessage`
//! - **System Configuration**: `OutputsMessage`, `WorkspacesMessage`, `EnvironmentMessage`,
//!   `StartupMessage`, `MiscellaneousMessage`
//! - **Advanced**: `DebugMessage`, `SwitchEventsMessage`, `RecentWindowsMessage`
//! - **App Management**: `ToolsMessage`, `ConfigEditorMessage`, `BackupsMessage`, `PreferencesMessage`
//!
//! # Why Nested Enums?
//!
//! - **Namespacing**: Avoids name collisions (e.g., `WindowRulesMessage::AddRule` vs `LayerRulesMessage::AddRule`)
//! - **Handler Organization**: Each category can have its own handler function
//! - **IDE Navigation**: Easy to find all messages for a specific feature
//! - **Testing**: Categories can be unit tested independently

use iced::widget::text_editor;

use crate::config::ColumnWidthType;
use crate::types::{
    AccelProfile, CenterFocusedColumn, ClickMethod, ModKey, ScrollMethod, TapButtonMap,
    WarpMouseMode,
};
use crate::views::widgets::GradientPickerMessage;

/// Root message enum - all possible application events
#[derive(Debug, Clone)]
pub enum Message {
    /// No-op message (used when text parse fails in slider inputs)
    NoOp,

    // ═══════════════════════════════════════════════════════════════════════════
    // Navigation & UI
    // ═══════════════════════════════════════════════════════════════════════════
    NavigateToPage(Page),
    ToggleSidebar,
    SearchQueryChanged(String),
    SearchResultSelected(usize),
    ClearSearch,
    ChangeTheme(crate::theme::AppTheme),
    /// System theme event from portal or file watcher
    SystemThemeEvent(crate::system_theme::SystemThemeEvent),
    /// Toggle search bar visibility / focus (Ctrl+K)
    ToggleSearch,

    // ═══════════════════════════════════════════════════════════════════════════
    // Visual Settings
    // ═══════════════════════════════════════════════════════════════════════════
    Appearance(AppearanceMessage),
    Animations(AnimationsMessage),
    Cursor(CursorMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // Behavior & Layout
    // ═══════════════════════════════════════════════════════════════════════════
    Behavior(BehaviorMessage),
    LayoutExtras(LayoutExtrasMessage),
    Gestures(GesturesMessage),
    Workspaces(WorkspacesMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // Input Devices
    // ═══════════════════════════════════════════════════════════════════════════
    Keyboard(KeyboardMessage),
    Mouse(MouseMessage),
    Touchpad(TouchpadMessage),
    Trackpoint(TrackpointMessage),
    Trackball(TrackballMessage),
    Tablet(TabletMessage),
    Touch(TouchMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // Rules & Bindings
    // ═══════════════════════════════════════════════════════════════════════════
    WindowRules(WindowRulesMessage),
    LayerRules(LayerRulesMessage),
    Keybindings(KeybindingsMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // System Configuration
    // ═══════════════════════════════════════════════════════════════════════════
    Overview(OverviewMessage),
    Outputs(OutputsMessage),
    Miscellaneous(MiscellaneousMessage),
    Environment(EnvironmentMessage),
    Startup(StartupMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // Advanced Features
    // ═══════════════════════════════════════════════════════════════════════════
    Debug(DebugMessage),
    SwitchEvents(SwitchEventsMessage),
    RecentWindows(RecentWindowsMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // App Management
    // ═══════════════════════════════════════════════════════════════════════════
    Tools(ToolsMessage),
    Preferences(PreferencesMessage),
    ConfigEditor(ConfigEditorMessage),
    Backups(BackupsMessage),

    // ═══════════════════════════════════════════════════════════════════════════
    // Save & Persistence
    // ═══════════════════════════════════════════════════════════════════════════
    Save(SaveMessage),
    SaveCompleted(crate::save_manager::SaveResult),
    ReloadCompleted(crate::save_manager::ReloadResult),

    // ═══════════════════════════════════════════════════════════════════════════
    // Dialogs & Modals
    // ═══════════════════════════════════════════════════════════════════════════
    ShowDialog(DialogState),
    CloseDialog,
    DialogConfirm,
    WizardNext,
    WizardBack,
    WizardSetupConfig,
    /// Toggle a wizard consolidation suggestion
    WizardConsolidationToggle(usize),
    /// Apply selected wizard consolidation suggestions
    WizardConsolidationApply,
    /// Skip wizard consolidation step
    WizardConsolidationSkip,
    /// Analyze rules and show consolidation dialog if suggestions found
    AnalyzeConsolidation,
    /// Toggle selection of a consolidation suggestion
    ConsolidationToggle(usize),
    /// Apply selected consolidation suggestions
    ConsolidationApply,

    // ═══════════════════════════════════════════════════════════════════════════
    // System Events
    // ═══════════════════════════════════════════════════════════════════════════
    WindowCloseRequested,
    /// Trigger async niri status check
    CheckNiriStatus,
    /// Async niri status check completed
    NiriStatusChecked(bool),
    ClearToast,
    /// No-op message (for optional callbacks that don't need action)
    None,

    // ═══════════════════════════════════════════════════════════════════════════
    // Redesign Navigation
    // ═══════════════════════════════════════════════════════════════════════════
    /// Navigate to a redesigned screen
    NavigateToScreen(Screen),
    /// Change sub-tab within the Input screen (legacy)
    SetInputSubTab(InputSubTab),
    /// Open a section editor modal (Layout/Visuals/System)
    OpenSectionEditor(EditableSection),
    /// Close the section editor modal
    CloseSectionEditor,
    /// Open a device editor modal
    OpenDeviceEditor(EditableDevice),
    /// Close the device editor modal
    CloseDeviceEditor,
    /// Open a keybinding editor modal
    OpenKeybindingEditor(usize),
    /// Close the keybinding editor modal
    CloseKeybindingEditor,
    /// Set keybindings search filter
    SetKeybindingsSearch(String),
    /// Change sub-tab within the Rules screen
    SetRulesSubTab(RulesSubTab),
    /// Change sub-tab within the Gear screen
    SetGearSubTab(GearSubTab),
}

/// Page navigation enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Overview,
    Appearance,
    Behavior,
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
    Animations,
    Cursor,
    LayoutExtras,
    Gestures,
    Workspaces,
    WindowRules,
    LayerRules,
    Keybindings,
    Outputs,
    Miscellaneous,
    Startup,
    Environment,
    Debug,
    SwitchEvents,
    RecentWindows,
    Tools,
    Preferences,
    ConfigEditor,
    Backups,
}

impl Page {
    /// Returns the display name for the page
    pub fn name(&self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Appearance => "Appearance",
            Page::Behavior => "Behavior",
            Page::Keyboard => "Keyboard",
            Page::Mouse => "Mouse",
            Page::Touchpad => "Touchpad",
            Page::Trackpoint => "Trackpoint",
            Page::Trackball => "Trackball",
            Page::Tablet => "Tablet",
            Page::Touch => "Touch",
            Page::Animations => "Animations",
            Page::Cursor => "Cursor",
            Page::LayoutExtras => "Layout Extras",
            Page::Gestures => "Gestures",
            Page::Workspaces => "Workspaces",
            Page::WindowRules => "Window Rules",
            Page::LayerRules => "Layer Rules",
            Page::Keybindings => "Keybindings",
            Page::Outputs => "Outputs",
            Page::Miscellaneous => "Miscellaneous",
            Page::Startup => "Startup",
            Page::Environment => "Environment",
            Page::Debug => "Debug",
            Page::SwitchEvents => "Switch Events",
            Page::RecentWindows => "Recent Windows",
            Page::Tools => "Tools",
            Page::Preferences => "Preferences",
            Page::ConfigEditor => "Config Editor",
            Page::Backups => "Backups",
        }
    }

    /// Returns the category group for sidebar organization
    pub fn category(&self) -> PageCategory {
        match self {
            Page::Overview => PageCategory::System,
            Page::Appearance => PageCategory::Visual,
            Page::Behavior => PageCategory::Visual,
            Page::Keyboard
            | Page::Mouse
            | Page::Touchpad
            | Page::Trackpoint
            | Page::Trackball
            | Page::Tablet
            | Page::Touch => PageCategory::Input,
            Page::Animations | Page::Cursor => PageCategory::Visual,
            Page::LayoutExtras | Page::Workspaces => PageCategory::Layout,
            Page::WindowRules | Page::LayerRules => PageCategory::Rules,
            Page::Keybindings | Page::Gestures => PageCategory::Input,
            Page::Outputs => PageCategory::System,
            Page::Miscellaneous | Page::Startup | Page::Environment => PageCategory::System,
            Page::Debug | Page::SwitchEvents | Page::RecentWindows => PageCategory::Advanced,
            Page::Tools | Page::Preferences => PageCategory::System,
            Page::ConfigEditor | Page::Backups => PageCategory::System,
        }
    }
}

/// Page category for sidebar grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageCategory {
    System,
    Visual,
    Input,
    Layout,
    Rules,
    Advanced,
}

impl PageCategory {
    pub fn name(&self) -> &'static str {
        match self {
            PageCategory::System => "System",
            PageCategory::Visual => "Visual",
            PageCategory::Input => "Input Devices",
            PageCategory::Layout => "Layout",
            PageCategory::Rules => "Rules",
            PageCategory::Advanced => "Advanced",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCREEN NAVIGATION (Redesign)
// ═══════════════════════════════════════════════════════════════════════════════

/// Top-level screen in the redesigned navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Layout,
    Visuals,
    Input,
    Rules,
    Displays,
    System,
    Gear,
}

impl Screen {
    /// Returns the display name for the screen
    pub fn name(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Layout => "Layout",
            Screen::Visuals => "Visuals",
            Screen::Input => "Input",
            Screen::Rules => "Rules",
            Screen::Displays => "Displays",
            Screen::System => "System",
            Screen::Gear => "Settings",
        }
    }

    /// Maps a legacy Page to the Screen it now lives in
    pub fn from_page(page: Page) -> Screen {
        match page {
            Page::Overview => Screen::Dashboard,
            Page::Appearance | Page::Animations | Page::Cursor => Screen::Visuals,
            Page::Behavior | Page::LayoutExtras | Page::Workspaces => Screen::Layout,
            Page::Keyboard
            | Page::Mouse
            | Page::Touchpad
            | Page::Trackpoint
            | Page::Trackball
            | Page::Tablet
            | Page::Touch
            | Page::Keybindings
            | Page::Gestures => Screen::Input,
            Page::WindowRules | Page::LayerRules => Screen::Rules,
            Page::Outputs => Screen::Displays,
            Page::Miscellaneous
            | Page::Startup
            | Page::Environment
            | Page::Debug
            | Page::SwitchEvents
            | Page::RecentWindows => Screen::System,
            Page::Tools | Page::Preferences | Page::ConfigEditor | Page::Backups => Screen::Gear,
        }
    }

    /// Maps a legacy Page to the InputSubTab (if applicable)
    pub fn input_sub_tab_from_page(page: Page) -> Option<InputSubTab> {
        match page {
            Page::Keybindings => Some(InputSubTab::Keybindings),
            Page::Keyboard => Some(InputSubTab::Keyboard),
            Page::Mouse => Some(InputSubTab::Mouse),
            Page::Touchpad => Some(InputSubTab::Touchpad),
            Page::Trackpoint => Some(InputSubTab::Trackpoint),
            Page::Trackball => Some(InputSubTab::Trackball),
            Page::Tablet => Some(InputSubTab::Tablet),
            Page::Touch => Some(InputSubTab::Touch),
            Page::Gestures => Some(InputSubTab::Gestures),
            _ => None,
        }
    }

    /// All screens in sidebar order (excluding Gear which is bottom-anchored)
    pub fn sidebar_items() -> &'static [Screen] {
        &[
            Screen::Dashboard,
            Screen::Layout,
            Screen::Visuals,
            Screen::Input,
            Screen::Rules,
            Screen::Displays,
            Screen::System,
        ]
    }
}

/// Sub-tab within the Input screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputSubTab {
    #[default]
    Keybindings,
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
    Gestures,
}

impl InputSubTab {
    pub fn name(&self) -> &'static str {
        match self {
            InputSubTab::Keybindings => "Keybindings",
            InputSubTab::Keyboard => "Keyboard",
            InputSubTab::Mouse => "Mouse",
            InputSubTab::Touchpad => "Touchpad",
            InputSubTab::Trackpoint => "Trackpoint",
            InputSubTab::Trackball => "Trackball",
            InputSubTab::Tablet => "Tablet",
            InputSubTab::Touch => "Touch",
            InputSubTab::Gestures => "Gestures",
        }
    }

    pub fn all() -> &'static [InputSubTab] {
        &[
            InputSubTab::Keybindings,
            InputSubTab::Keyboard,
            InputSubTab::Mouse,
            InputSubTab::Touchpad,
            InputSubTab::Trackpoint,
            InputSubTab::Trackball,
            InputSubTab::Tablet,
            InputSubTab::Touch,
            InputSubTab::Gestures,
        ]
    }
}

/// Device types that can be edited in a modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableDevice {
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
    Gestures,
}

impl EditableDevice {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Keyboard => "Keyboard",
            Self::Mouse => "Mouse",
            Self::Touchpad => "Touchpad",
            Self::Trackpoint => "Trackpoint",
            Self::Trackball => "Trackball",
            Self::Tablet => "Tablet",
            Self::Touch => "Touch Screen",
            Self::Gestures => "Gestures",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Keyboard => "⌨",
            Self::Mouse => "◎",
            Self::Touchpad => "▦",
            Self::Trackpoint => "◉",
            Self::Trackball => "◉",
            Self::Tablet => "▭",
            Self::Touch => "☐",
            Self::Gestures => "✋",
        }
    }
}

/// Sub-tab within the Rules screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RulesSubTab {
    #[default]
    WindowRules,
    LayerRules,
}

impl RulesSubTab {
    pub fn name(&self) -> &'static str {
        match self {
            RulesSubTab::WindowRules => "Window Rules",
            RulesSubTab::LayerRules => "Layer Rules",
        }
    }

    pub fn all() -> &'static [RulesSubTab] {
        &[RulesSubTab::WindowRules, RulesSubTab::LayerRules]
    }
}

/// Filter for rules display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RulesFilter {
    #[default]
    All,
    Active,
    Disabled,
}

/// Sections that can be edited in a modal (Layout, Visuals, System screens)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableSection {
    // Layout
    SpatialGaps,
    CenteringDynamics,
    ColumnManager,
    ScreenEdgeStruts,
    TabIndicator,
    InsertHint,
    NamedWorkspaces,
    // Visuals
    FocusRing,
    WindowBorder,
    WindowShadow,
    ModifierKeys,
    Animations,
    Cursor,
    // System
    StartupPrograms,
    EnvironmentVars,
    Miscellaneous,
    SwitchEvents,
    Debug,
    RecentWindows,
}

impl EditableSection {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SpatialGaps => "Spatial Gaps",
            Self::CenteringDynamics => "Centering Dynamics",
            Self::ColumnManager => "Column Manager",
            Self::ScreenEdgeStruts => "Screen Edge Struts",
            Self::TabIndicator => "Tab Indicator",
            Self::InsertHint => "Insert Hint",
            Self::NamedWorkspaces => "Named Workspaces",
            Self::FocusRing => "Focus Ring",
            Self::WindowBorder => "Window Border",
            Self::WindowShadow => "Window Shadow",
            Self::ModifierKeys => "Modifier Keys",
            Self::Animations => "Animations",
            Self::Cursor => "Cursor",
            Self::StartupPrograms => "Startup Programs",
            Self::EnvironmentVars => "Environment Variables",
            Self::Miscellaneous => "Miscellaneous",
            Self::SwitchEvents => "Switch Events",
            Self::Debug => "Debug",
            Self::RecentWindows => "Recent Windows",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::SpatialGaps => "⊞",
            Self::CenteringDynamics => "◎",
            Self::ColumnManager => "▦",
            Self::ScreenEdgeStruts => "◧",
            Self::TabIndicator => "▤",
            Self::InsertHint => "◇",
            Self::NamedWorkspaces => "▥",
            Self::FocusRing => "◉",
            Self::WindowBorder => "▭",
            Self::WindowShadow => "◌",
            Self::ModifierKeys => "⌥",
            Self::Animations => "◈",
            Self::Cursor => "↗",
            Self::StartupPrograms => "⚡",
            Self::EnvironmentVars => "⚙",
            Self::Miscellaneous => "⬡",
            Self::SwitchEvents => "⏻",
            Self::Debug => "⊘",
            Self::RecentWindows => "◫",
        }
    }

    pub fn accent(&self) -> iced::Color {
        use crate::theme::neon;
        match self {
            Self::SpatialGaps
            | Self::CenteringDynamics
            | Self::FocusRing
            | Self::Animations
            | Self::StartupPrograms => neon::PRIMARY,
            Self::ColumnManager
            | Self::TabIndicator
            | Self::WindowBorder
            | Self::ModifierKeys
            | Self::EnvironmentVars
            | Self::SwitchEvents => neon::SECONDARY,
            _ => neon::TERTIARY,
        }
    }
}

/// Sub-tab within the Gear screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GearSubTab {
    #[default]
    Tools,
    Preferences,
    ConfigEditor,
    Backups,
}

impl GearSubTab {
    pub fn name(&self) -> &'static str {
        match self {
            GearSubTab::Tools => "Tools",
            GearSubTab::Preferences => "Preferences",
            GearSubTab::ConfigEditor => "Config Editor",
            GearSubTab::Backups => "Backups",
        }
    }

    pub fn all() -> &'static [GearSubTab] {
        &[
            GearSubTab::Tools,
            GearSubTab::Preferences,
            GearSubTab::ConfigEditor,
            GearSubTab::Backups,
        ]
    }
}

// Implement TabLabel for sub-tab enums
impl crate::views::screens::TabLabel for InputSubTab {
    fn label(&self) -> &'static str {
        self.name()
    }
}
impl crate::views::screens::TabLabel for RulesSubTab {
    fn label(&self) -> &'static str {
        self.name()
    }
}
impl crate::views::screens::TabLabel for GearSubTab {
    fn label(&self) -> &'static str {
        self.name()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VISUAL SETTINGS MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Appearance settings messages
///
/// Controls visual elements: focus ring, border, gaps, corner radius, background
#[derive(Debug, Clone)]
pub enum AppearanceMessage {
    // Focus ring
    ToggleFocusRing(bool),
    SetFocusRingWidth(f32),
    FocusRingActive(GradientPickerMessage),
    FocusRingInactive(GradientPickerMessage),
    FocusRingUrgent(GradientPickerMessage),

    // Border
    ToggleBorder(bool),
    SetBorderThickness(f32),
    BorderActive(GradientPickerMessage),
    BorderInactive(GradientPickerMessage),
    BorderUrgent(GradientPickerMessage),

    // Layout
    SetGaps(f32),
    SetCornerRadius(f32),

    // Background
    SetBackgroundColor(Option<String>), // Optional hex color string
}

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR & LAYOUT MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Behavior settings messages
///
/// Controls: focus behavior, workspace navigation, column defaults, modifier keys
#[derive(Debug, Clone)]
pub enum BehaviorMessage {
    // Focus
    ToggleFocusFollowsMouse(bool),
    SetFocusFollowsMouseMaxScroll(Option<f32>),
    SetWarpMouseToFocus(WarpMouseMode),

    // Workspace
    ToggleWorkspaceAutoBackAndForth(bool),
    ToggleAlwaysCenterSingleColumn(bool),
    ToggleEmptyWorkspaceAboveFirst(bool),
    SetCenterFocusedColumn(CenterFocusedColumn),

    // Default column width
    SetDefaultColumnWidthType(ColumnWidthType),

    // Struts
    SetStrutLeft(f32),
    SetStrutRight(f32),
    SetStrutTop(f32),
    SetStrutBottom(f32),

    // Modifier keys
    SetModKey(ModKey),
    SetModKeyNested(Option<ModKey>),

    // Power
    ToggleDisablePowerKeyHandling(bool),
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT DEVICE MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Keyboard settings messages
///
/// Controls: XKB layout, variant, options, repeat rate
#[derive(Debug, Clone)]
pub enum KeyboardMessage {
    SetXkbLayout(String),
    SetXkbVariant(String),
    SetXkbOptions(String),
    SetXkbModel(String),
    SetXkbRules(String),
    SetXkbFile(String),
    SetRepeatDelay(i32),
    SetRepeatRate(i32),
    SetTrackLayout(String),
    SetNumlock(bool),
}

/// Mouse settings messages
///
/// Controls: natural scroll, acceleration, scroll method, button emulation
#[derive(Debug, Clone)]
pub enum MouseMessage {
    ToggleOffOnTouchpad(bool),
    ToggleNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollFactor(f32),
    SetScrollFactorHorizontal(Option<f32>),
    SetScrollMethod(ScrollMethod),
    SetScrollButton(Option<i32>),
    ToggleLeftHanded(bool),
    ToggleMiddleEmulation(bool),
    ToggleScrollButtonLock(bool),
}

/// Touchpad settings messages
///
/// Controls: tap-to-click, DWT, gestures, scroll, acceleration
#[derive(Debug, Clone)]
pub enum TouchpadMessage {
    ToggleTapToClick(bool),
    ToggleDwt(bool),
    ToggleDwtp(bool),
    ToggleNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollFactor(f32),
    SetScrollFactorHorizontal(Option<f32>),
    SetScrollMethod(ScrollMethod),
    SetScrollButton(Option<i32>),
    SetClickMethod(ClickMethod),
    SetTapButtonMap(TapButtonMap),
    ToggleLeftHanded(bool),
    ToggleDrag(bool),
    ToggleDragLock(bool),
    ToggleMiddleEmulation(bool),
    ToggleDisabledOnExternalMouse(bool),
}

/// Animations settings messages
///
/// Controls all 11 animation types: duration, curve, spring parameters, custom shaders
#[derive(Debug, Clone)]
pub enum AnimationsMessage {
    ToggleSlowdown(bool),
    SetSlowdownFactor(f32),

    // Per-animation messages (11 animation types)
    SetAnimationEnabled(String, bool), // (animation_name, enabled)
    SetAnimationDuration(String, i32), // (animation_name, duration_ms)
    SetAnimationCurve(String, String), // (animation_name, curve_name)
    SetAnimationSpringDampingRatio(String, f32),
    SetAnimationSpringEpsilon(String, f32),

    // Animation type selection (Default, Off, Spring, Easing, CustomShader)
    SetAnimationType(String, i32), // (animation_name, type_index: 0=Default, 1=Off, 2=Spring, 3=Easing, 4=CustomShader)

    // Custom shader support (only for window-open, window-close, window-resize)
    SetCustomShader(String, String), // (animation_name, shader_code)
    ClearCustomShader(String),       // (animation_name)
    InsertShaderTemplate(String),    // (animation_name) - inserts default function signature
}

/// Cursor settings messages
#[derive(Debug, Clone)]
pub enum CursorMessage {
    SetTheme(String),
    SetSize(i32),
    ToggleHideWhenTyping(bool),
    SetHideAfterInactive(Option<i32>),
}

/// Workspaces settings messages
#[derive(Debug, Clone)]
pub enum WorkspacesMessage {
    AddWorkspace,
    RemoveWorkspace(usize),
    UpdateWorkspaceName(usize, String),
    UpdateWorkspaceOutput(usize, Option<String>),
    MoveWorkspaceUp(usize),
    MoveWorkspaceDown(usize),
}

// ═══════════════════════════════════════════════════════════════════════════════
// RULES & BINDINGS MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Window rules settings messages
///
/// Manages per-application rules: matching criteria, open behavior, sizing, styling
#[derive(Debug, Clone)]
pub enum WindowRulesMessage {
    // List management
    AddRule,
    DeleteRule(u32),    // Rule ID
    SelectRule(u32),    // Rule ID
    DuplicateRule(u32), // Rule ID
    SetRuleEnabled(u32, bool),

    // UI state (card grid)
    OpenEditor(u32),
    CloseEditor,
    SetSearch(String),
    SetFilter(RulesFilter),

    // Name
    SetRuleName(u32, String),

    // Match criteria
    AddMatch(u32),                                        // Rule ID
    RemoveMatch(u32, usize),                              // (rule_id, match_index)
    SetMatchAppId(u32, usize, Option<String>),            // (rule_id, match_index, value)
    SetMatchTitle(u32, usize, Option<String>),            // (rule_id, match_index, value)
    SetMatchIsFloating(u32, usize, Option<bool>),         // (rule_id, match_index, value)
    SetMatchIsFocused(u32, usize, Option<bool>),          // (rule_id, match_index, value)
    SetMatchIsActive(u32, usize, Option<bool>),           // (rule_id, match_index, value)
    SetMatchIsActiveInColumn(u32, usize, Option<bool>),   // (rule_id, match_index, value) v0.1.6+
    SetMatchIsWindowCastTarget(u32, usize, Option<bool>), // (rule_id, match_index, value) v25.02+
    SetMatchIsUrgent(u32, usize, Option<bool>),           // (rule_id, match_index, value) v25.05+
    SetMatchAtStartup(u32, usize, Option<bool>),          // (rule_id, match_index, value) v0.1.6+

    // Exclude criteria
    AddExclude(u32),                                        // Rule ID
    RemoveExclude(u32, usize),                              // (rule_id, exclude_index)
    SetExcludeAppId(u32, usize, Option<String>),            // (rule_id, exclude_index, value)
    SetExcludeTitle(u32, usize, Option<String>),            // (rule_id, exclude_index, value)
    SetExcludeIsFloating(u32, usize, Option<bool>),         // (rule_id, exclude_index, value)
    SetExcludeIsFocused(u32, usize, Option<bool>),          // (rule_id, exclude_index, value)
    SetExcludeIsActive(u32, usize, Option<bool>),           // (rule_id, exclude_index, value)
    SetExcludeIsActiveInColumn(u32, usize, Option<bool>),   // (rule_id, exclude_index, value)
    SetExcludeIsWindowCastTarget(u32, usize, Option<bool>), // (rule_id, exclude_index, value)
    SetExcludeIsUrgent(u32, usize, Option<bool>),           // (rule_id, exclude_index, value)
    SetExcludeAtStartup(u32, usize, Option<bool>),          // (rule_id, exclude_index, value)

    // Opening behavior
    SetOpenBehavior(u32, crate::config::models::OpenBehavior),
    SetOpenFocused(u32, Option<bool>),
    SetOpenOnOutput(u32, Option<String>),
    SetOpenOnWorkspace(u32, Option<String>),
    SetBlockScreencast(u32, bool),

    // Sizing
    SetDefaultColumnWidth(u32, Option<f32>),
    SetDefaultWindowHeight(u32, Option<f32>),
    SetMinWidth(u32, Option<i32>),
    SetMaxWidth(u32, Option<i32>),
    SetMinHeight(u32, Option<i32>),
    SetMaxHeight(u32, Option<i32>),

    // Styling
    SetFocusRingEnabled(u32, Option<bool>),
    SetFocusRingWidth(u32, Option<i32>),
    SetBorderEnabled(u32, Option<bool>),
    SetBorderWidth(u32, Option<i32>),
    SetOpacity(u32, Option<f32>),
    SetCornerRadius(u32, Option<i32>),
    SetClipToGeometry(u32, Option<bool>),
    SetDrawBorderWithBackground(u32, Option<bool>),

    // Layout
    SetDefaultColumnDisplay(u32, Option<crate::config::models::DefaultColumnDisplay>),
    SetOpenMaximizedToEdges(u32, Option<bool>),
    SetScrollFactor(u32, Option<f64>),

    // Color overrides (whole-value set)
    SetFocusRingActive(u32, Option<crate::types::ColorOrGradient>),
    SetFocusRingInactive(u32, Option<crate::types::ColorOrGradient>),
    SetFocusRingUrgent(u32, Option<crate::types::ColorOrGradient>),
    SetBorderActive(u32, Option<crate::types::ColorOrGradient>),
    SetBorderInactive(u32, Option<crate::types::ColorOrGradient>),
    SetBorderUrgent(u32, Option<crate::types::ColorOrGradient>),

    // Complex struct overrides
    SetShadow(u32, Option<crate::config::models::ShadowSettings>),
    SetTabIndicator(u32, Option<crate::config::models::TabIndicatorSettings>),
    SetDefaultFloatingPosition(u32, Option<crate::config::models::FloatingPosition>),

    // Advanced
    SetVariableRefreshRate(u32, Option<bool>),
    SetBabaIsFloat(u32, Option<bool>),
    SetTiledState(u32, Option<bool>),

    // UI state
    ToggleSection(u32, String),
}

/// Layer rules settings messages
///
/// Manages layer-shell surface rules: panels, docks, notifications, overlays
#[derive(Debug, Clone)]
pub enum LayerRulesMessage {
    // List management
    AddRule,
    DeleteRule(u32), // Rule ID
    SelectRule(u32), // Rule ID
    DuplicateRule(u32),
    ReorderRule(u32, bool), // (rule_id, move_up)
    SetRuleEnabled(u32, bool),

    // UI state (card grid)
    OpenEditor(u32),
    CloseEditor,
    SetSearch(String),
    SetFilter(RulesFilter),

    // Name
    SetRuleName(u32, String),

    // Match criteria
    AddMatch(u32),
    RemoveMatch(u32, usize), // (rule_id, match_index)
    SetMatchNamespace(u32, usize, String),
    SetMatchAtStartup(u32, usize, Option<bool>),

    // Properties
    SetBlockOutFrom(u32, Option<crate::config::models::BlockOutFrom>),
    SetOpacity(u32, Option<f32>),
    SetCornerRadius(u32, Option<i32>),
    SetPlaceWithinBackdrop(u32, bool),
    SetBabaIsFloat(u32, bool),

    // Shadow (nested)
    SetShadow(u32, Option<crate::config::models::ShadowSettings>),

    // UI state
    ToggleSection(u32, String),
    ValidateRegex(u32, usize, String, String), // (rule_id, match_index, field_name, regex)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYSTEM CONFIGURATION MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outputs (displays) settings messages
///
/// Manages monitors: resolution, scale, position, VRR, hot corners
#[derive(Debug, Clone)]
pub enum OutputsMessage {
    // List management
    AddOutput,
    RemoveOutput(usize),
    SelectOutput(usize),

    // Basic properties
    SetOutputName(usize, String),
    SetEnabled(usize, bool),
    SetScale(usize, f64),
    SetMode(usize, String),
    SetModeCustom(usize, bool),
    SetModeline(usize, Option<String>),
    SetPositionX(usize, i32),
    SetPositionY(usize, i32),
    SetTransform(usize, crate::types::Transform),
    SetVrr(usize, crate::types::VrrMode),
    SetFocusAtStartup(usize, bool),
    SetBackdropColor(usize, Option<crate::types::Color>),

    // Hot corners
    SetHotCornersEnabled(usize, Option<bool>),
    SetHotCornerTopLeft(usize, bool),
    SetHotCornerTopRight(usize, bool),
    SetHotCornerBottomLeft(usize, bool),
    SetHotCornerBottomRight(usize, bool),

    // Layout override (nested structure)
    SetLayoutOverride(usize, Option<crate::config::models::LayoutOverride>),

    // UI state
    ToggleSection(String),
    OpenEditor(usize),
    CloseEditor,
}

/// Keybindings settings messages
#[derive(Debug, Clone)]
pub enum KeybindingsMessage {
    // List management
    AddKeybinding,
    RemoveKeybinding(usize),
    SelectKeybinding(usize),

    // Key capture
    UpdateModifiers(usize, Vec<ModKey>),
    StartKeyCapture(usize),
    CapturedKey(String),
    CancelKeyCapture,

    // Action
    UpdateAction(usize, String),
    SetCommand(usize, String),

    // Advanced options
    SetAllowWhenLocked(usize, bool),
    SetRepeat(usize, bool),
    SetCooldown(usize, Option<i32>),
    SetHotkeyOverlayTitle(usize, Option<String>),

    // UI state
    ToggleSection(String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADVANCED SETTINGS MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Debug settings messages
///
/// Expert-only options: rendering, device config, performance, compatibility
#[derive(Debug, Clone)]
pub enum DebugMessage {
    // Expert mode
    SetExpertMode(bool),

    // Rendering options
    SetPreviewRender(crate::config::models::PreviewRenderMode),
    SetEnableOverlayPlanes(bool),
    SetDisableCursorPlane(bool),
    SetDisableDirectScanout(bool),
    SetRestrictPrimaryScanoutToMatchingFormat(bool),

    // Device configuration
    SetRenderDrmDevice(Option<String>),
    AddIgnoreDrmDevice(String),
    RemoveIgnoreDrmDevice(usize),

    // Performance & synchronization
    SetWaitForFrameCompletionBeforeQueueing(bool),
    SetDisableResizeThrottling(bool),
    SetDisableTransactions(bool),
    SetEmulateZeroPresentationTime(bool),
    SetSkipCursorOnlyUpdatesDuringVrr(bool),

    // Hardware & compatibility
    SetDbusInterfacesInNonSessionInstances(bool),
    SetKeepLaptopPanelOnWhenLidIsClosed(bool),
    SetDisableMonitorNames(bool),
    SetForceDisableConnectorsOnResume(bool),

    // Window behavior
    SetStrictNewWindowFocusPolicy(bool),
    SetHonorXdgActivationWithInvalidSerial(bool),
    SetDeactivateUnfocusedWindows(bool),

    // Screencasting
    SetForcePipewireInvalidModifier(bool),
}

/// Miscellaneous settings messages
#[derive(Debug, Clone)]
pub enum MiscellaneousMessage {
    SetPreferNoCsd(bool),
    SetScreenshotPath(String),
    SetDisablePrimaryClipboard(bool),
    SetHotkeyOverlaySkipAtStartup(bool),
    SetHotkeyOverlayHideNotBound(bool),
    SetConfigNotificationDisableFailed(bool),
    SetSpawnShAtStartup(String),
    SetXWaylandSatellite(crate::config::models::XWaylandSatelliteConfig),
}

/// Overview settings messages (workspace overview / exposé)
///
/// Controls the appearance of the workspace overview mode (triggered by toggle-overview)
#[derive(Debug, Clone)]
pub enum OverviewMessage {
    /// Set the overview zoom level (how much to scale down windows)
    SetZoom(f64),
    /// Set the backdrop color (optional)
    SetBackdropColor(Option<String>),
    /// Toggle workspace shadow in overview
    ToggleWorkspaceShadow(bool),
    /// Set workspace shadow softness (blur radius)
    SetWorkspaceShadowSoftness(i32),
    /// Set workspace shadow spread
    SetWorkspaceShadowSpread(i32),
    /// Set workspace shadow X offset
    SetWorkspaceShadowOffsetX(i32),
    /// Set workspace shadow Y offset
    SetWorkspaceShadowOffsetY(i32),
    /// Set workspace shadow color
    SetWorkspaceShadowColor(String),
}

/// Environment settings messages
#[derive(Debug, Clone)]
pub enum EnvironmentMessage {
    AddVariable,
    RemoveVariable(u32), // Variable ID
    SetVariableName(u32, String),
    SetVariableValue(u32, String),
}

/// Switch events settings messages
#[derive(Debug, Clone)]
pub enum SwitchEventsMessage {
    SetLidCloseCommand(String),
    SetLidOpenCommand(String),
    SetTabletModeOnCommand(String),
    SetTabletModeOffCommand(String),
}

/// Recent windows settings messages
#[derive(Debug, Clone)]
pub enum RecentWindowsMessage {
    // Top-level
    SetOff(bool),
    SetDebounceMs(i32),
    SetOpenDelayMs(i32),

    // Highlight settings
    SetActiveColor(String), // Hex color
    SetUrgentColor(String), // Hex color
    SetHighlightPadding(i32),
    SetHighlightCornerRadius(i32),

    // Preview settings
    SetPreviewMaxHeight(i32),
    SetPreviewMaxScale(f64),

    // Keybind management
    AddBind,
    RemoveBind(usize),
    SetBindKeyCombo(usize, String),
    SetBindIsNext(usize, bool),
    SetBindFilterAppId(usize, bool),
    SetBindScope(usize, Option<crate::config::models::RecentWindowsScope>),
    SetBindCooldown(usize, Option<i32>),
}

/// Trackpoint settings messages
#[derive(Debug, Clone)]
pub enum TrackpointMessage {
    SetOff(bool),
    SetNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollMethod(ScrollMethod),
    SetLeftHanded(bool),
    SetMiddleEmulation(bool),
    SetScrollButtonLock(bool),
    SetScrollButton(Option<i32>),
}

/// Trackball settings messages
#[derive(Debug, Clone)]
pub enum TrackballMessage {
    SetOff(bool),
    SetNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollMethod(ScrollMethod),
    SetLeftHanded(bool),
    SetMiddleEmulation(bool),
    SetScrollButtonLock(bool),
    SetScrollButton(Option<i32>),
}

/// Tablet settings messages
#[derive(Debug, Clone)]
pub enum TabletMessage {
    SetOff(bool),
    SetLeftHanded(bool),
    SetMapToOutput(String),
    SetCalibrationMatrix(Option<[f64; 6]>),
    // Calibration matrix individual value changes
    SetCalibrationValue(usize, String), // (index 0-5, value as string)
    ClearCalibration,
    ResetCalibration,
}

/// Touch screen settings messages
#[derive(Debug, Clone)]
pub enum TouchMessage {
    SetOff(bool),
    SetMapToOutput(String),
    SetCalibrationMatrix(Option<[f64; 6]>),
    // Calibration matrix individual value changes
    SetCalibrationValue(usize, String), // (index 0-5, value as string)
    ClearCalibration,
    ResetCalibration,
}

/// Gestures settings messages
#[derive(Debug, Clone)]
pub enum GesturesMessage {
    // Hot corners
    SetHotCornersEnabled(bool),
    SetHotCornerTopLeft(bool),
    SetHotCornerTopRight(bool),
    SetHotCornerBottomLeft(bool),
    SetHotCornerBottomRight(bool),

    // DnD edge view scroll
    SetDndScrollEnabled(bool),
    SetDndScrollTriggerWidth(i32),
    SetDndScrollDelayMs(i32),
    SetDndScrollMaxSpeed(i32),

    // DnD edge workspace switch
    SetDndWorkspaceEnabled(bool),
    SetDndWorkspaceTriggerHeight(i32),
    SetDndWorkspaceDelayMs(i32),
    SetDndWorkspaceMaxSpeed(i32),
}

/// Layout extras settings messages
#[derive(Debug, Clone)]
pub enum LayoutExtrasMessage {
    // Shadow settings
    SetShadowEnabled(bool),
    SetShadowSoftness(i32),
    SetShadowSpread(i32),
    SetShadowOffsetX(i32),
    SetShadowOffsetY(i32),
    SetShadowDrawBehindWindow(bool),
    SetShadowColor(String),
    SetShadowInactiveColor(String),

    // Tab indicator
    SetTabIndicatorEnabled(bool),
    SetTabIndicatorHideWhenSingleTab(bool),
    SetTabIndicatorPlaceWithinColumn(bool),
    SetTabIndicatorGap(i32),
    SetTabIndicatorWidth(i32),
    SetTabIndicatorLengthProportion(f32),
    SetTabIndicatorCornerRadius(i32),
    SetTabIndicatorGapsBetweenTabs(i32),
    SetTabIndicatorPosition(crate::config::models::TabIndicatorPosition),
    SetTabIndicatorActiveColor(String),
    SetTabIndicatorInactiveColor(String),
    SetTabIndicatorUrgentColor(String),

    // Insert hint
    SetInsertHintEnabled(bool),
    SetInsertHintColor(String),

    // Preset widths/heights
    AddPresetWidth,
    RemovePresetWidth(usize),
    SetPresetWidth(usize, crate::config::models::PresetWidth),
    AddPresetHeight,
    RemovePresetHeight(usize),
    SetPresetHeight(usize, crate::config::models::PresetHeight),

    // Default column display
    SetDefaultColumnDisplay(crate::config::models::DefaultColumnDisplay),
}

/// Startup commands messages
#[derive(Debug, Clone)]
pub enum StartupMessage {
    AddCommand,
    RemoveCommand(u32), // Command ID
    SetCommand(u32, String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP MANAGEMENT MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Tools page messages for IPC operations
///
/// Niri IPC queries and actions: windows, workspaces, outputs, config reload
#[derive(Debug, Clone)]
pub enum ToolsMessage {
    // Query actions
    RefreshWindows,
    RefreshWorkspaces,
    RefreshOutputs,
    RefreshFocusedWindow,
    RefreshVersion,

    // Action results (for async Task completion)
    WindowsLoaded(Result<Vec<crate::ipc::WindowInfo>, String>),
    WorkspacesLoaded(Result<Vec<crate::ipc::WorkspaceInfo>, String>),
    OutputsLoaded(Result<Vec<crate::ipc::FullOutputInfo>, String>),
    FocusedWindowLoaded(Result<Option<crate::ipc::WindowInfo>, String>),
    VersionLoaded(Result<String, String>),

    // IPC actions
    ReloadConfig,
    ValidateConfig,

    // Action results
    ReloadCompleted(Result<(), String>),
    ValidateCompleted(Result<String, String>),
}

/// App preferences messages
#[derive(Debug, Clone)]
pub enum PreferencesMessage {
    /// Toggle whether the settings app should float or tile
    SetFloatSettingsApp(bool),
    /// Toggle whether to show the search bar in navigation
    SetShowSearchBar(bool),
    /// Set the keyboard shortcut for opening search (e.g., "Ctrl+K", "Ctrl+/", or empty to disable)
    SetSearchHotkey(String),
}

/// Config editor messages
#[derive(Debug, Clone)]
pub enum ConfigEditorMessage {
    /// Select a file to view by index
    SelectFile(usize),
    /// Refresh the current file
    Refresh,
    /// File content loaded
    FileLoaded(Result<String, String>),
    /// Toggle edit mode on/off
    ToggleEditMode(bool),
    /// Editor action (edit, cursor movement, etc.)
    EditorAction(text_editor::Action),
    /// Save edited content to file
    SaveEdits,
    /// Discard changes and exit edit mode
    DiscardEdits,
    /// Save completed
    SaveCompleted(Result<(), String>),
}

/// Backups management messages
#[derive(Debug, Clone)]
pub enum BackupsMessage {
    /// Refresh the backup list
    RefreshList,
    /// Backup list loaded
    ListLoaded(Result<Vec<BackupEntry>, String>),
    /// Select a backup to preview
    SelectBackup(usize),
    /// Preview content loaded
    PreviewLoaded(Result<String, String>),
    /// Request to restore a backup
    RestoreBackup(usize),
    /// Show restore confirmation dialog
    ConfirmRestore(usize),
    /// Restore completed
    RestoreCompleted(Result<(), String>),
}

/// Entry in the backups list
#[derive(Debug, Clone)]
pub struct BackupEntry {
    /// Filename of the backup
    pub filename: String,
    /// Human-readable date
    pub date: String,
    /// Human-readable size
    pub size: String,
    /// Full path to the backup file
    pub path: std::path::PathBuf,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SAVE & PERSISTENCE MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Save subsystem messages
///
/// Periodic auto-save triggers from subscription
#[derive(Debug, Clone)]
pub enum SaveMessage {
    /// Periodic check if save is needed (from subscription)
    CheckSave,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIALOG & MODAL TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Dialog state - defines the content and behavior of modal dialogs
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DialogState {
    #[default]
    None,
    Error {
        title: String,
        message: String,
        details: Option<String>,
    },
    Confirm {
        title: String,
        message: String,
        confirm_label: String,
        on_confirm: ConfirmAction,
    },
    FirstRunWizard {
        step: WizardStep,
    },
    ImportSummary {
        imported_count: usize,
        defaulted_count: usize,
        warnings: Vec<String>,
    },
    Consolidation {
        suggestions: Vec<ConsolidationSuggestion>,
    },
    DiffView {
        title: String,
        before: String,
        after: String,
    },
}

/// First-run wizard steps
#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Welcome,
    ConfigSetup,
    ImportResults,
    Consolidation,
    Complete,
}

/// Actions that can be confirmed
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    DeleteRule(u32), // Rule ID
    ResetSettings,
    ClearAllKeybindings,
}

/// Consolidation suggestion for rules
#[derive(Debug, Clone, PartialEq)]
pub struct ConsolidationSuggestion {
    pub description: String,
    /// IDs of rules that can be merged
    pub rule_ids: Vec<u32>,
    /// Number of rules that can be merged
    pub rule_count: usize,
    /// Original patterns (app_ids or namespaces) being merged
    pub patterns: Vec<String>,
    /// The suggested merged regex pattern
    pub merged_pattern: String,
    /// Whether this is a window rule (true) or layer rule (false)
    pub is_window_rule: bool,
    /// Whether this suggestion is selected for merging
    pub selected: bool,
}
