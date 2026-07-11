//! Top-level background blur loader (niri >= 26.04)

use super::helpers::read_kdl_file;
use crate::config::models::Settings;
use crate::config::parser::{get_f64, get_i64, has_flag};
use kdl::KdlDocument;
use log::debug;
use std::path::Path;

/// Parse the top-level `blur` node from a document into settings.
///
/// Lenient: unknown children are ignored, missing children keep defaults.
/// Accepts both `off` (disable) and `on` (re-enable) flags; `on` is never
/// emitted but is accepted for forward compatibility.
pub fn parse_blur_from_doc(doc: &KdlDocument, settings: &mut Settings) {
    let Some(blur_node) = doc.get("blur") else {
        return;
    };
    let Some(children) = blur_node.children() else {
        return;
    };

    // `off` disables all blur; `on` re-enables (merge override). Apply in order
    // so a trailing `on` wins over an earlier `off`.
    if has_flag(children, &["off"]) {
        settings.blur.enabled = false;
    }
    if has_flag(children, &["on"]) {
        settings.blur.enabled = true;
    }

    if let Some(v) = get_i64(children, &["passes"]) {
        settings.blur.passes = v as i32;
    }
    // offset / noise / saturation accept both integer and float args.
    if let Some(v) = get_f64(children, &["offset"]) {
        settings.blur.offset = v;
    }
    if let Some(v) = get_f64(children, &["noise"]) {
        settings.blur.noise = v;
    }
    if let Some(v) = get_f64(children, &["saturation"]) {
        settings.blur.saturation = v;
    }
}

/// Load blur settings from the given path (falls back to defaults if missing).
pub fn load_blur(path: &Path, settings: &mut Settings) {
    let Some(doc) = read_kdl_file(path) else {
        return;
    };
    parse_blur_from_doc(&doc, settings);
    debug!("Loaded blur settings from {:?}", path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Settings {
        let doc: KdlDocument = src.parse().expect("test kdl parses");
        let mut settings = Settings::default();
        parse_blur_from_doc(&doc, &mut settings);
        settings
    }

    #[test]
    fn test_blur_loader_accepts_integer_args() {
        let s = parse("blur {\n passes 4\n offset 3\n noise 1\n saturation 2\n}");
        assert_eq!(s.blur.passes, 4);
        assert_eq!(s.blur.offset, 3.0);
        assert_eq!(s.blur.noise, 1.0);
        assert_eq!(s.blur.saturation, 2.0);
    }

    #[test]
    fn test_blur_loader_accepts_on_flag() {
        let s = parse("blur {\n on\n}");
        assert!(s.blur.enabled);
    }

    #[test]
    fn test_blur_loader_off_flag() {
        let s = parse("blur {\n off\n passes 5\n}");
        assert!(!s.blur.enabled);
        assert_eq!(s.blur.passes, 5);
    }

    #[test]
    fn test_blur_missing_node_keeps_defaults() {
        let s = parse("// nothing here\n");
        assert_eq!(s.blur, crate::config::models::BlurSettings::default());
    }
}
