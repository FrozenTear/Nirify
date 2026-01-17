//! Dynamic Named Workspaces UI callbacks
//!
//! Handles workspace configuration using model-driven dynamic UI.
//! Replaces 16+ individual callbacks with generic ones.

use crate::config::models::{LayoutOverride, NamedWorkspace};
use crate::config::{Settings, SettingsCategory};
use crate::constants::MAX_WORKSPACES;
use crate::types::CenterFocusedColumn;
use crate::{MainWindow, WorkspaceSettingModel};
use log::{debug, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

// ============================================================================
// HELPER FUNCTIONS FOR CREATING SETTING MODELS
// ============================================================================

fn make_toggle(id: &str, label: &str, desc: &str, value: bool, visible: bool) -> WorkspaceSettingModel {
    WorkspaceSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 0,
        bool_value: value,
        visible,
        ..Default::default()
    }
}

fn make_slider_float(
    id: &str,
    label: &str,
    desc: &str,
    value: f32,
    min: f32,
    max: f32,
    suffix: &str,
    visible: bool,
) -> WorkspaceSettingModel {
    WorkspaceSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 1,
        float_value: value,
        min_value: min,
        max_value: max,
        suffix: suffix.into(),
        use_float: true,
        visible,
        ..Default::default()
    }
}

fn make_combo(
    id: &str,
    label: &str,
    desc: &str,
    index: i32,
    options: &[&str],
    visible: bool,
) -> WorkspaceSettingModel {
    let opts: Vec<SharedString> = options.iter().map(|s| (*s).into()).collect();
    WorkspaceSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 2,
        combo_index: index,
        combo_options: ModelRc::new(VecModel::from(opts)),
        visible,
        ..Default::default()
    }
}

fn make_text(
    id: &str,
    label: &str,
    desc: &str,
    value: &str,
    placeholder: &str,
    visible: bool,
) -> WorkspaceSettingModel {
    WorkspaceSettingModel {
        id: id.into(),
        label: label.into(),
        description: desc.into(),
        setting_type: 3,
        text_value: value.into(),
        placeholder: placeholder.into(),
        visible,
        ..Default::default()
    }
}

// ============================================================================
// MODEL POPULATION FUNCTIONS
// ============================================================================

/// Build workspace list model for UI display
fn build_workspace_list_model(workspaces: &[NamedWorkspace]) -> ModelRc<SharedString> {
    let names: Vec<SharedString> = workspaces.iter().map(|w| w.name.as_str().into()).collect();
    ModelRc::new(VecModel::from(names))
}

/// Populate identity settings (name, open on output)
fn populate_identity_settings(ws: &NamedWorkspace) -> ModelRc<WorkspaceSettingModel> {
    let model = vec![
        make_text(
            "name",
            "Workspace name",
            "Display name for this workspace",
            &ws.name,
            "Workspace name",
            true,
        ),
        make_text(
            "open_on_output",
            "Open on output",
            "Force this workspace to open on a specific monitor",
            ws.open_on_output.as_deref().unwrap_or(""),
            "e.g., eDP-1",
            true,
        ),
    ];
    ModelRc::new(VecModel::from(model))
}

/// Populate layout override settings
fn populate_layout_override_settings(ws: &NamedWorkspace) -> ModelRc<WorkspaceSettingModel> {
    let has_override = ws.layout_override.is_some();
    let model = vec![make_toggle(
        "layout_override",
        "Custom layout for this workspace",
        "Override global layout settings",
        has_override,
        true,
    )];
    ModelRc::new(VecModel::from(model))
}

