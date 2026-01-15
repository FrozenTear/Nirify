use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during config file operations
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read a config file (includes file path for context)
    #[error("Failed to read config file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse KDL (includes file path for context)
    #[error("Failed to parse KDL in '{path}': {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: kdl::KdlError,
    },

    /// Generic I/O error without path context (for compatibility)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic KDL parse error without path context (for compatibility)
    #[error("KDL parse error: {0}")]
    KdlError(#[from] kdl::KdlError),

    /// Invalid configuration value or state
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// XDG config directory could not be determined
    #[error("Config directory not found")]
    ConfigDirNotFound,

    /// Failed to create a config directory
    #[error("Failed to create config directory '{path}': {message}")]
    CreateDirError { path: PathBuf, message: String },

    /// Backup operation failed
    #[error("Backup failed for '{path}': {message}")]
    BackupError { path: PathBuf, message: String },
}

impl ConfigError {
    /// Create a ReadError with file path context
    pub fn read_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    /// Create a ParseError with file path context
    pub fn parse_error(path: impl Into<PathBuf>, source: kdl::KdlError) -> Self {
        Self::ParseError {
            path: path.into(),
            source,
        }
    }

    /// Create a CreateDirError with path context
    pub fn create_dir_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::CreateDirError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a BackupError with path context
    pub fn backup_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::BackupError {
            path: path.into(),
            message: message.into(),
        }
    }
}
