//! IPC communication with niri compositor
//!
//! Provides functions to check niri status and send commands.
//!
//! # Sync vs Async
//!
//! This module provides both synchronous and asynchronous IPC functions:
//!
//! - **Sync functions** (e.g., `reload_config()`) block the calling thread until complete.
//!   Use these for initialization or when blocking is acceptable.
//!
//! - **Async functions** in [`async_ops`] (e.g., `reload_config_async()`) run on background
//!   threads and deliver results via callbacks on the UI thread. Use these from UI callbacks
//!   to prevent freezes.

pub mod async_ops;

use log::{debug, info, warn};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Unified error type for all IPC operations
///
/// This allows callers to handle errors uniformly across all IPC functions.
#[derive(Debug, Error)]
pub enum IpcError {
    /// Niri is not running or socket not available
    #[error("Niri is not running")]
    NotRunning,

    /// Failed to connect to niri socket
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// I/O error during communication
    #[error("I/O error: {0}")]
    IoError(String),

    /// Niri returned an error response
    #[error("Niri error: {0}")]
    NiriError(String),

    /// Failed to parse niri's response
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// External command failed (e.g., niri validate)
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

/// Result type alias for IPC operations
pub type IpcResult<T> = std::result::Result<T, IpcError>;

/// Legacy alias for backwards compatibility
#[deprecated(since = "0.2.0", note = "Use IpcError instead")]
pub type IpcQueryError = IpcError;

/// Maximum allowed response size from niri socket (10MB)
/// Prevents OOM attacks from malicious/compromised sockets.
const MAX_RESPONSE_SIZE: u64 = 10 * 1024 * 1024;

/// Helper function to deserialize null or missing strings as empty strings
fn deserialize_nullable_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Information about a running window from niri IPC
/// We use permissive defaults to handle different niri versions
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WindowInfo {
    #[serde(default)]
    pub id: u64,
    /// Window title - can be null in some cases, defaults to empty string
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub title: String,
    /// App ID - can be null for some windows (like niri-settings itself), defaults to empty string
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub app_id: String,
    #[serde(default)]
    pub is_floating: bool,
    #[serde(default)]
    pub workspace_id: Option<u64>,
    /// Whether this window is focused (added in newer niri versions)
    #[serde(default)]
    pub is_focused: bool,
}

// Response wrapper types for proper JSON parsing
// Using a struct-based approach for more robust parsing
#[derive(Deserialize)]
struct NiriOkResponse<T> {
    #[serde(rename = "Ok")]
    ok: T,
}

#[derive(Debug, Deserialize)]
struct NiriErrResponse {
    #[serde(rename = "Err")]
    err: serde_json::Value,
}

// Helper function to parse responses - tries Ok first, then Err
fn parse_niri_response<T: serde::de::DeserializeOwned>(
    response: &str,
) -> Result<T, (Option<serde_json::Value>, Option<serde_json::Error>)> {
    // First try to parse as an Ok response
    match serde_json::from_str::<NiriOkResponse<T>>(response) {
        Ok(ok_resp) => return Ok(ok_resp.ok),
        Err(ok_err) => {
            // Then try to parse as an Err response
            if let Ok(err_resp) = serde_json::from_str::<NiriErrResponse>(response) {
                return Err((Some(err_resp.err), None));
            }
            // If neither works, return the Ok parse error (more informative)
            Err((None, Some(ok_err)))
        }
    }
}

// Keep the old enum for backwards compatibility in some code paths
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NiriResponse<T> {
    Ok {
        #[serde(rename = "Ok")]
        inner: T,
    },
    Err {
        #[serde(rename = "Err")]
        error: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    #[serde(rename = "Version")]
    version: String,
}

#[derive(Debug, Deserialize)]
struct WindowsResponse {
    #[serde(rename = "Windows")]
    windows: Vec<WindowInfo>,
}

#[derive(Debug, Deserialize)]
struct OutputsResponse {
    #[serde(rename = "Outputs")]
    outputs: std::collections::HashMap<String, serde_json::Value>,
}

/// Response for action requests that don't return data (e.g., LoadConfigFile, Quit)
/// The Ok variant contains "Handled" for successful actions
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ActionResponse {
    Ok {
        #[serde(rename = "Ok")]
        _result: ActionResult,
    },
    Err {
        #[serde(rename = "Err")]
        error: serde_json::Value,
    },
}

/// The result inside a successful action response
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ActionResult {
    Handled(#[allow(dead_code)] String),
    Other(#[allow(dead_code)] serde_json::Value),
}

/// Format a JSON Value error into a human-readable string
fn format_json_error(error: &serde_json::Value) -> String {
    match error {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => {
            // Try to extract a meaningful error message from common patterns
            if let Some(msg) = obj.get("message").and_then(|v| v.as_str()) {
                msg.to_string()
            } else if let Some(msg) = obj.get("error").and_then(|v| v.as_str()) {
                msg.to_string()
            } else {
                // Fall back to compact JSON representation
                serde_json::to_string(error).unwrap_or_else(|_| format!("{:?}", error))
            }
        }
        _ => serde_json::to_string(error).unwrap_or_else(|_| format!("{:?}", error)),
    }
}

/// Get the niri socket path from environment
///
/// Returns the path from NIRI_SOCKET if set. Connection validation happens
/// when actually connecting, avoiding TOCTOU issues from pre-validation.
/// If the path doesn't exist or isn't a socket, `UnixStream::connect` will
/// return an appropriate error.
#[must_use]
fn get_socket_path() -> Option<PathBuf> {
    std::env::var("NIRI_SOCKET").ok().map(PathBuf::from)
}

/// Check if NIRI_SOCKET environment variable is set
///
/// Returns true if NIRI_SOCKET is set. This is a quick check that doesn't
/// verify the socket actually exists or is valid. For verifying niri is
/// responsive, use `is_niri_running()`.
#[must_use]
pub fn niri_socket_exists() -> bool {
    get_socket_path().is_some()
}

/// Check if niri is currently running and responsive
///
/// This attempts to connect to the niri socket to verify it's actually
/// available, not just that the environment variable is set. The socket
/// file may exist after niri crashes, so this provides a more reliable check.
#[must_use]
pub fn is_niri_running() -> bool {
    let Some(socket_path) = get_socket_path() else {
        return false;
    };

    // Try to connect with a short timeout
    match UnixStream::connect(&socket_path) {
        Ok(stream) => {
            // Set a short timeout and try to verify the connection
            let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
            true
        }
        Err(_) => false,
    }
}

/// Send a raw JSON request to niri and get the response
fn send_raw_request(json_request: &str) -> IpcResult<String> {
    let socket_path = get_socket_path()
        .ok_or_else(|| IpcError::ConnectionFailed("NIRI_SOCKET not set".to_string()))?;

    debug!("Connecting to niri socket: {:?}", socket_path);

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| IpcError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

    // Set timeout for read/write (2000ms to handle slow systems or disk I/O delays)
    stream
        .set_read_timeout(Some(Duration::from_millis(2000)))
        .map_err(|e| IpcError::IoError(format!("Failed to set read timeout: {}", e)))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(2000)))
        .map_err(|e| IpcError::IoError(format!("Failed to set write timeout: {}", e)))?;

    // Send the request (niri uses JSON, needs trailing newline)
    let request_with_newline = format!("{}\n", json_request);
    debug!("Sending request: {}", json_request);

    stream
        .write_all(request_with_newline.as_bytes())
        .map_err(|e| IpcError::IoError(format!("Failed to write: {}", e)))?;
    stream
        .flush()
        .map_err(|e| IpcError::IoError(format!("Failed to flush: {}", e)))?;

    // Read response with size limit to prevent OOM from malicious sockets
    // Use take() to limit bytes read before buffering
    let limited_stream = stream.take(MAX_RESPONSE_SIZE + 1);
    let mut reader = BufReader::new(limited_stream);
    let mut response = String::new();
    let bytes_read = reader
        .read_line(&mut response)
        .map_err(|e| IpcError::IoError(format!("Failed to read: {}", e)))?;

    if bytes_read == 0 {
        return Err(IpcError::IoError("Socket closed unexpectedly".to_string()));
    }

    // If we hit the limit, the response is too large
    if bytes_read as u64 > MAX_RESPONSE_SIZE {
        return Err(IpcError::IoError(format!(
            "Response exceeds maximum size of {} bytes",
            MAX_RESPONSE_SIZE
        )));
    }

    debug!("Received response: {}", response.trim());
    Ok(response)
}

/// Send an Action request to niri
fn send_action(action: &str) -> IpcResult<String> {
    let json = format!("{{\"Action\":{}}}", action);
    send_raw_request(&json)
}

/// Send a simple Request variant to niri (e.g., "Version", "Outputs")
fn send_simple_request(request_type: &str) -> IpcResult<String> {
    let json = format!("\"{}\"", request_type);
    send_raw_request(&json)
}

/// Reload niri configuration
pub fn reload_config() -> IpcResult<()> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    info!("Requesting niri config reload...");

    // The action to reload config - niri uses struct variant format since v0.1.6+
    let response = send_action("{\"LoadConfigFile\":{}}")?;

    // Parse the response using typed deserialization
    match serde_json::from_str::<ActionResponse>(&response) {
        Ok(ActionResponse::Ok { .. }) => {
            info!("Niri config reloaded successfully");
            Ok(())
        }
        Ok(ActionResponse::Err { error }) => {
            let error_msg = format_json_error(&error);
            warn!("Niri returned an error for config reload: {}", error_msg);
            Err(IpcError::NiriError(format!(
                "Config reload failed: {}",
                error_msg
            )))
        }
        Err(e) => {
            // Fallback: if parsing fails, check if response looks successful
            // This handles potential future response format changes gracefully
            if response.contains("\"Ok\"") {
                info!("Niri config reloaded successfully (fallback parsing)");
                Ok(())
            } else {
                warn!(
                    "Failed to parse config reload response: {} (response: {})",
                    e,
                    response.chars().take(100).collect::<String>()
                );
                Err(IpcError::ParseError(format!(
                    "Failed to parse reload response: {}",
                    e
                )))
            }
        }
    }
}

/// Request niri to quit
pub fn quit_niri() -> IpcResult<()> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    info!("Requesting niri to quit...");

