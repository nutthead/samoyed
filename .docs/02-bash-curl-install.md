# Installing Programs with curl and bash: A Comprehensive Implementation Guide

**The curl/bash installation pattern has become the de facto standard for quickly installing developer tools across Unix-like systems.** This guide examines different implementation techniques, analyzes their trade-offs, and provides a complete implementation roadmap for Rust projects, using Samoyed as a detailed case study.

## The power and simplicity of single-command installation

The curl | bash pattern revolutionized software distribution by enabling installations with a single memorable command. Rather than requiring users to manually download files, extract archives, and configure their systems, tools like rustup (`curl https://sh.rustup.rs | sh`) and nvm (`curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash`) deliver immediate installation with zero prerequisites beyond curl and bash—tools already present on virtually every Unix-like system.

This approach succeeds because it eliminates friction. Users don't need to understand package managers, dependency chains, or system-specific installation procedures. **The installer script handles platform detection, binary selection, extraction, PATH configuration, and verification automatically**. For developers distributing CLI tools, this pattern has become expected rather than optional.

## Installation techniques from major projects

Analyzing production installers from Homebrew, RVM, NVM, SDKMAN, and Rustup reveals sophisticated patterns that have evolved to handle edge cases and ensure reliability across diverse environments.

### Two-stage installation architecture

**Modern installers universally employ a two-stage approach**: a lightweight shell script performs environment validation and bootstrapping, then downloads and executes the actual installer. This pattern prevents partial execution failures and enables sophisticated error handling.

Rustup exemplifies this best. The shell script at `sh.rustup.rs` is minimal—it detects the platform, downloads the appropriate `rustup-init` binary for that architecture, verifies it, then executes it. The real installation logic lives in the compiled binary, which provides superior reliability, better error messages, and the ability to implement complex features like component management and self-updates.

```bash
# Rustup's approach - minimal shell script
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Behind the scenes:
# 1. Detect platform (x86_64-unknown-linux-gnu, etc.)
# 2. Download rustup-init binary for that platform
# 3. Verify checksum (optional but available)
# 4. Execute: ./rustup-init [args]
```

RVM takes a similar approach but implements more logic in the shell script itself, demonstrating the trade-off between shell-based and binary-based implementations. The shell script handles version resolution, GPG verification, and multiple installation modes before downloading the actual RVM installation archive.

### Platform and architecture detection

**Every installer must accurately identify the host system to download the correct binary**. The standard approach uses `uname` commands combined with case statements:

```bash
detect_platform() {
  local os arch
  
  # Detect operating system
  case "$(uname -s)" in
    Linux*)   os='linux';;
    Darwin*)  os='darwin';;
    CYGWIN*|MINGW*|MSYS*) os='windows';;
    *)        echo "Unsupported OS: $(uname -s)" >&2; exit 1;;
  esac
  
  # Detect CPU architecture
  case "$(uname -m)" in
    x86_64|amd64)    arch='x86_64';;
    aarch64|arm64)   arch='aarch64';;
    armv7l)          arch='armv7';;
    i386|i686)       arch='i686';;
    *)               echo "Unsupported architecture: $(uname -m)" >&2; exit 1;;
  esac
  
  echo "${os}-${arch}"
}
```

SDKMAN demonstrates comprehensive architecture detection across Linux variants, macOS (Intel and Apple Silicon), and even Solaris, constructing platform identifiers like `linuxx64`, `darwinarm64`, and `linuxarm32`. **This level of detail ensures users get optimized binaries rather than falling back to universal builds**.

Homebrew adds sophistication by detecting not just the architecture but also the appropriate installation prefix—`/opt/homebrew` for Apple Silicon Macs versus `/usr/local` for Intel Macs. This shows how platform detection extends beyond just identifying the OS to handling platform-specific conventions.

### Download strategies and retry logic

**Network reliability cannot be assumed**. Production installers implement retry mechanisms with exponential backoff:

```bash
retry() {
  local tries="$1" n="$1" pause=2
  shift
  while [[ $((--n)) -gt 0 ]]; do
    if "$@"; then return 0; fi
    warn "Retrying in ${pause} seconds..."
    sleep "${pause}"
    ((pause *= 2))
  done
  abort "Failed after ${tries} attempts"
}

# Usage
retry 5 curl -fsSL "${url}" -o "${output}"
```

Homebrew's implementation demonstrates best practices: it uses `--fail` to catch HTTP errors, `--location` to follow redirects, and implements comprehensive timeout handling. The retry logic doubles the pause duration after each failure, preventing overwhelming a temporarily unavailable server while still providing reasonable user experience.

RVM implements an interesting fallback strategy—if GitHub fails, it automatically tries Bitbucket. This multi-source approach significantly improves reliability for users in regions with poor connectivity to specific hosting providers.

### Shell profile modification and PATH management

**Adding installed binaries to the user's PATH requires detecting which shell they use and modifying the appropriate configuration file**. Every installer handles this differently:

```bash
# NVM's comprehensive profile detection
nvm_detect_profile() {
  if [ "${SHELL#*bash}" != "$SHELL" ]; then
    if [ -f "$HOME/.bashrc" ]; then
      DETECTED_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      DETECTED_PROFILE="$HOME/.bash_profile"
    fi
  elif [ "${SHELL#*zsh}" != "$SHELL" ]; then
    DETECTED_PROFILE="$HOME/.zshrc"
  elif [ "${SHELL#*fish}" != "$SHELL" ]; then
    DETECTED_PROFILE="$HOME/.config/fish/config.fish"
  fi
  echo "$DETECTED_PROFILE"
}
```

Homebrew takes a system-level approach on macOS by adding entries to `/etc/paths.d/homebrew`, which automatically affects all users and shells. For user-specific installations, it writes shell-specific initialization code to the detected profile.

**All modern installers support a non-interactive mode** that prevents shell modification—critical for CI/CD environments:

