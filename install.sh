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
die() {
    error "$*"
    exit 1
}

# Banner
show_banner() {
    echo -e "${BLUE}"
    cat <<'EOF'
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
    Linux*) os='unknown-linux-musl' ;;
    Darwin*) os='apple-darwin' ;;
    CYGWIN* | MINGW* | MSYS*)
        warn "Detected Windows environment"
        echo ""
        echo "For Windows, please use one of:"
        echo "  1. WSL (Windows Subsystem for Linux) - recommended"
        echo "  2. cargo install samoyed"
        echo ""
        die "Native Windows installation not supported"
        ;;
    *) die "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
    x86_64 | amd64)
        arch='x86_64'
        ;;
    aarch64 | arm64)
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
    version=$(curl -fsSL "$api_url" 2>/dev/null |
        grep '"tag_name":' |
        sed -E 's/.*"v?([^"]+)".*/\1/') || {
        # Fallback to checking /releases/latest redirect
        warn "Could not query GitHub API, trying direct redirect"
        version=$(curl -fsSL -o /dev/null -w '%{redirect_url}' \
            "https://github.com/${REPO}/releases/latest" |
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
        return 0 # Already in PATH
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
    } >>"$shell_config"

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
