//! Config diff functionality
//!
//! This module provides tools for comparing settings changes
//! before they are saved to disk.

mod generator;
mod types;

pub use generator::generate_diff;
pub use types::{CategoryDiff, ConfigDiff, DiffLine, DiffLineType};
