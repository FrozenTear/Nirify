//! Logical-pixel layout helpers for the Displays arrangement canvas.
//!
//! niri's `output { position x= y= }` is in **logical** pixels. Sizes and
//! auto→manual seeding come from Config (`estimated_logical_size`,
//! `pack_to_the_right`, `FullOutputInfo::logical_size`). Auto-positioned
//! outputs must not stack at `(0, 0)` in the preview — use live IPC
//! `logical.x/y` when available, otherwise pack with [`pack_to_the_right`].

use crate::config::models::{OutputConfig, OutputSettings};
use crate::config::{estimated_logical_size, pack_to_the_right};
use crate::ipc::FullOutputInfo;
use crate::types::VrrMode;

/// Max preview height used by the Displays canvas (shared with hit-testing).
pub const PREVIEW_MAX_HEIGHT: f32 = 200.0;
/// Max preview width used by the Displays canvas (shared with hit-testing).
pub const PREVIEW_MAX_WIDTH: f32 = 960.0;
/// Edge-align snap threshold in logical pixels.
pub const SNAP_THRESHOLD: i32 = 32;

/// A monitor rectangle in niri logical coordinates, ready for preview/hit-test.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorRect {
    /// Index in `OutputSettings.outputs` when this row is configured.
    pub config_index: Option<usize>,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
    /// `true` when config omits `position` (niri auto-places).
    pub is_auto: bool,
    pub focus_at_startup: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewLayout {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub monitors: Vec<PreviewMonitor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewMonitor {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub rect: MonitorRect,
}

/// Build an `OutputConfig` from a connected niri output (auto-positioned).
#[must_use]
pub fn output_from_ipc(info: &FullOutputInfo) -> OutputConfig {
    OutputConfig {
        name: info.name.clone(),
        enabled: true,
        scale: info.logical.as_ref().map(|logical| logical.scale),
        mode: info.current_mode_string(),
        transform: info.transform(),
        vrr: if info.vrr_enabled {
            VrrMode::On
        } else {
            VrrMode::Off
        },
        position: None,
        ..Default::default()
    }
}

/// Connected IPC outputs that do not yet have a matching config entry.
#[must_use]
pub fn unconfigured_outputs<'a>(
    configured: &[OutputConfig],
    available: &'a [FullOutputInfo],
) -> Vec<&'a FullOutputInfo> {
    available
        .iter()
        .filter(|info| {
            !configured
                .iter()
                .any(|output| output.name == info.name && !output.name.is_empty())
        })
        .collect()
}

/// Logical size for a configured output, preferring live IPC geometry.
#[must_use]
pub fn configured_logical_size(output: &OutputConfig, available: &[FullOutputInfo]) -> (i32, i32) {
    let (width, height) = estimated_logical_size(output, available);
    (width as i32, height as i32)
}

/// Collect configured + connected monitors in logical space for the canvas.
#[must_use]
pub fn collect_monitors(
    outputs: &OutputSettings,
    available: &[FullOutputInfo],
) -> Vec<MonitorRect> {
    let mut monitors: Vec<MonitorRect> = Vec::new();
    let mut pack_later: Vec<MonitorRect> = Vec::new();

    for (idx, output) in outputs.outputs.iter().enumerate() {
        let ipc = available.iter().find(|info| info.name == output.name);
        let (width, height) = configured_logical_size(output, available);
        let name = if output.name.is_empty() {
            format!("Output {}", idx + 1)
        } else {
            output.name.clone()
        };

        let mut rect = MonitorRect {
            config_index: Some(idx),
            name,
            x: 0,
            y: 0,
            width,
            height,
            enabled: output.enabled,
            is_auto: output.position.is_none(),
            focus_at_startup: output.focus_at_startup,
        };

        if let Some((x, y)) = output.position {
            rect.x = x;
            rect.y = y;
            monitors.push(rect);
        } else if let Some(info) = ipc {
            rect.x = info.position_x();
            rect.y = info.position_y();
            monitors.push(rect);
        } else {
            pack_later.push(rect);
        }
    }

    for info in available {
        if monitors.iter().any(|monitor| monitor.name == info.name)
            || pack_later.iter().any(|monitor| monitor.name == info.name)
        {
            continue;
        }
        let (width, height) = info.logical_size();
        monitors.push(MonitorRect {
            config_index: None,
            name: info.name.clone(),
            x: info.position_x(),
            y: info.position_y(),
            width: width as i32,
            height: height as i32,
            enabled: true,
            is_auto: true,
            focus_at_startup: false,
        });
    }

    pack_unpositioned(&mut monitors, pack_later);
    monitors
}