/// Populate gaps settings
fn populate_gaps_settings(ws: &NamedWorkspace) -> ModelRc<WorkspaceSettingModel> {
    let lo = ws.layout_override.as_ref();
    let has_gaps = lo.map(|l| l.gaps_inner.is_some() || l.gaps_outer.is_some()).unwrap_or(false);
    let gaps_inner = lo.and_then(|l| l.gaps_inner).unwrap_or(16.0);
    let gaps_outer = lo.and_then(|l| l.gaps_outer).unwrap_or(8.0);

    let model = vec![
        make_toggle(
            "gaps_override",
            "Custom gaps",
            "Override window spacing",
            has_gaps,
            true,
        ),
        make_slider_float(
            "gaps_inner",
            "Inner gaps",
            "Space between windows",
            gaps_inner,
            0.0,
            64.0,
            "px",
            has_gaps,
        ),
        make_slider_float(
            "gaps_outer",
            "Outer gaps",
            "Space around workspace edges",
            gaps_outer,
            0.0,
            64.0,
            "px",
            has_gaps,
        ),
    ];
    ModelRc::new(VecModel::from(model))
}

/// Populate struts settings
fn populate_struts_settings(ws: &NamedWorkspace) -> ModelRc<WorkspaceSettingModel> {
    let lo = ws.layout_override.as_ref();
    let has_struts = lo.map(|l| {
        l.strut_left.is_some() || l.strut_right.is_some() ||
        l.strut_top.is_some() || l.strut_bottom.is_some()
    }).unwrap_or(false);
    let strut_left = lo.and_then(|l| l.strut_left).unwrap_or(0.0);
    let strut_right = lo.and_then(|l| l.strut_right).unwrap_or(0.0);
    let strut_top = lo.and_then(|l| l.strut_top).unwrap_or(0.0);
    let strut_bottom = lo.and_then(|l| l.strut_bottom).unwrap_or(0.0);

    let model = vec![
        make_toggle(
            "struts_override",
            "Custom struts",
            "Reserved screen edges",
            has_struts,
            true,
        ),
        make_slider_float(
            "strut_left",
            "Left strut",
            "Reserved space on left edge",
            strut_left,
            0.0,
            200.0,
            "px",
            has_struts,
        ),
        make_slider_float(
            "strut_right",
            "Right strut",
            "Reserved space on right edge",
            strut_right,
            0.0,
            200.0,
            "px",
            has_struts,
        ),
        make_slider_float(
            "strut_top",
            "Top strut",
            "Reserved space on top edge",
            strut_top,
            0.0,
            200.0,
            "px",
            has_struts,
        ),
        make_slider_float(
            "strut_bottom",
            "Bottom strut",
            "Reserved space on bottom edge",
            strut_bottom,
            0.0,
            200.0,
            "px",
            has_struts,
        ),
    ];
    ModelRc::new(VecModel::from(model))
}

/// Populate centering settings
fn populate_centering_settings(ws: &NamedWorkspace) -> ModelRc<WorkspaceSettingModel> {
    let lo = ws.layout_override.as_ref();
    let has_center = lo.map(|l| {
        l.center_focused_column.is_some() || l.always_center_single_column.is_some()
    }).unwrap_or(false);
    let center_index = lo
        .and_then(|l| l.center_focused_column)
        .unwrap_or(CenterFocusedColumn::Never)
        .to_index();
    let always_center = lo
        .and_then(|l| l.always_center_single_column)
        .unwrap_or(false);

    let model = vec![
        make_toggle(
            "center_override",
            "Custom centering behavior",
            "Override column centering",
            has_center,
            true,
        ),
        make_combo(
            "center_focused_column",
            "Center focused column",
            "When to center the active column",
            center_index,
            &["Never", "Always", "On Overflow"],
            has_center,
        ),
        make_toggle(
            "always_center_single_column",
            "Always center single column",
            "Center even when there's only one column",
            always_center,
            has_center,
        ),
    ];
    ModelRc::new(VecModel::from(model))
}

// ============================================================================
// SYNC ALL MODELS
// ============================================================================

/// Sync all workspace models to the UI
pub fn sync_workspaces_models(ui: &MainWindow, workspaces: &[NamedWorkspace], selected_idx: i32) {
    // Update workspace list
    ui.set_workspaces_list_dynamic(build_workspace_list_model(workspaces));
    ui.set_selected_workspace_index_dynamic(selected_idx);

    // If a workspace is selected, populate its settings
    if selected_idx >= 0 {
        if let Some(ws) = workspaces.get(selected_idx as usize) {
            populate_workspace_models(ui, ws);
        }
    }
}

