//! Rule consolidation detection and suggestions
//!
//! Analyzes imported rules to find consolidation opportunities where multiple
//! rules with the same settings could be merged into a single rule with a
//! regex pattern.

use super::models::{LayerRule, WindowRule};
use std::collections::HashMap;

/// A suggestion to consolidate multiple rules into one
#[derive(Debug, Clone)]
pub struct ConsolidationSuggestion {
    /// Human-readable description of the suggestion
    pub description: String,
    /// IDs of rules that could be merged
    pub rule_ids: Vec<u32>,
    /// Names/app-ids that would be combined
    pub patterns: Vec<String>,
    /// The suggested merged regex pattern
    pub merged_pattern: String,
    /// What settings these rules share (e.g., "opacity 0.9")
    pub shared_settings: String,
}

/// Result of analyzing rules for consolidation opportunities
#[derive(Debug, Clone, Default)]
pub struct ConsolidationAnalysis {
    /// Suggestions for window rules
    pub window_suggestions: Vec<ConsolidationSuggestion>,
    /// Suggestions for layer rules
    pub layer_suggestions: Vec<ConsolidationSuggestion>,
}

impl ConsolidationAnalysis {
    /// Returns true if there are any consolidation suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.window_suggestions.is_empty() || !self.layer_suggestions.is_empty()
    }

    /// Total number of suggestions
    pub fn total_suggestions(&self) -> usize {
        self.window_suggestions.len() + self.layer_suggestions.len()
    }

    /// Total number of rules that could be consolidated
    pub fn total_affected_rules(&self) -> usize {
        self.window_suggestions.iter().map(|s| s.rule_ids.len()).sum::<usize>()
            + self.layer_suggestions.iter().map(|s| s.rule_ids.len()).sum::<usize>()
    }
}

/// Analyze window and layer rules for consolidation opportunities
pub fn analyze_rules(window_rules: &[WindowRule], layer_rules: &[LayerRule]) -> ConsolidationAnalysis {
    ConsolidationAnalysis {
        window_suggestions: analyze_window_rules(window_rules),
        layer_suggestions: analyze_layer_rules(layer_rules),
    }
}

/// Analyze window rules for consolidation opportunities
fn analyze_window_rules(rules: &[WindowRule]) -> Vec<ConsolidationSuggestion> {
    // Group rules by their "effect signature" (everything except id, name, matches)
    let mut effect_groups: HashMap<WindowRuleEffectKey, Vec<&WindowRule>> = HashMap::new();

    for rule in rules {
        // Only consider rules that have simple app-id matches
        if !is_simple_app_id_rule(rule) {
            continue;
        }

        let key = WindowRuleEffectKey::from(rule);
        effect_groups.entry(key).or_default().push(rule);
    }

    // Generate suggestions for groups with 2+ rules
    effect_groups
        .into_iter()
        .filter(|(_, group)| group.len() >= 2)
        .map(|(key, group)| {
            let patterns: Vec<String> = group
                .iter()
                .filter_map(|r| {
                    r.matches.first().and_then(|m| m.app_id.clone())
                })
                .collect();

            let merged = create_merged_regex(&patterns);

            ConsolidationSuggestion {
                description: format!(
                    "{} window rules with same settings",
                    group.len()
                ),
                rule_ids: group.iter().map(|r| r.id).collect(),
                patterns: patterns.clone(),
                merged_pattern: merged,
                shared_settings: key.describe(),
            }
        })
        .collect()
}

/// Analyze layer rules for consolidation opportunities
fn analyze_layer_rules(rules: &[LayerRule]) -> Vec<ConsolidationSuggestion> {
    // Group rules by their "effect signature"
    let mut effect_groups: HashMap<LayerRuleEffectKey, Vec<&LayerRule>> = HashMap::new();

    for rule in rules {
        // Only consider rules that have simple namespace matches
        if !is_simple_namespace_rule(rule) {
            continue;
        }

        let key = LayerRuleEffectKey::from(rule);
        effect_groups.entry(key).or_default().push(rule);
    }

    // Generate suggestions for groups with 2+ rules
    effect_groups
        .into_iter()
        .filter(|(_, group)| group.len() >= 2)
        .map(|(key, group)| {
            let patterns: Vec<String> = group
                .iter()
                .filter_map(|r| {
                    r.matches.first().and_then(|m| m.namespace.clone())
                })
                .collect();

            let merged = create_merged_regex(&patterns);

            ConsolidationSuggestion {
                description: format!(
                    "{} layer rules with same settings",
                    group.len()
                ),
                rule_ids: group.iter().map(|r| r.id).collect(),
                patterns: patterns.clone(),
                merged_pattern: merged,
                shared_settings: key.describe(),
            }
        })
        .collect()
}

