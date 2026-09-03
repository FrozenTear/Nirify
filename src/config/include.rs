//! Shared niri `include` resolution for conflict scan, import, and keybindings.
//!
//! Aligns with niri **26.04**:
//! - `~/` expands to the home directory ([Include § Home dir](https://niri-wm.github.io/niri/Configuration%3A-Include.html)).
//! - `optional=true` means a missing file is not an error (niri still warns in
//!   its own logs; Nirify does not treat it as a failed walk).
//!
//! Nirify still **writes** the relative form `include "nirify/main.kdl"` (last
//! top-level node). That works on 25.11+ and does not depend on tilde expansion.
//!
//! # Jail policy
//!
//! niri itself will load any resolvable include, including `~/dotfiles/…`.
//! Nirify **import** and **keybindings load** still refuse paths whose
//! canonical location is outside `$XDG_CONFIG_HOME/niri` (typically
//! `~/.config/niri`). Those includes stay in `config.kdl` as unmanaged nodes
//! and niri will apply them; Nirify just will not copy their settings into
//! `nirify/*.kdl`. Conflict scan is read-only and is **not** jailed, so a
//! `~/` include that declares managed sections is still surfaced.

use kdl::KdlNode;
use log::{debug, warn};
use std::path::{Path, PathBuf};

/// A parsed top-level `include` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeDirective {
    /// Path argument as written (`./foo.kdl`, `~/bar.kdl`, …).
    pub path: String,
    /// `optional=true` on the include node (niri 26.04).
    pub optional: bool,
}

/// How a resolved include should be treated by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncludeOpen {
    /// File exists and (for import) is inside the jail.
    Ready(PathBuf),
    /// File is missing. Not an error when [`IncludeDirective::optional`].
    Missing { resolved: PathBuf, optional: bool },
    /// Import/keybindings jail: canonical path is outside `~/.config/niri`.
    Jailed { resolved: PathBuf },
    /// Could not expand or resolve (no home dir, empty path, …).
    Unresolvable { reason: String },
}

/// Parse an `include` node. Supports both
/// `include "file.kdl" optional=true` and `include optional=true "file.kdl"`.
#[must_use]
pub fn parse_include_node(node: &KdlNode) -> Option<IncludeDirective> {
    if node.name().value() != "include" {
        return None;
    }

    let mut path = None;
    let mut optional = false;
    for entry in node.entries() {
        if let Some(name) = entry.name() {
            if name.value() == "optional" && entry.value().as_bool() == Some(true) {
                optional = true;
            }
        } else if path.is_none() {
            if let Some(s) = entry.value().as_string() {
                path = Some(s.to_string());
            }
        }
    }

    path.map(|path| IncludeDirective { path, optional })
}

/// Expand `~/` and join relative paths against `base_dir` (the current file's parent).
///
/// `home` is injectable so tests do not mutate `$HOME`. Production callers pass
/// [`dirs::home_dir()`].
#[must_use]
pub fn expand_include_path(
    include_path: &str,
    base_dir: &Path,
    home: Option<&Path>,
) -> Option<PathBuf> {
    let path = include_path.trim().trim_matches('"');
    if path.is_empty() {
        return None;
    }
    if path == "~" {
        return home.map(Path::to_path_buf);
    }
    if let Some(stripped) = path.strip_prefix("~/") {
        return Some(home?.join(stripped));
    }
    let p = Path::new(path);
    if p.is_absolute() {
        Some(p.to_path_buf())
    } else {
        Some(base_dir.join(p))
    }
}

/// Resolve an include for **conflict scan** (no jail).
///
/// Missing files yield [`IncludeOpen::Missing`]; the caller treats optional
/// missing as non-conflicting and required missing the same way (cannot scan).
#[must_use]
pub fn open_include_for_scan(
    directive: &IncludeDirective,
    base_dir: &Path,
    home: Option<&Path>,
) -> IncludeOpen {
    let Some(resolved) = expand_include_path(&directive.path, base_dir, home) else {
        return IncludeOpen::Unresolvable {
            reason: format!("cannot expand include path {:?}", directive.path),
        };
    };
    if resolved.is_file() {
        IncludeOpen::Ready(resolved)
    } else {
        IncludeOpen::Missing {
            resolved,
            optional: directive.optional,
        }
    }
}

