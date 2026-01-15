//! Loader helper macros for reducing KDL parsing boilerplate
//!
//! These macros standardize the common patterns used when loading settings
//! from KDL documents, reducing code duplication and potential errors.
//!
//! # Examples
//!
//! ```ignore
//! use crate::config::loader::macros::*;
//!
//! // Load a boolean flag (presence-based)
//! load_flag!(doc, settings.behavior.focus_follows_mouse, ["focus-follows-mouse"]);
//!
//! // Load an integer and convert to f32
//! load_i64_as_f32!(doc, settings.appearance.focus_ring_width, ["focus-ring", "width"]);
//!
//! // Load a string value
//! load_string!(doc, settings.keyboard.xkb_layout, ["xkb", "layout"]);
//!
//! // Load an enum with from_kdl conversion
//! load_enum!(doc, settings.behavior.mod_key, ["mod-key"], ModKey);
//! ```

/// Load a boolean flag from KDL (presence-based or explicit boolean)
///
/// Uses `has_flag` which handles:
/// - `flag` (no value) → true
/// - `flag true` → true
/// - `flag false` → false
/// - (no node) → unchanged
#[macro_export]
macro_rules! load_flag {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::has_flag;
        if has_flag($doc, &[$($path),+]) {
            $target = true;
        }
    }};
}

/// Load a flag and invert it (for "off" flags that disable features)
///
/// Example: `off` in KDL means enabled = false
#[macro_export]
macro_rules! load_flag_inverted {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::has_flag;
        if has_flag($doc, &[$($path),+]) {
            $target = false;
        }
    }};
}

/// Load an i64 value from KDL
#[macro_export]
macro_rules! load_i64 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_i64;
        if let Some(val) = get_i64($doc, &[$($path),+]) {
            $target = val;
        }
    }};
}

/// Load an i64 value from KDL and convert to i32 (with bounds checking)
#[macro_export]
macro_rules! load_i64_as_i32 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_i64;
        if let Some(val) = get_i64($doc, &[$($path),+]) {
            if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                $target = val as i32;
            } else {
                log::warn!("Value {} out of range for i32 at {:?}, ignoring", val, &[$($path),+]);
            }
        }
    }};
}

/// Load an i64 value from KDL and convert to f32
#[macro_export]
macro_rules! load_i64_as_f32 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_i64;
        if let Some(val) = get_i64($doc, &[$($path),+]) {
            $target = val as f32;
        }
    }};
}

/// Load an i64 value from KDL and convert to u32 (with bounds checking)
#[macro_export]
macro_rules! load_i64_as_u32 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_i64;
        if let Some(val) = get_i64($doc, &[$($path),+]) {
            if val >= 0 && val <= u32::MAX as i64 {
                $target = val as u32;
            } else {
                log::warn!("Value {} out of range for u32 at {:?}, ignoring", val, &[$($path),+]);
            }
        }
    }};
}

/// Load an f64 value from KDL
#[macro_export]
macro_rules! load_f64 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_f64;
        if let Some(val) = get_f64($doc, &[$($path),+]) {
            $target = val;
        }
    }};
}

/// Load an f64 value from KDL and convert to f32
#[macro_export]
macro_rules! load_f64_as_f32 {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_f64;
        if let Some(val) = get_f64($doc, &[$($path),+]) {
            $target = val as f32;
        }
    }};
}

/// Load a string value from KDL
#[macro_export]
macro_rules! load_string {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_string;
        if let Some(val) = get_string($doc, &[$($path),+]) {
            $target = val;
        }
    }};
}

/// Load an optional string value from KDL (into Option<String>)
#[macro_export]
macro_rules! load_optional_string {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_string;
        if let Some(val) = get_string($doc, &[$($path),+]) {
            $target = Some(val);
        }
    }};
}

/// Load an enum value from KDL using from_kdl conversion
///
/// The enum type must implement a `from_kdl(&str) -> Option<Self>` method.
#[macro_export]
macro_rules! load_enum {
    ($doc:expr, $target:expr, [$($path:expr),+], $enum_type:ty) => {{
        use $crate::config::parser::get_string;
        if let Some(val) = get_string($doc, &[$($path),+]) {
            if let Some(parsed) = <$enum_type>::from_kdl(&val) {
                $target = parsed;
            }
        }
    }};
}

/// Load an optional enum value from KDL (into Option<EnumType>)
#[macro_export]
macro_rules! load_optional_enum {
    ($doc:expr, $target:expr, [$($path:expr),+], $enum_type:ty) => {{
        use $crate::config::parser::get_string;
        if let Some(val) = get_string($doc, &[$($path),+]) {
            if let Some(parsed) = <$enum_type>::from_kdl(&val) {
                $target = Some(parsed);
            }
        }
    }};
}