    // niri uses struct variant format since v0.1.6+
    let response = send_action("{\"Quit\":{}}")?;

    // Parse the response - though niri may close before we get a response
    match serde_json::from_str::<ActionResponse>(&response) {
        Ok(ActionResponse::Ok { .. }) => {
            info!("Niri quit request acknowledged");
            Ok(())
        }
        Ok(ActionResponse::Err { error }) => {
            let error_msg = format_json_error(&error);
            warn!("Niri returned an error for quit request: {}", error_msg);
            Err(IpcError::NiriError(format!("Quit failed: {}", error_msg)))
        }
        Err(_) => {
            // Parsing may fail if niri quit before responding - that's fine
            debug!("Could not parse quit response (niri may have already quit)");
            Ok(())
        }
    }
}

/// Validate niri configuration by running `niri validate`
///
/// Returns Ok(message) if valid, Err(error_details) if invalid.
pub fn validate_config() -> IpcResult<String> {
    use std::process::Command;

    let output = Command::new("niri")
        .arg("validate")
        .output()
        .map_err(|e| IpcError::CommandFailed(format!("Failed to run 'niri validate': {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // niri validate prints to stdout on success
        let msg = stdout.trim();
        if msg.is_empty() {
            Ok("Configuration is valid".to_string())
        } else {
            Ok(msg.to_string())
        }
    } else {
        // On failure, error details are in stderr
        let error_msg = if stderr.is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        Err(IpcError::CommandFailed(error_msg))
    }
}

/// Get niri version (if available)
#[must_use = "the version should be used or the call is pointless"]
pub fn get_version() -> IpcResult<String> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    // Version is a Request variant, not an Action
    let response = send_simple_request("Version")?;

    // Parse version using typed response struct
    match serde_json::from_str::<NiriResponse<VersionResponse>>(&response) {
        Ok(NiriResponse::Ok { inner }) => Ok(inner.version),
        Ok(NiriResponse::Err { error }) => {
            warn!("Niri returned error for version request: {:?}", error);
            Ok("unknown".to_string())
        }
        Err(e) => {
            warn!("Failed to parse version response: {}", e);
            Ok("unknown".to_string())
        }
    }
}

/// Get list of running windows from niri
///
/// Returns `Err(IpcError)` if the query fails, allowing callers to distinguish
/// between "no windows" (empty vec) and "couldn't query" (error).
pub fn get_windows() -> IpcResult<Vec<WindowInfo>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching running windows from niri...");

    // Windows is a Request variant
    let response = send_simple_request("Windows")?;

    // Parse using the more robust helper
    match parse_niri_response::<WindowsResponse>(&response) {
        Ok(inner) => {
            debug!("Found {} running windows", inner.windows.len());
            Ok(inner.windows)
        }
        Err((Some(error), _)) => {
            warn!("Niri returned error for windows request: {:?}", error);
            Err(IpcError::NiriError(format_json_error(&error)))
        }
        Err((None, Some(e))) => {
            let preview: String = response.chars().take(200).collect();
            warn!(
                "Failed to parse windows response: {} (preview: {}...)",
                e, preview
            );
            Err(IpcError::ParseError(format!("{}", e)))
        }
        Err((None, None)) => {
            warn!("Failed to parse windows response: unknown error");
            Err(IpcError::ParseError("Unknown parse error".to_string()))
        }
    }
}