/// Check if a window rule has only a simple app-id match (no other criteria)
fn is_simple_app_id_rule(rule: &WindowRule) -> bool {
    // Must have exactly one match with only app_id set
    if rule.matches.len() != 1 {
        return false;
    }

    let m = &rule.matches[0];
    m.app_id.is_some()
        && m.title.is_none()
        && m.is_floating.is_none()
        && m.is_active.is_none()
        && m.is_focused.is_none()
        && m.is_active_in_column.is_none()
        && m.is_window_cast_target.is_none()
        && m.is_urgent.is_none()
        && m.at_startup.is_none()
}

/// Check if a layer rule has only a simple namespace match (no other criteria)
fn is_simple_namespace_rule(rule: &LayerRule) -> bool {
    if rule.matches.len() != 1 {
        return false;
    }

    let m = &rule.matches[0];
    m.namespace.is_some() && m.at_startup.is_none()
}

/// Create a merged regex pattern from multiple simple patterns
fn create_merged_regex(patterns: &[String]) -> String {
    if patterns.is_empty() {
        return String::new();
    }

    // Check if patterns are already regexes or simple strings
    let are_simple: Vec<bool> = patterns
        .iter()
        .map(|p| !p.contains('^') && !p.contains('$') && !p.contains('|') && !p.contains('('))
        .collect();

    if are_simple.iter().all(|&s| s) {
        // All simple patterns - create a clean alternation
        format!("^({})$", patterns.join("|"))
    } else {
        // Some are already regexes - just combine with alternation
        // Strip existing anchors and combine
        let cleaned: Vec<String> = patterns
            .iter()
            .map(|p| {
                p.trim_start_matches('^')
                    .trim_end_matches('$')
                    .to_string()
            })
            .collect();
        format!("^({})$", cleaned.join("|"))
    }
}

/// Key for grouping window rules by their effect (non-match settings)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct WindowRuleEffectKey {
    open_behavior: u8,
    opacity: Option<u32>, // f32 as bits for hashing
    block_out_from_screencast: bool,
    corner_radius: Option<i32>,
    open_focused: Option<bool>,
    open_on_output: Option<String>,
    open_on_workspace: Option<String>,
    default_column_width: Option<u32>,
    default_window_height: Option<u32>,
    open_maximized_to_edges: Option<bool>,
    scroll_factor: Option<u64>,
    draw_border_with_background: Option<bool>,
    min_width: Option<i32>,
    max_width: Option<i32>,
    min_height: Option<i32>,
    max_height: Option<i32>,
}

impl From<&WindowRule> for WindowRuleEffectKey {
    fn from(rule: &WindowRule) -> Self {
        Self {
            open_behavior: rule.open_behavior as u8,
            opacity: rule.opacity.map(|f| f.to_bits()),
            block_out_from_screencast: rule.block_out_from_screencast,
            corner_radius: rule.corner_radius,
            open_focused: rule.open_focused,
            open_on_output: rule.open_on_output.clone(),
            open_on_workspace: rule.open_on_workspace.clone(),
            default_column_width: rule.default_column_width.map(|f| f.to_bits()),
            default_window_height: rule.default_window_height.map(|f| f.to_bits()),
            open_maximized_to_edges: rule.open_maximized_to_edges,
            scroll_factor: rule.scroll_factor.map(|f| f.to_bits()),
            draw_border_with_background: rule.draw_border_with_background,
            min_width: rule.min_width,
            max_width: rule.max_width,
            min_height: rule.min_height,
            max_height: rule.max_height,
        }
    }
}

