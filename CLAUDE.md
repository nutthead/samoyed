# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Samoyed is a single-binary, minimal, cross-platform Git hooks manager written in Rust. The project implements a complete Git hooks management system that allows users to manage client-side Git hooks with a simple, consistent interface.

## Architecture

### Core Implementation

- **Single-file architecture**: All Rust code resides in `src/main.rs` (currently ~700 lines) following the principle of minimalism and avoiding feature creep
- **Embedded wrapper script**: The shell script at `assets/samoyed` is embedded into the binary using `include_bytes!` macro
- **Hook wrapper pattern**: Each Git hook is a symlink to the wrapper script which delegates to user-defined hooks

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
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --no-deps --verbose

# Security audit
cargo audit
```

### Code Coverage
```bash
# Install tarpaulin if not present
cargo install cargo-tarpaulin

# Generate coverage with multiple output formats
cargo tarpaulin --verbose --bins --all-features --timeout 120

# Output locations:
# - HTML: target/tarpaulin/coverage/index.html
# - XML: target/tarpaulin/coverage/cobertura.xml
# - JSON: target/tarpaulin/coverage/coverage.json
# - LCOV: target/tarpaulin/coverage/lcov.info
```

## Project Structure

```
.
├── src/
│   └── main.rs                     # Complete implementation (init, hook management)
├── assets/
│   └── samoyed                     # POSIX shell wrapper script (embedded in binary)
├── .docs/
│   └── 01-vision-version-2.0.0.md  # Detailed specification
├── Cargo.toml                      # Optimized release profile (fat LTO, stripped)
├── clippy.toml                     # Cognitive complexity: 21
└── .tarpaulin.toml                 # Coverage config (HTML, XML, JSON, LCOV)
```

## Implementation Status

Currently implemented:
- Full `init` command with validation
- Git repository detection
- Hook directory structure creation
- Wrapper script installation
- Git config management
- Sample pre-commit hook generation
- Cross-platform path handling
- Comprehensive error handling

## Environment Variables

- `SAMOYED=0` - Bypass all hooks (checked by wrapper and init)
- `SAMOYED=2` - Enable shell debug mode in wrapper script
- `XDG_CONFIG_HOME` - Config directory (defaults to `~/.config`)

## Key Functions in main.rs

- `main()` - CLI entry point using clap
- `init_samoyed()` - Core initialization logic
- `get_git_root()` - Find repository root
- `validate_samoyed_path()` - Ensure path is within repository
- `create_directory_structure()` - Set up hook directories
- `copy_samoyed_wrapper()` - Install wrapper script
- `create_hook_scripts()` - Generate hook symlinks/scripts
- `create_sample_precommit()` - Example hook creation
- `set_git_hooks_path()` - Configure Git to use hooks
- `create_gitignore()` - Ignore wrapper directory

## Testing Approach

Tests are integrated into `main.rs` using `#[cfg(test)]` modules. Key test areas:
- Path validation
- Git repository detection
- Hook script generation
- Cross-platform compatibility

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