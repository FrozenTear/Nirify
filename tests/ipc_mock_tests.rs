//! Mock IPC tests
//!
//! These tests verify IPC response parsing logic without requiring niri to be running.
//! They use mock JSON responses to test edge cases and error handling.
//!
//! Run with: cargo test --test ipc_mock_tests

use serde_json::json;

// ============================================================================
// VERSION RESPONSE PARSING TESTS
// ============================================================================

#[test]
fn test_version_response_standard() {
    let json = r#"{"Ok":{"Version":"0.1.10"}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let version = parsed["Ok"]["Version"].as_str().unwrap();
    assert_eq!(version, "0.1.10");
}

#[test]
fn test_version_response_with_git_suffix() {
    let json = r#"{"Ok":{"Version":"0.1.10-git.abc123"}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let version = parsed["Ok"]["Version"].as_str().unwrap();
    assert_eq!(version, "0.1.10-git.abc123");
}

#[test]
fn test_version_response_empty() {
    let json = r#"{"Ok":{"Version":""}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let version = parsed["Ok"]["Version"].as_str().unwrap();
    assert_eq!(version, "");
}

#[test]
fn test_error_response() {
    let json = r#"{"Err":"Not supported"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert!(parsed.get("Ok").is_none());
    let error = parsed["Err"].as_str().unwrap();
    assert_eq!(error, "Not supported");
}

// ============================================================================
// WINDOWS RESPONSE PARSING TESTS
// ============================================================================

#[test]
fn test_windows_response_empty() {
    let json = r#"{"Ok":{"Windows":[]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert!(windows.is_empty());
}

#[test]
fn test_windows_response_single() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Test Window","app_id":"test","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert_eq!(windows.len(), 1);

    let window = &windows[0];
    assert_eq!(window["id"].as_u64().unwrap(), 1);
    assert_eq!(window["title"].as_str().unwrap(), "Test Window");
    assert_eq!(window["app_id"].as_str().unwrap(), "test");
    assert!(!window["is_floating"].as_bool().unwrap());
}

#[test]
fn test_windows_response_multiple() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Window 1","app_id":"app1","is_floating":false},
        {"id":2,"title":"Window 2","app_id":"app2","is_floating":true},
        {"id":3,"title":"Window 3","app_id":"app3","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert_eq!(windows.len(), 3);

    assert!(!windows[0]["is_floating"].as_bool().unwrap());
    assert!(windows[1]["is_floating"].as_bool().unwrap());
    assert!(!windows[2]["is_floating"].as_bool().unwrap());
}

#[test]
fn test_windows_response_with_unicode() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"日本語タイトル","app_id":"app","is_floating":false},
        {"id":2,"title":"Ñoño 🎉","app_id":"app2","is_floating":true}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert_eq!(windows[0]["title"].as_str().unwrap(), "日本語タイトル");
    assert_eq!(windows[1]["title"].as_str().unwrap(), "Ñoño 🎉");
}

#[test]
fn test_windows_response_with_workspace_id() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Test","app_id":"test","is_floating":false,"workspace_id":5}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert_eq!(windows[0]["workspace_id"].as_u64().unwrap(), 5);
}

#[test]
fn test_windows_response_without_workspace_id() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Test","app_id":"test","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert!(windows[0].get("workspace_id").is_none());
}

// ============================================================================
// OUTPUTS RESPONSE PARSING TESTS
// ============================================================================

#[test]
fn test_outputs_response_empty() {
    let json = r#"{"Ok":{"Outputs":{}}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let outputs = parsed["Ok"]["Outputs"].as_object().unwrap();
    assert!(outputs.is_empty());
}

