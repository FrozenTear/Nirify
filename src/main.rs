//! niri-settings - Native settings application for the niri Wayland compositor
//!
//! This is the main entry point. Uses Dioxus with the Blitz native renderer.

use anyhow::Result;
use dioxus_native::prelude::*;
use log::{error, info, warn};
use std::sync::{LazyLock, Mutex, RwLock};

use niri_settings::config;
use niri_settings::config::models::{
    AnimationSettings, AppearanceSettings, BehaviorSettings, KeyboardSettings,
    KeybindingsSettings, MouseSettings, OutputSettings, Settings, StartupSettings,
    TouchpadSettings, WindowRulesSettings,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Appearance,
    Layout,
    Animations,
    Keyboard,
    Mouse,
    Touchpad,
    WindowRules,
    Keybindings,
    Outputs,
    Startup,
}

impl Category {
    fn label(&self) -> &'static str {
        match self {
            Category::Appearance => "Appearance",
            Category::Layout => "Layout",
            Category::Animations => "Animations",
            Category::Keyboard => "Keyboard",
            Category::Mouse => "Mouse",
            Category::Touchpad => "Touchpad",
            Category::WindowRules => "Window Rules",
            Category::Keybindings => "Keybindings",
            Category::Outputs => "Outputs",
            Category::Startup => "Startup",
        }
    }

    fn all() -> &'static [Category] {
        &[
            Category::Appearance,
            Category::Layout,
            Category::Animations,
            Category::Keyboard,
            Category::Mouse,
            Category::Touchpad,
            Category::WindowRules,
            Category::Keybindings,
            Category::Outputs,
            Category::Startup,
        ]
    }
}

// ============================================================================
// Root App
// ============================================================================

#[component]
fn App() -> Element {
    let selected = use_signal(|| Category::Appearance);

    // Initialize all settings signals from the loaded config
    let settings_guard = SETTINGS.read().unwrap();
    let appearance = use_signal(|| settings_guard.appearance.clone());
    let behavior = use_signal(|| settings_guard.behavior.clone());
    let animations = use_signal(|| settings_guard.animations.clone());
    let keyboard = use_signal(|| settings_guard.keyboard.clone());
    let mouse = use_signal(|| settings_guard.mouse.clone());
    let touchpad = use_signal(|| settings_guard.touchpad.clone());
    let window_rules = use_signal(|| settings_guard.window_rules.clone());
    let keybindings = use_signal(|| settings_guard.keybindings.clone());
    let outputs = use_signal(|| settings_guard.outputs.clone());
    let startup = use_signal(|| settings_guard.startup.clone());
    drop(settings_guard);

    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            Sidebar { selected }
            PageContent {
                selected: selected(),
                appearance,
                behavior,
                animations,
                keyboard,
                mouse,
                touchpad,
                window_rules,
                keybindings,
                outputs,
                startup,
            }
        }
    }
}

#[component]
fn Sidebar(selected: Signal<Category>) -> Element {
    rsx! {
        nav { class: "sidebar",
            h2 { "Settings" }
            ul {
                for category in Category::all() {
                    li {
                        class: if selected() == *category { "active" } else { "" },
                        onclick: move |_| selected.set(*category),
                        "{category.label()}"
                    }
                }
            }
        }
    }
}

#[component]
fn PageContent(
    selected: Category,
    appearance: Signal<AppearanceSettings>,
    behavior: Signal<BehaviorSettings>,
    animations: Signal<AnimationSettings>,
    keyboard: Signal<KeyboardSettings>,
    mouse: Signal<MouseSettings>,
    touchpad: Signal<TouchpadSettings>,
    window_rules: Signal<WindowRulesSettings>,
    keybindings: Signal<KeybindingsSettings>,
    outputs: Signal<OutputSettings>,
    startup: Signal<StartupSettings>,
) -> Element {
    rsx! {
        main { class: "content",
            match selected {
                Category::Appearance => rsx! { AppearancePage { settings: appearance } },
                Category::Layout => rsx! { LayoutPage { settings: behavior } },
                Category::Animations => rsx! { AnimationsPage { settings: animations } },
                Category::Keyboard => rsx! { KeyboardPage { settings: keyboard } },
                Category::Mouse => rsx! { MousePage { settings: mouse } },
                Category::Touchpad => rsx! { TouchpadPage { settings: touchpad } },
                Category::WindowRules => rsx! { WindowRulesPage { settings: window_rules } },
                Category::Keybindings => rsx! { KeybindingsPage { settings: keybindings } },
                Category::Outputs => rsx! { OutputsPage { settings: outputs } },
                Category::Startup => rsx! { StartupPage { settings: startup } },
            }
        }
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
