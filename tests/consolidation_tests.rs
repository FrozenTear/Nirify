//! Integration tests for rule consolidation feature
//!
//! Tests the full flow from detecting consolidation opportunities
//! to applying merges and verifying the results.

use niri_settings::config::analyze_rules;
use niri_settings::config::models::{
    LayerRule, LayerRuleMatch, Settings, WindowRule, WindowRuleMatch,
};

/// Helper to create a simple window rule with app-id match
fn window_rule(id: u32, app_id: &str, opacity: f32) -> WindowRule {
    WindowRule {
        id,
        name: format!("Rule for {}", app_id),
        matches: vec![WindowRuleMatch {
            app_id: Some(app_id.to_string()),
            ..Default::default()
        }],
        opacity: Some(opacity),
        ..Default::default()
    }
}

/// Helper to create a simple layer rule with namespace match
fn layer_rule(id: u32, namespace: &str, opacity: f32) -> LayerRule {
    LayerRule {
        id,
        name: format!("Rule for {}", namespace),
        matches: vec![LayerRuleMatch {
            namespace: Some(namespace.to_string()),
            ..Default::default()
        }],
        opacity: Some(opacity),
        ..Default::default()
    }
}

// =============================================================================
// DETECTION TESTS
// =============================================================================

#[test]
fn test_detects_window_rule_consolidation_opportunity() {
    let rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.9),
        window_rule(3, "heroic", 0.9),
    ];

    let analysis = analyze_rules(&rules, &[]);

    assert!(analysis.has_suggestions());
    assert_eq!(analysis.window_suggestions.len(), 1);
    assert_eq!(analysis.layer_suggestions.len(), 0);

    let suggestion = &analysis.window_suggestions[0];
    assert_eq!(suggestion.rule_ids.len(), 3);
    assert!(suggestion.rule_ids.contains(&1));
    assert!(suggestion.rule_ids.contains(&2));
    assert!(suggestion.rule_ids.contains(&3));
    assert_eq!(suggestion.merged_pattern, "^(steam|lutris|heroic)$");
}

#[test]
fn test_detects_layer_rule_consolidation_opportunity() {
    let rules = vec![
        layer_rule(1, "waybar", 0.95),
        layer_rule(2, "rofi", 0.95),
        layer_rule(3, "mako", 0.95),
    ];

    let analysis = analyze_rules(&[], &rules);

    assert!(analysis.has_suggestions());
    assert_eq!(analysis.window_suggestions.len(), 0);
    assert_eq!(analysis.layer_suggestions.len(), 1);

    let suggestion = &analysis.layer_suggestions[0];
    assert_eq!(suggestion.rule_ids.len(), 3);
    assert_eq!(suggestion.merged_pattern, "^(waybar|rofi|mako)$");
}

#[test]
fn test_no_consolidation_for_different_settings() {
    let rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.8),  // Different opacity
        window_rule(3, "heroic", 0.95), // Different opacity
    ];

    let analysis = analyze_rules(&rules, &[]);

    assert!(!analysis.has_suggestions());
    assert_eq!(analysis.total_suggestions(), 0);
}

#[test]
fn test_no_consolidation_for_single_rule() {
    let rules = vec![window_rule(1, "steam", 0.9)];

    let analysis = analyze_rules(&rules, &[]);

    assert!(!analysis.has_suggestions());
}

#[test]
fn test_multiple_consolidation_groups() {
    // Two groups: steam/lutris at 0.9, firefox/chrome at 0.8
    let rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.9),
        window_rule(3, "firefox", 0.8),
        window_rule(4, "chrome", 0.8),
    ];

    let analysis = analyze_rules(&rules, &[]);

    assert!(analysis.has_suggestions());
    assert_eq!(analysis.window_suggestions.len(), 2);
    assert_eq!(analysis.total_affected_rules(), 4);
}

#[test]
fn test_ignores_complex_match_criteria() {
    // Rules with additional match criteria should not be consolidated
    let rules = vec![
        WindowRule {
            id: 1,
            matches: vec![WindowRuleMatch {
                app_id: Some("steam".to_string()),
                is_floating: Some(true), // Additional criteria
                ..Default::default()
            }],
            opacity: Some(0.9),
            ..Default::default()
        },
        WindowRule {
            id: 2,
            matches: vec![WindowRuleMatch {
                app_id: Some("lutris".to_string()),
                is_floating: Some(true),
                ..Default::default()
            }],
            opacity: Some(0.9),
            ..Default::default()
        },
    ];

    let analysis = analyze_rules(&rules, &[]);

    // Should not suggest consolidation because rules have additional criteria
    assert!(!analysis.has_suggestions());
}

