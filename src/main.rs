//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point. Uses Dioxus with the Blitz native renderer.

use anyhow::Result;
use dioxus_native::prelude::*;
use log::{error, info, warn};
use std::sync::{LazyLock, Mutex, RwLock};

use niri_settings::config;
use niri_settings::config::models::{
    AnimationSettings, AppearanceSettings, BehaviorSettings, CursorSettings, DebugSettings,
    EnvironmentSettings, EnvironmentVariable, GestureSettings, Keybinding, KeybindAction,
    KeyboardSettings, KeybindingsSettings, LayerRule, LayerRuleMatch, LayerRulesSettings,
    LayoutExtrasSettings, MiscSettings, MouseSettings, NamedWorkspace, OutputConfig,
    OutputSettings, OverviewSettings, RecentWindowsSettings, Settings, StartupCommand,
    StartupSettings, SwitchEventsSettings, TabletSettings, TouchSettings, TouchpadSettings,
    TrackballSettings, TrackpointSettings, WindowRule, WindowRuleMatch, WindowRulesSettings,
    WorkspacesSettings,
};
use niri_settings::ipc::get_full_outputs;
use niri_settings::types::{Transform, VrrMode};
use niri_settings::constants::*;
use niri_settings::ipc;
use niri_settings::types::{Color, ColorOrGradient};

/// Global config paths - initialized once at startup
static CONFIG_PATHS: LazyLock<config::ConfigPaths> = LazyLock::new(|| {
    config::ConfigPaths::new().expect("Failed to initialize config paths")
});

/// Global settings - loaded once at startup, updated on changes
static SETTINGS: LazyLock<RwLock<Settings>> = LazyLock::new(|| {
    let paths = &*CONFIG_PATHS;
    let is_first_run = paths.is_first_run();

    let loaded_settings = if is_first_run {
        info!("First run - importing settings from existing niri config");
        let result = config::import_from_niri_config_with_result(&paths.niri_config);
        info!("Import result: {}", result.summary());

        if let Err(e) = config::save_settings(paths, &result.settings) {
            error!("Failed to save imported settings: {}", e);
        }

        result.settings
    } else {
        let load_result = config::load_settings_with_result(paths);
        info!("{}", load_result.summary());
        load_result.settings
    };

    RwLock::new(loaded_settings)
});

/// Rate limiter for IPC reload requests
static RELOAD_PENDING: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

fn main() -> Result<()> {
    init_logging();

    // Force initialization of static settings
    let settings = SETTINGS.read().unwrap();
    info!(
        "Starting niri-settings ({} outputs, {} window rules, {} keybindings)",
        settings.outputs.outputs.len(),
        settings.window_rules.rules.len(),
        settings.keybindings.bindings.len()
    );
    drop(settings);

    // Launch Dioxus app
    dioxus_native::launch(App);

    Ok(())
}

/// Initialize logging with env_logger
fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

/// Save settings to disk and reload niri config
fn save_and_reload() {
    let paths = &*CONFIG_PATHS;
    let settings = SETTINGS.read().unwrap();

    if let Err(e) = config::save_settings(paths, &settings) {
        error!("Failed to save settings: {}", e);
        return;
    }

    drop(settings);

    // Rate-limit IPC reload calls
    if let Ok(mut pending) = RELOAD_PENDING.try_lock() {
        if !*pending {
            *pending = true;
            // Spawn reload in background to not block UI
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Err(e) = ipc::reload_config() {
                    warn!("Failed to reload niri config: {}", e);
                }
                if let Ok(mut pending) = RELOAD_PENDING.lock() {
                    *pending = false;
                }
            });
        }
    }
}

// ============================================================================
// Navigation
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    // Appearance group
    Appearance,
    Cursor,
    // Input group
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
    // Visuals group
    Animations,
    Overview,
    RecentWindows,
    // Layout group
    Layout,
    LayoutExtras,
    Workspaces,
    // Rules group
    WindowRules,
    LayerRules,
    Gestures,
    // System group
    Outputs,
    Keybindings,
    Startup,
    Environment,
    SwitchEvents,
    Tools,
    Backups,
    Miscellaneous,
    Debug,
}

impl Category {
    fn label(&self) -> &'static str {
        match self {
            Category::Appearance => "Windows",
            Category::Cursor => "Cursor",
            Category::Keyboard => "Keyboard",
            Category::Mouse => "Mouse",
            Category::Touchpad => "Touchpad",
            Category::Trackpoint => "Trackpoint",
            Category::Trackball => "Trackball",
            Category::Tablet => "Tablet",
            Category::Touch => "Touch",
            Category::Animations => "Animations",
            Category::Overview => "Overview",
            Category::RecentWindows => "Recent Windows",
            Category::Layout => "Gaps",
            Category::LayoutExtras => "Extras",
            Category::Workspaces => "Workspaces",
            Category::WindowRules => "Windows",
            Category::LayerRules => "Layers",
            Category::Gestures => "Gestures",
            Category::Outputs => "Displays",
            Category::Keybindings => "Keybindings",
            Category::Startup => "Startup",
            Category::Environment => "Environment",
            Category::SwitchEvents => "Switch Events",
            Category::Tools => "Tools",
            Category::Backups => "Backups",
            Category::Miscellaneous => "Miscellaneous",
            Category::Debug => "Debug",
        }
    }
}

/// Primary navigation groups (top bar tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NavGroup {
    Appearance,
    Input,
    Visuals,
    Layout,
    Rules,
    System,
}

impl NavGroup {
    fn label(&self) -> &'static str {
        match self {
            NavGroup::Appearance => "Appearance",
            NavGroup::Input => "Input",
            NavGroup::Visuals => "Visuals",
            NavGroup::Layout => "Layout",
            NavGroup::Rules => "Rules",
            NavGroup::System => "System",
        }
    }

    fn categories(&self) -> &'static [Category] {
        match self {
            NavGroup::Appearance => &[Category::Appearance, Category::Cursor],
            NavGroup::Input => &[
                Category::Keyboard,
                Category::Mouse,
                Category::Touchpad,
                Category::Trackpoint,
                Category::Trackball,
                Category::Tablet,
                Category::Touch,
            ],
            NavGroup::Visuals => &[
                Category::Animations,
                Category::Overview,
                Category::RecentWindows,
            ],
            NavGroup::Layout => &[
                Category::Layout,
                Category::LayoutExtras,
                Category::Workspaces,
            ],
            NavGroup::Rules => &[
                Category::WindowRules,
                Category::LayerRules,
                Category::Gestures,
            ],
            NavGroup::System => &[
                Category::Outputs,
                Category::Keybindings,
                Category::Startup,
                Category::Environment,
                Category::SwitchEvents,
                Category::Tools,
                Category::Backups,
                Category::Miscellaneous,
                Category::Debug,
            ],
        }
    }

    fn default_category(&self) -> Category {
        self.categories()[0]
    }

    fn all() -> &'static [NavGroup] {
        &[
            NavGroup::Appearance,
            NavGroup::Input,
            NavGroup::Visuals,
            NavGroup::Layout,
            NavGroup::Rules,
            NavGroup::System,
        ]
    }

    fn for_category(cat: Category) -> NavGroup {
        for group in Self::all() {
            if group.categories().contains(&cat) {
                return *group;
            }
        }
        NavGroup::Appearance
    }
}

// ============================================================================
// Custom Dropdown Component (select elements not supported in Blitz)
// Renders inline and pushes content down (z-index doesn't work in Blitz)
// ============================================================================

/// A single option for the Dropdown component
#[derive(Clone, PartialEq)]
struct DropdownOption {
    value: String,
    label: String,
}

/// Custom dropdown component using buttons (since <select> is not supported in Blitz)
#[component]
fn Dropdown(
    options: Vec<DropdownOption>,
    selected_value: String,
    on_change: EventHandler<String>,
) -> Element {
    let mut is_open = use_signal(|| false);

    let selected_label = options
        .iter()
        .find(|o| o.value == selected_value)
        .map(|o| o.label.clone())
        .unwrap_or_else(|| selected_value.clone());

    rsx! {
        div { class: "dropdown",
            button {
                class: if is_open() { "dropdown-trigger open" } else { "dropdown-trigger" },
                onclick: move |_| { is_open.set(!is_open()); },
                "{selected_label}"
                span { class: "dropdown-arrow", if is_open() { "▲" } else { "▼" } }
            }
            if is_open() {
                div { class: "dropdown-menu",
                    for opt in options.iter() {
                        button {
                            class: if opt.value == selected_value { "dropdown-item selected" } else { "dropdown-item" },
                            onclick: {
                                let value = opt.value.clone();
                                move |_| {
                                    on_change.call(value.clone());
                                    is_open.set(false);
                                }
                            },
                            "{opt.label}"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Root App
// ============================================================================

#[component]
fn App() -> Element {
    let selected = use_signal(|| Category::Appearance);
    let search_query = use_signal(|| String::new());

    // Initialize all settings signals from the loaded config
    let s = SETTINGS.read().unwrap();
    let appearance = use_signal(|| s.appearance.clone());
    let behavior = use_signal(|| s.behavior.clone());
    let keyboard = use_signal(|| s.keyboard.clone());
    let mouse = use_signal(|| s.mouse.clone());
    let touchpad = use_signal(|| s.touchpad.clone());
    let trackpoint = use_signal(|| s.trackpoint.clone());
    let trackball = use_signal(|| s.trackball.clone());
    let tablet = use_signal(|| s.tablet.clone());
    let touch = use_signal(|| s.touch.clone());
    let outputs = use_signal(|| s.outputs.clone());
    let animations = use_signal(|| s.animations.clone());
    let cursor = use_signal(|| s.cursor.clone());
    let overview = use_signal(|| s.overview.clone());
    let recent_windows = use_signal(|| s.recent_windows.clone());
    let layout_extras = use_signal(|| s.layout_extras.clone());
    let workspaces = use_signal(|| s.workspaces.clone());
    let window_rules = use_signal(|| s.window_rules.clone());
    let layer_rules = use_signal(|| s.layer_rules.clone());
    let gestures = use_signal(|| s.gestures.clone());
    let keybindings = use_signal(|| s.keybindings.clone());
    let startup = use_signal(|| s.startup.clone());
    let environment = use_signal(|| s.environment.clone());
    let switch_events = use_signal(|| s.switch_events.clone());
    let miscellaneous = use_signal(|| s.miscellaneous.clone());
    let debug = use_signal(|| s.debug.clone());
    drop(s);

    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            Header { selected, search_query }
            main { class: "content",
                PageContent {
                    selected: selected(),
                    appearance, behavior, keyboard, mouse, touchpad, trackpoint, trackball,
                    tablet, touch, outputs, animations, cursor, overview, recent_windows,
                    layout_extras, workspaces, window_rules, layer_rules, gestures,
                    keybindings, startup, environment, switch_events, miscellaneous, debug,
                }
            }
            Footer {}
        }
    }
}

#[component]
fn Header(selected: Signal<Category>, search_query: Signal<String>) -> Element {
    let current_group = NavGroup::for_category(selected());

    rsx! {
        header { class: "header",
            // App title
            div { class: "header-title", "niri settings" }

            // Primary navigation (groups)
            nav { class: "nav-primary",
                for group in NavGroup::all() {
                    button {
                        class: if current_group == *group { "nav-tab active" } else { "nav-tab" },
                        onclick: move |_| selected.set(group.default_category()),
                        "{group.label()}"
                    }
                }
            }

            // Secondary navigation (categories within group)
            nav { class: "nav-secondary",
                for category in current_group.categories() {
                    button {
                        class: if selected() == *category { "nav-subtab active" } else { "nav-subtab" },
                        onclick: move |_| selected.set(*category),
                        "{category.label()}"
                    }
                }
            }

            // Search bar row
            div { class: "search-row",
                div { class: "search-container",
                    span { class: "search-icon", "⌕" }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Search settings...",
                        value: "{search_query()}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer { class: "footer",
            span { class: "footer-status", "Changes saved automatically" }
        }
    }
}

#[component]
fn PageContent(
    selected: Category,
    appearance: Signal<AppearanceSettings>,
    behavior: Signal<BehaviorSettings>,
    keyboard: Signal<KeyboardSettings>,
    mouse: Signal<MouseSettings>,
    touchpad: Signal<TouchpadSettings>,
    trackpoint: Signal<TrackpointSettings>,
    trackball: Signal<TrackballSettings>,
    tablet: Signal<TabletSettings>,
    touch: Signal<TouchSettings>,
    outputs: Signal<OutputSettings>,
    animations: Signal<AnimationSettings>,
    cursor: Signal<CursorSettings>,
    overview: Signal<OverviewSettings>,
    recent_windows: Signal<RecentWindowsSettings>,
    layout_extras: Signal<LayoutExtrasSettings>,
    workspaces: Signal<WorkspacesSettings>,
    window_rules: Signal<WindowRulesSettings>,
    layer_rules: Signal<LayerRulesSettings>,
    gestures: Signal<GestureSettings>,
    keybindings: Signal<KeybindingsSettings>,
    startup: Signal<StartupSettings>,
    environment: Signal<EnvironmentSettings>,
    switch_events: Signal<SwitchEventsSettings>,
    miscellaneous: Signal<MiscSettings>,
    debug: Signal<DebugSettings>,
) -> Element {
    match selected {
        Category::Appearance => rsx! { AppearancePage { settings: appearance } },
        Category::Keyboard => rsx! { KeyboardPage { settings: keyboard } },
        Category::Mouse => rsx! { MousePage { settings: mouse } },
        Category::Touchpad => rsx! { TouchpadPage { settings: touchpad } },
        Category::Trackpoint => rsx! { TrackpointPage { settings: trackpoint } },
        Category::Trackball => rsx! { TrackballPage { settings: trackball } },
        Category::Tablet => rsx! { TabletPage { settings: tablet } },
        Category::Touch => rsx! { TouchPage { settings: touch } },
        Category::Outputs => rsx! { OutputsPage { settings: outputs } },
        Category::Animations => rsx! { AnimationsPage { settings: animations } },
        Category::Cursor => rsx! { CursorPage { settings: cursor } },
        Category::Overview => rsx! { OverviewPage { settings: overview } },
        Category::RecentWindows => rsx! { RecentWindowsPage { settings: recent_windows } },
        Category::Layout => rsx! { LayoutPage { behavior: behavior, appearance: appearance } },
        Category::LayoutExtras => rsx! { LayoutExtrasPage { settings: layout_extras } },
        Category::Workspaces => rsx! { WorkspacesPage { settings: workspaces } },
        Category::WindowRules => rsx! { WindowRulesPage { settings: window_rules } },
        Category::LayerRules => rsx! { LayerRulesPage { settings: layer_rules } },
        Category::Gestures => rsx! { GesturesPage { settings: gestures } },
        Category::Keybindings => rsx! { KeybindingsPage { settings: keybindings } },
        Category::Startup => rsx! { StartupPage { settings: startup } },
        Category::Environment => rsx! { EnvironmentPage { settings: environment } },
        Category::SwitchEvents => rsx! { SwitchEventsPage { settings: switch_events } },
        Category::Tools => rsx! { ToolsPage {} },
        Category::Backups => rsx! { BackupsPage {} },
        Category::Miscellaneous => rsx! { MiscellaneousPage { settings: miscellaneous } },
        Category::Debug => rsx! { DebugPage { settings: debug } },
    }
}

// ============================================================================
// Reusable Components
// ============================================================================

#[component]
fn Section(title: &'static str, children: Element) -> Element {
    rsx! {
        section { class: "settings-section",
            h2 { class: "section-title", "{title}" }
            div { class: "section-content", {children} }
        }
    }
}

#[component]
fn ToggleRow(label: &'static str, description: Option<&'static str>, value: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
                if let Some(desc) = description {
                    span { class: "setting-description", "{desc}" }
                }
            }
            button {
                class: if value { "toggle-btn on" } else { "toggle-btn off" },
                onclick: move |_| on_change.call(!value),
                if value { "On" } else { "Off" }
            }
        }
    }
}

#[component]
fn SliderRow(
    label: &'static str,
    description: Option<&'static str>,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    unit: &'static str,
    on_change: EventHandler<f32>,
) -> Element {
    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
                if let Some(desc) = description {
                    span { class: "setting-description", "{desc}" }
                }
            }
            div { class: "slider-control",
                button {
                    class: "slider-btn",
                    onclick: move |_| {
                        let new_val = (value - step).max(min);
                        on_change.call(new_val);
                    },
                    "-"
                }
                span { class: "slider-value", "{value:.0}{unit}" }
                button {
                    class: "slider-btn",
                    onclick: move |_| {
                        let new_val = (value + step).min(max);
                        on_change.call(new_val);
                    },
                    "+"
                }
            }
        }
    }
}

#[component]
fn ColorRow(
    label: &'static str,
    description: Option<&'static str>,
    color: ColorOrGradient,
    on_change: EventHandler<Color>,
) -> Element {
    let hex = color.to_hex();
    // Ensure hex is 7 chars for color input (strip alpha if present)
    let input_hex = if hex.len() > 7 { &hex[0..7] } else { &hex };

    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
                if let Some(desc) = description {
                    span { class: "setting-description", "{desc}" }
                }
            }
            div { class: "color-picker",
                input {
                    r#type: "color",
                    class: "color-input",
                    value: "{input_hex}",
                    oninput: move |e| {
                        if let Some(new_color) = Color::from_hex(&e.value()) {
                            on_change.call(new_color);
                        }
                    }
                }
                span { class: "color-hex", "{hex}" }
            }
        }
    }
}

