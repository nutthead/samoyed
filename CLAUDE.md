# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Samoyed is a single-binary, minimal, cross-platform Git hooks manager written in Rust. The project implements a complete Git hooks management system that allows users to manage client-side Git hooks with a simple, consistent interface.

## Architecture

### Core Implementation

- **Single-file architecture**: All Rust code resides in `src/main.rs` (currently ~1200 lines) following the principle of minimalism and avoiding feature creep
- **Embedded wrapper script**: The shell script at `assets/samoyed` is embedded into the binary using `include_bytes!` macro
- **Hook wrapper pattern**: Each Git hook in `.samoyed/_/` is generated as an executable stub that points contributors to the user-editable scripts in `.samoyed/`; the embedded wrapper script at `.samoyed/_/samoyed` is copied alongside for hooks (like the sample pre-commit) that source it.

### Key Components

1. **CLI Interface** (using clap):
   - `samoyed init [dirname]` - Initialize hooks in a repository
   - Default dirname: `.samoyed`

2. **Hook Management**:
   - Supports 14 standard Git hooks (pre-commit, commit-msg, pre-push, etc.)
   - Creates structure: `[dirname]/_/` for wrapper scripts, `[dirname]/` for user hooks
   - Configures Git's `core.hooksPath` to point to the wrapper directory

3. **Wrapper Script** (`assets/samoyed`):
   - POSIX-compliant shell script
   - Provides debug mode (`SAMOYED=2`) and bypass mode (`SAMOYED=0`)
   - Loads user configuration from `${XDG_CONFIG_HOME:-$HOME/.config}/samoyed/init.sh`
   - Handles hook execution with proper exit code propagation

### Design Constraints

- All Rust code MUST fit in single file
- Cognitive complexity threshold: 21 (enforced by clippy)
- No runtime dependencies (only clap for CLI)
- Must be cross-platform (Unix/Windows)
- **Cross-platform linting**: Code must pass Clippy on both Unix and Windows; use conditional `#[allow(clippy::...)]` for platform-specific type conflicts
- Follow DRY principle and maintain testability
- Rust 2024 edition with four-space indent, trailing commas
- Functions and variables use `snake_case`, types use `UpperCamelCase`
- CLI subcommands remain lowercase per clap conventions

## Development Commands

### Building
```bash
# Debug build
cargo build --verbose

# Release build (optimized with fat LTO, single codegen unit)
cargo build --release --verbose
```

### Testing
```bash
# Run all tests (unit tests are in main.rs)
# IMPORTANT: Tests must run serially to prevent intermittent failures
cargo test -- --test-threads=1

# Run specific test
cargo test test_name -- --test-threads=1

# With output display
cargo test -- --test-threads=1 --nocapture

# WARNING: Never run 'samoyed init' in this repository!
# Create a throwaway git repo for testing:
cd tmp && git init testbed && cd testbed
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Check formatting (required for CI)
cargo fmt --all -- --check

# Run Clippy linter (cognitive complexity limit: 21)
# NOTE: Cross-platform projects may have platform-specific lint differences
cargo clippy --all-targets --all-features -- -D warnings

# For cross-platform development, test Clippy on multiple platforms:
# - Linux/macOS: cargo clippy --all-targets --all-features -- -D warnings
# - Windows: Same command, but may require different code due to type differences
# - If platform-specific warnings conflict, use conditional #[allow(clippy::...)]

# Alternative: Allow specific cross-platform lint conflicts
# cargo clippy --all-targets --all-features -- -D warnings -A clippy::needless-borrow

# Generate documentation
cargo doc --no-deps --verbose

# Security audit
cargo audit
```

### Code Coverage
```bash
# Install tarpaulin if not present
cargo install cargo-tarpaulin

# Generate coverage (args and outputs are configured in .tarpaulin.toml)
cargo tarpaulin -- --test-threads=1

# Output locations:
# - HTML: target/tarpaulin/tarpaulin-report.html (flag : `-Html`)
# - XML: target/tarpaulin/cobertura.xml (flag: `-Xml`)
# - JSON: target/tarpaulin/tarpaulin-report.json (flag: `-Json`)
# - LCOV: target/tarpaulin/lcov.info (flag: `-Lcov`)
```

## Cross-Platform Development

### Conditional Compilation
Samoyed uses `#[cfg(windows)]` and `#[cfg(unix)]` attributes for platform-specific code:
- **File permissions**: Unix uses mode bits (0o644, 0o755), Windows uses different permission model
- **Path handling**: Windows may require backslash normalization for extended-length paths
- **Type differences**: Platform-specific APIs may return different types (String vs &str)

