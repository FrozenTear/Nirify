//! Config health checking and repair
//!
//! Provides functionality to check the health of config files and repair
//! corrupted ones by backing them up and regenerating with defaults.

use super::super::parser::parse_document;
use super::super::paths::ConfigPaths;
use super::super::registry::ConfigFile;
use super::super::storage::{atomic_write, save_settings};
use crate::config::models::Settings;
use chrono::Local;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Status of a single config file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFileStatus {
    /// File exists and is valid KDL
    Ok,
    /// File does not exist (will use defaults)
    Missing,
    /// File exists but contains invalid KDL
    Corrupted(String),
    /// File could not be read (permissions, etc.)
    Unreadable(String),
}

/// Health report for all config files
///
/// Uses the ConfigFile registry to avoid duplicating file mappings.
#[derive(Debug, Clone)]
pub struct ConfigHealthReport {
    statuses: HashMap<ConfigFile, ConfigFileStatus>,
}

impl ConfigHealthReport {
    /// Create a new health report by checking all config files
    pub(crate) fn new(paths: &ConfigPaths) -> Self {
        let mut statuses = HashMap::new();
        for file in ConfigFile::HEALTH_CHECK {
            statuses.insert(*file, check_file_status(&paths.path_for(*file)));
        }
        Self { statuses }
    }

    /// Get the status of a specific config file
    pub fn status(&self, file: ConfigFile) -> &ConfigFileStatus {
        self.statuses
            .get(&file)
            .unwrap_or(&ConfigFileStatus::Missing)
    }

    /// Returns true if all files are either Ok or Missing (no corruption)
    pub fn is_healthy(&self) -> bool {
        self.corrupted_files().is_empty() && self.unreadable_files().is_empty()
    }

    /// Returns list of corrupted file names
    pub fn corrupted_files(&self) -> Vec<&'static str> {
        self.statuses
            .iter()
            .filter(|(_, status)| matches!(status, ConfigFileStatus::Corrupted(_)))
            .map(|(file, _)| file.file_name())
            .collect()
    }

    /// Returns list of unreadable file names
    pub fn unreadable_files(&self) -> Vec<&'static str> {
        self.statuses
            .iter()
            .filter(|(_, status)| matches!(status, ConfigFileStatus::Unreadable(_)))
            .map(|(file, _)| file.file_name())
            .collect()
    }

    // Legacy field accessors for backward compatibility
    // These can be removed once all callers are migrated to use status()

    /// Get appearance config status
    pub fn appearance(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Appearance)
    }

    /// Get behavior config status
    pub fn behavior(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Behavior)
    }

    /// Get keyboard config status
    pub fn keyboard(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Keyboard)
    }

    /// Get mouse config status
    pub fn mouse(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Mouse)
    }

    /// Get touchpad config status
    pub fn touchpad(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Touchpad)
    }

    /// Get animations config status
    pub fn animations(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Animations)
    }

    /// Get cursor config status
    pub fn cursor(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Cursor)
    }

    /// Get overview config status
    pub fn overview(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Overview)
    }

    /// Get outputs config status
    pub fn outputs(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Outputs)
    }

    /// Get layout_extras config status
    pub fn layout_extras(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::LayoutExtras)
    }

    /// Get gestures config status
    pub fn gestures(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Gestures)
    }

    /// Get misc config status
    pub fn misc(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Misc)
    }

    /// Get workspaces config status
    pub fn workspaces(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::Workspaces)
    }

    /// Get layer_rules config status
    pub fn layer_rules(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::LayerRules)
    }

    /// Get window_rules config status
    pub fn window_rules(&self) -> &ConfigFileStatus {
        self.status(ConfigFile::WindowRules)
    }
}

/// Check the status of a single config file
fn check_file_status(path: &Path) -> ConfigFileStatus {
    if !path.exists() {
        return ConfigFileStatus::Missing;
    }

    match fs::read_to_string(path) {
        Ok(content) => match parse_document(&content) {
            Ok(_) => ConfigFileStatus::Ok,
            Err(e) => ConfigFileStatus::Corrupted(e.to_string()),
        },
        Err(e) => ConfigFileStatus::Unreadable(e.to_string()),
    }
}

/// Check the health of all config files
///
/// Returns a report indicating which files are valid, missing, or corrupted.
/// This is useful for diagnosing configuration issues.
///
/// # Example
///
/// ```ignore
/// let paths = ConfigPaths::new()?;
/// let health = check_config_health(&paths);
/// if !health.is_healthy() {
///     for file in health.corrupted_files() {
///         println!("Corrupted: {}", file);
///     }
/// }
/// ```
#[must_use]
pub fn check_config_health(paths: &ConfigPaths) -> ConfigHealthReport {
    ConfigHealthReport::new(paths)
}

/// Repair corrupted config files by backing them up and regenerating with defaults
///
/// This function:
/// 1. Checks which files are corrupted
/// 2. Creates timestamped backups of corrupted files in the backup directory
/// 3. Regenerates the corrupted files with current settings (or defaults for corrupted sections)
///
/// Returns the list of files that were repaired.
///
/// # Arguments
/// * `paths` - Configuration paths
/// * `current_settings` - Current settings to preserve for non-corrupted sections
///
/// # Example
///
/// ```ignore
/// let paths = ConfigPaths::new()?;
/// let settings = load_settings(&paths); // Falls back to defaults for corrupted
/// let repaired = repair_corrupted_configs(&paths, &settings)?;
/// for file in &repaired {
///     println!("Repaired: {}", file);
/// }
/// ```
pub fn repair_corrupted_configs(
    paths: &ConfigPaths,
    current_settings: &Settings,
) -> anyhow::Result<Vec<String>> {
    let health = check_config_health(paths);
    let corrupted = health.corrupted_files();

    if corrupted.is_empty() {
        debug!("No corrupted config files to repair");
        return Ok(Vec::new());
    }

    // Ensure backup directory exists
    fs::create_dir_all(&paths.backup_dir)?;

    let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S");
    let mut repaired = Vec::new();

    for file_name in &corrupted {
        // Get path from ConfigPaths helper to avoid duplicating file mappings
        let file_path = match paths.path_for_file(file_name) {
            Some(path) => path,
            None => continue,
        };

        // Create backup using read + atomic_write to avoid TOCTOU race
        let backup_name = format!("{}.{}.corrupted.bak", file_name, timestamp);
        let backup_path = paths.backup_dir.join(&backup_name);

        // Read content first, then write atomically
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read {:?} for backup: {}", file_path, e);
                continue;
            }
        };

        if let Err(e) = atomic_write(&backup_path, &content) {
            warn!("Failed to write backup {:?}: {}", backup_path, e);
            continue;
        }

        info!(
            "Backed up corrupted {} to {:?}",
            file_name,
            backup_path.file_name().unwrap_or_default()
        );
        repaired.push(file_name.to_string());
    }

    // Regenerate all files with current settings
    // (corrupted sections will have fallen back to defaults during load)
    if !repaired.is_empty() {
        save_settings(paths, current_settings)?;
        info!("Regenerated {} corrupted config file(s)", repaired.len());
    }

    Ok(repaired)
}