fn pack_unpositioned(monitors: &mut Vec<MonitorRect>, pack_later: Vec<MonitorRect>) {
    for mut rect in pack_later {
        let occupied: Vec<(i32, i32, u32, u32)> = monitors
            .iter()
            .map(|monitor| {
                (
                    monitor.x,
                    monitor.y,
                    monitor.width.max(0) as u32,
                    monitor.height.max(0) as u32,
                )
            })
            .collect();
        let (x, y) = pack_to_the_right(&occupied);
        rect.x = x;
        rect.y = y;
        monitors.push(rect);
    }
}

#[must_use]
pub fn monitor_bounds(monitors: &[MonitorRect]) -> Option<MonitorBounds> {
    Some(MonitorBounds {
        min_x: monitors.iter().map(|monitor| monitor.x).min()?,
        min_y: monitors.iter().map(|monitor| monitor.y).min()?,
        max_x: monitors
            .iter()
            .map(|monitor| monitor.x + monitor.width)
            .max()?,
        max_y: monitors
            .iter()
            .map(|monitor| monitor.y + monitor.height)
            .max()?,
    })
}

#[must_use]
pub fn compute_preview_layout(
    monitors: &[MonitorRect],
    max_height: f32,
    max_width: f32,
) -> Option<PreviewLayout> {
    let bounds = monitor_bounds(monitors)?;
    let total_width = (bounds.max_x - bounds.min_x) as f32;
    let total_height = (bounds.max_y - bounds.min_y) as f32;

    if total_width <= 0.0 || total_height <= 0.0 {
        return None;
    }

    let scale = (max_height / total_height).min(max_width / total_width);
    let preview_monitors = monitors
        .iter()
        .cloned()
        .map(|rect| PreviewMonitor {
            left: (rect.x - bounds.min_x) as f32 * scale,
            top: (rect.y - bounds.min_y) as f32 * scale,
            width: rect.width as f32 * scale,
            height: rect.height as f32 * scale,
            rect,
        })
        .collect();

    Some(PreviewLayout {
        width: total_width * scale,
        height: total_height * scale,
        scale,
        origin_x: bounds.min_x,
        origin_y: bounds.min_y,
        monitors: preview_monitors,
    })
}

/// Hit-test a canvas-local point (later-drawn monitors win).
#[must_use]
pub fn hit_test(layout: &PreviewLayout, canvas_x: f32, canvas_y: f32) -> Option<&PreviewMonitor> {
    layout.monitors.iter().rev().find(|preview| {
        canvas_x >= preview.left
            && canvas_y >= preview.top
            && canvas_x <= preview.left + preview.width
            && canvas_y <= preview.top + preview.height
    })
}

/// Snap `(x, y)` so this monitor's edges align with others.
#[must_use]
pub fn snap_position(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    others: &[(i32, i32, i32, i32)],
    threshold: i32,
) -> (i32, i32) {
    let mut snapped_x = x;
    let mut snapped_y = y;
    let mut best_dx = threshold + 1;
    let mut best_dy = threshold + 1;

    for &(ox, oy, ow, oh) in others {
        for candidate in [ox, ox + ow, ox - width, ox + ow - width] {
            let d = (x - candidate).abs();
            if d < best_dx && d <= threshold {
                best_dx = d;
                snapped_x = candidate;
            }
        }
        for candidate in [oy, oy + oh, oy - height, oy + oh - height] {
            let d = (y - candidate).abs();
            if d < best_dy && d <= threshold {
                best_dy = d;
                snapped_y = candidate;
            }
        }
    }

    (snapped_x, snapped_y)
}

