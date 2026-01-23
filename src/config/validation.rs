//! String and collection validation helpers
//!
//! These functions help enforce limits on user input to prevent memory issues
//! and ensure reasonable bounds on collections.

use crate::constants::MAX_STRING_LENGTH;
use log::warn;

/// Truncate a string to MAX_STRING_LENGTH if needed.
///
/// This prevents memory issues from excessively long user input.
/// If the string is truncated, the truncation happens at a character boundary.
/// Logs a warning when truncation occurs.
pub fn validate_string(s: &str) -> String {
    if s.len() > MAX_STRING_LENGTH {
        warn!(
            "String truncated from {} to {} characters",
            s.len(),
            MAX_STRING_LENGTH
        );
        s.chars().take(MAX_STRING_LENGTH).collect()
    } else {
        s.to_string()
    }
}

/// Validate a regex pattern for obvious errors
///
/// Performs basic validation without requiring the regex crate:
/// - Checks for balanced parentheses, brackets, and braces
/// - Detects unterminated escape sequences
/// - Warns about patterns that may cause issues
///
/// Returns the pattern as-is (niri will do final validation), but logs warnings
/// for problematic patterns that could crash niri at runtime.
pub fn validate_regex_pattern(pattern: &str, context: &str) -> Option<String> {
    if pattern.is_empty() {
        return Some(pattern.to_string());
    }

    // Check for balanced brackets/parens
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut in_escape = false;
    let mut in_char_class = false;

    for c in pattern.chars() {
        if in_escape {
            in_escape = false;
            continue;
        }

        match c {
            '\\' => in_escape = true,
            '[' if !in_char_class => {
                in_char_class = true;
                bracket_depth += 1;
            }
            ']' if in_char_class => {
                in_char_class = false;
                bracket_depth -= 1;
            }
            '(' if !in_char_class => paren_depth += 1,
            ')' if !in_char_class => paren_depth -= 1,
            '{' if !in_char_class => brace_depth += 1,
            '}' if !in_char_class => brace_depth -= 1,
            _ => {}
        }

        // Negative depth means closing without opening
        if paren_depth < 0 || bracket_depth < 0 || brace_depth < 0 {
            warn!(
                "Invalid regex pattern for {}: unmatched closing bracket in {:?}",
                context, pattern
            );
            return None;
        }
    }

    // Check for unterminated escape at end
    if in_escape {
        warn!(
            "Invalid regex pattern for {}: unterminated escape sequence in {:?}",
            context, pattern
        );
        return None;
    }

    // Check for unclosed groups
    if paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
        warn!(
            "Invalid regex pattern for {}: unbalanced brackets in {:?} (parens={}, brackets={}, braces={})",
            context, pattern, paren_depth, bracket_depth, brace_depth
        );
        return None;
    }

    Some(pattern.to_string())
}

/// Truncate a string option to MAX_STRING_LENGTH if needed.
///
/// Returns None if the input is None, otherwise validates the string.
pub fn validate_string_opt(s: Option<&str>) -> Option<String> {
    s.map(validate_string)
}

// Pre-save validation for settings
// Validates settings before writing to disk to prevent invalid configs.

use super::models::{LayerRule, Settings, WindowRule};

/// Validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub category: String,
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}: {}", self.category, self.field, self.message)
    }
}

/// Result of validation
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, category: &str, field: &str, message: &str) {
        self.errors.push(ValidationError {
            category: category.to_string(),
            field: field.to_string(),
            message: message.to_string(),
        });
    }

    pub fn add_warning(&mut self, category: &str, field: &str, message: &str) {
        self.warnings.push(ValidationError {
            category: category.to_string(),
            field: field.to_string(),
            message: message.to_string(),
        });
    }
}