/// Populate all models for a single workspace
fn populate_workspace_models(ui: &MainWindow, ws: &NamedWorkspace) {
    let has_layout_override = ws.layout_override.is_some();

    // Set visibility flags
    ui.set_workspace_has_layout_override_dynamic(has_layout_override);

    // Populate all section models
    ui.set_workspace_identity_settings_dynamic(populate_identity_settings(ws));
    ui.set_workspace_layout_override_settings_dynamic(populate_layout_override_settings(ws));
    ui.set_workspace_gaps_settings_dynamic(populate_gaps_settings(ws));
    ui.set_workspace_struts_settings_dynamic(populate_struts_settings(ws));
    ui.set_workspace_centering_settings_dynamic(populate_centering_settings(ws));
}

// ============================================================================
// CALLBACK SETUP
// ============================================================================

/// Set up dynamic workspace callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Initialize workspace list
    if let Ok(s) = settings.lock() {
        ui.set_workspaces_list_dynamic(build_workspace_list_model(&s.workspaces.workspaces));
    }

    // Add workspace callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_add_dynamic(move || {
            if let Ok(mut s) = settings.lock() {
                if s.workspaces.workspaces.len() >= MAX_WORKSPACES {
                    warn!("Maximum workspaces limit ({}) reached", MAX_WORKSPACES);
                    return;
                }

                let new_id = s.workspaces.next_id;
                s.workspaces.next_id += 1;

                let workspace = NamedWorkspace {
                    id: new_id,
                    name: format!("Workspace {}", new_id + 1),
                    open_on_output: None,
                    layout_override: None,
                };
                s.workspaces.workspaces.push(workspace);

                let new_idx = (s.workspaces.workspaces.len() - 1) as i32;
                selected_idx.set(new_idx);

                if let Some(ui) = ui_weak.upgrade() {
                    sync_workspaces_models(&ui, &s.workspaces.workspaces, new_idx);
                }

                debug!("Added new workspace with id {}", new_id);
                save_manager.mark_dirty(SettingsCategory::Workspaces);
                save_manager.request_save();
            }
        });
    }

    // Remove workspace callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_remove_dynamic(move |index| {
            let idx = index as usize;
            if let Ok(mut s) = settings.lock() {
                if idx < s.workspaces.workspaces.len() {
                    let name = s.workspaces.workspaces[idx].name.clone();
                    s.workspaces.workspaces.remove(idx);

                    let new_sel = if s.workspaces.workspaces.is_empty() {
                        -1
                    } else if idx >= s.workspaces.workspaces.len() {
                        (s.workspaces.workspaces.len() - 1) as i32
                    } else {
                        idx as i32
                    };

                    selected_idx.set(new_sel);

                    if let Some(ui) = ui_weak.upgrade() {
                        sync_workspaces_models(&ui, &s.workspaces.workspaces, new_sel);
                    }

                    debug!("Removed workspace at index {}: {}", idx, name);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }

    // Select workspace callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        ui.on_workspace_select_dynamic(move |index| {
            selected_idx.set(index);

            if let Ok(s) = settings.lock() {
                if let Some(ui) = ui_weak.upgrade() {
                    if let Some(ws) = s.workspaces.workspaces.get(index as usize) {
                        populate_workspace_models(&ui, ws);
                    }
                }
            }

            debug!("Selected workspace at index {}", index);
        });
    }

    // Reorder workspace callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_reorder_dynamic(move |from, to| {
            if let Ok(mut s) = settings.lock() {
                let from_idx = from as usize;
                let to_idx = to as usize;
                if from_idx < s.workspaces.workspaces.len() && to_idx <= s.workspaces.workspaces.len() {
                    let workspace = s.workspaces.workspaces.remove(from_idx);
                    let insert_idx = if to_idx > from_idx {
                        to_idx - 1
                    } else {
                        to_idx
                    };
                    s.workspaces.workspaces.insert(insert_idx, workspace);

                    selected_idx.set(insert_idx as i32);

                    if let Some(ui) = ui_weak.upgrade() {
                        sync_workspaces_models(&ui, &s.workspaces.workspaces, insert_idx as i32);
                    }

                    debug!("Reordered workspace from {} to {}", from_idx, insert_idx);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic toggle callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_setting_toggle_changed(move |id, value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                    let id_str = id.as_str();

                    match id_str {
                        "layout_override" => {
                            ws.layout_override = if value {
                                Some(LayoutOverride::default())
                            } else {
                                None
                            };
                            // Refresh all models as visibility changed
                            if let Some(ui) = ui_weak.upgrade() {
                                populate_workspace_models(&ui, ws);
                            }
                        }
                        "gaps_override" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                if value {
                                    lo.gaps_inner = Some(16.0);
                                    lo.gaps_outer = Some(8.0);
                                } else {
                                    lo.gaps_inner = None;
                                    lo.gaps_outer = None;
                                }
                            }
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_workspace_gaps_settings_dynamic(populate_gaps_settings(ws));
                            }
                        }
                        "struts_override" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                if value {
                                    lo.strut_left = Some(0.0);
                                    lo.strut_right = Some(0.0);
                                    lo.strut_top = Some(0.0);
                                    lo.strut_bottom = Some(0.0);
                                } else {
                                    lo.strut_left = None;
                                    lo.strut_right = None;
                                    lo.strut_top = None;
                                    lo.strut_bottom = None;
                                }
                            }
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_workspace_struts_settings_dynamic(populate_struts_settings(ws));
                            }
                        }
                        "center_override" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                if value {
                                    lo.center_focused_column = Some(CenterFocusedColumn::Never);
                                    lo.always_center_single_column = Some(false);
                                } else {
                                    lo.center_focused_column = None;
                                    lo.always_center_single_column = None;
                                }
                            }
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_workspace_centering_settings_dynamic(populate_centering_settings(ws));
                            }
                        }
                        "always_center_single_column" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.always_center_single_column = Some(value);
                            }
                        }
                        _ => {
                            debug!("Unknown workspace toggle setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Workspace toggle {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic slider float callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_setting_slider_float_changed(move |id, value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                    let id_str = id.as_str();

                    match id_str {
                        "gaps_inner" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.gaps_inner = Some(value);
                            }
                        }
                        "gaps_outer" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.gaps_outer = Some(value);
                            }
                        }
                        "strut_left" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.strut_left = Some(value);
                            }
                        }
                        "strut_right" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.strut_right = Some(value);
                            }
                        }
                        "strut_top" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.strut_top = Some(value);
                            }
                        }
                        "strut_bottom" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.strut_bottom = Some(value);
                            }
                        }
                        _ => {
                            debug!("Unknown workspace slider float setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Workspace slider float {} = {}", id_str, value);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic combo callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_setting_combo_changed(move |id, index| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                    let id_str = id.as_str();

                    match id_str {
                        "center_focused_column" => {
                            if let Some(ref mut lo) = ws.layout_override {
                                lo.center_focused_column = Some(CenterFocusedColumn::from_index(index));
                            }
                        }
                        _ => {
                            debug!("Unknown workspace combo setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Workspace combo {} = {}", id_str, index);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }

    // Generic text callback
    {
        let settings = Arc::clone(&settings);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_workspace_setting_text_changed(move |id, value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            if let Ok(mut s) = settings.lock() {
                if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                    let id_str = id.as_str();
                    let value_str = value.to_string();

                    match id_str {
                        "name" => {
                            ws.name = value_str.clone();
                            // Update workspace list display
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_workspaces_list_dynamic(build_workspace_list_model(
                                    &s.workspaces.workspaces,
                                ));
                            }
                        }
                        "open_on_output" => {
                            ws.open_on_output = if value_str.is_empty() {
                                None
                            } else {
                                Some(value_str.clone())
                            };
                        }
                        _ => {
                            debug!("Unknown workspace text setting: {}", id_str);
                            return;
                        }
                    }

                    debug!("Workspace text {} = {}", id_str, value_str);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
            }
        });
    }
}
