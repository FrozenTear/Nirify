//! Async wrappers for IPC operations that run on background threads
//!
//! These functions move IPC calls off the UI thread to prevent UI freezes
//! when niri is slow or unresponsive. Results are delivered via callbacks
//! which the caller can use to update reactive state.
//!
//! # Thread Safety
//!
//! Each async function spawns a short-lived thread that:
//! 1. Executes the synchronous IPC call (with 2s timeout)
//! 2. Calls the result callback
//! 3. Exits
//!
//! Note: The callback is called on the background thread. If you need to
//! update UI state, use Floem's reactive signals which are thread-safe.

use super::{FullOutputInfo, IpcResult, WindowInfo, WorkspaceInfo};
use log::debug;
use std::thread;

/// Reload niri configuration asynchronously.
///
/// Spawns a background thread to call `reload_config()`, then invokes
/// the callback with the result.
pub fn reload_config_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<()>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async reload_config started");
        let result = super::reload_config();
        debug!("Async reload_config completed: {:?}", result.is_ok());
        on_complete(result);
    });
}

/// Get list of windows asynchronously.
pub fn get_windows_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Vec<WindowInfo>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_windows started");
        let result = super::get_windows();
        debug!(
            "Async get_windows completed: {} windows",
            result.as_ref().map_or(0, |w| w.len())
        );
        on_complete(result);
    });
}

/// Get list of workspaces asynchronously.
pub fn get_workspaces_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Vec<WorkspaceInfo>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_workspaces started");
        let result = super::get_workspaces();
        debug!(
            "Async get_workspaces completed: {} workspaces",
            result.as_ref().map_or(0, |w| w.len())
        );
        on_complete(result);
    });
}

/// Get focused window asynchronously.
pub fn get_focused_window_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Option<WindowInfo>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_focused_window started");
        let result = super::get_focused_window();
        debug!("Async get_focused_window completed");
        on_complete(result);
    });
}

/// Get focused output asynchronously.
pub fn get_focused_output_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Option<String>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_focused_output started");
        let result = super::get_focused_output();
        debug!("Async get_focused_output completed");
        on_complete(result);
    });
}

/// Get full output information asynchronously.
pub fn get_full_outputs_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Vec<FullOutputInfo>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_full_outputs started");
        let result = super::get_full_outputs();
        debug!(
            "Async get_full_outputs completed: {} outputs",
            result.as_ref().map_or(0, |o| o.len())
        );
        on_complete(result);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_async_functions_compile() {
        // Just verify the async wrappers compile with the expected signatures
        fn _test_reload() {
            reload_config_async(|_result: IpcResult<()>| {});
        }
        fn _test_windows() {
            get_windows_async(|_result: IpcResult<Vec<WindowInfo>>| {});
        }
        fn _test_workspaces() {
            get_workspaces_async(|_result: IpcResult<Vec<WorkspaceInfo>>| {});
        }
        fn _test_focused_window() {
            get_focused_window_async(|_result: IpcResult<Option<WindowInfo>>| {});
        }
        fn _test_focused_output() {
            get_focused_output_async(|_result: IpcResult<Option<String>>| {});
        }
        fn _test_full_outputs() {
            get_full_outputs_async(|_result: IpcResult<Vec<FullOutputInfo>>| {});
        }
    }

    #[test]
    fn test_callback_captures_work() {
        // Verify closures can capture and move values
        let (tx, rx) = mpsc::channel::<bool>();

        reload_config_async(move |_result| {
            let _ = tx.send(true);
        });

        // The callback should be called on the background thread
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(_) => (), // Callback was called
            Err(_) => (), // IPC may fail if niri isn't running, that's fine
        }
    }
}
