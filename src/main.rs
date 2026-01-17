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
    EnvironmentSettings, GestureSettings, KeyboardSettings, KeybindingsSettings,
    LayerRulesSettings, LayoutExtrasSettings, MiscSettings, MouseSettings, OutputSettings,
    OverviewSettings, RecentWindowsSettings, Settings, StartupSettings, SwitchEventsSettings,
    TabletSettings, TouchSettings, TouchpadSettings, TrackballSettings, TrackpointSettings,
    WindowRulesSettings, WorkspacesSettings,
};
use niri_settings::constants::*;
use niri_settings::ipc;

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
        Category::Layout => rsx! { LayoutPage { settings: behavior } },
        Category::LayoutExtras => rsx! { LayoutExtrasPage { settings: layout_extras } },
        Category::Workspaces => rsx! { WorkspacesPage { settings: workspaces } },
        Category::WindowRules => rsx! { WindowRulesPage { settings: window_rules } },
        Category::LayerRules => rsx! { LayerRulesPage { settings: layer_rules } },
        Category::Gestures => rsx! { GesturesPage { settings: gestures } },
        Category::Keybindings => rsx! { KeybindingsPage { settings: keybindings } },
        Category::Startup => rsx! { StartupPage { settings: startup } },
        Category::Environment => rsx! { EnvironmentPage { settings: environment } },
        Category::SwitchEvents => rsx! { SwitchEventsPage { settings: switch_events } },
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

