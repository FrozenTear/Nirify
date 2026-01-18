//! Output/display-related UI callbacks
//!
//! Handles output configuration including scale, mode, position, transform, and VRR.
//! Phase 8 adds: custom mode/modeline, per-output hot corners, per-output layout override.

use crate::config::{LayoutOverride, OutputConfig, OutputHotCorners, Settings, SettingsCategory};
use crate::ipc;
use crate::MainWindow;
use log::{debug, error, info, warn};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

use super::super::indices::{OUTPUT_SCALE_MAX, OUTPUT_SCALE_MIN};
use crate::types::{CenterFocusedColumn, Transform, VrrMode};

/// Cached output modes: output_name -> (display_modes, storage_modes, current_index)
type ModeCache = Rc<RefCell<HashMap<String, (Vec<String>, Vec<String>, i32)>>>;

/// Fetch and cache all output modes from niri IPC
fn build_mode_cache() -> ModeCache {
    let mut cache = HashMap::new();

    match ipc::get_full_outputs() {
        Ok(outputs) => {
            info!("Caching modes for {} outputs", outputs.len());
            for output in outputs {
                let (display_modes, storage_modes, current_idx) = extract_modes_from_output(&output);
                debug!("Cached {} modes for output {}", display_modes.len(), output.name);
                cache.insert(output.name.clone(), (display_modes, storage_modes, current_idx));
            }
        }
        Err(e) => {
            debug!("Could not fetch output modes from niri: {}", e);
        }
    }

    Rc::new(RefCell::new(cache))
}

/// Get modes from cache, or return empty if not cached
fn get_modes_from_cache(cache: &ModeCache, output_name: &str) -> (Vec<String>, Vec<String>, i32) {
    cache.borrow()
        .get(output_name)
        .cloned()
        .unwrap_or_else(|| (vec![], vec![], -1))
}

