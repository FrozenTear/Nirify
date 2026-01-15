//! KDL Builder for generating niri configuration files
//!
//! This module provides a builder API for generating well-formatted KDL output,
//! handling indentation, nesting, and common patterns automatically.
//!
//! # Examples
//!
//! ```ignore
//! use crate::config::storage::builder::KdlBuilder;
//!
//! let mut kdl = KdlBuilder::new();
//! kdl.comment("Focus ring settings");
//! kdl.block("focus-ring", |b| {
//!     b.field("width", 4);
//!     b.field_string("active-color", "#ff0000");
//!     b.optional_flag("off", false); // Only outputs if true
//! });
//!
//! let output = kdl.build();
//! // Outputs:
//! // // Focus ring settings
//! // focus-ring {
//! //     width 4
//! //     active-color "#ff0000"
//! // }
//! ```

use super::gradient::color_or_gradient_to_kdl;
use super::helpers::escape_kdl_string;
use crate::types::{Color, ColorOrGradient};

/// Initial capacity for KDL content buffer.
/// Most config sections are 200-800 bytes, 1KB provides headroom without waste.
const INITIAL_KDL_CAPACITY: usize = 1024;

/// Builder for generating KDL configuration content
///
/// Handles indentation, nesting, and common field patterns.
#[derive(Default)]
pub struct KdlBuilder {
    content: String,
    indent_level: usize,
    /// Whether to skip empty blocks (blocks with no content)
    skip_empty_blocks: bool,
}

impl KdlBuilder {
    /// Create a new KDL builder
    ///
    /// Pre-allocates buffer for the content string. This is a reasonable default
    /// for typical config sections (most are 200-800 bytes). The String will
    /// grow automatically if needed.
    pub fn new() -> Self {
        Self {
            content: String::with_capacity(INITIAL_KDL_CAPACITY),
            indent_level: 0,
            skip_empty_blocks: true,
        }
    }

    /// Create a new builder with a file header comment
    pub fn with_header(comment: &str) -> Self {
        let mut builder = Self::new();
        builder.comment(comment);
        builder.newline();
        builder
    }