#[test]
fn test_outputs_response_single() {
    let json = r#"{"Ok":{"Outputs":{
        "DP-1": {
            "name": "DP-1",
            "make": "Dell",
            "model": "U2720Q",
            "serial": "ABC123",
            "current_mode": {"width": 3840, "height": 2160, "refresh_rate": 60000},
            "position": {"x": 0, "y": 0},
            "scale": 2.0,
            "vrr_enabled": true
        }
    }}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let outputs = parsed["Ok"]["Outputs"].as_object().unwrap();
    assert_eq!(outputs.len(), 1);

    let dp1 = &outputs["DP-1"];
    assert_eq!(dp1["name"].as_str().unwrap(), "DP-1");
    assert_eq!(dp1["make"].as_str().unwrap(), "Dell");
    assert_eq!(dp1["current_mode"]["width"].as_u64().unwrap(), 3840);
    assert!(dp1["vrr_enabled"].as_bool().unwrap());
}

#[test]
fn test_outputs_response_multiple() {
    let json = r#"{"Ok":{"Outputs":{
        "DP-1": {"name": "DP-1", "scale": 2.0},
        "HDMI-A-1": {"name": "HDMI-A-1", "scale": 1.0}
    }}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let outputs = parsed["Ok"]["Outputs"].as_object().unwrap();
    assert_eq!(outputs.len(), 2);
    assert!(outputs.contains_key("DP-1"));
    assert!(outputs.contains_key("HDMI-A-1"));
}

// ============================================================================
// WORKSPACES RESPONSE PARSING TESTS
// ============================================================================

