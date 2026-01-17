//! First-run wizard and config setup logic
//!
//! Handles the initial setup experience for new users, including:
//! - Detecting first-run state
//! - Import summary display
//! - Adding include line to config.kdl
//! - Smart replacement of managed sections

use anyhow::{Context, Result};
use log::{error, info, warn};
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

use crate::config::{
    self, analyze_rules, ConfigPaths, ConsolidationAnalysis, Settings, SmartReplaceResult,
};
use crate::MainWindow;

/// Configure UI state for first run or existing installation
pub fn setup_first_run(ui: &MainWindow, paths: &ConfigPaths, is_first_run: bool) {
    ui.set_config_path(paths.niri_config.to_string_lossy().to_string().into());

    if is_first_run {
        info!("First run detected - showing setup wizard");
        ui.set_wizard_visible(true);
        ui.set_has_include_line(false);
    } else {
        let has_include = paths.has_include_line();
        ui.set_has_include_line(has_include);
        if !has_include {
            info!("Include line not found in config.kdl");
        }
    }
}

/// Set up wizard completion and cancellation callbacks
pub fn setup_wizard_callbacks(ui: &MainWindow, settings: Arc<Mutex<Settings>>) {
    use slint::Model;

    let ui_weak = ui.as_weak();
    let settings_for_complete = settings.clone();
    ui.on_wizard_completed(move || {
        if let Some(ui) = ui_weak.upgrade() {
            info!("Wizard completed");
            ui.set_wizard_visible(false);

            // Show import details dialog if there are warnings
            let warnings = ui.get_import_warnings();
            if warnings.row_count() > 0 {
                info!(
                    "Showing import details dialog ({} warnings)",
                    warnings.row_count()
                );
                ui.set_show_import_details(true);
            }

            // Check for consolidation suggestions after wizard completes
            if let Ok(s) = settings_for_complete.lock() {
                let analysis = analyze_rules(&s.window_rules.rules, &s.layer_rules.rules);
                if analysis.has_suggestions() {
                    info!(
                        "Showing consolidation dialog with {} suggestions",
                        analysis.total_suggestions()
                    );
                    ui.set_show_consolidation_dialog(true);
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_wizard_cancelled(move || {
        if let Some(ui) = ui_weak.upgrade() {
            info!("Wizard cancelled/skipped");
            ui.set_wizard_visible(false);

            // Also show consolidation suggestions even if wizard was skipped
            if let Ok(s) = settings.lock() {
                let analysis = analyze_rules(&s.window_rules.rules, &s.layer_rules.rules);
                if analysis.has_suggestions() {
                    info!(
                        "Showing consolidation dialog with {} suggestions (after skip)",
                        analysis.total_suggestions()
                    );
                    ui.set_show_consolidation_dialog(true);
                }
            }
        }
    });
}

/// Set up callback for adding include line to config.kdl with smart replacement
pub fn setup_include_line_handler(
    ui: &MainWindow,
    paths: Arc<ConfigPaths>,
    settings: Arc<Mutex<Settings>>,
    show_error_fn: impl Fn(&MainWindow, &str, &str, &str) + 'static,
    show_status_fn: impl Fn(&MainWindow, &str, bool) + 'static,
) {
    let ui_weak = ui.as_weak();
    ui.on_add_include_line_requested(move || {
        if let Some(ui) = ui_weak.upgrade() {
            match setup_config(&paths) {
                Ok(result) => {
                    info!(
                        "Config setup complete: {} replaced, {} preserved",
                        result.replaced_count, result.preserved_count
                    );

                    // Log backup path if one was created
                    if !result.backup_path.as_os_str().is_empty() {
                        info!("Backup created at {:?}", result.backup_path);
                    }

                    // Log any warnings
                    for warning in &result.warnings {
                        warn!("Config setup warning: {}", warning);
                    }

                    // IMPORTANT: Save settings immediately so the included files exist
                    match settings.lock() {
                        Ok(s) => {
                            if let Err(e) = config::save_settings(&paths, &s) {
                                error!("Failed to save settings after setup: {}", e);
                                show_error_fn(
                                    &ui,
                                    "Failed to Save Settings",
                                    "Config.kdl was updated but settings files could not be created.",
                                    &e.to_string(),
                                );
                                return;
                            }
                            info!("Settings files created successfully");
                        }
                        Err(e) => {
                            error!("Settings lock error: {}", e);
                            return;
                        }
                    }

                    ui.set_has_include_line(true);
                    ui.set_wizard_step(2); // Move to completion step

                    // Show descriptive status message
                    let status_msg = if result.replaced_count > 0 && result.preserved_count > 0 {
                        format!(
                            "Config updated: {} sections managed, {} custom settings preserved",
                            result.replaced_count, result.preserved_count
                        )
                    } else if result.replaced_count > 0 {
                        format!(
                            "Config updated: {} sections now managed by niri-settings",
                            result.replaced_count
                        )
                    } else if result.include_added {
                        "Config setup complete".to_string()
                    } else {
                        "Config already set up".to_string()
                    };
                    show_status_fn(&ui, &status_msg, false);
                }
                Err(e) => {
                    error!("Failed to set up config: {}", e);
                    show_error_fn(
                        &ui,
                        "Failed to Set Up Config",
                        "Could not modify your config.kdl file.",
                        &e.to_string(),
                    );
                }
            }
        }
    });
}

/// Set up config.kdl with smart replacement
///
/// This function intelligently replaces managed sections of the user's config.kdl
/// while preserving any custom (unmanaged) settings. It:
/// 1. Analyzes the config and classifies each top-level node
/// 2. Creates a timestamped backup
/// 3. Generates a clean config with our include line + preserved custom content
///
/// Returns a result describing what was done (replaced/preserved counts, backup path).
pub fn setup_config(paths: &ConfigPaths) -> Result<SmartReplaceResult> {
    use std::fs;

    // Ensure parent directory exists
    if let Some(parent) = paths.niri_config.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    // Use ~/.config/niri/.backup/ so backups survive if user deletes niri-settings folder
    let backup_dir = paths
        .niri_config
        .parent()
        .map(|p| p.join(".backup"))
        .unwrap_or_else(|| paths.backup_dir.clone());

    config::smart_replace_config(&paths.niri_config, &backup_dir)
}

/// Set import summary data on UI for first-run wizard display
pub fn set_import_summary(ui: &MainWindow, import_result: &config::ImportResult) {
    ui.set_import_summary(import_result.summary().into());
    ui.set_has_imports(import_result.has_imports());
    ui.set_import_count(import_result.imported_sections.len() as i32);
    ui.set_import_defaulted_count(import_result.defaulted_sections.len() as i32);
    ui.set_import_includes_count(import_result.includes_processed as i32);

    // Convert warnings to Slint model
    let warnings_model: Vec<slint::SharedString> = import_result
        .warnings
        .iter()
        .map(|w| slint::SharedString::from(w.as_str()))
        .collect();
    let warnings_rc = std::rc::Rc::new(slint::VecModel::from(warnings_model));
    ui.set_import_warnings(slint::ModelRc::from(warnings_rc));

    // Auto-show import details dialog if there are warnings
    if !import_result.warnings.is_empty() {
        info!(
            "Import completed with {} warnings, will show details dialog",
            import_result.warnings.len()
        );
    }
}

/// Analyze rules for consolidation opportunities and set UI state
pub fn set_consolidation_suggestions(ui: &MainWindow, settings: &Settings) {
    let analysis = analyze_rules(&settings.window_rules.rules, &settings.layer_rules.rules);

    if !analysis.has_suggestions() {
        info!("No consolidation opportunities found");
        return;
    }

    info!(
        "Found {} consolidation suggestions affecting {} rules",
        analysis.total_suggestions(),
        analysis.total_affected_rules()
    );

    // Convert to Slint model
    let items: Vec<ConsolidationItem> = analysis
        .window_suggestions
        .iter()
        .chain(analysis.layer_suggestions.iter())
        .enumerate()
        .map(|(idx, s)| ConsolidationItem {
            index: idx as i32,
            description: s.description.clone().into(),
            rule_count: s.rule_ids.len() as i32,
            patterns: s.patterns.join(", ").into(),
            merged_pattern: s.merged_pattern.clone().into(),
            shared_settings: s.shared_settings.clone().into(),
            selected: false,
        })
        .collect();

    let items_rc = std::rc::Rc::new(slint::VecModel::from(items));
    ui.set_consolidation_suggestions(slint::ModelRc::from(items_rc));
    ui.set_consolidation_affected_rules(analysis.total_affected_rules() as i32);
}

/// Set up consolidation dialog callbacks
pub fn setup_consolidation_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    paths: Arc<ConfigPaths>,
) {
    // Track selected suggestions internally (Slint doesn't make this easy)
    let selected_indices = std::rc::Rc::new(std::cell::RefCell::new(Vec::<i32>::new()));

    // Dismiss callback - just hide the dialog
    let ui_weak = ui.as_weak();
    ui.on_consolidation_dismissed(move || {
        if let Some(ui) = ui_weak.upgrade() {
            info!("Consolidation dialog dismissed");
            ui.set_show_consolidation_dialog(false);
        }
    });

    // Toggle a suggestion's selected state
    let ui_weak = ui.as_weak();
    let selected_for_toggle = selected_indices.clone();
    ui.on_consolidation_suggestion_toggled(move |index, selected| {
        use slint::Model;

        if let Some(ui) = ui_weak.upgrade() {
            let mut sel = selected_for_toggle.borrow_mut();
            if selected {
                if !sel.contains(&index) {
                    sel.push(index);
                }
            } else {
                sel.retain(|&i| i != index);
            }

            // Update the UI model to reflect selection
            let suggestions = ui.get_consolidation_suggestions();
            if let Some(model) = suggestions
                .as_any()
                .downcast_ref::<slint::VecModel<ConsolidationItem>>()
            {
                if let Some(mut item) = model.row_data(index as usize) {
                    item.selected = selected;
                    model.set_row_data(index as usize, item);
                }
            }
        }
    });

    // Apply selected consolidations
    let ui_weak = ui.as_weak();
    let selected_for_apply = selected_indices;
    ui.on_consolidation_apply_selected(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let sel = selected_for_apply.borrow();
            if sel.is_empty() {
                info!("No consolidation suggestions selected");
                ui.set_show_consolidation_dialog(false);
                return;
            }

            info!("Applying {} consolidation suggestions", sel.len());

            // Get the current analysis and settings
            match settings.lock() {
                Ok(mut s) => {
                    let analysis = analyze_rules(&s.window_rules.rules, &s.layer_rules.rules);

                    // Apply each selected suggestion
                    for &idx in sel.iter() {
                        apply_consolidation_suggestion(&mut s, &analysis, idx as usize);
                    }

                    // Save the updated settings
                    if let Err(e) = config::save_settings(&paths, &s) {
                        error!("Failed to save consolidated settings: {}", e);
                    } else {
                        info!("Consolidated settings saved successfully");
                    }
                }
                Err(e) => {
                    error!("Failed to lock settings for consolidation: {}", e);
                }
            }

            ui.set_show_consolidation_dialog(false);
        }
    });
}

/// Apply a single consolidation suggestion to the settings
fn apply_consolidation_suggestion(
    settings: &mut Settings,
    analysis: &ConsolidationAnalysis,
    suggestion_index: usize,
) {
    // Determine if it's a window or layer rule suggestion
    let window_count = analysis.window_suggestions.len();

    if suggestion_index < window_count {
        // Window rule consolidation
        let suggestion = &analysis.window_suggestions[suggestion_index];
        apply_window_rule_consolidation(settings, suggestion);
    } else {
        // Layer rule consolidation
        let layer_index = suggestion_index - window_count;
        if layer_index < analysis.layer_suggestions.len() {
            let suggestion = &analysis.layer_suggestions[layer_index];
            apply_layer_rule_consolidation(settings, suggestion);
        }
    }
}

/// Apply window rule consolidation - merge multiple rules into one
fn apply_window_rule_consolidation(
    settings: &mut Settings,
    suggestion: &config::ConsolidationSuggestion,
) {
    use crate::config::models::WindowRuleMatch;

    // Find the first rule to keep (will be modified to use merged pattern)
    let first_id = suggestion.rule_ids.first().copied();
    let Some(first_id) = first_id else { return };

    // Find the first rule and update its match pattern
    if let Some(rule) = settings
        .window_rules
        .rules
        .iter_mut()
        .find(|r| r.id == first_id)
    {
        // Update the match to use the merged regex pattern
        if !rule.matches.is_empty() {
            rule.matches[0].app_id = Some(suggestion.merged_pattern.clone());
        } else {
            rule.matches.push(WindowRuleMatch {
                app_id: Some(suggestion.merged_pattern.clone()),
                ..Default::default()
            });
        }

        // Update the name to reflect consolidation
        rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
    }

    // Remove all other rules that were consolidated
    let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
    settings
        .window_rules
        .rules
        .retain(|r| !other_ids.contains(&r.id));

    info!(
        "Consolidated {} window rules into one with pattern: {}",
        suggestion.rule_ids.len(),
        suggestion.merged_pattern
    );
}

/// Apply layer rule consolidation - merge multiple rules into one
fn apply_layer_rule_consolidation(
    settings: &mut Settings,
    suggestion: &config::ConsolidationSuggestion,
) {
    use crate::config::models::LayerRuleMatch;

    // Find the first rule to keep
    let first_id = suggestion.rule_ids.first().copied();
    let Some(first_id) = first_id else { return };

    // Find the first rule and update its match pattern
    if let Some(rule) = settings
        .layer_rules
        .rules
        .iter_mut()
        .find(|r| r.id == first_id)
    {
        // Update the match to use the merged regex pattern
        if !rule.matches.is_empty() {
            rule.matches[0].namespace = Some(suggestion.merged_pattern.clone());
        } else {
            rule.matches.push(LayerRuleMatch {
                namespace: Some(suggestion.merged_pattern.clone()),
                ..Default::default()
            });
        }

        // Update the name to reflect consolidation
        rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
    }

    // Remove all other rules that were consolidated
    let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
    settings
        .layer_rules
        .rules
        .retain(|r| !other_ids.contains(&r.id));

    info!(
        "Consolidated {} layer rules into one with pattern: {}",
        suggestion.rule_ids.len(),
        suggestion.merged_pattern
    );
}

// Re-export ConsolidationItem for use in callbacks
use crate::ConsolidationItem;
