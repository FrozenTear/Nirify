//! Named workspaces UI callbacks
//!
//! Handles workspace configuration including add, remove, select, and property changes.

use crate::config::models::{LayoutOverride, NamedWorkspace};
use crate::config::{Settings, SettingsCategory};
use crate::constants::MAX_WORKSPACES;
use crate::types::CenterFocusedColumn;
use crate::MainWindow;
use log::{debug, error, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

/// Build workspace list model for UI display
fn build_workspace_list_model(workspaces: &[NamedWorkspace]) -> ModelRc<SharedString> {
    let mut names = Vec::with_capacity(workspaces.len());
    for w in workspaces {
        names.push(SharedString::from(w.name.as_str()));
    }
    ModelRc::new(VecModel::from(names))
}

// Note: Uses CenterFocusedColumn::to_index() and CenterFocusedColumn::from_index()
// derived via #[derive(SlintIndex)] - indices: Never=0, OnOverflow=1, Always=2

/// Set up workspace callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Clone once for all callbacks in this module
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Shared state for tracking selected index (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Add workspace callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Check limit before adding
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

                    // Update UI with new workspace list
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_workspaces_list(build_workspace_list_model(
                            &s.workspaces.workspaces,
                        ));
                        ui.set_selected_workspace_index(new_idx);

                        // Sync current workspace properties
                        if let Some(ws) = s.workspaces.workspaces.get(new_idx as usize) {
                            sync_current_workspace(&ui, ws);
                        }
                    }

                    debug!("Added new workspace with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::Workspaces);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Remove workspace callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.workspaces.workspaces.len() {
                        let name = s.workspaces.workspaces[idx].name.clone();
                        s.workspaces.workspaces.remove(idx);

                        // Update selected index
                        let new_sel = if s.workspaces.workspaces.is_empty() {
                            -1
                        } else if idx >= s.workspaces.workspaces.len() {
                            (s.workspaces.workspaces.len() - 1) as i32
                        } else {
                            idx as i32
                        };

                        selected_idx.set(new_sel);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_workspaces_list(build_workspace_list_model(
                                &s.workspaces.workspaces,
                            ));
                            ui.set_selected_workspace_index(new_sel);

                            // Sync current workspace properties if there's a selection
                            if new_sel >= 0 {
                                if let Some(ws) = s.workspaces.workspaces.get(new_sel as usize) {
                                    sync_current_workspace(&ui, ws);
                                }
                            }
                        }

                        debug!("Removed workspace at index {}: {}", idx, name);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select workspace callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        ui.on_workspace_select(move |index| {
            selected_idx.set(index);

            // Sync current workspace properties to UI
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    if index >= 0 && (index as usize) < s.workspaces.workspaces.len() {
                        if let Some(ws) = s.workspaces.workspaces.get(index as usize) {
                            sync_current_workspace(&ui, ws);
                        }
                    }
                }
            }

            debug!("Selected workspace at index {}", index);
        });
    }

    // Workspace name changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_name_changed(move |name| {
            let name_str = name.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                        if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                            ws.name = name_str.clone();

                            // Update workspace list display
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_workspaces_list(build_workspace_list_model(
                                    &s.workspaces.workspaces,
                                ));
                            }

                            debug!("Workspace {} name changed to: {}", idx, name_str);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Open on output changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_open_on_output_changed(move |output| {
            let output_str = output.to_string();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                        if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                            ws.open_on_output = if output_str.is_empty() {
                                None
                            } else {
                                Some(output_str.clone())
                            };
                            debug!("Workspace {} open on output: {:?}", idx, ws.open_on_output);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Layout override toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_layout_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if enabled {
                            ws.layout_override = Some(LayoutOverride::default());
                        } else {
                            ws.layout_override = None;
                        }
                        debug!("Workspace {} layout override enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Gaps override toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_gaps_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            if enabled {
                                lo.gaps_inner = Some(16.0);
                                lo.gaps_outer = Some(8.0);
                            } else {
                                lo.gaps_inner = None;
                                lo.gaps_outer = None;
                            }
                        }
                        debug!("Workspace {} gaps override enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Gaps inner changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_gaps_inner_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.gaps_inner = Some(val);
                        }
                        debug!("Workspace {} gaps inner: {}", idx, val);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Gaps outer changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_gaps_outer_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.gaps_outer = Some(val);
                        }
                        debug!("Workspace {} gaps outer: {}", idx, val);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Struts override toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_struts_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            if enabled {
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
                        debug!("Workspace {} struts override enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Strut left changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_strut_left_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.strut_left = Some(val);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Strut right changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_strut_right_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.strut_right = Some(val);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Strut top changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_strut_top_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.strut_top = Some(val);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Strut bottom changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_strut_bottom_changed(move |val| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.strut_bottom = Some(val);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Center override toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_center_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            if enabled {
                                lo.center_focused_column = Some(CenterFocusedColumn::Never);
                                lo.always_center_single_column = Some(false);
                            } else {
                                lo.center_focused_column = None;
                                lo.always_center_single_column = None;
                            }
                        }
                        debug!("Workspace {} center override enabled: {}", idx, enabled);
                        save_manager.mark_dirty(SettingsCategory::Workspaces);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Center focused column changed callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_center_focused_column_changed(move |index| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.center_focused_column = Some(CenterFocusedColumn::from_index(index));
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Always center single column toggled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_workspace_always_center_single_column_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx >= 0 && (idx as usize) < s.workspaces.workspaces.len() {
                    if let Some(ws) = s.workspaces.workspaces.get_mut(idx as usize) {
                        if let Some(ref mut lo) = ws.layout_override {
                            lo.always_center_single_column = Some(enabled);
                            save_manager.mark_dirty(SettingsCategory::Workspaces);
                            save_manager.request_save();
                        }
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}

/// Sync UI properties with the currently selected workspace
fn sync_current_workspace(ui: &MainWindow, ws: &NamedWorkspace) {
    ui.set_current_workspace_name(ws.name.as_str().into());
    ui.set_current_workspace_open_on_output(ws.open_on_output.clone().unwrap_or_default().into());

    let has_override = ws.layout_override.is_some();
    ui.set_current_workspace_has_layout_override(has_override);

    if let Some(ref lo) = ws.layout_override {
        // Gaps
        let has_gaps = lo.gaps_inner.is_some() || lo.gaps_outer.is_some();
        ui.set_current_workspace_has_gaps_override(has_gaps);
        ui.set_current_workspace_gaps_inner(lo.gaps_inner.unwrap_or(16.0));
        ui.set_current_workspace_gaps_outer(lo.gaps_outer.unwrap_or(8.0));

        // Struts
        let has_struts = lo.strut_left.is_some()
            || lo.strut_right.is_some()
            || lo.strut_top.is_some()
            || lo.strut_bottom.is_some();
        ui.set_current_workspace_has_struts_override(has_struts);
        ui.set_current_workspace_strut_left(lo.strut_left.unwrap_or(0.0));
        ui.set_current_workspace_strut_right(lo.strut_right.unwrap_or(0.0));
        ui.set_current_workspace_strut_top(lo.strut_top.unwrap_or(0.0));
        ui.set_current_workspace_strut_bottom(lo.strut_bottom.unwrap_or(0.0));

        // Center
        let has_center =
            lo.center_focused_column.is_some() || lo.always_center_single_column.is_some();
        ui.set_current_workspace_has_center_override(has_center);
        ui.set_current_workspace_center_focused_column_index(
            lo.center_focused_column
                .unwrap_or(CenterFocusedColumn::Never)
                .to_index(),
        );
        ui.set_current_workspace_always_center_single_column(
            lo.always_center_single_column.unwrap_or(false),
        );
    } else {
        // Reset override state
        ui.set_current_workspace_has_gaps_override(false);
        ui.set_current_workspace_gaps_inner(16.0);
        ui.set_current_workspace_gaps_outer(8.0);

        ui.set_current_workspace_has_struts_override(false);
        ui.set_current_workspace_strut_left(0.0);
        ui.set_current_workspace_strut_right(0.0);
        ui.set_current_workspace_strut_top(0.0);
        ui.set_current_workspace_strut_bottom(0.0);

        ui.set_current_workspace_has_center_override(false);
        ui.set_current_workspace_center_focused_column_index(0);
        ui.set_current_workspace_always_center_single_column(false);
    }
}

/// Public function to build workspace list model for sync
pub fn build_workspaces_list_model(workspaces: &[NamedWorkspace]) -> ModelRc<SharedString> {
    build_workspace_list_model(workspaces)
}