/// Get unique app IDs from running windows
pub fn get_unique_app_ids() -> IpcResult<Vec<String>> {
    let windows = get_windows()?;
    let mut app_ids: Vec<String> = windows.into_iter().map(|w| w.app_id).collect();
    app_ids.sort();
    app_ids.dedup();
    Ok(app_ids)
}

/// Information about a workspace from niri IPC
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceInfo {
    pub id: u64,
    #[serde(default)]
    pub idx: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_focused: bool,
    #[serde(default)]
    pub active_window_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WorkspacesResponse {
    #[serde(rename = "Workspaces")]
    workspaces: Vec<WorkspaceInfo>,
}

/// Get list of workspaces from niri
///
/// Returns `Err(IpcError)` if the query fails, allowing callers to distinguish
/// between "no workspaces" (empty vec) and "couldn't query" (error).
pub fn get_workspaces() -> IpcResult<Vec<WorkspaceInfo>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching workspaces from niri...");
    let response = send_simple_request("Workspaces")?;

    match serde_json::from_str::<NiriResponse<WorkspacesResponse>>(&response) {
        Ok(NiriResponse::Ok { inner }) => {
            debug!("Found {} workspaces", inner.workspaces.len());
            Ok(inner.workspaces)
        }
        Ok(NiriResponse::Err { error }) => {
            warn!("Niri returned error for workspaces request: {:?}", error);
            Err(IpcError::NiriError(format!("{:?}", error)))
        }
        Err(e) => {
            warn!("Failed to parse workspaces response: {}", e);
            Err(IpcError::ParseError(format!("{}", e)))
        }
    }
}

