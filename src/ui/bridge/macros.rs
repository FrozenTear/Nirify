//! Helper types for UI bridge
//!
//! This module previously contained callback registration macros, but those
//! have been replaced by the dynamic UI pattern which uses generic callbacks
//! with setting IDs instead of per-property callbacks.

// Re-export SaveManager for use by callback modules
pub(crate) use super::save_manager::SaveManager;
