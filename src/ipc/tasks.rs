//! Async IPC helpers using iced Task
//!
//! This module provides helper functions that wrap sync IPC calls in `iced::Task`,
//! allowing non-blocking IPC operations in iced handlers.
//!
//! # Usage
//!
//! ```ignore
//! use crate::ipc::tasks;
//!
//! fn update(&mut self, message: Message) -> Task<Message> {
//!     match message {
//!         Message::CheckNiri => tasks::check_niri_running(Message::NiriStatusChecked),
//!         Message::RefreshWindows => tasks::get_windows(|r| Message::WindowsLoaded(r)),
//!         _ => Task::none(),
//!     }
//! }
//! ```
//!
//! # Threading
//!
//! Each helper routes its synchronous IPC call through
//! `tokio::task::spawn_blocking`, so the socket round-trip runs on tokio's
//! blocking thread pool and never stalls the async executor (and therefore the
//! UI). A failure to join the background task is surfaced as an
//! [`IpcError::IoError`] via [`join_err`]. Callers that expect a
//! `Result<_, String>` message payload map the error in their message closure,
//! e.g. `|r| Message::Foo(r.map_err(|e| e.to_string()))`.
//!
//! Prefer these helpers over inline `Task::perform(async { crate::ipc::... })`:
//! calling the blocking `crate::ipc` functions directly inside a `Task` future
//! blocks the executor thread.

use iced::Task;

use super::{
    get_focused_window, get_full_outputs, get_version, get_windows, get_workspaces,
    is_niri_running, reload_config, validate_config, FullOutputInfo, IpcError, IpcResult,
    WindowInfo, WorkspaceInfo,
};

/// Run a blocking closure on tokio's blocking thread pool.
async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, tokio::task::JoinError> {
    tokio::task::spawn_blocking(f).await
}

/// Adapt a `JoinError` from a background IPC task into an `IpcError`.
fn join_err<T>(e: tokio::task::JoinError) -> IpcResult<T> {
    Err(IpcError::IoError(format!("background task failed: {e}")))
}

/// Check if niri is running asynchronously.
///
/// Returns a Task that completes with a boolean indicating connection status.
///
/// # Example
///
/// ```ignore
/// tasks::check_niri_running(|connected| Message::NiriStatusChecked(connected))
/// ```
pub fn check_niri_running<M>(f: impl FnOnce(bool) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(is_niri_running).await.unwrap_or(false) },
        f,
    )
}

/// Get windows asynchronously.
///
/// Returns a Task that completes with the windows result.
pub fn get_windows_async<M>(
    f: impl FnOnce(IpcResult<Vec<WindowInfo>>) -> M + Send + 'static,
) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(get_windows).await.unwrap_or_else(join_err) },
        f,
    )
}

/// Get workspaces asynchronously.
///
/// Returns a Task that completes with the workspaces result.
pub fn get_workspaces_async<M>(
    f: impl FnOnce(IpcResult<Vec<WorkspaceInfo>>) -> M + Send + 'static,
) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(get_workspaces).await.unwrap_or_else(join_err) },
        f,
    )
}

/// Get full output info asynchronously.
///
/// Returns a Task that completes with the full outputs result.
pub fn get_full_outputs_async<M>(
    f: impl FnOnce(IpcResult<Vec<FullOutputInfo>>) -> M + Send + 'static,
) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async {
            run_blocking(get_full_outputs)
                .await
                .unwrap_or_else(join_err)
        },
        f,
    )
}

/// Get focused window asynchronously.
///
/// Returns a Task that completes with the focused window result.
pub fn get_focused_window_async<M>(
    f: impl FnOnce(IpcResult<Option<WindowInfo>>) -> M + Send + 'static,
) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async {
            run_blocking(get_focused_window)
                .await
                .unwrap_or_else(join_err)
        },
        f,
    )
}

/// Get niri version asynchronously.
///
/// Returns a Task that completes with the version result.
pub fn get_version_async<M>(f: impl FnOnce(IpcResult<String>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(get_version).await.unwrap_or_else(join_err) },
        f,
    )
}

/// Reload niri config asynchronously.
///
/// Returns a Task that completes with the reload result.
pub fn reload_config_async<M>(f: impl FnOnce(IpcResult<()>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(reload_config).await.unwrap_or_else(join_err) },
        f,
    )
}

/// Validate niri config asynchronously.
///
/// Returns a Task that completes with the validation result.
/// Ok(message) if valid, Err(error_details) if invalid.
pub fn validate_config_async<M>(f: impl FnOnce(IpcResult<String>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(
        async { run_blocking(validate_config).await.unwrap_or_else(join_err) },
        f,
    )
}