### Clippy Cross-Platform Considerations
Due to conditional compilation, Clippy may produce different warnings on different platforms:
- **Type conflicts**: `&str` vs `String` differences between platforms
- **Needless borrow**: May be needed on one platform but not another
- **Solution**: Use unified types (convert both to `String`) or conditional `#[allow(clippy::...)]`

### Testing Strategy
- All unit tests must pass on both Unix and Windows
- Integration tests use graceful degradation for platform-specific limitations
- CI runs on Linux, macOS, and Windows to catch platform-specific issues

## Project Structure

```text
.
├── .assets/                        # README.md assets (images, etc.)
├── .docs/                          # Documents and prompts for AI agents
├── assets/
│   └── samoyed                     # POSIX shell wrapper script (embedded in binary)
├── src/
│   └── main.rs                     # Complete implementation (init, hook management)
├── tests/
│   └── integration/                # Shell-based integration tests
│       ├── functions.sh            # Shared test functions
│       ├── 01_default.sh           # Default initialization test
│       ├── 02_custom_dir.sh        # Custom directory test
│       └── [other test files]      # Various edge case tests
├── .markdownlint.json              # Markdown linting configuration
├── .release-plz.toml               # Release automation config
├── .tarpaulin.toml                 # Coverage config (HTML, XML, JSON, LCOV)
├── AGENTS.md                       # AI agent configuration
├── Cargo.toml                      # Optimized release profile (fat LTO, stripped)
├── clippy.toml                     # Cognitive complexity: 21
├── CLAUDE.md                       # This file - Claude Code guidance
├── flake.nix                       # Nix development environment
└── README.md                       # Project documentation
```

## Implementation Status

Currently implemented:
- Full `init` command with validation
- Git repository detection
- Hook directory structure creation
- Wrapper script installation
- Git config management with cross-platform path normalization
- Sample pre-commit hook generation
- Cross-platform path handling (Windows extended-length path support)
- Platform-specific file permissions (Unix mode bits, Windows defaults)
- Comprehensive error handling and graceful degradation
- Cross-platform testing with CI coverage (Linux, macOS, Windows)

## Environment Variables

- `SAMOYED=0` - Bypass all hooks (checked by wrapper and init)
- `SAMOYED=2` - Enable shell debug mode in wrapper script
- `XDG_CONFIG_HOME` - Config directory (defaults to `~/.config`)

## Key Functions in main.rs

- `main()` - CLI entry point using clap
- `init_samoyed()` - Core initialization logic
- `check_bypass_mode()` - Check if SAMOYED=0 bypass mode is enabled
- `get_git_root()` - Find repository root
- `validate_samoyed_dir()` - Ensure samoyed directory is within repository
- `canonicalize_allowing_nonexistent()` - Cross-platform path canonicalization
- `create_directory_structure()` - Set up hook directories
- `copy_wrapper_script()` - Install wrapper script
- `create_hook_scripts()` - Generate hook stub scripts
- `create_sample_pre_commit()` - Example hook creation
- `set_git_hooks_path()` - Configure Git to use hooks with cross-platform path normalization
- `create_gitignore()` - Ignore wrapper directory

## Testing Approach

Tests are integrated into `main.rs` using `#[cfg(test)]` modules. Key test areas:
- Path validation and cross-platform path handling
- Git repository detection
- Hook script generation with proper permissions
- Cross-platform compatibility (Unix/Windows)
- Git hooks execution with graceful degradation on unsupported environments

Integration tests in `tests/integration/` cover:
- End-to-end hook management workflows
- Cross-platform Git configuration handling
- Windows extended-length path support
- Platform-specific error handling and recovery

Run tests with `cargo test --verbose` to see all test output.

## Release Optimization

The release profile in `Cargo.toml` includes:
- `lto = "fat"` - Full link-time optimization
- `codegen-units = 1` - Single compilation unit
- `strip = true` - Remove debug symbols
- `opt-level = 3` - Maximum optimization

This produces a minimal, fast binary suitable for distribution.

## Development Environment

The project includes a Nix flake for consistent development environments. Run `nix develop` to enter a shell with all required tools including Rust toolchain, cargo-tarpaulin, and other dependencies.

## Commit Guidelines

Commits follow Conventional Commits format:
- `feat:` for new features
- `fix:` for bug fixes
- `chore:` for maintenance tasks
- Add `!` for breaking changes (e.g., `feat!:`)
- Use concise, imperative descriptions
- Group related changes, avoid catch-all commits

## Hard rules

**Rule 1:** Whenever a function is modified, ensure its rustdoc comments are updated to reflect the changes, including parameters, return values, and behavior.
**Rule 2:** Whenever a new function is added, it must include rustdoc comments explaining its purpose and usage, including parameters and return values.
