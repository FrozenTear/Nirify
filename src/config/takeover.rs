//! First-run import and launch-time absorb of managed nodes.
//!
//! Two data-loss paths this module closes:
//!
//! 1. **Wizard / first run:** import the user's current `config.kdl` (and
//!    non-Nirify includes) *before* `smart_replace_config` strips managed
//!    nodes, then write those imported settings to `nirify/*.kdl`.
//! 2. **Every launch:** when `config.kdl` still contains top-level managed
//!    nodes (hand-edits), merge them into existing managed settings *before*
//!    stripping, so they are not discarded.
//!
//! # Merge policy (launch absorb)
//!
//! Prefer the safer of the two sources when both exist:
//!
//! - **Scalar / section categories** (appearance, behavior, keyboard, …):
//!   if the corresponding managed file was successfully loaded, keep the
//!   Nirify-managed value. Adopt the stripped value only when that file is
//!   missing (not represented). Failed reads are never overwritten.
//! - **Collections** (outputs, workspaces, window-rules, layer-rules,
//!   keybindings, startup commands, environment variables, spawn-sh):
//!   adopt stripped items whose identity is not already present. Existing
//!   managed items are never replaced.
//!
//! Identity keys:
//! - output / workspace: `name`
//! - keybinding: [`normalized_key_combo`]
//! - window-rule / layer-rule: `matches` + `excludes`
//! - startup command: command argv
//! - environment variable: name
//! - spawn-sh: command string

use std::collections::HashSet;

use anyhow::Context;
use log::{info, warn};

use super::dirty::SettingsCategory;
use super::loader::{
    import_from_kdl_str, import_from_niri_config_with_result, load_settings_with_result,
    ImportResult, LoadResult,
};
use super::models::{
    normalized_key_combo, Keybinding, LayerRule, Settings, StartupCommand, WindowRule,
};
use super::paths::ConfigPaths;
use super::replace::{analyze_config, smart_replace_config, SmartReplaceResult};
use super::storage::{atomic_write, save_dirty, save_settings};
use crate::version::FeatureCompat;

/// Outcome of first-run / wizard setup.
#[derive(Debug)]
pub struct FirstRunSetupResult {
    /// What was imported from the user's existing config.
    pub import: ImportResult,
    /// What `smart_replace_config` did to `config.kdl`.
    pub replace: SmartReplaceResult,
}

/// Outcome of launch-time absorb + replace.
#[derive(Debug)]
pub struct AbsorbResult {
    /// Categories adopted from stripped `config.kdl` nodes.
    pub adopted: HashSet<SettingsCategory>,
    /// What `smart_replace_config` did to `config.kdl`.
    pub replace: SmartReplaceResult,
    /// Settings after merge (managed + adopted).
    pub settings: Settings,
}

/// First-run setup: import → replace → write imported settings.
///
/// Directories are created only *after* import so a leftover `nirify/` include
/// cannot be followed into files we are about to overwrite with defaults.
pub fn first_run_setup(
    paths: &ConfigPaths,
    compat: FeatureCompat,
) -> anyhow::Result<FirstRunSetupResult> {
    // Re-entering the wizard after nirify/ already has files must not
    // re-import the now-stripped config.kdl (that would write defaults
    // over the settings we just saved).
    if paths.managed_dir.exists() && paths.appearance_kdl.exists() {
        info!("First-run setup: nirify/ already initialized, not re-importing");
        paths
            .ensure_directories()
            .with_context(|| "Failed to create Nirify config directories")?;
        let load = load_settings_with_result(paths);
        let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;
        return Ok(FirstRunSetupResult {
            import: ImportResult {
                settings: load.settings,
                imported_sections: load.loaded_files,
                defaulted_sections: load.missing_files,
                warnings: load.warnings,
                includes_processed: 0,
            },
            replace,
        });
    }

    let import = if paths.niri_config.exists() {
        info!(
            "First-run setup: importing settings from {:?}",
            paths.niri_config
        );
        import_from_niri_config_with_result(&paths.niri_config)
    } else {
        info!("First-run setup: no config.kdl yet, using defaults");
        ImportResult {
            settings: Settings::default(),
            imported_sections: Vec::new(),
            defaulted_sections: Vec::new(),
            warnings: Vec::new(),
            includes_processed: 0,
        }
    };

    paths
        .ensure_directories()
        .with_context(|| "Failed to create Nirify config directories")?;

    if !paths.main_kdl.exists() {
        let main_kdl = super::storage::generate_main_kdl(compat);
        atomic_write(&paths.main_kdl, &main_kdl)
            .with_context(|| format!("Failed to create {:?}", paths.main_kdl))?;
        info!("Created {:?}", paths.main_kdl);
    }

    let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;

    save_settings(paths, &import.settings, compat)
        .with_context(|| "Failed to write imported settings to nirify/")?;

    info!(
        "First-run setup complete: {} section(s) imported, {} node(s) replaced",
        import.imported_sections.len(),
        replace.replaced_count
    );

    Ok(FirstRunSetupResult { import, replace })
}

