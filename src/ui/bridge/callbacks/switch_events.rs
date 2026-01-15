//! Switch events UI callbacks
//!
//! Handles lid close/open and tablet mode switch event configuration.

use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use log::{debug, error};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::SaveManager;

/// Build command list model for UI display
fn build_command_list(commands: &[String]) -> ModelRc<SharedString> {
    let display_list: Vec<SharedString> = commands
        .iter()
        .map(|s| SharedString::from(s.as_str()))
        .collect();
    ModelRc::new(VecModel::from(display_list))
}

/// Set up switch events callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // ========================================================================
    // LID CLOSE
    // ========================================================================
    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_close_add(move || {
            match settings.lock() {
                Ok(mut s) => {
                    // Get current text from UI
                    if let Some(ui) = ui_weak.upgrade() {
                        let cmd = ui.get_switch_lid_close_current().to_string();
                        if !cmd.trim().is_empty() {
                            s.switch_events.lid_close.spawn.push(cmd.clone());
                            ui.set_switch_lid_close_commands(build_command_list(
                                &s.switch_events.lid_close.spawn,
                            ));
                            ui.set_switch_lid_close_current(SharedString::new());
                            ui.set_switch_lid_close_selected(-1);
                            debug!("Added lid-close command: {}", cmd);
                            save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                            save_manager.request_save();
                        }
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_close_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.switch_events.lid_close.spawn.len() {
                        s.switch_events.lid_close.spawn.remove(idx);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_switch_lid_close_commands(build_command_list(
                                &s.switch_events.lid_close.spawn,
                            ));
                            ui.set_switch_lid_close_selected(-1);
                            ui.set_switch_lid_close_current(SharedString::new());
                        }
                        debug!("Removed lid-close command at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        ui.on_switch_lid_close_select(move |index| match settings.lock() {
            Ok(s) => {
                if let Some(cmd) = s.switch_events.lid_close.spawn.get(index as usize) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_switch_lid_close_current(SharedString::from(cmd.as_str()));
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_close_changed(move |value| {
            if let Some(ui) = ui_weak.upgrade() {
                let idx = ui.get_switch_lid_close_selected();
                if idx >= 0 {
                    match settings.lock() {
                        Ok(mut s) => {
                            if let Some(cmd) = s.switch_events.lid_close.spawn.get_mut(idx as usize)
                            {
                                *cmd = value.to_string();
                                ui.set_switch_lid_close_commands(build_command_list(
                                    &s.switch_events.lid_close.spawn,
                                ));
                                save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                                save_manager.request_save();
                            }
                        }
                        Err(e) => error!("Settings lock error: {}", e),
                    }
                }
            }
        });
    }

    // ========================================================================
    // LID OPEN
    // ========================================================================
    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_open_add(move || match settings.lock() {
            Ok(mut s) => {
                if let Some(ui) = ui_weak.upgrade() {
                    let cmd = ui.get_switch_lid_open_current().to_string();
                    if !cmd.trim().is_empty() {
                        s.switch_events.lid_open.spawn.push(cmd.clone());
                        ui.set_switch_lid_open_commands(build_command_list(
                            &s.switch_events.lid_open.spawn,
                        ));
                        ui.set_switch_lid_open_current(SharedString::new());
                        ui.set_switch_lid_open_selected(-1);
                        debug!("Added lid-open command: {}", cmd);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_open_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.switch_events.lid_open.spawn.len() {
                        s.switch_events.lid_open.spawn.remove(idx);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_switch_lid_open_commands(build_command_list(
                                &s.switch_events.lid_open.spawn,
                            ));
                            ui.set_switch_lid_open_selected(-1);
                            ui.set_switch_lid_open_current(SharedString::new());
                        }
                        debug!("Removed lid-open command at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        ui.on_switch_lid_open_select(move |index| match settings.lock() {
            Ok(s) => {
                if let Some(cmd) = s.switch_events.lid_open.spawn.get(index as usize) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_switch_lid_open_current(SharedString::from(cmd.as_str()));
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_lid_open_changed(move |value| {
            if let Some(ui) = ui_weak.upgrade() {
                let idx = ui.get_switch_lid_open_selected();
                if idx >= 0 {
                    match settings.lock() {
                        Ok(mut s) => {
                            if let Some(cmd) = s.switch_events.lid_open.spawn.get_mut(idx as usize)
                            {
                                *cmd = value.to_string();
                                ui.set_switch_lid_open_commands(build_command_list(
                                    &s.switch_events.lid_open.spawn,
                                ));
                                save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                                save_manager.request_save();
                            }
                        }
                        Err(e) => error!("Settings lock error: {}", e),
                    }
                }
            }
        });
    }

    // ========================================================================
    // TABLET MODE ON
    // ========================================================================
    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_on_add(move || match settings.lock() {
            Ok(mut s) => {
                if let Some(ui) = ui_weak.upgrade() {
                    let cmd = ui.get_switch_tablet_mode_on_current().to_string();
                    if !cmd.trim().is_empty() {
                        s.switch_events.tablet_mode_on.spawn.push(cmd.clone());
                        ui.set_switch_tablet_mode_on_commands(build_command_list(
                            &s.switch_events.tablet_mode_on.spawn,
                        ));
                        ui.set_switch_tablet_mode_on_current(SharedString::new());
                        ui.set_switch_tablet_mode_on_selected(-1);
                        debug!("Added tablet-mode-on command: {}", cmd);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_on_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.switch_events.tablet_mode_on.spawn.len() {
                        s.switch_events.tablet_mode_on.spawn.remove(idx);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_switch_tablet_mode_on_commands(build_command_list(
                                &s.switch_events.tablet_mode_on.spawn,
                            ));
                            ui.set_switch_tablet_mode_on_selected(-1);
                            ui.set_switch_tablet_mode_on_current(SharedString::new());
                        }
                        debug!("Removed tablet-mode-on command at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        ui.on_switch_tablet_mode_on_select(move |index| match settings.lock() {
            Ok(s) => {
                if let Some(cmd) = s.switch_events.tablet_mode_on.spawn.get(index as usize) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_switch_tablet_mode_on_current(SharedString::from(cmd.as_str()));
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_on_changed(move |value| {
            if let Some(ui) = ui_weak.upgrade() {
                let idx = ui.get_switch_tablet_mode_on_selected();
                if idx >= 0 {
                    match settings.lock() {
                        Ok(mut s) => {
                            if let Some(cmd) =
                                s.switch_events.tablet_mode_on.spawn.get_mut(idx as usize)
                            {
                                *cmd = value.to_string();
                                ui.set_switch_tablet_mode_on_commands(build_command_list(
                                    &s.switch_events.tablet_mode_on.spawn,
                                ));
                                save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                                save_manager.request_save();
                            }
                        }
                        Err(e) => error!("Settings lock error: {}", e),
                    }
                }
            }
        });
    }

    // ========================================================================
    // TABLET MODE OFF
    // ========================================================================
    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_off_add(move || match settings.lock() {
            Ok(mut s) => {
                if let Some(ui) = ui_weak.upgrade() {
                    let cmd = ui.get_switch_tablet_mode_off_current().to_string();
                    if !cmd.trim().is_empty() {
                        s.switch_events.tablet_mode_off.spawn.push(cmd.clone());
                        ui.set_switch_tablet_mode_off_commands(build_command_list(
                            &s.switch_events.tablet_mode_off.spawn,
                        ));
                        ui.set_switch_tablet_mode_off_current(SharedString::new());
                        ui.set_switch_tablet_mode_off_selected(-1);
                        debug!("Added tablet-mode-off command: {}", cmd);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_off_remove(move |index| {
            let idx = index as usize;
            match settings.lock() {
                Ok(mut s) => {
                    if idx < s.switch_events.tablet_mode_off.spawn.len() {
                        s.switch_events.tablet_mode_off.spawn.remove(idx);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_switch_tablet_mode_off_commands(build_command_list(
                                &s.switch_events.tablet_mode_off.spawn,
                            ));
                            ui.set_switch_tablet_mode_off_selected(-1);
                            ui.set_switch_tablet_mode_off_current(SharedString::new());
                        }
                        debug!("Removed tablet-mode-off command at index {}", idx);
                        save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                        save_manager.request_save();
                    }
                }
                Err(e) => error!("Settings lock error: {}", e),
            }
        });
    }

    {
        let settings = settings.clone();
        let ui_weak = ui.as_weak();
        ui.on_switch_tablet_mode_off_select(move |index| match settings.lock() {
            Ok(s) => {
                if let Some(cmd) = s.switch_events.tablet_mode_off.spawn.get(index as usize) {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_switch_tablet_mode_off_current(SharedString::from(cmd.as_str()));
                    }
                }
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }

    {
        let ui_weak = ui.as_weak();
        let save_manager = Rc::clone(&save_manager);
        ui.on_switch_tablet_mode_off_changed(move |value| {
            if let Some(ui) = ui_weak.upgrade() {
                let idx = ui.get_switch_tablet_mode_off_selected();
                if idx >= 0 {
                    match settings.lock() {
                        Ok(mut s) => {
                            if let Some(cmd) =
                                s.switch_events.tablet_mode_off.spawn.get_mut(idx as usize)
                            {
                                *cmd = value.to_string();
                                ui.set_switch_tablet_mode_off_commands(build_command_list(
                                    &s.switch_events.tablet_mode_off.spawn,
                                ));
                                save_manager.mark_dirty(SettingsCategory::SwitchEvents);
                                save_manager.request_save();
                            }
                        }
                        Err(e) => error!("Settings lock error: {}", e),
                    }
                }
            }
        });
    }
}
