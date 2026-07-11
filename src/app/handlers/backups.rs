//! Backups message handler

use crate::config::registry::ConfigFile;
use crate::config::ConfigPaths;
use crate::messages::{BackupEntry, BackupsMessage, DialogState, Message, RestoredTarget};
use iced::Task;
use std::path::{Path, PathBuf};

impl super::super::App {
    /// Updates backups state
    pub(in crate::app) fn update_backups(&mut self, msg: BackupsMessage) -> Task<Message> {
        match msg {
            BackupsMessage::RefreshList => {
                self.ui.backups_state.loading_list = true;
                self.ui.backups_state.status_message = None;

                let backup_dir = self.paths.backup_dir.clone();

                Task::perform(async move { list_backups(&backup_dir) }, |result| {
                    Message::Backups(BackupsMessage::ListLoaded(result))
                })
            }

            BackupsMessage::ListLoaded(result) => {
                self.ui.backups_state.loading_list = false;
                match result {
                    Ok(backups) => {
                        let count = backups.len();
                        self.ui.backups_state.backups = backups;
                        self.ui.backups_state.status_message =
                            Some(format!("Found {} backup(s)", count));
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
                    // Show confirmation dialog; the actual restore runs on confirm.
                    self.ui.dialog_state = DialogState::Confirm {
                        title: "Restore Backup".to_string(),
                        message: format!(
                            "Are you sure you want to restore '{}'?\n\n\
                             This overwrites the corresponding config file. A backup of \
                             the current file is created first, and any unsaved changes \
                             will be reloaded from disk.",
                            backup.filename
                        ),
                        confirm_label: "Restore".to_string(),
                        on_confirm: crate::messages::ConfirmAction::RestoreBackup(idx),
                    };
                }
                Task::none()
            }

            BackupsMessage::RestoreBackup(idx) => {
                self.ui.backups_state.restoring = true;
                self.ui.backups_state.status_message = Some("Restoring...".to_string());

                if let Some(backup) = self.ui.backups_state.backups.get(idx) {
                    let filename = backup.filename.clone();
                    let backup_path = backup.path.clone();
                    let backup_dir = self.paths.backup_dir.clone();

                    match resolve_restore_target(&filename, &self.paths) {
                        Some((target_path, target)) => Task::perform(
                            async move {
                                restore_backup(&backup_path, &target_path, target, &backup_dir)
                            },
                            |result| Message::Backups(BackupsMessage::RestoreCompleted(result)),
                        ),
                        None => {
                            self.ui.backups_state.restoring = false;
                            self.ui.backups_state.status_message =
                                Some(format!("Cannot determine restore target for {}", filename));
                            Task::none()
                        }
                    }
                } else {
                    self.ui.backups_state.restoring = false;
                    self.ui.backups_state.status_message =
                        Some("Error: Backup not found".to_string());
                    Task::none()
                }
            }

            BackupsMessage::RestoreCompleted(result) => {
                self.ui.backups_state.restoring = false;
                match result {
                    Ok(target) => {
                        self.ui.backups_state.status_message =
                            Some("Backup restored successfully!".to_string());
                        match target {
                            RestoredTarget::Managed => {
                                // Reload settings fresh from disk and drop all in-memory
                                // dirty/backup bookkeeping (restore is explicit). Rebuild the
                                // blocked set from files that failed to read so a restore that
                                // reintroduces an unreadable file re-pauses saving that
                                // category (mirrors App::new), rather than clearing wholesale.
                                let load_result =
                                    crate::config::load_settings_with_result(&self.paths);
                                self.settings = load_result.settings;
                                self.save.dirty_tracker = crate::config::DirtyTracker::new();
                                self.save.blocked = load_result
                                    .failed_files
                                    .iter()
                                    .filter_map(|f| {
                                        crate::config::SettingsCategory::from_relative_path(f)
                                    })
                                    .collect();
                                self.save.in_flight.clear();
                                self.save.backed_up.clear();
                                self.save.last_change_time = None;
                                self.save.last_failure_time = None;
                                self.ui.error_banner = None;
                                self.ui.tablet_calibration_cache =
                                    crate::views::widgets::format_matrix_values(
                                        self.settings.tablet.calibration_matrix,
                                    );
                                self.ui.touch_calibration_cache =
                                    crate::views::widgets::format_matrix_values(
                                        self.settings.touch.calibration_matrix,
                                    );
                                self.ui.mouse_scroll_factor_text =
                                    format!("{}", self.settings.mouse.scroll_factor);
                                self.ui.touchpad_scroll_factor_text =
                                    format!("{}", self.settings.touchpad.scroll_factor);
                                self.ui.toast = Some("Backup restored".to_string());
                                self.ui.toast_shown_at = Some(std::time::Instant::now());
                                self.reload_niri_config_task()
                            }
                            RestoredTarget::NiriConfig => {
                                self.ui.toast = Some(
                                    "Backup restored! Restart Nirify to see changes.".to_string(),
                                );
                                self.ui.toast_shown_at = Some(std::time::Instant::now());
                                self.reload_niri_config_task()
                            }
                        }
                    }
                    Err(e) => {
                        self.ui.backups_state.status_message =
                            Some(format!("Failed to restore: {}", e));
                        Task::none()
                    }
                }
            }
        }
    }
}

/// Determine which file a backup should be restored to, and whether it is the
/// user's main niri config or a managed category file.
///
/// Returns `None` when the target cannot be resolved (never falls back to
/// overwriting the main config for a non-`config.kdl` backup).
fn resolve_restore_target(
    filename: &str,
    paths: &ConfigPaths,
) -> Option<(PathBuf, RestoredTarget)> {
    if filename.starts_with("config.kdl.backup-") {
        return Some((paths.niri_config.clone(), RestoredTarget::NiriConfig));
    }

    // Category backups look like "<name>.kdl.<timestamp>.bak"
    let base = &filename[..filename.find(".kdl")? + 4];
    let cf = ConfigFile::from_file_name(base)?;
    Some((cf.full_path(&paths.managed_dir), RestoredTarget::Managed))
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
            // Include main-config backups (config.kdl.backup-*) and per-category
            // backups (<name>.kdl.<ts>.bak).
            if filename.starts_with("config.kdl.backup-")
                || filename.contains(".backup-")
                || filename.ends_with(".bak")
            {
                let metadata = std::fs::metadata(&path).ok();

                let date = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(format_system_time)
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
        let a_time = std::fs::metadata(&a.path).and_then(|m| m.modified()).ok();
        let b_time = std::fs::metadata(&b.path).and_then(|m| m.modified()).ok();
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
        year,
        month.min(12),
        day.min(31),
        hours,
        minutes
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

/// Restore a backup to the resolved target file.
///
/// Before overwriting, the current target file (if present) is snapshotted:
/// the main config uses the `config.kdl.backup-<ts>` scheme, managed files use
/// the `<filename>.<ts>.bak` scheme (matching `save_with_backup`).
fn restore_backup(
    backup_path: &Path,
    target_path: &Path,
    target: RestoredTarget,
    backup_dir: &Path,
) -> Result<RestoredTarget, String> {
    use chrono::Local;

    // Read backup content first (validates it exists and is readable)
    let backup_content = std::fs::read_to_string(backup_path)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    // Validate backup contains valid KDL before restoring
    if let Err(e) = backup_content.parse::<kdl::KdlDocument>() {
        return Err(format!("Backup contains invalid KDL: {}", e));
    }

    // Create a backup of the current target file (read first to avoid TOCTOU)
    if target_path.exists() {
        let current_content = std::fs::read_to_string(target_path)
            .map_err(|e| format!("Failed to read current config: {}", e))?;

        let current_backup_name = match target {
            RestoredTarget::NiriConfig => {
                let timestamp = Local::now().format("%Y%m%dT%H%M%S%.6f");
                format!("config.kdl.backup-{}", timestamp)
            }
            RestoredTarget::Managed => {
                let fname = target_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("config.kdl");
                let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S%.6f");
                format!("{}.{}.bak", fname, timestamp)
            }
        };
        let current_backup_path = backup_dir.join(current_backup_name);

        // Ensure backup directory exists
        if !backup_dir.exists() {
            std::fs::create_dir_all(backup_dir)
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }

        // Use atomic write for backup
        crate::config::atomic_write(&current_backup_path, &current_content)
            .map_err(|e| format!("Failed to backup current config: {}", e))?;

        log::info!(
            "Created backup of current file: {}",
            current_backup_path.display()
        );
    }

    // Write to target file using atomic write (safe against crashes)
    crate::config::atomic_write(target_path, &backup_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    log::info!(
        "Restored backup from {} to {}",
        backup_path.display(),
        target_path.display()
    );

    Ok(target)
}

#[cfg(test)]
// Test setup mutates a couple fields after default() for readability.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    fn test_paths() -> ConfigPaths {
        let mut p = ConfigPaths::default();
        p.niri_config = PathBuf::from("/tmp/nirify-test/config.kdl");
        p.managed_dir = PathBuf::from("/tmp/nirify-test/managed");
        p
    }

    #[test]
    fn resolve_restore_target_config_kdl() {
        let paths = test_paths();
        let (path, target) =
            resolve_restore_target("config.kdl.backup-20260704T120000.000000", &paths).unwrap();
        assert_eq!(path, paths.niri_config);
        assert_eq!(target, RestoredTarget::NiriConfig);
    }

    #[test]
    fn resolve_restore_target_category_bak() {
        let paths = test_paths();
        let (path, target) =
            resolve_restore_target("appearance.kdl.2026-07-04T12-00-00.123456.bak", &paths)
                .unwrap();
        assert_eq!(path, paths.managed_dir.join("appearance.kdl"));
        assert_eq!(target, RestoredTarget::Managed);
    }

    #[test]
    fn resolve_restore_target_unknown_is_none() {
        let paths = test_paths();
        assert!(resolve_restore_target("garbage.txt.bak", &paths).is_none());
    }
}
