//! Output/display configuration

use crate::constants::DEFAULT_HDR_REFERENCE_LUMINANCE;
use crate::types::{Color, HdrMode, Transform, VrrMode};

use super::layout::LayoutOverride;

/// Spicy / niri-spicy-git `hdr { }` block on an output.
///
/// `None` on [`OutputConfig::hdr`] means omit the node (off). Upstream niri
/// 26.04 does not define this; Nirify still models it so Displays can edit
/// Robert’s `hdr mode="on" { reference-luminance 300 }` without wiping it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputHdr {
    pub mode: HdrMode,
    /// SDR reference white in nits. `None` = omit `reference-luminance`.
    pub reference_luminance: Option<u32>,
}

impl OutputHdr {
    /// HDR on/auto with the default reference white (203 nits).
    #[must_use]
    pub fn with_mode(mode: HdrMode) -> Option<Self> {
        mode.to_kdl().map(|_| Self {
            mode,
            reference_luminance: Some(DEFAULT_HDR_REFERENCE_LUMINANCE),
        })
    }
}

/// Per-output hot corners configuration (v25.11+)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutputHotCorners {
    /// Whether hot corners are enabled for this output (None = use global)
    pub enabled: Option<bool>,
    /// Top-left corner enabled
    pub top_left: bool,
    /// Top-right corner enabled
    pub top_right: bool,
    /// Bottom-left corner enabled
    pub bottom_left: bool,
    /// Bottom-right corner enabled
    pub bottom_right: bool,
}

impl OutputHotCorners {
    /// Returns true if any corner is enabled
    pub fn has_any_enabled(&self) -> bool {
        self.top_left || self.top_right || self.bottom_left || self.bottom_right
    }

    /// Returns true if this is just "off" (disabled)
    pub fn is_off(&self) -> bool {
        self.enabled == Some(false) && !self.has_any_enabled()
    }
}

/// Single output/display configuration
#[derive(Debug, Clone, PartialEq)]
pub struct OutputConfig {
    pub name: String,
    pub enabled: bool,
    /// Output scale. `None` = omit from KDL (niri auto-guesses from
    /// physical size/resolution). `Some(1.0)` is an explicit 1× and must
    /// be written — niri does **not** treat unset scale as 1.0.
    pub scale: Option<f64>,
    pub mode: String, // e.g., "1920x1080@60.000"
    /// Whether mode uses custom=true flag (v25.11+)
    pub mode_custom: bool,
    /// Custom modeline string (v25.11+) - WARNING: can damage monitors
    pub modeline: Option<String>,
    /// Explicit position; `None` means automatic placement by niri
    pub position: Option<(i32, i32)>,
    pub transform: Transform,
    pub vrr: VrrMode,
    pub focus_at_startup: bool,
    /// Per-output solid background color behind windows (niri Since 0.1.8)
    pub background_color: Option<Color>,
    pub backdrop_color: Option<Color>,
    /// Per-output hot corners (v25.11+)
    pub hot_corners: Option<OutputHotCorners>,
    /// Per-output layout override (v25.11+)
    pub layout_override: Option<LayoutOverride>,
    /// Spicy / niri-spicy-git HDR. `None` = omit `hdr { }` (off).
    ///
    /// Not an upstream 26.04 key. Labeled spicy in the Displays UI.
    pub hdr: Option<OutputHdr>,
    /// Unmodeled `output { }` children, re-emitted on save.
    ///
    /// Covers remaining spicy / community keys such as `allow-tearing` and
    /// `max-bpc`. `hdr` is modeled separately so Displays can edit it.
    pub unknown_children: Vec<UnknownOutputChild>,
}

/// A child of `output { }` that Nirify does not model.
pub type UnknownOutputChild = crate::config::unknown::UnknownKdlChild;
pub use crate::config::unknown::UnknownKdlChild;

/// Child names Nirify models and writes itself. Anything else is preserved
/// in [`OutputConfig::unknown_children`].
pub const MODELED_OUTPUT_CHILD_NAMES: &[&str] = &[
    "off",
    "scale",
    "mode",
    "modeline",
    "position",
    "transform",
    "variable-refresh-rate",
    "focus-at-startup",
    "background-color",
    "backdrop-color",
    "hot-corners",
    "layout",
    "hdr",
];

/// Returns true if `name` is an `output { }` child Nirify already models.
#[must_use]
pub fn is_modeled_output_child(name: &str) -> bool {
    MODELED_OUTPUT_CHILD_NAMES.contains(&name)
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            scale: None,
            mode: String::new(),
            mode_custom: false,
            modeline: None,
            position: None,
            transform: Transform::Normal,
            vrr: VrrMode::Off,
            focus_at_startup: false,
            background_color: None,
            backdrop_color: None,
            hot_corners: None,
            layout_override: None,
            hdr: None,
            unknown_children: Vec::new(),
        }
    }
}

impl OutputConfig {
    /// Scale for UI sliders and logical-size estimates.
    ///
    /// Unset (`None`, niri auto-guess) displays as 1.0 until the user picks
    /// an explicit value. Layout helpers should still prefer live IPC size
    /// when available.
    #[must_use]
    pub fn display_scale(&self) -> f64 {
        self.scale.unwrap_or(1.0)
    }

    /// Effective HDR mode for pickers (`Off` when `hdr` is absent).
    #[must_use]
    pub fn hdr_mode(&self) -> HdrMode {
        self.hdr.as_ref().map(|h| h.mode).unwrap_or(HdrMode::Off)
    }

    /// Reference luminance for sliders (default 203 nits when unset).
    #[must_use]
    pub fn hdr_reference_luminance(&self) -> u32 {
        self.hdr
            .as_ref()
            .and_then(|h| h.reference_luminance)
            .unwrap_or(DEFAULT_HDR_REFERENCE_LUMINANCE)
    }

    /// Adopt spicy HDR when this row has none. Does not clobber On/Auto.
    pub fn adopt_hdr(&mut self, incoming: Option<&OutputHdr>) -> bool {
        if self.hdr.is_some() {
            return false;
        }
        if let Some(hdr) = incoming {
            if hdr.mode.to_kdl().is_some() {
                self.hdr = Some(hdr.clone());
                return true;
            }
        }
        false
    }

    /// Adopt unmodeled children whose node name is not already present.
    ///
    /// Used by launch-time absorb so spicy keys in a leftover `output`
    /// block are kept without replacing modeled fields on the managed row.
    pub fn adopt_unknown_children(&mut self, incoming: &[UnknownOutputChild]) -> bool {
        crate::config::unknown::adopt_unknown_children(&mut self.unknown_children, incoming)
    }
}

/// Display/output settings - holds configured outputs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OutputSettings {
    pub outputs: Vec<OutputConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adopt_hdr_fills_missing_and_does_not_clobber() {
        let incoming = OutputHdr {
            mode: HdrMode::On,
            reference_luminance: Some(300),
        };
        let mut empty = OutputConfig::default();
        assert!(empty.adopt_hdr(Some(&incoming)));
        assert_eq!(empty.hdr.as_ref().unwrap().reference_luminance, Some(300));

        let mut existing = OutputConfig {
            hdr: Some(OutputHdr {
                mode: HdrMode::Auto,
                reference_luminance: Some(203),
            }),
            ..Default::default()
        };
        assert!(!existing.adopt_hdr(Some(&incoming)));
        assert_eq!(existing.hdr_mode(), HdrMode::Auto);
        assert_eq!(existing.hdr_reference_luminance(), 203);
    }
}