#[component]
fn OptionalColorRow(
    label: &'static str,
    description: Option<&'static str>,
    color: Option<Color>,
    on_change: EventHandler<Option<Color>>,
) -> Element {
    let has_color = color.is_some();
    let hex = color.as_ref().map(|c| c.to_hex()).unwrap_or_else(|| "#1e1e2e".to_string());
    let input_hex = if hex.len() > 7 { &hex[0..7] } else { &hex };

    rsx! {
        div { class: "setting-row",
            div { class: "setting-info",
                span { class: "setting-label", "{label}" }
                if let Some(desc) = description {
                    span { class: "setting-description", "{desc}" }
                }
            }
            div { class: "color-picker",
                button {
                    class: if has_color { "toggle-btn on" } else { "toggle-btn off" },
                    onclick: move |_| {
                        if has_color {
                            on_change.call(None);
                        } else {
                            on_change.call(Some(Color { r: 30, g: 30, b: 46, a: 255 }));
                        }
                    },
                }
                if has_color {
                    input {
                        r#type: "color",
                        class: "color-input",
                        value: "{input_hex}",
                        oninput: move |e| {
                            if let Some(new_color) = Color::from_hex(&e.value()) {
                                on_change.call(Some(new_color));
                            }
                        }
                    }
                    span { class: "color-hex", "{hex}" }
                }
            }
        }
    }
}

// ============================================================================
// Appearance Page
// ============================================================================

/// Sync appearance to global settings and save
fn sync_appearance(local: &AppearanceSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.appearance = local.clone();
    }
    save_and_reload();
}

/// Sync behavior to global settings and save
fn sync_behavior(local: &BehaviorSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.behavior = local.clone();
    }
    save_and_reload();
}

/// Sync animations to global settings and save
fn sync_animations(local: &AnimationSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.animations = local.clone();
    }
    save_and_reload();
}

/// Sync keyboard to global settings and save
fn sync_keyboard(local: &KeyboardSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.keyboard = local.clone();
    }
    save_and_reload();
}

/// Sync mouse to global settings and save
fn sync_mouse(local: &MouseSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.mouse = local.clone();
    }
    save_and_reload();
}

/// Sync touchpad to global settings and save
fn sync_touchpad(local: &TouchpadSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.touchpad = local.clone();
    }
    save_and_reload();
}

/// Sync keybindings to global settings and save
fn sync_keybindings(local: &KeybindingsSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.keybindings = local.clone();
    }
    save_and_reload();
}

/// Sync outputs to global settings and save
fn sync_outputs(local: &OutputSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.outputs = local.clone();
    }
    save_and_reload();
}

/// Sync startup commands to global settings and save
fn sync_startup(local: &StartupSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.startup = local.clone();
    }
    save_and_reload();
}

/// Sync environment variables to global settings and save
fn sync_environment(local: &EnvironmentSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.environment = local.clone();
    }
    save_and_reload();
}

/// Sync workspaces to global settings and save
fn sync_workspaces(local: &WorkspacesSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.workspaces = local.clone();
    }
    save_and_reload();
}

/// Sync switch events to global settings and save
fn sync_switch_events(local: &SwitchEventsSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.switch_events = local.clone();
    }
    save_and_reload();
}

/// Sync recent windows to global settings and save
fn sync_recent_windows(local: &RecentWindowsSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.recent_windows = local.clone();
    }
    save_and_reload();
}

/// Sync layout extras to global settings and save
fn sync_layout_extras(local: &LayoutExtrasSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.layout_extras = local.clone();
    }
    save_and_reload();
}

/// Sync layer rules to global settings and save
fn sync_layer_rules(local: &LayerRulesSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.layer_rules = local.clone();
    }
    save_and_reload();
}

/// Sync window rules to global settings and save
fn sync_window_rules(local: &WindowRulesSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.window_rules = local.clone();
    }
    save_and_reload();
}

#[component]
fn AppearancePage(settings: Signal<AppearanceSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        Section { title: "Focus Ring",
            ToggleRow {
                label: "Enable focus ring",
                description: Some("Show a colored ring around the focused window"),
                value: s.focus_ring_enabled,
                on_change: move |v| {
                    settings.write().focus_ring_enabled = v;
                    sync_appearance(&settings());
                }
            }

            if s.focus_ring_enabled {
                SliderRow {
                    label: "Ring width",
                    description: Some("Thickness of the focus ring in pixels"),
                    value: s.focus_ring_width,
                    min: FOCUS_RING_WIDTH_MIN,
                    max: FOCUS_RING_WIDTH_MAX,
                    step: 1.0,
                    unit: "px",
                    on_change: move |v| {
                        settings.write().focus_ring_width = v;
                        sync_appearance(&settings());
                    }
                }

                ColorRow {
                    label: "Active color",
                    description: Some("Color when window is focused"),
                    color: s.focus_ring_active.clone(),
                    on_change: move |c| {
                        settings.write().focus_ring_active = ColorOrGradient::Color(c);
                        sync_appearance(&settings());
                    }
                }

                ColorRow {
                    label: "Inactive color",
                    description: Some("Color when window is not focused"),
                    color: s.focus_ring_inactive.clone(),
                    on_change: move |c| {
                        settings.write().focus_ring_inactive = ColorOrGradient::Color(c);
                        sync_appearance(&settings());
                    }
                }

                ColorRow {
                    label: "Urgent color",
                    description: Some("Color when window needs attention"),
                    color: s.focus_ring_urgent.clone(),
                    on_change: move |c| {
                        settings.write().focus_ring_urgent = ColorOrGradient::Color(c);
                        sync_appearance(&settings());
                    }
                }
            }
        }

        Section { title: "Window Border",
            ToggleRow {
                label: "Enable window border",
                description: Some("Show a border around windows (inside the focus ring)"),
                value: s.border_enabled,
                on_change: move |v| {
                    settings.write().border_enabled = v;
                    sync_appearance(&settings());
                }
            }
        }

        Section { title: "Background",
            OptionalColorRow {
                label: "Window background",
                description: Some("Default background color for windows"),
                color: s.background_color.clone(),
                on_change: move |c| {
                    settings.write().background_color = c;
                    sync_appearance(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Layout Page (Behavior Settings)
// ============================================================================

#[component]
fn LayoutPage(behavior: Signal<BehaviorSettings>, appearance: Signal<AppearanceSettings>) -> Element {
    let b = behavior();
    let a = appearance();
    let mut behavior = behavior;
    let mut appearance = appearance;

    rsx! {
        Section { title: "Gaps",
            SliderRow {
                label: "Window gaps",
                description: Some("Space between windows"),
                value: a.gaps,
                min: GAP_SIZE_MIN,
                max: GAP_SIZE_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    appearance.write().gaps = v;
                    sync_appearance(&appearance());
                }
            }

            SliderRow {
                label: "Corner radius",
                description: Some("Rounded corners on windows"),
                value: a.corner_radius,
                min: CORNER_RADIUS_MIN,
                max: CORNER_RADIUS_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    appearance.write().corner_radius = v;
                    sync_appearance(&appearance());
                }
            }
        }

        Section { title: "Focus Behavior",
            ToggleRow {
                label: "Focus follows mouse",
                description: Some("Windows gain focus when the mouse hovers over them"),
                value: b.focus_follows_mouse,
                on_change: move |v| {
                    behavior.write().focus_follows_mouse = v;
                    sync_behavior(&behavior());
                }
            }

            ToggleRow {
                label: "Workspace auto back-and-forth",
                description: Some("Switching to current workspace goes to previous"),
                value: b.workspace_auto_back_and_forth,
                on_change: move |v| {
                    behavior.write().workspace_auto_back_and_forth = v;
                    sync_behavior(&behavior());
                }
            }
        }

        Section { title: "Column Centering",
            ToggleRow {
                label: "Always center single column",
                description: Some("Center the column when there's only one on screen"),
                value: b.always_center_single_column,
                on_change: move |v| {
                    behavior.write().always_center_single_column = v;
                    sync_behavior(&behavior());
                }
            }

            ToggleRow {
                label: "Empty workspace above first",
                description: Some("Always keep an empty workspace above the first one"),
                value: b.empty_workspace_above_first,
                on_change: move |v| {
                    behavior.write().empty_workspace_above_first = v;
                    sync_behavior(&behavior());
                }
            }
        }

        Section { title: "Screen Edge Margins (Struts)",
            SliderRow {
                label: "Left margin",
                value: b.strut_left,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    behavior.write().strut_left = v;
                    sync_behavior(&behavior());
                }
            }

            SliderRow {
                label: "Right margin",
                value: b.strut_right,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    behavior.write().strut_right = v;
                    sync_behavior(&behavior());
                }
            }

            SliderRow {
                label: "Top margin",
                value: b.strut_top,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    behavior.write().strut_top = v;
                    sync_behavior(&behavior());
                }
            }

            SliderRow {
                label: "Bottom margin",
                value: b.strut_bottom,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    behavior.write().strut_bottom = v;
                    sync_behavior(&behavior());
                }
            }
        }
    }
}

// ============================================================================
// Animations Page
// ============================================================================

