//! Helper functions for KDL parsing
//!
//! Contains shared utilities for loading settings from KDL documents.

use super::super::parser::get_string;
use crate::types::*;
use kdl::KdlDocument;
use log::{debug, warn};
use std::fs;
use std::path::Path;

/// Status of attempting to load a KDL file
#[derive(Debug, Clone)]
pub enum FileLoadStatus {
    /// File was loaded and parsed successfully
    Loaded(KdlDocument),
    /// File does not exist (not an error for optional configs)
    Missing,
    /// File exists but failed to parse
    ParseError(String),
    /// File exists but could not be read
    ReadError(String),
}

impl FileLoadStatus {
    /// Returns true if the file was successfully loaded
    pub fn is_loaded(&self) -> bool {
        matches!(self, FileLoadStatus::Loaded(_))
    }

    /// Returns true if there was an error (parse or read)
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            FileLoadStatus::ParseError(_) | FileLoadStatus::ReadError(_)
        )
    }

    /// Get the document if loaded
    pub fn document(&self) -> Option<&KdlDocument> {
        match self {
            FileLoadStatus::Loaded(doc) => Some(doc),
            _ => None,
        }
    }

    /// Get error message if any
    pub fn error_message(&self) -> Option<&str> {
        match self {
            FileLoadStatus::ParseError(msg) | FileLoadStatus::ReadError(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Read and parse a KDL file, returning detailed status
pub fn read_kdl_file_with_status(path: &Path) -> FileLoadStatus {
    use super::super::parser::parse_document;

    match fs::read_to_string(path) {
        Ok(content) => match parse_document(&content) {
            Ok(doc) => FileLoadStatus::Loaded(doc),
            Err(e) => {
                let msg = format!("{}", e);
                warn!(
                    "Corrupted config {:?}: {} (falling back to defaults)",
                    path, e
                );
                FileLoadStatus::ParseError(msg)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Config file not found: {:?}", path);
            FileLoadStatus::Missing
        }
        Err(e) => {
            let msg = format!("{}", e);
            warn!(
                "Cannot read config {:?}: {} (falling back to defaults)",
                path, e
            );
            FileLoadStatus::ReadError(msg)
        }
    }
}

/// Try to read and parse a KDL file, returning None if it doesn't exist or fails to parse
pub fn read_kdl_file(path: &Path) -> Option<KdlDocument> {
    read_kdl_file_with_status(path).document().cloned()
}

/// Parse a color from a KDL string value
pub fn parse_color(hex: &str) -> Option<Color> {
    Color::from_hex(hex)
}

/// Load a color from KDL into a target field
///
/// Helper to reduce the repetitive pattern of:
/// ```ignore
/// if let Some(color) = get_string(doc, &["key"]) {
///     if let Some(c) = parse_color(&color) {
///         target = c;
///     }
/// }
/// ```
pub fn load_color(doc: &KdlDocument, path: &[&str], target: &mut Color) {
    if let Some(hex) = get_string(doc, path) {
        if let Some(c) = parse_color(&hex) {
            *target = c;
        }
    }
}

/// Parse scroll method from string
pub fn parse_scroll_method(s: &str) -> ScrollMethod {
    match s {
        "two-finger" => ScrollMethod::TwoFinger,
        "edge" => ScrollMethod::Edge,
        "on-button-down" => ScrollMethod::OnButtonDown,
        _ => ScrollMethod::NoScroll,
    }
}

/// Parse accel profile from string
pub fn parse_accel_profile(s: &str) -> AccelProfile {
    match s {
        "flat" => AccelProfile::Flat,
        _ => AccelProfile::Adaptive,
    }
}

/// Parse click method from string
pub fn parse_click_method(s: &str) -> ClickMethod {
    match s {
        "clickfinger" => ClickMethod::Clickfinger,
        _ => ClickMethod::ButtonAreas,
    }
}

/// Parse tap button map from string
pub fn parse_tap_button_map(s: &str) -> TapButtonMap {
    match s {
        "left-middle-right" => TapButtonMap::LeftMiddleRight,
        _ => TapButtonMap::LeftRightMiddle,
    }
}

/// Returns the raw UTF-8 content of a file if it can be read.
/// Intended for secondary analysis passes (e.g. slashdash disabled rule extraction)
/// that require source text the KDL parser elides.
pub(crate) fn read_raw_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Represents a slashdash-elided (disabled) rule found via raw text scanning.
/// This is used for Option 2 persistent disabled rules support.
#[derive(Debug, Clone)]
pub struct SlashdashRule {
    /// "window-rule" or "layer-rule"
    pub kind: String,
    /// The name extracted from a preceding `// Name` comment, if any.
    pub name: Option<String>,
    /// The full matched text including the `/-` prefix.
    pub full_block: String,
    /// The inner content (everything between the first `{` and the matching `}`).
    pub inner_content: String,
}

/// Scans raw KDL text for `/-window-rule` and `/-layer-rule` blocks using
/// simple brace counting.
///
/// This is deliberately lightweight (not a full KDL parser) because we control
/// the output format from the storage side.
pub fn extract_slashdash_rule_blocks(raw: &str) -> Vec<SlashdashRule> {
    let mut results = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let is_window = bytes[i..].starts_with(b"/-window-rule");
        let is_layer = bytes[i..].starts_with(b"/-layer-rule");

        if is_window || is_layer {
            let block_start = i;
            let kind = if is_window { "window-rule" } else { "layer-rule" };

            let name = extract_preceding_name_comment(raw, block_start);

            // Advance to the opening '{'
            while i < bytes.len() && bytes[i] != b'{' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            let open_brace_pos = i;
            let mut depth = 1;
            i += 1; // skip '{'

            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }

            if depth == 0 {
                let full_block = &raw[block_start..i];
                let inner_content = &raw[open_brace_pos + 1..i - 1];

                results.push(SlashdashRule {
                    kind: kind.to_string(),
                    name,
                    full_block: full_block.to_string(),
                    inner_content: inner_content.to_string(),
                });
            }
        } else {
            i += 1;
        }
    }

    results
}

/// Looks backwards from `pos` for a Nirify-style `// Name` comment.
fn extract_preceding_name_comment(text: &str, pos: usize) -> Option<String> {
    let before = &text[..pos];
    let lines: Vec<&str> = before.lines().rev().take(6).collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix("//") {
            let name = comment.trim();
            if !name.is_empty()
                && !name.starts_with("Window rules")
                && !name.starts_with("Layer rules")
                && !name.starts_with("managed by Nirify")
            {
                return Some(name.to_string());
            }
        }
        if !trimmed.starts_with("//") {
            break;
        }
    }
    None
}
