# Packaging & Distribution Guide

**Last Updated**: January 16, 2026

This document outlines the packaging strategy for distributing niri-settings across multiple Linux package managers and formats.

---

## Executive Summary

| Format | Priority | User Base Coverage | Complexity |
|--------|----------|-------------------|------------|
| GitHub Releases | 1 | Fallback for all | Easy |
| Flatpak | 2 | Universal (GUI installs) | Medium |
| Deb | 3 | ~50% of Linux users | Medium-Hard |
| AUR / AUR-bin | 4 | Arch + Steam Deck | Easy |
| Brioche | 5 | Personal use | Easy |
| RPM | 6 | Fedora/RHEL/openSUSE | Medium |
| Nix | 7 | NixOS + Nix users | Medium |
| Guix | 8 | Guix users | Medium |

---

## Linux Distribution Market Share (2025)

| Distribution Family | Market Share (of Linux) |
|---------------------|------------------------|
| Ubuntu + derivatives | ~34% |
| Debian + derivatives | ~16% |
| Arch + derivatives | Growing (Steam Deck) |
| Fedora/RHEL/openSUSE | ~15-20% |
| NixOS | Niche (ranked 19th) |
| Guix | Very small (ranked 58th) |

---

## Package Format Details

### Rust Compilation & Caching

| Format | User Compiles? | Rust Toolchain | Dependency Caching |
|--------|---------------|----------------|-------------------|
| Brioche | First build only | Cached after first use | All artifacts cached & shareable |
| Nix | Usually no | Binary cache (cache.nixos.org) | Per-crate with naersk/crane |
| Guix | Usually no | Substitutes (pre-built) | Similar to Nix |
| AUR | Yes (regular) | User's system or makedep | None |
| AUR-bin | No | Not needed | N/A - pre-built |
| Flatpak | No | Flathub builds it | flatpak-cargo-generator |
| Deb | No | Not needed | N/A - pre-built |
| RPM | No | Not needed | N/A - pre-built |

---

## Format-Specific Details

### 1. GitHub Releases (Required)

Pre-built binaries that feed into other package formats.

**Output**: `niri-settings-linux-x86_64.tar.gz`

**CI Workflow**: Build on tag push, attach binary to release.

---

### 2. Flatpak

Best option for non-technical users. Appears in GUI software centers.

**Files needed**:
- `packaging/flatpak/org.niri.Settings.json` (manifest)
- `packaging/flatpak/org.niri.Settings.desktop`
- `packaging/flatpak/org.niri.Settings.metainfo.xml`
- `cargo-sources.json` (generated)

**Build process**:
```bash
# Generate cargo sources for offline build
python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json

# Build
flatpak-builder --force-clean build-dir org.niri.Settings.json
```

**Key considerations**:
- Uses `org.freedesktop.Sdk.Extension.rust-stable` for Rust toolchain
- Requires `flatpak-cargo-generator.py` to vendor dependencies
- Sandbox permissions needed for niri socket + config directory

---

### 3. Deb (Debian/Ubuntu)

**Files needed**:
```
packaging/debian/
├── control
├── rules
├── changelog
├── copyright
└── compat
```

**Build process**:
```bash
dpkg-buildpackage -us -uc -b
```

**Distribution options**:
- PPA (Personal Package Archive)
- Direct `.deb` download from GitHub Releases

---

### 4. AUR (Arch Linux)

Two packages recommended:

**`niri-settings` (build from source)**:
```bash
# PKGBUILD
pkgname=niri-settings
pkgver=0.1.0
pkgrel=1
pkgdesc="Native settings application for the niri Wayland compositor"
arch=('x86_64')
url="https://github.com/USER/niri-settings-rust"
license=('MIT')  # adjust to actual license
depends=('gtk4')  # adjust dependencies
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/niri-settings" "$pkgdir/usr/bin/niri-settings"
}
```

**`niri-settings-bin` (pre-built)**:
```bash
# PKGBUILD
pkgname=niri-settings-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Native settings application for the niri Wayland compositor (binary)"
arch=('x86_64')
url="https://github.com/USER/niri-settings-rust"
license=('MIT')
provides=('niri-settings')
conflicts=('niri-settings')
source=("$url/releases/download/v$pkgver/niri-settings-linux-x86_64.tar.gz")
sha256sums=('...')

package() {
    install -Dm755 "niri-settings" "$pkgdir/usr/bin/niri-settings"
}
```

---

### 5. Brioche

Simple TypeScript-based package definition.

**File**: `packaging/brioche/project.bri`

