//! Behavior and main KDL generation
//!
//! Generates KDL configuration for behavior settings and the main include file.

use super::builder::KdlBuilder;
use crate::config::models::BehaviorSettings;

/// Generate the main.kdl content that includes all other configuration files.
///
/// This creates the entry point KDL file that uses `include` directives to
/// pull in all the individual settings files. The generated file should not
/// be edited manually as changes will be overwritten.
///
/// # Returns
/// A string containing the complete main.kdl content with include directives.
pub fn generate_main_kdl() -> String {
    let mut kdl = KdlBuilder::with_header("niri-settings-rust managed configuration");
    kdl.comment("Do not edit manually - changes will be overwritten");
    kdl.newline();

    kdl.comment("Core settings");
    kdl.field_string("include", "appearance.kdl");
    kdl.field_string("include", "behavior.kdl");
    kdl.field_string("include", "input/keyboard.kdl");
    kdl.field_string("include", "input/mouse.kdl");
    kdl.field_string("include", "input/touchpad.kdl");
    kdl.field_string("include", "input/trackpoint.kdl");
    kdl.field_string("include", "input/trackball.kdl");
    kdl.field_string("include", "input/tablet.kdl");
    kdl.field_string("include", "input/touch.kdl");
    kdl.newline();

    kdl.comment("Display & visual");
    kdl.field_string("include", "outputs.kdl");
    kdl.field_string("include", "animations.kdl");
    kdl.field_string("include", "cursor.kdl");
    kdl.field_string("include", "overview.kdl");
    kdl.newline();

    kdl.comment("Workspaces");
    kdl.field_string("include", "workspaces.kdl");
    kdl.newline();

    kdl.comment("Keybindings");
    kdl.field_string("include", "keybindings.kdl");
    kdl.newline();

    kdl.comment("Advanced settings");
    kdl.field_string("include", "advanced/layout-extras.kdl");
    kdl.field_string("include", "advanced/gestures.kdl");
    kdl.field_string("include", "advanced/layer-rules.kdl");
    kdl.field_string("include", "advanced/misc.kdl");
    kdl.field_string("include", "advanced/window-rules.kdl");
    kdl.field_string("include", "advanced/startup.kdl");
    kdl.field_string("include", "advanced/environment.kdl");
    kdl.field_string("include", "advanced/debug.kdl");
    kdl.field_string("include", "advanced/switch-events.kdl");
    kdl.field_string("include", "advanced/recent-windows.kdl");

    kdl.build()
}

/// Generate behavior.kdl content from settings.
///
/// Creates KDL configuration for behavior settings including:
/// - Focus follows mouse (inside input block)
/// - Warp mouse to focus (inside input block)
/// - Workspace auto back-and-forth (inside input block)
///
/// # Arguments
/// * `settings` - The behavior settings to convert
///
/// # Returns
/// A string containing valid KDL configuration for niri.
pub fn generate_behavior_kdl(settings: &BehaviorSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Behavior settings - managed by niri-settings-rust");

    // Check if we have any input settings to output
    let has_mod_key = settings.mod_key != crate::types::ModKey::Super;
    let has_nested_key = settings.mod_key_nested.is_some();
    let has_warp = settings.warp_mouse_to_focus != crate::types::WarpMouseMode::Off;
    let has_input_settings = has_mod_key
        || has_nested_key
        || settings.disable_power_key_handling
        || settings.focus_follows_mouse
        || has_warp
        || settings.workspace_auto_back_and_forth;

    if has_input_settings {
        kdl.block("input", |b| {
            // Modifier keys - only output if not default (Super)
            if has_mod_key {
                b.field_string("mod-key", settings.mod_key.to_kdl());
            }

            // Modifier key for nested niri instances
            if let Some(nested_key) = &settings.mod_key_nested {
                b.field_string("mod-key-nested", nested_key.to_kdl());
            }

            // Disable power key handling
            b.optional_flag(
                "disable-power-key-handling",
                settings.disable_power_key_handling,
            );

            // Focus follows mouse
            if settings.focus_follows_mouse {
                if let Some(max_scroll) = settings.focus_follows_mouse_max_scroll_amount {
                    b.raw(&format!(
                        "focus-follows-mouse max-scroll-amount=\"{}%\"",
                        max_scroll as i32
                    ));
                } else {
                    b.flag("focus-follows-mouse");
                }
            }

            // Warp mouse to focus
            match settings.warp_mouse_to_focus {
                crate::types::WarpMouseMode::Off => {}
                crate::types::WarpMouseMode::CenterXY => {
                    b.raw("warp-mouse-to-focus mode=\"center-xy\"");
                }
                crate::types::WarpMouseMode::CenterXYAlways => {
                    b.raw("warp-mouse-to-focus mode=\"center-xy-always\"");
                }
            }

            b.optional_flag(
                "workspace-auto-back-and-forth",
                settings.workspace_auto_back_and_forth,
            );
        });
    }

    // prefer_no_csd, screenshot_path, and hotkey_overlay_skip_at_startup
    // are in misc.kdl (MiscSettings)
    kdl.build()
}
