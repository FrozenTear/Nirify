//! System theme detection and live updates
//!
//! Supports two sources:
//! 1. XDG Desktop Portal (freedesktop standard) - GNOME, KDE, DankMaterialShell
//! 2. File-based color generators - pywal, wallust, matugen
//!
//! The portal is tried first; if unavailable, falls back to file watching.

use iced::Color;
use std::path::PathBuf;

/// Events emitted by the system theme watcher
#[derive(Debug, Clone)]
pub enum SystemThemeEvent {
    /// Color scheme changed via portal (dark/light preference)
    ColorScheme(ColorScheme),
    /// Accent color changed via portal (RGB in 0-1 range)
    AccentColor { r: f64, g: f64, b: f64 },
    /// Colors loaded from pywal/wallust file
    FileColors(WalColors),
    /// System theme detection is not available
    Unavailable,
    /// Error during theme detection
    Error(String),
}

/// Color scheme preference from freedesktop portal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    /// No preference set
    #[default]
    NoPreference,
    /// User prefers dark theme
    PreferDark,
    /// User prefers light theme
    PreferLight,
}

impl ColorScheme {
    /// Returns true if this represents a dark theme preference
    pub fn is_dark(self) -> bool {
        matches!(self, ColorScheme::PreferDark | ColorScheme::NoPreference)
    }
}

/// Colors loaded from pywal/wallust JSON file
#[derive(Debug, Clone, Default)]
pub struct WalColors {
    pub wallpaper: Option<String>,
    pub background: String,
    pub foreground: String,
    pub cursor: Option<String>,
    pub colors: [String; 16],
}

impl WalColors {
    /// Parse colors from pywal/wallust JSON format
    pub fn from_json(json: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("JSON parse error: {e}"))?;

        let obj = value.as_object().ok_or("Expected JSON object")?;

        // Get special colors (pywal format)
        let special = obj.get("special").and_then(|s| s.as_object());

        let background = special
            .and_then(|s| s.get("background"))
            .and_then(|v| v.as_str())
            .or_else(|| obj.get("background").and_then(|v| v.as_str()))
            .unwrap_or("#1d1f21")
            .to_string();

        let foreground = special
            .and_then(|s| s.get("foreground"))
            .and_then(|v| v.as_str())
            .or_else(|| obj.get("foreground").and_then(|v| v.as_str()))
            .unwrap_or("#c5c8c6")
            .to_string();

        let cursor = special
            .and_then(|s| s.get("cursor"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let wallpaper = obj
            .get("wallpaper")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Get color palette
        let colors_obj = obj.get("colors").and_then(|c| c.as_object());
        let mut colors: [String; 16] = std::array::from_fn(|_| String::new());

        for i in 0..16 {
            let key = format!("color{i}");
            colors[i] = colors_obj
                .and_then(|c| c.get(&key))
                .and_then(|v| v.as_str())
                .unwrap_or("#000000")
                .to_string();
        }

        Ok(Self {
            wallpaper,
            background,
            foreground,
            cursor,
            colors,
        })
    }

    /// Determine if this is a dark or light theme based on background luminance
    pub fn is_dark(&self) -> bool {
        parse_hex_luminance(&self.background) < 0.5
    }

    /// Get the primary accent color (color1 - typically red/primary)
    pub fn accent(&self) -> &str {
        &self.colors[1]
    }

    /// Get color by index (0-15)
    pub fn color(&self, index: usize) -> &str {
        self.colors.get(index).map(|s| s.as_str()).unwrap_or("#000000")
    }
}

// ============================================================================
// System Theme State
// ============================================================================

use iced::theme::Palette;
use iced::Theme;

/// Holds the current system theme state and builds iced themes from it
#[derive(Debug, Clone, Default)]
pub struct SystemThemeState {
    /// Color scheme preference (dark/light)
    pub color_scheme: ColorScheme,
    /// Accent color from portal (RGB in 0-1 range)
    pub accent_color: Option<(f64, f64, f64)>,
    /// Colors from pywal/wallust file
    pub wal_colors: Option<WalColors>,
    /// Whether system theme is available
    pub available: bool,
}

impl SystemThemeState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state from a system theme event
    pub fn handle_event(&mut self, event: SystemThemeEvent) {
        match event {
            SystemThemeEvent::ColorScheme(scheme) => {
                log::info!("System theme: color scheme changed to {:?}", scheme);
                self.color_scheme = scheme;
                self.available = true;
            }
            SystemThemeEvent::AccentColor { r, g, b } => {
                log::info!(
                    "System theme: accent color changed to RGB({:.3}, {:.3}, {:.3})",
                    r, g, b
                );
                self.accent_color = Some((r, g, b));
                self.available = true;
            }
            SystemThemeEvent::FileColors(colors) => {
                log::info!("System theme: loaded colors from file (bg: {})", colors.background);
                self.wal_colors = Some(colors);
                self.available = true;
            }
            SystemThemeEvent::Unavailable => {
                log::info!("System theme: no source available");
                self.available = false;
            }
            SystemThemeEvent::Error(e) => {
                log::warn!("System theme error: {e}");
            }
        }
    }