/// Build a semicolon-separated string of output names
fn build_output_names_string(outputs: &[OutputConfig]) -> String {
    outputs
        .iter()
        .map(|o| o.name.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

/// Build a model for the output list
fn build_output_list_model(outputs: &[OutputConfig]) -> ModelRc<SharedString> {
    let names: Vec<SharedString> = outputs
        .iter()
        .map(|o| SharedString::from(o.name.as_str()))
        .collect();
    ModelRc::new(VecModel::from(names))
}

/// Format an output mode for display (e.g., "1920x1080 @ 144Hz")
pub fn format_mode_for_display(mode: &ipc::OutputMode, is_preferred: bool) -> String {
    let refresh_hz = mode.refresh_rate as f64 / 1000.0;
    let preferred = if is_preferred { " (Recommended)" } else { "" };
    format!("{}x{} @ {:.0}Hz{}", mode.width, mode.height, refresh_hz, preferred)
}

/// Format an output mode for storage (e.g., "1920x1080@144.00")
pub fn format_mode_for_storage(mode: &ipc::OutputMode) -> String {
    let refresh_hz = mode.refresh_rate as f64 / 1000.0;
    format!("{}x{}@{:.2}", mode.width, mode.height, refresh_hz)
}

/// Extract modes from an output as (display_modes, storage_modes, current_index)
fn extract_modes_from_output(output: &ipc::FullOutputInfo) -> (Vec<String>, Vec<String>, i32) {
    let display_modes: Vec<String> = output.modes
        .iter()
        .map(|m| format_mode_for_display(m, m.is_preferred))
        .collect();

    let storage_modes: Vec<String> = output.modes
        .iter()
        .map(format_mode_for_storage)
        .collect();

    let current_idx = output.current_mode.map(|i| i as i32).unwrap_or(-1);

    (display_modes, storage_modes, current_idx)
}

/// Get available modes for an output from niri IPC
fn get_modes_for_output(output_name: &str) -> (Vec<String>, Vec<String>, i32) {
    match ipc::get_full_outputs() {
        Ok(outputs) => {
            if let Some(output) = outputs.iter().find(|o| o.name == output_name) {
                extract_modes_from_output(output)
            } else {
                (vec![], vec![], -1)
            }
        }
        Err(e) => {
            debug!("Could not get modes for output {}: {}", output_name, e);
            (vec![], vec![], -1)
        }
    }
}

/// Update UI with the selected output's properties
fn update_output_ui(ui: &MainWindow, output: &OutputConfig, index: i32, mode_cache: &ModeCache) {
    ui.set_selected_output_index(index);
    ui.set_current_output_name(output.name.as_str().into());
    ui.set_current_output_enabled(output.enabled);
    ui.set_current_output_scale(output.scale as f32);
    ui.set_current_output_mode(output.mode.as_str().into());
    ui.set_current_output_pos_x(output.position_x);
    ui.set_current_output_pos_y(output.position_y);
    ui.set_current_output_transform_index(output.transform.to_index());
    ui.set_current_output_vrr_index(output.vrr.to_index());

    // Load available modes from cache (or fallback to IPC if not cached)
    let (display_modes, storage_modes, current_idx) = {
        let cached = get_modes_from_cache(mode_cache, &output.name);
        if cached.0.is_empty() {
            // Fallback to IPC for newly connected outputs
            get_modes_for_output(&output.name)
        } else {
            cached
        }
    };
    let modes_model: Vec<SharedString> = display_modes.iter().map(|s| SharedString::from(s.as_str())).collect();
    ui.set_available_modes(ModelRc::new(VecModel::from(modes_model)));

    // Find the selected mode index by matching the current mode string
    let selected_idx = if !output.mode.is_empty() {
        storage_modes.iter().position(|m| m == &output.mode)
            .map(|i| i as i32)
            .unwrap_or(current_idx)
    } else {
        current_idx
    };
    ui.set_selected_mode_index(selected_idx);
    ui.set_use_custom_mode(false); // Reset custom mode toggle when switching outputs

    // Phase 8: Custom mode/modeline
    ui.set_current_output_mode_custom(output.mode_custom);
    ui.set_current_output_modeline(output.modeline.clone().unwrap_or_default().into());

    // Phase 8: Per-output hot corners
    if let Some(ref hc) = output.hot_corners {
        ui.set_current_output_hc_override_enabled(true);
        ui.set_current_output_hot_corners_enabled(hc.enabled.unwrap_or(true));
        ui.set_current_output_hc_top_left(hc.top_left);
        ui.set_current_output_hc_top_right(hc.top_right);
        ui.set_current_output_hc_bottom_left(hc.bottom_left);
        ui.set_current_output_hc_bottom_right(hc.bottom_right);
    } else {
        ui.set_current_output_hc_override_enabled(false);
        ui.set_current_output_hot_corners_enabled(false);
        ui.set_current_output_hc_top_left(false);
        ui.set_current_output_hc_top_right(false);
        ui.set_current_output_hc_bottom_left(false);
        ui.set_current_output_hc_bottom_right(false);
    }

    // Phase 8: Per-output layout override
    if let Some(ref lo) = output.layout_override {
        ui.set_current_output_layout_override_enabled(true);
        ui.set_current_output_layout_gaps_inner(lo.gaps_inner.unwrap_or(8.0));
        ui.set_current_output_layout_gaps_outer(lo.gaps_outer.unwrap_or(8.0));
        ui.set_current_output_layout_struts_left(lo.strut_left.unwrap_or(0.0));
        ui.set_current_output_layout_struts_right(lo.strut_right.unwrap_or(0.0));
        ui.set_current_output_layout_struts_top(lo.strut_top.unwrap_or(0.0));
        ui.set_current_output_layout_struts_bottom(lo.strut_bottom.unwrap_or(0.0));
        ui.set_current_output_layout_center_index(
            lo.center_focused_column.unwrap_or_default().to_index(),
        );
    } else {
        ui.set_current_output_layout_override_enabled(false);
        ui.set_current_output_layout_gaps_inner(8.0);
        ui.set_current_output_layout_gaps_outer(8.0);
        ui.set_current_output_layout_struts_left(0.0);
        ui.set_current_output_layout_struts_right(0.0);
        ui.set_current_output_layout_struts_top(0.0);
        ui.set_current_output_layout_struts_bottom(0.0);
        ui.set_current_output_layout_center_index(0);
    }
}

/// Set up output/display-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Clone once for all callbacks in this module
    let s = Arc::clone(&settings);
    let sm = Rc::clone(&save_manager);

    // Shared state for tracking selected output index
    // Rc<Cell> since Slint callbacks are single-threaded
    let selected_idx = Rc::new(Cell::new(0i32));

    // Pre-fetch and cache all output modes from niri IPC on startup
    let mode_cache = build_mode_cache();

    // Add output callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        let mode_cache = Rc::clone(&mode_cache);
        ui.on_output_add(move |name| {
            let name_str = name.to_string();
            if name_str.is_empty() {
                return;
            }
            match settings.lock() {
                Ok(mut s) => {
                    // Check if output already exists
                    if s.outputs.outputs.iter().any(|o| o.name == name_str) {
                        debug!("Output {} already exists", name_str);
                        return;
                    }

                    // Try to get full output info from niri
                    let output = match ipc::get_full_outputs() {
                        Ok(full_outputs) => {
                            // Find the matching output by name
                            if let Some(full_info) = full_outputs.iter().find(|o| o.name == name_str)
                            {
                                debug!(
                                    "Found full info for output {}: mode={}, scale={}, pos=({},{}), transform={:?}",
                                    name_str,
                                    full_info.current_mode_string(),
                                    full_info.scale(),
                                    full_info.position_x(),
                                    full_info.position_y(),
                                    full_info.transform()
                                );
                                crate::config::OutputConfig {
                                    name: name_str.clone(),
                                    enabled: true,
                                    scale: full_info.scale(),
                                    mode: full_info.current_mode_string(),
                                    position_x: full_info.position_x(),
                                    position_y: full_info.position_y(),
                                    transform: full_info.transform(),
                                    ..Default::default()
                                }
                            } else {
                                debug!(
                                    "Output {} not found in niri IPC, using defaults",
                                    name_str
                                );
                                crate::config::OutputConfig {
                                    name: name_str.clone(),
                                    ..Default::default()
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get full output info: {}, using defaults", e);
                            crate::config::OutputConfig {
                                name: name_str.clone(),
                                ..Default::default()
                            }
                        }
                    };

                    s.outputs.outputs.push(output);
                    let new_idx = (s.outputs.outputs.len() - 1) as i32;

                    // Select the newly added output
                    selected_idx.set(new_idx);

                    // Update UI
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_output_names(build_output_names_string(&s.outputs.outputs).into());
                        ui.set_output_list(build_output_list_model(&s.outputs.outputs));
                        if let Some(output) = s.outputs.outputs.last() {
                            update_output_ui(&ui, output, new_idx, &mode_cache);
                        }
                    }

                    debug!("Added output: {}", name_str);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Discover outputs callback
    {
        let settings = Arc::clone(&s);
        let ui_weak = ui.as_weak();
        ui.on_output_discover(move || {
            debug!("Discovering outputs from niri...");
            match ipc::get_output_names() {
                Ok(names) => {
                    if let Some(ui) = ui_weak.upgrade() {
                        // Filter out already configured outputs
                        let configured: Vec<String> = if let Ok(s) = settings.lock() {
                            s.outputs.outputs.iter().map(|o| o.name.clone()).collect()
                        } else {
                            vec![]
                        };

                        let discovered: Vec<SharedString> = names
                            .into_iter()
                            .filter(|n| !configured.contains(n))
                            .map(|n| SharedString::from(n.as_str()))
                            .collect();

                        debug!("Discovered {} new outputs", discovered.len());
                        ui.set_discovered_outputs(ModelRc::new(VecModel::from(discovered)));
                    }
                }
                Err(e) => {
                    warn!("Failed to discover outputs: {}", e);
                }
            }
        });
    }

    // Remove output callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&sm);
        let mode_cache = Rc::clone(&mode_cache);
        ui.on_output_remove(move |index| {
            // Guard against negative indices from UI
            if index < 0 {
                error!("Invalid output remove index: {}", index);
                return;
            }
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.outputs.outputs.len() {
                        let name = s.outputs.outputs[idx].name.clone();
                        s.outputs.outputs.remove(idx);

                        // Update selected index
                        let new_sel = if s.outputs.outputs.is_empty() {
                            -1
                        } else if idx >= s.outputs.outputs.len() {
                            (s.outputs.outputs.len() - 1) as i32
                        } else {
                            idx as i32
                        };

                        selected_idx.set(new_sel);

                        // Update UI
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_output_names(
                                build_output_names_string(&s.outputs.outputs).into(),
                            );
                            ui.set_output_list(build_output_list_model(&s.outputs.outputs));
                            if new_sel >= 0 {
                                if let Some(output) = s.outputs.outputs.get(new_sel as usize) {
                                    update_output_ui(&ui, output, new_sel, &mode_cache);
                                }
                            } else {
                                // No outputs left, clear selection
                                ui.set_selected_output_index(-1);
                                ui.set_current_output_name("".into());
                            }
                        }

                        debug!("Removed output at index {}: {}", idx, name);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select output callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let ui_weak = ui.as_weak();
        let mode_cache = Rc::clone(&mode_cache);
        ui.on_output_select(move |index| {
            selected_idx.set(index);

            // Update UI with selected output's properties
            if let Ok(s) = settings.lock() {
                if index >= 0 && (index as usize) < s.outputs.outputs.len() {
                    if let Some(ui) = ui_weak.upgrade() {
                        if let Some(output) = s.outputs.outputs.get(index as usize) {
                            update_output_ui(&ui, output, index, &mode_cache);
                        }
                    }
                }
            }

            debug!("Selected output at index {}", index);
        });
    }

    // Output enabled callback - acquire settings lock first, then check index
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_enabled_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    output.enabled = enabled;
                    debug!("Output {} enabled: {}", output.name, enabled);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Output scale callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_scale_changed(move |scale| {
            let clamped = (scale as f64).clamp(OUTPUT_SCALE_MIN, OUTPUT_SCALE_MAX);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        output.scale = clamped;
                        debug!("Output {} scale: {}", output.name, clamped);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Output mode callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_mode_changed(move |mode| {
            let mode_str: String = mode.into();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        output.mode = mode_str.clone();
                        debug!("Output {} mode: {}", output.name, mode_str);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Output position X callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_pos_x_changed(move |x| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    output.position_x = x;
                    debug!("Output {} position X: {}", output.name, x);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Output position Y callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_pos_y_changed(move |y| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    output.position_y = y;
                    debug!("Output {} position Y: {}", output.name, y);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Output transform callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_transform_changed(move |index| {
            let transform = Transform::from_index(index);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        output.transform = transform;
                        debug!("Output {} transform: {:?}", output.name, transform);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Output VRR callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_vrr_changed(move |index| {
            let vrr = VrrMode::from_index(index);
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        output.vrr = vrr;
                        debug!("Output {} VRR: {:?}", output.name, vrr);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Mode selected from dropdown callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        let ui_weak = ui.as_weak();
        let mode_cache = Rc::clone(&mode_cache);
        ui.on_output_mode_selected(move |mode_index| {
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        // Get the storage format mode string from cache
                        let (_, storage_modes, _) = get_modes_from_cache(&mode_cache, &output.name);
                        if mode_index >= 0 && (mode_index as usize) < storage_modes.len() {
                            let mode_str = &storage_modes[mode_index as usize];
                            output.mode = mode_str.clone();
                            debug!("Output {} mode selected: {}", output.name, mode_str);

                            // Update UI
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_current_output_mode(mode_str.as_str().into());
                            }

                            save_manager.mark_dirty(SettingsCategory::Outputs);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Use custom mode toggle callback
    {
        let ui_weak = ui.as_weak();
        ui.on_use_custom_mode_toggled(move |custom| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_use_custom_mode(custom);
                debug!("Use custom mode: {}", custom);
            }
        });
    }

    // Phase 8: Custom mode callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_mode_custom_toggled(move |custom| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    output.mode_custom = custom;
                    debug!("Output {} mode_custom: {}", output.name, custom);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Modeline callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_modeline_changed(move |modeline| {
            let modeline_str: String = modeline.into();
            match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        output.modeline = if modeline_str.is_empty() {
                            None
                        } else {
                            Some(modeline_str.clone())
                        };
                        debug!("Output {} modeline: {:?}", output.name, modeline_str);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Phase 8: Per-output hot corners override callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_hc_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    if enabled {
                        output.hot_corners = Some(OutputHotCorners::default());
                    } else {
                        output.hot_corners = None;
                    }
                    debug!("Output {} hot corners override: {}", output.name, enabled);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Per-output hot corners enabled callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_hc_enabled_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    if let Some(ref mut hc) = output.hot_corners {
                        hc.enabled = Some(enabled);
                        debug!("Output {} hot corners enabled: {}", output.name, enabled);
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Hot corner callbacks (top-left, top-right, bottom-left, bottom-right)
    macro_rules! register_hc_corner_callback {
        ($ui:expr, $callback:ident, $settings:expr, $selected_idx:expr, $save_manager:expr, $field:ident, $label:expr) => {{
            let settings = Arc::clone(&$settings);
            let selected_idx = Rc::clone(&$selected_idx);
            let save_manager = Rc::clone(&$save_manager);
            $ui.$callback(move |val| match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        if let Some(ref mut hc) = output.hot_corners {
                            hc.$field = val;
                            debug!("Output {} {}: {}", output.name, $label, val);
                            save_manager.mark_dirty(SettingsCategory::Outputs);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            });
        }};
    }

    register_hc_corner_callback!(
        ui,
        on_output_hc_top_left_toggled,
        s,
        selected_idx,
        sm,
        top_left,
        "hot corner top-left"
    );
    register_hc_corner_callback!(
        ui,
        on_output_hc_top_right_toggled,
        s,
        selected_idx,
        sm,
        top_right,
        "hot corner top-right"
    );
    register_hc_corner_callback!(
        ui,
        on_output_hc_bottom_left_toggled,
        s,
        selected_idx,
        sm,
        bottom_left,
        "hot corner bottom-left"
    );
    register_hc_corner_callback!(
        ui,
        on_output_hc_bottom_right_toggled,
        s,
        selected_idx,
        sm,
        bottom_right,
        "hot corner bottom-right"
    );

    // Phase 8: Per-output layout override callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_layout_override_toggled(move |enabled| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    if enabled {
                        output.layout_override = Some(LayoutOverride::default());
                    } else {
                        output.layout_override = None;
                    }
                    debug!("Output {} layout override: {}", output.name, enabled);
                    save_manager.mark_dirty(SettingsCategory::Outputs);
                    save_manager.request_save();
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    // Phase 8: Layout override callbacks (gaps, struts, center)
    macro_rules! register_layout_callback {
        ($ui:expr, $callback:ident, $settings:expr, $selected_idx:expr, $save_manager:expr, $field:ident, $label:expr) => {{
            let settings = Arc::clone(&$settings);
            let selected_idx = Rc::clone(&$selected_idx);
            let save_manager = Rc::clone(&$save_manager);
            $ui.$callback(move |val| match settings.lock() {
                Ok(mut s) => {
                    let idx = selected_idx.get();
                    if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                        return;
                    }
                    if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                        if let Some(ref mut lo) = output.layout_override {
                            lo.$field = Some(val as f32);
                            debug!("Output {} {}: {}", output.name, $label, val);
                            save_manager.mark_dirty(SettingsCategory::Outputs);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            });
        }};
    }

    register_layout_callback!(
        ui,
        on_output_layout_gaps_inner_changed,
        s,
        selected_idx,
        sm,
        gaps_inner,
        "gaps inner"
    );
    register_layout_callback!(
        ui,
        on_output_layout_gaps_outer_changed,
        s,
        selected_idx,
        sm,
        gaps_outer,
        "gaps outer"
    );
    register_layout_callback!(
        ui,
        on_output_layout_struts_left_changed,
        s,
        selected_idx,
        sm,
        strut_left,
        "strut left"
    );
    register_layout_callback!(
        ui,
        on_output_layout_struts_right_changed,
        s,
        selected_idx,
        sm,
        strut_right,
        "strut right"
    );
    register_layout_callback!(
        ui,
        on_output_layout_struts_top_changed,
        s,
        selected_idx,
        sm,
        strut_top,
        "strut top"
    );
    register_layout_callback!(
        ui,
        on_output_layout_struts_bottom_changed,
        s,
        selected_idx,
        sm,
        strut_bottom,
        "strut bottom"
    );

    // Phase 8: Layout center focused column callback
    {
        let settings = Arc::clone(&s);
        let selected_idx = Rc::clone(&selected_idx);
        let save_manager = Rc::clone(&sm);
        ui.on_output_layout_center_changed(move |index| match settings.lock() {
            Ok(mut s) => {
                let idx = selected_idx.get();
                if idx < 0 || idx as usize >= s.outputs.outputs.len() {
                    return;
                }
                if let Some(output) = s.outputs.outputs.get_mut(idx as usize) {
                    if let Some(ref mut lo) = output.layout_override {
                        lo.center_focused_column = Some(CenterFocusedColumn::from_index(index));
                        debug!(
                            "Output {} center focused column: {:?}",
                            output.name, lo.center_focused_column
                        );
                        save_manager.mark_dirty(SettingsCategory::Outputs);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
