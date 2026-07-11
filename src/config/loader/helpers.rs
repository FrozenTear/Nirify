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

/// Parse a color from a KDL string value.
///
/// Accepts our hex forms first (fast path, keeps existing normalization), then
/// falls back to any CSS color string that niri accepts — `rgb()`, `rgba()`,
/// `hsl()`, named colors, etc. (niri parses colors with `csscolorparser`; see
/// niri-appearance.rs). Storage still emits hex, so CSS forms are normalized on
/// the next save.
pub fn parse_color(value: &str) -> Option<Color> {
    if let Some(c) = Color::from_hex(value) {
        return Some(c);
    }
    let [r, g, b, a] = csscolorparser::parse(value.trim()).ok()?.clamp().to_rgba8();
    Some(Color { r, g, b, a })
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

/// Parse scroll method from string.
///
/// Returns `None` for an unrecognized value (caller treats absent/unknown as
/// "use libinput device default").
pub fn parse_scroll_method(s: &str) -> Option<ScrollMethod> {
    match s {
        "two-finger" => Some(ScrollMethod::TwoFinger),
        "edge" => Some(ScrollMethod::Edge),
        "on-button-down" => Some(ScrollMethod::OnButtonDown),
        "no-scroll" => Some(ScrollMethod::NoScroll),
        other => {
            log::warn!("Unknown scroll-method \"{}\"; ignoring", other);
            None
        }
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
/// Intended for secondary analysis passes (e.g. the disabled-rule preprocessor)
/// that require source text the KDL parser elides.
pub(crate) fn read_raw_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// UTF-8 sequence length for a leading byte.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

/// Rewrites top-level slashdashed nodes `/-<name>` into `nirify-disabled-<name>`
/// so the kdl crate parses them as real, accessible nodes (kdl treats `/-`
/// nodes as trivia). String- and comment-aware: `/-`, `{`, `}` occurring inside
/// quoted/raw/multiline strings or line/block comments are never interpreted.
///
/// Only names present in `node_names` and only at brace-depth 0 are rewritten;
/// everything else (including leading `// name` comments) is copied verbatim so
/// name comments survive for the loader.
pub fn preprocess_disabled_rules(raw: &str, node_names: &[&str]) -> String {
    let b = raw.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n + 32);
    let mut i = 0usize;
    let mut depth: i32 = 0;

    while i < n {
        let c = b[i];

        // Line comment: // .. EOL
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let start = i;
            i += 2;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Block comment: /* .. */  (nested per KDL)
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let start = i;
            i += 2;
            let mut bdepth = 1;
            while i < n && bdepth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    bdepth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    bdepth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Raw string: (#+)" .. "(#+)  — also accepts the legacy v1 form r#"..."#
        if c == b'#' || (c == b'r' && i + 1 < n && (b[i + 1] == b'#' || b[i + 1] == b'"')) {
            let mut j = i;
            if c == b'r' {
                j += 1;
            }
            let hash_start = j;
            while j < n && b[j] == b'#' {
                j += 1;
            }
            if j < n && b[j] == b'"' {
                let hashes = j - hash_start;
                let start = i;
                i = j + 1;
                loop {
                    if i >= n {
                        break;
                    }
                    if b[i] == b'"' {
                        let mut k = i + 1;
                        let mut cnt = 0;
                        while k < n && cnt < hashes && b[k] == b'#' {
                            k += 1;
                            cnt += 1;
                        }
                        if cnt == hashes {
                            i = k;
                            break;
                        }
                    }
                    i += 1;
                }
                out.push_str(&raw[start..i]);
                continue;
            }
            // Not a raw-string opener: fall through to copy the char.
        }

        // Quoted string, including multiline """ .. """
        if c == b'"' {
            let start = i;
            if i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                // Multiline string.
                i += 3;
                while i < n {
                    if b[i] == b'"' && i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                out.push_str(&raw[start..i]);
                continue;
            }
            // Single-line quoted string with backslash escapes.
            i += 1;
            while i < n {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Brace depth tracking (Normal state only).
        if c == b'{' {
            depth += 1;
            out.push('{');
            i += 1;
            continue;
        }
        if c == b'}' {
            depth -= 1;
            out.push('}');
            i += 1;
            continue;
        }

        // Slashdash node at top level.
        if c == b'/' && i + 1 < n && b[i + 1] == b'-' && depth == 0 {
            let mut k = i + 2;
            while k < n && (b[k] == b' ' || b[k] == b'\t') {
                k += 1;
            }
            let id_start = k;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'-' || b[k] == b'_') {
                k += 1;
            }
            let ident = &raw[id_start..k];
            let follows_ok = k >= n
                || b[k] == b' '
                || b[k] == b'\t'
                || b[k] == b'\n'
                || b[k] == b'\r'
                || b[k] == b'{';
            if follows_ok && !ident.is_empty() && node_names.contains(&ident) {
                out.push_str("nirify-disabled-");
                out.push_str(ident);
                i = k;
                continue;
            }
            out.push_str("/-");
            i += 2;
            continue;
        }

        // Default: copy one (possibly multibyte) character.
        let ch_len = utf8_len(c).min(n - i);
        out.push_str(&raw[i..i + ch_len]);
        i += ch_len;
    }

    out
}

/// Un-slashdashes version-gated child content so the kdl crate parses it back
/// into the model. The generators write `background-effect`, `popups` and the
/// `layer=` match criterion slashdashed (`/-…`) when the detected niri version
/// doesn't support them (policy P1: gate without data loss); this pass strips
/// that `/-` at any brace depth so the loader reads the values normally.
///
/// String- and comment-aware, mirroring [`preprocess_disabled_rules`]: `/-`,
/// braces and identifiers occurring inside quoted/raw/multiline strings or
/// line/block comments are never interpreted. Only the exact gated names are
/// affected; other slashdashed nodes (e.g. a manually disabled `/-shadow`) are
/// copied verbatim.
pub fn unslashdash_gated_content(raw: &str) -> String {
    const GATED: &[&str] = &["background-effect", "popups", "layer"];
    let b = raw.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n);
    let mut i = 0usize;

    while i < n {
        let c = b[i];

        // Line comment: // .. EOL
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let start = i;
            i += 2;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Block comment: /* .. */ (nested per KDL)
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let start = i;
            i += 2;
            let mut bdepth = 1;
            while i < n && bdepth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    bdepth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    bdepth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Raw string: (#+)" .. "(#+) — also accepts the legacy v1 form r#"..."#
        if c == b'#' || (c == b'r' && i + 1 < n && (b[i + 1] == b'#' || b[i + 1] == b'"')) {
            let mut j = i;
            if c == b'r' {
                j += 1;
            }
            let hash_start = j;
            while j < n && b[j] == b'#' {
                j += 1;
            }
            if j < n && b[j] == b'"' {
                let hashes = j - hash_start;
                let start = i;
                i = j + 1;
                loop {
                    if i >= n {
                        break;
                    }
                    if b[i] == b'"' {
                        let mut k = i + 1;
                        let mut cnt = 0;
                        while k < n && cnt < hashes && b[k] == b'#' {
                            k += 1;
                            cnt += 1;
                        }
                        if cnt == hashes {
                            i = k;
                            break;
                        }
                    }
                    i += 1;
                }
                out.push_str(&raw[start..i]);
                continue;
            }
            // Not a raw-string opener: fall through to copy the char.
        }

        // Quoted string, including multiline """ .. """
        if c == b'"' {
            let start = i;
            if i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                i += 3;
                while i < n {
                    if b[i] == b'"' && i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                out.push_str(&raw[start..i]);
                continue;
            }
            i += 1;
            while i < n {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push_str(&raw[start..i]);
            continue;
        }

        // Slashdash gated node/property at any depth.
        if c == b'/' && i + 1 < n && b[i + 1] == b'-' {
            let mut k = i + 2;
            while k < n && (b[k] == b' ' || b[k] == b'\t') {
                k += 1;
            }
            let id_start = k;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'-' || b[k] == b'_') {
                k += 1;
            }
            let ident = &raw[id_start..k];
            // Node forms end in whitespace/brace/EOF; the `layer=` property form
            // ends in `=`.
            let follows_ok = k >= n
                || b[k] == b' '
                || b[k] == b'\t'
                || b[k] == b'\n'
                || b[k] == b'\r'
                || b[k] == b'{'
                || b[k] == b'=';
            if follows_ok && GATED.contains(&ident) {
                out.push_str(ident);
                i = k;
                continue;
            }
            out.push_str("/-");
            i += 2;
            continue;
        }

        // Default: copy one (possibly multibyte) character.
        let ch_len = utf8_len(c).min(n - i);
        out.push_str(&raw[i..i + ch_len]);
        i += ch_len;
    }

    out
}

/// Returns true if `raw` contains a slashdashed node named exactly `name`
/// (`/-name` followed by whitespace, `{` or EOF), ignoring any occurrence that
/// falls inside a quoted/raw/multiline string or a line/block comment.
///
/// Mirrors the string/comment-aware scanning of [`unslashdash_gated_content`].
/// Used to read back version-gated flags that the kdl parser drops because they
/// are slashdashed (e.g. tablet `/-map-to-focused-output`), without the false
/// positives of a naive `str::contains`.
pub fn slashdash_node_present(raw: &str, name: &str) -> bool {
    let b = raw.as_bytes();
    let n = b.len();
    let mut i = 0usize;

    while i < n {
        let c = b[i];

        // Line comment: // .. EOL
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            i += 2;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Block comment: /* .. */ (nested per KDL)
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            i += 2;
            let mut bdepth = 1;
            while i < n && bdepth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    bdepth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    bdepth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Raw string: (#+)" .. "(#+) — also accepts the legacy v1 form r#"..."#
        if c == b'#' || (c == b'r' && i + 1 < n && (b[i + 1] == b'#' || b[i + 1] == b'"')) {
            let mut j = i;
            if c == b'r' {
                j += 1;
            }
            let hash_start = j;
            while j < n && b[j] == b'#' {
                j += 1;
            }
            if j < n && b[j] == b'"' {
                let hashes = j - hash_start;
                i = j + 1;
                loop {
                    if i >= n {
                        break;
                    }
                    if b[i] == b'"' {
                        let mut k = i + 1;
                        let mut cnt = 0;
                        while k < n && cnt < hashes && b[k] == b'#' {
                            k += 1;
                            cnt += 1;
                        }
                        if cnt == hashes {
                            i = k;
                            break;
                        }
                    }
                    i += 1;
                }
                continue;
            }
            // Not a raw-string opener: fall through to copy the char.
        }

        // Quoted string, including multiline """ .. """
        if c == b'"' {
            if i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                i += 3;
                while i < n {
                    if b[i] == b'"' && i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            i += 1;
            while i < n {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Slashdash node at any depth.
        if c == b'/' && i + 1 < n && b[i + 1] == b'-' {
            let mut k = i + 2;
            while k < n && (b[k] == b' ' || b[k] == b'\t') {
                k += 1;
            }
            let id_start = k;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'-' || b[k] == b'_') {
                k += 1;
            }
            let ident = &raw[id_start..k];
            let follows_ok = k >= n
                || b[k] == b' '
                || b[k] == b'\t'
                || b[k] == b'\n'
                || b[k] == b'\r'
                || b[k] == b'{';
            if follows_ok && ident == name {
                return true;
            }
            i += 2;
            continue;
        }

        // Default: advance one (possibly multibyte) character.
        let ch_len = utf8_len(c).min(n - i);
        i += ch_len;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::preprocess_disabled_rules;

    const NAMES: &[&str] = &["window-rule", "layer-rule"];

    #[test]
    fn rewrites_top_level_disabled_rule() {
        let out = preprocess_disabled_rules("/-window-rule {\n    opacity 0.5\n}\n", NAMES);
        assert!(out.contains("nirify-disabled-window-rule {"));
        assert!(!out.contains("/-window-rule"));
    }

    #[test]
    fn rewrites_disabled_rule_with_space() {
        let out = preprocess_disabled_rules("/- window-rule {\n}\n", NAMES);
        assert!(out.contains("nirify-disabled-window-rule"));
    }

    #[test]
    fn braces_in_quoted_string_do_not_confuse_depth() {
        let input = "/-window-rule {\n    match title=\"foo}bar{\"\n}\nwindow-rule {\n}\n";
        let out = preprocess_disabled_rules(input, NAMES);
        assert!(out.contains("nirify-disabled-window-rule"));
        assert!(out.contains("\nwindow-rule {"));
        assert!(out.contains("title=\"foo}bar{\""));
    }

    #[test]
    fn slashdash_inside_string_is_not_rewritten() {
        let input = "window-rule {\n    match title=\"/-window-rule\"\n}\n";
        let out = preprocess_disabled_rules(input, NAMES);
        assert!(!out.contains("nirify-disabled"));
        assert!(out.contains("\"/-window-rule\""));
    }

    #[test]
    fn slashdash_inside_line_comment_is_not_rewritten() {
        let input = "// /-window-rule here\nwindow-rule {\n}\n";
        let out = preprocess_disabled_rules(input, NAMES);
        assert!(!out.contains("nirify-disabled"));
    }

    #[test]
    fn slashdash_inside_block_comment_is_not_rewritten() {
        let input = "/* /-window-rule */\nwindow-rule {\n}\n";
        let out = preprocess_disabled_rules(input, NAMES);
        assert!(!out.contains("nirify-disabled"));
    }

    #[test]
    fn nested_depth_slashdash_not_rewritten() {
        let input = "window-rule {\n    /-shadow {\n        off\n    }\n}\n";
        let out = preprocess_disabled_rules(input, NAMES);
        assert!(!out.contains("nirify-disabled"));
        assert!(out.contains("/-shadow"));
    }

    // ---- unslashdash_gated_content: adversarial probes for the lexer's
    // string/comment handling, so regressions there are caught. ----

    fn chain(raw: &str) -> String {
        super::preprocess_disabled_rules(&super::unslashdash_gated_content(raw), NAMES)
    }

    fn parses(s: &str) -> bool {
        s.parse::<kdl::KdlDocument>().is_ok()
    }

    #[test]
    fn double_slashdash_restores_gated_children_of_disabled_rule() {
        let input = "// My rule\n/-layer-rule {\n    match namespace=\"^x$\" /-layer=\"top\"\n    // (preserved via /- for older niri; applies on niri 26.04+)\n    /-background-effect {\n        blur true\n    }\n    /-popups {\n        opacity 0.85\n    }\n}\n";
        let out = chain(input);
        assert!(out.contains("nirify-disabled-layer-rule {"));
        assert!(out.contains("\n    background-effect {") && !out.contains("/-background-effect"));
        assert!(out.contains("\n    popups {") && !out.contains("/-popups"));
        assert!(out.contains(" layer=\"top\"") && !out.contains("/-layer="));
        let doc = out.parse::<kdl::KdlDocument>().expect("output parses");
        let node = &doc.nodes()[0];
        assert_eq!(node.name().value(), "nirify-disabled-layer-rule");
        let kids = node.children().unwrap();
        assert!(kids.get("background-effect").is_some() && kids.get("popups").is_some());
        let m = kids.get("match").unwrap();
        assert!(m
            .entries()
            .iter()
            .any(|e| e.name().map(|n| n.value()) == Some("layer")
                && e.value().as_string() == Some("top")));
    }

    #[test]
    fn gated_names_inside_quoted_string_untouched() {
        let input =
            "window-rule {\n    match title=\"/-popups and /-background-effect and /-layer=\"\n}\n";
        let out = chain(input);
        assert!(out.contains("\"/-popups and /-background-effect and /-layer=\""));
        assert!(parses(&out));
    }

    #[test]
    fn gated_name_inside_line_comment_untouched_but_real_node_restored() {
        let input = "window-rule {\n    // keep /-popups literal\n    /-popups {\n        opacity 0.5\n    }\n}\n";
        let out = chain(input);
        assert!(out.contains("// keep /-popups literal"));
        assert!(out.contains("\n    popups {"));
    }

    #[test]
    fn gated_name_inside_block_comment_untouched() {
        let input = "/* /-background-effect */\nwindow-rule {\n}\n";
        let out = chain(input);
        assert!(out.contains("/* /-background-effect */"));
    }

    #[test]
    fn disabled_layer_rule_not_falsely_matched_by_layer_gate() {
        let input = "/-layer-rule {\n    opacity 0.5\n}\n";
        let out = chain(input);
        assert!(out.contains("nirify-disabled-layer-rule"));
    }

    #[test]
    fn user_disabled_shadow_preserved_verbatim() {
        let input = "window-rule {\n    /-shadow {\n        off\n    }\n}\n";
        let out = chain(input);
        assert!(out.contains("/-shadow"));
    }

    #[test]
    fn escaped_quote_string_before_fake_slashdash_untouched() {
        let input = "window-rule {\n    match title=\"a\\\"/-popups\"\n}\n";
        let out = chain(input);
        assert!(out.contains("\"a\\\"/-popups\""));
        assert!(parses(&out));
    }

    #[test]
    fn raw_string_with_gated_slashdash_untouched() {
        let input = "window-rule {\n    match title=#\"/-popups\"#\n}\n";
        let out = chain(input);
        assert!(out.contains("#\"/-popups\"#"));
    }

    #[test]
    fn multiline_string_with_gated_slashdash_untouched() {
        let input = "window-rule {\n    match title=\"\"\"\n/-popups\n\"\"\"\n}\n";
        let out = chain(input);
        assert!(out.contains("\"\"\"\n/-popups\n\"\"\""));
    }

    #[test]
    fn enabled_rule_gated_content_restored_with_nested_background_effect() {
        let input = "// Effects\nwindow-rule {\n    match app-id=\"^foo$\"\n    // (preserved via /- for older niri; applies on niri 26.04+)\n    /-background-effect {\n        xray true\n        noise 0.05\n    }\n    /-popups {\n        opacity 0.85\n        background-effect {\n            blur true\n        }\n    }\n}\n";
        let out = chain(input);
        assert!(out.contains("\n    background-effect {"));
        assert!(out.contains("\n    popups {"));
        assert!(parses(&out));
    }

    // ---- parse_color: CSS color acceptance + hex round-trip (item E1.4) ----

    #[test]
    fn parse_color_accepts_css_rgba_and_roundtrips_via_hex() {
        let c = super::parse_color("rgba(25, 25, 102, 1.0)").expect("css rgba parses");
        assert_eq!((c.r, c.g, c.b, c.a), (25, 25, 102, 255));
        // Storage emits hex; the normalized hex must re-parse to the same color.
        let hex = c.to_hex();
        let reparsed = super::parse_color(&hex).expect("emitted hex re-parses");
        assert_eq!(reparsed, c);
    }

    #[test]
    fn parse_color_accepts_named_color_and_rejects_garbage() {
        assert_eq!(
            super::parse_color("red").unwrap(),
            crate::types::Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert!(super::parse_color("#7fc8ff").is_some());
        assert!(super::parse_color("not-a-color").is_none());
    }
}
