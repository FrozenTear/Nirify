pub mod category_section;
pub mod consolidation;
pub mod dirty;
pub mod error;
pub mod loader;
pub mod models;
pub mod parser;
pub mod paths;
pub mod registry;
pub mod replace;
pub mod storage;
pub mod validation;

pub use crate::types::CenterFocusedColumn;
pub use category_section::CategorySection;
pub use consolidation::{analyze_rules, ConsolidationAnalysis, ConsolidationSuggestion};
pub use dirty::{DirtyTracker, SettingsCategory};
pub use error::ConfigError;
pub use loader::{
    check_config_health, import_from_niri_config, import_from_niri_config_with_result,
    load_settings, load_settings_with_result, repair_corrupted_configs, ConfigFileStatus,
    ConfigHealthReport, FileLoadStatus, ImportResult, LoadResult,
};
pub use models::{
    ColumnWidthType, LayoutOverride, OutputConfig, OutputHotCorners, OutputSettings, Settings,
    WorkspaceShadow,
};
pub use paths::ConfigPaths;
pub use registry::ConfigFile;
pub use replace::{smart_replace_config, SmartReplaceResult};
pub use storage::{atomic_write, save_dirty, save_settings};
pub use validation::{validate_string, validate_string_opt};
