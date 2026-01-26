#!/bin/bash
#
# Nirify Installer
# A native settings application for the niri Wayland compositor
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/FrozenTear/nirify/main/install.sh | bash
#
# Options (via environment variables):
#   NIRIFY_PREFIX=/usr/local    Install prefix (default: /usr/local)
#   NIRIFY_METHOD=binary        Installation method: binary (default) or source
#   NIRIFY_VERSION=latest       Version to install (default: latest)
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/FrozenTear/nirify"
REPO_NAME="nirify"
PREFIX="${NIRIFY_PREFIX:-/usr/local}"
METHOD="${NIRIFY_METHOD:-binary}"
VERSION="${NIRIFY_VERSION:-latest}"
TEMP_DIR=""

# Cleanup on exit
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# Logging functions
info() {
    echo -e "${BLUE}::${NC} $1"
}

success() {
    echo -e "${GREEN}::${NC} $1"
}

warn() {
    echo -e "${YELLOW}::${NC} $1"
}

error() {
    echo -e "${RED}error:${NC} $1" >&2
}

die() {
    error "$1"
    exit 1
}

# Print banner
print_banner() {
    echo -e "${BOLD}"
    echo "  _   _ _      _  __       "
    echo " | \ | (_)_ __(_)/ _|_   _ "
    echo " |  \| | | '__| | |_| | | |"
    echo " | |\  | | |  | |  _| |_| |"
    echo " |_| \_|_|_|  |_|_|  \__, |"
    echo "                     |___/ "
    echo -e "${NC}"
    echo "Settings application for niri Wayland compositor"
    echo ""
}

# Check if running on Linux
check_platform() {
    local os
    os="$(uname -s)"

    if [[ "$os" != "Linux" ]]; then
        die "Nirify is only supported on Linux (detected: $os)"
    fi

    local arch
    arch="$(uname -m)"

    if [[ "$arch" != "x86_64" && "$arch" != "aarch64" ]]; then
        warn "Unsupported architecture: $arch. Only x86_64 has pre-built binaries."
        warn "Will attempt to build from source."
        METHOD="source"
    fi

    info "Detected platform: Linux $arch"
}

# Check for required commands
check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    fi
    return 0
}

# Check dependencies for binary installation
check_binary_deps() {
    local missing=()

    if ! check_command curl && ! check_command wget; then
        missing+=("curl or wget")
    fi

    if ! check_command tar; then
        missing+=("tar")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        die "Missing required tools: ${missing[*]}"
    fi
}

# Check dependencies for source installation
check_source_deps() {
    local missing=()

    if ! check_command cargo; then
        missing+=("cargo (Rust toolchain)")
    fi

    if ! check_command git; then
        missing+=("git")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo ""
        error "Missing build dependencies: ${missing[*]}"
        echo ""
        echo "To install Rust toolchain:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo ""
        echo "To install git on Debian/Ubuntu:"
        echo "  sudo apt install git"
        echo ""
        echo "To install git on Fedora:"
        echo "  sudo dnf install git"
        echo ""
        echo "To install git on Arch:"
        echo "  sudo pacman -S git"
        echo ""
        die "Please install missing dependencies and try again."
    fi

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oP '\d+\.\d+' | head -1)
    local required_version="1.82"

    if [[ "$(printf '%s\n' "$required_version" "$rust_version" | sort -V | head -n1)" != "$required_version" ]]; then
        die "Rust $required_version or later is required (found: $rust_version). Run 'rustup update' to upgrade."
    fi

    info "Rust version: $rust_version"
}

# Get latest version from GitHub
get_latest_version() {
    local version

    if check_command curl; then
        version=$(curl -fsSL "https://api.github.com/repos/FrozenTear/nirify/releases/latest" 2>/dev/null | grep -oP '"tag_name":\s*"\K[^"]+' || echo "")
    elif check_command wget; then
        version=$(wget -qO- "https://api.github.com/repos/FrozenTear/nirify/releases/latest" 2>/dev/null | grep -oP '"tag_name":\s*"\K[^"]+' || echo "")
    fi

    if [[ -z "$version" ]]; then
        warn "Could not fetch latest version from GitHub. Using v0.2.7"
        version="v0.2.7"
    fi

    echo "$version"
}

# Download file
download() {
    local url="$1"
    local dest="$2"

    if check_command curl; then
        curl -fsSL "$url" -o "$dest"
    elif check_command wget; then
        wget -q "$url" -O "$dest"
    else
        die "No download tool available (curl or wget)"
    fi
}