#[derive(Debug, Deserialize)]
struct FocusedWindowResponse {
    #[serde(rename = "FocusedWindow")]
    window: Option<WindowInfo>,
}

/// Get the currently focused window (if any)
///
/// Returns `Ok(None)` if no window is focused.
/// Returns `Err(IpcError)` if the query fails.
pub fn get_focused_window() -> IpcResult<Option<WindowInfo>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching focused window from niri...");
    let response = send_simple_request("FocusedWindow")?;

    match parse_niri_response::<FocusedWindowResponse>(&response) {
        Ok(inner) => Ok(inner.window),
        Err((Some(error), _)) => {
            warn!(
                "Niri returned error for focused window request: {:?}",
                error
            );
            Err(IpcError::NiriError(format_json_error(&error)))
        }
        Err((None, Some(e))) => {
            let preview: String = response.chars().take(200).collect();
            warn!(
                "Failed to parse focused window response: {} (preview: {}...)",
                e, preview
            );
            Err(IpcError::ParseError(format!("{}", e)))
        }
        Err((None, None)) => {
            warn!("Failed to parse focused window response: unknown error");
            Err(IpcError::ParseError("Unknown parse error".to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
struct FocusedOutputResponse {
    #[serde(rename = "FocusedOutput")]
    output: Option<String>,
}

/// Get the name of the currently focused output
///
/// Returns `Ok(None)` if no output is focused.
/// Returns `Err(IpcError)` if the query fails.
pub fn get_focused_output() -> IpcResult<Option<String>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching focused output from niri...");
    let response = send_simple_request("FocusedOutput")?;

    match serde_json::from_str::<NiriResponse<FocusedOutputResponse>>(&response) {
        Ok(NiriResponse::Ok { inner }) => Ok(inner.output),
        Ok(NiriResponse::Err { error }) => {
            warn!(
                "Niri returned error for focused output request: {:?}",
                error
            );
            Err(IpcError::NiriError(format!("{:?}", error)))
        }
        Err(e) => {
            warn!("Failed to parse focused output response: {}", e);
            Err(IpcError::ParseError(format!("{}", e)))
        }
    }
}

/// Information about an output/display from niri IPC (minimal version)
/// We use deny_unknown_fields=false (default) to ignore extra fields from niri
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OutputInfo {
    /// Output name (e.g., "DP-1", "HDMI-A-1") - set from map key
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub make: String,
    #[serde(default)]
    pub model: String,
}

/// Logical output info from niri (position, scale, transform)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OutputLogical {
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    #[serde(default)]
    pub transform: String,
}

/// Display mode info from niri
#[derive(Debug, Clone, Deserialize)]
pub struct OutputMode {
    pub width: i32,
    pub height: i32,
    /// Refresh rate in millihertz (e.g., 60000 = 60Hz)
    pub refresh_rate: i32,
    #[serde(default)]
    pub is_preferred: bool,
}

/// Full output info from niri IPC (includes all settings)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FullOutputInfo {
    /// Output name (e.g., "DP-1", "HDMI-A-1") - set from map key
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub make: String,
    #[serde(default)]
    pub model: String,
    /// Index into the modes array for current mode
    #[serde(default)]
    pub current_mode: Option<usize>,
    /// Available display modes
    #[serde(default)]
    pub modes: Vec<OutputMode>,
    /// Logical output settings (position, scale, transform)
    #[serde(default)]
    pub logical: Option<OutputLogical>,
    /// VRR (variable refresh rate) enabled
    #[serde(default)]
    pub vrr_enabled: bool,
}

impl FullOutputInfo {
    /// Get the current mode as a string (e.g., "1920x1080@60.00")
    #[must_use]
    pub fn current_mode_string(&self) -> String {
        if let Some(mode_idx) = self.current_mode {
            if let Some(mode) = self.modes.get(mode_idx) {
                // Convert millihertz to Hz with 2 decimal places
                let refresh_hz = mode.refresh_rate as f64 / 1000.0;
                return format!("{}x{}@{:.2}", mode.width, mode.height, refresh_hz);
            }
        }
        String::new()
    }

    /// Get the scale, defaulting to 1.0 if not set
    #[must_use]
    pub fn scale(&self) -> f64 {
        self.logical.as_ref().map_or(1.0, |l| l.scale)
    }

    /// Get the X position, defaulting to 0 if not set
    #[must_use]
    pub fn position_x(&self) -> i32 {
        self.logical.as_ref().map_or(0, |l| l.x)
    }

    /// Get the Y position, defaulting to 0 if not set
    #[must_use]
    pub fn position_y(&self) -> i32 {
        self.logical.as_ref().map_or(0, |l| l.y)
    }

    /// Get the transform string from niri
    #[must_use]
    pub fn transform_string(&self) -> String {
        self.logical
            .as_ref()
            .map_or_else(|| "Normal".to_string(), |l| l.transform.clone())
    }

    /// Get the transform as our Transform enum
    #[must_use]
    pub fn transform(&self) -> crate::types::Transform {
        use crate::types::Transform;
        let transform_str = self.transform_string();
        match transform_str.as_str() {
            "Normal" | "normal" => Transform::Normal,
            "90" => Transform::Rotate90,
            "180" => Transform::Rotate180,
            "270" => Transform::Rotate270,
            "Flipped" | "flipped" => Transform::Flipped,
            "Flipped90" | "flipped-90" => Transform::Flipped90,
            "Flipped180" | "flipped-180" => Transform::Flipped180,
            "Flipped270" | "flipped-270" => Transform::Flipped270,
            _ => {
                warn!("Unknown transform '{}', using Normal", transform_str);
                Transform::Normal
            }
        }
    }
}

/// Get list of outputs/displays from niri
///
/// Returns `Err(IpcError)` if the query fails, allowing callers to distinguish
/// between "no outputs" (empty vec) and "couldn't query" (error).
pub fn get_outputs() -> IpcResult<Vec<OutputInfo>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching outputs from niri...");

    // Outputs is a Request variant
    let response = send_simple_request("Outputs")?;

    // Parse using typed response struct
    match serde_json::from_str::<NiriResponse<OutputsResponse>>(&response) {
        Ok(NiriResponse::Ok { inner }) => {
            let outputs: Vec<OutputInfo> = inner
                .outputs
                .keys()
                .map(|name| OutputInfo {
                    name: name.clone(),
                    ..Default::default()
                })
                .collect();
            debug!("Found {} outputs", outputs.len());
            Ok(outputs)
        }
        Ok(NiriResponse::Err { error }) => {
            warn!("Niri returned error for outputs request: {:?}", error);
            Err(IpcError::NiriError(format!("{:?}", error)))
        }
        Err(e) => {
            let preview: String = response.chars().take(100).collect();
            warn!(
                "Failed to parse outputs response: {} (preview: {}...)",
                e, preview
            );
            Err(IpcError::ParseError(format!("{}", e)))
        }
    }
}