```bash
# Various non-interactive patterns
NONINTERACTIVE=1 bash -c "$(curl ...)"          # Homebrew
PROFILE=/dev/null bash -c "$(curl ...)"         # NVM
curl ... | sh -s -- --no-modify-path -y         # Rustup
curl -s "https://get.sdkman.io?rcupdate=false"  # SDKMAN
```

### Security implementation patterns

**Installers implement security at multiple levels, though approaches vary significantly**. The most comprehensive is RVM's GPG signature verification:

```bash
verify_package_pgp() {
  if "${rvm_gpg_command}" --verify "$2" "$1"; then
    log "GPG verified '$1'"
  else
    log "GPG signature verification failed!"
    log "Import key: gpg --keyserver hkp://keyserver.ubuntu.com --recv-keys 409B6B1796C275462A1703113804BB82D39DC0E3"
    exit 1
  fi
}
```

Rustup enforces TLS 1.2+ and HTTPS-only with `curl --proto '=https' --tlsv1.2`, preventing downgrade attacks. While it doesn't verify checksums by default in the quick-install script, checksums are available at `https://static.rust-lang.org/rustup/dist/{target}/rustup-init.sha256` for users who want to verify before executing.

Most installers use strict error handling with `set -euo pipefail`:
- `set -e`: Exit immediately on any command failure
- `set -u`: Exit on undefined variable access  
- `set -o pipefail`: Catch failures in pipes (e.g., `curl | tar`)

**Homebrew demonstrates transparent security**: it shows exactly what it will do before requiring confirmation, validates that it's not running as root unnecessarily, and checks sudo access before starting potentially dangerous operations.

## Rust ecosystem installation patterns

The Rust ecosystem has developed particularly mature installation patterns, driven by the need to distribute pre-compiled binaries for performance-critical tools across diverse platforms.

### The cargo-binstall revolution

**cargo-binstall emerged as the definitive solution for installing Rust CLI tools from pre-built binaries**. It automates the entire process: detecting platform, querying GitHub releases, downloading the correct binary, and installing it to `$CARGO_HOME/bin`.

```bash
# cargo-binstall installation
curl -L --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Then use it to install other tools
cargo binstall ripgrep bat fd-find starship
```

The installer demonstrates the two-stage pattern: download a minimal installer binary, which then installs itself through its own installation logic. This self-installation approach (`./cargo-binstall -y --force cargo-binstall`) is elegant and ensures the installer uses its production code path rather than duplicate logic in the shell script.

cargo-binstall supports sophisticated configuration through Cargo.toml metadata, allowing project maintainers to specify custom release URL patterns and handle platform-specific differences:

```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.{ archive-format }"
pkg-fmt = "tgz"
bin-dir = "{ bin }{ binary-ext }"

[package.metadata.binstall.overrides.x86_64-pc-windows-msvc]
pkg-fmt = "zip"
```

**The key insight is that standardized release naming makes tools automatically discoverable**. If a project publishes releases named `tool-x86_64-unknown-linux-musl.tar.gz`, cargo-binstall can install it without any configuration.

### Release asset naming conventions

**Rust tools have converged on a standard naming pattern** that enables automatic binary selection:

```
{binary-name}-{version}-{target-triple}.{archive-format}

Examples:
cargo-binstall-x86_64-unknown-linux-musl.tgz
starship-x86_64-apple-darwin.tar.gz
bat-v0.24.0-x86_64-pc-windows-msvc.zip
ripgrep-14.1.1-aarch64-unknown-linux-gnu.tar.gz
```

Target triples follow Rust's platform naming:
- **Linux GNU**: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- **Linux musl** (static): `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- **macOS**: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- **Windows**: `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu`

The musl targets are particularly valuable because they produce fully static binaries with no libc dependencies, ensuring maximum compatibility across Linux distributions.

### Installation script generators

**Several frameworks have emerged to automate installation script generation**:

**instl.sh** provides zero-setup installation for any GitHub project with releases. It analyzes the repository structure server-side and generates an installation script on-the-fly:

```bash
# Works for any GitHub project with releases
curl -sSL instl.sh/sharkdp/bat/linux | bash
curl -sSL instl.sh/casey/just/macos | bash
```

**webinstall.dev** offers memorable short URLs for popular tools:

```bash
curl https://webi.sh/rg | sh      # ripgrep
curl https://webi.sh/zoxide | sh  # zoxide
```

These services solve the cold-start problem: projects without custom installation scripts still get easy one-command installation, **encouraging more projects to publish pre-built binaries**.

### Starship and zoxide patterns

Both starship and zoxide demonstrate minimalist installers that focus on simplicity:

```bash
# Starship with custom binary directory
curl -sS https://starship.rs/install.sh | sh -s -- --bin-dir "$HOME/.local/bin"

# zoxide
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

Their installers are straightforward: detect platform, construct download URL, download and extract, install to an appropriate directory, provide setup instructions. **The key insight is that simplicity often beats sophistication**—most users just want the binary in their PATH.

## Comprehensive pros and cons analysis

Understanding the trade-offs of curl/bash installation helps inform implementation decisions.

### Advantages that drive adoption

**Zero prerequisites beyond curl and bash**: Every Unix-like system has these tools, making installation universally accessible. Users don't need root access, package managers, or build toolchains.

**Single-command installation**: The pattern is memorable and documentable. Compare `curl https://sh.rustup.rs | sh` to multi-step package manager installations or manual binary extraction.

**Cross-platform from one script**: A well-written shell script handles Linux distributions, macOS versions, and even Windows WSL from a single codebase. The installer detects platform differences and handles them automatically.

**Perfect for documentation**: The one-liner fits cleanly into README files and getting-started guides. Users can copy-paste a single command rather than following multi-step procedures.

**Automatic updates possible**: Installers can be updated independently of releases. Bug fixes in the installation logic benefit all new users without requiring new releases.

**Excellent for CI/CD**: Non-interactive installation works perfectly in automated environments. Services like GitHub Actions can install tools in one line without complex dependency management.

### Security concerns and mitigation strategies

**Executing arbitrary code from the internet is inherently risky**. The pattern asks users to trust both the source (the domain) and the transport (HTTPS). A compromised server or man-in-the-middle attack could deliver malicious code that executes with the user's privileges.

