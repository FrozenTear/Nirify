use super::error::ConfigError;
use kdl::{KdlDocument, KdlNode};

/// Parse a KDL document from string
pub fn parse_document(content: &str) -> Result<KdlDocument, ConfigError> {
    content.parse().map_err(ConfigError::KdlError)
}

/// Navigate through a KDL document to find a node at the given path
fn navigate_to_node<'a>(doc: &'a KdlDocument, path: &[&str]) -> Option<&'a KdlNode> {
    let mut current_doc = doc;

    for (i, &name) in path.iter().enumerate() {
        let node = current_doc.get(name)?;
        if i == path.len() - 1 {
            return Some(node);
        }
        current_doc = node.children()?;
    }
    None
}

/// Get a string value from a KDL node
pub fn get_string(doc: &KdlDocument, path: &[&str]) -> Option<String> {
    navigate_to_node(doc, path)?
        .entries()
        .first()
        .and_then(|e| e.value().as_string())
        .map(|s| s.to_string())
}

/// Get an integer value from a KDL node
pub fn get_i64(doc: &KdlDocument, path: &[&str]) -> Option<i64> {
    navigate_to_node(doc, path)?
        .entries()
        .first()
        .and_then(|e| e.value().as_integer())
        .and_then(|i| i64::try_from(i).ok())
}

/// Get a float value from a KDL node
pub fn get_f64(doc: &KdlDocument, path: &[&str]) -> Option<f64> {
    let node = navigate_to_node(doc, path)?;
    let entry = node.entries().first()?;

    // Try float first, then integer (kdl 6.x stores ints as i128)
    entry
        .value()
        .as_float()
        .or_else(|| entry.value().as_integer().map(|i| i as f64))
}

/// Check if a flag is enabled
///
/// Handles both niri's idiomatic presence-based flags and explicit booleans:
/// - `tap` (no value) → true
/// - `tap true` → true
/// - `tap false` → false
/// - (no node) → false
pub fn has_flag(doc: &KdlDocument, path: &[&str]) -> bool {
    match navigate_to_node(doc, path) {
        None => false,
        Some(node) => {
            // Check if there's an explicit boolean value
            if let Some(entry) = node.entries().first() {
                if entry.name().is_none() {
                    // Positional argument - check if it's a boolean
                    if let Some(b) = entry.value().as_bool() {
                        return b;
                    }
                }
            }
            // No explicit false, presence means enabled
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document() {
        let content = r#"
            layout {
                gaps 16
            }
        "#;
        let doc = parse_document(content).unwrap();
        assert!(doc.get("layout").is_some());
    }

    #[test]
    fn test_get_i64() {
        let content = r#"
            layout {
                gaps 16
            }
        "#;
        let doc = parse_document(content).unwrap();
        let layout = doc.get("layout").unwrap().children().unwrap();
        assert_eq!(get_i64(layout, &["gaps"]), Some(16));
    }

    #[test]
    fn test_has_flag_presence() {
        // Presence-based flag (niri's idiomatic style)
        let content = r#"
            touchpad {
                tap
                natural-scroll
            }
        "#;
        let doc = parse_document(content).unwrap();
        let touchpad = doc.get("touchpad").unwrap().children().unwrap();
        assert!(has_flag(touchpad, &["tap"]));
        assert!(has_flag(touchpad, &["natural-scroll"]));
        assert!(!has_flag(touchpad, &["dwt"])); // Not present
    }

    #[test]
    fn test_has_flag_explicit_false() {
        // Explicit false should return false
        let content = r#"
            touchpad {
                tap false
                natural-scroll true
            }
        "#;
        let doc = parse_document(content).unwrap();
        let touchpad = doc.get("touchpad").unwrap().children().unwrap();
        assert!(!has_flag(touchpad, &["tap"])); // Explicitly false
        assert!(has_flag(touchpad, &["natural-scroll"])); // Explicitly true
    }

    #[test]
    fn test_has_flag_with_other_values() {
        // Flags with non-boolean values should return true (presence)
        let content = r#"
            settings {
                mode "custom"
                size 32
            }
        "#;
        let doc = parse_document(content).unwrap();
        let settings = doc.get("settings").unwrap().children().unwrap();
        // These have values but not boolean false, so presence = true
        assert!(has_flag(settings, &["mode"]));
        assert!(has_flag(settings, &["size"]));
    }
}
