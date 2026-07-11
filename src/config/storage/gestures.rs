//! Gesture settings KDL generation
//!
//! Generates KDL for hot corners and DND settings.

use super::builder::KdlBuilder;
use crate::config::models::{DndEdgeSettings, GestureSettings, HotCorners};

pub fn generate_gestures_kdl(settings: &GestureSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Gesture settings - managed by Nirify");

    // niri's default hot-corners behavior (top-left active) is represented by
    // HotCorners::default(); only emit a hot-corners block when it differs.
    let hot_corners_differ = settings.hot_corners != HotCorners::default();

    let has_dnd_view_scroll = has_dnd_edge_settings(
        &settings.dnd_edge_view_scroll,
        &DndEdgeSettings::default_scroll(),
    );

    let has_dnd_workspace_switch = has_dnd_edge_settings(
        &settings.dnd_edge_workspace_switch,
        &DndEdgeSettings::default_workspace(),
    );

    // Only output gestures block if there's something to output
    if hot_corners_differ || has_dnd_view_scroll || has_dnd_workspace_switch {
        kdl.block("gestures", |g| {
            // Hot corners
            if hot_corners_differ {
                g.block("hot-corners", |b| {
                    let hc = &settings.hot_corners;
                    if !hc.enabled {
                        // niri has no "empty" hot-corners; `off` disables entirely.
                        b.flag("off");
                    } else {
                        b.optional_flag("top-left", hc.top_left);
                        b.optional_flag("top-right", hc.top_right);
                        b.optional_flag("bottom-left", hc.bottom_left);
                        b.optional_flag("bottom-right", hc.bottom_right);
                    }
                });
            }

            // DND edge view scroll
            if has_dnd_view_scroll {
                generate_dnd_edge_view_scroll(g, &settings.dnd_edge_view_scroll);
            }

            // DND edge workspace switch
            if has_dnd_workspace_switch {
                generate_dnd_edge_workspace_switch(g, &settings.dnd_edge_workspace_switch);
            }
        });
    }

    kdl.build()
}

/// Check if DND edge settings differ from defaults or are disabled
fn has_dnd_edge_settings(settings: &DndEdgeSettings, defaults: &DndEdgeSettings) -> bool {
    // Emit the block if disabled (persisted as a `trigger-*` of 0) or if any
    // value differs from the niri default.
    !settings.enabled
        || settings.trigger_size != defaults.trigger_size
        || settings.delay_ms != defaults.delay_ms
        || settings.max_speed != defaults.max_speed
}

fn generate_dnd_edge_view_scroll(builder: &mut KdlBuilder, settings: &DndEdgeSettings) {
    let defaults = DndEdgeSettings::default_scroll();
    builder.block("dnd-edge-view-scroll", |b| {
        if !settings.enabled {
            // niri has no `off` for this block; a zero-width trigger zone never
            // fires, which is a functional disable.
            b.field_i32("trigger-width", 0);
            b.field_i32_if_not("delay-ms", settings.delay_ms, defaults.delay_ms);
            b.field_i32_if_not("max-speed", settings.max_speed, defaults.max_speed);
        } else {
            b.field_i32_if_not(
                "trigger-width",
                settings.trigger_size,
                defaults.trigger_size,
            );
            b.field_i32_if_not("delay-ms", settings.delay_ms, defaults.delay_ms);
            b.field_i32_if_not("max-speed", settings.max_speed, defaults.max_speed);
        }
    });
}

fn generate_dnd_edge_workspace_switch(builder: &mut KdlBuilder, settings: &DndEdgeSettings) {
    let defaults = DndEdgeSettings::default_workspace();
    builder.block("dnd-edge-workspace-switch", |b| {
        if !settings.enabled {
            // niri has no `off` for this block; a zero-height trigger zone never
            // fires, which is a functional disable.
            b.field_i32("trigger-height", 0);
            b.field_i32_if_not("delay-ms", settings.delay_ms, defaults.delay_ms);
            b.field_i32_if_not("max-speed", settings.max_speed, defaults.max_speed);
        } else {
            b.field_i32_if_not(
                "trigger-height",
                settings.trigger_size,
                defaults.trigger_size,
            );
            b.field_i32_if_not("delay-ms", settings.delay_ms, defaults.delay_ms);
            b.field_i32_if_not("max-speed", settings.max_speed, defaults.max_speed);
        }
    });
}