#[component]
fn AppearancePage(settings: Signal<AppearanceSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Appearance" }

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
                    label: "Width",
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
            }
        }

        Section { title: "Window Border",
            ToggleRow {
                label: "Enable window border",
                description: Some("Draw a border around windows"),
                value: s.border_enabled,
                on_change: move |v| {
                    settings.write().border_enabled = v;
                    sync_appearance(&settings());
                }
            }

            if s.border_enabled {
                SliderRow {
                    label: "Thickness",
                    value: s.border_thickness,
                    min: BORDER_THICKNESS_MIN,
                    max: BORDER_THICKNESS_MAX,
                    step: 0.5,
                    unit: "px",
                    on_change: move |v| {
                        settings.write().border_thickness = v;
                        sync_appearance(&settings());
                    }
                }
            }
        }

        Section { title: "Layout",
            SliderRow {
                label: "Window gaps",
                value: s.gaps,
                min: GAP_SIZE_MIN,
                max: GAP_SIZE_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().gaps = v;
                    sync_appearance(&settings());
                }
            }

            SliderRow {
                label: "Corner radius",
                value: s.corner_radius,
                min: CORNER_RADIUS_MIN,
                max: CORNER_RADIUS_MAX,
                step: 1.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().corner_radius = v;
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
fn LayoutPage(settings: Signal<BehaviorSettings>) -> Element {
    let s = settings();
    let mut settings = settings;

    rsx! {
        h1 { "Layout" }

        Section { title: "Focus Behavior",
            ToggleRow {
                label: "Focus follows mouse",
                description: Some("Windows gain focus when the mouse hovers over them"),
                value: s.focus_follows_mouse,
                on_change: move |v| {
                    settings.write().focus_follows_mouse = v;
                    sync_behavior(&settings());
                }
            }

            ToggleRow {
                label: "Workspace auto back-and-forth",
                description: Some("Switching to current workspace goes to previous"),
                value: s.workspace_auto_back_and_forth,
                on_change: move |v| {
                    settings.write().workspace_auto_back_and_forth = v;
                    sync_behavior(&settings());
                }
            }
        }

        Section { title: "Column Centering",
            ToggleRow {
                label: "Always center single column",
                description: Some("Center the column when there's only one on screen"),
                value: s.always_center_single_column,
                on_change: move |v| {
                    settings.write().always_center_single_column = v;
                    sync_behavior(&settings());
                }
            }

            ToggleRow {
                label: "Empty workspace above first",
                description: Some("Always keep an empty workspace above the first one"),
                value: s.empty_workspace_above_first,
                on_change: move |v| {
                    settings.write().empty_workspace_above_first = v;
                    sync_behavior(&settings());
                }
            }
        }

        Section { title: "Screen Edge Margins (Struts)",
            SliderRow {
                label: "Left margin",
                value: s.strut_left,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().strut_left = v;
                    sync_behavior(&settings());
                }
            }

            SliderRow {
                label: "Right margin",
                value: s.strut_right,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().strut_right = v;
                    sync_behavior(&settings());
                }
            }

            SliderRow {
                label: "Top margin",
                value: s.strut_top,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().strut_top = v;
                    sync_behavior(&settings());
                }
            }

            SliderRow {
                label: "Bottom margin",
                value: s.strut_bottom,
                min: STRUT_SIZE_MIN,
                max: STRUT_SIZE_MAX,
                step: 5.0,
                unit: "px",
                on_change: move |v| {
                    settings.write().strut_bottom = v;
                    sync_behavior(&settings());
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
// Window Rules Page (Read-only list view)
// ============================================================================

#[component]
fn WindowRulesPage(settings: Signal<WindowRulesSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Window Rules" }

        Section { title: "Configured Rules",
            if s.rules.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No window rules configured." }
                }
            } else {
                for rule in s.rules.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{rule.name}" }
                            span { class: "setting-description",
                                if let Some(ref m) = rule.matches.first() {
                                    if let Some(ref app_id) = m.app_id {
                                        "app-id: {app_id}"
                                    } else if let Some(ref title) = m.title {
                                        "title: {title}"
                                    } else {
                                        "No match criteria"
                                    }
                                } else {
                                    "No match criteria"
                                }
                            }
                        }
                        span { class: "setting-value",
                            match rule.open_behavior {
                                niri_settings::config::models::OpenBehavior::Normal => "Normal",
                                niri_settings::config::models::OpenBehavior::Maximized => "Maximized",
                                niri_settings::config::models::OpenBehavior::Fullscreen => "Fullscreen",
                                niri_settings::config::models::OpenBehavior::Floating => "Floating",
                            }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit window rules in ~/.config/niri/niri-settings/window-rules.kdl" }
    }
}

// ============================================================================
// Keybindings Page (Read-only list view)
// ============================================================================

#[component]
fn KeybindingsPage(settings: Signal<KeybindingsSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Keybindings" }

        if let Some(ref err) = s.error {
            Section { title: "Error",
                div { class: "setting-row",
                    span { class: "setting-description", "{err}" }
                }
            }
        }

        Section { title: "Configured Shortcuts",
            if s.bindings.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No keybindings configured." }
                }
            } else {
                for binding in s.bindings.iter().take(20) {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{binding.key_combo}" }
                            span { class: "setting-description", "{binding.display_name()}" }
                        }
                        if binding.allow_when_locked {
                            span { class: "setting-value", "🔓" }
                        }
                    }
                }
                if s.bindings.len() > 20 {
                    div { class: "setting-row",
                        span { class: "setting-description", "... and {s.bindings.len() - 20} more" }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit keybindings in ~/.config/niri/niri-settings/keybindings.kdl" }
    }
}

// ============================================================================
// Outputs Page (Read-only list view)
// ============================================================================

#[component]
fn OutputsPage(settings: Signal<OutputSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Outputs" }

        Section { title: "Configured Displays",
            if s.outputs.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No outputs configured." }
                }
            } else {
                for output in s.outputs.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{output.name}" }
                            span { class: "setting-description",
                                if output.mode.is_empty() {
                                    "Default mode"
                                } else {
                                    "{output.mode}"
                                }
                            }
                        }
                        div { class: "slider-control",
                            span { class: "setting-value", "{output.scale}x" }
                            if !output.enabled {
                                span { class: "setting-value", " (disabled)" }
                            }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit outputs in ~/.config/niri/niri-settings/outputs.kdl" }
    }
}

// ============================================================================
// Startup Page (Read-only list view)
// ============================================================================

#[component]
fn StartupPage(settings: Signal<StartupSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Startup Commands" }

        Section { title: "Commands to run at startup",
            if s.commands.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No startup commands configured." }
                }
            } else {
                for cmd in s.commands.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{cmd.display()}" }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit startup commands in ~/.config/niri/niri-settings/startup.kdl" }
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
// Recent Windows Page
// ============================================================================

