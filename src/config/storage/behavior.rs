//! Behavior and main KDL generation
//!
//! Generates KDL configuration for behavior settings and the main include file.

use super::builder::KdlBuilder;
use crate::config::models::BehaviorSettings;
use crate::version::FeatureCompat;

pub fn generate_main_kdl(compat: FeatureCompat) -> String {
    use crate::config::registry::ConfigFile;

    let mut kdl = KdlBuilder::with_header("Nirify managed configuration");
    kdl.comment("Do not edit manually - changes will be overwritten");
    kdl.newline();

    kdl.comment("Included settings files");
    for file in ConfigFile::ALL {
        if !file.included_in_main() {
            continue;
        }
        if file.requires_recent_windows() && !compat.recent_windows {
            // Recent windows requires niri 25.11+
            kdl.comment("recent-windows.kdl requires niri 25.11+ (skipped)");
            continue;
        }
        if file.requires_blur() && !compat.blur {
            // Top-level blur requires niri 26.04+
            kdl.comment("blur.kdl requires niri 26.04+ (skipped)");
            continue;
        }
        kdl.field_string("include", file.relative_path());
    }

    kdl.build()
}

/// Generate behavior.kdl content from settings.
///
/// Creates KDL configuration for behavior settings including:
/// - Focus follows mouse (inside input block)
/// - Warp mouse to focus (inside input block)
/// - Workspace auto back-and-forth (inside input block)
///
/// # Omit-default policy
///
/// Nirify always emits keywords it owns when this category file is managed,
/// rather than only when a `ConflictingInclude` exists. `workspace-auto-back-and-forth`
/// is a niri [`Flag`](https://github.com/niri-wm/niri/blob/main/niri-config/src/utils.rs)
/// (`presence` / `true` / `false`), so we write `false` for the Off default
/// and it last-wins against an earlier include that enabled it.
///
/// `focus-follows-mouse` and `warp-mouse-to-focus` are **not** Flags in niri
/// (`Option<FocusFollowsMouse>` / `Option<WarpMouseToFocus>`, merged with
/// `merge_clone_opt`). There is no valid `false` form: writing the node
/// would *enable* them, and knuffel rejects a boolean argument. Off remains
/// omit-only — a known niri limitation, documented here rather than emitting
/// invalid KDL.
///
/// `scale` is unrelated (`Option<f64>`: `None` = niri auto).
///
/// # Arguments
/// * `settings` - The behavior settings to convert
///
/// # Returns
/// A string containing valid KDL configuration for niri.
pub fn generate_behavior_kdl(settings: &BehaviorSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Behavior settings - managed by Nirify");

    // Always emit `input {}` so owned Flag defaults (workspace-auto-back-and-forth
    // false) are written even when every other field is niri-default.
    kdl.block("input", |b| {
        // Modifier keys - only output if not default (Super)
        if settings.mod_key != crate::types::ModKey::Super {
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

        // Focus follows mouse — presence enables; no niri-valid Off form.
        if settings.focus_follows_mouse {
            if let Some(max_scroll) = settings.focus_follows_mouse_max_scroll_amount {
                // niri's Percent parses an f64 before '%', so fractional
                // values like 12.5% are valid; preserve them.
                b.raw(&format!(
                    "focus-follows-mouse max-scroll-amount=\"{}%\"",
                    max_scroll
                ));
            } else {
                b.flag("focus-follows-mouse");
            }
        }

        // Warp mouse to focus — same Option-struct merge; Off cannot be written.
        match settings.warp_mouse_to_focus {
            crate::types::WarpMouseMode::Off => {}
            crate::types::WarpMouseMode::Enabled => {
                // Bare flag = warp with no mode (minimal cursor movement).
                b.flag("warp-mouse-to-focus");
            }
            crate::types::WarpMouseMode::CenterXY => {
                b.raw("warp-mouse-to-focus mode=\"center-xy\"");
            }
            crate::types::WarpMouseMode::CenterXYAlways => {
                b.raw("warp-mouse-to-focus mode=\"center-xy-always\"");
            }
        }

        b.flag_or_false(
            "workspace-auto-back-and-forth",
            settings.workspace_auto_back_and_forth,
        );
    });

    // prefer_no_csd, screenshot_path, and hotkey_overlay_skip_at_startup
    // are in misc.kdl (MiscSettings)
    kdl.build()
}

#[cfg(test)]
// Test setup mutates a couple fields after default() for readability.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_main_kdl_excludes_preferences() {
        let content = generate_main_kdl(FeatureCompat::all_enabled());
        // main.kdl must re-parse as valid KDL
        let _doc: kdl::KdlDocument = content.parse().expect("main.kdl should parse");
        assert!(
            !content.contains("preferences.kdl"),
            "main.kdl must not include preferences.kdl"
        );
        assert!(
            content.contains("advanced/recent-windows.kdl"),
            "main.kdl should include recent-windows.kdl when enabled"
        );
    }

    #[test]
    fn test_main_kdl_skips_recent_windows_when_unsupported() {
        let compat = FeatureCompat {
            recent_windows: false,
            ..Default::default()
        };
        let content = generate_main_kdl(compat);
        assert!(!content.contains("recent-windows.kdl\""));
    }

    fn roundtrip_behavior(settings: &BehaviorSettings) -> crate::config::models::Settings {
        let kdl = generate_behavior_kdl(settings);
        let doc: kdl::KdlDocument = kdl.parse().expect("generated behavior KDL must re-parse");
        let mut loaded = crate::config::models::Settings::default();
        crate::config::loader::parse_behavior_from_doc(&doc, &mut loaded);
        loaded
    }

    #[test]
    fn always_emits_workspace_auto_back_and_forth_false() {
        let settings = BehaviorSettings::default();
        let kdl = generate_behavior_kdl(&settings);
        assert!(
            kdl.contains("workspace-auto-back-and-forth false"),
            "owned Flag default must last-wins:\n{kdl}"
        );
        assert!(!kdl.contains("focus-follows-mouse"));
        assert!(!kdl.contains("warp-mouse-to-focus"));
        let loaded = roundtrip_behavior(&settings);
        assert!(!loaded.behavior.workspace_auto_back_and_forth);
    }

    #[test]
    fn warp_mouse_modeless_roundtrip() {
        let mut settings = BehaviorSettings::default();
        settings.warp_mouse_to_focus = crate::types::WarpMouseMode::Enabled;
        let kdl = generate_behavior_kdl(&settings);
        assert!(kdl.contains("warp-mouse-to-focus"));
        assert!(!kdl.contains("mode="));
        let loaded = roundtrip_behavior(&settings);
        assert_eq!(
            loaded.behavior.warp_mouse_to_focus,
            crate::types::WarpMouseMode::Enabled
        );

        // Existing mode variants still roundtrip.
        for mode in [
            crate::types::WarpMouseMode::CenterXY,
            crate::types::WarpMouseMode::CenterXYAlways,
        ] {
            let mut s = BehaviorSettings::default();
            s.warp_mouse_to_focus = mode;
            assert_eq!(roundtrip_behavior(&s).behavior.warp_mouse_to_focus, mode);
        }
    }

    #[test]
    fn ffm_fractional_percent_roundtrip() {
        let mut settings = BehaviorSettings::default();
        settings.focus_follows_mouse = true;
        settings.focus_follows_mouse_max_scroll_amount = Some(12.5);
        let kdl = generate_behavior_kdl(&settings);
        assert!(kdl.contains("max-scroll-amount=\"12.5%\""));
        let loaded = roundtrip_behavior(&settings);
        assert_eq!(
            loaded.behavior.focus_follows_mouse_max_scroll_amount,
            Some(12.5)
        );
    }
}
