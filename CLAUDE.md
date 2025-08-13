# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with the samoyed repository - a modern native Git hooks manager implemented in Rust.

<context>
- Main branch: master
- GitHub repository: nutthead/samoyed
</context>

## Project Overview

### Architecture

<binaries>
- **samoyed**: Main CLI binary from `src/main.rs` with `init` command using clap
- **samoyed-hook**: Deprecated shim from when samoyed had 2 binaries (see: https://github.com/nutthead/samoyed/issues/63)
</binaries>

<modules>
**Public API (lib.rs exports)**:
- `environment`: Dependency injection traits and implementations
- `git`: Git repository validation and configuration
- `hooks`: Hook file creation and management
- `installer`: Main installation logic
- `logging`: Logging utilities and output formatting

**Internal modules (used in main.rs)**:
- `config`: TOML-based configuration (SamoyedConfig, SamoyedSettings)
- `project`: Project type detection (Node.js, Rust, Python, etc.)
- `init`: Implementation of the `samoyed init` command
- `exit_codes`: Standardized exit codes for error conditions
</modules>

### Dependencies
- **clap** (v4.5): Command-line argument parsing with derive macros
- **toml** (v0.9): Configuration file parsing
- **serde** (v1.0): Serialization/deserialization with derive support
- **anyhow** (v1.0): Flexible error handling and propagation

### Testing
- **Unit tests**: Located in `src/unit_tests/` directory, included via `#[path]` attributes
- **Integration tests**: In `tests/` directory
- **Benchmarks**: Criterion benchmarks in `tests/benchmark_tests/benchmark.rs`
- **Coverage**: Use `cargo tarpaulin` (see `.tarpaulin.toml`)
- **Test pattern**: Dependency injection with Mock* implementations for isolation

### Rust Edition
This project uses Rust 2024 edition.

### Development Commands

```bash
# Build
cargo build                      # Debug build
cargo build --release            # Release build
cargo check --all-targets        # Check all targets

# Testing
cargo test                      # All tests
cargo test --lib                # Unit tests only
cargo test --test <name>        # Specific integration test
cargo bench                     # Run benchmarks

# Code Quality
cargo fmt                                                       # Format code
cargo fmt --check                                               # Check formatting
cargo clippy --all-targets --all-features -- -D warnings        # Lint with warnings as errors

# Documentation & Coverage
cargo doc                                                                      # Generate docs

# Tarpaulin is [configured](file:.tarpaulin.toml) to store reports in <target/tarpaulin/coverage/{cobertura.xml,lcov.info,tarpaulin-report.json}>.
cargo tarpaulin --verbose --bins --all-features --timeout 120
cargo tarpaulin --verbose --bins --all-features -o Stdout --timeout 120
```

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

## Vocabulary

- **Read issue #n**: Use the `gh` CLI to view issue #n
- **Ultraread issue #n**: Use the `gh` CLI to view issue #n
- **Find review comment**: Extract ID from GitHub URL and use `gh api` to get comment details

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

### GitHub API Patterns
- PR comments vs review comments are different endpoints
- Review comment IDs in URLs can be used directly with API
- Line-specific comments require `/pulls/{pr}/comments` endpoint
- Use `gh api` for precise data access when `gh` subcommands insufficient

### Review Comment Workflow
- **Comment ID vs Thread ID**: Comments have individual IDs, threads have separate GraphQL IDs
- **Replying**: Use `/pulls/{pr}/comments` POST with `in_reply_to` parameter
- **Resolving**: Requires GraphQL mutation with thread ID (not comment ID)
- **Thread ID Discovery**: Query GraphQL reviewThreads and match by comment `databaseId`
- **Required Reply Parameters**: body, in_reply_to, path, line number

## Quick Reference

<frequently-used-commands>
### Run specific test
```bash
cargo test --test installation_tests
```

### Check coverage
```bash
cargo tarpaulin
```
</frequently-used-commands>
