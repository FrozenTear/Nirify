//! Populate functions for layer rules UI models
//!
//! Functions that create and populate UI model data from LayerRule settings.

use crate::config::models::{LayerRule, LayerRuleMatch};
use crate::LayerRuleSettingModel;
use crate::MainWindow;
use slint::{ModelRc, SharedString, VecModel};

use super::super::rules_common;

// Generate helper functions for LayerRuleSettingModel
crate::impl_setting_builders!(LayerRuleSettingModel);

/// Build rule list model for UI display
pub fn build_rule_list_model(rules: &[LayerRule]) -> ModelRc<SharedString> {
    rules_common::build_names_list(rules)
}

/// Build matches list model for UI display
pub fn build_matches_list_model(matches: &[LayerRuleMatch]) -> ModelRc<SharedString> {
    let mut labels = Vec::with_capacity(matches.len());
    for (i, m) in matches.iter().enumerate() {
        let ns = m.namespace.as_deref().unwrap_or("*");
        labels.push(format!("{}. namespace={}", i + 1, ns).into());
    }
    ModelRc::new(VecModel::from(labels))
}

/// Populate rule settings model
pub fn populate_rule_settings(rule: &LayerRule) -> ModelRc<LayerRuleSettingModel> {
    let settings = vec![make_text(
        "rule_name",
        "Rule name",
        "Display name for this rule",
        &rule.name,
        "Layer Rule",
        true,
    )];
    ModelRc::new(VecModel::from(settings))
}

