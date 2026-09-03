//! Snapshot live niri output layout into managed settings, plus arrangement helpers.
//!
//! This is the Config-side API for Displays rearrange. The iced drag canvas stays
//! in UI; call these functions to seed `OutputConfig` from IPC and to compute
//! logical sizes / pack positions.
//!
//! # Source of truth
//!
//! `OutputConfig.position: Option<(i32, i32)>` remains the only stored layout.
//! `None` means niri automatic placement. There is no relative-position file
//! format.
//!
//! # Match / merge policy (`apply_live_outputs_to_settings`)
//!
//! - **Match by connector first**, then by niri `"Make Model Serial"`
//!   (`FullOutputInfo.make_model_serial()`). A config named
//!   `"Dell Inc. U2720Q ABC"` updates the matching live connector instead of
//!   creating a duplicate row. New rows still default to the connector name.
//! - **Connected outputs**: create a row if missing; update `position`, `scale`,
//!   `transform`, and `mode` from live info when present. `enabled` is `true`
//!   when niri reports a logical mapping, `false` when `logical` is `None`
//!   (disabled / unmapped). `vrr` is set from `vrr_enabled` (`On` / `Off`);
//!   `OnDemand` cannot be observed over IPC; an existing `OnDemand` setting is
//!   preserved so snapshot / rearrange does not clobber `on-demand=true`.
//!   Identity fields (background, hot corners, layout override, modeline,
//!   focus-at-startup) are preserved on existing rows.
//! - **Unmatched managed outputs** (configured but not currently connected)
//!   are **left unchanged**. They are not deleted and not auto-disabled, so
//!   dock / TV profiles survive unplug. niri ignores missing connectors.
//!
//! UI should call [`apply_live_outputs_to_settings`] (or
//! `OutputsMessage::ImportConnectedLayout`) when the user asks to import the
//! connected layout.

use crate::config::models::{OutputConfig, OutputSettings};
use crate::ipc::FullOutputInfo;
use crate::types::{logical_size_after_scale_transform, Transform, VrrMode};

/// Summary of a live-layout snapshot apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiveOutputsApplyResult {
    /// New `OutputConfig` rows created for previously unknown connectors.
    pub created: usize,
    /// Existing rows updated from a matching live connector.
    pub updated: usize,
    /// Managed rows with no matching live connector (left unchanged).
    pub unmatched_left_alone: usize,
}

impl LiveOutputsApplyResult {
    /// Short toast / log line for the UI.
    #[must_use]
    pub fn summary(self) -> String {
        format!(
            "Imported connected layout: {} updated, {} added, {} unchanged (not connected)",
            self.updated, self.created, self.unmatched_left_alone
        )
    }
}

/// Copy the current niri output layout into `settings`.
///
/// See the module docs for the match / merge policy. Passing an empty `live`
/// list is a no-op besides counting unmatched rows.
pub fn apply_live_outputs_to_settings(
    settings: &mut OutputSettings,
    live: &[FullOutputInfo],
) -> LiveOutputsApplyResult {
    let mut result = LiveOutputsApplyResult {
        unmatched_left_alone: settings.outputs.len(),
        ..LiveOutputsApplyResult::default()
    };

    for info in live {
        if info.name.is_empty() {
            continue;
        }

        if let Some(idx) = find_config_index_for_live(&settings.outputs, info) {
            update_output_from_live(&mut settings.outputs[idx], info);
            result.updated += 1;
            result.unmatched_left_alone = result.unmatched_left_alone.saturating_sub(1);
        } else {
            settings.outputs.push(output_from_live(info));
            result.created += 1;
        }
    }

    result
}

fn output_from_live(info: &FullOutputInfo) -> OutputConfig {
    let mut output = OutputConfig {
        name: info.name.clone(),
        ..OutputConfig::default()
    };
    update_output_from_live(&mut output, info);
    output
}