#[test]
fn test_ignores_multi_match_rules() {
    // Rules with multiple match blocks should not be consolidated
    let rules = vec![
        WindowRule {
            id: 1,
            matches: vec![
                WindowRuleMatch {
                    app_id: Some("steam".to_string()),
                    ..Default::default()
                },
                WindowRuleMatch {
                    app_id: Some("steam-runtime".to_string()),
                    ..Default::default()
                },
            ],
            opacity: Some(0.9),
            ..Default::default()
        },
        window_rule(2, "lutris", 0.9),
    ];

    let analysis = analyze_rules(&rules, &[]);

    // Rule 1 has multiple matches, so only rule 2 is eligible
    // But you need 2+ rules to consolidate, so no suggestions
    assert!(!analysis.has_suggestions());
}

// =============================================================================
// APPLICATION TESTS
// =============================================================================

/// Apply window rule consolidation directly (mimics wizard.rs logic)
fn apply_window_consolidation(
    settings: &mut Settings,
    suggestion: &niri_settings::config::ConsolidationSuggestion,
) {
    let first_id = suggestion.rule_ids.first().copied().unwrap();

    // Update first rule with merged pattern
    if let Some(rule) = settings
        .window_rules
        .rules
        .iter_mut()
        .find(|r| r.id == first_id)
    {
        if !rule.matches.is_empty() {
            rule.matches[0].app_id = Some(suggestion.merged_pattern.clone());
        }
        rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
    }

    // Remove other rules
    let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
    settings
        .window_rules
        .rules
        .retain(|r| !other_ids.contains(&r.id));
}

/// Apply layer rule consolidation directly
fn apply_layer_consolidation(
    settings: &mut Settings,
    suggestion: &niri_settings::config::ConsolidationSuggestion,
) {
    let first_id = suggestion.rule_ids.first().copied().unwrap();

    if let Some(rule) = settings
        .layer_rules
        .rules
        .iter_mut()
        .find(|r| r.id == first_id)
    {
        if !rule.matches.is_empty() {
            rule.matches[0].namespace = Some(suggestion.merged_pattern.clone());
        }
        rule.name = format!("Merged: {}", suggestion.patterns.join(", "));
    }

    let other_ids: Vec<u32> = suggestion.rule_ids.iter().skip(1).copied().collect();
    settings
        .layer_rules
        .rules
        .retain(|r| !other_ids.contains(&r.id));
}

#[test]
fn test_apply_window_rule_consolidation() {
    let mut settings = Settings::default();
    settings.window_rules.rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.9),
        window_rule(3, "heroic", 0.9),
    ];
    settings.window_rules.next_id = 4;

    // Get suggestion
    let analysis = analyze_rules(&settings.window_rules.rules, &[]);
    assert_eq!(analysis.window_suggestions.len(), 1);

    // Apply consolidation
    apply_window_consolidation(&mut settings, &analysis.window_suggestions[0]);

    // Verify result
    assert_eq!(settings.window_rules.rules.len(), 1);
    let merged_rule = &settings.window_rules.rules[0];
    assert_eq!(merged_rule.id, 1); // First rule kept
    assert_eq!(
        merged_rule.matches[0].app_id.as_deref(),
        Some("^(steam|lutris|heroic)$")
    );
    assert_eq!(merged_rule.opacity, Some(0.9));
    assert!(merged_rule.name.contains("Merged"));
}

#[test]
fn test_apply_layer_rule_consolidation() {
    let mut settings = Settings::default();
    settings.layer_rules.rules = vec![layer_rule(1, "waybar", 0.95), layer_rule(2, "rofi", 0.95)];
    settings.layer_rules.next_id = 3;

    let analysis = analyze_rules(&[], &settings.layer_rules.rules);
    assert_eq!(analysis.layer_suggestions.len(), 1);

    apply_layer_consolidation(&mut settings, &analysis.layer_suggestions[0]);

    assert_eq!(settings.layer_rules.rules.len(), 1);
    let merged_rule = &settings.layer_rules.rules[0];
    assert_eq!(
        merged_rule.matches[0].namespace.as_deref(),
        Some("^(waybar|rofi)$")
    );
}