Mitigation strategies include:
- **Enforce HTTPS with TLS 1.2+**: `curl --proto '=https' --tlsv1.2` prevents downgrade attacks
- **Provide inspection alternatives**: Encourage `curl -fsSL https://url -o install.sh && less install.sh && bash install.sh`
- **Publish checksums**: Let users verify scripts before execution
- **Sign scripts with GPG**: Enable cryptographic verification of authenticity
- **Make scripts transparent**: Clear, readable code enables security audits

**The user experience trade-off is real**: requiring verification steps defeats the simplicity advantage. Most projects balance this by making quick installation the default while documenting secure verification for sensitive environments.

### Platform limitations

**Windows presents significant challenges**. While Git Bash and WSL provide bash environments, line ending issues cause frequent failures. CRLF line endings (`\r\n`) versus Unix LF (`\n`) break script execution with cryptic errors like `/bin/bash^M: bad interpreter`.

The solution is separate Windows installers—either PowerShell scripts (`iwr https://url | iex`) or native executables. Rustup demonstrates the gold standard: `sh.rustup.rs` for Unix, `win.rustup.rs` for Windows, with clear platform detection on the landing page.

**Binary distribution requires multiple builds**: Unlike package managers that compile on the target system, curl/bash installers must ship pre-built binaries for every platform/architecture combination. This increases release complexity and storage requirements.

**Shell compatibility issues** occasionally arise. While POSIX shell features work broadly, some installers assume bash-specific behavior. The solution is either strict POSIX compliance or requiring bash explicitly in the shebang (`#!/usr/bin/env bash`).

### Operational considerations

**Version pinning is less obvious**: Unlike package managers with explicit version selection, users typically get the "latest" release. While installers can accept version arguments (`curl https://url | bash -s -- v1.2.3`), this pattern is less discoverable.

**Uninstallation lacks standardization**: Package managers provide `apt remove`, `brew uninstall`, etc. For curl/bash installations, projects must provide separate uninstall scripts or documentation on manual removal.

**No automatic updates**: Package managers track installed software and provide update mechanisms. Manually installed binaries require users to re-run the installer or implement self-update functionality in the tool itself.

**Limited rollback capabilities**: If an installation fails midway, the system may be in an inconsistent state. Package managers handle transactions atomically; shell scripts require careful error handling and cleanup logic.

Despite these limitations, **the curl/bash pattern remains dominant for developer tools** because the advantages—simplicity, memorability, and cross-platform support—outweigh the drawbacks for this use case.

## Complete implementation guide for Rust projects

Implementing curl/bash installation for a Rust project requires both release infrastructure and an installation script. This guide covers the complete process.

### Phase 1: Setting up GitHub Actions for releases

