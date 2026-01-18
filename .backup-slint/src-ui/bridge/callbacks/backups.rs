//! Backup browser UI callbacks
//!
//! Handles backup listing, preview, and restore operations.

use crate::config;
use crate::config::paths::ConfigPaths;
use crate::MainWindow;
use log::{info, warn};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

// Import BackupEntry from the generated Slint bindings
use crate::BackupEntry;

/// Set up backup browser callbacks
pub fn setup(ui: &MainWindow, paths: Arc<ConfigPaths>) {
    // Store backup file paths for preview/restore operations
    // Using RefCell since Slint callbacks are single-threaded
    let backup_paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));

    // Refresh callback - list all backups in the backup directory
    setup_refresh_callback(ui, paths.backup_dir.clone(), backup_paths.clone());

    // Preview callback - show contents of selected backup
    setup_preview_callback(ui, backup_paths.clone());

    // Restore callback - restore selected backup to config.kdl
    setup_restore_callback(ui, paths.niri_config.clone(), backup_paths);
}

/// Set up the refresh callback to list backups
fn setup_refresh_callback(
    ui: &MainWindow,
    backup_dir: PathBuf,
    backup_paths: Rc<RefCell<Vec<PathBuf>>>,
) {
    let ui_weak = ui.as_weak();

    ui.on_backup_refresh(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let (entries, paths_list) = list_backups(&backup_dir);
            *backup_paths.borrow_mut() = paths_list;

            let has_backups = !entries.is_empty();
            let count = entries.len();

            let model = Rc::new(VecModel::from(entries));
            ui.set_backup_list(ModelRc::from(model));
            ui.set_backup_has_backups(has_backups);
            ui.set_backup_status(format!("Found {} backup(s)", count).into());
            ui.set_backup_preview("".into());
            ui.set_backup_selected_index(-1);
        }
    });
}

/// Set up the preview callback to show backup contents
fn setup_preview_callback(ui: &MainWindow, backup_paths: Rc<RefCell<Vec<PathBuf>>>) {
    let ui_weak = ui.as_weak();

    ui.on_backup_preview_requested(move |idx| {
        if let Some(ui) = ui_weak.upgrade() {
            let paths = backup_paths.borrow();
            if let Some(path) = paths.get(idx as usize) {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        ui.set_backup_preview(content.into());
                    }
                    Err(e) => {
                        ui.set_backup_preview(format!("Error reading backup: {}", e).into());
                    }
                }
            }
        }
    });
}

/// Set up the restore callback to restore a backup
fn setup_restore_callback(
    ui: &MainWindow,
    config_path: PathBuf,
    backup_paths: Rc<RefCell<Vec<PathBuf>>>,
) {
    let ui_weak = ui.as_weak();

    ui.on_backup_restore_requested(move |idx| {
        if let Some(ui) = ui_weak.upgrade() {
            let paths = backup_paths.borrow();
            if let Some(backup_path) = paths.get(idx as usize) {
                match std::fs::read_to_string(backup_path) {
                    Ok(content) => {
                        // Write backup content to config.kdl using atomic write for safety
                        match config::atomic_write(&config_path, &content) {
                            Ok(()) => {
                                info!("Restored config from backup: {:?}", backup_path);
                                ui.invoke_show_status_toast(
                                    "Config restored from backup".into(),
                                    false,
                                );
                                ui.set_backup_status("Backup restored successfully".into());
                            }
                            Err(e) => {
                                warn!("Failed to restore backup: {}", e);
                                ui.invoke_show_status_toast(
                                    format!("Restore failed: {}", e).into(),
                                    true,
                                );
                                ui.set_backup_status(format!("Error: {}", e).into());
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read backup: {}", e);
                        ui.invoke_show_status_toast(
                            format!("Failed to read backup: {}", e).into(),
                            true,
                        );
                    }
                }
            }
        }
    });
}

/// List backup files and return entries for UI display
fn list_backups(backup_dir: &std::path::Path) -> (Vec<BackupEntry>, Vec<PathBuf>) {
    let mut entries = Vec::new();
    let mut paths = Vec::new();

    if !backup_dir.exists() {
        return (entries, paths);
    }

    let Ok(read_dir) = std::fs::read_dir(backup_dir) else {
        return (entries, paths);
    };

    let mut backup_files: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("config.kdl.backup-")
        })
        .collect();

    // Sort by modification time, newest first
    backup_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    for entry in backup_files {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();

        // Extract date from filename (format: config.kdl.backup-YYYYMMDD_HHMMSS)
        let date = if let Some(timestamp) = filename.strip_prefix("config.kdl.backup-") {
            format_backup_timestamp(timestamp)
        } else {
            "Unknown date".to_string()
        };

        // Get file size
        let size = entry
            .metadata()
            .map(|m| format_file_size(m.len()))
            .unwrap_or_else(|_| "?".to_string());

        entries.push(BackupEntry {
            filename: filename.into(),
            date: date.into(),
            size: size.into(),
        });
        paths.push(path);
    }

    (entries, paths)
}

/// Format backup timestamp for display
/// Format: YYYYMMDD_HHMMSS -> "YYYY-MM-DD HH:MM:SS"
fn format_backup_timestamp(timestamp: &str) -> String {
    // Check is_ascii() to ensure byte indexing is safe (no multi-byte chars)
    if timestamp.len() >= 15 && timestamp.is_ascii() {
        let year = &timestamp[0..4];
        let month = &timestamp[4..6];
        let day = &timestamp[6..8];
        let hour = &timestamp[9..11];
        let min = &timestamp[11..13];
        let sec = &timestamp[13..15];
        format!("{year}-{month}-{day} {hour}:{min}:{sec}")
    } else {
        timestamp.to_string()
    }
}

/// Format file size for display
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