    /// Get the current indentation string
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Add a comment line
    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str("// ");
        self.content.push_str(text);
        self.content.push('\n');
        self
    }

    /// Add an empty line
    pub fn newline(&mut self) -> &mut Self {
        self.content.push('\n');
        self
    }

    /// Add a raw line (with current indentation)
    pub fn raw(&mut self, line: &str) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(line);
        self.content.push('\n');
        self
    }

    /// Add a flag (presence-based boolean, only if true)
    pub fn flag(&mut self, name: &str) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push('\n');
        self
    }

    /// Add a flag only if the condition is true
    pub fn optional_flag(&mut self, name: &str, condition: bool) -> &mut Self {
        if condition {
            self.flag(name);
        }
        self
    }

    /// Add an integer field
    pub fn field_i32(&mut self, name: &str, value: i32) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push(' ');
        self.content.push_str(&value.to_string());
        self.content.push('\n');
        self
    }

    /// Add an integer field only if it differs from default
    pub fn field_i32_if_not(&mut self, name: &str, value: i32, default: i32) -> &mut Self {
        if value != default {
            self.field_i32(name, value);
        }
        self
    }

    /// Add a float field (with sufficient precision to round-trip)
    pub fn field_f32(&mut self, name: &str, value: f32) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push(' ');
        // Use integer if it's a whole number within i32 range, otherwise float format
        // Check is_finite() to handle NaN and Infinity safely
        if value.is_finite()
            && value.fract() == 0.0
            && value >= i32::MIN as f32
            && value <= i32::MAX as f32
        {
            self.content.push_str(&(value as i32).to_string());
        } else {
            // Use 6 decimal places (enough for f32 precision), trim trailing zeros
            let formatted = format!("{:.6}", value);
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            self.content.push_str(trimmed);
        }
        self.content.push('\n');
        self
    }

    /// Add a float field only if it differs from default
    pub fn field_f32_if_not(&mut self, name: &str, value: f32, default: f32) -> &mut Self {
        if (value - default).abs() > f32::EPSILON {
            self.field_f32(name, value);
        }
        self
    }

    /// Add a float field formatted as integer (rounds value)
    ///
    /// Clamps to i32 range if value is out of bounds. Falls back to float format
    /// for NaN or Infinity.
    pub fn field_f32_as_int(&mut self, name: &str, value: f32) -> &mut Self {
        if value.is_finite() {
            let rounded = value.round();
            let clamped = rounded.clamp(i32::MIN as f32, i32::MAX as f32) as i32;
            self.field_i32(name, clamped)
        } else {
            // NaN/Infinity: fall back to float format
            self.field_f32(name, value)
        }
    }

    /// Add a string field (quoted, with proper escaping)
    pub fn field_string(&mut self, name: &str, value: &str) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push_str(" \"");
        self.content.push_str(&escape_kdl_string(value));
        self.content.push_str("\"\n");
        self
    }

    /// Add a string field only if not empty
    pub fn field_string_if_not_empty(&mut self, name: &str, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.field_string(name, value);
        }
        self
    }

    /// Add a string field only if it differs from default
    pub fn field_string_if_not(&mut self, name: &str, value: &str, default: &str) -> &mut Self {
        if value != default {
            self.field_string(name, value);
        }
        self
    }

    /// Add a color field (as hex string)
    pub fn field_color(&mut self, name: &str, color: &Color) -> &mut Self {
        self.field_string(name, &color.to_hex())
    }

    /// Add a color-or-gradient field using niri's format
    ///
    /// For colors, outputs: `{name}-color "#rrggbb"`
    /// For gradients, outputs: `{name}-gradient from="#..." to="#..." ...`
    ///
    /// Uses the existing `color_or_gradient_to_kdl` helper for correct formatting.
    pub fn field_color_or_gradient(&mut self, name: &str, value: &ColorOrGradient) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content
            .push_str(&color_or_gradient_to_kdl(value, name));
        self.content.push('\n');
        self
    }

    /// Add an attribute (name=value format, with proper escaping)
    pub fn attr_string(&mut self, name: &str, value: &str) -> &mut Self {
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push_str("=\"");
        self.content.push_str(&escape_kdl_string(value));
        self.content.push_str("\"\n");
        self
    }

    /// Format an inline attribute (name="value" format, with proper escaping)
    ///
    /// This is a utility function for building attribute strings, not a builder method.
    pub fn format_inline_attr(name: &str, value: &str) -> String {
        format!("{}=\"{}\"", name, escape_kdl_string(value))
    }

    /// Add a block with content generated by a closure
    ///
    /// The closure receives a mutable reference to the builder with
    /// increased indentation.
    ///
    /// This method writes directly to the parent buffer and truncates if the
    /// block is empty, avoiding allocation of a temporary inner builder.
    pub fn block<F>(&mut self, name: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut KdlBuilder),
    {
        let start_pos = self.content.len();

        // Write opening
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push_str(" {\n");

        let after_opening_pos = self.content.len();

        // Write content with increased indentation
        self.indent_level += 1;
        f(self);
        self.indent_level -= 1;

        // Check if any content was added
        if self.content.len() == after_opening_pos && self.skip_empty_blocks {
            // No content was added, truncate the opening
            self.content.truncate(start_pos);
        } else {
            // Write closing brace
            self.content.push_str(&self.indent());
            self.content.push_str("}\n");
        }
        self
    }

    /// Add a block only if a condition is true
    pub fn block_if<F>(&mut self, name: &str, condition: bool, f: F) -> &mut Self
    where
        F: FnOnce(&mut KdlBuilder),
    {
        if condition {
            self.block(name, f);
        }
        self
    }

    /// Add a block with inline attributes
    ///
    /// Example: `node key="value" { ... }`
    ///
    /// Writes directly to parent buffer, truncates if empty.
    pub fn block_with_attrs<F>(&mut self, name: &str, attrs: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut KdlBuilder),
    {
        let start_pos = self.content.len();

        // Write opening with attributes
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        if !attrs.is_empty() {
            self.content.push(' ');
            self.content.push_str(attrs);
        }
        self.content.push_str(" {\n");

        let after_opening_pos = self.content.len();

        // Write content with increased indentation
        self.indent_level += 1;
        f(self);
        self.indent_level -= 1;

        // Check if any content was added
        if self.content.len() == after_opening_pos && self.skip_empty_blocks {
            self.content.truncate(start_pos);
        } else {
            self.content.push_str(&self.indent());
            self.content.push_str("}\n");
        }
        self
    }

    /// Add a node with a single positional argument and children
    ///
    /// Example: `window-rule "app-id" { ... }`
    ///
    /// Always outputs the block (does not skip if empty).
    /// Writes directly to parent buffer without allocating a temporary builder.
    pub fn node_with_arg<F>(&mut self, name: &str, arg: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut KdlBuilder),
    {
        // Write opening
        self.content.push_str(&self.indent());
        self.content.push_str(name);
        self.content.push_str(" \"");
        self.content.push_str(&escape_kdl_string(arg));
        self.content.push_str("\" {\n");

        // Write content with increased indentation
        self.indent_level += 1;
        f(self);
        self.indent_level -= 1;

        // Write closing
        self.content.push_str(&self.indent());
        self.content.push_str("}\n");
        self
    }

    /// Consume the builder and return the generated KDL string
    pub fn build(self) -> String {
        self.content
    }

    /// Get the current content without consuming the builder
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Check if the builder has any content
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Append another builder's content
    pub fn append(&mut self, other: &KdlBuilder) -> &mut Self {
        self.content.push_str(&other.content);
        self
    }

    /// Clear the content
    pub fn clear(&mut self) -> &mut Self {
        self.content.clear();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fields() {
        let mut kdl = KdlBuilder::new();
        kdl.field_i32("width", 42);
        kdl.field_f32("scale", 1.5);
        kdl.field_string("name", "hello");

        let output = kdl.build();
        assert!(output.contains("width 42"));
        assert!(output.contains("scale 1.5")); // Trailing zeros trimmed
        assert!(output.contains("name \"hello\""));
    }

    #[test]
    fn test_float_precision() {
        let mut kdl = KdlBuilder::new();
        kdl.field_f32("scale", 1.333333);
        kdl.field_f32("epsilon", 0.0001);
        kdl.field_f32("whole", 2.0);

        let output = kdl.build();
        assert!(output.contains("scale 1.333333")); // Full precision preserved
        assert!(output.contains("epsilon 0.0001")); // Small values preserved
        assert!(output.contains("whole 2")); // Whole numbers as integers
    }

    #[test]
    fn test_flags() {
        let mut kdl = KdlBuilder::new();
        kdl.flag("enabled");
        kdl.optional_flag("disabled", false);
        kdl.optional_flag("active", true);

        let output = kdl.build();
        assert!(output.contains("enabled"));
        assert!(!output.contains("disabled"));
        assert!(output.contains("active"));
    }

    #[test]
    fn test_block() {
        let mut kdl = KdlBuilder::new();
        kdl.block("parent", |b| {
            b.field_i32("child", 123);
        });

        let output = kdl.build();
        assert!(output.contains("parent {"));
        assert!(output.contains("    child 123"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_nested_blocks() {
        let mut kdl = KdlBuilder::new();
        kdl.block("outer", |b| {
            b.block("inner", |b| {
                b.field_i32("value", 42);
            });
        });

        let output = kdl.build();
        assert!(output.contains("outer {"));
        assert!(output.contains("    inner {"));
        assert!(output.contains("        value 42"));
    }

    #[test]
    fn test_empty_block_skipped() {
        let mut kdl = KdlBuilder::new();
        kdl.block("empty", |_b| {
            // No content
        });

        let output = kdl.build();
        assert!(!output.contains("empty"));
    }

    #[test]
    fn test_color_field() {
        let mut kdl = KdlBuilder::new();
        let color = Color {
            r: 255,
            g: 128,
            b: 64,
            a: 255,
        };
        kdl.field_color("my-color", &color);

        let output = kdl.build();
        assert!(output.contains("my-color \"#ff8040\""));
    }

    #[test]
    fn test_conditional_fields() {
        let mut kdl = KdlBuilder::new();
        kdl.field_i32_if_not("changed", 10, 5); // Different, should appear
        kdl.field_i32_if_not("default", 5, 5); // Same, should not appear
        kdl.field_string_if_not_empty("has-value", "hello");
        kdl.field_string_if_not_empty("empty", "");

        let output = kdl.build();
        assert!(output.contains("changed 10"));
        assert!(!output.contains("default 5"));
        assert!(output.contains("has-value \"hello\""));
        assert!(!output.contains("empty \"\""));
    }

    #[test]
    fn test_with_header() {
        let kdl = KdlBuilder::with_header("Auto-generated config");
        let output = kdl.build();
        assert!(output.starts_with("// Auto-generated config"));
    }

    #[test]
    fn test_string_escaping() {
        let mut kdl = KdlBuilder::new();
        kdl.field_string("with-quotes", "say \"hello\"");
        kdl.field_string("with-backslash", r"path\to\file");
        kdl.field_string("with-newline", "line1\nline2");

        let output = kdl.build();
        // Quotes should be escaped
        assert!(output.contains(r#"with-quotes "say \"hello\"""#));
        // Backslashes should be escaped
        assert!(output.contains(r#"with-backslash "path\\to\\file""#));
        // Newlines should be escaped
        assert!(output.contains(r#"with-newline "line1\nline2""#));
    }

    #[test]
    fn test_format_inline_attr() {
        let attr = KdlBuilder::format_inline_attr("key", "value with \"quotes\"");
        assert_eq!(attr, r#"key="value with \"quotes\"""#);
    }
}
