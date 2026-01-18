//! Common utilities for rule editor callbacks (window rules and layer rules)
//!
//! Provides shared functionality for add/remove/select operations on rules.

use slint::{ModelRc, SharedString, VecModel};
use std::cell::Cell;
use std::rc::Rc;

/// Trait for rules that have a name field
pub trait Named {
    fn name(&self) -> &str;
}

/// Build a list model from rule names for display in the UI
///
/// This is used to populate rule list views in both window rules and layer rules.
pub fn build_names_list<T: Named>(items: &[T]) -> ModelRc<SharedString> {
    let names: Vec<SharedString> = items.iter().map(|item| item.name().into()).collect();
    ModelRc::new(VecModel::from(names))
}

/// Helper to get selected index from Cell
///
/// Rc<Cell<i32>> is used since Slint callbacks are single-threaded.
pub fn get_selected_index(idx_cell: &Rc<Cell<i32>>) -> i32 {
    idx_cell.get()
}

/// Helper to set selected index in Cell
pub fn set_selected_index(idx_cell: &Rc<Cell<i32>>, value: i32) {
    idx_cell.set(value);
}

/// Calculate new selection index after removing an item
///
/// When an item is removed from a list:
/// - If the list becomes empty, return -1 (no selection)
/// - If the removed item was at or after the new end, select the last item
/// - Otherwise, keep the same index (which now points to the next item)
pub fn calculate_new_selection_after_remove(removed_idx: usize, total_after: usize) -> i32 {
    if total_after == 0 {
        -1
    } else if removed_idx >= total_after {
        (total_after - 1) as i32
    } else {
        removed_idx as i32
    }
}

/// Calculate new selection for match criteria after removal
///
/// Similar to `calculate_new_selection_after_remove` but matches cannot be
/// empty (there's always at least one), so -1 is never returned.
pub fn calculate_new_match_selection_after_remove(removed_idx: usize, total_after: usize) -> i32 {
    if removed_idx >= total_after {
        (total_after - 1) as i32
    } else {
        removed_idx as i32
    }
}

/// Reset match index to 0 (used when selecting a new rule or adding a rule)
pub fn reset_match_index(match_idx_cell: &Rc<Cell<i32>>) {
    set_selected_index(match_idx_cell, 0);
}

/// Parse optional string field from UI input
///
/// Returns None if the string is empty, Some(value) otherwise.
/// Used for app_id, title, namespace, and similar optional fields.
pub fn parse_optional_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Validate that a rule index is within bounds
pub fn is_valid_rule_index(index: i32, rules_len: usize) -> bool {
    index >= 0 && (index as usize) < rules_len
}

/// Validate that a match index is within bounds
pub fn is_valid_match_index(index: i32, matches_len: usize) -> bool {
    index >= 0 && (index as usize) < matches_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_new_selection_after_remove() {
        // Empty list after removal
        assert_eq!(calculate_new_selection_after_remove(0, 0), -1);

        // Removed last item, select new last
        assert_eq!(calculate_new_selection_after_remove(2, 2), 1);

        // Removed item in middle, keep same index
        assert_eq!(calculate_new_selection_after_remove(1, 3), 1);

        // Removed first item, keep same index (now points to what was second)
        assert_eq!(calculate_new_selection_after_remove(0, 2), 0);
    }

    #[test]
    fn test_calculate_new_match_selection_after_remove() {
        // Removed last match, select new last
        assert_eq!(calculate_new_match_selection_after_remove(2, 2), 1);

        // Removed match in middle, keep same index
        assert_eq!(calculate_new_match_selection_after_remove(1, 3), 1);
    }

    #[test]
    fn test_parse_optional_string() {
        assert_eq!(parse_optional_string(""), None);
        assert_eq!(parse_optional_string("test"), Some("test".to_string()));
    }

    #[test]
    fn test_is_valid_rule_index() {
        assert!(!is_valid_rule_index(-1, 5));
        assert!(is_valid_rule_index(0, 5));
        assert!(is_valid_rule_index(4, 5));
        assert!(!is_valid_rule_index(5, 5));
    }

    #[test]
    fn test_is_valid_match_index() {
        assert!(!is_valid_match_index(-1, 3));
        assert!(is_valid_match_index(0, 3));
        assert!(is_valid_match_index(2, 3));
        assert!(!is_valid_match_index(3, 3));
    }

    #[test]
    fn test_get_set_selected_index() {
        let idx = Rc::new(Cell::new(5i32));
        assert_eq!(get_selected_index(&idx), 5);

        set_selected_index(&idx, 10);
        assert_eq!(get_selected_index(&idx), 10);

        set_selected_index(&idx, -1);
        assert_eq!(get_selected_index(&idx), -1);
    }
}