    /// Build an iced Theme from the current state
    ///
    /// Priority:
    /// 1. Wal colors (if available) - builds full theme from palette
    /// 2. Portal accent + scheme - builds theme with accent color
    /// 3. Fallback to NiriAmber
    pub fn build_theme(&self) -> Theme {
        // If we have wal colors, build theme from them
        if let Some(wal) = &self.wal_colors {
            return self.build_theme_from_wal(wal);
        }

        // If we have accent color from portal, use that
        if let Some((r, g, b)) = self.accent_color {
            return self.build_theme_from_accent(r, g, b);
        }

        // If we have color scheme but no colors, pick appropriate built-in theme
        match self.color_scheme {
            ColorScheme::PreferLight => Theme::CatppuccinLatte,
            ColorScheme::PreferDark | ColorScheme::NoPreference => {
                // Use our custom NiriAmber as dark fallback
                crate::theme::AppTheme::NiriAmber.to_iced_theme()
            }
        }
    }

    /// Build theme from pywal/wallust colors
    fn build_theme_from_wal(&self, wal: &WalColors) -> Theme {
        let bg = parse_hex(&wal.background);
        let fg = parse_hex(&wal.foreground);
        let accent = parse_hex(wal.accent()); // color1
        let success = parse_hex(wal.color(2)); // color2 = green
        let warning = parse_hex(wal.color(3)); // color3 = yellow
        let danger = parse_hex(wal.color(1)); // color1 = red (same as accent typically)

        let palette = Palette {
            background: bg,
            text: fg,
            primary: accent,
            success,
            warning,
            danger,
        };

        Theme::custom("System (Wallpaper)".to_string(), palette)
    }

    /// Build theme from portal accent color
    fn build_theme_from_accent(&self, r: f64, g: f64, b: f64) -> Theme {
        let accent = Color::from_rgb(r as f32, g as f32, b as f32);
        let is_dark = self.color_scheme.is_dark();

        let palette = if is_dark {
            Palette {
                background: Color::from_rgb(0.102, 0.114, 0.137), // #1a1d23
                text: Color::from_rgb(0.902, 0.910, 0.922),       // #e6e8eb
                primary: accent,
                success: Color::from_rgb(0.063, 0.725, 0.506),    // #10b981
                warning: Color::from_rgb(0.961, 0.620, 0.043),    // #f59e0b
                danger: Color::from_rgb(0.937, 0.267, 0.267),     // #ef4444
            }
        } else {
            Palette {
                background: Color::from_rgb(0.98, 0.98, 0.98),
                text: Color::from_rgb(0.1, 0.1, 0.1),
                primary: accent,
                success: Color::from_rgb(0.2, 0.7, 0.3),
                warning: Color::from_rgb(0.85, 0.6, 0.1),
                danger: Color::from_rgb(0.85, 0.2, 0.2),
            }
        };

        let name = if is_dark {
            "System (Dark)"
        } else {
            "System (Light)"
        };

        Theme::custom(name.to_string(), palette)
    }

    /// Returns true if system theme is available and has colors
    pub fn has_colors(&self) -> bool {
        self.available && (self.wal_colors.is_some() || self.accent_color.is_some())
    }

    /// Returns whether the current system theme is light
    pub fn is_light(&self) -> bool {
        if let Some(wal) = &self.wal_colors {
            !wal.is_dark()
        } else {
            !self.color_scheme.is_dark()
        }
    }
}

/// Parse a hex color string to iced Color
pub fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return Color::BLACK;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

/// Calculate relative luminance of a hex color (0.0 = black, 1.0 = white)
fn parse_hex_luminance(hex: &str) -> f32 {
    let color = parse_hex(hex);
    // Simplified luminance calculation
    0.299 * color.r + 0.587 * color.g + 0.114 * color.b
}