/// Load a color value from KDL hex string
#[macro_export]
macro_rules! load_color {
    ($doc:expr, $target:expr, [$($path:expr),+]) => {{
        use $crate::config::parser::get_string;
        use $crate::types::Color;
        if let Some(hex) = get_string($doc, &[$($path),+]) {
            if let Some(color) = Color::from_hex(&hex) {
                $target = color;
            }
        }
    }};
}

/// Load settings from a nested block if it exists
///
/// This macro handles the common pattern of:
/// ```kdl
/// parent-block {
///     child-setting value
/// }
/// ```
///
/// # Example
/// ```ignore
/// with_block!(doc, "focus-ring", fr_doc, {
///     load_i64_as_f32!(fr_doc, settings.appearance.focus_ring_width, ["width"]);
///     load_color!(fr_doc, settings.appearance.focus_ring_active, ["active-color"]);
/// });
/// ```
#[macro_export]
macro_rules! with_block {
    ($doc:expr, $block_name:expr, $block_var:ident, $body:block) => {{
        if let Some(node) = $doc.get($block_name) {
            if let Some($block_var) = node.children() {
                $body
            }
        }
    }};
}

/// Load settings from a nested block, with an "off" flag check
///
/// Handles the pattern where a block can be disabled with an "off" child:
/// ```kdl
/// focus-ring {
///     off  // disables the feature
/// }
/// ```
///
/// # Example
/// ```ignore
/// with_block_or_off!(doc, "focus-ring", settings.appearance.focus_ring_enabled, fr_doc, {
///     load_i64_as_f32!(fr_doc, settings.appearance.focus_ring_width, ["width"]);
/// });
/// ```
#[macro_export]
macro_rules! with_block_or_off {
    ($doc:expr, $block_name:expr, $enabled_field:expr, $block_var:ident, $body:block) => {{
        if let Some(node) = $doc.get($block_name) {
            if let Some($block_var) = node.children() {
                if $block_var.get("off").is_some() {
                    $enabled_field = false;
                } else {
                    $enabled_field = true;
                    $body
                }
            }
        }
    }};
}

// Re-export macros at module level
pub use load_color;
pub use load_enum;
pub use load_f64;
pub use load_f64_as_f32;
pub use load_flag;
pub use load_flag_inverted;
pub use load_i64;
pub use load_i64_as_f32;
pub use load_i64_as_i32;
pub use load_i64_as_u32;
pub use load_optional_enum;
pub use load_optional_string;
pub use load_string;
pub use with_block;
pub use with_block_or_off;

#[cfg(test)]
mod tests {
    use crate::config::parser::parse_document;

    #[test]
    fn test_load_flag() {
        let doc = parse_document("enabled").unwrap();
        let mut value = false;
        load_flag!(&doc, value, ["enabled"]);
        assert!(value);
    }

    #[test]
    fn test_load_flag_missing() {
        let doc = parse_document("other").unwrap();
        let mut value = false;
        load_flag!(&doc, value, ["enabled"]);
        assert!(!value); // Unchanged
    }

