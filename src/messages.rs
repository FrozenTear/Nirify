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

use crate::config::models::{ActionCategory, HotkeyOverlayTitle};
use crate::config::ColumnWidthType;
use crate::types::{
    AccelProfile, CenterFocusedColumn, ClickMethod, ModKey, ScrollMethod, TapButtonMap,
    WarpMouseMode,
};
use crate::views::widgets::GradientPickerMessage;

/// Root message enum - all possible application events
// Boxing large variants would ripple through every handler/view; churn outweighs the win.
#[allow(clippy::large_enum_variant)]
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
    Blur(BlurMessage),
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
    /// Dismiss the persistent error banner (any kind)
    DismissErrorBanner,
    /// Overwrite load-blocked categories with current in-memory values
    OverwriteFailedCategories,

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

    // ═══════════════════════════════════════════════════════════════════════════
    // UX safety (revert countdown, search nav, wizard skip)
    // ═══════════════════════════════════════════════════════════════════════════
    /// 1-second tick while a revert countdown is pending
    RevertTick,
    /// Keep the risky change (dismiss the revert countdown)
    RevertKeep,
    /// Revert the risky change now (also used on timeout / Escape)
    RevertNow,
    /// Move search selection up
    SearchNavUp,
    /// Move search selection down
    SearchNavDown,
    /// Activate the currently selected search result (Enter)
    SearchNavActivate,
    /// Escape pressed: close the topmost overlay layer
    EscapePressed,
    /// User acknowledged skipping first-run setup
    WizardSkipConfirmed,
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
    Blur,
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
    /// Every page variant, in declaration order.
    ///
    /// Used to drive the search index so that adding a `Page` variant forces a
    /// corresponding search entry (see `crate::search::page_entries`, which has
    /// a wildcard-free match). Bump `search::tests` count when editing this.
    pub const ALL: &'static [Page] = &[
        Page::Overview,
        Page::Appearance,
        Page::Behavior,
        Page::Keyboard,
        Page::Mouse,
        Page::Touchpad,
        Page::Trackpoint,
        Page::Trackball,
        Page::Tablet,
        Page::Touch,
        Page::Animations,
        Page::Cursor,
        Page::Blur,
        Page::LayoutExtras,
        Page::Gestures,
        Page::Workspaces,
        Page::WindowRules,
        Page::LayerRules,
        Page::Keybindings,
        Page::Outputs,
        Page::Miscellaneous,
        Page::Startup,
        Page::Environment,
        Page::Debug,
        Page::SwitchEvents,
        Page::RecentWindows,
        Page::Tools,
        Page::Preferences,
        Page::ConfigEditor,
        Page::Backups,
    ];

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
            Page::Blur => "Blur",
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
            Page::Animations | Page::Cursor | Page::Blur => PageCategory::Visual,
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
            Page::Overview => Screen::Visuals,
            Page::Appearance | Page::Animations | Page::Cursor | Page::Blur => Screen::Visuals,
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

    /// Every device variant (keeps search coverage exhaustive).
    pub const ALL: &'static [EditableDevice] = &[
        Self::Keyboard,
        Self::Mouse,
        Self::Touchpad,
        Self::Trackpoint,
        Self::Trackball,
        Self::Tablet,
        Self::Touch,
        Self::Gestures,
    ];

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

