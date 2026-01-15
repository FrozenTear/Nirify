//! Switch events settings (lid close/open, tablet mode)

/// A switch event action (spawn command when event occurs)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SwitchEventAction {
    /// Command to spawn when event occurs (empty means no action)
    pub spawn: Vec<String>,
}

impl SwitchEventAction {
    /// Returns true if this action has a command configured
    pub fn has_action(&self) -> bool {
        !self.spawn.is_empty()
    }

    /// Get display string for the command
    pub fn display(&self) -> String {
        self.spawn.join(" ")
    }
}

/// Switch events settings
///
/// Actions to perform when hardware switches change state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SwitchEventsSettings {
    /// Action when laptop lid closes
    pub lid_close: SwitchEventAction,
    /// Action when laptop lid opens
    pub lid_open: SwitchEventAction,
    /// Action when entering tablet mode
    pub tablet_mode_on: SwitchEventAction,
    /// Action when exiting tablet mode
    pub tablet_mode_off: SwitchEventAction,
}
