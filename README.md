# Niri Settings

A native settings application for the [niri](https://github.com/YaLTeR/niri) Wayland compositor, built with Rust and [iced](https://iced.rs/).

## Features

- **Native Performance**: Built in Rust with iced UI for minimal resource usage
- **Live Preview**: Changes apply immediately with auto-save (no Apply button needed)
- **Safe Setup**: Automatically backs up your config before making changes
- **Smart Config Management**: Preserves your custom settings while managing common options
- **Floats by Default**: Settings app floats above your tiled windows (configurable)
- **Theme Support**: NiriAmber and Catppuccin Mocha themes included
- **Comprehensive Coverage**:
  - Appearance (gaps, focus ring, borders, corner radius, background color)
  - Input devices (keyboard, mouse, touchpad, trackpoint, trackball, tablet, touch)
  - Animations (per-animation control with spring/easing curves)
  - Cursor settings
  - Window and layer rules (with regex support)
  - Workspaces and outputs
  - Keybindings (view, edit, and capture new keys)
  - Startup applications and environment variables
  - Debug and advanced options
  - Niri IPC tools (query windows, workspaces, outputs)

## Screenshots

*Coming soon*

## Requirements

- Rust 1.82 or later
- niri v25.11 or later (for `include` directive support)

## Building

```bash
# Clone the repository
git clone https://github.com/FrozenTear/niri-tweaks
cd niri-tweaks

# Build in release mode
cargo build --release

# Run the application
./target/release/niri-settings
```

## Installation

### Using Make (recommended)

```bash
# Build and install to /usr/local
make
sudo make install

# Or install to a custom prefix
make PREFIX=/usr
sudo make PREFIX=/usr install

# Uninstall
sudo make uninstall
```

This installs:
- Binary to `$PREFIX/bin/niri-settings`
- Desktop entry to `$PREFIX/share/applications/niri-settings.desktop`
- Icon to `$PREFIX/share/icons/hicolor/scalable/apps/niri-settings.svg`

### Manual Installation

```bash
cargo build --release
sudo install -Dm755 target/release/niri-settings /usr/local/bin/niri-settings
sudo install -Dm644 resources/niri-settings.desktop /usr/local/share/applications/niri-settings.desktop
sudo install -Dm644 resources/icons/niri-settings.svg /usr/local/share/icons/hicolor/scalable/apps/niri-settings.svg
```

## First Run

On first launch, a setup wizard will guide you through connecting the app to your niri config:

1. **Automatic setup** (recommended): Click "Add Automatically" and the app will:
   - Create a timestamped backup of your `config.kdl` in `~/.config/niri/.backup/`
   - Reorganize your config to use niri-settings for managed options
   - Preserve any custom settings you've added

2. **Manual setup**: Add this line to your `~/.config/niri/config.kdl`:
   ```kdl
   include "~/.config/niri/niri-settings/main.kdl"
   ```

## Configuration Structure

Niri Settings manages configuration files in `~/.config/niri/niri-settings/`:

```
~/.config/niri/niri-settings/
├── main.kdl              # Entry point (includes all other files)
├── preferences.kdl       # App preferences (theme, float behavior)
├── appearance.kdl        # Gaps, focus ring, borders, corner radius
├── behavior.kdl          # Focus follows mouse, workspace behavior
├── animations.kdl        # Animation settings
├── cursor.kdl            # Cursor theme and size
├── overview.kdl          # Overview zoom and backdrop
├── outputs.kdl           # Monitor configuration
├── workspaces.kdl        # Named workspaces
├── keybindings.kdl       # Keyboard shortcuts
├── input/
│   ├── keyboard.kdl      # XKB layout, repeat rate
│   ├── mouse.kdl         # Acceleration, scroll settings
│   ├── touchpad.kdl      # Tap, gestures, DWT
│   ├── trackpoint.kdl    # Trackpoint settings
│   ├── trackball.kdl     # Trackball settings
│   ├── tablet.kdl        # Drawing tablet settings
│   └── touch.kdl         # Touchscreen settings
└── advanced/
    ├── layout-extras.kdl # Advanced layout options
    ├── gestures.kdl      # Touchpad gestures
    ├── window-rules.kdl  # Per-window rules
    ├── layer-rules.kdl   # Layer shell rules
    ├── startup.kdl       # Startup applications
    ├── environment.kdl   # Environment variables
    ├── debug.kdl         # Debug options
    ├── switch-events.kdl # Lid/tablet mode events
    ├── recent-windows.kdl # Recent windows settings
    └── misc.kdl          # Miscellaneous settings
```

## App Preferences

The settings app has its own preferences (separate from niri config):

- **Theme**: Choose between NiriAmber (default) and Catppuccin Mocha
- **Float Settings App**: When enabled (default), the app floats above tiled windows

These can be changed in the **Tools** page under "App Preferences".

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## Architecture

The app follows the Elm Architecture pattern:
- **State**: All application state in a single `App` struct
- **Messages**: Events represented as enum variants
- **Update**: Pure functions that handle state changes
- **View**: UI constructed from current state

See `CLAUDE.md` for detailed architecture documentation.

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- [niri](https://github.com/YaLTeR/niri) - The scrollable-tiling Wayland compositor
- [iced](https://iced.rs/) - Cross-platform GUI library for Rust
- [niri-settings](https://github.com/stefonarch/niri-settings) - Original Python/Qt implementation (inspiration)