#[component]
fn AnimationsPage(settings: Signal<AnimationSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Animations" }

        Section { title: "Global Settings",
            ToggleRow {
                label: "Enable animations",
                description: Some("Enable window and workspace animations"),
                value: s.enabled,
                on_change: move |v| {
                    settings.write().enabled = v;
                    sync_animations(&settings());
                }
            }

            if s.enabled {
                SliderRow {
                    label: "Animation speed",
                    value: s.slowdown as f32,
                    min: ANIMATION_SLOWDOWN_MIN as f32,
                    max: ANIMATION_SLOWDOWN_MAX as f32,
                    step: 0.1,
                    unit: "x",
                    on_change: move |v| {
                        settings.write().slowdown = v as f64;
                        sync_animations(&settings());
                    }
                }
            }
        }
    }
}

// ============================================================================
// Keyboard Page
// ============================================================================

#[component]
fn KeyboardPage(settings: Signal<KeyboardSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Keyboard" }

        Section { title: "Key Repeat",
            SliderRow {
                label: "Repeat delay",
                value: s.repeat_delay as f32,
                min: REPEAT_DELAY_MIN as f32,
                max: REPEAT_DELAY_MAX as f32,
                step: 25.0,
                unit: "ms",
                on_change: move |v| {
                    settings.write().repeat_delay = v as i32;
                    sync_keyboard(&settings());
                }
            }

            SliderRow {
                label: "Repeat rate",
                value: s.repeat_rate as f32,
                min: REPEAT_RATE_MIN as f32,
                max: REPEAT_RATE_MAX as f32,
                step: 5.0,
                unit: "/s",
                on_change: move |v| {
                    settings.write().repeat_rate = v as i32;
                    sync_keyboard(&settings());
                }
            }
        }

        Section { title: "Numlock",
            ToggleRow {
                label: "Enable numlock on startup",
                description: Some("Turn on numlock when niri starts"),
                value: s.numlock,
                on_change: move |v| {
                    settings.write().numlock = v;
                    sync_keyboard(&settings());
                }
            }
        }

        Section { title: "Layout",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Keyboard layout" }
                    span { class: "setting-description", "XKB layout (e.g., us, de, fr)" }
                }
                span { class: "setting-value", "{s.xkb_layout}" }
            }

            if !s.xkb_variant.is_empty() {
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Layout variant" }
                    }
                    span { class: "setting-value", "{s.xkb_variant}" }
                }
            }

            if !s.xkb_options.is_empty() {
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "XKB options" }
                    }
                    span { class: "setting-value", "{s.xkb_options}" }
                }
            }
        }
    }
}

// ============================================================================
// Mouse Page
// ============================================================================

