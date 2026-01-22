//! Loader for application preferences
//!
//! Preferences are app-specific settings (like UI theme) that are not part
//! of niri's configuration.

use crate::config::models::Settings;
use crate::config::parser;
use kdl::KdlDocument;
use std::path::Path;

/// Load preferences from preferences.kdl file
pub fn load_preferences(path: &Path, settings: &mut Settings) {
    let status = super::helpers::read_kdl_file_with_status(path);
    if let Some(doc) = status.document() {
        parse_preferences_from_doc(doc, settings);
    }
}

/// Parse preferences from a KDL document
pub fn parse_preferences_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    // preferences { theme "NiriAmber" }
    if let Some(prefs_node) = doc.get("preferences") {
        if let Some(children) = prefs_node.children() {
            // Read theme
            if let Some(theme) = parser::get_string(children, &["theme"]) {
                settings.preferences.theme = theme.to_string();
            }
        }
    }
}