/// Validate a regex pattern using regex_syntax for robust parsing
fn validate_regex_strict(pattern: &str) -> Result<(), String> {
    if pattern.is_empty() {
        return Ok(());
    }
    regex_syntax::Parser::new()
        .parse(pattern)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Validate a window rule
fn validate_window_rule(rule: &WindowRule, result: &mut ValidationResult) {
    let rule_name = format!("WindowRule[{}]", rule.id);

    for (idx, m) in rule.matches.iter().enumerate() {
        let match_prefix = format!("{}.match[{}]", rule_name, idx);

        // Validate regex patterns with strict parser
        if let Some(ref pattern) = m.app_id {
            if let Err(e) = validate_regex_strict(pattern) {
                result.add_error("WindowRules", &format!("{}.app_id", match_prefix), &e);
            }
        }
        if let Some(ref pattern) = m.title {
            if let Err(e) = validate_regex_strict(pattern) {
                result.add_error("WindowRules", &format!("{}.title", match_prefix), &e);
            }
        }
    }

    // Validate opacity range
    if let Some(opacity) = rule.opacity {
        if !(0.0..=1.0).contains(&opacity) {
            result.add_warning(
                "WindowRules",
                &format!("{}.opacity", rule_name),
                &format!("Opacity {} is outside valid range [0.0, 1.0]", opacity),
            );
        }
    }
}

/// Validate a layer rule
fn validate_layer_rule(rule: &LayerRule, result: &mut ValidationResult) {
    let rule_name = format!("LayerRule[{}]", rule.id);

    for (idx, m) in rule.matches.iter().enumerate() {
        let match_prefix = format!("{}.match[{}]", rule_name, idx);

        // Validate namespace regex with strict parser
        if let Some(ref pattern) = m.namespace {
            if let Err(e) = validate_regex_strict(pattern) {
                result.add_error("LayerRules", &format!("{}.namespace", match_prefix), &e);
            }
        }
    }

    // Validate opacity range
    if let Some(opacity) = rule.opacity {
        if !(0.0..=1.0).contains(&opacity) {
            result.add_warning(
                "LayerRules",
                &format!("{}.opacity", rule_name),
                &format!("Opacity {} is outside valid range [0.0, 1.0]", opacity),
            );
        }
    }
}

/// Validate all settings before saving
///
/// Returns validation result with errors (which should block save) and warnings.
pub fn validate_settings(settings: &Settings) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Validate window rules
    for rule in &settings.window_rules.rules {
        validate_window_rule(rule, &mut result);
    }

    // Validate layer rules
    for rule in &settings.layer_rules.rules {
        validate_layer_rule(rule, &mut result);
    }

    // Validate keybindings - check for empty key combos
    for (idx, binding) in settings.keybindings.bindings.iter().enumerate() {
        if binding.key_combo.trim().is_empty() {
            result.add_warning(
                "Keybindings",
                &format!("binding[{}].key_combo", idx),
                "Empty key combo",
            );
        }
    }

    // Log validation results
    if result.errors.is_empty() && result.warnings.is_empty() {
        log::debug!("Settings validation passed");
    } else {
        if !result.errors.is_empty() {
            log::warn!("Settings validation found {} errors", result.errors.len());
            for err in &result.errors {
                log::warn!("  Validation error: {}", err);
            }
        }
        if !result.warnings.is_empty() {
            log::info!("Settings validation found {} warnings", result.warnings.len());
            for warn in &result.warnings {
                log::info!("  Validation warning: {}", warn);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_short() {
        let input = "hello";
        assert_eq!(validate_string(input), "hello");
    }

    #[test]
    fn test_validate_string_exactly_max() {
        let input: String = "a".repeat(MAX_STRING_LENGTH);
        assert_eq!(validate_string(&input), input);
    }

    #[test]
    fn test_validate_string_over_max() {
        let input: String = "a".repeat(MAX_STRING_LENGTH + 100);
        let result = validate_string(&input);
        assert_eq!(result.len(), MAX_STRING_LENGTH);
    }

    #[test]
    fn test_validate_string_opt_none() {
        assert_eq!(validate_string_opt(None), None);
    }

    #[test]
    fn test_validate_string_opt_some() {
        assert_eq!(
            validate_string_opt(Some("hello")),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_validate_regex_strict_valid() {
        assert!(validate_regex_strict("^foo$").is_ok());
        assert!(validate_regex_strict(".*bar.*").is_ok());
        assert!(validate_regex_strict("").is_ok());
        assert!(validate_regex_strict(r"\d+").is_ok());
    }

    #[test]
    fn test_validate_regex_strict_invalid() {
        assert!(validate_regex_strict("[unclosed").is_err());
        assert!(validate_regex_strict("(unclosed").is_err());
        assert!(validate_regex_strict("*invalid").is_err());
        assert!(validate_regex_strict(r"\").is_err());
    }

    #[test]
    fn test_validate_empty_settings() {
        let settings = Settings::default();
        let result = validate_settings(&settings);
        assert!(result.is_valid());
    }
}