/// Default niri config directory (`$XDG_CONFIG_HOME/niri` or `~/.config/niri`).
#[must_use]
pub fn default_niri_config_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("niri"))
}

/// Resolve an include for **import / keybindings** (tilde expand + jail).
///
/// Paths whose canonical form is not under `niri_config_dir` are
/// [`IncludeOpen::Jailed`] even when niri would load them.
#[must_use]
pub fn open_include_for_import(
    directive: &IncludeDirective,
    base_dir: &Path,
    home: Option<&Path>,
    niri_config_dir: Option<&Path>,
) -> IncludeOpen {
    let Some(resolved) = expand_include_path(&directive.path, base_dir, home) else {
        return IncludeOpen::Unresolvable {
            reason: format!("cannot expand include path {:?}", directive.path),
        };
    };

    if !resolved.exists() {
        return IncludeOpen::Missing {
            resolved,
            optional: directive.optional,
        };
    }

    let canonical = match resolved.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            debug!("Include path {:?} cannot be canonicalized: {}", resolved, e);
            return IncludeOpen::Unresolvable {
                reason: format!("cannot canonicalize {:?}: {}", resolved, e),
            };
        }
    };

    let Some(jail) = niri_config_dir else {
        warn!(
            "Include path {:?} refused: niri config directory is unknown",
            directive.path
        );
        return IncludeOpen::Jailed {
            resolved: canonical,
        };
    };

    if canonical.starts_with(jail) {
        IncludeOpen::Ready(canonical)
    } else {
        warn!(
            "Include path {:?} expands to {:?} which is outside {:?}; \
             Nirify will not import it (niri still loads the include)",
            directive.path, canonical, jail
        );
        IncludeOpen::Jailed {
            resolved: canonical,
        }
    }
}