/// Get output names from niri
pub fn get_output_names() -> IpcResult<Vec<String>> {
    let outputs = get_outputs()?;
    Ok(outputs.into_iter().map(|o| o.name).collect())
}

/// Get full output info from niri (includes mode, scale, position, transform)
///
/// This function parses the complete output data from niri IPC, which includes:
/// - Current mode (resolution and refresh rate)
/// - Scale factor
/// - Position (x, y)
/// - Transform (rotation)
/// - VRR enabled status
///
/// Returns `Err(IpcError)` if the query fails.
pub fn get_full_outputs() -> IpcResult<Vec<FullOutputInfo>> {
    if !is_niri_running() {
        return Err(IpcError::NotRunning);
    }

    debug!("Fetching full output info from niri...");

    // Outputs is a Request variant
    let response = send_simple_request("Outputs")?;

    // Parse using typed response struct
    match serde_json::from_str::<NiriResponse<OutputsResponse>>(&response) {
        Ok(NiriResponse::Ok { inner }) => {
            let outputs: Vec<FullOutputInfo> = inner
                .outputs
                .into_iter()
                .filter_map(|(name, value)| {
                    // Try to parse the full output info from the value
                    match serde_json::from_value::<FullOutputInfo>(value) {
                        Ok(mut info) => {
                            info.name = name;
                            Some(info)
                        }
                        Err(e) => {
                            warn!("Failed to parse output '{}': {}", name, e);
                            None
                        }
                    }
                })
                .collect();
            debug!("Parsed {} outputs with full info", outputs.len());
            Ok(outputs)
        }
        Ok(NiriResponse::Err { error }) => {
            warn!("Niri returned error for outputs request: {:?}", error);
            Err(IpcError::NiriError(format!("{:?}", error)))
        }
        Err(e) => {
            let preview: String = response.chars().take(100).collect();
            warn!(
                "Failed to parse outputs response: {} (preview: {}...)",
                e, preview
            );
            Err(IpcError::ParseError(format!("{}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // ========== Socket existence tests ==========
    // These tests manipulate NIRI_SOCKET env var, so they must run serially

    #[test]
    #[serial]
    fn test_is_niri_running_without_socket() {
        // Remove NIRI_SOCKET if set (for isolated test)
        std::env::remove_var("NIRI_SOCKET");
        assert!(!is_niri_running());
        assert!(!niri_socket_exists());
    }

    #[test]
    #[serial]
    fn test_niri_socket_exists_with_invalid_path() {
        // Set to a path that doesn't exist
        // niri_socket_exists() only checks if env var is set, not if path is valid
        // is_niri_running() actually validates by attempting connection
        std::env::set_var("NIRI_SOCKET", "/tmp/nonexistent-niri-socket-12345");
        assert!(niri_socket_exists()); // Env var IS set, so returns true
        assert!(!is_niri_running()); // But connection fails, so returns false
        std::env::remove_var("NIRI_SOCKET");
    }

    #[test]
    #[serial]
    fn test_niri_socket_exists_with_regular_file() {
        use std::io::Write;

        // Create a temporary regular file (not a socket)
        let temp_dir = std::env::temp_dir();
        let fake_socket = temp_dir.join("fake-niri-socket-test-file");
        let mut file = std::fs::File::create(&fake_socket).unwrap();
        file.write_all(b"not a socket").unwrap();
        drop(file);

        // niri_socket_exists() only checks if env var is set
        // is_niri_running() validates by attempting connection (fails on regular file)
        std::env::set_var("NIRI_SOCKET", &fake_socket);
        assert!(niri_socket_exists()); // Env var IS set, so returns true
        assert!(!is_niri_running()); // But it's not a socket, so connection fails

        std::fs::remove_file(&fake_socket).ok();
        std::env::remove_var("NIRI_SOCKET");
    }

    // ========== JSON parsing tests ==========

    #[test]
    fn test_parse_version_response_ok() {
        let json = r#"{"Ok":{"Version":"0.1.10"}}"#;
        let parsed: NiriResponse<VersionResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => assert_eq!(inner.version, "0.1.10"),
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_version_response_err() {
        let json = r#"{"Err":"Some error message"}"#;
        let parsed: NiriResponse<VersionResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { .. } => panic!("Expected Err variant"),
            NiriResponse::Err { error } => {
                assert_eq!(error.as_str().unwrap(), "Some error message");
            }
        }
    }

    #[test]
    fn test_parse_windows_response_empty() {
        let json = r#"{"Ok":{"Windows":[]}}"#;
        let parsed: NiriResponse<WindowsResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => assert!(inner.windows.is_empty()),
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_windows_response_with_windows() {
        let json = r#"{"Ok":{"Windows":[
            {"id":1,"title":"Firefox","app_id":"firefox","is_floating":false,"workspace_id":1},
            {"id":2,"title":"Terminal","app_id":"kitty","is_floating":true}
        ]}}"#;
        let parsed: NiriResponse<WindowsResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => {
                assert_eq!(inner.windows.len(), 2);
                assert_eq!(inner.windows[0].id, 1);
                assert_eq!(inner.windows[0].title, "Firefox");
                assert_eq!(inner.windows[0].app_id, "firefox");
                assert!(!inner.windows[0].is_floating);
                assert_eq!(inner.windows[0].workspace_id, Some(1));
                assert_eq!(inner.windows[1].id, 2);
                assert!(inner.windows[1].is_floating);
                assert_eq!(inner.windows[1].workspace_id, None); // Missing field defaults to None
            }
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_workspaces_response() {
        let json = r#"{"Ok":{"Workspaces":[
            {"id":1,"idx":1,"name":"main","output":"DP-1","is_active":true,"is_focused":true,"active_window_id":42},
            {"id":2,"idx":2,"output":"DP-1","is_active":false,"is_focused":false}
        ]}}"#;
        let parsed: NiriResponse<WorkspacesResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => {
                assert_eq!(inner.workspaces.len(), 2);
                assert_eq!(inner.workspaces[0].id, 1);
                assert_eq!(inner.workspaces[0].name, Some("main".to_string()));
                assert_eq!(inner.workspaces[0].output, Some("DP-1".to_string()));
                assert!(inner.workspaces[0].is_active);
                assert!(inner.workspaces[0].is_focused);
                assert_eq!(inner.workspaces[0].active_window_id, Some(42));
                assert_eq!(inner.workspaces[1].name, None); // Missing field
                assert!(!inner.workspaces[1].is_focused);
            }
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_focused_window_response_some() {
        let json = r#"{"Ok":{"FocusedWindow":{"id":5,"title":"Editor","app_id":"code","is_floating":false}}}"#;
        let parsed: NiriResponse<FocusedWindowResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => {
                let window = inner.window.unwrap();
                assert_eq!(window.id, 5);
                assert_eq!(window.app_id, "code");
            }
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_focused_window_response_none() {
        let json = r#"{"Ok":{"FocusedWindow":null}}"#;
        let parsed: NiriResponse<FocusedWindowResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => assert!(inner.window.is_none()),
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_focused_output_response() {
        let json = r#"{"Ok":{"FocusedOutput":"HDMI-A-1"}}"#;
        let parsed: NiriResponse<FocusedOutputResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => {
                assert_eq!(inner.output, Some("HDMI-A-1".to_string()));
            }
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_outputs_response() {
        let json = r#"{"Ok":{"Outputs":{"DP-1":{"make":"Dell","model":"U2720Q"},"HDMI-A-1":{}}}}"#;
        let parsed: NiriResponse<OutputsResponse> = serde_json::from_str(json).unwrap();
        match parsed {
            NiriResponse::Ok { inner } => {
                assert_eq!(inner.outputs.len(), 2);
                assert!(inner.outputs.contains_key("DP-1"));
                assert!(inner.outputs.contains_key("HDMI-A-1"));
            }
            NiriResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_full_output_info() {
        let json = r#"{
            "make": "Dell",
            "model": "U2720Q",
            "current_mode": 0,
            "modes": [
                {"width": 3840, "height": 2160, "refresh_rate": 60000, "is_preferred": true},
                {"width": 1920, "height": 1080, "refresh_rate": 60000, "is_preferred": false}
            ],
            "logical": {
                "x": 0,
                "y": 0,
                "scale": 2.0,
                "transform": "Normal"
            },
            "vrr_enabled": false
        }"#;
        let mut info: FullOutputInfo = serde_json::from_str(json).unwrap();
        info.name = "DP-1".to_string();

        assert_eq!(info.make, "Dell");
        assert_eq!(info.model, "U2720Q");
        assert_eq!(info.current_mode, Some(0));
        assert_eq!(info.modes.len(), 2);
        assert_eq!(info.modes[0].width, 3840);
        assert_eq!(info.modes[0].height, 2160);
        assert!(info.modes[0].is_preferred);
        assert_eq!(info.scale(), 2.0);
        assert_eq!(info.position_x(), 0);
        assert_eq!(info.position_y(), 0);
        assert!(!info.vrr_enabled);
    }

    #[test]
    fn test_full_output_current_mode_string() {
        let mut info = FullOutputInfo {
            current_mode: Some(0),
            modes: vec![OutputMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60000,
                is_preferred: true,
            }],
            ..Default::default()
        };
        assert_eq!(info.current_mode_string(), "1920x1080@60.00");

        // Test with no current mode
        info.current_mode = None;
        assert_eq!(info.current_mode_string(), "");

        // Test with out of bounds index
        info.current_mode = Some(999);
        assert_eq!(info.current_mode_string(), "");
    }

    #[test]
    fn test_full_output_transform_parsing() {
        let test_cases = [
            ("Normal", crate::types::Transform::Normal),
            ("normal", crate::types::Transform::Normal),
            ("90", crate::types::Transform::Rotate90),
            ("180", crate::types::Transform::Rotate180),
            ("270", crate::types::Transform::Rotate270),
            ("Flipped", crate::types::Transform::Flipped),
            ("flipped", crate::types::Transform::Flipped),
            ("Flipped90", crate::types::Transform::Flipped90),
            ("Flipped180", crate::types::Transform::Flipped180),
            ("Flipped270", crate::types::Transform::Flipped270),
            ("unknown", crate::types::Transform::Normal), // Unknown defaults to Normal
        ];

        for (input, expected) in test_cases {
            let info = FullOutputInfo {
                logical: Some(OutputLogical {
                    x: 0,
                    y: 0,
                    scale: 1.0,
                    transform: input.to_string(),
                }),
                ..Default::default()
            };
            assert_eq!(info.transform(), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_full_output_defaults_without_logical() {
        let info = FullOutputInfo::default();
        assert_eq!(info.scale(), 1.0);
        assert_eq!(info.position_x(), 0);
        assert_eq!(info.position_y(), 0);
        assert_eq!(info.transform_string(), "Normal");
    }

    #[test]
    fn test_parse_malformed_json_returns_error() {
        let malformed = r#"{"Ok": not valid json"#;
        let result: Result<NiriResponse<VersionResponse>, _> = serde_json::from_str(malformed);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unexpected_structure() {
        // Valid JSON but wrong structure
        let wrong_structure = r#"{"Something":"else"}"#;
        let result: Result<NiriResponse<VersionResponse>, _> =
            serde_json::from_str(wrong_structure);
        // This should fail to parse as NiriResponse expects Ok or Err
        assert!(result.is_err());
    }

    #[test]
    fn test_window_info_optional_fields() {
        // Test that missing optional fields don't cause parse failures
        let minimal = r#"{"id":1,"title":"Test","app_id":"test","is_floating":false}"#;
        let info: WindowInfo = serde_json::from_str(minimal).unwrap();
        assert_eq!(info.id, 1);
        assert_eq!(info.workspace_id, None);
    }

    #[test]
    fn test_window_info_ignores_extra_fields() {
        // Test that extra fields from newer niri versions are ignored
        let json = r#"{
            "id": 42,
            "title": "Firefox",
            "app_id": "firefox",
            "pid": 12345,
            "workspace_id": 1,
            "is_focused": true,
            "is_floating": false,
            "is_urgent": false,
            "layout": {"type": "tiled", "size": [800, 600]},
            "focus_timestamp": {"secs": 123, "nanos": 456}
        }"#;
        let info: WindowInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, 42);
        assert_eq!(info.app_id, "firefox");
        assert_eq!(info.title, "Firefox");
        assert!(info.is_focused);
        assert!(!info.is_floating);
        // Extra fields like pid, is_urgent, layout, focus_timestamp should be ignored
    }

    #[test]
    fn test_window_info_handles_null_app_id() {
        // Test that null app_id values are handled (e.g., for niri-settings itself)
        let json = r#"{
            "id": 27,
            "title": "niri-settings",
            "app_id": null,
            "pid": 68767,
            "workspace_id": 3,
            "is_focused": true,
            "is_floating": false
        }"#;
        let info: WindowInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, 27);
        assert_eq!(info.title, "niri-settings");
        assert_eq!(info.app_id, ""); // null should become empty string
        assert!(info.is_focused);
    }

    #[test]
    fn test_window_info_handles_null_title() {
        // Test that null title values are handled
        let json = r#"{
            "id": 1,
            "title": null,
            "app_id": "some-app",
            "is_floating": false
        }"#;
        let info: WindowInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.app_id, "some-app");
        assert_eq!(info.title, ""); // null should become empty string
    }

    #[test]
    fn test_windows_response_with_extra_fields() {
        // Test full Windows response parsing with extra fields
        let json = r#"{"Ok":{"Windows":[
            {"id":1,"title":"Firefox","app_id":"firefox","pid":1234,"is_floating":false,"workspace_id":1,"layout":{}},
            {"id":2,"title":"Terminal","app_id":"kitty","pid":5678,"is_floating":true,"is_urgent":false}
        ]}}"#;

        // Test using the parse_niri_response helper
        let result = parse_niri_response::<WindowsResponse>(&json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let windows = result.unwrap().windows;
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].app_id, "firefox");
        assert_eq!(windows[1].app_id, "kitty");
    }

    #[test]
    fn test_workspace_info_optional_fields() {
        // Test minimal workspace with only required id field
        let minimal = r#"{"id":1}"#;
        let info: WorkspaceInfo = serde_json::from_str(minimal).unwrap();
        assert_eq!(info.id, 1);
        assert_eq!(info.idx, 0); // Default
        assert_eq!(info.name, None);
        assert_eq!(info.output, None);
        assert!(!info.is_active);
        assert!(!info.is_focused);
        assert_eq!(info.active_window_id, None);
    }

    #[test]
    fn test_output_mode_parsing() {
        let json = r#"{"width":2560,"height":1440,"refresh_rate":144000,"is_preferred":true}"#;
        let mode: OutputMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode.width, 2560);
        assert_eq!(mode.height, 1440);
        assert_eq!(mode.refresh_rate, 144000); // 144Hz in millihertz
        assert!(mode.is_preferred);
    }

    #[test]
    fn test_output_logical_parsing() {
        let json = r#"{"x":1920,"y":0,"scale":1.5,"transform":"90"}"#;
        let logical: OutputLogical = serde_json::from_str(json).unwrap();
        assert_eq!(logical.x, 1920);
        assert_eq!(logical.y, 0);
        assert_eq!(logical.scale, 1.5);
        assert_eq!(logical.transform, "90");
    }

    // ========== ActionResponse parsing tests ==========

    #[test]
    fn test_parse_action_response_ok_handled() {
        let json = r#"{"Ok":"Handled"}"#;
        let parsed: ActionResponse = serde_json::from_str(json).unwrap();
        match parsed {
            ActionResponse::Ok { _result } => match _result {
                ActionResult::Handled(s) => assert_eq!(s, "Handled"),
                ActionResult::Other(_) => panic!("Expected Handled variant"),
            },
            ActionResponse::Err { .. } => panic!("Expected Ok variant"),
        }
    }

    #[test]
    fn test_parse_action_response_err_string() {
        let json = r#"{"Err":"Config file has errors"}"#;
        let parsed: ActionResponse = serde_json::from_str(json).unwrap();
        match parsed {
            ActionResponse::Ok { .. } => panic!("Expected Err variant"),
            ActionResponse::Err { error } => {
                assert_eq!(error.as_str().unwrap(), "Config file has errors");
            }
        }
    }

    #[test]
    fn test_parse_action_response_err_object() {
        let json = r#"{"Err":{"message":"Invalid configuration","line":42}}"#;
        let parsed: ActionResponse = serde_json::from_str(json).unwrap();
        match parsed {
            ActionResponse::Ok { .. } => panic!("Expected Err variant"),
            ActionResponse::Err { error } => {
                assert!(error.is_object());
                assert_eq!(
                    error.get("message").unwrap().as_str().unwrap(),
                    "Invalid configuration"
                );
            }
        }
    }

    // ========== format_json_error tests ==========

    #[test]
    fn test_format_json_error_string() {
        let error = serde_json::json!("Simple error message");
        assert_eq!(format_json_error(&error), "Simple error message");
    }

    #[test]
    fn test_format_json_error_object_with_message() {
        let error = serde_json::json!({"message": "Error occurred", "code": 123});
        assert_eq!(format_json_error(&error), "Error occurred");
    }

    #[test]
    fn test_format_json_error_object_with_error_field() {
        let error = serde_json::json!({"error": "Something went wrong", "details": "more info"});
        assert_eq!(format_json_error(&error), "Something went wrong");
    }

    #[test]
    fn test_format_json_error_object_without_known_fields() {
        let error = serde_json::json!({"foo": "bar"});
        let result = format_json_error(&error);
        // Should return JSON representation
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
    }

    #[test]
    fn test_format_json_error_null() {
        let error = serde_json::json!(null);
        let result = format_json_error(&error);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_format_json_error_number() {
        let error = serde_json::json!(42);
        let result = format_json_error(&error);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_json_error_array() {
        let error = serde_json::json!(["error1", "error2"]);
        let result = format_json_error(&error);
        assert!(result.contains("error1"));
        assert!(result.contains("error2"));
    }
}
