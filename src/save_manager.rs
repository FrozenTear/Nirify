//! Result types for the App save pipeline (the pipeline itself lives in app/mod.rs).
//!
//! The debounced auto-save state machine (dirty tracking, debounce timing,
//! validation gating, backups, retry backoff) is implemented directly on the
//! `App` struct in `app/mod.rs`. This module only holds the result enums that
//! the async save/reload tasks report back through `Message`.

use crate::config::SettingsCategory;

/// Result of a save operation
#[derive(Debug, Clone)]
pub enum SaveResult {
    /// Save completed successfully
    Success {
        files_written: usize,
        categories: Vec<SettingsCategory>,
    },
    /// Save failed with error. `categories` are the categories that were being
    /// written so the caller can re-mark them dirty and retry.
    Error {
        message: String,
        categories: Vec<SettingsCategory>,
    },
    /// Nothing needed saving
    NothingToSave,
}

/// Result of niri config reload
#[derive(Debug, Clone)]
pub enum ReloadResult {
    /// Reload successful
    Success,
    /// Reload failed
    Error { message: String },
}