/// Known locations for color scheme files
pub fn color_file_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(cache) = dirs::cache_dir() {
        // pywal
        paths.push(cache.join("wal/colors.json"));
        // wallust
        paths.push(cache.join("wallust/colors.json"));
        // matugen
        paths.push(cache.join("matugen/colors.json"));
    }

    paths
}

/// Check if any color file exists and return the first one found
pub fn find_color_file() -> Option<PathBuf> {
    color_file_paths().into_iter().find(|p| p.exists())
}

/// Try to read colors from pywal/wallust file
pub fn read_color_file() -> Option<WalColors> {
    let path = find_color_file()?;
    let content = std::fs::read_to_string(&path).ok()?;
    WalColors::from_json(&content).ok()
}

// ============================================================================
// Portal Integration (using ashpd)
// ============================================================================

pub mod portal {
    use super::{ColorScheme, SystemThemeEvent};
    use iced::futures::StreamExt;

    /// Read current color scheme from portal
    pub async fn read_color_scheme() -> Option<ColorScheme> {
        let settings = ashpd::desktop::settings::Settings::new().await.ok()?;
        let scheme = settings.color_scheme().await.ok()?;
        Some(convert_scheme(scheme))
    }

    /// Read current accent color from portal (RGB in 0-1 range)
    pub async fn read_accent_color() -> Option<(f64, f64, f64)> {
        let settings = ashpd::desktop::settings::Settings::new().await.ok()?;
        let color = settings.accent_color().await.ok()?;
        Some((color.red(), color.green(), color.blue()))
    }

    /// Convert ashpd ColorScheme to our enum
    fn convert_scheme(scheme: ashpd::desktop::settings::ColorScheme) -> ColorScheme {
        match scheme {
            ashpd::desktop::settings::ColorScheme::PreferDark => ColorScheme::PreferDark,
            ashpd::desktop::settings::ColorScheme::PreferLight => ColorScheme::PreferLight,
            ashpd::desktop::settings::ColorScheme::NoPreference => ColorScheme::NoPreference,
        }
    }

    /// Watch for portal theme changes, yielding events
    pub async fn watch_portal(
        mut sender: iced::futures::channel::mpsc::Sender<SystemThemeEvent>,
    ) -> Result<(), String> {
        use iced::futures::SinkExt;

        let settings = ashpd::desktop::settings::Settings::new()
            .await
            .map_err(|e| format!("Failed to connect to portal: {e}"))?;

        // Send initial values
        if let Ok(scheme) = settings.color_scheme().await {
            let _ = sender
                .send(SystemThemeEvent::ColorScheme(convert_scheme(scheme)))
                .await;
        }

        if let Ok(color) = settings.accent_color().await {
            let _ = sender
                .send(SystemThemeEvent::AccentColor {
                    r: color.red(),
                    g: color.green(),
                    b: color.blue(),
                })
                .await;
        }

        // Subscribe to changes
        let mut scheme_stream = settings
            .receive_color_scheme_changed()
            .await
            .map_err(|e| format!("Failed to subscribe to color scheme: {e}"))?;

        let mut accent_stream = settings
            .receive_accent_color_changed()
            .await
            .map_err(|e| format!("Failed to subscribe to accent color: {e}"))?;

        loop {
            tokio::select! {
                Some(scheme) = scheme_stream.next() => {
                    let _ = sender.send(SystemThemeEvent::ColorScheme(convert_scheme(scheme))).await;
                }
                Some(color) = accent_stream.next() => {
                    let _ = sender.send(SystemThemeEvent::AccentColor {
                        r: color.red(),
                        g: color.green(),
                        b: color.blue(),
                    }).await;
                }
                else => break,
            }
        }

        Ok(())
    }

    /// Check if portal is available
    pub async fn is_available() -> bool {
        ashpd::desktop::settings::Settings::new().await.is_ok()
    }
}

// ============================================================================
// File Watcher Integration (using notify)
// ============================================================================

pub mod file_watcher {
    use super::{color_file_paths, find_color_file, SystemThemeEvent};
    use iced::futures::SinkExt;
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    /// Watch color files for changes, yielding events
    pub async fn watch_files(
        mut sender: iced::futures::channel::mpsc::Sender<SystemThemeEvent>,
    ) -> Result<(), String> {
        // Send initial colors if file exists
        if let Some(colors) = super::read_color_file() {
            let _ = sender.send(SystemThemeEvent::FileColors(colors)).await;
        }

        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| format!("Failed to create file watcher: {e}"))?;

