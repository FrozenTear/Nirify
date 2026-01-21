//! Keybindings-related UI callbacks
//!
//! Handles keybinding viewing, editing, and management.

use crate::config::loader::load_keybindings;
use crate::config::models::{KeybindAction, Keybinding};
use crate::config::paths::ConfigPaths;
use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use log::{debug, error, info, warn};
use slint::ComponentHandle;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::{key_parts_to_model, parse_key_combo_parts};
use super::super::key_mapping::{build_key_combo, map_char_to_niri};
use super::super::macros::SaveManager;
use super::super::sync::sync_keybindings;

/// Set up keybindings-related callbacks
pub fn setup(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    paths: Arc<ConfigPaths>,
    save_manager: Rc<SaveManager>,
) {
    // Track selected keybinding index (Rc<Cell> since Slint callbacks are single-threaded)
    let selected_idx = Rc::new(Cell::new(-1i32));

    // Refresh keybindings (reload from managed keybindings.kdl)
    {
        let ui_handle = ui.as_weak();
        let settings = settings.clone();
        let paths = paths.clone();

        ui.on_keybindings_refresh(move || {
            debug!("Refreshing keybindings from managed file");

            match settings.lock() {
                Ok(mut s) => {
                    // Reload keybindings from our managed keybindings.kdl
                    load_keybindings(&paths.keybindings_kdl, &mut s.keybindings);

                    // Update UI
                    if let Some(ui) = ui_handle.upgrade() {
                        sync_keybindings(&ui, &s);
                        debug!(
                            "Keybindings refreshed: {} bindings loaded",
                            s.keybindings.bindings.len()
                        );
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Select keybinding callback
    {
        let selected_idx = selected_idx.clone();
        let settings = settings.clone();
        let ui_weak = ui.as_weak();

        ui.on_keybinding_selected(move |idx| {
            selected_idx.set(idx);

            // Sync selected keybinding details to editor
            if let Ok(s) = settings.lock() {
                if idx >= 0 && (idx as usize) < s.keybindings.bindings.len() {
                    if let Some(ui) = ui_weak.upgrade() {
                        if let Some(binding) = s.keybindings.bindings.get(idx as usize) {
                            sync_keybinding_to_editor(&ui, binding);
                        }
                    }
                }
            }
        });
    }

    // Add new keybinding
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Generate new ID
                    let new_id = s
                        .keybindings
                        .bindings
                        .iter()
                        .map(|b| b.id)
                        .max()
                        .unwrap_or(0)
                        + 1;

                    let binding = Keybinding {
                        id: new_id,
                        key_combo: "Mod+".to_string(),
                        hotkey_overlay_title: None,
                        allow_when_locked: false,
                        cooldown_ms: None,
                        repeat: false,
                        action: KeybindAction::NiriAction("close-window".to_string()),
                    };

                    s.keybindings.bindings.push(binding);
                    let new_idx = (s.keybindings.bindings.len() - 1) as i32;

                    selected_idx.set(new_idx);

                    if let Some(ui) = ui_weak.upgrade() {
                        sync_keybindings(&ui, &s);
                        ui.set_selected_keybinding_index(new_idx);
                        ui.set_keybinding_editor_visible(true);

                        if let Some(binding) = s.keybindings.bindings.get(new_idx as usize) {
                            sync_keybinding_to_editor(&ui, binding);
                        }
                    }

                    info!("Added new keybinding with id {}", new_id);
                    save_manager.mark_dirty(SettingsCategory::Keybindings);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Delete keybinding
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_delete(move |idx| {
            match settings.lock() {
                Ok(mut s) => {
                    let idx = idx as usize;
                    if idx < s.keybindings.bindings.len() {
                        let removed = s.keybindings.bindings.remove(idx);
                        info!("Deleted keybinding: {}", removed.key_combo);

                        // Update selected index
                        let new_idx = if s.keybindings.bindings.is_empty() {
                            -1
                        } else if idx >= s.keybindings.bindings.len() {
                            (s.keybindings.bindings.len() - 1) as i32
                        } else {
                            idx as i32
                        };

                        selected_idx.set(new_idx);

                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(new_idx);
                            ui.set_keybinding_editor_visible(false);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Update keybinding key combo
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_key_combo_changed(move |new_combo| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            let mut combo_str = new_combo.to_string();

            // Validate string length to prevent memory issues
            if combo_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Key combo exceeds maximum length, truncating");
                combo_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        binding.key_combo = combo_str;
                        debug!("Updated key combo to: {}", binding.key_combo);

                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Handle raw key press from capture widget
    {
        let ui_weak = ui.as_weak();

        ui.on_keybinding_raw_key_pressed(move |text, meta, ctrl, alt, shift| {
            // Skip if text is empty or just modifiers
            let text_str = text.to_string();
            if text_str.is_empty() {
                return;
            }

            // Map the key text to niri format
            let key_name = if text_str.len() == 1 {
                // Single character - use if-let to safely handle edge cases
                if let Some(c) = text_str.chars().next() {
                    map_char_to_niri(c)
                } else {
                    // Shouldn't happen since len() == 1, but handle gracefully
                    None
                }
            } else {
                // Multi-character text (shouldn't happen for single key presses)
                None
            };

            let Some(key_name) = key_name else {
                debug!("Ignoring unmappable key: {:?}", text_str);
                return;
            };

            // Check if we have at least one modifier for regular keys
            let has_modifier = meta || ctrl || alt || shift;
            if !has_modifier && key_name.len() <= 2 {
                debug!("Ignoring key without modifier: {}", key_name);
                return;
            }

            // Build the combo string
            let combo = build_key_combo(&key_name, meta, ctrl, alt, shift);
            debug!("Captured key combo: {}", combo);

            // Update UI
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_edit_key_combo(combo.clone().into());

                // Parse and set key parts
                let parts = parse_key_combo_parts(&combo);
                ui.set_edit_key_parts(key_parts_to_model(parts));

                // Stop capture mode
                ui.set_keybinding_capturing(false);
            }
        });
    }

    // Update keybinding action type
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_action_type_changed(move |action_type| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        // Convert action type index to KeybindAction
                        binding.action = match action_type {
                            0 => KeybindAction::Spawn(vec!["".to_string()]),
                            1 => KeybindAction::NiriAction("close-window".to_string()),
                            2 => KeybindAction::NiriActionWithArgs(
                                "focus-workspace".to_string(),
                                vec![],
                            ),
                            _ => binding.action.clone(),
                        };

                        debug!("Updated action type to: {:?}", binding.action);

                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybinding_to_editor(&ui, binding);
                            // Refresh list to show updated display name
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Update keybinding action command (for spawn)
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_action_command_changed(move |command| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            let mut command_str = command.to_string();

            // Validate string length to prevent memory issues
            if command_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Action command exceeds maximum length, truncating");
                command_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        // Parse command string into args
                        let args: Vec<String> = shell_words::split(&command_str)
                            .unwrap_or_else(|_| vec![command_str.clone()]);

                        binding.action = KeybindAction::Spawn(args);
                        debug!("Updated spawn command: {:?}", binding.action);

                        // Refresh list to show updated action detail
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Update keybinding action name (for niri actions)
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_action_name_changed(move |action_name| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            let mut action_str = action_name.to_string();

            // Validate string length to prevent memory issues
            if action_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Action name exceeds maximum length, truncating");
                action_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        match &binding.action {
                            KeybindAction::NiriAction(_) => {
                                binding.action = KeybindAction::NiriAction(action_str);
                            }
                            KeybindAction::NiriActionWithArgs(_, args) => {
                                binding.action = KeybindAction::NiriActionWithArgs(
                                    action_str,
                                    args.clone(),
                                );
                            }
                            _ => {}
                        }
                        debug!("Updated action name: {:?}", binding.action);

                        // Refresh list to show updated display name and action detail
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Update keybinding properties
    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_allow_when_locked_changed(move |value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        binding.allow_when_locked = value;

                        // Refresh list to show updated badge
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_repeat_changed(move |value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        binding.repeat = value;

                        // Refresh list to show updated badge
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_cooldown_changed(move |value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        binding.cooldown_ms = if value > 0 { Some(value) } else { None };

                        // Refresh list to show updated cooldown info
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybinding_overlay_title_changed(move |value| {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            let mut value_str = value.to_string();

            // Validate string length to prevent memory issues
            if value_str.len() > crate::constants::MAX_STRING_LENGTH {
                warn!("Overlay title exceeds maximum length, truncating");
                value_str.truncate(crate::constants::MAX_STRING_LENGTH);
            }

            match settings.lock() {
                Ok(mut s) => {
                    if let Some(binding) = s.keybindings.bindings.get_mut(idx as usize) {
                        binding.hotkey_overlay_title = if value_str.is_empty() {
                            None
                        } else {
                            Some(value_str)
                        };

                        // Refresh list to show updated display name
                        if let Some(ui) = ui_weak.upgrade() {
                            sync_keybindings(&ui, &s);
                            ui.set_selected_keybinding_index(idx);
                        }

                        save_manager.mark_dirty(SettingsCategory::Keybindings);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Import keybindings from user's config
    {
        let settings = settings.clone();
        let paths = paths.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybindings_import(move || {
            info!("Importing keybindings from user's config");

            match settings.lock() {
                Ok(mut s) => {
                    // Load keybindings from user's config
                    let mut temp_settings = crate::config::models::KeybindingsSettings::default();
                    load_keybindings(&paths.niri_config, &mut temp_settings);

                    if temp_settings.bindings.is_empty() {
                        warn!("No keybindings found in user's config to import");
                        return;
                    }

                    // Assign new IDs to avoid conflicts
                    let max_id = s
                        .keybindings
                        .bindings
                        .iter()
                        .map(|b| b.id)
                        .max()
                        .unwrap_or(0);

                    for (i, mut binding) in temp_settings.bindings.into_iter().enumerate() {
                        binding.id = max_id + 1 + i as u32;
                        s.keybindings.bindings.push(binding);
                    }

                    s.keybindings.loaded = true;
                    s.keybindings.error = None;

                    if let Some(ui) = ui_weak.upgrade() {
                        sync_keybindings(&ui, &s);
                    }

                    info!(
                        "Imported {} keybindings",
                        s.keybindings.bindings.len() - max_id as usize
                    );
                    save_manager.mark_dirty(SettingsCategory::Keybindings);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Import keybindings from a user-selected file
    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);

        ui.on_keybindings_import_from_file(move || {
            info!("Opening file picker for keybindings import");

            // Open file picker dialog
            let file = rfd::FileDialog::new()
                .add_filter("KDL files", &["kdl"])
                .add_filter("All files", &["*"])
                .set_title("Import Keybindings")
                .pick_file();

            let Some(file_path) = file else {
                debug!("File picker cancelled");
                return;
            };

            info!("Importing keybindings from {:?}", file_path);

            match settings.lock() {
                Ok(mut s) => {
                    // Load keybindings from the selected file
                    let mut temp_settings = crate::config::models::KeybindingsSettings::default();
                    load_keybindings(&file_path, &mut temp_settings);

                    if temp_settings.bindings.is_empty() {
                        warn!("No keybindings found in selected file");
                        return;
                    }

                    let imported_count = temp_settings.bindings.len();

                    // Assign new IDs to avoid conflicts
                    let max_id = s
                        .keybindings
                        .bindings
                        .iter()
                        .map(|b| b.id)
                        .max()
                        .unwrap_or(0);

                    for (i, mut binding) in temp_settings.bindings.into_iter().enumerate() {
                        binding.id = max_id + 1 + i as u32;
                        s.keybindings.bindings.push(binding);
                    }

                    s.keybindings.loaded = true;
                    s.keybindings.error = None;

                    if let Some(ui) = ui_weak.upgrade() {
                        sync_keybindings(&ui, &s);
                    }

                    info!("Imported {} keybindings from file", imported_count);
                    save_manager.mark_dirty(SettingsCategory::Keybindings);
                    save_manager.request_save();
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    // Close editor
    {
        let ui_weak = ui.as_weak();

        ui.on_keybinding_editor_close(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_keybinding_editor_visible(false);
            }
        });
    }

    // Edit selected keybinding
    {
        let ui_weak = ui.as_weak();
        let settings = settings.clone();
        let selected_idx = selected_idx.clone();

        ui.on_keybinding_edit(move || {
            let idx = selected_idx.get();
            if idx < 0 {
                return;
            }

            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(s) = settings.lock() {
                    if let Some(binding) = s.keybindings.bindings.get(idx as usize) {
                        sync_keybinding_to_editor(&ui, binding);
                        ui.set_keybinding_editor_visible(true);
                    }
                }
            }
        });
    }
}

/// Sync keybinding data to the editor fields
fn sync_keybinding_to_editor(ui: &MainWindow, binding: &Keybinding) {
    ui.set_edit_key_combo(binding.key_combo.clone().into());

    // Set parsed key parts for badge display
    let parts = parse_key_combo_parts(&binding.key_combo);
    ui.set_edit_key_parts(key_parts_to_model(parts));

    ui.set_edit_overlay_title(
        binding
            .hotkey_overlay_title
            .clone()
            .unwrap_or_default()
            .into(),
    );
    ui.set_edit_allow_when_locked(binding.allow_when_locked);
    ui.set_edit_repeat(binding.repeat);
    ui.set_edit_cooldown(binding.cooldown_ms.unwrap_or(0));

    // Set action type and details
    match &binding.action {
        KeybindAction::Spawn(args) => {
            ui.set_edit_action_type(0);
            ui.set_edit_action_command(args.join(" ").into());
            ui.set_edit_action_name("".into());
        }
        KeybindAction::NiriAction(name) => {
            ui.set_edit_action_type(1);
            ui.set_edit_action_name(name.clone().into());
            ui.set_edit_action_command("".into());
        }
        KeybindAction::NiriActionWithArgs(name, args) => {
            ui.set_edit_action_type(2);
            ui.set_edit_action_name(name.clone().into());
            ui.set_edit_action_command(args.join(" ").into());
        }
    }
}