**Before creating an installation script, you need binaries to install**. GitHub Actions provides free build infrastructure for open-source projects.

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          # Linux builds
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            cross: true
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            cross: true
          - os: ubuntu-latest
            target: armv7-unknown-linux-musleabihf
            cross: true
          
          # macOS builds
          - os: macos-latest
            target: x86_64-apple-darwin
            cross: false
          - os: macos-latest
            target: aarch64-apple-darwin
            cross: false
          
          # Windows builds
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            cross: false

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross-compilation tools
        if: matrix.cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build binary
        run: |
          if [ "${{ matrix.cross }}" == "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
        shell: bash

      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            7z a ../../../${{ github.event.repository.name }}-${{ matrix.target }}.zip ${{ github.event.repository.name }}.exe
          else
            tar czf ../../../${{ github.event.repository.name }}-${{ matrix.target }}.tar.gz ${{ github.event.repository.name }}
          fi
        shell: bash

      - name: Generate checksum
        run: |
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            certutil -hashfile ${{ github.event.repository.name }}-${{ matrix.target }}.zip SHA256 | findstr /v "hash" > ${{ github.event.repository.name }}-${{ matrix.target }}.zip.sha256
          else
            shasum -a 256 ${{ github.event.repository.name }}-${{ matrix.target }}.tar.gz > ${{ github.event.repository.name }}-${{ matrix.target }}.tar.gz.sha256
          fi
        shell: bash

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: |
            *.tar.gz
            *.zip
            *.sha256

  release:
    name: Create release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          generate_release_notes: true
```

**This workflow triggers on version tags** (e.g., `git tag v0.2.0 && git push --tags`), builds binaries for all major platforms, generates SHA256 checksums, and creates a GitHub Release with all assets attached.

The musl targets are particularly important for Linux—they produce fully static binaries that work on any Linux distribution without libc version dependencies.

### Phase 2: Creating the installation script

Create `install.sh` at the repository root:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Configuration
readonly REPO="username/projectname"
readonly BINARY="projectname"
readonly INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

# Logging functions
info() { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
die() { error "$*"; exit 1; }

# Detect platform
detect_platform() {
  local os arch target
  
  # Operating system detection
  case "$(uname -s)" in
    Linux*)   os='unknown-linux-musl';;
    Darwin*)  os='apple-darwin';;
    *)        die "Unsupported operating system: $(uname -s)";;
  esac
  
  # Architecture detection
  case "$(uname -m)" in
    x86_64|amd64) arch='x86_64';;
    aarch64|arm64) arch='aarch64';;
    armv7l) arch='armv7';;
    *) die "Unsupported architecture: $(uname -m)";;
  esac
  
  target="${arch}-${os}"
  echo "$target"
}

# Get latest release tag
get_latest_version() {
  info "Fetching latest release information..."
  
  local api_url="https://api.github.com/repos/${REPO}/releases/latest"
  local version
  
  # Use GitHub API to get latest version
  version=$(curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/') || \
    die "Failed to fetch release information"
  
  echo "$version"
}

# Download and verify binary
download_and_verify() {
  local version="$1"
  local target="$2"
  local tmpdir="$3"
  
  local archive="${BINARY}-${target}.tar.gz"
  local url="https://github.com/${REPO}/releases/download/v${version}/${archive}"
  local checksum_url="${url}.sha256"
  
  info "Downloading ${BINARY} v${version} for ${target}..."
  
  # Download binary archive
  if ! curl -fsSL --proto '=https' --tlsv1.2 "$url" -o "${tmpdir}/${archive}"; then
    die "Download failed. Please check your internet connection and try again."
  fi
  
  # Download and verify checksum
  if curl -fsSL "$checksum_url" -o "${tmpdir}/checksum" 2>/dev/null; then
    info "Verifying checksum..."
    
    local expected_checksum
    expected_checksum=$(awk '{print $1}' "${tmpdir}/checksum")
    
    local actual_checksum
    if command -v sha256sum >/dev/null; then
      actual_checksum=$(sha256sum "${tmpdir}/${archive}" | awk '{print $1}')
    elif command -v shasum >/dev/null; then
      actual_checksum=$(shasum -a 256 "${tmpdir}/${archive}" | awk '{print $1}')
    else
      warn "No checksum utility found, skipping verification"
      return 0
    fi
    
    if [ "$actual_checksum" != "$expected_checksum" ]; then
      error "Checksum verification failed!"
      error "Expected: $expected_checksum"
      error "Got: $actual_checksum"
      die "Aborting installation for security reasons"
    fi
    
    info "Checksum verified successfully"
  else
    warn "Checksum file not available, skipping verification"
  fi
}

# Extract and install binary
install_binary() {
  local tmpdir="$1"
  local target="$2"
  local archive="${BINARY}-${target}.tar.gz"
  
  info "Extracting..."
  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir" || \
    die "Failed to extract archive"
  
  # Create install directory
  mkdir -p "$INSTALL_DIR" || \
    die "Failed to create installation directory: $INSTALL_DIR"
  
  # Install binary
  info "Installing to ${INSTALL_DIR}/${BINARY}..."
  cp "${tmpdir}/${BINARY}" "${INSTALL_DIR}/" || \
    die "Failed to copy binary"
  
  chmod 755 "${INSTALL_DIR}/${BINARY}" || \
    die "Failed to set executable permissions"
}

# Configure PATH
configure_path() {
  # Check if already in PATH
  if echo "$PATH" | grep -q "$INSTALL_DIR"; then
    return 0
  fi
  
  info "$INSTALL_DIR is not in your PATH"
  
  # Detect shell and appropriate config file
  local shell_config=""
  if [ -n "${BASH_VERSION:-}" ]; then
    shell_config="$HOME/.bashrc"
  elif [ -n "${ZSH_VERSION:-}" ]; then
    shell_config="$HOME/.zshrc"
  else
    # Generic fallback
    shell_config="$HOME/.profile"
  fi
  
  # Add to PATH
  {
    echo ""
    echo "# Added by ${BINARY} installer"
    echo "export PATH=\"${INSTALL_DIR}:\$PATH\""
  } >> "$shell_config"
  
  info "Added ${INSTALL_DIR} to PATH in ${shell_config}"
  warn "Run 'source ${shell_config}' or restart your shell to use ${BINARY}"
}

# Main installation function
main() {
  info "Installing ${BINARY}..."
  
  # Check prerequisites
  for cmd in curl tar; do
    command -v "$cmd" >/dev/null || die "Required command not found: $cmd"
  done
  
  # Detect platform
  local target
  target=$(detect_platform)
  info "Detected platform: $target"
  
  # Get latest version
  local version
  version=$(get_latest_version)
  info "Latest version: $version"
  
  # Create temporary directory
  local tmpdir
  tmpdir=$(mktemp -d) || die "Failed to create temporary directory"
  trap 'rm -rf "$tmpdir"' EXIT
  
  # Download and verify
  download_and_verify "$version" "$target" "$tmpdir"
  
  # Install
  install_binary "$tmpdir" "$target"
  
  # Configure PATH
  configure_path
  
  info "Installation complete!"
  info "Try running: ${BINARY} --version"
}

# Execute
main "$@"
```

**This script implements all best practices**: strict error handling, platform detection, checksum verification, secure downloads with TLS enforcement, clean temporary file handling, and intelligent PATH configuration.

### Phase 3: Testing across platforms

**Test the installation script on multiple platforms before announcing it**:

```bash
# Local testing
bash install.sh

# Test in fresh environments
docker run -it --rm ubuntu:latest bash -c "apt-get update && apt-get install -y curl && curl -fsSL https://raw.githubusercontent.com/user/repo/main/install.sh | bash"

# Test on macOS (Intel and Apple Silicon if available)
# Test in WSL on Windows
```

Create a pre-release (e.g., `v0.2.1-rc1`) for testing without affecting stable users.

### Phase 4: Documentation

Update your README.md:

```markdown
## Installation

### Quick Install (Linux, macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/username/projectname/main/install.sh | bash
```

### Install Specific Version

```bash
curl -fsSL https://raw.githubusercontent.com/username/projectname/main/install.sh | bash -s -- v0.2.0
```

### Custom Installation Directory

```bash
curl -fsSL https://raw.githubusercontent.com/username/projectname/main/install.sh | INSTALL_DIR=~/bin bash
```

### Verify Before Installing

```bash
curl -fsSL https://raw.githubusercontent.com/username/projectname/main/install.sh -o install.sh
less install.sh  # Review the script
bash install.sh
```

### Alternative: cargo install

```bash
cargo install projectname
```

### Windows

Windows users should use WSL or install via cargo.

### Uninstalling

```bash
rm ~/.local/bin/projectname
# Then remove the PATH entry from your shell config
```
```

## Samoyed case study: Complete implementation

Samoyed, a fast Rust-based Git hooks manager, provides an ideal case study because it currently lacks pre-built binaries entirely—making this a greenfield implementation that demonstrates the full process from zero to production-ready curl/bash installation.

### Understanding Samoyed's architecture

**Samoyed is a single-binary tool with zero runtime dependencies**—perfect for straightforward distribution. Written in approximately 1,000 lines of Rust, it compiles to a self-contained executable that embeds its shell wrapper script via `include_bytes!`. The architecture is deliberately minimal:

```
samoyed/
├── src/
│   └── main.rs          # Complete implementation
├── assets/
│   └── samoyed          # POSIX shell wrapper (embedded)
├── tests/
│   └── integration/     # Shell-based integration tests
└── Cargo.toml
```

This simplicity is a **significant advantage for installation**—no external assets, configuration files, or dependencies to manage. Users need only the binary in their PATH.

Samoyed replaces Husky for managing Git hooks, offering better performance and fewer dependencies. After installation, users run `samoyed init` in their repository, which creates a `.samoyed/_/` directory with hook scripts and configures Git's `core.hooksPath`.

### Critical finding: No release infrastructure exists

**The most important discovery is that Samoyed currently publishes no pre-built binaries**. GitHub Releases contains no releases, and there's no CI/CD infrastructure for automated builds. Installation is only possible via:

```bash
cargo install samoyed  # Requires Rust toolchain
```

This means implementing curl/bash installation requires **building the entire release pipeline first**. You cannot write an installation script without binaries to install. This is actually common for young Rust projects that haven't yet scaled beyond Rust-developer users.

### Step 1: Implementing GitHub Actions for multi-platform builds

Create `.github/workflows/release.yml`:

```yaml
name: Release Samoyed

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create GitHub Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: false
          prerelease: false

  build:
    name: Build ${{ matrix.target }}
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          # Linux builds (prioritize musl for static linking)
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            use_cross: true
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            use_cross: true
          
          # macOS builds (Intel and Apple Silicon)
          - os: macos-latest
            target: x86_64-apple-darwin
            use_cross: false
          - os: macos-latest
            target: aarch64-apple-darwin
            use_cross: false

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross
        run: |
          cargo install cross --git https://github.com/cross-rs/cross

      - name: Build binary
        run: |
          if [ "${{ matrix.use_cross }}" == "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi

      - name: Strip binary (Linux and macOS)
        run: |
          strip target/${{ matrix.target }}/release/samoyed || true

      - name: Create archive
        id: archive
        run: |
          cd target/${{ matrix.target }}/release
          ARCHIVE="samoyed-${{ matrix.target }}.tar.gz"
          tar czf "$ARCHIVE" samoyed
          echo "archive_name=$ARCHIVE" >> $GITHUB_OUTPUT
          echo "archive_path=target/${{ matrix.target }}/release/$ARCHIVE" >> $GITHUB_OUTPUT

      - name: Generate checksum
        run: |
          cd target/${{ matrix.target }}/release
          shasum -a 256 ${{ steps.archive.outputs.archive_name }} > ${{ steps.archive.outputs.archive_name }}.sha256

      - name: Upload release asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ${{ steps.archive.outputs.archive_path }}
          asset_name: ${{ steps.archive.outputs.archive_name }}
          asset_content_type: application/gzip

      - name: Upload checksum
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ${{ steps.archive.outputs.archive_path }}.sha256
          asset_name: ${{ steps.archive.outputs.archive_name }}.sha256
          asset_content_type: text/plain
```

**This workflow automates the entire release process**. When you push a tag (e.g., `git tag v0.2.1 && git push origin v0.2.1`), GitHub Actions builds binaries for major platforms, generates SHA256 checksums, and publishes everything as a GitHub Release.

The use of musl targets for Linux is **critical for Samoyed**—it ensures the binary runs on any Linux distribution without glibc version dependencies. This is especially important for a Git hooks tool that might run in diverse CI/CD environments.

### Step 2: Adding a --version flag

Before users can verify installation, Samoyed needs a `--version` flag. Add to `src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".samoyed")]
        dirname: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    // version is automatically handled by clap
    // --version will print the version from Cargo.toml
    
    match &cli.command {
        Some(Commands::Init { dirname }) => {
            // existing init logic
        }
        None => {
            eprintln!("Run 'samoyed init' to set up Git hooks");
            std::process::exit(1);
        }
    }
}
```

With clap's `#[command(version)]` attribute, running `samoyed --version` will print the version from `Cargo.toml`.