impl WindowRuleEffectKey {
    /// Generate a human-readable description of the shared settings
    fn describe(&self) -> String {
        let mut parts = Vec::new();

        if self.open_behavior != 0 {
            let behavior = match self.open_behavior {
                1 => "open-maximized",
                2 => "open-fullscreen",
                3 => "open-floating",
                _ => "normal",
            };
            parts.push(behavior.to_string());
        }

        if let Some(bits) = self.opacity {
            let opacity = f32::from_bits(bits);
            parts.push(format!("opacity {:.2}", opacity));
        }

        if self.block_out_from_screencast {
            parts.push("block-out-from".to_string());
        }

        if let Some(radius) = self.corner_radius {
            parts.push(format!("corner-radius {}", radius));
        }

        if let Some(ref output) = self.open_on_output {
            parts.push(format!("on output \"{}\"", output));
        }

        if let Some(ref workspace) = self.open_on_workspace {
            parts.push(format!("on workspace \"{}\"", workspace));
        }

        if parts.is_empty() {
            "default settings".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Key for grouping layer rules by their effect
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LayerRuleEffectKey {
    block_out_from: Option<u8>,
    opacity: Option<u32>,
    geometry_corner_radius: Option<i32>,
    place_within_backdrop: bool,
    baba_is_float: bool,
}

impl From<&LayerRule> for LayerRuleEffectKey {
    fn from(rule: &LayerRule) -> Self {
        Self {
            block_out_from: rule.block_out_from.map(|b| b as u8),
            opacity: rule.opacity.map(|f| f.to_bits()),
            geometry_corner_radius: rule.geometry_corner_radius,
            place_within_backdrop: rule.place_within_backdrop,
            baba_is_float: rule.baba_is_float,
        }
    }
}

impl LayerRuleEffectKey {
    fn describe(&self) -> String {
        let mut parts = Vec::new();

        if let Some(bits) = self.opacity {
            let opacity = f32::from_bits(bits);
            parts.push(format!("opacity {:.2}", opacity));
        }

        if self.block_out_from.is_some() {
            parts.push("block-out-from".to_string());
        }

        if let Some(radius) = self.geometry_corner_radius {
            parts.push(format!("corner-radius {}", radius));
        }

        if self.place_within_backdrop {
            parts.push("place-within-backdrop".to_string());
        }

        if self.baba_is_float {
            parts.push("baba-is-float".to_string());
        }

        if parts.is_empty() {
            "default settings".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::WindowRuleMatch;

    #[test]
    fn test_create_merged_regex_simple() {
        let patterns = vec![
            "steam".to_string(),
            "lutris".to_string(),
            "heroic".to_string(),
        ];
        assert_eq!(create_merged_regex(&patterns), "^(steam|lutris|heroic)$");
    }

    #[test]
    fn test_create_merged_regex_existing_anchors() {
        let patterns = vec![
            "^steam$".to_string(),
            "^lutris$".to_string(),
        ];
        assert_eq!(create_merged_regex(&patterns), "^(steam|lutris)$");
    }

    #[test]
    fn test_simple_app_id_rule() {
        let rule = WindowRule {
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(is_simple_app_id_rule(&rule));
    }

    #[test]
    fn test_complex_rule_not_simple() {
        let rule = WindowRule {
            matches: vec![WindowRuleMatch {
                app_id: Some("firefox".to_string()),
                is_floating: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(!is_simple_app_id_rule(&rule));
    }

    #[test]
    fn test_analyze_finds_consolidation() {
        let rules = vec![
            WindowRule {
                id: 1,
                matches: vec![WindowRuleMatch {
                    app_id: Some("steam".to_string()),
                    ..Default::default()
                }],
                opacity: Some(0.9),
                ..Default::default()
            },
            WindowRule {
                id: 2,
                matches: vec![WindowRuleMatch {
                    app_id: Some("lutris".to_string()),
                    ..Default::default()
                }],
                opacity: Some(0.9),
                ..Default::default()
            },
            WindowRule {
                id: 3,
                matches: vec![WindowRuleMatch {
                    app_id: Some("heroic".to_string()),
                    ..Default::default()
                }],
                opacity: Some(0.9),
                ..Default::default()
            },
        ];

        let analysis = analyze_rules(&rules, &[]);
        assert_eq!(analysis.window_suggestions.len(), 1);
        assert_eq!(analysis.window_suggestions[0].rule_ids.len(), 3);
        assert_eq!(
            analysis.window_suggestions[0].merged_pattern,
            "^(steam|lutris|heroic)$"
        );
    }

    #[test]
    fn test_no_consolidation_different_settings() {
        let rules = vec![
            WindowRule {
                id: 1,
                matches: vec![WindowRuleMatch {
                    app_id: Some("steam".to_string()),
                    ..Default::default()
                }],
                opacity: Some(0.9),
                ..Default::default()
            },
            WindowRule {
                id: 2,
                matches: vec![WindowRuleMatch {
                    app_id: Some("lutris".to_string()),
                    ..Default::default()
                }],
                opacity: Some(0.8), // Different opacity
                ..Default::default()
            },
        ];

        let analysis = analyze_rules(&rules, &[]);
        assert!(analysis.window_suggestions.is_empty());
    }
}