/// Populate match settings model for the selected match
pub fn populate_match_settings(rule: &LayerRule, match_idx: usize) -> ModelRc<LayerRuleSettingModel> {
    let m = rule.matches.get(match_idx);
    let namespace = m
        .map(|m| m.namespace.as_deref().unwrap_or(""))
        .unwrap_or("");
    let has_at_startup = m.map(|m| m.at_startup.is_some()).unwrap_or(false);
    let at_startup = m.map(|m| m.at_startup.unwrap_or(true)).unwrap_or(true);

    let settings = vec![
        make_text(
            "match_namespace",
            "Namespace",
            "Layer-shell namespace (regex supported)",
            namespace,
            "e.g., waybar",
            true,
        ),
        make_toggle(
            "match_has_at_startup",
            "Filter by startup state",
            "Only match layers at startup or after",
            has_at_startup,
            true,
        ),
        make_toggle(
            "match_at_startup",
            "At startup",
            "Match only layers created at compositor startup",
            at_startup,
            has_at_startup,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate visual settings model
pub fn populate_visual_settings(rule: &LayerRule) -> ModelRc<LayerRuleSettingModel> {
    let has_opacity = rule.opacity.is_some();
    let opacity = rule.opacity.unwrap_or(1.0);
    let has_corner_radius = rule.geometry_corner_radius.is_some();
    let corner_radius = rule.geometry_corner_radius.unwrap_or(12);
    let has_block_out = rule.block_out_from.is_some();
    let block_out_index = rule
        .block_out_from
        .as_ref()
        .map(|b| b.to_index())
        .unwrap_or(0);

    let settings = vec![
        make_toggle(
            "has_opacity",
            "Custom opacity",
            "Set layer transparency",
            has_opacity,
            true,
        ),
        make_slider_float(
            "opacity",
            "Opacity",
            "Layer transparency level",
            opacity,
            0.0,
            1.0,
            "%",
            has_opacity,
        ),
        make_toggle(
            "has_corner_radius",
            "Geometry corner radius",
            "Round layer surface corners",
            has_corner_radius,
            true,
        ),
        make_slider_int(
            "corner_radius",
            "Radius",
            "Corner radius in logical pixels",
            corner_radius,
            0.0,
            32.0,
            "px",
            has_corner_radius,
        ),
        make_toggle(
            "has_block_out_from",
            "Block from screen capture",
            "Hide this layer in screen recordings",
            has_block_out,
            true,
        ),
        make_combo(
            "block_out_from",
            "Block from",
            "What to block the layer from",
            block_out_index,
            &["Screencast", "Screen Capture"],
            has_block_out,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate shadow settings model
pub fn populate_shadow_settings(rule: &LayerRule) -> ModelRc<LayerRuleSettingModel> {
    let has_shadow = rule.shadow.is_some();
    let shadow = rule.shadow.as_ref();

    let enabled = shadow.map(|s| s.enabled).unwrap_or(true);
    let softness = shadow.map(|s| s.softness).unwrap_or(30);
    let spread = shadow.map(|s| s.spread).unwrap_or(5);
    let offset_x = shadow.map(|s| s.offset_x).unwrap_or(0);
    let offset_y = shadow.map(|s| s.offset_y).unwrap_or(5);
    let draw_behind = shadow.map(|s| s.draw_behind_window).unwrap_or(false);
    let color_hex = shadow
        .map(|s| s.color.to_hex())
        .unwrap_or_else(|| "#00000070".to_string());
    let inactive_hex = shadow
        .map(|s| s.inactive_color.to_hex())
        .unwrap_or_else(|| "#00000050".to_string());

    let show_details = has_shadow && enabled;

    let settings = vec![
        make_toggle(
            "has_shadow",
            "Custom shadow",
            "Override shadow settings for this layer",
            has_shadow,
            true,
        ),
        make_toggle(
            "shadow_enabled",
            "Shadow enabled",
            "Draw shadow for this layer",
            enabled,
            has_shadow,
        ),
        make_slider_int(
            "shadow_softness",
            "Softness",
            "Shadow blur amount",
            softness,
            0.0,
            100.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_spread",
            "Spread",
            "Shadow expansion",
            spread,
            0.0,
            50.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_x",
            "Offset X",
            "Horizontal shadow offset",
            offset_x,
            -50.0,
            50.0,
            "",
            show_details,
        ),
        make_slider_int(
            "shadow_offset_y",
            "Offset Y",
            "Vertical shadow offset",
            offset_y,
            -50.0,
            50.0,
            "",
            show_details,
        ),
        make_toggle(
            "shadow_draw_behind",
            "Draw behind window",
            "Render shadow behind transparent windows",
            draw_behind,
            show_details,
        ),
        make_color(
            "shadow_color",
            "Shadow color",
            "Active layer shadow color",
            &color_hex,
            show_details,
        ),
        make_color(
            "shadow_inactive_color",
            "Inactive shadow color",
            "Inactive layer shadow color",
            &inactive_hex,
            show_details,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Populate advanced settings model
pub fn populate_advanced_settings(rule: &LayerRule) -> ModelRc<LayerRuleSettingModel> {
    let settings = vec![
        make_toggle(
            "place_within_backdrop",
            "Place within backdrop",
            "Position layer within the overview backdrop",
            rule.place_within_backdrop,
            true,
        ),
        make_toggle(
            "baba_is_float",
            "Baba is float",
            "Treat this layer as floating (special behavior)",
            rule.baba_is_float,
            true,
        ),
    ];
    ModelRc::new(VecModel::from(settings))
}

/// Sync all UI models for the selected rule
pub fn sync_rule_models(ui: &MainWindow, rule: &LayerRule, match_idx: i32) {
    let match_idx_usize = match_idx.max(0) as usize;

    ui.set_layer_rules_rule_settings(populate_rule_settings(rule));
    ui.set_layer_rules_match_settings(populate_match_settings(rule, match_idx_usize));
    ui.set_layer_rules_visual_settings(populate_visual_settings(rule));
    ui.set_layer_rules_shadow_settings(populate_shadow_settings(rule));
    ui.set_layer_rules_advanced_settings(populate_advanced_settings(rule));

    // Also update matches list and count
    ui.set_layer_rules_matches_list(build_matches_list_model(&rule.matches));
    ui.set_layer_rules_matches_count(rule.matches.len() as i32);
    ui.set_layer_rules_selected_match_index(match_idx);
}
