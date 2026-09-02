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

/// Validate a regex pattern, warning (never dropping) on syntax errors.
///
/// The loader assigns the returned value directly, so returning `None` would be
/// silent data loss. niri does the authoritative validation; here we only log a
/// warning when `regex_syntax` rejects the pattern. Strict blocking validation
/// lives in `validate_settings`.
pub fn validate_regex_pattern(pattern: &str, context: &str) -> Option<String> {
    if let Err(e) = validate_regex_strict(pattern) {
        warn!(
            "Regex pattern for {} may be invalid ({:?}): {}",
            context, pattern, e
        );
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

    // Strict regex validation gates saving; skip disabled rules (slashdashed on
    // disk, never parsed by niri) so a bad regex in one can't block every save (P3).
    if rule.enabled {
        for (idx, m) in rule.matches.iter().enumerate() {
            let match_prefix = format!("{}.match[{}]", rule_name, idx);

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
        // Excludes are emitted to niri too, so a bad exclude regex is equally fatal.
        for (idx, e) in rule.excludes.iter().enumerate() {
            let exclude_prefix = format!("{}.exclude[{}]", rule_name, idx);

            if let Some(ref pattern) = e.app_id {
                if let Err(err) = validate_regex_strict(pattern) {
                    result.add_error("WindowRules", &format!("{}.app_id", exclude_prefix), &err);
                }
            }
            if let Some(ref pattern) = e.title {
                if let Err(err) = validate_regex_strict(pattern) {
                    result.add_error("WindowRules", &format!("{}.title", exclude_prefix), &err);
                }
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

    // Skip disabled rules from the save-gating strict regex check (P3).
    if rule.enabled {
        for (idx, m) in rule.matches.iter().enumerate() {
            let match_prefix = format!("{}.match[{}]", rule_name, idx);

            if let Some(ref pattern) = m.namespace {
                if let Err(e) = validate_regex_strict(pattern) {
                    result.add_error("LayerRules", &format!("{}.namespace", match_prefix), &e);
                }
            }
        }
        // Excludes are emitted to niri too; validate their regexes as well.
        for (idx, e) in rule.excludes.iter().enumerate() {
            let exclude_prefix = format!("{}.exclude[{}]", rule_name, idx);

            if let Some(ref pattern) = e.namespace {
                if let Err(err) = validate_regex_strict(pattern) {
                    result.add_error("LayerRules", &format!("{}.namespace", exclude_prefix), &err);
                }
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

    // Empty connector names must never be written (`output "" { ... }`).
    // Storage skips them; warn so the Displays UI can prompt for a name.
    for (idx, output) in settings.outputs.outputs.iter().enumerate() {
        if output.name.trim().is_empty() {
            result.add_warning(
                "Outputs",
                &format!("output[{}].name", idx),
                "Empty connector name; this output will not be written to config",
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
            log::info!(
                "Settings validation found {} warnings",
                result.warnings.len()
            );
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
    fn validate_regex_pattern_never_drops_valid_lone_brace() {
        // Rust regex accepts a lone `}` as a literal; the pattern must survive.
        assert_eq!(
            validate_regex_pattern("foo}bar", "test"),
            Some("foo}bar".to_string())
        );
        assert_eq!(
            validate_regex_pattern("^\\{", "test"),
            Some("^\\{".to_string())
        );
        // Even a genuinely invalid pattern is preserved (warned, not dropped).
        assert_eq!(
            validate_regex_pattern("(unclosed", "test"),
            Some("(unclosed".to_string())
        );
    }

    #[test]
    fn test_validate_empty_settings() {
        let settings = Settings::default();
        let result = validate_settings(&settings);
        assert!(result.is_valid());
    }

    #[test]
    fn empty_output_name_is_a_warning_not_an_error() {
        use crate::config::models::OutputConfig;

        let mut settings = Settings::default();
        settings.outputs.outputs.push(OutputConfig::default());
        let result = validate_settings(&settings);
        assert!(
            result.is_valid(),
            "empty connector names must not block other saves"
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.category == "Outputs" && w.field == "output[0].name"),
            "expected Outputs warning, got {:?}",
            result.warnings
        );
    }

    #[test]
    fn enabled_rule_with_bad_exclude_regex_fails_but_disabled_passes() {
        use crate::config::models::{LayerRuleMatch, WindowRuleMatch};

        let mut wr = WindowRule {
            enabled: true,
            ..Default::default()
        };
        wr.excludes.push(WindowRuleMatch {
            app_id: Some("[invalid(regex".to_string()),
            ..Default::default()
        });
        let mut r = ValidationResult::default();
        validate_window_rule(&wr, &mut r);
        assert!(
            !r.errors.is_empty(),
            "enabled rule with bad exclude regex must gate the save"
        );

        wr.enabled = false;
        let mut r2 = ValidationResult::default();
        validate_window_rule(&wr, &mut r2);
        assert!(
            r2.errors.is_empty(),
            "disabled rule is slashdashed on disk and must not gate the save"
        );

        // Same behaviour for layer rules.
        let mut lr = LayerRule {
            enabled: true,
            ..Default::default()
        };
        lr.excludes.push(LayerRuleMatch {
            namespace: Some("(unclosed".to_string()),
            ..Default::default()
        });
        let mut r3 = ValidationResult::default();
        validate_layer_rule(&lr, &mut r3);
        assert!(!r3.errors.is_empty(), "bad layer exclude regex must error");

        lr.enabled = false;
        let mut r4 = ValidationResult::default();
        validate_layer_rule(&lr, &mut r4);
        assert!(r4.errors.is_empty(), "disabled layer rule must not gate");
    }
}
