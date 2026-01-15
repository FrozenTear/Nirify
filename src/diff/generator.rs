//! Diff generation for config changes

use super::types::{CategoryDiff, ConfigDiff, DiffLine, DiffLineType};
use crate::config::paths::ConfigPaths;
use crate::config::storage::*;
use crate::config::{Settings, SettingsCategory};
use std::collections::HashSet;
use std::fs;

/// Generate a diff between current settings and what's on disk
pub fn generate_diff(
    settings: &Settings,
    paths: &ConfigPaths,
    dirty_categories: &HashSet<SettingsCategory>,
) -> ConfigDiff {
    let mut diff = ConfigDiff::new();

    for category in dirty_categories {
        if let Some(cat_diff) = generate_category_diff(settings, paths, *category) {
            if cat_diff.has_changes {
                diff.add_category(cat_diff);
            }
        }
    }

    diff
}

/// Generate diff for a single category
fn generate_category_diff(
    settings: &Settings,
    paths: &ConfigPaths,
    category: SettingsCategory,
) -> Option<CategoryDiff> {
    let (name, file_path, new_content) = match category {
        SettingsCategory::Appearance => (
            "Appearance",
            paths.appearance_kdl.clone(),
            generate_appearance_kdl(&settings.appearance, &settings.behavior),
        ),
        SettingsCategory::Behavior => (
            "Behavior",
            paths.behavior_kdl.clone(),
            generate_behavior_kdl(&settings.behavior),
        ),
        SettingsCategory::Keyboard => (
            "Keyboard",
            paths.keyboard_kdl.clone(),
            generate_keyboard_kdl(&settings.keyboard),
        ),
        SettingsCategory::Mouse => (
            "Mouse",
            paths.mouse_kdl.clone(),
            generate_mouse_kdl(&settings.mouse),
        ),
        SettingsCategory::Touchpad => (
            "Touchpad",
            paths.touchpad_kdl.clone(),
            generate_touchpad_kdl(&settings.touchpad),
        ),
        SettingsCategory::Trackpoint => (
            "Trackpoint",
            paths.trackpoint_kdl.clone(),
            generate_trackpoint_kdl(&settings.trackpoint),
        ),
        SettingsCategory::Trackball => (
            "Trackball",
            paths.trackball_kdl.clone(),
            generate_trackball_kdl(&settings.trackball),
        ),
        SettingsCategory::Tablet => (
            "Tablet",
            paths.tablet_kdl.clone(),
            generate_tablet_kdl(&settings.tablet),
        ),
        SettingsCategory::Touch => (
            "Touch",
            paths.touch_kdl.clone(),
            generate_touch_kdl(&settings.touch),
        ),
        SettingsCategory::Cursor => (
            "Cursor",
            paths.cursor_kdl.clone(),
            generate_cursor_kdl(&settings.cursor),
        ),
        SettingsCategory::Animations => (
            "Animations",
            paths.animations_kdl.clone(),
            generate_animations_kdl(&settings.animations),
        ),
        SettingsCategory::Overview => (
            "Overview",
            paths.overview_kdl.clone(),
            generate_overview_kdl(&settings.overview),
        ),
        SettingsCategory::Gestures => (
            "Gestures",
            paths.gestures_kdl.clone(),
            generate_gestures_kdl(&settings.gestures),
        ),
        SettingsCategory::Outputs => (
            "Outputs",
            paths.outputs_kdl.clone(),
            generate_outputs_kdl(&settings.outputs),
        ),
        SettingsCategory::Workspaces => (
            "Workspaces",
            paths.workspaces_kdl.clone(),
            generate_workspaces_kdl(&settings.workspaces),
        ),
        SettingsCategory::LayoutExtras => (
            "Layout Extras",
            paths.layout_extras_kdl.clone(),
            generate_layout_extras_kdl(&settings.layout_extras),
        ),
        SettingsCategory::WindowRules => (
            "Window Rules",
            paths.window_rules_kdl.clone(),
            generate_window_rules_kdl(&settings.window_rules),
        ),
        SettingsCategory::LayerRules => (
            "Layer Rules",
            paths.layer_rules_kdl.clone(),
            generate_layer_rules_kdl(&settings.layer_rules),
        ),
        SettingsCategory::Keybindings => (
            "Keybindings",
            paths.keybindings_kdl.clone(),
            generate_keybindings_kdl(&settings.keybindings),
        ),
        SettingsCategory::SwitchEvents => (
            "Switch Events",
            paths.switch_events_kdl.clone(),
            generate_switch_events_kdl(&settings.switch_events),
        ),
        SettingsCategory::Startup => (
            "Startup",
            paths.startup_kdl.clone(),
            generate_startup_kdl(&settings.startup),
        ),
        SettingsCategory::Environment => (
            "Environment",
            paths.environment_kdl.clone(),
            generate_environment_kdl(&settings.environment),
        ),
        SettingsCategory::Debug => (
            "Debug",
            paths.debug_kdl.clone(),
            generate_debug_kdl(&settings.debug),
        ),
        SettingsCategory::Miscellaneous => (
            "Miscellaneous",
            paths.misc_kdl.clone(),
            generate_misc_kdl(&settings.miscellaneous),
        ),
        SettingsCategory::RecentWindows => (
            "Recent Windows",
            paths.recent_windows_kdl.clone(),
            generate_recent_windows_kdl(&settings.recent_windows),
        ),
    };

    // Read old content from disk
    let old_content = fs::read_to_string(&file_path).unwrap_or_default();

    // Generate line diff
    Some(compute_line_diff(
        name,
        file_path,
        &old_content,
        &new_content,
    ))
}

