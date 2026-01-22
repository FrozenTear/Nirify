//! Helper functions for the App module
//!
//! Contains utility functions used across multiple handlers.

use crate::types::{Color, ColorOrGradient, Gradient};
use crate::views::widgets::GradientPickerMessage;

/// Result of parsing a spawn command
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// The parsed arguments
    pub args: Vec<String>,
    /// Warning message if the command looks dangerous (but is still allowed)
    pub warning: Option<String>,
}

/// Dangerous command patterns that warrant a warning
const DANGEROUS_PATTERNS: &[(&str, &str)] = &[
    ("rm", "removes files - be careful with arguments"),
    ("sudo", "runs with elevated privileges"),
    ("su", "switches user context"),
    ("dd", "can overwrite disks"),
    ("mkfs", "formats filesystems"),
    ("chmod 777", "sets overly permissive permissions"),
    (":(){", "fork bomb pattern detected"),
    (">/dev/sd", "writes directly to disk device"),
    ("| sh", "pipes to shell execution"),
    ("| bash", "pipes to shell execution"),
    ("; rm", "chained delete command"),
    ("&& rm", "chained delete command"),
];

/// Parses a command string into arguments, handling quoted strings properly.
///
/// Supports:
/// - Single quotes: 'hello world' -> "hello world"
/// - Double quotes: "hello world" -> "hello world"
/// - Escaped quotes within quotes
/// - Unquoted arguments separated by whitespace
///
/// Returns a ParsedCommand with the args and an optional warning for dangerous commands.
pub fn parse_spawn_command(command: &str) -> Result<ParsedCommand, String> {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return Ok(ParsedCommand {
            args: Vec::new(),
            warning: None,
        });
    }

    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = trimmed.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                if in_single_quote {
                    // End of single-quoted section
                    in_single_quote = false;
                } else {
                    // Start of single-quoted section
                    in_single_quote = true;
                }
            }
            '"' if !in_single_quote => {
                if in_double_quote {
                    // End of double-quoted section
                    in_double_quote = false;
                } else {
                    // Start of double-quoted section
                    in_double_quote = true;
                }
            }
            '\\' if in_double_quote => {
                // Handle escape sequences in double quotes
                if let Some(&next) = chars.peek() {
                    match next {
                        '"' | '\\' | '$' | '`' => {
                            current_arg.push(chars.next().unwrap());
                        }
                        _ => {
                            current_arg.push(c);
                        }
                    }
                } else {
                    current_arg.push(c);
                }
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                // Whitespace outside quotes - end of argument
                if !current_arg.is_empty() {
                    args.push(std::mem::take(&mut current_arg));
                }
            }
            _ => {
                current_arg.push(c);
            }
        }
    }

    // Check for unclosed quotes
    if in_single_quote {
        return Err("Unclosed single quote in command".to_string());
    }
    if in_double_quote {
        return Err("Unclosed double quote in command".to_string());
    }

    // Don't forget the last argument
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    // Check for dangerous patterns
    let warning = check_dangerous_command(&args, trimmed);

    Ok(ParsedCommand { args, warning })
}

/// Checks if a command contains dangerous patterns
fn check_dangerous_command(args: &[String], full_command: &str) -> Option<String> {
    if args.is_empty() {
        return None;
    }

    let executable = &args[0];
    let lower_command = full_command.to_lowercase();

    for (pattern, description) in DANGEROUS_PATTERNS {
        // Check if the pattern appears in the command
        if lower_command.contains(&pattern.to_lowercase()) {
            return Some(format!(
                "Warning: '{}' - {}. Please verify this is intentional.",
                pattern, description
            ));
        }
    }

    // Check for shell metacharacters that could be dangerous
    if full_command.contains('`') {
        return Some("Warning: Command contains backticks which may execute subcommands".to_string());
    }

    if full_command.contains("$(") {
        return Some("Warning: Command contains $() which may execute subcommands".to_string());
    }

    // Warn about commands with many semicolons or pipes (potential command chaining)
    let semicolons = full_command.matches(';').count();
    let pipes = full_command.matches('|').count();
    if semicolons > 2 || pipes > 3 {
        return Some("Warning: Complex command with multiple operations - please verify".to_string());
    }

    // Basic executable validation - just check it doesn't start with suspicious paths
    if executable.starts_with("/dev/") {
        return Some("Warning: Executing from /dev is unusual - please verify".to_string());
    }

    None
}

/// Validates and parses a command for use in keybindings.
/// Returns the parsed args, or an error message if the command is invalid.
pub fn validate_spawn_command(command: &str) -> Result<Vec<String>, String> {
    let parsed = parse_spawn_command(command)?;

    // Log warning if present (but don't block the command)
    if let Some(warning) = &parsed.warning {
        log::warn!("Keybinding command: {}", warning);
    }

    Ok(parsed.args)
}

