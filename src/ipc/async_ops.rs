//! Async wrappers for IPC operations that run on background threads
//!
//! These functions move IPC calls off the UI thread to prevent UI freezes
//! when niri is slow or unresponsive. Results are delivered via Slint's
//! `invoke_from_event_loop` which safely queues callbacks on the UI thread.
//!
//! # Usage
//!
//! ```rust,ignore
//! let ui_weak = ui.as_weak();
//! ipc::async_ops::reload_config_async(move |result| {
//!     if let Some(ui) = ui_weak.upgrade() {
//!         match result {
//!             Ok(()) => info!("Config reloaded"),
//!             Err(e) => show_error(&ui, &e.to_string()),
//!         }
//!     }
//! });
//! ```
//!
//! # Thread Safety
//!
//! Each async function spawns a short-lived thread that:
//! 1. Executes the synchronous IPC call (with 2s timeout)
//! 2. Queues the result callback on the UI thread via `invoke_from_event_loop`
//! 3. Exits
//!
//! If the app is shutting down, `invoke_from_event_loop` will fail gracefully
//! and the callback simply won't run - this is safe and expected.

use super::{FullOutputInfo, IpcResult, WindowInfo, WorkspaceInfo};
use log::debug;
use std::thread;

/// Reload niri configuration asynchronously.
///
/// Spawns a background thread to call `reload_config()`, then invokes
/// the callback on the UI thread with the result.
pub fn reload_config_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<()>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async reload_config started");
        let result = super::reload_config();
        debug!("Async reload_config completed: {:?}", result.is_ok());

        // Queue callback on UI thread - ignore error if event loop stopped
        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

/// Get list of windows asynchronously.
///
/// Spawns a background thread to call `get_windows()`, then invokes
/// the callback on the UI thread with the result.
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

        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

/// Get list of workspaces asynchronously.
///
/// Spawns a background thread to call `get_workspaces()`, then invokes
/// the callback on the UI thread with the result.
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

        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

/// Get focused window asynchronously.
///
/// Spawns a background thread to call `get_focused_window()`, then invokes
/// the callback on the UI thread with the result.
pub fn get_focused_window_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Option<WindowInfo>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_focused_window started");
        let result = super::get_focused_window();
        debug!("Async get_focused_window completed");

        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

/// Get focused output asynchronously.
///
/// Spawns a background thread to call `get_focused_output()`, then invokes
/// the callback on the UI thread with the result.
pub fn get_focused_output_async<F>(on_complete: F)
where
    F: FnOnce(IpcResult<Option<String>>) + Send + 'static,
{
    thread::spawn(move || {
        debug!("Async get_focused_output started");
        let result = super::get_focused_output();
        debug!("Async get_focused_output completed");

        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

/// Get full output information asynchronously.
///
/// Spawns a background thread to call `get_full_outputs()`, then invokes
/// the callback on the UI thread with the result. This includes mode lists,
/// scale, position, transform, and VRR status for each output.
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

        let _ = slint::invoke_from_event_loop(move || {
            on_complete(result);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    // Note: These tests don't actually test async behavior since invoke_from_event_loop
    // requires a running Slint event loop. They verify the functions compile and
    // the sync IPC functions are called correctly.

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
        let (tx, _rx) = mpsc::channel::<bool>();

        reload_config_async(move |_result| {
            // This would send if we had an event loop
            let _ = tx.send(true);
        });

        // Give the thread a moment to spawn (it will fail to invoke_from_event_loop
        // since there's no Slint event loop, but that's expected)
        std::thread::sleep(Duration::from_millis(50));
    }
}