fn update_output_from_live(output: &mut OutputConfig, info: &FullOutputInfo) {
    output.enabled = info.logical.is_some();
    if let Some(logical) = &info.logical {
        output.position = Some((logical.x, logical.y));
        output.scale = Some(logical.scale);
        output.transform = info.transform();
    }
    let mode = info.current_mode_string();
    if !mode.is_empty() {
        output.mode = mode;
    }
    // IPC reports only whether VRR is currently active. OnDemand means
    // "sometimes on" and cannot be distinguished from On/Off. Keep it so
    // #22 snapshot / rearrange does not rewrite `on-demand=true` to On/Off.
    if output.vrr != VrrMode::OnDemand {
        output.vrr = if info.vrr_enabled {
            VrrMode::On
        } else {
            VrrMode::Off
        };
    }
}

/// Seed an explicit position when switching an output off automatic placement.
///
/// Order:
/// 1. Keep an already-explicit position.
/// 2. Use the matching live connector's logical `x`/`y` when present.
/// 3. Pack to the right of other enabled outputs that already have an explicit
///    position (using estimated logical sizes).
/// 4. `(0, 0)` only when this is the first explicit output (nothing to stack
///    against).
#[must_use]
pub fn seed_manual_position(
    idx: usize,
    outputs: &[OutputConfig],
    live: &[FullOutputInfo],
) -> (i32, i32) {
    let Some(output) = outputs.get(idx) else {
        return (0, 0);
    };

    if let Some(position) = output.position {
        return position;
    }

    if let Some(info) = find_live_output(&output.name, live) {
        if let Some(logical) = &info.logical {
            return (logical.x, logical.y);
        }
    }

    let occupied: Vec<(i32, i32, u32, u32)> = outputs
        .iter()
        .enumerate()
        .filter(|(other_idx, other)| *other_idx != idx && other.enabled && other.position.is_some())
        .map(|(_, other)| {
            let (x, y) = other.position.expect("filtered to Some");
            let (width, height) = estimated_logical_size(other, live);
            (x, y, width, height)
        })
        .collect();

    pack_to_the_right(&occupied)
}

/// Place a rectangle immediately to the right of the rightmost occupied box,
/// aligned to the top (`min y`) of the occupied set. Empty → `(0, 0)`.
#[must_use]
pub fn pack_to_the_right(occupied: &[(i32, i32, u32, u32)]) -> (i32, i32) {
    if occupied.is_empty() {
        return (0, 0);
    }
    let max_right = occupied
        .iter()
        .map(|(x, _, width, _)| *x + *width as i32)
        .max()
        .unwrap_or(0);
    let min_y = occupied.iter().map(|(_, y, _, _)| *y).min().unwrap_or(0);
    (max_right, min_y)
}

/// Estimated logical size for a configured output, preferring live geometry.
#[must_use]
pub fn estimated_logical_size(output: &OutputConfig, live: &[FullOutputInfo]) -> (u32, u32) {
    if let Some(info) = find_live_output(&output.name, live) {
        return info.logical_size();
    }
    logical_size_from_mode(
        output.mode.as_str(),
        output.display_scale(),
        output.transform,
    )
}

/// Logical size from a mode string (`1920x1080@60`), scale, and transform.
#[must_use]
pub fn logical_size_from_mode(mode: &str, scale: f64, transform: Transform) -> (u32, u32) {
    let (physical_w, physical_h) = parse_mode_resolution(mode).unwrap_or((1920, 1080));
    logical_size_after_scale_transform(physical_w, physical_h, scale, transform)
}

/// Whether a configured `output "name"` matches this live connector.
///
/// Connector is tried first (case-insensitive, niri 0.1.6+), then
/// `"Make Model Serial"`.
#[must_use]
pub fn output_name_matches_live(config_name: &str, info: &FullOutputInfo) -> bool {
    let name = config_name.trim();
    if name.is_empty() {
        return false;
    }
    if name.eq_ignore_ascii_case(info.name.trim()) {
        return true;
    }
    name.eq_ignore_ascii_case(&info.make_model_serial())
}

/// Live IPC row for a configured output name (connector or make/model/serial).
#[must_use]
pub fn find_live_output<'a>(
    config_name: &str,
    live: &'a [FullOutputInfo],
) -> Option<&'a FullOutputInfo> {
    live.iter()
        .find(|info| info.name.eq_ignore_ascii_case(config_name.trim()))
        .or_else(|| {
            live.iter()
                .find(|info| output_name_matches_live(config_name, info))
        })
}

