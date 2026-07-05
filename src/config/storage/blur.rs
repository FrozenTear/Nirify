//! Top-level background blur storage (niri >= 26.04)

use super::builder::KdlBuilder;
use crate::config::models::BlurSettings;

/// Generate the `blur.kdl` content for the top-level blur section.
///
/// When disabled, emits the `off` flag alongside the values so the disabled
/// state round-trips losslessly (BlurPart decodes each child independently).
pub fn generate_blur_kdl(settings: &BlurSettings) -> String {
    let mut kdl = KdlBuilder::with_header("Background blur - managed by Nirify");
    kdl.comment("Requires niri 26.04+. Applies to windows/layers that request blur");
    kdl.comment("(ext-background-effect) or have it forced via window/layer rules.");
    kdl.newline();
    kdl.block("blur", |b| {
        if !settings.enabled {
            b.flag("off");
        }
        b.field_i32("passes", settings.passes);
        b.field_f64("offset", settings.offset);
        b.field_f64("noise", settings.noise);
        b.field_f64("saturation", settings.saturation);
    });
    kdl.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::loader::parse_blur_from_doc;
    use crate::config::models::Settings;
    use kdl::KdlDocument;

    /// Generate KDL, re-parse it via the kdl crate (validation gate), then load
    /// it back through the loader and return the resulting BlurSettings.
    fn roundtrip(s: &BlurSettings) -> BlurSettings {
        let generated = generate_blur_kdl(s);
        let doc: KdlDocument = generated
            .parse()
            .expect("generated blur KDL must parse via kdl crate");
        let mut settings = Settings::default();
        parse_blur_from_doc(&doc, &mut settings);
        settings.blur
    }

    #[test]
    fn test_blur_roundtrip_defaults() {
        let s = BlurSettings::default();
        assert_eq!(roundtrip(&s), s);
    }

    #[test]
    fn test_blur_roundtrip_custom() {
        let s = BlurSettings {
            enabled: true,
            passes: 5,
            offset: 10.5,
            noise: 0.1,
            saturation: 0.8,
        };
        assert_eq!(roundtrip(&s), s);
    }

    #[test]
    fn test_blur_roundtrip_disabled_preserves_values() {
        let s = BlurSettings {
            enabled: false,
            passes: 6,
            offset: 2.0,
            noise: 0.5,
            saturation: 1.0,
        };
        let out = roundtrip(&s);
        assert_eq!(out, s);
        assert!(!out.enabled);
    }

    #[test]
    fn test_blur_kdl_reparses() {
        let generated = generate_blur_kdl(&BlurSettings::default());
        let doc: KdlDocument = generated.parse().expect("must parse");
        assert!(doc.get("blur").is_some());
    }

    #[test]
    fn test_blur_precision() {
        // noise 0.02 must serialize containing "0.02", not an f32 artifact.
        let generated = generate_blur_kdl(&BlurSettings::default());
        assert!(
            generated.contains("noise 0.02"),
            "expected 'noise 0.02' in:\n{}",
            generated
        );
    }
}
