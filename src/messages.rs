//! Message types for the Elm Architecture
//!
//! This module defines all possible events/actions in the application.
//! Messages flow: User interaction → Message → update() → State change → view()

use crate::types::{AccelProfile, CenterFocusedColumn, ClickMethod, ModKey, ScrollMethod, TapButtonMap, WarpMouseMode};
use crate::config::ColumnWidthType;
use crate::views::widgets::GradientPickerMessage;

/// Root message enum - all possible application events
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateToPage(Page),
    ToggleSidebar,

    // Search
    SearchQueryChanged(String),
    SearchResultSelected(usize),
    ClearSearch,

    // Theme
    ChangeTheme(crate::theme::AppTheme),

    // Settings categories (nested enums)
    Appearance(AppearanceMessage),
    Behavior(BehaviorMessage),
    Keyboard(KeyboardMessage),
    Mouse(MouseMessage),
    Touchpad(TouchpadMessage),
    Animations(AnimationsMessage),
    Cursor(CursorMessage),
    Workspaces(WorkspacesMessage),
    WindowRules(WindowRulesMessage),
    LayerRules(LayerRulesMessage),
    Outputs(OutputsMessage),
    Keybindings(KeybindingsMessage),
    Debug(DebugMessage),
    Miscellaneous(MiscellaneousMessage),
    Environment(EnvironmentMessage),
    SwitchEvents(SwitchEventsMessage),
    RecentWindows(RecentWindowsMessage),
    Trackpoint(TrackpointMessage),
    Trackball(TrackballMessage),
    Tablet(TabletMessage),
    Touch(TouchMessage),
    Gestures(GesturesMessage),
    LayoutExtras(LayoutExtrasMessage),
    Startup(StartupMessage),

    // Save subsystem
    Save(SaveMessage),
    SaveCompleted(crate::save_manager::SaveResult),
    ReloadCompleted(crate::save_manager::ReloadResult),

    // Dialogs
    ShowDialog(DialogState),
    CloseDialog,
    DialogConfirm,
    WizardNext,
    WizardBack,
    WizardSetupConfig,
    ConsolidationApply,

    // System
    WindowCloseRequested,
    CheckNiriStatus,
    ClearToast,
    None, // No-op message
}

