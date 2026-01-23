//! Gesture settings KDL generation
//!
//! Generates KDL for hot corners and DND settings.

use super::builder::KdlBuilder;
use crate::config::models::{DndEdgeSettings, GestureSettings};

/// Generate gestures.kdl content from settings.
///
/// Creates KDL configuration for gestures including:
/// - Hot corners
/// - DND edge view scroll settings
/// - DND workspace switch settings
pub fn generate_gestures_kdl(settings: &GestureSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Gesture settings - managed by Nirify");

    // Check if we have any gesture settings to output
    let has_hot_corners = settings.hot_corners.enabled
        && (settings.hot_corners.top_left
            || settings.hot_corners.top_right
            || settings.hot_corners.bottom_left
            || settings.hot_corners.bottom_right);

    let has_dnd_view_scroll = has_dnd_edge_settings(
        &settings.dnd_edge_view_scroll,
        &DndEdgeSettings::default_scroll(),
    );

    let has_dnd_workspace_switch = has_dnd_edge_settings(
        &settings.dnd_edge_workspace_switch,
        &DndEdgeSettings::default_workspace(),
    );

    // Only output gestures block if there's something to output
    if has_hot_corners || has_dnd_view_scroll || has_dnd_workspace_switch {
        kdl.block("gestures", |g| {
            // Hot corners
            if has_hot_corners {
                g.block("hot-corners", |b| {
                    b.optional_flag("top-left", settings.hot_corners.top_left);
                    b.optional_flag("top-right", settings.hot_corners.top_right);
                    b.optional_flag("bottom-left", settings.hot_corners.bottom_left);
                    b.optional_flag("bottom-right", settings.hot_corners.bottom_right);
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
    // Output if disabled (need to add "off" flag) or if any value differs from default
    !settings.enabled
        || settings.trigger_size != defaults.trigger_size
        || settings.delay_ms != defaults.delay_ms
        || settings.max_speed != defaults.max_speed
}

/// Generate KDL for dnd-edge-view-scroll block
fn generate_dnd_edge_view_scroll(builder: &mut KdlBuilder, settings: &DndEdgeSettings) {
    let defaults = DndEdgeSettings::default_scroll();
    builder.block("dnd-edge-view-scroll", |b| {
        if !settings.enabled {
            b.flag("off");
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

/// Generate KDL for dnd-edge-workspace-switch block
fn generate_dnd_edge_workspace_switch(builder: &mut KdlBuilder, settings: &DndEdgeSettings) {
    let defaults = DndEdgeSettings::default_workspace();
    builder.block("dnd-edge-workspace-switch", |b| {
        if !settings.enabled {
            b.flag("off");
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
mod tests {
    use super::*;
    use crate::config::models::HotCorners;

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
        assert!(kdl.contains("off"));
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
        assert!(kdl.contains("off"));
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
}