#[component]
fn RecentWindowsPage(settings: Signal<RecentWindowsSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Recent Windows" }

        Section { title: "Switcher",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Status" }
                }
                span { class: "setting-value", if s.off { "Disabled" } else { "Enabled" } }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Open delay" }
                }
                span { class: "setting-value", "{s.open_delay_ms}ms" }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Debounce" }
                }
                span { class: "setting-value", "{s.debounce_ms}ms" }
            }
        }

        Section { title: "Keybindings",
            if s.binds.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "Using default Alt+Tab binding" }
                }
            } else {
                for bind in s.binds.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{bind.key_combo}" }
                            span { class: "setting-description",
                                if bind.is_next { "Next window" } else { "Previous window" }
                            }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/recent-windows.kdl" }
    }
}

// ============================================================================
// Layout Extras Page
// ============================================================================

#[component]
fn LayoutExtrasPage(settings: Signal<LayoutExtrasSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Layout Extras" }

        Section { title: "Window Shadows",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Shadows enabled" }
                }
                span { class: "setting-value", if s.shadow.enabled { "Yes" } else { "No" } }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Shadow softness" }
                }
                span { class: "setting-value", "{s.shadow.softness}px" }
            }
        }

        Section { title: "Tab Indicator",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Tab indicator enabled" }
                }
                span { class: "setting-value", if s.tab_indicator.enabled { "Yes" } else { "No" } }
            }
        }

        Section { title: "Insert Hint",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Insert hint enabled" }
                }
                span { class: "setting-value", if s.insert_hint.enabled { "Yes" } else { "No" } }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/layout-extras.kdl" }
    }
}

// ============================================================================
// Workspaces Page
// ============================================================================

#[component]
fn WorkspacesPage(settings: Signal<WorkspacesSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Workspaces" }

        Section { title: "Named Workspaces",
            if s.workspaces.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No named workspaces configured." }
                }
            } else {
                for ws in s.workspaces.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{ws.name}" }
                            if let Some(ref output) = ws.open_on_output {
                                span { class: "setting-description", "on {output}" }
                            }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/workspaces.kdl" }
    }
}

// ============================================================================
// Layer Rules Page
// ============================================================================

#[component]
fn LayerRulesPage(settings: Signal<LayerRulesSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Layer Rules" }

        Section { title: "Configured Rules",
            if s.rules.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No layer rules configured." }
                }
            } else {
                for rule in s.rules.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{rule.name}" }
                            if let Some(ref m) = rule.matches.first() {
                                if let Some(ref ns) = m.namespace {
                                    span { class: "setting-description", "namespace: {ns}" }
                                }
                            }
                        }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/layer-rules.kdl" }
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
// Environment Page
// ============================================================================

#[component]
fn EnvironmentPage(settings: Signal<EnvironmentSettings>) -> Element {
    let s = settings();

    rsx! {
        h1 { "Environment" }

        Section { title: "Environment Variables",
            if s.variables.is_empty() {
                div { class: "setting-row",
                    span { class: "setting-description", "No environment variables configured." }
                }
            } else {
                for var in s.variables.iter() {
                    div { class: "setting-row",
                        div { class: "setting-info",
                            span { class: "setting-label", "{var.name}" }
                        }
                        span { class: "setting-value", "{var.value}" }
                    }
                }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/environment.kdl" }
    }
}

// ============================================================================
// Switch Events Page
// ============================================================================

#[component]
fn SwitchEventsPage(settings: Signal<SwitchEventsSettings>) -> Element {
    let s = settings();
    let lid_close_action = if s.lid_close.has_action() { s.lid_close.display() } else { "None".to_string() };
    let lid_open_action = if s.lid_open.has_action() { s.lid_open.display() } else { "None".to_string() };
    let tablet_on_action = if s.tablet_mode_on.has_action() { s.tablet_mode_on.display() } else { "None".to_string() };
    let tablet_off_action = if s.tablet_mode_off.has_action() { s.tablet_mode_off.display() } else { "None".to_string() };

    rsx! {
        h1 { "Switch Events" }

        Section { title: "Lid Events",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Lid close action" }
                }
                span { class: "setting-value", "{lid_close_action}" }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Lid open action" }
                }
                span { class: "setting-value", "{lid_open_action}" }
            }
        }

        Section { title: "Tablet Mode",
            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Tablet mode on action" }
                }
                span { class: "setting-value", "{tablet_on_action}" }
            }

            div { class: "setting-row",
                div { class: "setting-info",
                    span { class: "setting-label", "Tablet mode off action" }
                }
                span { class: "setting-value", "{tablet_off_action}" }
            }
        }

        p { class: "placeholder", "Edit in ~/.config/niri/niri-settings/switch-events.kdl" }
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