/// Launch-time: merge stripped managed nodes into existing settings, then replace.
///
/// No-op (aside from `smart_replace_config`) when `config.kdl` has no
/// top-level managed nodes to strip.
pub fn absorb_stripped_nodes(
    paths: &ConfigPaths,
    compat: FeatureCompat,
) -> anyhow::Result<AbsorbResult> {
    if !paths.niri_config.exists() {
        let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;
        return Ok(AbsorbResult {
            adopted: HashSet::new(),
            replace,
            settings: Settings::default(),
        });
    }

    let analysis = match analyze_config(&paths.niri_config) {
        Ok(a) => a,
        Err(e) => {
            // Unparseable config: let smart_replace back up and write a minimal file.
            warn!(
                "Launch absorb: could not parse config.kdl ({}); falling back to smart_replace",
                e
            );
            let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;
            return Ok(AbsorbResult {
                adopted: HashSet::new(),
                replace,
                settings: load_settings_with_result(paths).settings,
            });
        }
    };

    if analysis.managed_count == 0 {
        let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;
        return Ok(AbsorbResult {
            adopted: HashSet::new(),
            replace,
            settings: load_settings_with_result(paths).settings,
        });
    }

    let stripped_kdl = analysis.managed_nodes_kdl();
    let stripped = import_from_kdl_str(&stripped_kdl);
    let mut load = load_settings_with_result(paths);
    let managed = std::mem::take(&mut load.settings);
    let (merged, adopted) = merge_stripped_into_managed(managed, &stripped.settings, &load);

    // Persist adopted categories *before* stripping config.kdl so a crash
    // between write and replace cannot lose the hand-edits.
    if !adopted.is_empty() {
        save_dirty(paths, &merged, &adopted, compat, &HashSet::new()).with_context(|| {
            format!(
                "Failed to write {} absorbed categor(ies) to nirify/",
                adopted.len()
            )
        })?;
        info!(
            "Launch absorb: adopted {} categor(ies) from config.kdl: {:?}",
            adopted.len(),
            adopted.iter().map(|c| c.name()).collect::<Vec<_>>()
        );
    }

    let replace = smart_replace_config(&paths.niri_config, &paths.backup_dir)?;

    Ok(AbsorbResult {
        adopted,
        replace,
        settings: merged,
    })
}

