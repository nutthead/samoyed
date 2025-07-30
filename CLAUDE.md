# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

This project's root directory is <file:~/Projects/github.com/typicode/husky-to-samoid/>.

## Project Overview

This repository contains **Samoid**, a modern native Git hooks manager implemented in Rust. Samoid is a complete reimplementation and replacement of the original Husky Node.js implementation, providing better performance, reduced dependencies, and enhanced reliability.

### Samoid Architecture
Samoid is a fully functional Rust implementation with comprehensive features:

#### Binaries
- **CLI binary** (`samoid`): Main command-line interface built from `src/main.rs` with `init` command using clap
- **Hook runner binary** (`samoid-hook`): Separate binary built from `src/hook_runner.rs` for executing git hooks at runtime

#### Core Modules
- **`src/lib.rs`**: Public API exposing install_hooks function and module exports
- **`src/installer.rs`**: Main installation logic with comprehensive error handling
- **`src/environment.rs`**: Dependency injection traits (Environment, CommandRunner, FileSystem) and implementations
- **`src/git.rs`**: Git repository validation and configuration management
- **`src/hooks.rs`**: Hook file creation and management for all 14 standard git hooks
- **`src/config.rs`**: TOML-based configuration with SamoidConfig and SamoidSettings
- **`src/project.rs`**: Project type detection (Node.js, Rust, Python, etc.)
- **`src/init.rs`**: Implementation of the `samoid init` command functionality
- **`src/logging.rs`**: Logging utilities and output formatting
- **`src/exit_codes.rs`**: Standardized exit codes for error conditions

#### Testing & Quality
- **Unit tests**: Inline tests in each module using Rust's built-in test framework
- **Integration tests**: Comprehensive test suite in `tests/` directory
- **Performance benchmarks**: Criterion-based benchmarks in `tests/benches/`
- **Test utilities**: Mock implementations (MockEnvironment, MockCommandRunner, MockFileSystem) for complete test isolation

#### Dependencies
- **clap** (v4.5): Command-line argument parsing with derive macros
- **toml** (v0.8): Configuration file parsing
- **serde** (v1.0): Serialization/deserialization with derive support
- **anyhow** (v1.0): Flexible error handling and propagation

## Temp directory

- **Instead of `file:/tmp/`, use `file:tmp/`**. `file:tmp/` is intentionally gi ignored.
- If it doesn't exist, **YOU ARE PERMITTED** to create it.

## Development Commands

### Samoid (Root Level)
```bash
# Build commands
cargo build                  # Build debug version
cargo build --release       # Build optimized release version
cargo check                  # Fast syntax/type checking
cargo check --all-targets   # Check all targets including tests and benchmarks

# Testing commands
cargo test                   # Run all tests (unit + integration)
cargo test --lib            # Run unit tests only
cargo test --test <name>     # Run specific integration test
cargo bench                  # Run performance benchmarks

# Code quality
cargo fmt                    # Format code
cargo fmt --check            # Check code formatting
cargo clippy                 # Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings  # Strict linting

# Documentation and analysis
cargo doc                    # Generate documentation
cargo doc --open             # Generate and open documentation
cargo tarpaulin              # Generate test coverage report
```

## Testing Strategy

### Samoid Testing Architecture
Uses dependency injection pattern for complete test isolation and exceptional quality metrics:

**Architecture Pattern:**
- **Abstractions**: `Environment`, `CommandRunner`, `FileSystem` traits define interfaces
- **Production**: `SystemEnvironment`, `SystemCommandRunner`, `SystemFileSystem` for real operations
- **Testing**: `MockEnvironment`, `MockCommandRunner`, `MockFileSystem` with `Arc<Mutex<T>>` for thread safety

**Testing Reference Guide:**
Before writing new tests---or maintaining existing tests and fixing borken tests---proactively read [Rust Testing Catalog: Comprehensive Reference Guide](knol/references/002-rust-testing-reference.md) as a reference guide.

**Coverage Tools:** Use `cargo tarpaulin` with `.tarpaulin.toml`:
```toml
[default]
run-types = ["Tests"]

[report]
output-dir = "target/tarpaulin/coverage"
out = ["Html", "Json"]
```

### The `file:tmp/samoid/dummy/` dir

- Create and use file:tmp/samoid/dummy/ for ad-hoc and integration testing of Samoid.
- Freely use git operations inside file:tmp/samoid/dummy/
- Freely init new repos inside file:tmp/samoid/dummy/
- Freely delete and recreate file:tmp/samoid/dummy/ when needed for testing

## Directory Context Management
**ALWAYS:** verify working directory before Samoid-specific commands**
**REMEMBER:** the project's root directory is <file:~/Projects/github.com/typicode/husky-to-samoid/>
**REMEMBER:** you can always use `pwd` to verify current location
**REMEMBER:** some commands accept absolute paths via flags, and when possible, that can help you avoid changing directory with `cd`

- For Samoid, CONFIRM the current dir contains `Cargo.toml` before you run `cargo` commands
- The project root now contains the main `Cargo.toml`, `src/`, `tests/`, and other Rust project files

### Prefer absolute paths to relative paths

```bash
# BAD
cd src && rm tests/comprehensive_integration_tests.rs

# GOOD
rm ~/Projects/github.com/typicode/husky-to-samoid/tests/comprehensive_integration_tests.rs

###############################################################################

# BAD (outdated path)
rm samoid/tests/comprehensive_integration_tests.rs

# GOOD
rm ~/Projects/github.com/typicode/husky-to-samoid/tests/comprehensive_integration_tests.rs

###############################################################################

# BAD
cd src && cargo check --all-targets

# GOOD
cd ~/Projects/github.com/typicode/husky-to-samoid && cargo check --all-targets
```

## GitHub CLI
This project is under the `nutthead` organization, so ensure you use the `gh` CLI correctly:

```bash
# BAD (causes "Error: gh: Not Found (HTTP 404)")
$ gh api repos/nutthead/samoid/actions/runs/16605659171/jobs/46976672370/logs

# GOOD
$ gh run view 16605659171 --repo nutthead/samoid --log-failed

###############################################################################

# BAD
$ gh api repos/nutthead/samoid/pulls/comments/2242172228/replies --method POST --field body=@/tmp/concurrency-reply.md

# GOOD
$ gh pr comment 23 --repo nutthead/samoid --body-file /tmp/concurrency-reply.md

###############################################################################
```

Prefer --body-file to --body:
```bash
# BAD (could fail/err in case of special characters)
$ gh issue comment 7 --repo nutthead/samoid --body <stdin text>

# GOOD
# First write the body to a file, then use --body-file to read the body from the file
$ gh issue comment 7 --repo nutthead/samoid --body-file /tmp/issue-7-completion-comment.md


###############################################################################

# BAD
$ gh pr create --title "feat: implement comprehensive performance optimization (#8)" --body <stdin text>

# GOOD
$ gh pr create --title 'feat: implement comprehensive performance optimization (#8)' --body-file <path to file>
```
