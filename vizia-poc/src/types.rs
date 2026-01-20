//! Shared type definitions (simplified for PoC)
//!
//! In full migration, this would be copied from ../src/types.rs

use serde::{Deserialize, Serialize};

/// Acceleration profile for input devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AccelProfile {
    #[default]
    Flat,
    Adaptive,
}

impl AccelProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccelProfile::Flat => "flat",
            AccelProfile::Adaptive => "adaptive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "flat" => Some(AccelProfile::Flat),
            "adaptive" => Some(AccelProfile::Adaptive),
            _ => None,
        }
    }
}

/// Click method for touchpads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClickMethod {
    #[default]
    ButtonAreas,
    ClickFinger,
}

impl ClickMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClickMethod::ButtonAreas => "button-areas",
            ClickMethod::ClickFinger => "clickfinger",
        }
    }
}

/// Scroll method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScrollMethod {
    #[default]
    TwoFinger,
    Edge,
    OnButtonDown,
}

impl ScrollMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScrollMethod::TwoFinger => "two-finger",
            ScrollMethod::Edge => "edge",
            ScrollMethod::OnButtonDown => "on-button-down",
        }
    }
}