/// Sections that can be edited in a modal (Layout, Visuals, System, Dashboard)
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
    PresetSizes,
    // Visuals
    FocusRing,
    WindowBorder,
    WindowShadow,
    ModifierKeys,
    Animations,
    Cursor,
    Blur,
    WorkspaceBackground,
    // Dashboard
    Overview,
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
            Self::PresetSizes => "Preset Sizes",
            Self::FocusRing => "Focus Ring",
            Self::WindowBorder => "Window Border",
            Self::WindowShadow => "Window Shadow",
            Self::ModifierKeys => "Modifier Keys",
            Self::Animations => "Animations",
            Self::Cursor => "Cursor",
            Self::Blur => "Background Blur",
            Self::WorkspaceBackground => "Workspace Background",
            Self::Overview => "Workspace Overview",
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
            Self::PresetSizes => "▦",
            Self::FocusRing => "◉",
            Self::WindowBorder => "▭",
            Self::WindowShadow => "◌",
            Self::ModifierKeys => "⌥",
            Self::Animations => "◈",
            Self::Cursor => "↗",
            Self::Blur => "◍",
            Self::WorkspaceBackground => "▣",
            Self::Overview => "⧉",
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
            | Self::Blur
            | Self::Overview
            | Self::StartupPrograms => neon::PRIMARY,
            Self::ColumnManager
            | Self::TabIndicator
            | Self::PresetSizes
            | Self::WindowBorder
            | Self::ModifierKeys
            | Self::EnvironmentVars
            | Self::SwitchEvents => neon::SECONDARY,
            _ => neon::TERTIARY,
        }
    }

    /// Screen that hosts this section in the redesigned chrome
    pub fn screen(self) -> Screen {
        match self {
            Self::SpatialGaps
            | Self::CenteringDynamics
            | Self::ColumnManager
            | Self::ScreenEdgeStruts
            | Self::TabIndicator
            | Self::InsertHint
            | Self::NamedWorkspaces
            | Self::PresetSizes => Screen::Layout,
            Self::FocusRing
            | Self::WindowBorder
            | Self::WindowShadow
            | Self::ModifierKeys
            | Self::Animations
            | Self::Cursor
            | Self::Blur
            | Self::WorkspaceBackground
            | Self::Overview => Screen::Visuals,
            Self::StartupPrograms
            | Self::EnvironmentVars
            | Self::Miscellaneous
            | Self::SwitchEvents
            | Self::Debug
            | Self::RecentWindows => Screen::System,
        }
    }

    /// Sections shown as browse cards on a redesigned screen (no search required).
    pub fn cards_on(screen: Screen) -> &'static [EditableSection] {
        match screen {
            Screen::Layout => &[
                Self::SpatialGaps,
                Self::CenteringDynamics,
                Self::ColumnManager,
                Self::ScreenEdgeStruts,
                Self::TabIndicator,
                Self::InsertHint,
                Self::NamedWorkspaces,
                Self::PresetSizes,
            ],
            Screen::Visuals => &[
                Self::FocusRing,
                Self::WindowBorder,
                Self::WindowShadow,
                Self::ModifierKeys,
                Self::Animations,
                Self::Cursor,
                Self::Blur,
                Self::WorkspaceBackground,
                Self::Overview,
            ],
            Screen::System => &[
                Self::StartupPrograms,
                Self::EnvironmentVars,
                Self::Miscellaneous,
                Self::SwitchEvents,
                Self::Debug,
                Self::RecentWindows,
            ],
            Screen::Dashboard | Screen::Input | Screen::Rules | Screen::Displays | Screen::Gear => {
                &[]
            }
        }
    }

    /// Every section variant (keeps search coverage exhaustive).
    pub const ALL: &'static [EditableSection] = &[
        Self::SpatialGaps,
        Self::CenteringDynamics,
        Self::ColumnManager,
        Self::ScreenEdgeStruts,
        Self::TabIndicator,
        Self::InsertHint,
        Self::NamedWorkspaces,
        Self::PresetSizes,
        Self::FocusRing,
        Self::WindowBorder,
        Self::WindowShadow,
        Self::ModifierKeys,
        Self::Animations,
        Self::Cursor,
        Self::Blur,
        Self::WorkspaceBackground,
        Self::Overview,
        Self::StartupPrograms,
        Self::EnvironmentVars,
        Self::Miscellaneous,
        Self::SwitchEvents,
        Self::Debug,
        Self::RecentWindows,
    ];
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
    /// Live text of the exact scroll-factor entry (buffered so intermediate
    /// strings like "-", "1." and "" survive re-render).
    SetScrollFactorText(String),
    /// Commit/clamp the buffered scroll-factor entry (on submit/blur).
    CommitScrollFactor,
    SetScrollMethod(Option<ScrollMethod>),
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
    /// Live text of the exact scroll-factor entry (buffered so intermediate
    /// strings like "-", "1." and "" survive re-render).
    SetScrollFactorText(String),
    /// Commit/clamp the buffered scroll-factor entry (on submit/blur).
    CommitScrollFactor,
    SetScrollMethod(Option<ScrollMethod>),
    SetScrollButton(Option<i32>),
    ToggleScrollButtonLock(bool),
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
    SetAnimationBezier(String, f64, f64, f64, f64), // (animation_name, x1, y1, x2, y2)
    SetAnimationSpringDampingRatio(String, f32),
    SetAnimationSpringStiffness(String, i32),
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
    /// Replace a workspace's entire layout override (None removes it).
    /// Boxed to keep the message enum small (LayoutOverride is large).
    SetLayoutOverride(usize, Option<Box<crate::config::models::LayoutOverride>>),
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

    // Opening behavior (each independent tri-state; None = don't emit)
    SetOpenMaximized(u32, Option<bool>),
    SetOpenFullscreen(u32, Option<bool>),
    SetOpenFloating(u32, Option<bool>),
    SetOpenFocused(u32, Option<bool>),
    SetOpenOnOutput(u32, Option<String>),
    SetOpenOnWorkspace(u32, Option<String>),
    SetBlockOutFrom(u32, Option<crate::config::models::BlockOutFrom>),

    // Sizing
    SetDefaultColumnWidth(u32, Option<crate::config::models::RuleDefaultSize>),
    SetDefaultWindowHeight(u32, Option<crate::config::models::RuleDefaultSize>),
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
    SetCornerRadius(u32, Option<crate::config::models::CornerRadiusValue>),
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
    SetTabIndicator(u32, Option<crate::config::models::TabIndicatorOverride>),
    SetBackgroundEffect(u32, Option<crate::config::models::BackgroundEffectSettings>),
    SetPopups(u32, Option<crate::config::models::PopupsSettings>),
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
    SetMatchLayer(u32, usize, Option<crate::config::models::LayerKind>),

    // Exclude criteria
    AddExclude(u32),
    RemoveExclude(u32, usize), // (rule_id, exclude_index)
    SetExcludeNamespace(u32, usize, String),
    SetExcludeAtStartup(u32, usize, Option<bool>),
    SetExcludeLayer(u32, usize, Option<crate::config::models::LayerKind>),

    // Properties
    SetBlockOutFrom(u32, Option<crate::config::models::BlockOutFrom>),
    SetOpacity(u32, Option<f32>),
    SetCornerRadius(u32, Option<crate::config::models::CornerRadiusValue>),
    SetPlaceWithinBackdrop(u32, bool),
    SetBabaIsFloat(u32, bool),

    // Shadow (nested)
    SetShadow(u32, Option<crate::config::models::ShadowSettings>),
    SetBackgroundEffect(u32, Option<crate::config::models::BackgroundEffectSettings>),
    SetPopups(u32, Option<crate::config::models::PopupsSettings>),

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
// Boxing large variants would ripple through every handler/view; churn outweighs the win.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum OutputsMessage {
    // List management
    AddOutput,
    /// Create configs for every connected IPC output that is not already named.
    AdoptConnected,
    RemoveOutput(usize),
    SelectOutput(usize),

    // Arrangement canvas (pointer is in canvas-local pixels)
    CanvasMove(f32, f32),
    CanvasPress,
    CanvasRelease,

    // Basic properties
    SetOutputName(usize, String),
    SetEnabled(usize, bool),
    SetScale(usize, f64),
    SetMode(usize, String),
    SetModeCustom(usize, bool),
    SetModeline(usize, Option<String>),
    SetPositionX(usize, i32),
    SetPositionY(usize, i32),
    /// true = automatic placement (position None); false = explicit (Some)
    SetPositionAuto(usize, bool),
    /// Snapshot the live niri output layout into managed settings.
    ///
    /// Fetches `Outputs` over IPC, then applies
    /// [`crate::config::apply_live_outputs_to_settings`]. UI (Tools / Displays)
    /// can fire this without implementing the merge policy.
    ImportConnectedLayout,
    /// Result of the live-layout IPC fetch started by [`Self::ImportConnectedLayout`].
    LiveOutputsSnapshotLoaded(Result<Vec<crate::ipc::FullOutputInfo>, String>),
    SetTransform(usize, crate::types::Transform),
    SetVrr(usize, crate::types::VrrMode),
    SetFocusAtStartup(usize, bool),
    SetBackgroundColor(usize, Option<crate::types::Color>),
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
    SelectActionCategory(usize, ActionCategory),
    UpdateAction(usize, String),
    SetCommand(usize, String),
    SetSpawnShCommand(usize, String),
    SetCustomActionText(usize, String),

    // Typed action arguments
    SetActionArgText(usize, String),
    SetActionFocusFlag(usize, bool),
    SetActionSkipConfirmation(usize, bool),
    SetActionDelayMs(usize, Option<u16>),
    SetActionWriteToDisk(usize, bool),
    SetActionShowPointer(usize, bool),

    // Advanced options
    SetAllowWhenLocked(usize, bool),
    SetAllowInhibiting(usize, bool),
    SetRepeat(usize, bool),
    SetCooldown(usize, Option<i32>),
    SetHotkeyOverlayTitle(usize, HotkeyOverlayTitle),

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
    SetScreenshotPath(crate::config::models::ScreenshotPathConfig),
    SetDisablePrimaryClipboard(bool),
    SetHotkeyOverlaySkipAtStartup(bool),
    SetHotkeyOverlayHideNotBound(bool),
    SetConfigNotificationDisableFailed(bool),
    AddSpawnShAtStartup,
    RemoveSpawnShAtStartup(u32),
    SetSpawnShAtStartup(u32, String),
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
    /// true = unset the variable (value None); false = set to empty string
    SetVariableUnset(u32, bool),
}

