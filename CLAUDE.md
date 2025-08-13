# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the samoyed repository - a modern native Git hooks manager implemented in Rust.

<context>
- Main branch: master
- GitHub repository: nutthead/samoyed
- Current version: 0.1.10
</context>

## Project Overview

### Architecture

<binaries>
- **samoyed**: Main CLI binary from `src/main.rs` with unified CLI (`init` and `hook` subcommands)
- **samoyed-hook**: Deprecated backward compatibility shim from `src/hook_runner.rs`
</binaries>

### Hook Execution Flow
```
Git Hook Trigger → .samoyed/_/hook-name → samoyed hook subcommand
                                        ↓
                   Two-tier lookup system:
                   1. PRIMARY: samoyed.toml [hooks] section
                   2. FALLBACK: .samoyed/scripts/hook-name
```

<modules>
**Public API (lib.rs exports)**:
- `environment`: Dependency injection traits (Environment, CommandRunner, FileSystem) with Mock implementations
- `git`: Git repository validation and configuration management
- `hooks`: Hook file creation and management with cross-platform support
- `installer`: Main installation logic orchestration
- `logging`: Secure logging utilities with automatic sanitization of sensitive information

**Internal modules (used in main.rs)**:
- `config`: TOML-based configuration (SamoyedConfig, SamoyedSettings)
- `project`: Project type detection (Rust, Go, Node.js, Python) with sensible defaults
- `init`: Implementation of the `samoyed init` command
- `exit_codes`: Standardized sysexits.h-compliant exit codes
- `hook_runner`: Deprecated shim binary for backward compatibility
</modules>

### Dependencies
**Core Runtime Dependencies**:
- **clap** (v4.5): Command-line argument parsing with derive macros
- **toml** (v0.9): Configuration file parsing and serialization
- **serde** (v1.0): Serialization/deserialization with derive support
- **anyhow** (v1.0): Flexible error handling and context propagation

**Development Dependencies**:
- **tempfile** (v3.13): Temporary file creation for testing
- **criterion** (v0.7): Performance benchmarking framework

### Testing
- **Unit tests**: Located in `src/unit_tests/` directory, included via `#[path]` attributes
- **Integration tests**: In `tests/` directory with platform-specific test suites
- **Cross-platform tests**: Linux, macOS, Windows (PowerShell + Git Bash)
- **Benchmarks**: Criterion benchmarks in `tests/benchmark_tests/benchmark.rs`
- **Coverage**: 69% minimum threshold using `cargo tarpaulin` (target: 90%)
- **Test pattern**: Dependency injection with Mock* implementations for 100% testability
- **CI Matrix**: 8+ platform/toolchain combinations in GitHub Actions

### Security
- **Path validation**: Prevents directory traversal attacks (rejects "..")
- **Logging sanitization**: Automatic redaction of sensitive information (passwords, tokens, home directories)
- **Cross-platform security**: Handles Windows PowerShell, Git Bash, and Unix environments safely

### Performance
- **Release profile**: Optimized for size with LTO and single codegen unit
- **Binary size**: Stripped debug symbols for minimal distribution size
- **Cross-platform**: Native builds for Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64, i686)

### Rust Edition
This project uses Rust 2024 edition with minimum supported Rust version 1.88.0.

## Git Workflow

### Version Tags
```bash
# Use semantic versioning with 'v' prefix
# Example:
git tag -a v0.1.7 -m "Release version 0.1.7"
```

## Markdown Conventions
**YOU MUST** write tables in human-readabke format:

**✅ GOOD:**
```markdown
| Column 1 | Column 2 |
|----------|----------|
| Foo      | Bar      |
```

**❌ BAD:**
```markdown
| Column 1 | Column 2 |
|-|-|
| Foo | Bar |
```

## PR body conventions
**YOU MUST NOT** use `[ ]` and `[x]` to show scope of a PR.
**YOU MUST** use `☐` and `☑` to show scope of a PR.

### Examples

❌ BAD - DON'T DO THIS!
```markdown
## Type of Change

- [x] Chore/cleanup (non-breaking change that doesn't add features or fix bugs)
- [x] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
```

✅ GOOD - DO THIS!
```markdown
## Type of Change

- 🗵 Chore/cleanup (non-breaking change that doesn't add features or fix bugs)
- 🗵 Bug fix (non-breaking change which fixes an issue)
- ☐ New feature (non-breaking change which adds functionality)
- ☐ Breaking change (fix or feature that would cause existing functionality to not work as expected)
- ☐ Documentation update
```