#[test]
fn test_apply_preserves_other_rules() {
    let mut settings = Settings::default();
    settings.window_rules.rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.9),
        window_rule(3, "firefox", 0.8), // Different settings - not in consolidation
    ];

    let analysis = analyze_rules(&settings.window_rules.rules, &[]);
    assert_eq!(analysis.window_suggestions.len(), 1);
    assert_eq!(analysis.window_suggestions[0].rule_ids.len(), 2); // Only steam+lutris

    apply_window_consolidation(&mut settings, &analysis.window_suggestions[0]);

    // Should have 2 rules: merged steam+lutris, and untouched firefox
    assert_eq!(settings.window_rules.rules.len(), 2);

    let merged = settings
        .window_rules
        .rules
        .iter()
        .find(|r| r.id == 1)
        .unwrap();
    assert_eq!(
        merged.matches[0].app_id.as_deref(),
        Some("^(steam|lutris)$")
    );

    let firefox = settings
        .window_rules
        .rules
        .iter()
        .find(|r| r.id == 3)
        .unwrap();
    assert_eq!(firefox.matches[0].app_id.as_deref(), Some("firefox"));
    assert_eq!(firefox.opacity, Some(0.8));
}

#[test]
fn test_apply_multiple_consolidations() {
    let mut settings = Settings::default();
    settings.window_rules.rules = vec![
        window_rule(1, "steam", 0.9),
        window_rule(2, "lutris", 0.9),
        window_rule(3, "firefox", 0.8),
        window_rule(4, "chrome", 0.8),
    ];

    let analysis = analyze_rules(&settings.window_rules.rules, &[]);
    assert_eq!(analysis.window_suggestions.len(), 2);

    // Apply both consolidations
    // Note: In real code, we'd need to re-analyze between applications
    // but for this test, we apply them in order
    for suggestion in &analysis.window_suggestions {
        apply_window_consolidation(&mut settings, suggestion);
    }

    // Should have 2 rules: merged games, merged browsers
    assert_eq!(settings.window_rules.rules.len(), 2);
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_handles_empty_rules() {
    let analysis = analyze_rules(&[], &[]);
    assert!(!analysis.has_suggestions());
    assert_eq!(analysis.total_suggestions(), 0);
    assert_eq!(analysis.total_affected_rules(), 0);
}

#[test]
fn test_merged_pattern_handles_special_characters() {
    // App IDs with dots (common in flatpak)
    let rules = vec![
        window_rule(1, "com.valvesoftware.Steam", 0.9),
        window_rule(2, "net.lutris.Lutris", 0.9),
    ];

    let analysis = analyze_rules(&rules, &[]);

    assert!(analysis.has_suggestions());
    // Note: The pattern includes the dots literally, which works in niri's regex
    assert!(analysis.window_suggestions[0]
        .merged_pattern
        .contains("com.valvesoftware.Steam"));
}

#[test]
fn test_suggestion_metadata() {
    let rules = vec![window_rule(1, "steam", 0.9), window_rule(2, "lutris", 0.9)];

    let analysis = analyze_rules(&rules, &[]);
    let suggestion = &analysis.window_suggestions[0];

    // Check metadata is populated correctly
    assert!(!suggestion.description.is_empty());
    assert_eq!(suggestion.patterns.len(), 2);
    assert!(suggestion.patterns.contains(&"steam".to_string()));
    assert!(suggestion.patterns.contains(&"lutris".to_string()));
    assert!(!suggestion.shared_settings.is_empty());
}

#[test]
fn test_mixed_window_and_layer_suggestions() {
    let window_rules = vec![window_rule(1, "steam", 0.9), window_rule(2, "lutris", 0.9)];

    let layer_rules = vec![layer_rule(1, "waybar", 0.95), layer_rule(2, "rofi", 0.95)];

    let analysis = analyze_rules(&window_rules, &layer_rules);

    assert!(analysis.has_suggestions());
    assert_eq!(analysis.window_suggestions.len(), 1);
    assert_eq!(analysis.layer_suggestions.len(), 1);
    assert_eq!(analysis.total_suggestions(), 2);
    assert_eq!(analysis.total_affected_rules(), 4);
}
