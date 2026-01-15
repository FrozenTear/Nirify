//! Types for config diff functionality

use std::path::PathBuf;

/// Type of change for a diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Line is unchanged between versions
    Unchanged,
    /// Line was added in new version
    Added,
    /// Line was removed from old version
    Removed,
}

impl DiffLineType {
    /// Convert to integer for Slint (0=unchanged, 1=added, 2=removed)
    pub fn to_int(&self) -> i32 {
        match self {
            DiffLineType::Unchanged => 0,
            DiffLineType::Added => 1,
            DiffLineType::Removed => 2,
        }
    }
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Type of change
    pub line_type: DiffLineType,
    /// Content from old version (empty for added lines)
    pub old_text: String,
    /// Content from new version (empty for removed lines)
    pub new_text: String,
    /// Line number in the output
    pub line_num: i32,
}

/// Diff for a single category/file
#[derive(Debug, Clone)]
pub struct CategoryDiff {
    /// Category name (e.g., "Appearance", "Keyboard")
    pub name: String,
    /// Path to the KDL file
    pub file_path: PathBuf,
    /// Whether there are any changes
    pub has_changes: bool,
    /// Number of added lines
    pub additions: i32,
    /// Number of removed lines
    pub deletions: i32,
    /// The diff lines
    pub lines: Vec<DiffLine>,
}

impl CategoryDiff {
    /// Create a new empty diff for a category
    pub fn new(name: impl Into<String>, file_path: PathBuf) -> Self {
        Self {
            name: name.into(),
            file_path,
            has_changes: false,
            additions: 0,
            deletions: 0,
            lines: Vec::new(),
        }
    }

    /// Create a diff showing no changes
    pub fn no_changes(name: impl Into<String>, file_path: PathBuf) -> Self {
        Self::new(name, file_path)
    }
}

/// Complete diff across all changed categories
#[derive(Debug, Clone, Default)]
pub struct ConfigDiff {
    /// Diffs for each category that has changes
    pub categories: Vec<CategoryDiff>,
    /// Total additions across all categories
    pub total_additions: i32,
    /// Total deletions across all categories
    pub total_deletions: i32,
}

impl ConfigDiff {
    /// Create a new empty diff
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.categories.iter().any(|c| c.has_changes)
    }

    /// Add a category diff
    pub fn add_category(&mut self, diff: CategoryDiff) {
        self.total_additions += diff.additions;
        self.total_deletions += diff.deletions;
        self.categories.push(diff);
    }
}