#[cfg(test)]
// Test setup mutates a couple fields after default() for readability.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::config::loader::parse_gestures_from_doc;
    use crate::config::models::Settings;

    /// Generate gesture KDL, assert it re-parses via the kdl crate, then load it
    /// back through the loader and return the resulting Settings.
    fn roundtrip(settings: &GestureSettings) -> Settings {
        let kdl = generate_gestures_kdl(settings);
        let doc: kdl::KdlDocument = kdl.parse().expect("generated gesture KDL must re-parse");
        let mut loaded = Settings::default();
        parse_gestures_from_doc(&doc, &mut loaded);
        loaded
    }

    #[test]
    fn test_default_settings_no_output() {
        let settings = GestureSettings::default();
        let kdl = generate_gestures_kdl(&settings);
        // Default settings should only produce a header, no gestures block
        assert!(!kdl.contains("gestures {"));
        assert!(!kdl.contains("dnd-edge-view-scroll"));
        assert!(!kdl.contains("dnd-edge-workspace-switch"));
    }

    #[test]
    fn test_dnd_view_scroll_disabled() {
        let mut settings = GestureSettings::default();
        settings.dnd_edge_view_scroll.enabled = false;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("gestures {"));
        assert!(kdl.contains("dnd-edge-view-scroll {"));
        // niri has no `off`; disable = zero-width trigger zone.
        assert!(kdl.contains("trigger-width 0"));
        assert!(!kdl.contains("off"));
    }

    #[test]
    fn test_dnd_view_scroll_custom_values() {
        let mut settings = GestureSettings::default();
        settings.dnd_edge_view_scroll.trigger_size = 50;
        settings.dnd_edge_view_scroll.delay_ms = 200;
        settings.dnd_edge_view_scroll.max_speed = 2000;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("gestures {"));
        assert!(kdl.contains("dnd-edge-view-scroll {"));
        assert!(kdl.contains("trigger-width 50"));
        assert!(kdl.contains("delay-ms 200"));
        assert!(kdl.contains("max-speed 2000"));
    }

    #[test]
    fn test_dnd_workspace_switch_disabled() {
        let mut settings = GestureSettings::default();
        settings.dnd_edge_workspace_switch.enabled = false;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("gestures {"));
        assert!(kdl.contains("dnd-edge-workspace-switch {"));
        // niri has no `off`; disable = zero-height trigger zone.
        assert!(kdl.contains("trigger-height 0"));
        assert!(!kdl.contains("off"));
    }

    #[test]
    fn test_dnd_workspace_switch_custom_values() {
        let mut settings = GestureSettings::default();
        settings.dnd_edge_workspace_switch.trigger_size = 100;
        settings.dnd_edge_workspace_switch.delay_ms = 150;
        settings.dnd_edge_workspace_switch.max_speed = 1800;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("gestures {"));
        assert!(kdl.contains("dnd-edge-workspace-switch {"));
        assert!(kdl.contains("trigger-height 100"));
        assert!(kdl.contains("delay-ms 150"));
        assert!(kdl.contains("max-speed 1800"));
    }

    #[test]
    fn test_hot_corners_and_dnd_combined() {
        let mut settings = GestureSettings::default();
        settings.hot_corners = HotCorners {
            enabled: true,
            top_left: true,
            top_right: false,
            bottom_left: false,
            bottom_right: true,
        };
        settings.dnd_edge_view_scroll.trigger_size = 40;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("gestures {"));
        assert!(kdl.contains("hot-corners {"));
        assert!(kdl.contains("top-left"));
        assert!(kdl.contains("bottom-right"));
        assert!(kdl.contains("dnd-edge-view-scroll {"));
        assert!(kdl.contains("trigger-width 40"));
    }

    #[test]
    fn test_partial_dnd_values_only_outputs_changed() {
        let mut settings = GestureSettings::default();
        // Only change delay_ms, keep others at default
        settings.dnd_edge_view_scroll.delay_ms = 250;

        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("dnd-edge-view-scroll {"));
        assert!(kdl.contains("delay-ms 250"));
        // trigger-width and max-speed should not appear (they're at default)
        assert!(!kdl.contains("trigger-width"));
        assert!(!kdl.contains("max-speed"));
    }

    #[test]
    fn dnd_disabled_writes_trigger_zero() {
        let mut settings = GestureSettings::default();
        settings.dnd_edge_view_scroll.enabled = false;
        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("trigger-width 0"));
        assert!(!kdl.contains("off"));

        let loaded = roundtrip(&settings);
        assert!(!loaded.gestures.dnd_edge_view_scroll.enabled);
        // trigger_size untouched (model default 30) so re-enable is sane.
        assert_eq!(loaded.gestures.dnd_edge_view_scroll.trigger_size, 30);
    }

    #[test]
    fn dnd_legacy_off_still_loads() {
        let doc: kdl::KdlDocument = "gestures {\n  dnd-edge-view-scroll {\n    off\n  }\n}\n"
            .parse()
            .unwrap();
        let mut loaded = Settings::default();
        parse_gestures_from_doc(&doc, &mut loaded);
        assert!(!loaded.gestures.dnd_edge_view_scroll.enabled);
    }

    #[test]
    fn dnd_legacy_off_preserves_delay_and_max_speed() {
        // Regression: legacy `off` must not short-circuit reading of the
        // sibling delay-ms/max-speed values (WP-D item 6).
        let doc: kdl::KdlDocument =
            "gestures {\n  dnd-edge-view-scroll {\n    off\n    delay-ms 500\n    max-speed 9000\n  }\n}\n"
                .parse()
                .unwrap();
        let mut loaded = Settings::default();
        parse_gestures_from_doc(&doc, &mut loaded);
        assert!(!loaded.gestures.dnd_edge_view_scroll.enabled);
        assert_eq!(loaded.gestures.dnd_edge_view_scroll.delay_ms, 500);
        assert_eq!(loaded.gestures.dnd_edge_view_scroll.max_speed, 9000);
    }

    #[test]
    fn hot_corners_disabled_roundtrip() {
        let mut settings = GestureSettings::default();
        settings.hot_corners.enabled = false;
        let kdl = generate_gestures_kdl(&settings);
        assert!(kdl.contains("hot-corners {"));
        assert!(kdl.contains("off"));

        let loaded = roundtrip(&settings);
        assert!(!loaded.gestures.hot_corners.enabled);
    }

    #[test]
    fn hot_corners_default_omitted() {
        let settings = GestureSettings::default();
        let kdl = generate_gestures_kdl(&settings);
        assert!(!kdl.contains("hot-corners"));

        // Empty doc yields the niri default (enabled, top-left).
        let doc: kdl::KdlDocument = "".parse().unwrap();
        let mut loaded = Settings::default();
        parse_gestures_from_doc(&doc, &mut loaded);
        assert!(loaded.gestures.hot_corners.enabled);
        assert!(loaded.gestures.hot_corners.top_left);
    }

    #[test]
    fn hot_corners_custom_roundtrip() {
        let mut settings = GestureSettings::default();
        settings.hot_corners = HotCorners {
            enabled: true,
            top_left: false,
            top_right: true,
            bottom_left: false,
            bottom_right: true,
        };
        let loaded = roundtrip(&settings);
        assert_eq!(loaded.gestures.hot_corners, settings.hot_corners);
    }
}
