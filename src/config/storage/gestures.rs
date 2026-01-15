//! Gesture settings KDL generation
//!
//! Generates KDL for hot corners and DND settings.

use super::builder::KdlBuilder;
use crate::config::models::GestureSettings;

/// Generate gestures.kdl content from settings.
///
/// Creates KDL configuration for gestures including:
/// - Hot corners
/// - DND edge view scroll settings
/// - DND workspace switch settings
pub fn generate_gestures_kdl(settings: &GestureSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Gesture settings - managed by niri-settings-rust");

    // Hot corners - must be inside gestures block, corners are simple flags
    if settings.hot_corners.enabled {
        let has_any_corner = settings.hot_corners.top_left
            || settings.hot_corners.top_right
            || settings.hot_corners.bottom_left
            || settings.hot_corners.bottom_right;

        if has_any_corner {
            kdl.block("gestures", |g| {
                g.block("hot-corners", |b| {
                    b.optional_flag("top-left", settings.hot_corners.top_left);
                    b.optional_flag("top-right", settings.hot_corners.top_right);
                    b.optional_flag("bottom-left", settings.hot_corners.bottom_left);
                    b.optional_flag("bottom-right", settings.hot_corners.bottom_right);
                });
            });
        }
    }

    // TODO: DND settings - commented out as 'dnd' is not a valid top-level node in niri
    // These settings may need to go elsewhere or be removed entirely
    // DND edge view scroll and workspace switch settings are stored but not output

    kdl.build()
}
