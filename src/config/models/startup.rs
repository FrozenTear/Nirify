//! Startup commands and environment variables

/// A startup command to run when niri starts
#[derive(Debug, Clone, PartialEq)]
pub struct StartupCommand {
    /// Unique identifier for UI
    pub id: u32,
    /// The command and its arguments
    pub command: Vec<String>,
}

impl Default for StartupCommand {
    fn default() -> Self {
        Self {
            id: 0,
            command: vec![String::new()],
        }
    }
}

impl StartupCommand {
    /// Get display string for the command
    pub fn display(&self) -> String {
        self.command.join(" ")
    }
}

/// Startup settings (spawn-at-startup commands)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StartupSettings {
    /// Commands to run at startup
    pub commands: Vec<StartupCommand>,
    /// Counter for generating unique IDs
    pub next_id: u32,
}

/// An environment variable to set
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentVariable {
    /// Unique identifier for UI
    pub id: u32,
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
}

/// Environment settings
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentSettings {
    /// Environment variables to set
    pub variables: Vec<EnvironmentVariable>,
    /// Counter for generating unique IDs
    pub next_id: u32,
}