### Step 3: Creating Samoyed's installation script

Create `install.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Samoyed installer
# https://github.com/nutthead/samoyed

readonly REPO="nutthead/samoyed"
readonly BINARY="samoyed"
readonly INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# Logging
info() { echo -e "${GREEN}==>${NC} $*"; }
warn() { echo -e "${YELLOW}Warning:${NC} $*" >&2; }
error() { echo -e "${RED}Error:${NC} $*" >&2; }
die() { error "$*"; exit 1; }

# Banner
show_banner() {
  echo -e "${BLUE}"
  cat << 'EOF'
  ____                                   _ 
 / ___|  __ _ _ __ ___   ___  _   _  __| |
 \___ \ / _` | '_ ` _ \ / _ \| | | |/ _` |
  ___) | (_| | | | | | | (_) | |_| | (_| |
 |____/ \__,_|_| |_| |_|\___/ \__, |\__,_|
                              |___/        
Fast Git hooks manager in Rust
EOF
  echo -e "${NC}"
}

# Detect platform and architecture
detect_platform() {
  local os arch
  
  case "$(uname -s)" in
    Linux*)   os='unknown-linux-musl';;
    Darwin*)  os='apple-darwin';;
    CYGWIN*|MINGW*|MSYS*)
      warn "Detected Windows environment"
      echo ""
      echo "For Windows, please use one of:"
      echo "  1. WSL (Windows Subsystem for Linux) - recommended"
      echo "  2. cargo install samoyed"
      echo ""
      die "Native Windows installation not supported"
      ;;
    *) die "Unsupported operating system: $(uname -s)";;
  esac
  
  case "$(uname -m)" in
    x86_64|amd64)
      arch='x86_64'
      ;;
    aarch64|arm64)
      arch='aarch64'
      ;;
    *)
      die "Unsupported architecture: $(uname -m)"
      ;;
  esac
  
  echo "${arch}-${os}"
}