        // Watch directories containing color files
        let mut watched_any = false;
        for path in color_file_paths() {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    if watcher.watch(parent, RecursiveMode::NonRecursive).is_ok() {
                        watched_any = true;
                        log::debug!("Watching directory: {}", parent.display());
                    }
                }
            }
        }

        if !watched_any {
            // No directories to watch, but that's okay - user might create them later
            log::debug!("No color file directories found to watch");
        }

        // Process file events
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    if event.kind.is_modify() || event.kind.is_create() {
                        // Check if this affects a color file we care about
                        let color_files = color_file_paths();
                        let is_color_file = event
                            .paths
                            .iter()
                            .any(|p| color_files.iter().any(|cf| p == cf));

                        if is_color_file {
                            // Small delay to let file write complete
                            tokio::time::sleep(Duration::from_millis(50)).await;

                            if let Some(colors) = super::read_color_file() {
                                let _ = sender.send(SystemThemeEvent::FileColors(colors)).await;
                            }
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if sender is closed
                    if sender.is_closed() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            // Yield to async runtime
            tokio::task::yield_now().await;
        }

        Ok(())
    }

    /// Check if any color files exist
    pub fn is_available() -> bool {
        find_color_file().is_some()
    }
}

// ============================================================================
// iced Subscription
// ============================================================================

use iced::Subscription;

/// Create an iced subscription for system theme events
///
/// This tries the portal first, then falls back to file watching.
pub fn subscription() -> Subscription<SystemThemeEvent> {
    Subscription::run(system_theme_worker)
}

fn system_theme_worker() -> impl iced::futures::Stream<Item = SystemThemeEvent> {
    iced::stream::channel(
        32,
        |sender: iced::futures::channel::mpsc::Sender<SystemThemeEvent>| async move {
            // Try portal first
            if portal::is_available().await {
                log::info!("Using freedesktop portal for system theme");
                if let Err(e) = portal::watch_portal(sender.clone()).await {
                    log::warn!("Portal watch failed: {e}");
                    // Fall through to file watcher
                } else {
                    return; // Portal succeeded, we're done
                }
            }

            // Fall back to file watching
            if file_watcher::is_available() {
                log::info!("Using file watcher for system theme (pywal/wallust)");
                if let Err(e) = file_watcher::watch_files(sender.clone()).await {
                    log::warn!("File watcher failed: {e}");
                }
            } else {
                log::info!("No system theme source available");
                use iced::futures::SinkExt;
                let _ = sender.clone().send(SystemThemeEvent::Unavailable).await;
            }

            // Keep the subscription alive but idle
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        let color = parse_hex("#ff0000");
        assert!((color.r - 1.0).abs() < 0.01);
        assert!(color.g.abs() < 0.01);
        assert!(color.b.abs() < 0.01);

        let color = parse_hex("#1a1d23");
        assert!((color.r - 0.102).abs() < 0.01);
    }

    #[test]
    fn test_wal_colors_from_json() {
        let json = r##"{
            "wallpaper": "/path/to/wallpaper.jpg",
            "special": {
                "background": "#1d1f21",
                "foreground": "#c5c8c6",
                "cursor": "#c5c8c6"
            },
            "colors": {
                "color0": "#1d1f21",
                "color1": "#cc6666",
                "color2": "#b5bd68",
                "color3": "#f0c674",
                "color4": "#81a2be",
                "color5": "#b294bb",
                "color6": "#8abeb7",
                "color7": "#c5c8c6",
                "color8": "#969896",
                "color9": "#cc6666",
                "color10": "#b5bd68",
                "color11": "#f0c674",
                "color12": "#81a2be",
                "color13": "#b294bb",
                "color14": "#8abeb7",
                "color15": "#ffffff"
            }
        }"##;

        let colors = WalColors::from_json(json).unwrap();
        assert_eq!(colors.background, "#1d1f21");
        assert_eq!(colors.foreground, "#c5c8c6");
        assert_eq!(colors.accent(), "#cc6666");
        assert!(colors.is_dark());
    }

    #[test]
    fn test_color_scheme_is_dark() {
        assert!(ColorScheme::PreferDark.is_dark());
        assert!(ColorScheme::NoPreference.is_dark());
        assert!(!ColorScheme::PreferLight.is_dark());
    }
}