/// Formats a key press event into a niri-compatible key combo string
/// e.g., "Mod+Shift+Return" or "Ctrl+Alt+Delete"
pub fn format_key_combo(key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> String {
    use iced::keyboard::{Key, key::Named};

    // Skip if this is just a modifier key by itself
    let is_modifier_key = matches!(
        key,
        Key::Named(Named::Shift | Named::Control | Named::Alt | Named::Super)
    );
    if is_modifier_key {
        return String::new();
    }

    let mut parts = Vec::new();

    // Add modifiers in niri's expected order
    // Note: niri uses "Mod" for Super/Logo key
    if modifiers.logo() {
        parts.push("Mod");
    }
    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        parts.push("Alt");
    }
    if modifiers.shift() {
        parts.push("Shift");
    }

    // Add the key name
    let key_name = match key {
        Key::Named(named) => match named {
            Named::Enter => "Return",
            Named::Tab => "Tab",
            Named::Space => "space",
            Named::Backspace => "BackSpace",
            Named::Delete => "Delete",
            Named::Escape => "Escape",
            Named::Home => "Home",
            Named::End => "End",
            Named::PageUp => "Page_Up",
            Named::PageDown => "Page_Down",
            Named::ArrowUp => "Up",
            Named::ArrowDown => "Down",
            Named::ArrowLeft => "Left",
            Named::ArrowRight => "Right",
            Named::F1 => "F1",
            Named::F2 => "F2",
            Named::F3 => "F3",
            Named::F4 => "F4",
            Named::F5 => "F5",
            Named::F6 => "F6",
            Named::F7 => "F7",
            Named::F8 => "F8",
            Named::F9 => "F9",
            Named::F10 => "F10",
            Named::F11 => "F11",
            Named::F12 => "F12",
            Named::Insert => "Insert",
            Named::PrintScreen => "Print",
            Named::ScrollLock => "Scroll_Lock",
            Named::Pause => "Pause",
            Named::AudioVolumeUp => "XF86AudioRaiseVolume",
            Named::AudioVolumeDown => "XF86AudioLowerVolume",
            Named::AudioVolumeMute => "XF86AudioMute",
            Named::MediaPlayPause => "XF86AudioPlay",
            Named::MediaStop => "XF86AudioStop",
            Named::MediaTrackNext => "XF86AudioNext",
            Named::MediaTrackPrevious => "XF86AudioPrev",
            Named::BrightnessUp => "XF86MonBrightnessUp",
            Named::BrightnessDown => "XF86MonBrightnessDown",
            _ => return String::new(), // Unknown named key
        },
        Key::Character(c) => {
            // For character keys, uppercase for consistent display
            let s = c.as_str();
            if s.len() == 1 {
                // Single character - uppercase it for display
                let upper = s.to_uppercase();
                if parts.is_empty() {
                    return upper;
                } else {
                    return format!("{}+{}", parts.join("+"), upper);
                }
            } else {
                return String::new();
            }
        }
        Key::Unidentified => return String::new(),
    };

    parts.push(key_name);
    parts.join("+")
}

/// Helper to apply GradientPickerMessage to a ColorOrGradient field
pub fn apply_gradient_message(target: &mut ColorOrGradient, msg: GradientPickerMessage) {
    match msg {
            GradientPickerMessage::ToggleSolidGradient(is_gradient) => {
                *target = if is_gradient {
                    // Convert to gradient
                    match target {
                        ColorOrGradient::Color(color) => {
                            ColorOrGradient::Gradient(Gradient {
                                from: *color,
                                to: *color,
                                angle: 0,
                                ..Default::default()
                            })
                        }
                        ColorOrGradient::Gradient(_) => target.clone(),
                    }
                } else {
                    // Convert to solid color
                    match target {
                        ColorOrGradient::Color(_) => target.clone(),
                        ColorOrGradient::Gradient(gradient) => {
                            ColorOrGradient::Color(gradient.from)
                        }
                    }
                };
            }
            GradientPickerMessage::SetFromColor(hex) => {
                if let Some(color) = Color::from_hex(&hex) {
                    match target {
                        ColorOrGradient::Color(c) => *c = color,
                        ColorOrGradient::Gradient(g) => g.from = color,
                    }
                }
            }
            GradientPickerMessage::SetToColor(hex) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    if let Some(color) = Color::from_hex(&hex) {
                        gradient.to = color;
                    }
                }
            }
            GradientPickerMessage::SetAngle(angle) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.angle = angle;
                }
            }
            GradientPickerMessage::SetColorSpace(color_space) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.color_space = color_space;
                }
            }
            GradientPickerMessage::SetRelativeTo(relative_to) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.relative_to = relative_to;
                }
            }
        GradientPickerMessage::SetHueInterpolation(hue_interp) => {
            if let ColorOrGradient::Gradient(gradient) = target {
                gradient.hue_interpolation = Some(hue_interp);
            }
        }
    }
}
