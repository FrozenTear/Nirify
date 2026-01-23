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

use iced::Task;

use super::{
    get_focused_window, get_full_outputs, get_version, get_windows, get_workspaces,
    is_niri_running, reload_config, validate_config, FullOutputInfo, IpcResult, WindowInfo,
    WorkspaceInfo,
};

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
    Task::perform(async { is_niri_running() }, f)
}

/// Get windows asynchronously.
///
/// Returns a Task that completes with the windows result.
pub fn get_windows_async<M>(f: impl FnOnce(IpcResult<Vec<WindowInfo>>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(async { get_windows() }, f)
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
    Task::perform(async { get_workspaces() }, f)
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
    Task::perform(async { get_full_outputs() }, f)
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
    Task::perform(async { get_focused_window() }, f)
}

/// Get niri version asynchronously.
///
/// Returns a Task that completes with the version result.
pub fn get_version_async<M>(f: impl FnOnce(IpcResult<String>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(async { get_version() }, f)
}

/// Reload niri config asynchronously.
///
/// Returns a Task that completes with the reload result.
pub fn reload_config_async<M>(f: impl FnOnce(IpcResult<()>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(async { reload_config() }, f)
}

/// Validate niri config asynchronously.
///
/// Returns a Task that completes with the validation result.
/// Ok(message) if valid, Err(error_details) if invalid.
pub fn validate_config_async<M>(f: impl FnOnce(IpcResult<String>) -> M + Send + 'static) -> Task<M>
where
    M: Send + 'static,
{
    Task::perform(async { validate_config() }, f)
}
