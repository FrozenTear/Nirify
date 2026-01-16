//! System settings KDL generation
//!
//! Generates KDL for startup commands, environment variables, debug settings,
//! switch events, and recent windows.

use super::builder::KdlBuilder;
use super::helpers::escape_kdl_string;
use crate::config::models::{
    DebugSettings, EnvironmentSettings, RecentWindowsSettings, StartupSettings,
    SwitchEventsSettings,
};
use log::warn;

/// Validate environment variable name follows POSIX conventions
///
/// Must start with a letter or underscore, and contain only letters, digits, and underscores.
/// This prevents potential security issues from malicious variable names like `LD_PRELOAD`.
fn is_valid_env_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Generate startup.kdl from startup settings
///
/// Creates KDL configuration for spawn-at-startup commands.
/// Each command becomes a separate spawn-at-startup node.
pub fn generate_startup_kdl(settings: &StartupSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Startup commands - managed by niri-settings-rust");
    kdl.comment("These commands run when niri starts.");
    kdl.newline();

    if settings.commands.is_empty() {
        kdl.comment("No startup commands configured.");
        kdl.comment("Example:");
        kdl.comment("spawn-at-startup \"waybar\"");
        kdl.comment("spawn-at-startup \"swww-daemon\"");
        kdl.comment("spawn-at-startup \"bash\" \"-c\" \"command with args\"");
    } else {
        for cmd in &settings.commands {
            if !cmd.command.is_empty() {
                let args: Vec<String> = cmd
                    .command
                    .iter()
                    .map(|a| format!("\"{}\"", escape_kdl_string(a)))
                    .collect();
                kdl.raw(&format!("spawn-at-startup {}", args.join(" ")));
            }
        }
    }

    kdl.build()
}

/// Generate environment.kdl from environment settings
///
/// Creates KDL configuration for environment variables.
/// Variables are set for all spawned processes.
pub fn generate_environment_kdl(settings: &EnvironmentSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Environment variables - managed by niri-settings-rust");
    kdl.comment("These are set for all processes spawned by niri.");
    kdl.newline();

    if settings.variables.is_empty() {
        kdl.comment("No environment variables configured.");
        kdl.comment("Example:");
        kdl.comment("environment {");
        kdl.comment("    QT_QPA_PLATFORM \"wayland\"");
        kdl.comment("    ELECTRON_OZONE_PLATFORM_HINT \"wayland\"");
        kdl.comment("}");
    } else {
        kdl.block("environment", |b| {
            for var in &settings.variables {
                if !var.name.is_empty() {
                    // Validate environment variable name follows POSIX conventions
                    // Must start with letter or underscore, contain only alphanumeric and underscore
                    if is_valid_env_var_name(&var.name) {
                        b.field_string(&var.name, &var.value);
                    } else {
                        warn!(
                            "Skipping invalid environment variable name: {:?} (must match [A-Za-z_][A-Za-z0-9_]*)",
                            var.name
                        );
                    }
                }
            }
        });
    }

    kdl.build()
}