/// Merge stripped `config.kdl` nodes into already-loaded managed settings.
///
/// See the module docs for the adopt-if-not-represented policy.
pub fn merge_stripped_into_managed(
    mut managed: Settings,
    stripped: &Settings,
    load: &LoadResult,
) -> (Settings, HashSet<SettingsCategory>) {
    let defaults = Settings::default();
    let mut adopted = HashSet::new();

    macro_rules! adopt_scalar {
        ($cat:expr, $file:expr, $field:ident) => {
            if should_adopt_section(load, $file) && stripped.$field != defaults.$field {
                managed.$field = stripped.$field.clone();
                adopted.insert($cat);
            }
        };
    }

    adopt_scalar!(SettingsCategory::Appearance, "appearance.kdl", appearance);
    adopt_scalar!(SettingsCategory::Behavior, "behavior.kdl", behavior);
    adopt_scalar!(SettingsCategory::Keyboard, "input/keyboard.kdl", keyboard);
    adopt_scalar!(SettingsCategory::Mouse, "input/mouse.kdl", mouse);
    adopt_scalar!(SettingsCategory::Touchpad, "input/touchpad.kdl", touchpad);
    adopt_scalar!(
        SettingsCategory::Trackpoint,
        "input/trackpoint.kdl",
        trackpoint
    );
    adopt_scalar!(
        SettingsCategory::Trackball,
        "input/trackball.kdl",
        trackball
    );
    adopt_scalar!(SettingsCategory::Tablet, "input/tablet.kdl", tablet);
    adopt_scalar!(SettingsCategory::Touch, "input/touch.kdl", touch);
    adopt_scalar!(SettingsCategory::Animations, "animations.kdl", animations);
    adopt_scalar!(SettingsCategory::Cursor, "cursor.kdl", cursor);
    adopt_scalar!(SettingsCategory::Overview, "overview.kdl", overview);
    adopt_scalar!(SettingsCategory::Blur, "blur.kdl", blur);
    adopt_scalar!(
        SettingsCategory::LayoutExtras,
        "advanced/layout-extras.kdl",
        layout_extras
    );
    adopt_scalar!(
        SettingsCategory::Gestures,
        "advanced/gestures.kdl",
        gestures
    );
    adopt_scalar!(SettingsCategory::Debug, "advanced/debug.kdl", debug);
    adopt_scalar!(
        SettingsCategory::SwitchEvents,
        "advanced/switch-events.kdl",
        switch_events
    );
    adopt_scalar!(
        SettingsCategory::RecentWindows,
        "advanced/recent-windows.kdl",
        recent_windows
    );

    // Miscellaneous scalars: adopt the whole struct only when the file is
    // missing. spawn-sh commands are a collection and merge separately.
    if should_adopt_section(load, "advanced/misc.kdl")
        && stripped.miscellaneous != defaults.miscellaneous
    {
        managed.miscellaneous = stripped.miscellaneous.clone();
        adopted.insert(SettingsCategory::Miscellaneous);
    } else if !file_failed(load, "advanced/misc.kdl") {
        let mut added = false;
        for cmd in &stripped.miscellaneous.spawn_sh_at_startup {
            if !managed
                .miscellaneous
                .spawn_sh_at_startup
                .iter()
                .any(|c| c.command == cmd.command)
            {
                let mut owned = cmd.clone();
                owned.id = managed.miscellaneous.spawn_sh_next_id;
                managed.miscellaneous.spawn_sh_next_id += 1;
                managed.miscellaneous.spawn_sh_at_startup.push(owned);
                added = true;
            }
        }
        if added {
            adopted.insert(SettingsCategory::Miscellaneous);
        }
    }

    if !file_failed(load, "outputs.kdl")
        && merge_outputs(&mut managed.outputs.outputs, &stripped.outputs.outputs)
    {
        adopted.insert(SettingsCategory::Outputs);
    }

    if !file_failed(load, "workspaces.kdl")
        && merge_workspaces(&mut managed.workspaces, &stripped.workspaces.workspaces)
    {
        adopted.insert(SettingsCategory::Workspaces);
    }

    if !file_failed(load, "advanced/window-rules.kdl")
        && merge_window_rules(&mut managed.window_rules, &stripped.window_rules.rules)
    {
        adopted.insert(SettingsCategory::WindowRules);
    }

    if !file_failed(load, "advanced/layer-rules.kdl")
        && merge_layer_rules(&mut managed.layer_rules, &stripped.layer_rules.rules)
    {
        adopted.insert(SettingsCategory::LayerRules);
    }

    if !file_failed(load, "keybindings.kdl")
        && merge_keybindings(
            &mut managed.keybindings.bindings,
            &stripped.keybindings.bindings,
        )
    {
        adopted.insert(SettingsCategory::Keybindings);
    }

    if !file_failed(load, "advanced/startup.kdl")
        && merge_startup(&mut managed.startup, &stripped.startup.commands)
    {
        adopted.insert(SettingsCategory::Startup);
    }

    if !file_failed(load, "advanced/environment.kdl")
        && merge_environment(&mut managed.environment, &stripped.environment.variables)
    {
        adopted.insert(SettingsCategory::Environment);
    }

    (managed, adopted)
}