/// Compute a line-by-line diff between old and new content
fn compute_line_diff(
    name: &str,
    file_path: std::path::PathBuf,
    old_content: &str,
    new_content: &str,
) -> CategoryDiff {
    let mut diff = CategoryDiff::new(name, file_path);

    // Quick check: if content is identical, no changes
    if old_content == new_content {
        return diff;
    }

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    // Simple line-by-line diff algorithm
    // Uses longest common subsequence approach
    let lcs = compute_lcs(&old_lines, &new_lines);

    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut line_num = 1;

    for &(old_match, new_match) in &lcs {
        // Handle removed lines (in old but not in LCS before this match)
        while old_idx < old_match {
            diff.lines.push(DiffLine {
                line_type: DiffLineType::Removed,
                old_text: old_lines[old_idx].to_string(),
                new_text: String::new(),
                line_num,
            });
            diff.deletions += 1;
            old_idx += 1;
            line_num += 1;
        }

        // Handle added lines (in new but not in LCS before this match)
        while new_idx < new_match {
            diff.lines.push(DiffLine {
                line_type: DiffLineType::Added,
                old_text: String::new(),
                new_text: new_lines[new_idx].to_string(),
                line_num,
            });
            diff.additions += 1;
            new_idx += 1;
            line_num += 1;
        }

        // Handle matching line
        diff.lines.push(DiffLine {
            line_type: DiffLineType::Unchanged,
            old_text: old_lines[old_idx].to_string(),
            new_text: new_lines[new_idx].to_string(),
            line_num,
        });
        old_idx += 1;
        new_idx += 1;
        line_num += 1;
    }

    // Handle remaining removed lines
    while old_idx < old_lines.len() {
        diff.lines.push(DiffLine {
            line_type: DiffLineType::Removed,
            old_text: old_lines[old_idx].to_string(),
            new_text: String::new(),
            line_num,
        });
        diff.deletions += 1;
        old_idx += 1;
        line_num += 1;
    }

    // Handle remaining added lines
    while new_idx < new_lines.len() {
        diff.lines.push(DiffLine {
            line_type: DiffLineType::Added,
            old_text: String::new(),
            new_text: new_lines[new_idx].to_string(),
            line_num,
        });
        diff.additions += 1;
        new_idx += 1;
        line_num += 1;
    }

    diff.has_changes = diff.additions > 0 || diff.deletions > 0;
    diff
}

/// Compute longest common subsequence indices
fn compute_lcs(old: &[&str], new: &[&str]) -> Vec<(usize, usize)> {
    let m = old.len();
    let n = new.len();

    if m == 0 || n == 0 {
        return Vec::new();
    }

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find matching indices
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            result.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_line_diff_no_changes() {
        let content = "line1\nline2\nline3";
        let diff = compute_line_diff("test", "test.kdl".into(), content, content);
        assert!(!diff.has_changes);
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn test_compute_line_diff_additions() {
        let old = "line1\nline3";
        let new = "line1\nline2\nline3";
        let diff = compute_line_diff("test", "test.kdl".into(), old, new);
        assert!(diff.has_changes);
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn test_compute_line_diff_deletions() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline3";
        let diff = compute_line_diff("test", "test.kdl".into(), old, new);
        assert!(diff.has_changes);
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 1);
    }

    #[test]
    fn test_compute_line_diff_modifications() {
        let old = "line1\nold_line\nline3";
        let new = "line1\nnew_line\nline3";
        let diff = compute_line_diff("test", "test.kdl".into(), old, new);
        assert!(diff.has_changes);
        // A modification shows as 1 deletion + 1 addition
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }
}