/// Generate debug.kdl from debug settings
///
/// Creates KDL configuration for niri debug options.
/// These are advanced options primarily for developers and troubleshooting.
pub fn generate_debug_kdl(settings: &DebugSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Debug settings - managed by niri-settings-rust");
    kdl.comment("WARNING: These are advanced options. Use with caution.");
    kdl.newline();

    // Check if any debug option is enabled
    let has_any = settings.preview_render
        || settings.enable_overlay_planes
        || settings.disable_cursor_plane
        || settings.disable_direct_scanout
        || settings.restrict_primary_scanout_to_matching_format
        || settings.render_drm_device.is_some()
        || !settings.ignore_drm_devices.is_empty()
        || settings.wait_for_frame_completion_before_queueing
        || settings.disable_resize_throttling
        || settings.disable_transactions
        || settings.emulate_zero_presentation_time
        || settings.skip_cursor_only_updates_during_vrr
        || settings.dbus_interfaces_in_non_session_instances
        || settings.keep_laptop_panel_on_when_lid_is_closed
        || settings.disable_monitor_names
        || settings.force_disable_connectors_on_resume
        || settings.strict_new_window_focus_policy
        || settings.honor_xdg_activation_with_invalid_serial
        || settings.deactivate_unfocused_windows
        || settings.force_pipewire_invalid_modifier;

    if !has_any {
        kdl.comment("No debug options enabled.");
        kdl.comment("debug {");
        kdl.comment("    preview-render");
        kdl.comment("    disable-cursor-plane");
        kdl.comment("}");
        return kdl.build();
    }

    kdl.block("debug", |b| {
        b.optional_flag("preview-render", settings.preview_render);
        b.optional_flag("enable-overlay-planes", settings.enable_overlay_planes);
        b.optional_flag("disable-cursor-plane", settings.disable_cursor_plane);
        b.optional_flag("disable-direct-scanout", settings.disable_direct_scanout);
        b.optional_flag(
            "restrict-primary-scanout-to-matching-format",
            settings.restrict_primary_scanout_to_matching_format,
        );
        if let Some(ref device) = settings.render_drm_device {
            b.field_string("render-drm-device", device);
        }
        for device in &settings.ignore_drm_devices {
            b.field_string("ignore-drm-device", device);
        }
        b.optional_flag(
            "wait-for-frame-completion-before-queueing",
            settings.wait_for_frame_completion_before_queueing,
        );
        b.optional_flag(
            "disable-resize-throttling",
            settings.disable_resize_throttling,
        );
        b.optional_flag("disable-transactions", settings.disable_transactions);
        b.optional_flag(
            "emulate-zero-presentation-time",
            settings.emulate_zero_presentation_time,
        );
        b.optional_flag(
            "skip-cursor-only-updates-during-vrr",
            settings.skip_cursor_only_updates_during_vrr,
        );
        b.optional_flag(
            "dbus-interfaces-in-non-session-instances",
            settings.dbus_interfaces_in_non_session_instances,
        );
        b.optional_flag(
            "keep-laptop-panel-on-when-lid-is-closed",
            settings.keep_laptop_panel_on_when_lid_is_closed,
        );
        b.optional_flag("disable-monitor-names", settings.disable_monitor_names);
        b.optional_flag(
            "force-disable-connectors-on-resume",
            settings.force_disable_connectors_on_resume,
        );
        b.optional_flag(
            "strict-new-window-focus-policy",
            settings.strict_new_window_focus_policy,
        );
        b.optional_flag(
            "honor-xdg-activation-with-invalid-serial",
            settings.honor_xdg_activation_with_invalid_serial,
        );
        b.optional_flag(
            "deactivate-unfocused-windows",
            settings.deactivate_unfocused_windows,
        );
        b.optional_flag(
            "force-pipewire-invalid-modifier",
            settings.force_pipewire_invalid_modifier,
        );
    });

    kdl.build()
}