    #[test]
    fn test_load_i64() {
        let doc = parse_document("width 42").unwrap();
        let mut value: i64 = 0;
        load_i64!(&doc, value, ["width"]);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_load_i64_as_f32() {
        let doc = parse_document("width 42").unwrap();
        let mut value: f32 = 0.0;
        load_i64_as_f32!(&doc, value, ["width"]);
        assert_eq!(value, 42.0);
    }

    #[test]
    fn test_load_string() {
        let doc = parse_document("name \"hello\"").unwrap();
        let mut value = String::new();
        load_string!(&doc, value, ["name"]);
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_with_block() {
        let doc = parse_document("parent {\n    child 123\n}").unwrap();
        let mut value: i64 = 0;
        with_block!(&doc, "parent", child_doc, {
            load_i64!(child_doc, value, ["child"]);
        });
        assert_eq!(value, 123);
    }

    #[test]
    fn test_with_block_or_off_enabled() {
        let doc = parse_document("feature {\n    value 42\n}").unwrap();
        let mut enabled = false;
        let mut value: i64 = 0;
        with_block_or_off!(&doc, "feature", enabled, feat_doc, {
            load_i64!(feat_doc, value, ["value"]);
        });
        assert!(enabled);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_block_or_off_disabled() {
        let doc = parse_document("feature {\n    off\n}").unwrap();
        let mut enabled = true;
        let mut value: i64 = 99;
        with_block_or_off!(&doc, "feature", enabled, _feat_doc, {
            // This block should not execute
            value = 0;
        });
        assert!(!enabled);
        assert_eq!(value, 99); // Unchanged
    }

    #[test]
    fn test_load_flag_inverted() {
        let doc = parse_document("disabled").unwrap();
        let mut enabled = true;
        load_flag_inverted!(&doc, enabled, ["disabled"]);
        assert!(!enabled);
    }

    #[test]
    fn test_load_flag_inverted_missing() {
        let doc = parse_document("other").unwrap();
        let mut enabled = true;
        load_flag_inverted!(&doc, enabled, ["disabled"]);
        assert!(enabled); // Unchanged
    }

    #[test]
    fn test_load_i64_as_i32() {
        let doc = parse_document("value 42").unwrap();
        let mut value: i32 = 0;
        load_i64_as_i32!(&doc, value, ["value"]);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_load_i64_as_i32_out_of_range() {
        // Value larger than i32::MAX should be ignored
        let doc = parse_document("value 3000000000").unwrap();
        let mut value: i32 = 99;
        load_i64_as_i32!(&doc, value, ["value"]);
        assert_eq!(value, 99); // Unchanged due to bounds check
    }

    #[test]
    fn test_load_i64_as_u32() {
        let doc = parse_document("value 42").unwrap();
        let mut value: u32 = 0;
        load_i64_as_u32!(&doc, value, ["value"]);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_load_i64_as_u32_negative() {
        // Negative value should be ignored
        let doc = parse_document("value -1").unwrap();
        let mut value: u32 = 99;
        load_i64_as_u32!(&doc, value, ["value"]);
        assert_eq!(value, 99); // Unchanged due to bounds check
    }

    #[test]
    fn test_load_f64() {
        let doc = parse_document("scale 1.5").unwrap();
        let mut value: f64 = 0.0;
        load_f64!(&doc, value, ["scale"]);
        assert!((value - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_f64_as_f32() {
        let doc = parse_document("scale 2.5").unwrap();
        let mut value: f32 = 0.0;
        load_f64_as_f32!(&doc, value, ["scale"]);
        assert!((value - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_load_optional_string() {
        let doc = parse_document("name \"test\"").unwrap();
        let mut value: Option<String> = None;
        load_optional_string!(&doc, value, ["name"]);
        assert_eq!(value, Some("test".to_string()));
    }

    #[test]
    fn test_load_optional_string_missing() {
        let doc = parse_document("other \"test\"").unwrap();
        let mut value: Option<String> = None;
        load_optional_string!(&doc, value, ["name"]);
        assert_eq!(value, None);
    }

    #[test]
    fn test_load_enum() {
        use crate::types::ModKey;
        let doc = parse_document("mod-key \"Alt\"").unwrap();
        let mut value = ModKey::Super;
        load_enum!(&doc, value, ["mod-key"], ModKey);
        assert_eq!(value, ModKey::Alt);
    }

    #[test]
    fn test_load_enum_invalid() {
        use crate::types::ModKey;
        let doc = parse_document("mod-key \"NotAKey\"").unwrap();
        let mut value = ModKey::Super;
        load_enum!(&doc, value, ["mod-key"], ModKey);
        assert_eq!(value, ModKey::Super); // Unchanged - invalid value
    }

    #[test]
    fn test_load_optional_enum() {
        use crate::types::ModKey;
        let doc = parse_document("mod-key \"Ctrl\"").unwrap();
        let mut value: Option<ModKey> = None;
        load_optional_enum!(&doc, value, ["mod-key"], ModKey);
        assert_eq!(value, Some(ModKey::Ctrl));
    }

    #[test]
    fn test_load_optional_enum_missing() {
        use crate::types::ModKey;
        let doc = parse_document("other \"test\"").unwrap();
        let mut value: Option<ModKey> = None;
        load_optional_enum!(&doc, value, ["mod-key"], ModKey);
        assert_eq!(value, None);
    }

    #[test]
    fn test_load_color() {
        use crate::types::Color;
        let doc = parse_document("color \"#ff0000\"").unwrap();
        let mut value = Color::default();
        load_color!(&doc, value, ["color"]);
        assert_eq!(value.r, 255);
        assert_eq!(value.g, 0);
        assert_eq!(value.b, 0);
    }

    #[test]
    fn test_load_color_with_alpha() {
        use crate::types::Color;
        let doc = parse_document("color \"#ff0000cc\"").unwrap();
        let mut value = Color::default();
        load_color!(&doc, value, ["color"]);
        assert_eq!(value.r, 255);
        assert_eq!(value.g, 0);
        assert_eq!(value.b, 0);
        assert_eq!(value.a, 204); // 0xcc
    }

    #[test]
    fn test_load_color_invalid() {
        use crate::types::Color;
        let doc = parse_document("color \"not-a-color\"").unwrap();
        let mut value = Color {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        };
        load_color!(&doc, value, ["color"]);
        // Should be unchanged - invalid color
        assert_eq!(value.r, 128);
        assert_eq!(value.g, 128);
    }
}
