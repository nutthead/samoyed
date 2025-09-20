# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Samoyed is a single-binary, minimal, cross-platform Git hooks manager written in Rust. The project is currently in an architectural rewrite phase (version 0.2.0) with most of the implementation yet to be completed.

## Development Commands

### Building
```bash
# Debug build
cargo build --verbose

# Release build (optimized with LTO and stripped symbols)
cargo build --release --verbose
```

### Testing
```bash
# Run all tests
cargo test --verbose

# Platform-specific tests
cargo test --test linux_tests --verbose             # Linux only
cargo test --test macos_tests --verbose             # macOS only
cargo test --test windows_tests --verbose           # Windows only
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Check formatting (CI requirement)
cargo fmt --all -- --check

# Run Clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --no-deps --verbose

# Run security audit
cargo audit
```

### Code Coverage
```bash
# Install tarpaulin if needed
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --verbose --bins --all-features --timeout 120

# Coverage files are generated in target/tarpaulin/coverage/
```

## Architecture Notes

The project is undergoing a major architectural rewrite. Currently:
- Main entry point is in `src/main.rs` (currently empty - awaiting implementation)
- Git hook wrapper script is in `assets/samoyed` (shell script for hook execution)
- Release builds are optimized with fat LTO, single codegen unit, and symbol stripping

## Key Configuration Files

- `Cargo.toml` - Rust project manifest with optimized release profile
- `.tarpaulin.toml` - Code coverage configuration (HTML, XML, JSON, LCOV outputs)
- `clippy.toml` - Linter configuration (cognitive complexity threshold: 21)
- `.github/workflows/test.yml` - Main CI pipeline with matrix testing across Ubuntu, macOS, and Windows

## Important Notes

- The project uses Rust 2024 edition with minimum version 1.90.0
- Git hooks are managed through a wrapper script that provides consistent execution environment
- The `SAMOYED` environment variable controls debug mode (`SAMOYED=2`) and bypass mode (`SAMOYED=0`)
- User configuration can be loaded from `${XDG_CONFIG_HOME:-$HOME/.config}/samoyed/init.sh`