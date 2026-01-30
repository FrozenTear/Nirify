//! Niri version detection and feature compatibility
//!
//! This module provides utilities for parsing niri version strings and
//! determining which features are supported by the installed version.

use std::cmp::Ordering;

/// Represents a parsed niri version (e.g., "25.08" -> major=25, minor=8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NiriVersion {
    pub major: u32,
    pub minor: u32,
}

impl NiriVersion {
    /// Parse a version string like "25.08" or "25.08-123-gabcdef"
    ///
    /// Returns None if the version string cannot be parsed.
    pub fn parse(version_str: &str) -> Option<Self> {
        // Version format: "25.08" or "25.08-123-gabcdef" (git describe format)
        // Take the part before any dash
        let version_part = version_str.split('-').next()?;

        let mut parts = version_part.split('.');
        let major: u32 = parts.next()?.parse().ok()?;
        let minor: u32 = parts.next()?.parse().ok()?;

        Some(Self { major, minor })
    }

    /// Check if this version is at least the specified version
    pub fn at_least(&self, major: u32, minor: u32) -> bool {
        match self.major.cmp(&major) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => self.minor >= minor,
        }
    }
}

impl std::fmt::Display for NiriVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:02}", self.major, self.minor)
    }
}

impl PartialOrd for NiriVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NiriVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => self.minor.cmp(&other.minor),
            ord => ord,
        }
    }
}

/// Features that require specific niri versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NiriFeature {
    /// Recent windows (Alt-Tab) switcher configuration
    RecentWindows,
}

impl NiriFeature {
    /// Get the minimum niri version required for this feature
    pub fn min_version(&self) -> NiriVersion {
        match self {
            Self::RecentWindows => NiriVersion { major: 25, minor: 11 },
        }
    }

    /// Get a human-readable name for this feature
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::RecentWindows => "Recent Windows (Alt-Tab) Switcher",
        }
    }

    /// Check if this feature is supported by the given version
    pub fn is_supported_by(&self, version: NiriVersion) -> bool {
        version >= self.min_version()
    }
}

/// Check which features are unsupported by the given version
pub fn get_unsupported_features(version: NiriVersion) -> Vec<NiriFeature> {
    let all_features = [NiriFeature::RecentWindows];

    all_features
        .into_iter()
        .filter(|f| !f.is_supported_by(version))
        .collect()
}

/// Feature compatibility context for config generation
#[derive(Debug, Clone, Copy, Default)]
pub struct FeatureCompat {
    pub recent_windows: bool,
}

impl FeatureCompat {
    /// Create feature compatibility from detected niri version
    pub fn from_version(version: Option<NiriVersion>) -> Self {
        match version {
            Some(v) => Self {
                recent_windows: NiriFeature::RecentWindows.is_supported_by(v),
            },
            // If we can't detect version, be conservative and disable new features
            None => Self {
                recent_windows: false,
            },
        }
    }

    /// Create with all features enabled (for testing or when user confirms)
    pub fn all_enabled() -> Self {
        Self {
            recent_windows: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_version() {
        let v = NiriVersion::parse("25.08").unwrap();
        assert_eq!(v.major, 25);
        assert_eq!(v.minor, 8);
    }

    #[test]
    fn test_parse_git_describe_version() {
        let v = NiriVersion::parse("25.08-123-g4310c20c").unwrap();
        assert_eq!(v.major, 25);
        assert_eq!(v.minor, 8);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(NiriVersion::parse("invalid").is_none());
        assert!(NiriVersion::parse("").is_none());
        assert!(NiriVersion::parse("25").is_none());
    }

    #[test]
    fn test_version_comparison() {
        let v25_08 = NiriVersion { major: 25, minor: 8 };
        let v25_11 = NiriVersion { major: 25, minor: 11 };
        let v26_00 = NiriVersion { major: 26, minor: 0 };

        assert!(v25_08 < v25_11);
        assert!(v25_11 < v26_00);
        assert!(v25_08 < v26_00);

        assert!(v25_11.at_least(25, 11));
        assert!(v25_11.at_least(25, 8));
        assert!(!v25_08.at_least(25, 11));
        assert!(v26_00.at_least(25, 11));
    }

    #[test]
    fn test_feature_support() {
        let v25_08 = NiriVersion { major: 25, minor: 8 };
        let v25_11 = NiriVersion { major: 25, minor: 11 };

        assert!(!NiriFeature::RecentWindows.is_supported_by(v25_08));
        assert!(NiriFeature::RecentWindows.is_supported_by(v25_11));
    }

    #[test]
    fn test_get_unsupported_features() {
        let v25_08 = NiriVersion { major: 25, minor: 8 };
        let unsupported = get_unsupported_features(v25_08);
        assert!(unsupported.contains(&NiriFeature::RecentWindows));

        let v25_11 = NiriVersion { major: 25, minor: 11 };
        let unsupported = get_unsupported_features(v25_11);
        assert!(unsupported.is_empty());
    }

    #[test]
    fn test_display() {
        let v = NiriVersion { major: 25, minor: 8 };
        assert_eq!(v.to_string(), "25.08");
    }
}
