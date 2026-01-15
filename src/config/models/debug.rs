//! Debug settings for niri

/// Debug settings for niri
///
/// These are advanced settings primarily for debugging and development.
/// Most users won't need to change these.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DebugSettings {
    // Expert mode - gates dangerous settings across the app
    /// Enable expert mode to show potentially dangerous advanced settings
    pub expert_mode: bool,

    // Rendering options
    /// Render monitors the same way as for screencast
    pub preview_render: bool,
    /// Enable direct scanout into overlay planes
    pub enable_overlay_planes: bool,
    /// Disable cursor plane usage
    pub disable_cursor_plane: bool,
    /// Disable direct scanout to primary and overlay planes
    pub disable_direct_scanout: bool,
    /// Only scanout when buffer format matches composition format
    pub restrict_primary_scanout_to_matching_format: bool,

    // Device configuration
    /// Override DRM device for rendering
    pub render_drm_device: Option<String>,
    /// List of DRM devices to ignore (useful for GPU passthrough)
    pub ignore_drm_devices: Vec<String>,

    // Performance & synchronization
    /// Wait until every frame is done rendering before handing to DRM
    pub wait_for_frame_completion_before_queueing: bool,
    /// Send resize events as quickly as possible
    pub disable_resize_throttling: bool,
    /// Disable synchronized window resizing
    pub disable_transactions: bool,
    /// Simulate unknown presentation time
    pub emulate_zero_presentation_time: bool,
    /// Skip redraws caused by cursor movement during VRR
    pub skip_cursor_only_updates_during_vrr: bool,

    // Hardware & compatibility
    /// Create D-Bus interfaces in non-session instances
    pub dbus_interfaces_in_non_session_instances: bool,
    /// Keep laptop panel on when lid is closed
    pub keep_laptop_panel_on_when_lid_is_closed: bool,
    /// Disable EDID monitor name reading
    pub disable_monitor_names: bool,
    /// Force blank all outputs on TTY switch/resume
    pub force_disable_connectors_on_resume: bool,

    // Window behavior
    /// Only focus windows with valid xdg-activation token
    pub strict_new_window_focus_policy: bool,
    /// Allow focus via invalid xdg-activation serial
    pub honor_xdg_activation_with_invalid_serial: bool,
    /// Drop activated state for unfocused windows
    pub deactivate_unfocused_windows: bool,

    // Screencasting
    /// Force invalid DRM modifier for PipeWire
    pub force_pipewire_invalid_modifier: bool,
}