/// Page navigation enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
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
        }
    }

    /// Returns the category group for sidebar organization
    pub fn category(&self) -> PageCategory {
        match self {
            Page::Overview => PageCategory::System,
            Page::Appearance => PageCategory::Visual,
            Page::Behavior => PageCategory::Visual,
            Page::Keyboard | Page::Mouse | Page::Touchpad |
            Page::Trackpoint | Page::Trackball | Page::Tablet |
            Page::Touch => PageCategory::Input,
            Page::Animations | Page::Cursor => PageCategory::Visual,
            Page::LayoutExtras | Page::Gestures | Page::Workspaces => PageCategory::Layout,
            Page::WindowRules | Page::LayerRules => PageCategory::Rules,
            Page::Keybindings => PageCategory::Input,
            Page::Outputs => PageCategory::System,
            Page::Miscellaneous | Page::Startup | Page::Environment => PageCategory::System,
            Page::Debug | Page::SwitchEvents | Page::RecentWindows => PageCategory::Advanced,
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

/// Appearance settings messages
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

/// Behavior settings messages
#[derive(Debug, Clone)]
pub enum BehaviorMessage {
    // Focus
    ToggleFocusFollowsMouse(bool),
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

/// Keyboard settings messages
#[derive(Debug, Clone)]
pub enum KeyboardMessage {
    SetXkbLayout(String),
    SetXkbVariant(String),
    SetXkbOptions(String),
    SetXkbModel(String),
    SetRepeatDelay(i32),
    SetRepeatRate(i32),
    SetTrackLayout(String),
}

/// Mouse settings messages
#[derive(Debug, Clone)]
pub enum MouseMessage {
    ToggleOffOnTouchpad(bool),
    ToggleNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollFactor(f32),
    SetScrollMethod(ScrollMethod),
    ToggleLeftHanded(bool),
    ToggleMiddleEmulation(bool),
    ToggleScrollButtonLock(bool),
}

/// Touchpad settings messages
#[derive(Debug, Clone)]
pub enum TouchpadMessage {
    ToggleTapToClick(bool),
    ToggleDwt(bool),
    ToggleDwtp(bool),
    ToggleNaturalScroll(bool),
    SetAccelSpeed(f32),
    SetAccelProfile(AccelProfile),
    SetScrollFactor(f32),
    SetScrollMethod(ScrollMethod),
    SetClickMethod(ClickMethod),
    SetTapButtonMap(TapButtonMap),
    ToggleLeftHanded(bool),
    ToggleDrag(bool),
    ToggleDragLock(bool),
    ToggleMiddleEmulation(bool),
    ToggleDisabledOnExternalMouse(bool),
}

/// Animations settings messages (model-driven, more complex)
#[derive(Debug, Clone)]
pub enum AnimationsMessage {
    ToggleSlowdown(bool),
    SetSlowdownFactor(f32),

    // Per-animation messages (20 animation types)
    SetAnimationEnabled(String, bool), // (animation_name, enabled)
    SetAnimationDuration(String, i32),  // (animation_name, duration_ms)
    SetAnimationCurve(String, String),  // (animation_name, curve_name)
    SetAnimationSpringDampingRatio(String, f32),
    SetAnimationSpringEpsilon(String, f32),
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

/// Window rules settings messages (complex list + detail view)
#[derive(Debug, Clone)]
pub enum WindowRulesMessage {
    // List management
    AddRule,
    DeleteRule(u32),      // Rule ID
    SelectRule(u32),      // Rule ID
    DuplicateRule(u32),   // Rule ID

    // Name
    SetRuleName(u32, String),

    // Match criteria
    AddMatch(u32),                                    // Rule ID
    RemoveMatch(u32, usize),                          // (rule_id, match_index)
    SetMatchAppId(u32, usize, Option<String>),        // (rule_id, match_index, value)
    SetMatchTitle(u32, usize, Option<String>),        // (rule_id, match_index, value)
    SetMatchIsFloating(u32, usize, Option<bool>),     // (rule_id, match_index, value)
    SetMatchIsFocused(u32, usize, Option<bool>),      // (rule_id, match_index, value)

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
    SetOpacity(u32, Option<f32>),
    SetCornerRadius(u32, Option<i32>),
    SetClipToGeometry(u32, Option<bool>),
    SetDrawBorderWithBackground(u32, Option<bool>),

    // Advanced
    SetVariableRefreshRate(u32, Option<bool>),
    SetBabaIsFloat(u32, Option<bool>),
    SetTiledState(u32, Option<bool>),

    // UI state
    ToggleSection(u32, String),
}

/// Layer rules settings messages
#[derive(Debug, Clone)]
pub enum LayerRulesMessage {
    // List management
    AddRule,
    DeleteRule(u32), // Rule ID
    SelectRule(u32), // Rule ID
    DuplicateRule(u32),
    ReorderRule(u32, bool), // (rule_id, move_up)

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

/// Outputs (displays) settings messages
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

/// Debug settings messages
#[derive(Debug, Clone)]
pub enum DebugMessage {
    // Expert mode
    SetExpertMode(bool),

    // Rendering options
    SetPreviewRender(bool),
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
    SetSpawnShAtStartup(bool),
    SetXWaylandSatellite(crate::config::models::XWaylandSatelliteConfig),
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
    SetActiveColor(String),  // Hex color
    SetUrgentColor(String),  // Hex color
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
}

/// Touch screen settings messages
#[derive(Debug, Clone)]
pub enum TouchMessage {
    SetOff(bool),
    SetMapToOutput(String),
    SetCalibrationMatrix(Option<[f64; 6]>),
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

/// Save subsystem messages
#[derive(Debug, Clone)]
pub enum SaveMessage {
    /// Periodic check if save is needed (from subscription)
    CheckSave,
}

/// Dialog state
#[derive(Debug, Clone, PartialEq)]
pub enum DialogState {
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

impl Default for DialogState {
    fn default() -> Self {
        Self::None
    }
}

/// First-run wizard steps
#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Welcome,
    ConfigSetup,
    ImportResults,
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
    pub rule_count: usize,
    pub patterns: Vec<String>,
    pub merged_pattern: String,
    pub selected: bool,
}
