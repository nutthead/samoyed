# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

This project's root directory is <file:~/Projects/github.com/typicode/husky-to-samoid/>.

## Project Overview

This repository contains two implementations of Husky, a modern native Git hooks manager:

### Current Implementation (`husky/`)
The existing Node.js/JavaScript implementation consisting of:
- **Core module** (`index.js`): Default export function that installs Git hooks by configuring `core.hooksPath` and creating hook files
- **CLI binary** (`bin.js`): Command-line interface with `init` command and deprecated command warnings
- **Hook runner** (`husky` binary): Shell script that executes actual hook commands with proper environment setup
- **TypeScript definitions** (`index.d.ts`): Simple function signature for the default export

### Future Implementation (`samoid/`)
A Rust reimplementation of Husky that is currently in early development:
- **Cargo project** with edition 2024
- Currently contains only a basic "Hello, world!" main function
- Intended to replace the Node.js implementation with better performance and reduced dependencies

## Architecture

### Current Husky (`husky/`)
The system works in two phases:
1. **Installation**: Sets Git's `core.hooksPath` to `.husky/_` and creates hook files that delegate to the `husky` binary
2. **Execution**: The `husky` binary loads user configuration and runs the actual hook scripts

Key files:
- `husky/index.js:8-25` - Main installation logic with Git configuration and file creation
- `husky/bin.js:10-20` - CLI `init` command that modifies `package.json` and creates sample hooks
- `husky/husky:1-23` - Hook execution runtime with environment setup and error handling

### Samoid (`samoid/`)
A fully functional Rust reimplementation of Husky with comprehensive features:
- **CLI binary** (`samoid/src/main.rs`): Command-line interface with `init` command using clap
- **Hook runner binary** (`samoid-hook`): Separate binary for executing git hooks at runtime
- **Core modules**:
  - `samoid/src/lib.rs`: Public API exposing install_hooks function and modules
  - `samoid/src/installer.rs`: Main installation logic with error handling
  - `samoid/src/environment.rs`: Dependency injection traits (Environment, CommandRunner, FileSystem) and implementations
  - `samoid/src/git.rs`: Git repository validation and configuration management
  - `samoid/src/hooks.rs`: Hook file creation and management for all 14 standard git hooks
  - `samoid/src/config.rs`: TOML-based configuration with SamoidConfig and SamoidSettings
  - `samoid/src/project.rs`: Project type detection (Node.js, Rust, etc.)
- **Testing infrastructure**: Mock implementations for complete test isolation
- **Benchmarks**: Performance testing suite with Criterion
- **Dependencies**: clap (CLI), toml/serde (config), anyhow (error handling)

## Development Commands

### Current Husky (`husky/`)
```bash
cd husky/
./test.sh                    # Run all integration tests
sh test/1_default.sh         # Run specific test case
npm pack                     # Create distribution package (used by tests)
shellcheck husky             # Lint shell script
shellcheck test/*.sh         # Lint test scripts
```

### Samoid (`samoid/`)
```bash
cd samoid/
cargo build                  # Build the Rust implementation
cargo run                    # Run the current placeholder
cargo test                   # Run tests (when implemented)
cargo check                  # Fast syntax/type checking
```

## Testing Strategy

### Current Husky
Integration tests use shell scripts that:
1. Create temporary Git repositories in `/tmp/husky-test-*`
2. Install husky from packed tarball (`/tmp/husky.tgz`)
3. Test various scenarios (sub-directories, missing git, environment variables)
4. Verify Git configuration and hook execution behavior

Test utilities in `test/functions.sh` provide `setup()`, `install()`, `expect()`, and `expect_hooksPath_to_be()` functions.

### Samoid
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

## Directory Context Management
**ALWAYS:** verify working directory before Samoid/Husky-specific commands**
**REMEMBER:** the project's root directory is <file:~/Projects/github.com/typicode/husky-to-samoid/>
**REMEMBER:** you can always use `pwd` to verify current location
**REMEMBER:** some commands accept absolute paths via flags, and when possible, that can help you avoid changing directory with `cd`

- For Samoid, check for `Cargo.toml` before `cargo` commands
- For Husky, check for `package.json` before reading files.