#[component]
fn MousePage(settings: Signal<MouseSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Mouse" }

        Section { title: "Scrolling",
            ToggleRow {
                label: "Natural scrolling",
                description: Some("Scroll content in the direction of finger movement"),
                value: s.natural_scroll,
                on_change: move |v| {
                    settings.write().natural_scroll = v;
                    sync_mouse(&settings());
                }
            }

            SliderRow {
                label: "Scroll speed",
                value: s.scroll_factor as f32,
                min: SCROLL_FACTOR_MIN as f32,
                max: SCROLL_FACTOR_MAX as f32,
                step: 0.1,
                unit: "x",
                on_change: move |v| {
                    settings.write().scroll_factor = v as f64;
                    sync_mouse(&settings());
                }
            }
        }

        Section { title: "Pointer",
            ToggleRow {
                label: "Left-handed mode",
                description: Some("Swap primary and secondary mouse buttons"),
                value: s.left_handed,
                on_change: move |v| {
                    settings.write().left_handed = v;
                    sync_mouse(&settings());
                }
            }

            SliderRow {
                label: "Acceleration",
                value: s.accel_speed as f32,
                min: ACCEL_SPEED_MIN as f32,
                max: ACCEL_SPEED_MAX as f32,
                step: 0.1,
                unit: "",
                on_change: move |v| {
                    settings.write().accel_speed = v as f64;
                    sync_mouse(&settings());
                }
            }

            ToggleRow {
                label: "Middle click emulation",
                description: Some("Emulate middle click by pressing left and right together"),
                value: s.middle_emulation,
                on_change: move |v| {
                    settings.write().middle_emulation = v;
                    sync_mouse(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Touchpad Page
// ============================================================================

#[component]
fn TouchpadPage(settings: Signal<TouchpadSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Touchpad" }

        Section { title: "Tapping",
            ToggleRow {
                label: "Tap to click",
                description: Some("Tap the touchpad to click"),
                value: s.tap,
                on_change: move |v| {
                    settings.write().tap = v;
                    sync_touchpad(&settings());
                }
            }

            ToggleRow {
                label: "Tap and drag",
                description: Some("Tap and hold to drag"),
                value: s.drag,
                on_change: move |v| {
                    settings.write().drag = v;
                    sync_touchpad(&settings());
                }
            }

            ToggleRow {
                label: "Drag lock",
                description: Some("Continue dragging after lifting finger briefly"),
                value: s.drag_lock,
                on_change: move |v| {
                    settings.write().drag_lock = v;
                    sync_touchpad(&settings());
                }
            }
        }

        Section { title: "Scrolling",
            ToggleRow {
                label: "Natural scrolling",
                description: Some("Scroll content in the direction of finger movement"),
                value: s.natural_scroll,
                on_change: move |v| {
                    settings.write().natural_scroll = v;
                    sync_touchpad(&settings());
                }
            }

            SliderRow {
                label: "Scroll speed",
                value: s.scroll_factor as f32,
                min: SCROLL_FACTOR_MIN as f32,
                max: SCROLL_FACTOR_MAX as f32,
                step: 0.1,
                unit: "x",
                on_change: move |v| {
                    settings.write().scroll_factor = v as f64;
                    sync_touchpad(&settings());
                }
            }
        }

        Section { title: "Pointer",
            ToggleRow {
                label: "Left-handed mode",
                description: Some("Swap primary and secondary buttons"),
                value: s.left_handed,
                on_change: move |v| {
                    settings.write().left_handed = v;
                    sync_touchpad(&settings());
                }
            }

            SliderRow {
                label: "Acceleration",
                value: s.accel_speed as f32,
                min: ACCEL_SPEED_MIN as f32,
                max: ACCEL_SPEED_MAX as f32,
                step: 0.1,
                unit: "",
                on_change: move |v| {
                    settings.write().accel_speed = v as f64;
                    sync_touchpad(&settings());
                }
            }
        }

        Section { title: "Palm Rejection",
            ToggleRow {
                label: "Disable while typing",
                description: Some("Ignore touchpad input while typing"),
                value: s.dwt,
                on_change: move |v| {
                    settings.write().dwt = v;
                    sync_touchpad(&settings());
                }
            }

            ToggleRow {
                label: "Disable while trackpointing",
                description: Some("Ignore touchpad input while using trackpoint"),
                value: s.dwtp,
                on_change: move |v| {
                    settings.write().dwtp = v;
                    sync_touchpad(&settings());
                }
            }

            ToggleRow {
                label: "Disable on external mouse",
                description: Some("Disable touchpad when external mouse is connected"),
                value: s.disabled_on_external_mouse,
                on_change: move |v| {
                    settings.write().disabled_on_external_mouse = v;
                    sync_touchpad(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Window Rules Page (Full editor)
// ============================================================================

use niri_settings::config::models::OpenBehavior;

fn next_window_rule_id(settings: &WindowRulesSettings) -> u32 {
    settings.rules.iter().map(|r| r.id).max().unwrap_or(0) + 1
}

#[component]
fn WindowRulesPage(settings: Signal<WindowRulesSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Window Rules" }

        button {
            class: "btn-add-startup",
            onclick: move |_| {
                let next_id = next_window_rule_id(&settings());
                let new_rule = WindowRule { id: next_id, name: format!("Rule {}", next_id), ..Default::default() };
                settings.write().rules.push(new_rule);
                editing_id.set(Some(next_id));
                sync_window_rules(&settings());
            },
            "+ Add Window Rule"
        }

        Section { title: "Configured Rules",
            if s.rules.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No window rules configured." }
                }
            } else {
                for rule in s.rules.iter() {
                    WindowRuleRow {
                        rule: rule.clone(),
                        is_editing: editing_id() == Some(rule.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: WindowRule| {
                            if let Some(pos) = settings().rules.iter().position(|r| r.id == updated.id) {
                                settings.write().rules[pos] = updated;
                                sync_window_rules(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().rules.retain(|r| r.id != id);
                            editing_id.set(None);
                            sync_window_rules(&settings());
                        },
                    }
                }
            }
        }
    }
}

fn open_behavior_label(b: OpenBehavior) -> &'static str {
    match b {
        OpenBehavior::Normal => "Normal",
        OpenBehavior::Maximized => "Maximized",
        OpenBehavior::Fullscreen => "Fullscreen",
        OpenBehavior::Floating => "Floating",
    }
}

#[component]
fn WindowRuleRow(
    rule: WindowRule,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<WindowRule>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = rule.id;
    let rule_for_editor = rule.clone();
    let match_desc = rule.matches.first().map(|m| {
        if let Some(ref app_id) = m.app_id {
            format!("app-id: {}", app_id)
        } else if let Some(ref title) = m.title {
            format!("title: {}", title)
        } else {
            "No match".to_string()
        }
    }).unwrap_or_else(|| "No match".to_string());

    rsx! {
        div { class: "startup-row",
            div {
                class: "startup-collapsed",
                onclick: move |_| {
                    if is_editing { on_collapse.call(()); } else { on_expand.call(id); }
                },
                span { class: "startup-command",
                    "{rule.name} ({match_desc}) - {open_behavior_label(rule.open_behavior)}"
                }
            }

            if is_editing {
                WindowRuleEditor {
                    rule: rule_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

#[component]
fn WindowRuleEditor(
    rule: WindowRule,
    on_update: EventHandler<WindowRule>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = rule.id;
    let mut local_rule = use_signal(|| rule.clone());
    let mut app_id_text = use_signal(|| rule.matches.first().and_then(|m| m.app_id.clone()).unwrap_or_default());
    let mut title_text = use_signal(|| rule.matches.first().and_then(|m| m.title.clone()).unwrap_or_default());

    let behavior_idx = match local_rule().open_behavior {
        OpenBehavior::Normal => 0,
        OpenBehavior::Maximized => 1,
        OpenBehavior::Fullscreen => 2,
        OpenBehavior::Floating => 3,
    };

    rsx! {
        div { class: "startup-editor",
            div { class: "startup-editor-grid",
                div { class: "editor-field",
                    label { "Rule Name" }
                    input {
                        r#type: "text",
                        value: "{local_rule().name}",
                        placeholder: "Rule name",
                        oninput: move |e| { local_rule.write().name = e.value(); },
                        onblur: move |_| { on_update.call(local_rule()); }
                    }
                }

                div { class: "editor-row",
                    div { class: "editor-field",
                        label { "App ID (regex)" }
                        input {
                            r#type: "text",
                            value: "{app_id_text()}",
                            placeholder: "e.g., firefox, org.kde.*",
                            oninput: move |e| { app_id_text.set(e.value()); },
                            onblur: move |_| {
                                if local_rule().matches.is_empty() {
                                    local_rule.write().matches.push(WindowRuleMatch::default());
                                }
                                let val = app_id_text();
                                local_rule.write().matches[0].app_id = if val.is_empty() { None } else { Some(val) };
                                on_update.call(local_rule());
                            }
                        }
                    }
                    div { class: "editor-field",
                        label { "Title (regex)" }
                        input {
                            r#type: "text",
                            value: "{title_text()}",
                            placeholder: "e.g., .*YouTube.*",
                            oninput: move |e| { title_text.set(e.value()); },
                            onblur: move |_| {
                                if local_rule().matches.is_empty() {
                                    local_rule.write().matches.push(WindowRuleMatch::default());
                                }
                                let val = title_text();
                                local_rule.write().matches[0].title = if val.is_empty() { None } else { Some(val) };
                                on_update.call(local_rule());
                            }
                        }
                    }
                }

                div { class: "editor-field",
                    label { "Open Behavior" }
                    Dropdown {
                        options: vec![
                            DropdownOption { value: "0".into(), label: "Normal".into() },
                            DropdownOption { value: "1".into(), label: "Maximized".into() },
                            DropdownOption { value: "2".into(), label: "Fullscreen".into() },
                            DropdownOption { value: "3".into(), label: "Floating".into() },
                        ],
                        selected_value: behavior_idx.to_string(),
                        on_change: move |val: String| {
                            let idx: i32 = val.parse().unwrap_or(0);
                            local_rule.write().open_behavior = match idx {
                                0 => OpenBehavior::Normal,
                                1 => OpenBehavior::Maximized,
                                2 => OpenBehavior::Fullscreen,
                                3 => OpenBehavior::Floating,
                                _ => OpenBehavior::Normal,
                            };
                            on_update.call(local_rule());
                        },
                    }
                }

                div { class: "editor-toggles",
                    div { class: "editor-toggle",
                        button {
                            class: if local_rule().block_out_from_screencast { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_rule.write().block_out_from_screencast = !local_rule().block_out_from_screencast;
                                on_update.call(local_rule());
                            },
                        }
                        label { "Block from screencast" }
                    }
                }

                div { class: "editor-actions",
                    button { class: "btn-delete", onclick: move |_| on_delete.call(id), "Delete" }
                    button { class: "btn-done", onclick: move |_| on_done.call(()), "Done" }
                }
            }
        }
    }
}

// ============================================================================
// Keybindings Page (Full editor)
// ============================================================================

/// Get the next available ID for a new keybinding
fn next_keybinding_id(bindings: &[Keybinding]) -> u32 {
    bindings.iter().map(|b| b.id).max().unwrap_or(0) + 1
}

/// Create a new default keybinding
fn new_keybinding(id: u32) -> Keybinding {
    Keybinding {
        id,
        key_combo: "Mod+".to_string(),
        action: KeybindAction::NiriAction("close-window".to_string()),
        allow_when_locked: false,
        cooldown_ms: None,
        repeat: false,
        hotkey_overlay_title: None,
    }
}

#[component]
fn KeybindingsPage(settings: Signal<KeybindingsSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Keybindings" }

        if let Some(ref err) = s.error {
            Section { title: "Error",
                div { class: "setting-row",
                    span { class: "setting-description", "{err}" }
                }
            }
        }

        // Add new keybinding button
        button {
            class: "btn-add-keybinding",
            onclick: move |_| {
                let next_id = next_keybinding_id(&settings().bindings);
                let new_bind = new_keybinding(next_id);
                settings.write().bindings.push(new_bind);
                editing_id.set(Some(next_id));
                sync_keybindings(&settings());
            },
            "+ Add Keybinding"
        }

        Section { title: "Configured Shortcuts",
            if s.bindings.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No keybindings configured. Click the button above to add one." }
                }
            } else {
                for binding in s.bindings.iter() {
                    KeybindingRow {
                        binding: binding.clone(),
                        is_editing: editing_id() == Some(binding.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: Keybinding| {
                            if let Some(pos) = settings().bindings.iter().position(|b| b.id == updated.id) {
                                settings.write().bindings[pos] = updated;
                                sync_keybindings(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().bindings.retain(|b| b.id != id);
                            editing_id.set(None);
                            sync_keybindings(&settings());
                        },
                    }
                }
            }
        }
    }
}

/// Row component that shows collapsed or expanded editor
#[component]
fn KeybindingRow(
    binding: Keybinding,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<Keybinding>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = binding.id;
    let binding_for_editor = binding.clone();

    rsx! {
        div { class: "keybinding-row",
            // Collapsed view (always shown)
            div {
                class: "keybinding-collapsed",
                onclick: move |_| {
                    if is_editing {
                        on_collapse.call(());
                    } else {
                        on_expand.call(id);
                    }
                },

                span { class: "keybinding-key", "{binding.key_combo}" }
                span { class: "keybinding-action", "{binding.display_name()}" }

                div { class: "keybinding-badges",
                    if binding.allow_when_locked {
                        span { class: "keybinding-badge", "locked" }
                    }
                    if binding.repeat {
                        span { class: "keybinding-badge", "repeat" }
                    }
                    if binding.cooldown_ms.is_some() {
                        span { class: "keybinding-badge", "cooldown" }
                    }
                }
            }

            // Expanded editor (shown when editing)
            if is_editing {
                KeybindingEditor {
                    binding: binding_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

/// Inline editor for a keybinding
#[component]
fn KeybindingEditor(
    binding: Keybinding,
    on_update: EventHandler<Keybinding>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = binding.id;
    let mut local_binding = use_signal(|| binding.clone());

    // Action type dropdown index
    let action_type_idx = match local_binding().action {
        KeybindAction::Spawn(_) => 0,
        KeybindAction::NiriAction(_) => 1,
        KeybindAction::NiriActionWithArgs(_, _) => 2,
    };

    // Extract action fields for display
    let (action_name, action_args) = match &local_binding().action {
        KeybindAction::Spawn(args) => (String::new(), args.join(" ")),
        KeybindAction::NiriAction(name) => (name.clone(), String::new()),
        KeybindAction::NiriActionWithArgs(name, args) => (name.clone(), args.join(" ")),
    };

    rsx! {
        div { class: "keybinding-editor",
            div { class: "keybinding-editor-grid",

                // Key combo field
                div { class: "editor-field",
                    label { "Key Combo" }
                    input {
                        r#type: "text",
                        value: "{local_binding().key_combo}",
                        placeholder: "e.g., Mod+Space, Mod+Shift+Q",
                        oninput: move |e| {
                            local_binding.write().key_combo = e.value();
                            on_update.call(local_binding());
                        }
                    }
                }

                // Action type dropdown
                div { class: "editor-field",
                    label { "Action Type" }
                    Dropdown {
                        options: vec![
                            DropdownOption { value: "0".into(), label: "Spawn Command".into() },
                            DropdownOption { value: "1".into(), label: "Niri Action".into() },
                            DropdownOption { value: "2".into(), label: "Niri Action with Args".into() },
                        ],
                        selected_value: action_type_idx.to_string(),
                        on_change: move |val: String| {
                            let idx: i32 = val.parse().unwrap_or(1);
                            let new_action = match idx {
                                0 => KeybindAction::Spawn(vec![]),
                                1 => KeybindAction::NiriAction("close-window".to_string()),
                                2 => KeybindAction::NiriActionWithArgs("focus-workspace".to_string(), vec![]),
                                _ => KeybindAction::NiriAction("close-window".to_string()),
                            };
                            local_binding.write().action = new_action;
                            on_update.call(local_binding());
                        },
                    }
                }

                // Action-specific fields
                match action_type_idx {
                    0 => rsx! {
                        // Spawn: command input
                        div { class: "editor-field",
                            label { "Command" }
                            input {
                                r#type: "text",
                                value: "{action_args}",
                                placeholder: "e.g., alacritty, firefox --new-window",
                                oninput: move |e| {
                                    // Parse using shell_words-like simple split
                                    let args = parse_command(&e.value());
                                    local_binding.write().action = KeybindAction::Spawn(args);
                                    on_update.call(local_binding());
                                }
                            }
                        }
                    },
                    1 => rsx! {
                        // NiriAction: action name
                        div { class: "editor-field",
                            label { "Action" }
                            input {
                                r#type: "text",
                                value: "{action_name}",
                                placeholder: "e.g., close-window, toggle-overview",
                                oninput: move |e| {
                                    local_binding.write().action = KeybindAction::NiriAction(e.value());
                                    on_update.call(local_binding());
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        // NiriActionWithArgs: action name + args
                        div { class: "editor-field",
                            label { "Action" }
                            input {
                                r#type: "text",
                                value: "{action_name}",
                                placeholder: "e.g., focus-workspace, set-column-width",
                                oninput: move |e| {
                                    let args = match &local_binding().action {
                                        KeybindAction::NiriActionWithArgs(_, a) => a.clone(),
                                        _ => vec![],
                                    };
                                    local_binding.write().action = KeybindAction::NiriActionWithArgs(e.value(), args);
                                    on_update.call(local_binding());
                                }
                            }
                        }
                        div { class: "editor-field",
                            label { "Arguments" }
                            input {
                                r#type: "text",
                                value: "{action_args}",
                                placeholder: "e.g., 1, +10%",
                                oninput: move |e| {
                                    let name = match &local_binding().action {
                                        KeybindAction::NiriActionWithArgs(n, _) => n.clone(),
                                        _ => String::new(),
                                    };
                                    let args = parse_command(&e.value());
                                    local_binding.write().action = KeybindAction::NiriActionWithArgs(name, args);
                                    on_update.call(local_binding());
                                }
                            }
                        }
                    },
                }

                // Overlay title (optional)
                div { class: "editor-field",
                    label { "Overlay Title" }
                    input {
                        r#type: "text",
                        value: "{local_binding().hotkey_overlay_title.clone().unwrap_or_default()}",
                        placeholder: "Optional title for hotkey overlay",
                        oninput: move |e| {
                            let val = e.value();
                            local_binding.write().hotkey_overlay_title = if val.is_empty() { None } else { Some(val) };
                            on_update.call(local_binding());
                        }
                    }
                }

                // Toggle options row
                div { class: "editor-toggles",
                    div { class: "editor-toggle",
                        button {
                            class: if local_binding().allow_when_locked { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_binding.write().allow_when_locked = !local_binding().allow_when_locked;
                                on_update.call(local_binding());
                            },
                        }
                        label { "Allow when locked" }
                    }

                    div { class: "editor-toggle",
                        button {
                            class: if local_binding().repeat { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_binding.write().repeat = !local_binding().repeat;
                                on_update.call(local_binding());
                            },
                        }
                        label { "Repeat" }
                    }
                }

                // Cooldown slider (optional)
                div { class: "editor-field",
                    label { "Cooldown (ms)" }
                    div { class: "slider-control",
                        button {
                            class: if local_binding().cooldown_ms.is_some() { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                let current = local_binding().cooldown_ms;
                                local_binding.write().cooldown_ms = if current.is_some() { None } else { Some(500) };
                                on_update.call(local_binding());
                            },
                        }
                        if let Some(cooldown) = local_binding().cooldown_ms {
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    let new_val = (cooldown - 100).max(KEYBIND_COOLDOWN_MIN);
                                    local_binding.write().cooldown_ms = Some(new_val);
                                    on_update.call(local_binding());
                                },
                                "-"
                            }
                            span { class: "slider-value", "{cooldown}ms" }
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    let new_val = (cooldown + 100).min(KEYBIND_COOLDOWN_MAX);
                                    local_binding.write().cooldown_ms = Some(new_val);
                                    on_update.call(local_binding());
                                },
                                "+"
                            }
                        }
                    }
                }

                // Action buttons
                div { class: "editor-actions",
                    button {
                        class: "btn-delete",
                        onclick: move |_| on_delete.call(id),
                        "Delete"
                    }
                    button {
                        class: "btn-done",
                        onclick: move |_| on_done.call(()),
                        "Done"
                    }
                }
            }
        }
    }
}

/// Simple command parser (splits on whitespace, handles basic quoting)
fn parse_command(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in trimmed.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }

    args
}

// ============================================================================
// Outputs Page (Full editor)
// ============================================================================

/// Create a new default output config
fn new_output_config(name: String) -> OutputConfig {
    OutputConfig {
        name,
        enabled: true,
        scale: 1.0,
        mode: String::new(),
        mode_custom: false,
        modeline: None,
        position_x: 0,
        position_y: 0,
        transform: Transform::Normal,
        vrr: VrrMode::Off,
        focus_at_startup: false,
        backdrop_color: None,
        hot_corners: None,
        layout_override: None,
    }
}

/// Get Transform display name
fn transform_label(t: Transform) -> &'static str {
    match t {
        Transform::Normal => "Normal",
        Transform::Rotate90 => "90°",
        Transform::Rotate180 => "180°",
        Transform::Rotate270 => "270°",
        Transform::Flipped => "Flipped",
        Transform::Flipped90 => "Flipped 90°",
        Transform::Flipped180 => "Flipped 180°",
        Transform::Flipped270 => "Flipped 270°",
    }
}


#[component]
fn OutputsPage(settings: Signal<OutputSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_name: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        h1 { "Displays" }

        // Add new output button
        button {
            class: "btn-add-output",
            onclick: move |_| {
                // Generate unique name
                let count = settings().outputs.len();
                let name = format!("HDMI-A-{}", count + 1);
                let new_output = new_output_config(name.clone());
                settings.write().outputs.push(new_output);
                editing_name.set(Some(name));
                sync_outputs(&settings());
            },
            "+ Add Display"
        }

        Section { title: "Configured Displays",
            if s.outputs.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No displays configured. Click the button above to add one." }
                }
            } else {
                for output in s.outputs.iter() {
                    OutputRow {
                        output: output.clone(),
                        is_editing: editing_name() == Some(output.name.clone()),
                        on_expand: move |name: String| editing_name.set(Some(name)),
                        on_collapse: move |_| editing_name.set(None),
                        on_update: move |updated: OutputConfig| {
                            if let Some(pos) = settings().outputs.iter().position(|o| o.name == updated.name) {
                                settings.write().outputs[pos] = updated;
                                sync_outputs(&settings());
                            }
                        },
                        on_delete: move |name: String| {
                            settings.write().outputs.retain(|o| o.name != name);
                            editing_name.set(None);
                            sync_outputs(&settings());
                        },
                    }
                }
            }
        }
    }
}

/// Row component that shows collapsed or expanded editor
#[component]
fn OutputRow(
    output: OutputConfig,
    is_editing: bool,
    on_expand: EventHandler<String>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<OutputConfig>,
    on_delete: EventHandler<String>,
) -> Element {
    let name = output.name.clone();
    let name_for_expand = name.clone();
    let output_for_editor = output.clone();

    rsx! {
        div { class: "output-row",
            // Collapsed view
            div {
                class: "output-collapsed",
                onclick: move |_| {
                    if is_editing {
                        on_collapse.call(());
                    } else {
                        on_expand.call(name_for_expand.clone());
                    }
                },

                span { class: "output-name", "{output.name}" }

                div { class: "output-info",
                    span { class: "output-mode",
                        if output.mode.is_empty() {
                            "Default mode"
                        } else {
                            "{output.mode}"
                        }
                    }
                    span { class: "output-scale", "{output.scale}x scale" }
                }

                div { class: "output-badges",
                    if !output.enabled {
                        span { class: "output-badge disabled", "disabled" }
                    }
                    if output.transform != Transform::Normal {
                        span { class: "output-badge", "{transform_label(output.transform)}" }
                    }
                    if output.vrr != VrrMode::Off {
                        span { class: "output-badge", "VRR" }
                    }
                    if output.focus_at_startup {
                        span { class: "output-badge", "focus" }
                    }
                }
            }

            // Expanded editor
            if is_editing {
                OutputEditor {
                    output: output_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

/// Get available modes for an output from niri IPC
fn get_available_modes(output_name: &str) -> Vec<String> {
    let mut modes = vec!["".to_string()]; // Empty = default mode

    if let Ok(outputs) = get_full_outputs() {
        if let Some(output_info) = outputs.iter().find(|o| o.name == output_name) {
            for mode in &output_info.modes {
                // Convert millihertz to Hz
                let refresh_hz = mode.refresh_rate as f64 / 1000.0;
                let mode_str = format!("{}x{}@{:.3}", mode.width, mode.height, refresh_hz);
                modes.push(mode_str);
            }
        }
    }

    modes
}

/// Format mode string for display in dropdown
fn format_mode_display(mode: &str) -> String {
    if mode.is_empty() {
        "Default (auto)".to_string()
    } else {
        mode.to_string()
    }
}

/// Inline editor for an output
#[component]
fn OutputEditor(
    output: OutputConfig,
    on_update: EventHandler<OutputConfig>,
    on_delete: EventHandler<String>,
    on_done: EventHandler<()>,
) -> Element {
    let name = output.name.clone();
    let name_for_modes = name.clone();
    let mut local_output = use_signal(|| output.clone());

    // Get available modes from niri IPC
    let available_modes = use_signal(|| get_available_modes(&name_for_modes));

    // Transform dropdown index
    let transform_idx = match local_output().transform {
        Transform::Normal => 0,
        Transform::Rotate90 => 1,
        Transform::Rotate180 => 2,
        Transform::Rotate270 => 3,
        Transform::Flipped => 4,
        Transform::Flipped90 => 5,
        Transform::Flipped180 => 6,
        Transform::Flipped270 => 7,
    };

    // VRR dropdown index
    let vrr_idx = match local_output().vrr {
        VrrMode::Off => 0,
        VrrMode::On => 1,
        VrrMode::OnDemand => 2,
    };

    rsx! {
        div { class: "output-editor",
            div { class: "output-editor-grid",

                // Name field (updates on blur to avoid breaking references)
                div { class: "editor-field",
                    label { "Name" }
                    input {
                        r#type: "text",
                        value: "{local_output().name}",
                        placeholder: "e.g., HDMI-A-1, DP-1, eDP-1",
                        oninput: move |e| {
                            local_output.write().name = e.value();
                        },
                        onblur: move |_| {
                            on_update.call(local_output());
                        }
                    }
                }

                // Mode dropdown (populated from niri IPC)
                div { class: "editor-field",
                    label { "Mode" }
                    Dropdown {
                        options: available_modes().iter().map(|m| DropdownOption {
                            value: m.clone(),
                            label: format_mode_display(m),
                        }).collect(),
                        selected_value: local_output().mode.clone(),
                        on_change: move |val: String| {
                            local_output.write().mode = val;
                            on_update.call(local_output());
                        },
                    }
                }

                // Scale field
                div { class: "editor-row",
                    div { class: "editor-field",
                        label { "Scale" }
                        div { class: "slider-control",
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    let new_val = (local_output().scale - 0.25).max(0.25);
                                    local_output.write().scale = new_val;
                                    on_update.call(local_output());
                                },
                                "-"
                            }
                            span { class: "slider-value", "{local_output().scale:.2}x" }
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    let new_val = (local_output().scale + 0.25).min(4.0);
                                    local_output.write().scale = new_val;
                                    on_update.call(local_output());
                                },
                                "+"
                            }
                        }
                    }
                }

                // Position fields
                div { class: "editor-row",
                    div { class: "editor-field",
                        label { "Position X" }
                        div { class: "slider-control",
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    local_output.write().position_x -= 100;
                                    on_update.call(local_output());
                                },
                                "-"
                            }
                            span { class: "slider-value", "{local_output().position_x}" }
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    local_output.write().position_x += 100;
                                    on_update.call(local_output());
                                },
                                "+"
                            }
                        }
                    }

                    div { class: "editor-field",
                        label { "Position Y" }
                        div { class: "slider-control",
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    local_output.write().position_y -= 100;
                                    on_update.call(local_output());
                                },
                                "-"
                            }
                            span { class: "slider-value", "{local_output().position_y}" }
                            button {
                                class: "slider-btn",
                                onclick: move |_| {
                                    local_output.write().position_y += 100;
                                    on_update.call(local_output());
                                },
                                "+"
                            }
                        }
                    }
                }

                // Transform dropdown
                div { class: "editor-field",
                    label { "Transform" }
                    Dropdown {
                        options: vec![
                            DropdownOption { value: "0".into(), label: "Normal".into() },
                            DropdownOption { value: "1".into(), label: "90°".into() },
                            DropdownOption { value: "2".into(), label: "180°".into() },
                            DropdownOption { value: "3".into(), label: "270°".into() },
                            DropdownOption { value: "4".into(), label: "Flipped".into() },
                            DropdownOption { value: "5".into(), label: "Flipped 90°".into() },
                            DropdownOption { value: "6".into(), label: "Flipped 180°".into() },
                            DropdownOption { value: "7".into(), label: "Flipped 270°".into() },
                        ],
                        selected_value: transform_idx.to_string(),
                        on_change: move |val: String| {
                            let idx: i32 = val.parse().unwrap_or(0);
                            let transform = match idx {
                                0 => Transform::Normal,
                                1 => Transform::Rotate90,
                                2 => Transform::Rotate180,
                                3 => Transform::Rotate270,
                                4 => Transform::Flipped,
                                5 => Transform::Flipped90,
                                6 => Transform::Flipped180,
                                7 => Transform::Flipped270,
                                _ => Transform::Normal,
                            };
                            local_output.write().transform = transform;
                            on_update.call(local_output());
                        },
                    }
                }

                // VRR dropdown
                div { class: "editor-field",
                    label { "Variable Refresh Rate" }
                    Dropdown {
                        options: vec![
                            DropdownOption { value: "0".into(), label: "Off".into() },
                            DropdownOption { value: "1".into(), label: "On".into() },
                            DropdownOption { value: "2".into(), label: "On Demand".into() },
                        ],
                        selected_value: vrr_idx.to_string(),
                        on_change: move |val: String| {
                            let idx: i32 = val.parse().unwrap_or(0);
                            let vrr = match idx {
                                0 => VrrMode::Off,
                                1 => VrrMode::On,
                                2 => VrrMode::OnDemand,
                                _ => VrrMode::Off,
                            };
                            local_output.write().vrr = vrr;
                            on_update.call(local_output());
                        },
                    }
                }

                // Toggle options row
                div { class: "editor-toggles",
                    div { class: "editor-toggle",
                        button {
                            class: if local_output().enabled { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_output.write().enabled = !local_output().enabled;
                                on_update.call(local_output());
                            },
                        }
                        label { "Enabled" }
                    }

                    div { class: "editor-toggle",
                        button {
                            class: if local_output().focus_at_startup { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_output.write().focus_at_startup = !local_output().focus_at_startup;
                                on_update.call(local_output());
                            },
                        }
                        label { "Focus at startup" }
                    }
                }

                // Action buttons
                div { class: "editor-actions",
                    button {
                        class: "btn-delete",
                        onclick: move |_| on_delete.call(name.clone()),
                        "Delete"
                    }
                    button {
                        class: "btn-done",
                        onclick: move |_| on_done.call(()),
                        "Done"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Startup Page (Full editor)
// ============================================================================

/// Get the next available ID for a new startup command
fn next_startup_id(settings: &StartupSettings) -> u32 {
    settings.commands.iter().map(|c| c.id).max().unwrap_or(0) + 1
}

/// Create a new default startup command
fn new_startup_command(id: u32) -> StartupCommand {
    StartupCommand {
        id,
        command: vec![String::new()],
    }
}

#[component]
fn StartupPage(settings: Signal<StartupSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Startup Commands" }

        // Add new command button
        button {
            class: "btn-add-startup",
            onclick: move |_| {
                let next_id = next_startup_id(&settings());
                let new_cmd = new_startup_command(next_id);
                settings.write().commands.push(new_cmd);
                editing_id.set(Some(next_id));
                sync_startup(&settings());
            },
            "+ Add Startup Command"
        }

        Section { title: "Commands to run at startup",
            if s.commands.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No startup commands configured. Click the button above to add one." }
                }
            } else {
                for cmd in s.commands.iter() {
                    StartupRow {
                        command: cmd.clone(),
                        is_editing: editing_id() == Some(cmd.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: StartupCommand| {
                            if let Some(pos) = settings().commands.iter().position(|c| c.id == updated.id) {
                                settings.write().commands[pos] = updated;
                                sync_startup(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().commands.retain(|c| c.id != id);
                            editing_id.set(None);
                            sync_startup(&settings());
                        },
                    }
                }
            }
        }
    }
}

/// Row component for startup command
#[component]
fn StartupRow(
    command: StartupCommand,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<StartupCommand>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = command.id;
    let command_for_editor = command.clone();

    rsx! {
        div { class: "startup-row",
            // Collapsed view
            div {
                class: "startup-collapsed",
                onclick: move |_| {
                    if is_editing {
                        on_collapse.call(());
                    } else {
                        on_expand.call(id);
                    }
                },

                span { class: "startup-command",
                    if command.command.is_empty() || command.command[0].is_empty() {
                        "(empty command)"
                    } else {
                        "{command.display()}"
                    }
                }
            }

            // Expanded editor
            if is_editing {
                StartupEditor {
                    command: command_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

/// Inline editor for a startup command
#[component]
fn StartupEditor(
    command: StartupCommand,
    on_update: EventHandler<StartupCommand>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = command.id;
    let mut local_command = use_signal(|| command.clone());

    // Store command as a single string for editing
    let mut command_text = use_signal(|| command.display());

    rsx! {
        div { class: "startup-editor",
            div { class: "startup-editor-grid",

                // Command field
                div { class: "editor-field",
                    label { "Command" }
                    input {
                        r#type: "text",
                        value: "{command_text()}",
                        placeholder: "e.g., waybar, swww-daemon, mako",
                        oninput: move |e| {
                            command_text.set(e.value());
                        },
                        onblur: move |_| {
                            // Parse command string into args
                            let args = parse_command(&command_text());
                            local_command.write().command = if args.is_empty() {
                                vec![String::new()]
                            } else {
                                args
                            };
                            on_update.call(local_command());
                        }
                    }
                }

                // Action buttons
                div { class: "editor-actions",
                    button {
                        class: "btn-delete",
                        onclick: move |_| on_delete.call(id),
                        "Delete"
                    }
                    button {
                        class: "btn-done",
                        onclick: move |_| on_done.call(()),
                        "Done"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Trackpoint Page
// ============================================================================

fn sync_trackpoint(local: &TrackpointSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.trackpoint = local.clone();
    }
    save_and_reload();
}

#[component]
fn TrackpointPage(settings: Signal<TrackpointSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Trackpoint" }

        Section { title: "Scrolling",
            ToggleRow {
                label: "Natural scrolling",
                description: Some("Scroll content in the direction of finger movement"),
                value: s.natural_scroll,
                on_change: move |v| {
                    settings.write().natural_scroll = v;
                    sync_trackpoint(&settings());
                }
            }
        }

        Section { title: "Pointer",
            ToggleRow {
                label: "Left-handed mode",
                description: Some("Swap primary and secondary buttons"),
                value: s.left_handed,
                on_change: move |v| {
                    settings.write().left_handed = v;
                    sync_trackpoint(&settings());
                }
            }

            SliderRow {
                label: "Acceleration",
                value: s.accel_speed as f32,
                min: ACCEL_SPEED_MIN as f32,
                max: ACCEL_SPEED_MAX as f32,
                step: 0.1,
                unit: "",
                on_change: move |v| {
                    settings.write().accel_speed = v as f64;
                    sync_trackpoint(&settings());
                }
            }

            ToggleRow {
                label: "Middle click emulation",
                description: Some("Emulate middle click by pressing left and right together"),
                value: s.middle_emulation,
                on_change: move |v| {
                    settings.write().middle_emulation = v;
                    sync_trackpoint(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Trackball Page
// ============================================================================

fn sync_trackball(local: &TrackballSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.trackball = local.clone();
    }
    save_and_reload();
}

#[component]
fn TrackballPage(settings: Signal<TrackballSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Trackball" }

        Section { title: "Scrolling",
            ToggleRow {
                label: "Natural scrolling",
                description: Some("Scroll content in the direction of finger movement"),
                value: s.natural_scroll,
                on_change: move |v| {
                    settings.write().natural_scroll = v;
                    sync_trackball(&settings());
                }
            }
        }

        Section { title: "Pointer",
            ToggleRow {
                label: "Left-handed mode",
                description: Some("Swap primary and secondary buttons"),
                value: s.left_handed,
                on_change: move |v| {
                    settings.write().left_handed = v;
                    sync_trackball(&settings());
                }
            }

            SliderRow {
                label: "Acceleration",
                value: s.accel_speed as f32,
                min: ACCEL_SPEED_MIN as f32,
                max: ACCEL_SPEED_MAX as f32,
                step: 0.1,
                unit: "",
                on_change: move |v| {
                    settings.write().accel_speed = v as f64;
                    sync_trackball(&settings());
                }
            }

            ToggleRow {
                label: "Middle click emulation",
                description: Some("Emulate middle click by pressing left and right together"),
                value: s.middle_emulation,
                on_change: move |v| {
                    settings.write().middle_emulation = v;
                    sync_trackball(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Tablet Page
// ============================================================================

fn sync_tablet(local: &TabletSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.tablet = local.clone();
    }
    save_and_reload();
}

#[component]
fn TabletPage(settings: Signal<TabletSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Tablet" }

        Section { title: "Device",
            ToggleRow {
                label: "Disable tablet",
                description: Some("Disable this input device entirely"),
                value: s.off,
                on_change: move |v| {
                    settings.write().off = v;
                    sync_tablet(&settings());
                }
            }

            ToggleRow {
                label: "Left-handed mode",
                description: Some("Rotate tablet 180 degrees"),
                value: s.left_handed,
                on_change: move |v| {
                    settings.write().left_handed = v;
                    sync_tablet(&settings());
                }
            }
        }

        if !s.map_to_output.is_empty() {
            Section { title: "Output Mapping",
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Mapped to output" }
                    }
                    span { class: "setting-value", "{s.map_to_output}" }
                }
            }
        }
    }
}

// ============================================================================
// Touch Page
// ============================================================================

fn sync_touch(local: &TouchSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.touch = local.clone();
    }
    save_and_reload();
}

#[component]
fn TouchPage(settings: Signal<TouchSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Touch" }

        Section { title: "Device",
            ToggleRow {
                label: "Disable touch",
                description: Some("Disable touch input entirely"),
                value: s.off,
                on_change: move |v| {
                    settings.write().off = v;
                    sync_touch(&settings());
                }
            }
        }

        if !s.map_to_output.is_empty() {
            Section { title: "Output Mapping",
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Mapped to output" }
                    }
                    span { class: "setting-value", "{s.map_to_output}" }
                }
            }
        }
    }
}

// ============================================================================
// Cursor Page
// ============================================================================

fn sync_cursor(local: &CursorSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.cursor = local.clone();
    }
    save_and_reload();
}

#[component]
fn CursorPage(settings: Signal<CursorSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Cursor" }

        Section { title: "Appearance",
            SliderRow {
                label: "Cursor size",
                value: s.size as f32,
                min: CURSOR_SIZE_MIN as f32,
                max: CURSOR_SIZE_MAX as f32,
                step: 4.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().size = v as i32;
                    sync_cursor(&settings());
                }
            }

            if !s.theme.is_empty() {
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Theme" }
                    }
                    span { class: "setting-value", "{s.theme}" }
                }
            }
        }

        Section { title: "Behavior",
            ToggleRow {
                label: "Hide when typing",
                description: Some("Hide cursor while typing on keyboard"),
                value: s.hide_when_typing,
                on_change: move |v| {
                    settings.write().hide_when_typing = v;
                    sync_cursor(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Overview Page
// ============================================================================

fn sync_overview(local: &OverviewSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.overview = local.clone();
    }
    save_and_reload();
}

#[component]
fn OverviewPage(settings: Signal<OverviewSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Overview" }

        Section { title: "Appearance",
            SliderRow {
                label: "Zoom level",
                value: s.zoom as f32,
                min: OVERVIEW_ZOOM_MIN as f32,
                max: OVERVIEW_ZOOM_MAX as f32,
                step: 0.05,
                unit: "x",
                on_change: move |v| {
                    settings.write().zoom = v as f64;
                    sync_overview(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Recent Windows Page (Full editor)
// ============================================================================

#[component]
fn RecentWindowsPage(settings: Signal<RecentWindowsSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Recent Windows" }

        Section { title: "Switcher Settings",
            ToggleRow {
                label: "Enable switcher",
                description: Some("Enable the Alt-Tab recent windows switcher"),
                value: !s.off,
                on_change: move |v: bool| {
                    settings.write().off = !v;
                    sync_recent_windows(&settings());
                }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Open delay" }
                    span { class: "setting-description", "Delay before switcher UI appears (ms)" }
                }
                div { class: "slider-control",
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().open_delay_ms = (settings().open_delay_ms - 50).max(0);
                            sync_recent_windows(&settings());
                        },
                        "-"
                    }
                    span { class: "slider-value", "{s.open_delay_ms}ms" }
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().open_delay_ms = (settings().open_delay_ms + 50).min(1000);
                            sync_recent_windows(&settings());
                        },
                        "+"
                    }
                }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Debounce" }
                    span { class: "setting-description", "Delay before window is added to recent list (ms)" }
                }
                div { class: "slider-control",
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().debounce_ms = (settings().debounce_ms - 50).max(0);
                            sync_recent_windows(&settings());
                        },
                        "-"
                    }
                    span { class: "slider-value", "{s.debounce_ms}ms" }
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().debounce_ms = (settings().debounce_ms + 50).min(500);
                            sync_recent_windows(&settings());
                        },
                        "+"
                    }
                }
            }
        }

        Section { title: "Preview Settings",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Max preview height" }
                }
                div { class: "slider-control",
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().previews.max_height = (settings().previews.max_height - 25).max(50);
                            sync_recent_windows(&settings());
                        },
                        "-"
                    }
                    span { class: "slider-value", "{s.previews.max_height}px" }
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().previews.max_height = (settings().previews.max_height + 25).min(500);
                            sync_recent_windows(&settings());
                        },
                        "+"
                    }
                }
            }
        }

        Section { title: "Highlight Settings",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Highlight padding" }
                }
                div { class: "slider-control",
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().highlight.padding = (settings().highlight.padding - 2).max(0);
                            sync_recent_windows(&settings());
                        },
                        "-"
                    }
                    span { class: "slider-value", "{s.highlight.padding}px" }
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().highlight.padding = (settings().highlight.padding + 2).min(32);
                            sync_recent_windows(&settings());
                        },
                        "+"
                    }
                }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Corner radius" }
                }
                div { class: "slider-control",
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().highlight.corner_radius = (settings().highlight.corner_radius - 2).max(0);
                            sync_recent_windows(&settings());
                        },
                        "-"
                    }
                    span { class: "slider-value", "{s.highlight.corner_radius}px" }
                    button {
                        class: "slider-btn",
                        onclick: move |_| {
                            settings.write().highlight.corner_radius = (settings().highlight.corner_radius + 2).min(32);
                            sync_recent_windows(&settings());
                        },
                        "+"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Layout Extras Page (Full editor)
// ============================================================================

#[component]
fn LayoutExtrasPage(settings: Signal<LayoutExtrasSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Layout Extras" }

        Section { title: "Window Shadows",
            ToggleRow {
                label: "Enable shadows",
                description: Some("Draw shadows behind windows"),
                value: s.shadow.enabled,
                on_change: move |v| {
                    settings.write().shadow.enabled = v;
                    sync_layout_extras(&settings());
                }
            }

            if s.shadow.enabled {
                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Softness (blur)" }
                    }
                    div { class: "slider-control",
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.softness = (settings().shadow.softness - 5).max(0);
                                sync_layout_extras(&settings());
                            },
                            "-"
                        }
                        span { class: "slider-value", "{s.shadow.softness}" }
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.softness = (settings().shadow.softness + 5).min(100);
                                sync_layout_extras(&settings());
                            },
                            "+"
                        }
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Spread" }
                    }
                    div { class: "slider-control",
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.spread = (settings().shadow.spread - 1).max(0);
                                sync_layout_extras(&settings());
                            },
                            "-"
                        }
                        span { class: "slider-value", "{s.shadow.spread}" }
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.spread = (settings().shadow.spread + 1).min(50);
                                sync_layout_extras(&settings());
                            },
                            "+"
                        }
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Offset Y" }
                    }
                    div { class: "slider-control",
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.offset_y = settings().shadow.offset_y - 1;
                                sync_layout_extras(&settings());
                            },
                            "-"
                        }
                        span { class: "slider-value", "{s.shadow.offset_y}" }
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().shadow.offset_y = settings().shadow.offset_y + 1;
                                sync_layout_extras(&settings());
                            },
                            "+"
                        }
                    }
                }

                ToggleRow {
                    label: "Draw behind window",
                    description: Some("Draw shadow behind the window content"),
                    value: s.shadow.draw_behind_window,
                    on_change: move |v| {
                        settings.write().shadow.draw_behind_window = v;
                        sync_layout_extras(&settings());
                    }
                }
            }
        }

        Section { title: "Tab Indicator",
            ToggleRow {
                label: "Enable tab indicator",
                description: Some("Show indicator for tabbed windows"),
                value: s.tab_indicator.enabled,
                on_change: move |v| {
                    settings.write().tab_indicator.enabled = v;
                    sync_layout_extras(&settings());
                }
            }

            if s.tab_indicator.enabled {
                ToggleRow {
                    label: "Hide when single tab",
                    description: Some("Hide indicator when only one tab in column"),
                    value: s.tab_indicator.hide_when_single_tab,
                    on_change: move |v| {
                        settings.write().tab_indicator.hide_when_single_tab = v;
                        sync_layout_extras(&settings());
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Width" }
                    }
                    div { class: "slider-control",
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().tab_indicator.width = (settings().tab_indicator.width - 1).max(1);
                                sync_layout_extras(&settings());
                            },
                            "-"
                        }
                        span { class: "slider-value", "{s.tab_indicator.width}px" }
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().tab_indicator.width = (settings().tab_indicator.width + 1).min(20);
                                sync_layout_extras(&settings());
                            },
                            "+"
                        }
                    }
                }

                div { class: "setting-row",
                    div { class: "setting-info",
                        span { class: "setting-label", "Gap" }
                    }
                    div { class: "slider-control",
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().tab_indicator.gap = (settings().tab_indicator.gap - 1).max(0);
                                sync_layout_extras(&settings());
                            },
                            "-"
                        }
                        span { class: "slider-value", "{s.tab_indicator.gap}px" }
                        button {
                            class: "slider-btn",
                            onclick: move |_| {
                                settings.write().tab_indicator.gap = (settings().tab_indicator.gap + 1).min(20);
                                sync_layout_extras(&settings());
                            },
                            "+"
                        }
                    }
                }
            }
        }

        Section { title: "Insert Hint",
            ToggleRow {
                label: "Enable insert hint",
                description: Some("Show visual hint when inserting windows"),
                value: s.insert_hint.enabled,
                on_change: move |v| {
                    settings.write().insert_hint.enabled = v;
                    sync_layout_extras(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Workspaces Page (Full editor)
// ============================================================================

fn next_workspace_id(settings: &WorkspacesSettings) -> u32 {
    settings.workspaces.iter().map(|w| w.id).max().unwrap_or(0) + 1
}

#[component]
fn WorkspacesPage(settings: Signal<WorkspacesSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Workspaces" }

        button {
            class: "btn-add-startup",
            onclick: move |_| {
                let next_id = next_workspace_id(&settings());
                let new_ws = NamedWorkspace { id: next_id, name: format!("Workspace {}", next_id), open_on_output: None, layout_override: None };
                settings.write().workspaces.push(new_ws);
                editing_id.set(Some(next_id));
                sync_workspaces(&settings());
            },
            "+ Add Workspace"
        }

        Section { title: "Named Workspaces",
            if s.workspaces.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No named workspaces configured." }
                }
            } else {
                for ws in s.workspaces.iter() {
                    WorkspaceRow {
                        workspace: ws.clone(),
                        is_editing: editing_id() == Some(ws.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: NamedWorkspace| {
                            if let Some(pos) = settings().workspaces.iter().position(|w| w.id == updated.id) {
                                settings.write().workspaces[pos] = updated;
                                sync_workspaces(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().workspaces.retain(|w| w.id != id);
                            editing_id.set(None);
                            sync_workspaces(&settings());
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn WorkspaceRow(
    workspace: NamedWorkspace,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<NamedWorkspace>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = workspace.id;
    let ws_for_editor = workspace.clone();

    rsx! {
        div { class: "startup-row",
            div {
                class: "startup-collapsed",
                onclick: move |_| {
                    if is_editing { on_collapse.call(()); } else { on_expand.call(id); }
                },
                span { class: "startup-command",
                    "{workspace.name}"
                    if let Some(ref output) = workspace.open_on_output {
                        " (on {output})"
                    }
                }
            }

            if is_editing {
                WorkspaceEditor {
                    workspace: ws_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

#[component]
fn WorkspaceEditor(
    workspace: NamedWorkspace,
    on_update: EventHandler<NamedWorkspace>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = workspace.id;
    let mut local_ws = use_signal(|| workspace.clone());
    let mut output_text = use_signal(|| workspace.open_on_output.clone().unwrap_or_default());

    rsx! {
        div { class: "startup-editor",
            div { class: "startup-editor-grid",
                div { class: "editor-row",
                    div { class: "editor-field",
                        label { "Name" }
                        input {
                            r#type: "text",
                            value: "{local_ws().name}",
                            placeholder: "Workspace name",
                            oninput: move |e| { local_ws.write().name = e.value(); },
                            onblur: move |_| { on_update.call(local_ws()); }
                        }
                    }
                    div { class: "editor-field",
                        label { "Output" }
                        input {
                            r#type: "text",
                            value: "{output_text()}",
                            placeholder: "e.g., DP-1, HDMI-A-1 (optional)",
                            oninput: move |e| { output_text.set(e.value()); },
                            onblur: move |_| {
                                let val = output_text();
                                local_ws.write().open_on_output = if val.is_empty() { None } else { Some(val) };
                                on_update.call(local_ws());
                            }
                        }
                    }
                }
                div { class: "editor-actions",
                    button { class: "btn-delete", onclick: move |_| on_delete.call(id), "Delete" }
                    button { class: "btn-done", onclick: move |_| on_done.call(()), "Done" }
                }
            }
        }
    }
}

// ============================================================================
// Layer Rules Page (Full editor)
// ============================================================================

fn next_layer_rule_id(settings: &LayerRulesSettings) -> u32 {
    settings.rules.iter().map(|r| r.id).max().unwrap_or(0) + 1
}

#[component]
fn LayerRulesPage(settings: Signal<LayerRulesSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Layer Rules" }

        button {
            class: "btn-add-startup",
            onclick: move |_| {
                let next_id = next_layer_rule_id(&settings());
                let new_rule = LayerRule { id: next_id, name: format!("Rule {}", next_id), ..Default::default() };
                settings.write().rules.push(new_rule);
                editing_id.set(Some(next_id));
                sync_layer_rules(&settings());
            },
            "+ Add Layer Rule"
        }

        Section { title: "Configured Rules",
            if s.rules.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No layer rules configured." }
                }
            } else {
                for rule in s.rules.iter() {
                    LayerRuleRow {
                        rule: rule.clone(),
                        is_editing: editing_id() == Some(rule.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: LayerRule| {
                            if let Some(pos) = settings().rules.iter().position(|r| r.id == updated.id) {
                                settings.write().rules[pos] = updated;
                                sync_layer_rules(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().rules.retain(|r| r.id != id);
                            editing_id.set(None);
                            sync_layer_rules(&settings());
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn LayerRuleRow(
    rule: LayerRule,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<LayerRule>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = rule.id;
    let rule_for_editor = rule.clone();
    let namespace = rule.matches.first().and_then(|m| m.namespace.clone()).unwrap_or_default();

    rsx! {
        div { class: "startup-row",
            div {
                class: "startup-collapsed",
                onclick: move |_| {
                    if is_editing { on_collapse.call(()); } else { on_expand.call(id); }
                },
                span { class: "startup-command",
                    "{rule.name}"
                    if !namespace.is_empty() {
                        " (namespace: {namespace})"
                    }
                }
            }

            if is_editing {
                LayerRuleEditor {
                    rule: rule_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

#[component]
fn LayerRuleEditor(
    rule: LayerRule,
    on_update: EventHandler<LayerRule>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = rule.id;
    let mut local_rule = use_signal(|| rule.clone());
    let mut namespace_text = use_signal(|| rule.matches.first().and_then(|m| m.namespace.clone()).unwrap_or_default());

    rsx! {
        div { class: "startup-editor",
            div { class: "startup-editor-grid",
                div { class: "editor-field",
                    label { "Rule Name" }
                    input {
                        r#type: "text",
                        value: "{local_rule().name}",
                        placeholder: "Rule name",
                        oninput: move |e| { local_rule.write().name = e.value(); },
                        onblur: move |_| { on_update.call(local_rule()); }
                    }
                }

                div { class: "editor-field",
                    label { "Namespace (regex)" }
                    input {
                        r#type: "text",
                        value: "{namespace_text()}",
                        placeholder: "e.g., waybar, mako",
                        oninput: move |e| { namespace_text.set(e.value()); },
                        onblur: move |_| {
                            let ns = namespace_text();
                            if local_rule().matches.is_empty() {
                                local_rule.write().matches.push(LayerRuleMatch::default());
                            }
                            local_rule.write().matches[0].namespace = if ns.is_empty() { None } else { Some(ns) };
                            on_update.call(local_rule());
                        }
                    }
                }

                div { class: "editor-toggles",
                    div { class: "editor-toggle",
                        button {
                            class: if local_rule().place_within_backdrop { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_rule.write().place_within_backdrop = !local_rule().place_within_backdrop;
                                on_update.call(local_rule());
                            },
                        }
                        label { "Place within backdrop" }
                    }
                    div { class: "editor-toggle",
                        button {
                            class: if local_rule().baba_is_float { "toggle-btn on" } else { "toggle-btn off" },
                            onclick: move |_| {
                                local_rule.write().baba_is_float = !local_rule().baba_is_float;
                                on_update.call(local_rule());
                            },
                        }
                        label { "Floating animations" }
                    }
                }

                div { class: "editor-actions",
                    button { class: "btn-delete", onclick: move |_| on_delete.call(id), "Delete" }
                    button { class: "btn-done", onclick: move |_| on_done.call(()), "Done" }
                }
            }
        }
    }
}

// ============================================================================
// Gestures Page
// ============================================================================

fn sync_gestures(local: &GestureSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.gestures = local.clone();
    }
    save_and_reload();
}

#[component]
fn GesturesPage(settings: Signal<GestureSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Gestures" }

        Section { title: "Hot Corners",
            ToggleRow {
                label: "Enable hot corners",
                description: Some("Trigger actions by moving cursor to screen corners"),
                value: s.hot_corners.enabled,
                on_change: move |v| {
                    settings.write().hot_corners.enabled = v;
                    sync_gestures(&settings());
                }
            }

            if s.hot_corners.enabled {
                ToggleRow {
                    label: "Top-left corner",
                    description: None,
                    value: s.hot_corners.top_left,
                    on_change: move |v| {
                        settings.write().hot_corners.top_left = v;
                        sync_gestures(&settings());
                    }
                }

                ToggleRow {
                    label: "Top-right corner",
                    description: None,
                    value: s.hot_corners.top_right,
                    on_change: move |v| {
                        settings.write().hot_corners.top_right = v;
                        sync_gestures(&settings());
                    }
                }

                ToggleRow {
                    label: "Bottom-left corner",
                    description: None,
                    value: s.hot_corners.bottom_left,
                    on_change: move |v| {
                        settings.write().hot_corners.bottom_left = v;
                        sync_gestures(&settings());
                    }
                }

                ToggleRow {
                    label: "Bottom-right corner",
                    description: None,
                    value: s.hot_corners.bottom_right,
                    on_change: move |v| {
                        settings.write().hot_corners.bottom_right = v;
                        sync_gestures(&settings());
                    }
                }
            }
        }

        Section { title: "Drag and Drop",
            ToggleRow {
                label: "Edge view scroll",
                description: Some("Scroll view when dragging to screen edge"),
                value: s.dnd_edge_view_scroll.enabled,
                on_change: move |v| {
                    settings.write().dnd_edge_view_scroll.enabled = v;
                    sync_gestures(&settings());
                }
            }

            ToggleRow {
                label: "Edge workspace switch",
                description: Some("Switch workspace when dragging to top/bottom edge"),
                value: s.dnd_edge_workspace_switch.enabled,
                on_change: move |v| {
                    settings.write().dnd_edge_workspace_switch.enabled = v;
                    sync_gestures(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Environment Page (Full editor)
// ============================================================================

fn next_env_id(settings: &EnvironmentSettings) -> u32 {
    settings.variables.iter().map(|v| v.id).max().unwrap_or(0) + 1
}

#[component]
fn EnvironmentPage(settings: Signal<EnvironmentSettings>) -> Element {
    let s = settings();
    let mut settings = settings;
    let mut editing_id: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        h1 { "Environment" }

        button {
            class: "btn-add-startup",
            onclick: move |_| {
                let next_id = next_env_id(&settings());
                let new_var = EnvironmentVariable { id: next_id, name: String::new(), value: String::new() };
                settings.write().variables.push(new_var);
                editing_id.set(Some(next_id));
                sync_environment(&settings());
            },
            "+ Add Environment Variable"
        }

        Section { title: "Environment Variables",
            if s.variables.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No environment variables configured." }
                }
            } else {
                for var in s.variables.iter() {
                    EnvVarRow {
                        variable: var.clone(),
                        is_editing: editing_id() == Some(var.id),
                        on_expand: move |id| editing_id.set(Some(id)),
                        on_collapse: move |_| editing_id.set(None),
                        on_update: move |updated: EnvironmentVariable| {
                            if let Some(pos) = settings().variables.iter().position(|v| v.id == updated.id) {
                                settings.write().variables[pos] = updated;
                                sync_environment(&settings());
                            }
                        },
                        on_delete: move |id| {
                            settings.write().variables.retain(|v| v.id != id);
                            editing_id.set(None);
                            sync_environment(&settings());
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn EnvVarRow(
    variable: EnvironmentVariable,
    is_editing: bool,
    on_expand: EventHandler<u32>,
    on_collapse: EventHandler<()>,
    on_update: EventHandler<EnvironmentVariable>,
    on_delete: EventHandler<u32>,
) -> Element {
    let id = variable.id;
    let var_for_editor = variable.clone();

    rsx! {
        div { class: "startup-row",
            div {
                class: "startup-collapsed",
                onclick: move |_| {
                    if is_editing { on_collapse.call(()); } else { on_expand.call(id); }
                },
                span { class: "startup-command",
                    if variable.name.is_empty() {
                        "(empty)"
                    } else {
                        "{variable.name}={variable.value}"
                    }
                }
            }

            if is_editing {
                EnvVarEditor {
                    variable: var_for_editor,
                    on_update: on_update,
                    on_delete: on_delete,
                    on_done: on_collapse,
                }
            }
        }
    }
}

#[component]
fn EnvVarEditor(
    variable: EnvironmentVariable,
    on_update: EventHandler<EnvironmentVariable>,
    on_delete: EventHandler<u32>,
    on_done: EventHandler<()>,
) -> Element {
    let id = variable.id;
    let mut local_var = use_signal(|| variable.clone());

    rsx! {
        div { class: "startup-editor",
            div { class: "startup-editor-grid",
                div { class: "editor-row",
                    div { class: "editor-field",
                        label { "Name" }
                        input {
                            r#type: "text",
                            value: "{local_var().name}",
                            placeholder: "e.g., EDITOR, PATH",
                            oninput: move |e| { local_var.write().name = e.value(); },
                            onblur: move |_| { on_update.call(local_var()); }
                        }
                    }
                    div { class: "editor-field",
                        label { "Value" }
                        input {
                            r#type: "text",
                            value: "{local_var().value}",
                            placeholder: "e.g., vim, /usr/bin",
                            oninput: move |e| { local_var.write().value = e.value(); },
                            onblur: move |_| { on_update.call(local_var()); }
                        }
                    }
                }
                div { class: "editor-actions",
                    button { class: "btn-delete", onclick: move |_| on_delete.call(id), "Delete" }
                    button { class: "btn-done", onclick: move |_| on_done.call(()), "Done" }
                }
            }
        }
    }
}

// ============================================================================
// Switch Events Page (Full editor)
// ============================================================================

#[component]
fn SwitchEventsPage(settings: Signal<SwitchEventsSettings>) -> Element {
    let mut settings = settings;

    // Local signals for each command input
    let mut lid_close_text = use_signal(|| settings().lid_close.display());
    let mut lid_open_text = use_signal(|| settings().lid_open.display());
    let mut tablet_on_text = use_signal(|| settings().tablet_mode_on.display());
    let mut tablet_off_text = use_signal(|| settings().tablet_mode_off.display());

    rsx! {
        h1 { "Switch Events" }

        Section { title: "Lid Events",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Lid close" }
                    span { class: "setting-description", "Command to run when laptop lid closes" }
                }
                input {
                    r#type: "text",
                    value: "{lid_close_text()}",
                    placeholder: "e.g., systemctl suspend",
                    oninput: move |e| { lid_close_text.set(e.value()); },
                    onblur: move |_| {
                        settings.write().lid_close.spawn = parse_command(&lid_close_text());
                        sync_switch_events(&settings());
                    }
                }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Lid open" }
                    span { class: "setting-description", "Command to run when laptop lid opens" }
                }
                input {
                    r#type: "text",
                    value: "{lid_open_text()}",
                    placeholder: "e.g., brightnessctl set 100%",
                    oninput: move |e| { lid_open_text.set(e.value()); },
                    onblur: move |_| {
                        settings.write().lid_open.spawn = parse_command(&lid_open_text());
                        sync_switch_events(&settings());
                    }
                }
            }
        }

        Section { title: "Tablet Mode",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Tablet mode on" }
                    span { class: "setting-description", "Command to run when entering tablet mode" }
                }
                input {
                    r#type: "text",
                    value: "{tablet_on_text()}",
                    placeholder: "e.g., wvkbd-mobintl",
                    oninput: move |e| { tablet_on_text.set(e.value()); },
                    onblur: move |_| {
                        settings.write().tablet_mode_on.spawn = parse_command(&tablet_on_text());
                        sync_switch_events(&settings());
                    }
                }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Tablet mode off" }
                    span { class: "setting-description", "Command to run when exiting tablet mode" }
                }
                input {
                    r#type: "text",
                    value: "{tablet_off_text()}",
                    placeholder: "e.g., pkill wvkbd",
                    oninput: move |e| { tablet_off_text.set(e.value()); },
                    onblur: move |_| {
                        settings.write().tablet_mode_off.spawn = parse_command(&tablet_off_text());
                        sync_switch_events(&settings());
                    }
                }
            }
        }
    }
}

// ============================================================================
// Miscellaneous Page
// ============================================================================

fn sync_misc(local: &MiscSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.miscellaneous = local.clone();
    }
    save_and_reload();
}

#[component]
fn MiscellaneousPage(settings: Signal<MiscSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Miscellaneous" }

        Section { title: "Window Decorations",
            ToggleRow {
                label: "Prefer no client-side decorations",
                description: Some("Request apps to use server-side decorations"),
                value: s.prefer_no_csd,
                on_change: move |v| {
                    settings.write().prefer_no_csd = v;
                    sync_misc(&settings());
                }
            }
        }

        Section { title: "Clipboard",
            ToggleRow {
                label: "Disable primary clipboard",
                description: Some("Disable middle-click paste clipboard"),
                value: s.disable_primary_clipboard,
                on_change: move |v| {
                    settings.write().disable_primary_clipboard = v;
                    sync_misc(&settings());
                }
            }
        }

        Section { title: "Hotkey Overlay",
            ToggleRow {
                label: "Skip at startup",
                description: Some("Don't show hotkey overlay when niri starts"),
                value: s.hotkey_overlay_skip_at_startup,
                on_change: move |v| {
                    settings.write().hotkey_overlay_skip_at_startup = v;
                    sync_misc(&settings());
                }
            }

            ToggleRow {
                label: "Hide unbound actions",
                description: Some("Hide actions that have no keybinding"),
                value: s.hotkey_overlay_hide_not_bound,
                on_change: move |v| {
                    settings.write().hotkey_overlay_hide_not_bound = v;
                    sync_misc(&settings());
                }
            }
        }

        Section { title: "Screenshots",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Screenshot path" }
                }
                span { class: "setting-value", "{s.screenshot_path}" }
            }
        }
    }
}

// ============================================================================
// Debug Page
// ============================================================================

fn sync_debug(local: &DebugSettings) {
    if let Ok(mut global) = SETTINGS.write() {
        global.debug = local.clone();
    }
    save_and_reload();
}

#[component]
fn DebugPage(settings: Signal<DebugSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Debug" }

        Section { title: "Expert Mode",
            ToggleRow {
                label: "Enable expert mode",
                description: Some("Show advanced/dangerous settings throughout the app"),
                value: s.expert_mode,
                on_change: move |v| {
                    settings.write().expert_mode = v;
                    sync_debug(&settings());
                }
            }
        }

        Section { title: "Rendering",
            ToggleRow {
                label: "Preview render mode",
                description: Some("Render as if for screencast"),
                value: s.preview_render,
                on_change: move |v| {
                    settings.write().preview_render = v;
                    sync_debug(&settings());
                }
            }

            ToggleRow {
                label: "Enable overlay planes",
                description: Some("Use hardware overlay planes for direct scanout"),
                value: s.enable_overlay_planes,
                on_change: move |v| {
                    settings.write().enable_overlay_planes = v;
                    sync_debug(&settings());
                }
            }

            ToggleRow {
                label: "Disable cursor plane",
                description: Some("Don't use hardware cursor plane"),
                value: s.disable_cursor_plane,
                on_change: move |v| {
                    settings.write().disable_cursor_plane = v;
                    sync_debug(&settings());
                }
            }

            ToggleRow {
                label: "Disable direct scanout",
                description: Some("Disable all direct scanout"),
                value: s.disable_direct_scanout,
                on_change: move |v| {
                    settings.write().disable_direct_scanout = v;
                    sync_debug(&settings());
                }
            }
        }

        Section { title: "Performance",
            ToggleRow {
                label: "Disable resize throttling",
                description: Some("Send resize events as fast as possible"),
                value: s.disable_resize_throttling,
                on_change: move |v| {
                    settings.write().disable_resize_throttling = v;
                    sync_debug(&settings());
                }
            }

            ToggleRow {
                label: "Disable transactions",
                description: Some("Disable synchronized window resizing"),
                value: s.disable_transactions,
                on_change: move |v| {
                    settings.write().disable_transactions = v;
                    sync_debug(&settings());
                }
            }
        }
    }
}

// ============================================================================
// Tools Page - Query niri state via IPC
// ============================================================================

#[component]
fn ToolsPage() -> Element {
    let mut output_text = use_signal(|| String::new());
    let mut last_action = use_signal(|| String::new());
    let niri_running = ipc::is_niri_running();

    rsx! {
        div { class: "page",
            if !niri_running {
                div { class: "warning-banner",
                    span { class: "warning-dot" }
                    "Niri is not running. Tools require a running niri instance."
                }
            }

            Section { title: "Query Windows",
                div { class: "tools-buttons",
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_windows() {
                                Ok(windows) => {
                                    let text = if windows.is_empty() {
                                        "No windows found.".to_string()
                                    } else {
                                        windows.iter().map(|w| {
                                            format!("[{}] {} - {}{}",
                                                w.id,
                                                w.app_id(),
                                                w.title(),
                                                if w.is_floating { " (floating)" } else { "" }
                                            )
                                        }).collect::<Vec<_>>().join("\n")
                                    };
                                    output_text.set(text);
                                    last_action.set("List Windows".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("List Windows (failed)".to_string());
                                }
                            }
                        },
                        "List Windows"
                    }
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_focused_window() {
                                Ok(Some(w)) => {
                                    let text = format!("[{}] {} - {}{}",
                                        w.id, w.app_id(), w.title(),
                                        if w.is_floating { " (floating)" } else { "" }
                                    );
                                    output_text.set(text);
                                    last_action.set("Focused Window".to_string());
                                }
                                Ok(None) => {
                                    output_text.set("No window is focused.".to_string());
                                    last_action.set("Focused Window".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("Focused Window (failed)".to_string());
                                }
                            }
                        },
                        "Focused Window"
                    }
                }
            }

            Section { title: "Query Workspaces",
                div { class: "tools-buttons",
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_workspaces() {
                                Ok(workspaces) => {
                                    let text = if workspaces.is_empty() {
                                        "No workspaces found.".to_string()
                                    } else {
                                        workspaces.iter().map(|ws| {
                                            format!("[{}] #{} {} on {}{}{}",
                                                ws.id,
                                                ws.idx,
                                                ws.name.as_deref().unwrap_or("(unnamed)"),
                                                ws.output.as_deref().unwrap_or("?"),
                                                if ws.is_active { " (active)" } else { "" },
                                                if ws.is_focused { " (focused)" } else { "" }
                                            )
                                        }).collect::<Vec<_>>().join("\n")
                                    };
                                    output_text.set(text);
                                    last_action.set("List Workspaces".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("List Workspaces (failed)".to_string());
                                }
                            }
                        },
                        "List Workspaces"
                    }
                }
            }

            Section { title: "Query Outputs",
                div { class: "tools-buttons",
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_full_outputs() {
                                Ok(outputs) => {
                                    let text = if outputs.is_empty() {
                                        "No outputs found.".to_string()
                                    } else {
                                        outputs.iter().map(|o| {
                                            format!("{}: {} {} @ {}x scale",
                                                o.name,
                                                if o.make.is_empty() { "Unknown" } else { &o.make },
                                                o.model,
                                                o.scale()
                                            )
                                        }).collect::<Vec<_>>().join("\n")
                                    };
                                    output_text.set(text);
                                    last_action.set("List Outputs".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("List Outputs (failed)".to_string());
                                }
                            }
                        },
                        "List Outputs"
                    }
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_focused_output() {
                                Ok(Some(name)) => {
                                    output_text.set(format!("Focused output: {}", name));
                                    last_action.set("Focused Output".to_string());
                                }
                                Ok(None) => {
                                    output_text.set("No output is focused.".to_string());
                                    last_action.set("Focused Output".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("Focused Output (failed)".to_string());
                                }
                            }
                        },
                        "Focused Output"
                    }
                }
            }

            Section { title: "Actions",
                div { class: "tools-buttons",
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::reload_config() {
                                Ok(()) => {
                                    output_text.set("Config reloaded successfully.".to_string());
                                    last_action.set("Reload Config".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("Reload Config (failed)".to_string());
                                }
                            }
                        },
                        "Reload Config"
                    }
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::validate_config() {
                                Ok(msg) => {
                                    output_text.set(msg);
                                    last_action.set("Validate Config".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Validation error: {}", e));
                                    last_action.set("Validate Config (failed)".to_string());
                                }
                            }
                        },
                        "Validate Config"
                    }
                    button {
                        class: "btn-tool",
                        disabled: !niri_running,
                        onclick: move |_| {
                            match ipc::get_version() {
                                Ok(version) => {
                                    output_text.set(format!("Niri version: {}", version));
                                    last_action.set("Get Version".to_string());
                                }
                                Err(e) => {
                                    output_text.set(format!("Error: {}", e));
                                    last_action.set("Get Version (failed)".to_string());
                                }
                            }
                        },
                        "Get Version"
                    }
                }
            }

            if !last_action().is_empty() {
                Section { title: "Output",
                    div { class: "tools-output-header", "Last action: {last_action()}" }
                    div { class: "tools-output",
                        pre { "{output_text()}" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Backups Page - Browse and restore config backups
// ============================================================================

/// Backup entry for display
#[derive(Clone, PartialEq)]
struct BackupEntry {
    filename: String,
    full_path: std::path::PathBuf,
    date: String,
    size: String,
}

fn load_backups() -> Vec<BackupEntry> {
    let backup_dir = &CONFIG_PATHS.backup_dir;

    let mut entries = Vec::new();

    if let Ok(dir) = std::fs::read_dir(backup_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Parse date from filename (format: name.YYYY-MM-DDTHH-MM-SS.sss.bak)
                let date = if let Some(start) = filename.find('.') {
                    let rest = &filename[start + 1..];
                    if let Some(end) = rest.rfind('.') {
                        rest[..end].replace('T', " ").replace('-', ":")
                    } else {
                        "Unknown date".to_string()
                    }
                } else {
                    "Unknown date".to_string()
                };

                let size = std::fs::metadata(&path)
                    .map(|m| {
                        let bytes = m.len();
                        if bytes < 1024 {
                            format!("{} B", bytes)
                        } else if bytes < 1024 * 1024 {
                            format!("{:.1} KB", bytes as f64 / 1024.0)
                        } else {
                            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                        }
                    })
                    .unwrap_or_else(|_| "? B".to_string());

                entries.push(BackupEntry {
                    filename,
                    full_path: path,
                    date,
                    size,
                });
            }
        }
    }

    // Sort by filename (which includes timestamp) descending (newest first)
    entries.sort_by(|a, b| b.filename.cmp(&a.filename));
    entries
}

#[component]
fn BackupsPage() -> Element {
    let mut backups = use_signal(|| load_backups());
    let mut preview_content = use_signal(|| String::new());
    let mut preview_filename = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());

    let refresh = move |_| {
        backups.set(load_backups());
        status_message.set("Backups refreshed.".to_string());
    };

    rsx! {
        div { class: "page",
            div { class: "info-banner",
                span { class: "info-dot" }
                "Backups are created automatically when settings are saved. You can preview and restore them here."
            }

            Section { title: "Backup Files",
                div { class: "backup-actions",
                    button {
                        class: "btn-tool",
                        onclick: refresh,
                        "Refresh List"
                    }
                    span { class: "backup-count", "{backups().len()} backup(s) found" }
                }

                if backups().is_empty() {
                    div { class: "empty-state",
                        "No backups found. Backups are created when you modify settings."
                    }
                } else {
                    div { class: "backup-list",
                        for backup in backups().iter() {
                            div { class: "backup-item",
                                div { class: "backup-info",
                                    div { class: "backup-filename", "{backup.filename}" }
                                    div { class: "backup-meta", "{backup.date} • {backup.size}" }
                                }
                                div { class: "backup-item-actions",
                                    button {
                                        class: "btn-small",
                                        onclick: {
                                            let path = backup.full_path.clone();
                                            let filename = backup.filename.clone();
                                            move |_| {
                                                match std::fs::read_to_string(&path) {
                                                    Ok(content) => {
                                                        preview_content.set(content);
                                                        preview_filename.set(filename.clone());
                                                    }
                                                    Err(e) => {
                                                        status_message.set(format!("Failed to read backup: {}", e));
                                                    }
                                                }
                                            }
                                        },
                                        "Preview"
                                    }
                                    button {
                                        class: "btn-small btn-warning",
                                        onclick: {
                                            let path = backup.full_path.clone();
                                            let filename = backup.filename.clone();
                                            move |_| {
                                                // Determine original file from backup name
                                                // Format: original.TIMESTAMP.bak -> restore to managed_dir/original
                                                let original_name = filename.split('.').next().unwrap_or("unknown");
                                                let restore_path = CONFIG_PATHS.managed_dir.join(format!("{}.kdl", original_name));

                                                match std::fs::read_to_string(&path) {
                                                    Ok(content) => {
                                                        match std::fs::write(&restore_path, &content) {
                                                            Ok(()) => {
                                                                status_message.set(format!("Restored {} successfully. Reload niri config to apply.", original_name));
                                                                let _ = ipc::reload_config();
                                                            }
                                                            Err(e) => {
                                                                status_message.set(format!("Failed to restore: {}", e));
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        status_message.set(format!("Failed to read backup: {}", e));
                                                    }
                                                }
                                            }
                                        },
                                        "Restore"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !status_message().is_empty() {
                div { class: "status-message", "{status_message()}" }
            }

            if !preview_filename().is_empty() {
                Section { title: "Preview",
                    div { class: "backup-preview-header", "File: {preview_filename()}" }
                    div { class: "backup-preview",
                        pre { "{preview_content()}" }
                    }
                }
            }
        }
    }
}