/// Switch events settings messages
#[derive(Debug, Clone)]
pub enum SwitchEventsMessage {
    SetLidCloseCommand(String),
    SetLidOpenCommand(String),
    SetTabletModeOnCommand(String),
    SetTabletModeOffCommand(String),
}

/// Top-level background blur messages (niri 26.04+)
#[derive(Debug, Clone)]
pub enum BlurMessage {
    SetEnabled(bool),
    SetPasses(i32),
    SetOffset(f64),
    SetNoise(f64),
    SetSaturation(f64),
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
    SetScrollMethod(Option<ScrollMethod>),
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
    SetScrollMethod(Option<ScrollMethod>),
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
    SetMapToFocusedOutput(bool),
    SetMapToFocusedWindow(bool),
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
    SetTabIndicatorActiveColor(GradientPickerMessage),
    SetTabIndicatorInactiveColor(GradientPickerMessage),
    SetTabIndicatorUrgentColor(GradientPickerMessage),

    // Custom-color opt-ins: when off, niri falls back to focus-ring colors
    SetShadowUseInactiveColor(bool),
    SetTabIndicatorUseActiveColor(bool),
    SetTabIndicatorUseInactiveColor(bool),
    SetTabIndicatorUseUrgentColor(bool),

    // Insert hint
    SetInsertHintEnabled(bool),
    SetInsertHintColor(GradientPickerMessage),

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
    /// Restore completed (Ok carries which target was restored)
    RestoreCompleted(Result<RestoredTarget, String>),
}

