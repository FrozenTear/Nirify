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
}