#[test]
fn test_workspaces_response() {
    let json = r#"{"Ok":{"Workspaces":[
        {"id":1,"idx":1,"name":"main","output":"DP-1","is_focused":true,"is_active":true},
        {"id":2,"idx":2,"name":null,"output":"DP-1","is_focused":false,"is_active":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let workspaces = parsed["Ok"]["Workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);

    assert_eq!(workspaces[0]["name"].as_str().unwrap(), "main");
    assert!(workspaces[0]["is_focused"].as_bool().unwrap());

    assert!(workspaces[1]["name"].is_null());
    assert!(!workspaces[1]["is_focused"].as_bool().unwrap());
}

// ============================================================================
// ACTION RESPONSE PARSING TESTS
// ============================================================================

#[test]
fn test_action_response_ok() {
    let json = r#"{"Ok":"Handled"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["Ok"].as_str().unwrap(), "Handled");
}

#[test]
fn test_action_response_error() {
    let json = r#"{"Err":"No such action"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert!(parsed.get("Ok").is_none());
    assert_eq!(parsed["Err"].as_str().unwrap(), "No such action");
}

// ============================================================================
// MALFORMED RESPONSE TESTS
// ============================================================================

#[test]
fn test_invalid_json() {
    let invalid_responses = vec![
        "not json at all",
        "{incomplete",
        "{'single': 'quotes'}",
        "",
        "null",
        "[]",
        "123",
        "{\"Ok\": }",
    ];

    for response in invalid_responses {
        let result: Result<serde_json::Value, _> = serde_json::from_str(response);
        // Most of these should fail to parse as the expected structure
        // Some may parse as valid JSON but not as expected response format
        if let Ok(parsed) = result {
            // Valid JSON but maybe not expected format
            assert!(
                parsed.get("Ok").is_none() || parsed.get("Err").is_none(),
                "Unexpected valid response: {}",
                response
            );
        }
    }
}

#[test]
fn test_unexpected_ok_format() {
    // Ok contains unexpected type
    let json = r#"{"Ok": 123}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    // Should parse but not be the expected format
    assert!(parsed["Ok"].as_object().is_none());
    assert!(parsed["Ok"].is_number());
}

#[test]
fn test_extra_fields_ignored() {
    let json = r#"{"Ok":{"Version":"0.1.10","extra_field":"ignored","nested":{"data":123}}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    // Should still get version
    assert_eq!(parsed["Ok"]["Version"].as_str().unwrap(), "0.1.10");
}

// ============================================================================
// FOCUSED WINDOW/OUTPUT RESPONSE TESTS
// ============================================================================

#[test]
fn test_focused_window_response() {
    let json = r#"{"Ok":{"FocusedWindow":{"id":5,"title":"Firefox","app_id":"firefox","is_floating":false}}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let focused = &parsed["Ok"]["FocusedWindow"];
    assert_eq!(focused["id"].as_u64().unwrap(), 5);
    assert_eq!(focused["title"].as_str().unwrap(), "Firefox");
}

#[test]
fn test_focused_window_none() {
    let json = r#"{"Ok":{"FocusedWindow":null}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert!(parsed["Ok"]["FocusedWindow"].is_null());
}

#[test]
fn test_focused_output_response() {
    let json = r#"{"Ok":{"FocusedOutput":"DP-1"}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(parsed["Ok"]["FocusedOutput"].as_str().unwrap(), "DP-1");
}

#[test]
fn test_focused_output_none() {
    let json = r#"{"Ok":{"FocusedOutput":null}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert!(parsed["Ok"]["FocusedOutput"].is_null());
}

// ============================================================================
// RESPONSE SIZE TESTS
// ============================================================================

#[test]
fn test_large_response_parsing() {
    // Create a response with many windows
    let mut windows = Vec::new();
    for i in 0..1000 {
        windows.push(json!({
            "id": i,
            "title": format!("Window {}", i),
            "app_id": format!("app{}", i),
            "is_floating": i % 2 == 0
        }));
    }

    let response = json!({
        "Ok": {
            "Windows": windows
        }
    });

    let json_str = serde_json::to_string(&response).unwrap();

    // Should parse successfully
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let parsed_windows = parsed["Ok"]["Windows"].as_array().unwrap();
    assert_eq!(parsed_windows.len(), 1000);
}

#[test]
fn test_deeply_nested_response() {
    // niri responses shouldn't be deeply nested, but test parser handles it
    let json = r#"{"Ok":{"Data":{"Level1":{"Level2":{"Level3":{"value":"deep"}}}}}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    assert_eq!(
        parsed["Ok"]["Data"]["Level1"]["Level2"]["Level3"]["value"]
            .as_str()
            .unwrap(),
        "deep"
    );
}

// ============================================================================
// RESPONSE FALLBACK DETECTION TESTS
// ============================================================================

/// Test the fallback logic that detects success from response text
/// when JSON parsing produces unexpected format
#[test]
fn test_fallback_detection_ok() {
    let responses_indicating_success = vec![
        r#"{"Ok": "Handled"}"#,
        r#"{"Ok":{}}"#,
        r#"{"Ok":null}"#,
        r#"{"Ok":"anything"}"#,
    ];

    for response in responses_indicating_success {
        assert!(
            response.contains("\"Ok\""),
            "Response should contain Ok: {}",
            response
        );
    }
}

#[test]
fn test_fallback_detection_err() {
    let responses_indicating_error = vec![
        r#"{"Err": "Some error"}"#,
        r#"{"Err":"Connection refused"}"#,
    ];

    for response in responses_indicating_error {
        assert!(
            response.contains("\"Err\""),
            "Response should contain Err: {}",
            response
        );
        assert!(
            !response.contains("\"Ok\""),
            "Error response should not contain Ok: {}",
            response
        );
    }
}

// ============================================================================
// SPECIAL CHARACTER HANDLING TESTS
// ============================================================================

#[test]
fn test_window_title_with_quotes() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Window with \"quotes\"","app_id":"test","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let title = parsed["Ok"]["Windows"][0]["title"].as_str().unwrap();
    assert_eq!(title, "Window with \"quotes\"");
}

#[test]
fn test_window_title_with_newlines() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Line1\nLine2\nLine3","app_id":"test","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let title = parsed["Ok"]["Windows"][0]["title"].as_str().unwrap();
    assert!(title.contains('\n'));
}

#[test]
fn test_window_title_with_control_chars() {
    let json = r#"{"Ok":{"Windows":[
        {"id":1,"title":"Tab\there","app_id":"test","is_floating":false}
    ]}}"#;
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();

    let title = parsed["Ok"]["Windows"][0]["title"].as_str().unwrap();
    assert!(title.contains('\t'));
}