/// Which config file a restore operation targeted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestoredTarget {
    /// The user's main niri config.kdl
    NiriConfig,
    /// A managed category file under the niri-settings directory
    Managed,
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
    /// Apply-then-confirm countdown after a risky live-applied change.
    RevertCountdown {
        description: String,
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
    /// Confirmation shown when the user tries to skip setup before the
    /// niri include line is present.
    SkipWarning,
}

/// Actions that can be confirmed
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    /// Delete a window rule by its rule id
    DeleteWindowRule(u32),
    /// Delete a layer rule by its rule id
    DeleteLayerRule(u32),
    /// Delete a keybinding by its index in keybindings.bindings
    DeleteKeybinding(usize),
    /// Delete an output by its index in outputs.outputs
    DeleteOutput(usize),
    /// Delete a workspace by its index in workspaces.workspaces
    DeleteWorkspace(usize),
    ResetSettings,
    ClearAllKeybindings,
    /// Restore a backup by its index in the backups list
    RestoreBackup(usize),
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

#[cfg(test)]
mod tests {
    use super::{EditableSection, Screen};

    #[test]
    fn test_every_section_has_a_browse_card() {
        for section in EditableSection::ALL {
            let screen = section.screen();
            let cards = EditableSection::cards_on(screen);
            assert!(
                cards.contains(section),
                "{:?} maps to {:?} but has no browse card",
                section,
                screen
            );
        }
    }

    #[test]
    fn test_slice2_orphans_are_first_class_cards() {
        assert_eq!(EditableSection::Overview.screen(), Screen::Visuals);
        assert!(EditableSection::cards_on(Screen::Visuals).contains(&EditableSection::Overview));
        assert!(EditableSection::cards_on(Screen::Visuals)
            .contains(&EditableSection::WorkspaceBackground));
        assert!(EditableSection::cards_on(Screen::Layout).contains(&EditableSection::PresetSizes));
    }
}