```typescript
import * as std from "std";
import { cargoBuild } from "rust";

export const project = {
  name: "niri-settings",
  version: "0.1.0",
};

const source = std.download({
  url: "https://github.com/USER/niri-settings-rust/archive/v0.1.0.tar.gz",
  hash: std.sha256Hash("..."),
});

export default function () {
  return cargoBuild({
    source,
    runnable: "bin/niri-settings",
  });
}

export function test() {
  return std.runBash`
    ${cargoBuild({ source })}/bin/niri-settings --version
  `;
}
```

**Build**:
```bash
brioche build -o output/
```

**Features**:
- All build artifacts cached and reusable
- Cache shareable via S3-compatible storage
- Rust toolchain automatically downloaded and cached

---

### 6. RPM (Fedora/RHEL/openSUSE)

**File**: `packaging/rpm/niri-settings.spec`

```spec
Name:           niri-settings
Version:        0.1.0
Release:        1%{?dist}
Summary:        Native settings application for niri compositor

License:        MIT
URL:            https://github.com/USER/niri-settings-rust
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo rust >= 1.70
BuildRequires:  gtk4-devel

%description
Native Rust settings application for the niri Wayland compositor.
Uses Slint for UI and manages KDL config files.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dm755 target/release/niri-settings %{buildroot}%{_bindir}/niri-settings

%files
%license LICENSE
%{_bindir}/niri-settings
```

**Distribution**: COPR (Fedora's equivalent to PPA)

---

### 7. Nix

**File**: `packaging/nix/default.nix`

```nix
{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, gtk4
}:

rustPlatform.buildRustPackage rec {
  pname = "niri-settings";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "USER";
    repo = "niri-settings-rust";
    rev = "v${version}";
    hash = "sha256-...";
  };

  cargoHash = "sha256-...";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ gtk4 ];

  meta = with lib; {
    description = "Native settings application for niri compositor";
    homepage = "https://github.com/USER/niri-settings-rust";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
```

**Flake support** (optional): `packaging/nix/flake.nix`

**Binary cache**: If submitted to nixpkgs, users get pre-built binaries from cache.nixos.org.

---

### 8. Guix

**File**: `packaging/guix/niri-settings.scm`

```scheme
(define-module (niri-settings)
  #:use-module (guix packages)
  #:use-module (guix git-download)
  #:use-module (guix build-system cargo)
  #:use-module ((guix licenses) #:prefix license:))

(define-public niri-settings
  (package
    (name "niri-settings")
    (version "0.1.0")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/USER/niri-settings-rust")
             (commit (string-append "v" version))))
       (file-name (git-file-name name version))
       (sha256
        (base32 "..."))))
    (build-system cargo-build-system)
    (arguments
     `(#:cargo-inputs
       (("rust-slint" ,rust-slint)
        ("rust-kdl" ,rust-kdl)
        ;; ... other dependencies
        )))
    (synopsis "Native settings application for niri compositor")
    (description
     "Native Rust settings application for the niri Wayland compositor.
Uses Slint for UI and manages KDL config files without modifying
the user's main config directly.")
    (home-page "https://github.com/USER/niri-settings-rust")
    (license license:mit)))
```

**Substitutes**: If submitted to Guix proper, users get pre-built binaries.

---

## Directory Structure

```
packaging/
├── flatpak/
│   ├── org.niri.Settings.json
│   ├── org.niri.Settings.desktop
│   └── org.niri.Settings.metainfo.xml
├── debian/
│   ├── control
│   ├── rules
│   ├── changelog
│   ├── copyright
│   └── compat
├── aur/
│   ├── PKGBUILD
│   └── PKGBUILD-bin
├── brioche/
│   └── project.bri
├── rpm/
│   └── niri-settings.spec
├── nix/
│   ├── default.nix
│   └── flake.nix
└── guix/
    └── niri-settings.scm
```

---

## CI/CD Strategy

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/niri-settings

  flatpak:
    needs: build
    # Flathub handles this via their CI

  aur:
    needs: build
    # Update PKGBUILD checksums and push to AUR
```

---

## Recommended Implementation Order

1. **GitHub Releases** - Set up CI to build release binaries
2. **AUR + AUR-bin** - Submit PKGBUILDs to AUR
3. **Flatpak** - Create manifest, submit to Flathub
4. **Brioche** - Personal use, quick setup
5. **Deb** - PPA or direct download
6. **RPM** - COPR repository
7. **Nix** - Submit to nixpkgs or maintain overlay
8. **Guix** - Submit package definition

---

## References

- [Flatpak Rust Guide](https://develop.kde.org/docs/getting-started/rust/rust-flatpak/)
- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [Brioche Documentation](https://brioche.dev/docs/)
- [Nix Rust Guide](https://ryantm.github.io/nixpkgs/languages-frameworks/rust/)
- [Guix Packaging Guide](https://guix.gnu.org/manual/en/html_node/Packaging-Guidelines.html)
- [COPR Documentation](https://docs.pagure.org/copr.copr/)