# Install from pre-built binary
install_binary() {
    info "Installing from pre-built binary..."

    check_binary_deps

    # Get version
    local version="$VERSION"
    if [[ "$version" == "latest" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version)
    fi

    # Ensure version starts with 'v'
    if [[ "$version" != v* ]]; then
        version="v$version"
    fi

    info "Version: $version"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download archive
    local arch
    arch="$(uname -m)"
    local archive_name="nirify-${version}-${arch}-unknown-linux-gnu.tar.gz"
    local download_url="${REPO_URL}/releases/download/${version}/${archive_name}"

    info "Downloading $archive_name..."
    if ! download "$download_url" "$archive_name"; then
        warn "Pre-built binary not available for this version/architecture."
        warn "Falling back to source installation..."
        METHOD="source"
        install_source
        return
    fi

    # Extract
    info "Extracting..."
    tar -xzf "$archive_name"

    # Install
    info "Installing to $PREFIX..."

    if [[ -w "$PREFIX/bin" ]] || [[ ! -d "$PREFIX/bin" && -w "$(dirname "$PREFIX")" ]]; then
        install -Dm755 nirify "$PREFIX/bin/nirify"
        install -Dm644 nirify.desktop "$PREFIX/share/applications/nirify.desktop"
        install -Dm644 nirify.svg "$PREFIX/share/icons/hicolor/scalable/apps/nirify.svg"
    else
        info "Requesting sudo for installation..."
        sudo install -Dm755 nirify "$PREFIX/bin/nirify"
        sudo install -Dm644 nirify.desktop "$PREFIX/share/applications/nirify.desktop"
        sudo install -Dm644 nirify.svg "$PREFIX/share/icons/hicolor/scalable/apps/nirify.svg"
    fi

    success "Binary installation complete!"
}

# Install from source
install_source() {
    info "Installing from source..."

    check_source_deps

    # Create temp directory
    if [[ -z "$TEMP_DIR" ]]; then
        TEMP_DIR=$(mktemp -d)
    fi
    cd "$TEMP_DIR"

    # Clone repository
    info "Cloning repository..."

    local clone_args=("--depth=1")
    if [[ "$VERSION" != "latest" ]]; then
        local version="$VERSION"
        if [[ "$version" != v* ]]; then
            version="v$version"
        fi
        clone_args+=("--branch" "$version")
    fi

    git clone "${clone_args[@]}" "$REPO_URL" "$REPO_NAME"
    cd "$REPO_NAME"

    # Build
    info "Building (this may take a few minutes)..."
    cargo build --release

    # Install
    info "Installing to $PREFIX..."

    if [[ -w "$PREFIX/bin" ]] || [[ ! -d "$PREFIX/bin" && -w "$(dirname "$PREFIX")" ]]; then
        install -Dm755 target/release/nirify "$PREFIX/bin/nirify"
        install -Dm644 nirify.desktop "$PREFIX/share/applications/nirify.desktop"
        install -Dm644 resources/icons/nirify.svg "$PREFIX/share/icons/hicolor/scalable/apps/nirify.svg"
    else
        info "Requesting sudo for installation..."
        sudo install -Dm755 target/release/nirify "$PREFIX/bin/nirify"
        sudo install -Dm644 nirify.desktop "$PREFIX/share/applications/nirify.desktop"
        sudo install -Dm644 resources/icons/nirify.svg "$PREFIX/share/icons/hicolor/scalable/apps/nirify.svg"
    fi

    success "Source installation complete!"
}

# Print post-install instructions
print_post_install() {
    echo ""
    success "Nirify has been installed successfully!"
    echo ""
    echo "To get started:"
    echo "  1. Run 'nirify' to launch the application"
    echo "  2. Follow the setup wizard to connect to your niri config"
    echo ""
    echo "Requirements:"
    echo "  - niri v25.11 or later (for include directive support)"
    echo ""
    echo "To uninstall:"
    echo "  sudo rm -f $PREFIX/bin/nirify"
    echo "  sudo rm -f $PREFIX/share/applications/nirify.desktop"
    echo "  sudo rm -f $PREFIX/share/icons/hicolor/scalable/apps/nirify.svg"
    echo ""

    # Check if nirify is in PATH
    if ! command -v nirify &> /dev/null; then
        warn "$PREFIX/bin is not in your PATH"
        echo "Add it with:"
        echo "  export PATH=\"$PREFIX/bin:\$PATH\""
        echo ""
    fi
}

# Parse arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --prefix=*)
                PREFIX="${1#*=}"
                shift
                ;;
            --prefix)
                PREFIX="$2"
                shift 2
                ;;
            --method=*)
                METHOD="${1#*=}"
                shift
                ;;
            --method)
                METHOD="$2"
                shift 2
                ;;
            --version=*)
                VERSION="${1#*=}"
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --binary)
                METHOD="binary"
                shift
                ;;
            --source)
                METHOD="source"
                shift
                ;;
            --help|-h)
                echo "Nirify Installer"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --prefix=PATH    Installation prefix (default: /usr/local)"
                echo "  --method=METHOD  Installation method: binary or source (default: binary)"
                echo "  --version=VER    Version to install (default: latest)"
                echo "  --binary         Shorthand for --method=binary"
                echo "  --source         Shorthand for --method=source"
                echo "  -h, --help       Show this help message"
                echo ""
                echo "Environment variables:"
                echo "  NIRIFY_PREFIX    Same as --prefix"
                echo "  NIRIFY_METHOD    Same as --method"
                echo "  NIRIFY_VERSION   Same as --version"
                echo ""
                echo "Examples:"
                echo "  # Install latest binary to /usr/local"
                echo "  curl -fsSL https://raw.githubusercontent.com/FrozenTear/nirify/main/install.sh | bash"
                echo ""
                echo "  # Install specific version from source to /usr"
                echo "  curl -fsSL https://raw.githubusercontent.com/FrozenTear/nirify/main/install.sh | bash -s -- --source --prefix=/usr --version=0.2.7"
                echo ""
                exit 0
                ;;
            *)
                warn "Unknown option: $1"
                shift
                ;;
        esac
    done
}

# Main
main() {
    parse_args "$@"

    print_banner
    check_platform

    echo ""
    info "Installation prefix: $PREFIX"
    info "Installation method: $METHOD"
    info "Version: $VERSION"
    echo ""

    case "$METHOD" in
        binary)
            install_binary
            ;;
        source)
            install_source
            ;;
        *)
            die "Unknown installation method: $METHOD (use 'binary' or 'source')"
            ;;
    esac

    print_post_install
}

main "$@"