## Technical Learnings

### Testing with process::exit()
- Tests that call `process::exit()` use `std::panic::catch_unwind()` pattern
- This catches the panic from `process::exit()` calls during testing
- Pattern: `assert!(result.is_err())` verifies the function exited
- Limitation: Cannot easily verify specific exit codes in tests

### Test Code Refactoring Patterns
- Extract repeated struct creation patterns into helper functions
- Example: `make_output(status_code, stdout, stderr)` reduces `Output` duplication
- Benefits: Single point of change, improved readability, consistent patterns
- Apply when seeing 3+ similar patterns in test code

### Cross-Platform Testing Strategy
- **Dependency injection**: All external dependencies abstracted through traits for testability
- **Mock implementations**: MockEnvironment, MockCommandRunner, MockFileSystem for isolated testing
- **Platform-specific tests**: Separate test suites for Linux, macOS, and Windows behaviors
- **Shell compatibility**: Tests verify PowerShell, Git Bash, and Unix shell execution
- **Coverage tracking**: Comprehensive coverage analysis with multiple output formats (HTML, XML, JSON, LCOV)

## Project Type Defaults

Samoyed automatically detects project types and suggests appropriate hook commands:

| Project Type | Detection                        | Default Hook Command                               |
|--------------|----------------------------------|----------------------------------------------------|
| **Rust**     | Cargo.toml                       | `cargo fmt --check && cargo clippy -- -D warnings` |
| **Go**       | go.mod                           | `go fmt ./... && go vet ./...`                     |
| **Node.js**  | package.json                     | `npm run lint && npm test`                         |
| **Python**   | requirements.txt, pyproject.toml | `black --check . && flake8`                        |

## Configuration Structure

### TOML Schema (samoyed.toml)
```toml
[hooks]
pre-commit = "command to execute"
pre-push = "another command"

[settings]  # Optional
hook_directory = ".samoyed"  # Default
debug = false
fail_fast = true
skip_hooks = false
```

### Environment Variables
- **SAMOYED=0**: Skip all hooks
- **SAMOYED=1**: Normal execution (default)
- **SAMOYED=2**: Debug mode with verbose output

## CI/CD Workflow

### GitHub Actions
- **Test Suite**: Cross-platform testing (Ubuntu, macOS, Windows) with multiple Rust versions
- **Code Coverage**: cargo-tarpaulin analysis with 69% minimum threshold
- **Security Audit**: cargo-audit for dependency vulnerability scanning
- **Release Automation**: Cross-platform binary builds with automated changelog generation
- **Quality Gates**: rustfmt, clippy, documentation generation, benchmark execution

### Platform Coverage
- **Linux**: ubuntu-latest, ubuntu-24.04 (stable, beta, nightly)
- **macOS**: macos-latest (stable, beta)
- **Windows**: windows-latest (stable with PowerShell and Git Bash)

## Quick Reference

<frequently-used-commands>
### Build Commands
```bash
cargo build --verbose
cargo build --release --verbose
```

### Testing Suite
```bash
cargo test --verbose                              # All tests
cargo test --lib --verbose                        # Unit tests only
cargo test --test installation_tests              # Specific integration test
cargo test --test linux_tests                     # Platform-specific tests
cargo test --test cross_platform_shell_execution  # Cross-platform compatibility
```

### Code Quality
```bash
cargo fmt --all -- --check                                # Format checking
cargo clippy --all-targets --all-features -- -D warnings  # Linting (deny all warnings)
cargo doc --no-deps --verbose                             # Documentation generation
```

### Coverage Analysis
```bash
cargo tarpaulin                                                # Generate coverage report
cargo tarpaulin --verbose --bins --all-features --timeout 120  # Full coverage analysis
```

### Security & Performance
```bash
cargo audit                            # Dependency vulnerability scan
cargo bench --verbose                  # Run performance benchmarks
```

### Complete Development Check
```bash
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --verbose && \
cargo doc --no-deps --verbose
```

### Release Preparation
```bash
# Version validation
grep -E "^version" Cargo.toml | head -1 | cut -d'"' -f2

# Tag creation (semantic versioning)
git tag -a v0.1.10 -m "Release version 0.1.10"
```
</frequently-used-commands>
