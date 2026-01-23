//! Integration tests for smart config replacement
//!
//! Tests the smart replacement logic that analyzes config.kdl,
//! preserves unmanaged content, and replaces managed sections.

mod common;

use nirify::config::smart_replace_config;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_smart_replace_creates_config_when_missing() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Config doesn't exist yet
    assert!(!config_path.exists());

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should create minimal config with include
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("niri-settings/main.kdl"));

    // No backup needed for new file
    assert!(result.backup_path.as_os_str().is_empty());
    assert!(result.include_added);
    assert_eq!(result.replaced_count, 0);
    assert_eq!(result.preserved_count, 0);
}

#[test]
fn test_smart_replace_preserves_unmanaged_nodes() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Create config with both managed and unmanaged content
    let original = r#"
layout { gaps inner=16 }
custom-node { foo "bar" }
another-custom { baz 123 }
"#;
    fs::write(&config_path, original).unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should have replaced managed and preserved unmanaged
    assert_eq!(result.replaced_count, 1); // layout
    assert_eq!(result.preserved_count, 2); // custom-node, another-custom

    // Check the content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("niri-settings/main.kdl"));
    assert!(content.contains("custom-node"));
    assert!(content.contains("another-custom"));
    assert!(!content.contains("layout { gaps inner=16 }")); // removed
}

#[test]
fn test_smart_replace_creates_backup() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Create config with managed content
    fs::write(&config_path, "layout { gaps inner=20 }").unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should have created backup
    assert!(!result.backup_path.as_os_str().is_empty());
    assert!(result.backup_path.exists());

    // Backup should contain original content
    let backup_content = fs::read_to_string(&result.backup_path).unwrap();
    assert!(backup_content.contains("layout { gaps inner=20 }"));
}

#[test]
fn test_smart_replace_idempotent() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Create config with our include line already present, no managed nodes
    let content = r#"
include "niri-settings/main.kdl"
custom-node { foo "bar" }
"#;
    fs::write(&config_path, content).unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should detect it's already set up
    assert!(result.backup_path.as_os_str().is_empty()); // No backup needed
    assert!(!result.include_added);
    assert_eq!(result.replaced_count, 0);
    assert_eq!(result.preserved_count, 1); // just custom-node (niri-settings include is separate)

    // Content should be unchanged
    let new_content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(new_content, content);
}

#[test]
fn test_smart_replace_preserves_other_includes() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Config with another include
    let original = r#"
include "other-config.kdl"
layout { gaps inner=10 }
"#;
    fs::write(&config_path, original).unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Other include should be preserved
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("other-config.kdl"));
    assert!(content.contains("niri-settings/main.kdl"));

    assert_eq!(result.preserved_count, 1); // other include
    assert_eq!(result.replaced_count, 1); // layout
}

#[test]
fn test_smart_replace_handles_all_managed_nodes() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Config with multiple managed nodes
    let original = r#"
layout { gaps inner=16 }
input { keyboard { } }
animations { off }
cursor { size 32 }
output "eDP-1" { mode "1920x1080" }
workspace { name "main" }
window-rule { match app-id="test" }
"#;
    fs::write(&config_path, original).unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // All should be managed
    assert_eq!(result.replaced_count, 7);
    assert_eq!(result.preserved_count, 0);

    // Content should have our include and no managed nodes
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("niri-settings/main.kdl"));
    assert!(!content.contains("layout {"));
    assert!(!content.contains("input {"));
    assert!(!content.contains("animations {"));
}

#[test]
fn test_smart_replace_handles_corrupted_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Create corrupted config
    fs::write(&config_path, "this { is {{ invalid").unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should create backup of corrupted file
    assert!(!result.backup_path.as_os_str().is_empty());
    assert!(result.backup_path.exists());

    // Should have written minimal config
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("niri-settings/main.kdl"));

    // Warning should be present
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_smart_replace_removes_existing_niri_settings_include() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.kdl");
    let backup_dir = dir.path().join(".backup");

    // Config with our include AND managed nodes (shouldn't happen normally, but test it)
    let original = r#"
include "niri-settings/main.kdl"
layout { gaps inner=16 }
"#;
    fs::write(&config_path, original).unwrap();

    let result = smart_replace_config(&config_path, &backup_dir).unwrap();

    // Should have replaced the managed layout node
    assert_eq!(result.replaced_count, 1);

    // Only one include line in output
    let content = fs::read_to_string(&config_path).unwrap();
    let include_count = content.matches("niri-settings/main.kdl").count();
    assert_eq!(include_count, 1);
}