/// User-facing warning for an import walk outcome that is not [`IncludeOpen::Ready`].
#[must_use]
pub fn import_skip_warning(directive: &IncludeDirective, open: &IncludeOpen) -> Option<String> {
    match open {
        IncludeOpen::Ready(_) => None,
        IncludeOpen::Missing { optional: true, .. } => None,
        IncludeOpen::Missing { resolved, .. } => Some(format!(
            "Could not read included file: {} ({})",
            directive.path,
            resolved.display()
        )),
        IncludeOpen::Jailed { resolved } => Some(format!(
            "Skipped include (outside ~/.config/niri): {} ({})",
            directive.path,
            resolved.display()
        )),
        IncludeOpen::Unresolvable { reason } => {
            Some(format!("Skipped include {}: {}", directive.path, reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kdl::KdlDocument;
    use std::fs;
    use std::str::FromStr;

    fn parse_one(src: &str) -> IncludeDirective {
        let doc = KdlDocument::from_str(src).expect("kdl");
        parse_include_node(doc.nodes().first().expect("node")).expect("include")
    }

    #[test]
    fn parse_optional_after_path() {
        let d = parse_one(r#"include "foo.kdl" optional=true"#);
        assert_eq!(d.path, "foo.kdl");
        assert!(d.optional);
    }

    #[test]
    fn parse_optional_before_path() {
        // niri wiki form: include optional=true "optional-config.kdl"
        let d = parse_one(r#"include optional=true "optional-config.kdl""#);
        assert_eq!(d.path, "optional-config.kdl");
        assert!(d.optional);
    }

    #[test]
    fn parse_required_include() {
        let d = parse_one(r#"include "nirify/main.kdl""#);
        assert_eq!(d.path, "nirify/main.kdl");
        assert!(!d.optional);
    }

    #[test]
    fn expand_tilde_joins_home() {
        let home = Path::new("/home/tester");
        let base = Path::new("/cfg");
        assert_eq!(
            expand_include_path("~/extra.kdl", base, Some(home)),
            Some(PathBuf::from("/home/tester/extra.kdl"))
        );
        assert_eq!(
            expand_include_path("~/dotfiles/niri/layout.kdl", base, Some(home)),
            Some(PathBuf::from("/home/tester/dotfiles/niri/layout.kdl"))
        );
        assert_eq!(
            expand_include_path("~", base, Some(home)),
            Some(PathBuf::from("/home/tester"))
        );
        assert_eq!(expand_include_path("~/x.kdl", base, None), None);
    }

    #[test]
    fn expand_relative_and_absolute() {
        let home = Path::new("/home/tester");
        let base = Path::new("/cfg/niri");
        assert_eq!(
            expand_include_path("./extra.kdl", base, Some(home)),
            Some(PathBuf::from("/cfg/niri/./extra.kdl"))
        );
        assert_eq!(
            expand_include_path("/abs/x.kdl", base, Some(home)),
            Some(PathBuf::from("/abs/x.kdl"))
        );
    }

    #[test]
    fn scan_opens_tilde_file_and_skips_optional_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let niri = home.join(".config/niri");
        fs::create_dir_all(&niri).unwrap();
        let extra = home.join("extra-layout.kdl");
        fs::write(&extra, "layout { gaps 8 }\n").unwrap();

        let present = IncludeDirective {
            path: "~/extra-layout.kdl".into(),
            optional: false,
        };
        match open_include_for_scan(&present, &niri, Some(home)) {
            IncludeOpen::Ready(p) => assert_eq!(p, extra),
            other => panic!("expected Ready, got {other:?}"),
        }

        let missing = IncludeDirective {
            path: "~/no-such.kdl".into(),
            optional: true,
        };
        match open_include_for_scan(&missing, &niri, Some(home)) {
            IncludeOpen::Missing { optional: true, .. } => {}
            other => panic!("expected optional Missing, got {other:?}"),
        }
        assert!(import_skip_warning(
            &missing,
            &open_include_for_scan(&missing, &niri, Some(home))
        )
        .is_none());
    }

    #[test]
    fn import_jails_tilde_outside_niri_config_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let niri = home.join(".config/niri");
        fs::create_dir_all(&niri).unwrap();
        let outside = home.join("dotfiles/layout.kdl");
        fs::create_dir_all(outside.parent().unwrap()).unwrap();
        fs::write(&outside, "layout { gaps 4 }\n").unwrap();

        let directive = IncludeDirective {
            path: "~/dotfiles/layout.kdl".into(),
            optional: false,
        };
        let open = open_include_for_import(&directive, &niri, Some(home), Some(&niri));
        match &open {
            IncludeOpen::Jailed { resolved } => {
                assert_eq!(resolved, &outside.canonicalize().unwrap());
            }
            other => panic!("expected Jailed, got {other:?}"),
        }
        let warning = import_skip_warning(&directive, &open).expect("jail warning");
        assert!(
            warning.contains("outside ~/.config/niri"),
            "jail warning must document the policy: {warning}"
        );

        // Same path is visible to conflict scan (niri will load it).
        match open_include_for_scan(&directive, &niri, Some(home)) {
            IncludeOpen::Ready(p) => assert_eq!(p, outside),
            other => panic!("scan should see the file, got {other:?}"),
        }
    }

    #[test]
    fn import_follows_tilde_inside_niri_config_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let niri = home.join(".config/niri");
        fs::create_dir_all(&niri).unwrap();
        let extra = niri.join("extra.kdl");
        fs::write(&extra, "layout { gaps 2 }\n").unwrap();

        let directive = IncludeDirective {
            path: "~/.config/niri/extra.kdl".into(),
            optional: false,
        };
        match open_include_for_import(&directive, &niri, Some(home), Some(&niri)) {
            IncludeOpen::Ready(p) => assert_eq!(p, extra.canonicalize().unwrap()),
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[test]
    fn import_optional_missing_has_no_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let niri = tmp.path().join("niri");
        fs::create_dir_all(&niri).unwrap();
        let directive = IncludeDirective {
            path: "maybe.kdl".into(),
            optional: true,
        };
        let open = open_include_for_import(&directive, &niri, Some(tmp.path()), Some(&niri));
        assert!(matches!(open, IncludeOpen::Missing { optional: true, .. }));
        assert!(import_skip_warning(&directive, &open).is_none());
    }
}