/// Config index matching a live output: connector first, then make/model/serial.
#[must_use]
pub fn find_config_index_for_live(
    outputs: &[OutputConfig],
    info: &FullOutputInfo,
) -> Option<usize> {
    outputs
        .iter()
        .position(|output| {
            !output.name.is_empty() && output.name.eq_ignore_ascii_case(info.name.trim())
        })
        .or_else(|| {
            outputs.iter().position(|output| {
                !output.name.is_empty() && output_name_matches_live(&output.name, info)
            })
        })
}

/// Parse `WxH` or `WxH@refresh` into physical pixels.
#[must_use]
pub fn parse_mode_resolution(mode: &str) -> Option<(i32, i32)> {
    let at_idx = mode.find('@').unwrap_or(mode.len());
    let res = mode.get(..at_idx)?;
    let mut parts = res.split('x');
    let width = parts.next()?.parse().ok()?;
    let height = parts.next()?.parse().ok()?;
    if width <= 0 || height <= 0 {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{OutputLogical, OutputMode};

    fn live(
        name: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        scale: f64,
        mode_w: i32,
        mode_h: i32,
    ) -> FullOutputInfo {
        FullOutputInfo {
            name: name.to_string(),
            current_mode: Some(0),
            modes: vec![OutputMode {
                width: mode_w,
                height: mode_h,
                refresh_rate: 60000,
                is_preferred: true,
            }],
            logical: Some(OutputLogical {
                x,
                y,
                width: Some(width),
                height: Some(height),
                scale,
                transform: "Normal".to_string(),
            }),
            ..Default::default()
        }
    }

    fn named(name: &str) -> OutputConfig {
        OutputConfig {
            name: name.to_string(),
            ..OutputConfig::default()
        }
    }

    #[test]
    fn snapshot_applies_logical_positions_and_creates_missing_rows() {
        let mut settings = OutputSettings {
            outputs: vec![named("eDP-1")],
        };
        let live_outputs = vec![
            live("eDP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080),
            live("DP-1", 1920, 0, 2560, 1440, 1.0, 2560, 1440),
        ];

        let result = apply_live_outputs_to_settings(&mut settings, &live_outputs);

        assert_eq!(result.created, 1);
        assert_eq!(result.updated, 1);
        assert_eq!(result.unmatched_left_alone, 0);
        assert_eq!(settings.outputs.len(), 2);
        assert_eq!(settings.outputs[0].name, "eDP-1");
        assert_eq!(settings.outputs[0].position, Some((0, 0)));
        assert_eq!(settings.outputs[0].scale, Some(1.0));
        assert_eq!(settings.outputs[0].mode, "1920x1080@60.00");
        assert!(settings.outputs[0].enabled);
        assert_eq!(settings.outputs[1].name, "DP-1");
        assert_eq!(settings.outputs[1].position, Some((1920, 0)));
        assert_eq!(settings.outputs[1].scale, Some(1.0));
        assert_eq!(settings.outputs[1].mode, "2560x1440@60.00");
    }

    #[test]
    fn snapshot_leaves_unmatched_managed_outputs_alone() {
        let mut dock = named("DP-3");
        dock.position = Some((5000, 5000));
        dock.enabled = true;
        dock.mode = "3840x2160@60.00".to_string();
        let mut settings = OutputSettings {
            outputs: vec![named("eDP-1"), dock],
        };

        let result = apply_live_outputs_to_settings(
            &mut settings,
            &[live("eDP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080)],
        );

        assert_eq!(result.updated, 1);
        assert_eq!(result.created, 0);
        assert_eq!(result.unmatched_left_alone, 1);
        assert_eq!(settings.outputs[1].name, "DP-3");
        assert_eq!(settings.outputs[1].position, Some((5000, 5000)));
        assert!(settings.outputs[1].enabled);
        assert_eq!(settings.outputs[1].mode, "3840x2160@60.00");
    }

    #[test]
    fn snapshot_sets_scale_transform_mode_from_live() {
        let mut settings = OutputSettings {
            outputs: vec![named("HDMI-A-1")],
        };
        let mut info = live("HDMI-A-1", 2560, 0, 1080, 1920, 1.5, 1920, 1080);
        info.logical.as_mut().unwrap().transform = "90".to_string();
        info.vrr_enabled = true;

        apply_live_outputs_to_settings(&mut settings, &[info]);

        let output = &settings.outputs[0];
        assert_eq!(output.position, Some((2560, 0)));
        assert_eq!(output.scale, Some(1.5));
        assert_eq!(output.transform, Transform::Rotate90);
        assert_eq!(output.vrr, VrrMode::On);
        assert_eq!(output.mode, "1920x1080@60.00");
    }

    #[test]
    fn snapshot_marks_unmapped_live_output_disabled_without_clobbering_position() {
        let mut existing = named("DP-1");
        existing.position = Some((1920, 0));
        existing.scale = Some(1.25);
        let mut settings = OutputSettings {
            outputs: vec![existing],
        };
        let live_disabled = FullOutputInfo {
            name: "DP-1".to_string(),
            logical: None,
            ..Default::default()
        };

        apply_live_outputs_to_settings(&mut settings, &[live_disabled]);

        assert!(!settings.outputs[0].enabled);
        assert_eq!(settings.outputs[0].position, Some((1920, 0)));
        assert_eq!(settings.outputs[0].scale, Some(1.25));
    }

    #[test]
    fn snapshot_preserves_vrr_on_demand() {
        let mut existing = named("DP-1");
        existing.vrr = VrrMode::OnDemand;
        let mut settings = OutputSettings {
            outputs: vec![existing],
        };

        let mut live_on = live("DP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080);
        live_on.vrr_enabled = true;
        apply_live_outputs_to_settings(&mut settings, &[live_on]);
        assert_eq!(settings.outputs[0].vrr, VrrMode::OnDemand);

        let live_off = live("DP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080);
        apply_live_outputs_to_settings(&mut settings, &[live_off]);
        assert_eq!(
            settings.outputs[0].vrr,
            VrrMode::OnDemand,
            "IPC cannot observe on-demand; must not clobber to On/Off"
        );
    }

    #[test]
    fn snapshot_preserves_live_scale_1() {
        // Live logical.scale is often 1.0. That is an observed value and must
        // become Some(1.0) so the next save writes `scale 1.0` instead of
        // dropping it (niri would then auto-guess, often 2× on HiDPI).
        let mut settings = OutputSettings {
            outputs: vec![named("eDP-1")],
        };
        apply_live_outputs_to_settings(
            &mut settings,
            &[live("eDP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080)],
        );
        assert_eq!(settings.outputs[0].scale, Some(1.0));

        let kdl = crate::config::storage::generate_outputs_kdl(&settings);
        assert!(
            kdl.contains("scale 1.0") || kdl.contains("scale 1\n"),
            "live 1.0 must be written after snapshot, got:\n{kdl}"
        );
    }

    #[test]
    fn auto_to_manual_seeds_from_live_logical_not_origin() {
        let outputs = vec![named("eDP-1"), named("DP-1")];
        let live_outputs = vec![
            live("eDP-1", 0, 0, 1920, 1080, 1.0, 1920, 1080),
            live("DP-1", 1920, 240, 2560, 1440, 1.0, 2560, 1440),
        ];

        assert_eq!(seed_manual_position(0, &outputs, &live_outputs), (0, 0));
        assert_eq!(
            seed_manual_position(1, &outputs, &live_outputs),
            (1920, 240)
        );
    }

    #[test]
    fn auto_to_manual_packs_right_when_live_has_no_logical() {
        let mut left = named("eDP-1");
        left.position = Some((0, 0));
        left.mode = "1920x1080@60".to_string();
        let right = named("DP-1");
        let outputs = vec![left, right];

        // No live logical for DP-1 (and no live entry at all).
        assert_eq!(seed_manual_position(1, &outputs, &[]), (1920, 0));
    }

    #[test]
    fn auto_to_manual_does_not_stack_multiple_outputs_at_origin() {
        let mut first = named("eDP-1");
        first.position = Some((0, 0));
        first.mode = "1920x1080".to_string();
        let mut second = named("DP-1");
        second.position = Some((1920, 0));
        second.mode = "2560x1440".to_string();
        let third = named("HDMI-A-1");
        let outputs = vec![first, second, third];

        let seeded = seed_manual_position(2, &outputs, &[]);
        assert_eq!(seeded, (4480, 0));
        assert_ne!(seeded, (0, 0));
    }

    #[test]
    fn seed_keeps_existing_explicit_position() {
        let mut output = named("DP-1");
        output.position = Some((100, 200));
        let live_outputs = [live("DP-1", 1920, 0, 2560, 1440, 1.0, 2560, 1440)];
        assert_eq!(
            seed_manual_position(0, &[output], &live_outputs),
            (100, 200)
        );
    }

    #[test]
    fn pack_to_the_right_empty_is_origin() {
        assert_eq!(pack_to_the_right(&[]), (0, 0));
    }

    fn identified(
        connector: &str,
        make: &str,
        model: &str,
        serial: &str,
        x: i32,
        y: i32,
    ) -> FullOutputInfo {
        let mut info = live(connector, x, y, 2560, 1440, 1.0, 2560, 1440);
        info.make = make.to_string();
        info.model = model.to_string();
        info.serial = Some(serial.to_string());
        info
    }

    #[test]
    fn match_prefers_connector_then_make_model_serial() {
        let info = identified("DP-1", "Dell Inc.", "U2720Q", "ABC123", 1920, 0);
        assert_eq!(info.make_model_serial(), "Dell Inc. U2720Q ABC123");
        assert!(output_name_matches_live("DP-1", &info));
        assert!(output_name_matches_live("dp-1", &info));
        assert!(output_name_matches_live("Dell Inc. U2720Q ABC123", &info));
        assert!(output_name_matches_live("dell inc. u2720q abc123", &info));
        assert!(!output_name_matches_live("HDMI-A-1", &info));
        assert!(!output_name_matches_live("Dell Inc. U2720Q OTHER", &info));
        assert!(!output_name_matches_live("", &info));
    }

    #[test]
    fn find_helpers_connector_wins_over_mms_duplicate() {
        let info = identified("DP-1", "Dell Inc.", "U2720Q", "ABC123", 1920, 0);
        let outputs = vec![named("Dell Inc. U2720Q ABC123"), named("DP-1")];
        // Connector match is preferred when both exist.
        assert_eq!(find_config_index_for_live(&outputs, &info), Some(1));
        assert_eq!(
            find_live_output("Dell Inc. U2720Q ABC123", &[info.clone()]).map(|i| i.name.as_str()),
            Some("DP-1")
        );
    }

    #[test]
    fn snapshot_matches_make_model_serial_without_duplicate() {
        let mut existing = named("Dell Inc. U2720Q ABC123");
        existing.position = Some((10, 20));
        existing.background_color = Some(crate::types::Color::from_hex("#112233").unwrap());
        let mut settings = OutputSettings {
            outputs: vec![existing],
        };
        let result = apply_live_outputs_to_settings(
            &mut settings,
            &[identified(
                "DP-1",
                "Dell Inc.",
                "U2720Q",
                "ABC123",
                1920,
                240,
            )],
        );

        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 1);
        assert_eq!(result.unmatched_left_alone, 0);
        assert_eq!(settings.outputs.len(), 1);
        assert_eq!(settings.outputs[0].name, "Dell Inc. U2720Q ABC123");
        assert_eq!(settings.outputs[0].position, Some((1920, 240)));
        assert_eq!(settings.outputs[0].scale, Some(1.0));
        assert_eq!(
            settings.outputs[0].background_color,
            Some(crate::types::Color::from_hex("#112233").unwrap())
        );
    }

    #[test]
    fn seed_and_size_use_mms_identity() {
        let output = named("Dell Inc. U2720Q ABC123");
        let live_outputs = [identified(
            "HDMI-A-1",
            "Dell Inc.",
            "U2720Q",
            "ABC123",
            2560,
            80,
        )];
        assert_eq!(
            seed_manual_position(0, &[output.clone()], &live_outputs),
            (2560, 80)
        );
        assert_eq!(estimated_logical_size(&output, &live_outputs), (2560, 1440));
    }

    #[test]
    fn logical_size_helpers_apply_scale_and_rotation() {
        assert_eq!(
            logical_size_from_mode("3840x2160@60", 2.0, Transform::Normal),
            (1920, 1080)
        );
        assert_eq!(
            logical_size_from_mode("1920x1080", 1.0, Transform::Rotate90),
            (1080, 1920)
        );
    }
}