fn file_loaded(load: &LoadResult, filename: &str) -> bool {
    load.loaded_files.iter().any(|f| f == filename)
}

fn file_failed(load: &LoadResult, filename: &str) -> bool {
    load.failed_files.iter().any(|f| f == filename)
}

/// A section is safe to adopt when its managed file is not already loaded
/// and did not fail to read (overwriting a failed file would hide the error).
fn should_adopt_section(load: &LoadResult, filename: &str) -> bool {
    !file_loaded(load, filename) && !file_failed(load, filename)
}

fn merge_outputs(
    managed: &mut Vec<super::models::OutputConfig>,
    stripped: &[super::models::OutputConfig],
) -> bool {
    let mut added = false;
    for output in stripped {
        if output.name.is_empty() {
            continue;
        }
        if !managed.iter().any(|o| o.name == output.name) {
            managed.push(output.clone());
            added = true;
        }
    }
    added
}

fn merge_workspaces(
    managed: &mut super::models::WorkspacesSettings,
    stripped: &[super::models::NamedWorkspace],
) -> bool {
    let mut added = false;
    for ws in stripped {
        if ws.name.is_empty() {
            continue;
        }
        if !managed.workspaces.iter().any(|w| w.name == ws.name) {
            let mut owned = ws.clone();
            owned.id = managed.next_id;
            managed.next_id += 1;
            managed.workspaces.push(owned);
            added = true;
        }
    }
    added
}

fn window_rule_identity(
    rule: &WindowRule,
) -> (
    &[super::models::WindowRuleMatch],
    &[super::models::WindowRuleMatch],
) {
    (&rule.matches, &rule.excludes)
}

fn merge_window_rules(
    managed: &mut super::models::WindowRulesSettings,
    stripped: &[WindowRule],
) -> bool {
    let mut added = false;
    for rule in stripped {
        let ident = window_rule_identity(rule);
        if !managed
            .rules
            .iter()
            .any(|r| window_rule_identity(r) == ident)
        {
            let mut owned = rule.clone();
            owned.id = managed.next_id;
            managed.next_id += 1;
            managed.rules.push(owned);
            added = true;
        }
    }
    added
}

fn layer_rule_identity(
    rule: &LayerRule,
) -> (
    &[super::models::LayerRuleMatch],
    &[super::models::LayerRuleMatch],
) {
    (&rule.matches, &rule.excludes)
}

fn merge_layer_rules(
    managed: &mut super::models::LayerRulesSettings,
    stripped: &[LayerRule],
) -> bool {
    let mut added = false;
    for rule in stripped {
        let ident = layer_rule_identity(rule);
        if !managed
            .rules
            .iter()
            .any(|r| layer_rule_identity(r) == ident)
        {
            let mut owned = rule.clone();
            owned.id = managed.next_id;
            managed.next_id += 1;
            managed.rules.push(owned);
            added = true;
        }
    }
    added
}

fn merge_keybindings(managed: &mut Vec<Keybinding>, stripped: &[Keybinding]) -> bool {
    let mut added = false;
    let mut next_id = managed
        .iter()
        .map(|b| b.id)
        .max()
        .map(|id| id + 1)
        .unwrap_or(0);
    for binding in stripped {
        if binding.key_combo.is_empty() {
            continue;
        }
        let norm = normalized_key_combo(&binding.key_combo);
        if !managed
            .iter()
            .any(|b| normalized_key_combo(&b.key_combo) == norm)
        {
            let mut owned = binding.clone();
            owned.id = next_id;
            next_id += 1;
            managed.push(owned);
            added = true;
        }
    }
    added
}

