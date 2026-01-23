//! Backups message handler

use crate::messages::{BackupEntry, BackupsMessage, DialogState, Message};
use iced::Task;
use std::path::PathBuf;

impl super::super::App {
    /// Updates backups state
    pub(in crate::app) fn update_backups(&mut self, msg: BackupsMessage) -> Task<Message> {
        match msg {
            BackupsMessage::RefreshList => {
                self.ui.backups_state.loading_list = true;
                self.ui.backups_state.status_message = None;

                let backup_dir = self.paths.backup_dir.clone();

                Task::perform(
                    async move {
                        list_backups(&backup_dir)
                    },
                    |result| Message::Backups(BackupsMessage::ListLoaded(result)),
                )
            }

            BackupsMessage::ListLoaded(result) => {
                self.ui.backups_state.loading_list = false;
                match result {
                    Ok(backups) => {
                        let count = backups.len();
                        self.ui.backups_state.backups = backups;
                        self.ui.backups_state.status_message = Some(format!("Found {} backup(s)", count));
                        // Clear selection when list refreshes
                        self.ui.backups_state.selected_backup = None;
                        self.ui.backups_state.preview_content = None;
                    }
                    Err(e) => {
                        self.ui.backups_state.status_message = Some(format!("Error: {}", e));
                    }
                }
                Task::none()
            }

            BackupsMessage::SelectBackup(idx) => {
                self.ui.backups_state.selected_backup = Some(idx);
                self.ui.backups_state.loading_preview = true;
                self.ui.backups_state.preview_content = None;

                // Load preview
                if let Some(backup) = self.ui.backups_state.backups.get(idx) {
                    let path = backup.path.clone();

                    Task::perform(
                        async move {
                            std::fs::read_to_string(&path)
                                .map_err(|e| format!("Failed to read backup: {}", e))
                        },
                        |result| Message::Backups(BackupsMessage::PreviewLoaded(result)),
                    )
                } else {
                    self.ui.backups_state.loading_preview = false;
                    Task::none()
                }
            }

            BackupsMessage::PreviewLoaded(result) => {
                self.ui.backups_state.loading_preview = false;
                self.ui.backups_state.preview_content = Some(result);
                Task::none()
            }

            BackupsMessage::ConfirmRestore(idx) => {
                if let Some(backup) = self.ui.backups_state.backups.get(idx) {
                    // Show confirmation dialog
                    self.ui.dialog_state = DialogState::Confirm {
                        title: "Restore Backup".to_string(),
                        message: format!(
                            "Are you sure you want to restore '{}'?\n\n\
                             This will overwrite your current config.kdl file. \
                             A backup of your current config will be created first.",
                            backup.filename
                        ),
                        confirm_label: "Restore".to_string(),
                        on_confirm: crate::messages::ConfirmAction::ResetSettings, // We'll handle this specially
                    };
                    // Store the index for later
                    self.ui.pending_restore_idx = Some(idx);
                }
                Task::none()
            }

            BackupsMessage::RestoreBackup(idx) => {
                self.ui.backups_state.restoring = true;
                self.ui.backups_state.status_message = Some("Restoring...".to_string());

                if let Some(backup) = self.ui.backups_state.backups.get(idx) {
                    let backup_path = backup.path.clone();
                    let config_path = self.paths.niri_config.clone();
                    let backup_dir = self.paths.backup_dir.clone();

                    Task::perform(
                        async move {
                            restore_backup(&backup_path, &config_path, &backup_dir)
                        },
                        |result| Message::Backups(BackupsMessage::RestoreCompleted(result)),
                    )
                } else {
                    self.ui.backups_state.restoring = false;
                    self.ui.backups_state.status_message = Some("Error: Backup not found".to_string());
                    Task::none()
                }
            }

            BackupsMessage::RestoreCompleted(result) => {
                self.ui.backups_state.restoring = false;
                match result {
                    Ok(()) => {
                        self.ui.backups_state.status_message = Some("Backup restored successfully!".to_string());
                        self.ui.toast = Some("Backup restored! Restart Nirify to see changes.".to_string());
                        self.ui.toast_shown_at = Some(std::time::Instant::now());
                    }
                    Err(e) => {
                        self.ui.backups_state.status_message = Some(format!("Failed to restore: {}", e));
                    }
                }
                Task::none()
            }
        }
    }
}

/// List all backups in the backup directory
fn list_backups(backup_dir: &std::path::Path) -> Result<Vec<BackupEntry>, String> {
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(backup_dir)
        .map_err(|e| format!("Failed to read backup directory: {}", e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Only include backup files (config.kdl.backup-*)
            if filename.starts_with("config.kdl.backup-") || filename.contains(".backup-") {
                let metadata = std::fs::metadata(&path).ok();

                let date = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| format_system_time(t))
                    .unwrap_or_else(|| extract_timestamp_from_filename(filename));

                let size = metadata
                    .as_ref()
                    .map(|m| format_file_size(m.len()))
                    .unwrap_or_else(|| "?".to_string());

                entries.push(BackupEntry {
                    filename: filename.to_string(),
                    date,
                    size,
                    path: path.clone(),
                });
            }
        }
    }

    // Sort by modification time (newest first)
    entries.sort_by(|a, b| {
        let a_time = std::fs::metadata(&a.path)
            .and_then(|m| m.modified())
            .ok();
        let b_time = std::fs::metadata(&b.path)
            .and_then(|m| m.modified())
            .ok();
        b_time.cmp(&a_time)
    });

    Ok(entries)
}

/// Format a SystemTime as a human-readable string
fn format_system_time(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Convert to date/time components (simplified, not handling timezones)
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;

    // Approximate date calculation (good enough for display)
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month.min(12), day.min(31), hours, minutes
    )
}

/// Extract timestamp from backup filename (e.g., config.kdl.backup-20240115-143045)
fn extract_timestamp_from_filename(filename: &str) -> String {
    // Try to find the timestamp pattern
    if let Some(pos) = filename.rfind("-20") {
        let timestamp_part = &filename[pos + 1..];
        // Parse YYYYMMDD-HHMMSS or YYYYMMDD_HHMMSS
        if timestamp_part.len() >= 15 {
            let year = &timestamp_part[0..4];
            let month = &timestamp_part[4..6];
            let day = &timestamp_part[6..8];
            let hour = &timestamp_part[9..11];
            let minute = &timestamp_part[11..13];
            let second = &timestamp_part[13..15];
            return format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);
        }
    }
    "Unknown date".to_string()
}

/// Format file size in human-readable form
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Restore a backup to the main config file
fn restore_backup(
    backup_path: &PathBuf,
    config_path: &std::path::Path,
    backup_dir: &std::path::Path,
) -> Result<(), String> {
    // First, create a backup of the current config
    if config_path.exists() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let current_backup_name = format!("config.kdl.backup-{}", timestamp);
        let current_backup_path = backup_dir.join(current_backup_name);

        // Ensure backup directory exists
        if !backup_dir.exists() {
            std::fs::create_dir_all(backup_dir)
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }

        std::fs::copy(config_path, &current_backup_path)
            .map_err(|e| format!("Failed to backup current config: {}", e))?;

        log::info!("Created backup of current config: {}", current_backup_path.display());
    }

    // Read the backup content
    let backup_content = std::fs::read_to_string(backup_path)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    // Write to config file
    std::fs::write(config_path, backup_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!("Restored backup from: {}", backup_path.display());

    Ok(())
}