/// Generate switch-events.kdl from switch events settings
///
/// Creates KDL configuration for hardware switch events (lid, tablet mode).
/// Each event can trigger spawn commands.
pub fn generate_switch_events_kdl(settings: &SwitchEventsSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Switch events - managed by niri-settings-rust");
    kdl.comment("Configure actions for hardware switch events.");
    kdl.newline();

    // Check if any switch event has actions
    let has_any = !settings.lid_close.spawn.is_empty()
        || !settings.lid_open.spawn.is_empty()
        || !settings.tablet_mode_on.spawn.is_empty()
        || !settings.tablet_mode_off.spawn.is_empty();

    if !has_any {
        kdl.comment("No switch events configured.");
        kdl.comment("switch-events {");
        kdl.comment("    lid-close {");
        kdl.comment("        spawn \"swaylock\"");
        kdl.comment("    }");
        kdl.comment("}");
        return kdl.build();
    }

    kdl.block("switch-events", |b| {
        // Helper to generate spawn commands for a switch event
        // Note: This is a local fn so it can't capture `warn!` directly,
        // we return a bool to indicate if any command was skipped
        fn add_spawn_block(b: &mut KdlBuilder, name: &str, commands: &[String]) -> Vec<String> {
            let mut skipped = Vec::new();
            if !commands.is_empty() {
                b.block(name, |inner| {
                    for cmd in commands {
                        // Validate command can be properly shell-parsed
                        // Don't use fallback - reject malformed commands to prevent injection
                        match shell_words::split(cmd) {
                            Ok(args) if !args.is_empty() => {
                                let escaped: Vec<String> = args
                                    .iter()
                                    .map(|a| format!("\"{}\"", escape_kdl_string(a)))
                                    .collect();
                                inner.raw(&format!("spawn {}", escaped.join(" ")));
                            }
                            Ok(_) => {
                                // Empty command after parsing, skip silently
                            }
                            Err(_) => {
                                // Malformed command (unbalanced quotes, etc.) - skip it
                                skipped.push(cmd.clone());
                            }
                        }
                    }
                });
            }
            skipped
        }

        let mut all_skipped = Vec::new();
        all_skipped.extend(add_spawn_block(b, "lid-close", &settings.lid_close.spawn));
        all_skipped.extend(add_spawn_block(b, "lid-open", &settings.lid_open.spawn));
        all_skipped.extend(add_spawn_block(
            b,
            "tablet-mode-on",
            &settings.tablet_mode_on.spawn,
        ));
        all_skipped.extend(add_spawn_block(
            b,
            "tablet-mode-off",
            &settings.tablet_mode_off.spawn,
        ));

        // Log warnings for skipped commands outside the local fn
        for cmd in all_skipped {
            warn!(
                "Skipping malformed switch-event command (unbalanced quotes?): {:?}",
                cmd
            );
        }
    });

    kdl.build()
}

/// Generate recent-windows.kdl from recent windows settings (v25.05+)
///
/// Creates KDL configuration for the recent windows (Alt-Tab) switcher.
/// Includes highlight styling and preview settings.
pub fn generate_recent_windows_kdl(settings: &RecentWindowsSettings) -> String {
    let mut kdl =
        KdlBuilder::with_header("Recent windows switcher - managed by niri-settings-rust");
    kdl.comment("Configures the Alt-Tab window switcher appearance.");
    kdl.newline();

    // If disabled, just output the off flag
    if settings.off {
        kdl.raw("recent-windows { off }");
        return kdl.build();
    }

    kdl.block("recent-windows", |b| {
        // Timing settings
        b.field_i32("debounce-ms", settings.debounce_ms);
        b.field_i32("open-delay-ms", settings.open_delay_ms);

        // Highlight settings
        b.newline();
        b.block("highlight", |h| {
            h.field_color("active-color", &settings.highlight.active_color);
            h.field_color("urgent-color", &settings.highlight.urgent_color);
            h.field_i32("padding", settings.highlight.padding);
            h.field_i32("corner-radius", settings.highlight.corner_radius);
        });

        // Preview settings
        b.newline();
        b.block("previews", |p| {
            p.field_i32("max-height", settings.previews.max_height);
            p.field_f32("max-scale", settings.previews.max_scale as f32);
        });

        // Binds (if any custom binds configured)
        if !settings.binds.is_empty() {
            b.newline();
            b.block("binds", |binds| {
                for bind in &settings.binds {
                    // Build the bind line: KeyCombo cooldown-ms=50 { action filter="app-id" scope="output"; }
                    let action = if bind.is_next {
                        "next-window"
                    } else {
                        "previous-window"
                    };

                    let mut action_parts = vec![action.to_string()];
                    if bind.filter_app_id {
                        action_parts.push("filter=\"app-id\"".to_string());
                    }
                    if let Some(scope) = bind.scope {
                        action_parts.push(format!("scope=\"{}\"", scope.to_kdl()));
                    }

                    let cooldown_part = if let Some(ms) = bind.cooldown_ms {
                        format!(" cooldown-ms={}", ms)
                    } else {
                        String::new()
                    };

                    binds.raw(&format!(
                        "{}{}  {{ {}; }}",
                        bind.key_combo,
                        cooldown_part,
                        action_parts.join(" ")
                    ));
                }
            });
        }
    });

    kdl.build()
}