fn merge_startup(
    managed: &mut super::models::StartupSettings,
    stripped: &[StartupCommand],
) -> bool {
    let mut added = false;
    for cmd in stripped {
        if cmd.command.iter().all(|s| s.is_empty()) {
            continue;
        }
        if !managed.commands.iter().any(|c| c.command == cmd.command) {
            let mut owned = cmd.clone();
            owned.id = managed.next_id;
            managed.next_id += 1;
            managed.commands.push(owned);
            added = true;
        }
    }
    added
}

fn merge_environment(
    managed: &mut super::models::EnvironmentSettings,
    stripped: &[super::models::EnvironmentVariable],
) -> bool {
    let mut added = false;
    for var in stripped {
        if var.name.is_empty() {
            continue;
        }
        if !managed.variables.iter().any(|v| v.name == var.name) {
            let mut owned = var.clone();
            owned.id = managed.next_id;
            managed.next_id += 1;
            managed.variables.push(owned);
            added = true;
        }
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{OutputConfig, WindowRuleMatch};
    use std::fs;
    use tempfile::tempdir;

    fn test_paths(base: &std::path::Path) -> ConfigPaths {
        let managed_dir = base.join("nirify");
        let input_dir = managed_dir.join("input");
        let advanced_dir = managed_dir.join("advanced");
        ConfigPaths {
            niri_config: base.join("config.kdl"),
            managed_dir: managed_dir.clone(),
            input_dir: input_dir.clone(),
            advanced_dir: advanced_dir.clone(),
            backup_dir: base.join(".nirify-backups"),
            main_kdl: managed_dir.join("main.kdl"),
            appearance_kdl: managed_dir.join("appearance.kdl"),
            behavior_kdl: managed_dir.join("behavior.kdl"),
            keyboard_kdl: input_dir.join("keyboard.kdl"),
            mouse_kdl: input_dir.join("mouse.kdl"),
            touchpad_kdl: input_dir.join("touchpad.kdl"),
            trackpoint_kdl: input_dir.join("trackpoint.kdl"),
            trackball_kdl: input_dir.join("trackball.kdl"),
            tablet_kdl: input_dir.join("tablet.kdl"),
            touch_kdl: input_dir.join("touch.kdl"),
            outputs_kdl: managed_dir.join("outputs.kdl"),
            animations_kdl: managed_dir.join("animations.kdl"),
            cursor_kdl: managed_dir.join("cursor.kdl"),
            overview_kdl: managed_dir.join("overview.kdl"),
            workspaces_kdl: managed_dir.join("workspaces.kdl"),
            keybindings_kdl: managed_dir.join("keybindings.kdl"),
            layout_extras_kdl: advanced_dir.join("layout-extras.kdl"),
            gestures_kdl: advanced_dir.join("gestures.kdl"),
            layer_rules_kdl: advanced_dir.join("layer-rules.kdl"),
            window_rules_kdl: advanced_dir.join("window-rules.kdl"),
            misc_kdl: advanced_dir.join("misc.kdl"),
            startup_kdl: advanced_dir.join("startup.kdl"),
            environment_kdl: advanced_dir.join("environment.kdl"),
            debug_kdl: advanced_dir.join("debug.kdl"),
            switch_events_kdl: advanced_dir.join("switch-events.kdl"),
            recent_windows_kdl: advanced_dir.join("recent-windows.kdl"),
            preferences_kdl: advanced_dir.join("preferences.kdl"),
        }
    }

    #[test]
    fn first_run_imports_before_replace_and_writes_imported_settings() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());

        fs::write(
            &paths.niri_config,
            r#"
layout { gaps 24 }
output "DP-1" {
    position x=0 y=0
}
binds {
    Mod+Return { spawn "alacritty"; }
}
custom-keep "yes"
"#,
        )
        .unwrap();

        assert!(!paths.managed_dir.exists());

        let result =
            first_run_setup(&paths, FeatureCompat::all_enabled()).expect("first_run_setup");

        assert!(
            result
                .import
                .imported_sections
                .iter()
                .any(|s| s == "appearance"),
            "expected appearance imported, got {:?}",
            result.import.imported_sections
        );
        assert!(result
            .import
            .imported_sections
            .iter()
            .any(|s| s.starts_with("outputs")));
        assert!(result
            .import
            .imported_sections
            .iter()
            .any(|s| s.starts_with("keybindings")));

        // Imported values, not defaults, landed in managed files.
        let loaded = load_settings_with_result(&paths).settings;
        assert_eq!(loaded.appearance.gaps, 24.0);
        assert_eq!(loaded.outputs.outputs.len(), 1);
        assert_eq!(loaded.outputs.outputs[0].name, "DP-1");
        assert_eq!(loaded.outputs.outputs[0].position, Some((0, 0)));
        assert!(
            loaded
                .keybindings
                .bindings
                .iter()
                .any(|b| b.key_combo == "Mod+Return"),
            "expected imported bind, got {:?}",
            loaded.keybindings.bindings
        );

        // config.kdl was stripped of managed nodes but custom content + include remain.
        let rewritten = fs::read_to_string(&paths.niri_config).unwrap();
        assert!(rewritten.contains("include \"nirify/main.kdl\""));
        assert!(rewritten.contains("custom-keep"));
        assert!(!rewritten.contains("output \"DP-1\""));
        assert!(!rewritten.contains("Mod+Return"));
        assert!(!result.replace.backup_path.as_os_str().is_empty());
        assert!(result.replace.backup_path.exists());

        // Re-running setup must not overwrite imported files with defaults.
        let again =
            first_run_setup(&paths, FeatureCompat::all_enabled()).expect("re-run first_run_setup");
        let reloaded = load_settings_with_result(&paths).settings;
        assert_eq!(reloaded.appearance.gaps, 24.0);
        assert_eq!(reloaded.outputs.outputs.len(), 1);
        assert!(again.import.settings.appearance.gaps - 24.0 < f32::EPSILON);
    }

    #[test]
    fn absorb_adopts_stripped_output_and_bind_without_clobbering_managed_layout() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        paths.ensure_directories().unwrap();

        // Existing Nirify-managed state: gaps 16 and one output.
        let mut existing = Settings::default();
        existing.appearance.gaps = 16.0;
        existing.outputs.outputs.push(OutputConfig {
            name: "eDP-1".to_string(),
            position: Some((0, 0)),
            ..Default::default()
        });
        save_settings(&paths, &existing, FeatureCompat::all_enabled()).unwrap();

        // Hand-edits living only in config.kdl: a new output, a bind, and a
        // stale layout that must NOT clobber the managed gaps.
        fs::write(
            &paths.niri_config,
            r#"
layout { gaps 99 }
output "HDMI-A-1" {
    position x=1920 y=0
}
binds {
    Mod+Q { close-window; }
}
include "nirify/main.kdl"
"#,
        )
        .unwrap();

        let result = absorb_stripped_nodes(&paths, FeatureCompat::all_enabled()).expect("absorb");

        assert!(
            result.adopted.contains(&SettingsCategory::Outputs),
            "expected Outputs adopted, got {:?}",
            result.adopted
        );
        assert!(
            result.adopted.contains(&SettingsCategory::Keybindings),
            "expected Keybindings adopted, got {:?}",
            result.adopted
        );
        assert!(
            !result.adopted.contains(&SettingsCategory::Appearance),
            "must not clobber managed appearance with stale layout: {:?}",
            result.adopted
        );

        let loaded = load_settings_with_result(&paths).settings;
        assert_eq!(
            loaded.appearance.gaps, 16.0,
            "managed gaps must win over stale config.kdl layout"
        );
        assert!(
            loaded.outputs.outputs.iter().any(|o| o.name == "eDP-1"),
            "existing managed output kept"
        );
        let hdmi = loaded
            .outputs
            .outputs
            .iter()
            .find(|o| o.name == "HDMI-A-1")
            .expect("stripped output adopted");
        assert_eq!(hdmi.position, Some((1920, 0)));
        assert!(
            loaded
                .keybindings
                .bindings
                .iter()
                .any(|b| normalized_key_combo(&b.key_combo) == normalized_key_combo("Mod+Q")),
            "stripped bind adopted, got {:?}",
            loaded.keybindings.bindings
        );

        let rewritten = fs::read_to_string(&paths.niri_config).unwrap();
        assert!(!rewritten.contains("HDMI-A-1"));
        assert!(!rewritten.contains("Mod+Q"));
        assert!(rewritten.contains("include \"nirify/main.kdl\""));
    }

    #[test]
    fn absorb_does_not_duplicate_existing_output_or_bind() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        paths.ensure_directories().unwrap();

        let mut existing = Settings::default();
        existing.outputs.outputs.push(OutputConfig {
            name: "DP-1".to_string(),
            position: Some((10, 10)),
            ..Default::default()
        });
        existing.keybindings.bindings.push(Keybinding {
            id: 0,
            key_combo: "Mod+Return".to_string(),
            ..Default::default()
        });
        save_settings(&paths, &existing, FeatureCompat::all_enabled()).unwrap();

        fs::write(
            &paths.niri_config,
            r#"
output "DP-1" {
    position x=0 y=0
}
binds {
    Mod+Return { spawn "kitty"; }
}
include "nirify/main.kdl"
"#,
        )
        .unwrap();

        let result = absorb_stripped_nodes(&paths, FeatureCompat::all_enabled()).expect("absorb");
        assert!(!result.adopted.contains(&SettingsCategory::Outputs));
        assert!(!result.adopted.contains(&SettingsCategory::Keybindings));

        let loaded = load_settings_with_result(&paths).settings;
        assert_eq!(loaded.outputs.outputs.len(), 1);
        assert_eq!(loaded.outputs.outputs[0].position, Some((10, 10)));
        assert_eq!(loaded.keybindings.bindings.len(), 1);
    }

    #[test]
    fn merge_adopts_window_rule_by_match_identity() {
        let mut managed = Settings::default();
        managed.window_rules.rules.push(WindowRule {
            id: 0,
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".into()),
                ..Default::default()
            }],
            ..Default::default()
        });
        managed.window_rules.next_id = 1;

        let mut stripped = Settings::default();
        stripped.window_rules.rules.push(WindowRule {
            id: 0,
            matches: vec![WindowRuleMatch {
                app_id: Some("alacritty".into()),
                ..Default::default()
            }],
            ..Default::default()
        });
        // Duplicate of the managed firefox rule — must not be added again.
        stripped.window_rules.rules.push(WindowRule {
            id: 1,
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".into()),
                ..Default::default()
            }],
            ..Default::default()
        });

        let load = LoadResult::default();
        let (merged, adopted) = merge_stripped_into_managed(managed, &stripped, &load);
        assert!(adopted.contains(&SettingsCategory::WindowRules));
        assert_eq!(merged.window_rules.rules.len(), 2);
        assert!(merged.window_rules.rules.iter().any(|r| r
            .matches
            .iter()
            .any(|m| m.app_id.as_deref() == Some("alacritty"))));
    }

    #[test]
    fn merge_skips_failed_category() {
        let managed = Settings::default();
        let mut stripped = Settings::default();
        stripped.appearance.gaps = 32.0;

        let mut load = LoadResult::default();
        load.failed_files.push("appearance.kdl".to_string());

        let (merged, adopted) = merge_stripped_into_managed(managed, &stripped, &load);
        assert!(!adopted.contains(&SettingsCategory::Appearance));
        assert_eq!(merged.appearance.gaps, Settings::default().appearance.gaps);
    }
}
