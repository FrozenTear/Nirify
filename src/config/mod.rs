pub mod category_section;
pub mod consolidation;
pub mod dirty;
pub mod error;
pub mod include;
pub mod loader;
pub mod models;
pub mod outputs_layout;
pub mod parser;
pub mod paths;
pub mod registry;
pub mod replace;
pub mod storage;
pub mod takeover;
pub mod validation;

pub use crate::types::CenterFocusedColumn;
pub use category_section::CategorySection;
pub use consolidation::{analyze_rules, ConsolidationAnalysis, ConsolidationSuggestion};
pub use dirty::{DirtyTracker, SettingsCategory};
pub use error::ConfigError;
pub use include::{
    expand_include_path, normalize_include_optional_syntax, open_include_for_import,
    open_include_for_scan, parse_include_node, parse_kdl_with_niri_includes, IncludeDirective,
    IncludeOpen,
};
pub use loader::{
    check_config_health, ensure_required_files_exist, import_from_kdl_str, import_from_niri_config,
    import_from_niri_config_with_result, load_settings, load_settings_with_result,
    repair_corrupted_configs, ConfigFileStatus, ConfigHealthReport, FileLoadStatus, ImportResult,
    LoadResult,
};
pub use models::{
    ColumnWidthType, LayoutOverride, OutputConfig, OutputHotCorners, OutputSettings, Settings,
    WorkspaceShadow,
};
pub use outputs_layout::{
    apply_live_outputs_to_settings, estimated_logical_size, find_config_index_for_live,
    find_live_output, logical_size_from_mode, output_name_matches_live, pack_to_the_right,
    parse_mode_resolution, seed_manual_position, LiveOutputsApplyResult,
};
pub use paths::ConfigPaths;
pub use registry::ConfigFile;
pub use replace::{
    analyze_config, is_managed_node, smart_replace_config, ConfigAnalysis, SmartReplaceResult,
};
pub use storage::{atomic_write, save_dirty, save_settings};
pub use takeover::{
    absorb_stripped_nodes, first_run_setup, merge_stripped_into_managed, AbsorbResult,
    FirstRunSetupResult,
};
pub use validation::{validate_string, validate_string_opt};