# Get latest release version
get_latest_version() {
  local api_url="https://api.github.com/repos/${REPO}/releases/latest"
  
  # Try to get version from GitHub API
  local version
  version=$(curl -fsSL "$api_url" 2>/dev/null | \
    grep '"tag_name":' | \
    sed -E 's/.*"v?([^"]+)".*/\1/') || {
    # Fallback to checking /releases/latest redirect
    warn "Could not query GitHub API, trying direct redirect"
    version=$(curl -fsSL -o /dev/null -w '%{redirect_url}' \
      "https://github.com/${REPO}/releases/latest" | \
      sed 's|.*/v\{0,1\}||')
  }
  
  if [ -z "$version" ]; then
    die "Could not determine latest version"
  fi
  
  # Remove 'v' prefix if present
  version="${version#v}"
  echo "$version"
}

# Download and verify
download_and_verify() {
  local version="$1"
  local target="$2"
  local tmpdir="$3"
  
  local archive="${BINARY}-${target}.tar.gz"
  local url="https://github.com/${REPO}/releases/download/v${version}/${archive}"
  local checksum_url="${url}.sha256"
  
  info "Downloading Samoyed v${version} for ${target}..."
  
  # Download with progress
  if ! curl -fL --proto '=https' --tlsv1.2 \
    --progress-bar "$url" -o "${tmpdir}/${archive}"; then
    error "Download failed from: $url"
    die "Please check your internet connection and try again"
  fi
  
  # Download and verify checksum
  info "Verifying checksum..."
  if ! curl -fsSL "$checksum_url" -o "${tmpdir}/checksum" 2>/dev/null; then
    warn "Checksum file not available, skipping verification"
    return 0
  fi
  
  local expected_checksum
  expected_checksum=$(awk '{print $1}' "${tmpdir}/checksum")
  
  local actual_checksum
  if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum=$(sha256sum "${tmpdir}/${archive}" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual_checksum=$(shasum -a 256 "${tmpdir}/${archive}" | awk '{print $1}')
  else
    warn "No checksum utility found (sha256sum or shasum)"
    warn "Skipping checksum verification"
    return 0
  fi
  
  if [ "$actual_checksum" != "$expected_checksum" ]; then
    error "Checksum verification failed!"
    error "Expected: $expected_checksum"
    error "Got:      $actual_checksum"
    die "Aborting for security reasons"
  fi
  
  info "Checksum verified ✓"
}

# Extract and install
install_binary() {
  local tmpdir="$1"
  local target="$2"
  local archive="${BINARY}-${target}.tar.gz"
  
  info "Extracting..."
  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir"
  
  # Verify binary exists and is executable
  if [ ! -f "${tmpdir}/${BINARY}" ]; then
    die "Binary not found in archive"
  fi
  
  # Test execution (sanity check)
  if ! "${tmpdir}/${BINARY}" --version >/dev/null 2>&1; then
    warn "Binary verification failed, but continuing anyway"
  fi
  
  # Create install directory
  if [ ! -d "$INSTALL_DIR" ]; then
    info "Creating ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR" || die "Failed to create $INSTALL_DIR"
  fi
  
  # Install
  info "Installing to ${INSTALL_DIR}/${BINARY}..."
  cp "${tmpdir}/${BINARY}" "${INSTALL_DIR}/"
  chmod 755 "${INSTALL_DIR}/${BINARY}"
}

# Check if Git is available
check_git() {
  if ! command -v git >/dev/null 2>&1; then
    warn "Git not found in PATH"
    warn "Samoyed requires Git to function"
    echo ""
    echo "Install Git first:"
    echo "  Ubuntu/Debian: sudo apt-get install git"
    echo "  macOS: brew install git"
    echo ""
  fi
}

# Configure PATH
configure_path() {
  if echo "$PATH" | grep -q "$INSTALL_DIR"; then
    return 0  # Already in PATH
  fi
  
  warn "${INSTALL_DIR} is not in your PATH"
  
  # Detect shell
  local shell_config=""
  local shell_name=""
  
  if [ -n "${BASH_VERSION:-}" ]; then
    shell_config="$HOME/.bashrc"
    shell_name="bash"
  elif [ -n "${ZSH_VERSION:-}" ]; then
    shell_config="$HOME/.zshrc"
    shell_name="zsh"
  else
    shell_config="$HOME/.profile"
    shell_name="your shell"
  fi
  
  # Append to shell config
  {
    echo ""
    echo "# Added by Samoyed installer"
    echo 'export PATH="'"$INSTALL_DIR"':$PATH"'
  } >> "$shell_config"
  
  info "Added ${INSTALL_DIR} to PATH in ${shell_config}"
  echo ""
  echo "To use Samoyed immediately, run:"
  echo "  source ${shell_config}"
  echo ""
  echo "Or restart your ${shell_name} session"
}

# Show completion message
show_completion() {
  echo ""
  info "Installation complete! 🎉"
  echo ""
  echo "To get started:"
  echo "  cd your-git-repository"
  echo "  samoyed init"
  echo ""
  echo "For more information:"
  echo "  https://github.com/${REPO}"
  echo ""
}

# Main installation
main() {
  show_banner
  
  # Check prerequisites
  for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      die "Required command not found: $cmd"
    fi
  done
  
  # Detect platform
  local target
  target=$(detect_platform)
  info "Detected platform: ${target}"
  
  # Get version
  local version
  version="${1:-$(get_latest_version)}"
  info "Installing version: ${version}"
  
  # Create temp directory
  local tmpdir
  tmpdir=$(mktemp -d) || die "Failed to create temporary directory"
  trap 'rm -rf "$tmpdir"' EXIT INT TERM
  
  # Download and verify
  download_and_verify "$version" "$target" "$tmpdir"
  
  # Install
  install_binary "$tmpdir" "$target"
  
  # Configure PATH
  configure_path
  
  # Check for Git
  check_git
  
  # Done
  show_completion
}

# Execute with any arguments passed
main "$@"
```

**This installation script is production-ready**. It includes comprehensive error handling, checksum verification, platform detection with helpful error messages for unsupported platforms, PATH configuration with shell detection, and a polished user experience with colors and clear output.

### Step 4: Creating an uninstall script

Create `uninstall.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

readonly BINARY="samoyed"
readonly INSTALL_LOCATIONS=(
  "$HOME/.local/bin/$BINARY"
  "$HOME/bin/$BINARY"
  "/usr/local/bin/$BINARY"
)

echo "Uninstalling Samoyed..."
echo ""

found=false

# Check common installation locations
for location in "${INSTALL_LOCATIONS[@]}"; do
  if [ -f "$location" ]; then
    echo "Found: $location"
    rm -f "$location" && echo "  Removed ✓" || echo "  Failed to remove"
    found=true
  fi
done

if ! $found; then
  echo "Samoyed binary not found in common locations"
  echo ""
  echo "Checked:"
  printf '  %s\n' "${INSTALL_LOCATIONS[@]}"
  echo ""
  echo "If installed elsewhere, remove manually:"
  echo "  which samoyed"
  echo "  rm \$(which samoyed)"
fi

echo ""
echo "Note: Samoyed configuration remains in your repositories (.samoyed/ directories)"
echo "To fully remove:"
echo "  cd your-repo"
echo "  rm -rf .samoyed"
echo "  git config --unset core.hooksPath"
echo ""
```

### Step 5: Documentation for README

Update the README.md:

```markdown
# Samoyed 🐕

Ultra-fast Git hooks manager written in Rust. A modern alternative to Husky with zero runtime dependencies.

## Features

- ⚡ **Blazingly fast** - Written in Rust, compiled to native code
- 🛡️ **Zero runtime dependencies** - Single static binary
- 🎯 **Simple** - One command to set up: `samoyed init`
- 🌍 **Cross-platform** - Linux, macOS, Windows (WSL)
- 🔒 **Secure** - No npm scripts, no JS runtime required

## Quick Start

### Installation

**Linux and macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | bash
```

**Windows (WSL):**

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | bash
```

**Using Cargo:**

```bash
cargo install samoyed
```

### Initialize in your repository

```bash
cd your-git-repo
samoyed init
```

This creates `.samoyed/_/` directory with all Git hooks and sets up Git's `core.hooksPath`.

### Add your first hook

Edit `.samoyed/pre-commit`:

```bash
#!/bin/sh
echo "Running pre-commit hook..."
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Make it executable and commit:

```bash
chmod +x .samoyed/pre-commit
git add .samoyed/
git commit -m "Add pre-commit hook"
```

## Environment Variables

- `SAMOYED=0` - Bypass all hooks (for emergencies)
- `SAMOYED=2` - Debug mode (enables `set -x` in hook wrapper)

## Advanced Installation

### Install specific version

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | bash -s -- v0.2.0
```

### Custom installation directory

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | INSTALL_DIR=~/bin bash
```

### Verify before installing

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh -o install.sh
less install.sh  # Review the script
bash install.sh
```

## Uninstalling

```bash
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/uninstall.sh | bash
```

Or manually:

```bash
rm ~/.local/bin/samoyed
# Remove PATH entry from your shell config if added
```

To remove from a repository:

```bash
rm -rf .samoyed
git config --unset core.hooksPath
```

## Comparison with Husky

| Feature | Samoyed | Husky |
|---------|---------|-------|
| Language | Rust | JavaScript |
| Runtime | None | Node.js |
| Performance | Native speed | V8 interpreter |
| Binary size | ~1MB | ~20MB (with Node) |
| Installation | Single binary | npm package |
| Dependencies | Zero | Many |

## Building from Source

```bash
git clone https://github.com/nutthead/samoyed
cd samoyed
cargo build --release
```

Binary will be at `target/release/samoyed`.

## License

MIT License - see LICENSE file for details.
```

### Step 6: Testing the complete flow

**Test the entire release and installation process**:

1. **Create a pre-release to test infrastructure:**

```bash
git tag v0.2.1-rc1
git push origin v0.2.1-rc1
```

2. **Monitor GitHub Actions** to ensure builds succeed

3. **Test installation from the pre-release:**

```bash
# Edit install.sh temporarily to use the pre-release tag
bash install.sh
samoyed --version
```

4. **Test on multiple platforms:**
   - Ubuntu (x86_64)
   - macOS (Intel)
   - macOS (Apple Silicon if available)
   - Windows WSL

5. **When confident, create the stable release:**

```bash
git tag v0.2.1
git push origin v0.2.1
```

### Security considerations for Samoyed

**Git hooks are security-sensitive** because they execute arbitrary code in response to Git operations. Samoyed's installation script must emphasize security:

1. **Checksum verification is non-negotiable** - Users are installing a tool that will run code during Git operations
2. **HTTPS enforcement prevents MITM attacks** - `--proto '=https' --tlsv1.2` is mandatory
3. **Installation doesn't require sudo** - User-local installation reduces attack surface
4. **Source code transparency** - Single-file implementation means users can audit easily

The installation script warns Windows users specifically because line-ending issues could cause subtle bugs in hook execution. By directing them to WSL or cargo, we ensure they get a properly configured environment.

## Comparison: Rust tools using curl/bash installation

Different Rust projects have evolved distinct approaches to curl/bash installation, informed by their specific needs and user bases.

### Rustup: The gold standard

Rustup represents the most sophisticated implementation. Its shell script is minimal—around 100 lines—because it immediately downloads a platform-specific binary installer that handles the complex logic. This hybrid approach provides:

- **Reliability**: Compiled code handles error cases better than shell
- **Features**: Can implement interactive prompts, component selection, and complex configuration
- **Maintainability**: One Rust codebase instead of maintaining complex shell scripts
- **Security**: Strong TLS enforcement (`--proto '=https' --tlsv1.2`)

The script supports passing arguments through: `curl https://sh.rustup.rs | sh -s -- --no-modify-path` transparently forwards `--no-modify-path` to the binary installer.

### cargo-binstall: Self-installation pattern

cargo-binstall demonstrates elegant self-hosting: its installation script downloads a minimal version of itself, then uses that to perform the actual installation:

```bash
# Download minimal cargo-binstall binary
curl -L https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# That script essentially does:
# 1. Download cargo-binstall for your platform
# 2. Run: ./cargo-binstall -y --force cargo-binstall
```

This self-installation approach means **the installer uses the same code paths users will use**, ensuring consistency and eating your own dog food.

### Starship: Simplicity and user choice

Starship keeps its installer straightforward but offers explicit control over installation location:

```bash
# Default installation
curl -sS https://starship.rs/install.sh | sh

# Custom binary directory
sh -c "$(curl -fsSL https://starship.rs/install.sh)" -- --bin-dir "$HOME/.local/bin"
```

The pattern of accepting arguments after the script URL (`-- --bin-dir`) provides flexibility while keeping the common case simple.

### Common patterns across Rust tools

**Standard platform naming**: Rust tools universally adopt target triples in asset names:
- `tool-x86_64-unknown-linux-musl.tar.gz`
- `tool-aarch64-apple-darwin.tar.gz`

This consistency enables tools like cargo-binstall to automatically discover and install binaries without per-project configuration.

**Musl builds for Linux**: Most tools publish musl-linked binaries (e.g., `x86_64-unknown-linux-musl`) which are fully static and work across all Linux distributions. This solves the glibc version incompatibility problem that plagues dynamically-linked binaries.

**Cargo as fallback**: Nearly all Rust tool installation scripts mention `cargo install toolname` as an alternative, ensuring users with Rust toolchains have a guaranteed installation path even if binary distribution fails.

**GitHub Releases as distribution**: Using GitHub Releases provides free hosting, automatic CDN distribution, and a standardized API for querying latest versions. The pattern `https://github.com/owner/repo/releases/latest/download/asset-name` is universally understood.

## Practical recommendations and best practices

Based on analysis of production installers and real-world usage patterns, several recommendations emerge for implementing curl/bash installation.

### Design for failure gracefully

**Users will run your installer on systems you never tested.** Robust error handling with clear, actionable messages saves immense support burden:

```bash
# Bad: cryptic failure
tar -xzf archive.tar.gz

# Good: actionable error message
if ! tar -xzf archive.tar.gz; then
  error "Failed to extract archive"
  error "This might be caused by:"
  error "  - Corrupted download (try again)"
  error "  - Disk full (check: df -h)"
  error "  - Permission issues (check directory permissions)"
  die "Extraction failed"
fi
```

Every curl, tar, mkdir, and cp command should have explicit error handling with specific guidance on resolution.

### Optimize for the common case

**Most users should succeed with zero customization.** The default behavior should be:
- Install to user-local directory (no sudo)
- Automatically configure PATH
- Download latest stable version
- Work on the most common platforms (Linux x86_64, macOS)

Advanced options (custom directory, specific versions, etc.) can require explicit flags.

### Provide inspection capability

**Security-conscious users need to review scripts before execution.** Make this easy:

```bash
# Quick install (most users)
curl -fsSL https://url/install.sh | bash

# Security review path (should be equally documented)
curl -fsSL https://url/install.sh -o install.sh
less install.sh
bash install.sh
```

Keep installation scripts readable with clear structure, comments, and avoid obfuscation or clever tricks.

### Handle platform limitations explicitly

**Don't let Windows users encounter mysterious failures.** Detect and provide guidance:

```bash
case "$(uname -s)" in
  CYGWIN*|MINGW*|MSYS*)
    warn "Windows detected via $(uname -s)"
    echo ""
    echo "For best results, use one of:"
    echo "  1. WSL (Windows Subsystem for Linux) - recommended"
    echo "  2. cargo install $BINARY - requires Rust toolchain"
    echo ""
    die "Native Windows installation not available"
    ;;
esac
```

This transforms confusion into actionable next steps.

### Implement comprehensive testing

**Test installation on actual systems, not just your development machine:**

```bash
# Docker-based testing for multiple distros
docker run -it --rm ubuntu:22.04 bash -c "apt-get update && apt-get install -y curl && curl https://url/install.sh | bash"
docker run -it --rm debian:bullseye bash -c "apt-get update && apt-get install -y curl && curl https://url/install.sh | bash"
docker run -it --rm alpine:latest sh -c "apk add --no-cache curl bash && curl https://url/install.sh | bash"
```

Test on older systems too—not everyone runs the latest Ubuntu LTS.

### Provide uninstallation

**Every installation method should have a documented uninstallation path.** The uninstall script should:
- Find all possible installation locations
- Optionally remove configuration/data
- Show what was removed and what remains
- Provide manual removal instructions if automated removal fails

This builds trust and reduces anxiety about trying your tool.

## Conclusion and future directions

The curl/bash installation pattern has evolved from a convenience into an expectation for developer tools. While it carries inherent security risks—executing arbitrary code from the internet—careful implementation with checksums, HTTPS enforcement, transparent code, and clear documentation can mitigate these concerns.

**For Rust projects specifically, the ecosystem has standardized around patterns that make implementation straightforward**: GitHub Actions for multi-platform builds, target triples in release naming, musl builds for Linux, and GitHub Releases for distribution. Tools like cargo-binstall and instl.sh lower the barrier further by providing automatic installation even for projects without custom scripts.

The Samoyed case study demonstrates that implementing curl/bash installation is very achievable but requires **building the release infrastructure first**. The shell script itself is straightforward; the complexity lies in cross-compilation, checksum generation, and automated release workflows. Once that foundation exists, the installation script becomes almost boilerplate.

As the ecosystem continues maturing, we're seeing increasing sophistication: signature verification through cargo-binstall's signing support, self-update capabilities built into binaries themselves, and better integration with platform-specific package managers. The future likely holds standardized tooling that makes curl/bash installation generation completely automatic for projects following conventions.

For now, developers implementing this pattern should prioritize security (checksums, HTTPS, no sudo), user experience (clear errors, automatic PATH config, uninstallation), and maintainability (comprehensive testing, clear code, good documentation). Following the patterns established by rustup, cargo-binstall, and other mature tools provides a proven foundation for success.