/// Logical size of the arrangement used by the Displays header stat.
#[must_use]
pub fn calculate_canvas_size(outputs: &OutputSettings, available: &[FullOutputInfo]) -> (i32, i32) {
    let monitors: Vec<MonitorRect> = collect_monitors(outputs, available)
        .into_iter()
        .filter(|monitor| monitor.enabled)
        .collect();
    monitor_bounds(&monitors)
        .map(|bounds| (bounds.max_x - bounds.min_x, bounds.max_y - bounds.min_y))
        .unwrap_or((1920, 1080))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::OutputConfig;
    use crate::ipc::{FullOutputInfo, OutputLogical, OutputMode};
    use crate::types::Transform;

    fn ipc_output(
        name: &str,
        x: i32,
        y: i32,
        scale: f64,
        transform: &str,
        w: i32,
        h: i32,
    ) -> FullOutputInfo {
        FullOutputInfo {
            name: name.to_string(),
            make: "Dell".to_string(),
            model: "U2720Q".to_string(),
            current_mode: Some(0),
            modes: vec![OutputMode {
                width: w,
                height: h,
                refresh_rate: 60000,
                is_preferred: true,
            }],
            logical: Some(OutputLogical {
                x,
                y,
                scale,
                transform: transform.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn canvas_prefers_live_reported_logical_size() {
        let outputs = OutputSettings {
            outputs: vec![OutputConfig {
                name: "DP-1".to_string(),
                mode: "3840x2160@60.00".to_string(),
                scale: Some(1.0),
                position: Some((0, 0)),
                ..Default::default()
            }],
        };
        let mut info = ipc_output("DP-1", 0, 0, 1.0, "Normal", 3840, 2160);
        if let Some(logical) = info.logical.as_mut() {
            logical.width = Some(1920);
            logical.height = Some(1080);
        }
        assert_eq!(calculate_canvas_size(&outputs, &[info]), (1920, 1080));
    }

    #[test]
    fn canvas_size_uses_logical_not_physical_pixels() {
        let outputs = OutputSettings {
            outputs: vec![OutputConfig {
                name: "DP-1".to_string(),
                mode: "3840x2160@60.00".to_string(),
                scale: Some(2.0),
                position: Some((0, 0)),
                ..Default::default()
            }],
        };
        assert_eq!(calculate_canvas_size(&outputs, &[]), (1920, 1080));
    }

    #[test]
    fn canvas_size_swaps_on_90_transform() {
        let outputs = OutputSettings {
            outputs: vec![OutputConfig {
                name: "DP-1".to_string(),
                mode: "1920x1080@60.00".to_string(),
                scale: Some(1.0),
                transform: Transform::Rotate90,
                position: Some((0, 0)),
                ..Default::default()
            }],
        };
        assert_eq!(calculate_canvas_size(&outputs, &[]), (1080, 1920));
    }

    #[test]
    fn calculate_canvas_size_includes_negative_coordinates() {
        let outputs = OutputSettings {
            outputs: vec![
                OutputConfig {
                    name: "DP-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: Some((-1920, 0)),
                    ..Default::default()
                },
                OutputConfig {
                    name: "HDMI-A-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: Some((0, 0)),
                    ..Default::default()
                },
            ],
        };
        assert_eq!(calculate_canvas_size(&outputs, &[]), (3840, 1080));
    }

    #[test]
    fn auto_outputs_use_ipc_logical_positions() {
        let outputs = OutputSettings {
            outputs: vec![
                OutputConfig {
                    name: "DP-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: None,
                    ..Default::default()
                },
                OutputConfig {
                    name: "HDMI-A-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: None,
                    ..Default::default()
                },
            ],
        };
        let available = vec![
            ipc_output("DP-1", 0, 0, 1.0, "Normal", 1920, 1080),
            ipc_output("HDMI-A-1", 1920, 0, 1.0, "Normal", 1920, 1080),
        ];
        let monitors = collect_monitors(&outputs, &available);
        assert!(monitors.iter().all(|m| m.is_auto));
        let dp = monitors.iter().find(|m| m.name == "DP-1").unwrap();
        let hdmi = monitors.iter().find(|m| m.name == "HDMI-A-1").unwrap();
        assert_eq!((dp.x, dp.y), (0, 0));
        assert_eq!((hdmi.x, hdmi.y), (1920, 0));
        assert_ne!((dp.x, dp.y), (hdmi.x, hdmi.y));
    }

    #[test]
    fn auto_outputs_without_ipc_are_packed_not_stacked() {
        let outputs = OutputSettings {
            outputs: vec![
                OutputConfig {
                    name: "DP-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: None,
                    ..Default::default()
                },
                OutputConfig {
                    name: "HDMI-A-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position: None,
                    ..Default::default()
                },
            ],
        };
        let monitors = collect_monitors(&outputs, &[]);
        assert_eq!(monitors.len(), 2);
        assert_eq!((monitors[0].x, monitors[0].y), (0, 0));
        assert_eq!((monitors[1].x, monitors[1].y), (1920, 0));
        assert!(monitors.iter().all(|m| m.is_auto));
    }

    #[test]
    fn snap_aligns_right_edge_to_neighbor_left() {
        let others = [(1920, 0, 1920, 1080)];
        // 20px left of a flush left-of-neighbor placement (x=0) and 8px off the top.
        let (x, y) = snap_position(20, 8, 1920, 1080, &others, SNAP_THRESHOLD);
        assert_eq!((x, y), (0, 0));
    }

    #[test]
    fn hit_test_prefers_later_monitor() {
        let monitors = vec![
            MonitorRect {
                config_index: Some(0),
                name: "A".into(),
                x: 0,
                y: 0,
                width: 100,
                height: 100,
                enabled: true,
                is_auto: false,
                focus_at_startup: false,
            },
            MonitorRect {
                config_index: Some(1),
                name: "B".into(),
                x: 50,
                y: 50,
                width: 100,
                height: 100,
                enabled: true,
                is_auto: false,
                focus_at_startup: false,
            },
        ];
        let layout = compute_preview_layout(&monitors, 200.0, 960.0).unwrap();
        let hit = hit_test(
            &layout,
            layout.monitors[1].left + 2.0,
            layout.monitors[1].top + 2.0,
        )
        .unwrap();
        assert_eq!(hit.rect.name, "B");
    }

    #[test]
    fn unconfigured_filters_already_named() {
        let configured = vec![OutputConfig {
            name: "DP-1".into(),
            ..Default::default()
        }];
        let available = vec![
            ipc_output("DP-1", 0, 0, 1.0, "Normal", 1920, 1080),
            ipc_output("HDMI-A-1", 1920, 0, 1.0, "Normal", 1920, 1080),
        ];
        let leftover = unconfigured_outputs(&configured, &available);
        assert_eq!(leftover.len(), 1);
        assert_eq!(leftover[0].name, "HDMI-A-1");
    }

    #[test]
    fn output_from_ipc_is_auto_positioned_and_named() {
        let info = ipc_output("eDP-1", 100, 200, 2.0, "90", 3840, 2160);
        let cfg = output_from_ipc(&info);
        assert_eq!(cfg.name, "eDP-1");
        assert!(cfg.position.is_none());
        assert_eq!(cfg.scale, Some(2.0));
        assert_eq!(cfg.transform, Transform::Rotate90);
        assert_eq!(cfg.mode, "3840x2160@60.00");
    }

    #[test]
    fn compute_preview_layout_preserves_vertical_offsets() {
        let monitors = vec![
            MonitorRect {
                config_index: Some(0),
                name: "DP-1".to_string(),
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                enabled: true,
                is_auto: false,
                focus_at_startup: true,
            },
            MonitorRect {
                config_index: Some(1),
                name: "HDMI-A-1".to_string(),
                x: 0,
                y: 1080,
                width: 1920,
                height: 1080,
                enabled: true,
                is_auto: false,
                focus_at_startup: false,
            },
        ];

        let layout = compute_preview_layout(&monitors, 200.0, 960.0).unwrap();
        let top_monitor = layout
            .monitors
            .iter()
            .find(|monitor| monitor.rect.name == "DP-1")
            .unwrap();
        let bottom_monitor = layout
            .monitors
            .iter()
            .find(|monitor| monitor.rect.name == "HDMI-A-1")
            .unwrap();

        assert_eq!(top_monitor.left, bottom_monitor.left);
        assert!(bottom_monitor.top > top_monitor.top);
    }
}
