//! Application state and events for Vizia PoC
//!
//! This demonstrates the Lens + Event pattern used by Vizia for state management.

use vizia::prelude::*;
use crate::types::{AccelProfile, ClickMethod, ScrollMethod};
use crate::constants::*;

/// Main application state
///
/// All fields need #[derive(Lens)] to enable reactive UI bindings
#[derive(Debug, Clone, Lens)]
pub struct AppState {
    // Navigation
    pub current_panel: Panel,

    // Theme
    pub dark_mode: bool,

    // Sample settings from different categories (for PoC)
    pub keyboard: KeyboardSettings,
    pub mouse: MouseSettings,
    pub touchpad: TouchpadSettings,

    // UI state (not persisted to config)
    pub unsaved_changes: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum Panel {
    Keyboard,
    Mouse,
    Touchpad,
}

/// Keyboard settings (subset for PoC)
#[derive(Debug, Clone, Lens)]
pub struct KeyboardSettings {
    pub repeat_rate: i32,
    pub repeat_delay: i32,
    pub track_layout: TrackLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TrackLayout {
    Global,
    Window,
}

/// Mouse settings (subset for PoC)
#[derive(Debug, Clone, Lens)]
pub struct MouseSettings {
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub scroll_factor: f64,
    pub natural_scroll: bool,
}

/// Touchpad settings (subset for PoC)
#[derive(Debug, Clone, Lens)]
pub struct TouchpadSettings {
    pub enabled: bool,
    pub tap_to_click: bool,
    pub natural_scroll: bool,
    pub accel_speed: f64,
    pub accel_profile: AccelProfile,
    pub scroll_method: ScrollMethod,
    pub click_method: ClickMethod,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_panel: Panel::Keyboard,
            dark_mode: true,
            keyboard: KeyboardSettings::default(),
            mouse: MouseSettings::default(),
            touchpad: TouchpadSettings::default(),
            unsaved_changes: false,
            status_message: String::new(),
        }
    }
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            repeat_rate: KEYBOARD_REPEAT_RATE_DEFAULT,
            repeat_delay: KEYBOARD_REPEAT_DELAY_DEFAULT,
            track_layout: TrackLayout::Global,
        }
    }
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            accel_speed: MOUSE_ACCEL_SPEED_DEFAULT,
            accel_profile: AccelProfile::Flat,
            scroll_factor: MOUSE_SCROLL_FACTOR_DEFAULT,
            natural_scroll: false,
        }
    }
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            tap_to_click: true,
            natural_scroll: true,
            accel_speed: TOUCHPAD_ACCEL_SPEED_DEFAULT,
            accel_profile: AccelProfile::Flat,
            scroll_method: ScrollMethod::TwoFinger,
            click_method: ClickMethod::ButtonAreas,
        }
    }
}

/// All events that can modify the application state
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Navigation
    SelectPanel(Panel),

    // Theme
    ToggleDarkMode,

    // Keyboard settings
    SetKeyboardRepeatRate(i32),
    SetKeyboardRepeatDelay(i32),
    SetKeyboardTrackLayout(TrackLayout),

    // Mouse settings
    SetMouseAccelSpeed(f64),
    SetMouseAccelProfile(AccelProfile),
    SetMouseScrollFactor(f64),
    SetMouseNaturalScroll(bool),

    // Touchpad settings
    SetTouchpadEnabled(bool),
    SetTouchpadTapToClick(bool),
    SetTouchpadNaturalScroll(bool),
    SetTouchpadAccelSpeed(f64),
    SetTouchpadAccelProfile(AccelProfile),
    SetTouchpadScrollMethod(ScrollMethod),
    SetTouchpadClickMethod(ClickMethod),

    // Config management
    SaveConfig,
    ShowStatus(String),
}

impl Model for AppState {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            // Navigation
            AppEvent::SelectPanel(panel) => {
                self.current_panel = *panel;
            }

            AppEvent::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
            }

            // Keyboard settings
            AppEvent::SetKeyboardRepeatRate(val) => {
                self.keyboard.repeat_rate = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetKeyboardRepeatDelay(val) => {
                self.keyboard.repeat_delay = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetKeyboardTrackLayout(val) => {
                self.keyboard.track_layout = *val;
                self.unsaved_changes = true;
            }

            // Mouse settings
            AppEvent::SetMouseAccelSpeed(val) => {
                self.mouse.accel_speed = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetMouseAccelProfile(val) => {
                self.mouse.accel_profile = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetMouseScrollFactor(val) => {
                self.mouse.scroll_factor = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetMouseNaturalScroll(val) => {
                self.mouse.natural_scroll = *val;
                self.unsaved_changes = true;
            }

            // Touchpad settings
            AppEvent::SetTouchpadEnabled(val) => {
                self.touchpad.enabled = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadTapToClick(val) => {
                self.touchpad.tap_to_click = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadNaturalScroll(val) => {
                self.touchpad.natural_scroll = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadAccelSpeed(val) => {
                self.touchpad.accel_speed = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadAccelProfile(val) => {
                self.touchpad.accel_profile = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadScrollMethod(val) => {
                self.touchpad.scroll_method = *val;
                self.unsaved_changes = true;
            }

            AppEvent::SetTouchpadClickMethod(val) => {
                self.touchpad.click_method = *val;
                self.unsaved_changes = true;
            }

            // Config management
            AppEvent::SaveConfig => {
                // In full app, this would call save_to_kdl()
                self.unsaved_changes = false;
                self.status_message = "Settings saved!".to_string();
                log::info!("Settings saved (PoC - not actually persisted)");
            }

            AppEvent::ShowStatus(msg) => {
                self.status_message = msg.clone();
            }
        });
    }
}

impl AppState {
    /// Load settings from config files (stub for PoC)
    pub fn load() -> anyhow::Result<Self> {
        log::info!("Loading settings (PoC - using defaults)");
        Ok(Self::default())
    }

    /// Save settings to config files (stub for PoC)
    pub fn save_to_kdl(&self) -> anyhow::Result<()> {
        log::info!("Saving settings (PoC - not implemented)");
        Ok(())
    }
}
