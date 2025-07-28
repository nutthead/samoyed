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
Currently minimal Rust project structure:
- `samoid/src/main.rs:1-3` - Basic entry point (placeholder implementation)
- `samoid/Cargo.toml:1-6` - Cargo configuration using Rust 2024 edition

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

**Test Isolation Pattern:**
```rust
#[test]
fn test_example() {
    // Completely isolated - no shared state or system dependencies
    let env = MockEnvironment::new().with_var("HUSKY", "0");
    let runner = MockCommandRunner::new()
        .with_response("git", &["config", "core.hooksPath", ".samoid/_"], Ok(output));
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
}
```

**Quality Achievements:**
- **Coverage**: 94.33% (133/141 lines) through systematic testing approach
- **Reliability**: 100% test pass rate (was ~70% with environment contamination)
- **Performance**: 15x faster execution (~2s vs ~30s, 70 tests total)
- **Architecture**: Clean codebase with zero compiler warnings after removing 77 lines of legacy code

**Testing Strategy Levels:**
1. **Real System Integration**: Tests with `SystemFileSystem` validate production implementations
2. **Mock Error Scenarios**: Comprehensive edge case and error condition testing
3. **Main Logic Testing**: All execution paths without binary execution
4. **Parallel Execution**: Thread-safe mocks enable reliable concurrent testing

**Coverage Tools:** Use `cargo tarpaulin` with `.tarpaulin.toml`:
```toml
[default]
run-types = ["Tests"]

[report]
output-dir = "target/tarpaulin/coverage"
out = ["Html", "Json"]
```

**Implementation Lessons:**
- **Interface Simplification**: Environment trait reduced from 5 methods to 1 through usage analysis
- **Legacy Elimination**: Systematic removal of unused code improves coverage and reduces complexity
- **Iterative Improvement**: 4-step approach: baseline → DI implementation → legacy removal → comprehensive testing
- **Meaningful Coverage**: Focus on behavioral validation rather than just coverage numbers

## Directory Context Management
**ALWAYS:** verify working directory before Samoid/Husky-specific commands**
**REMEMBER:** the project's root directory is <file:~/Projects/github.com/typicode/husky-to-samoid/>
**REMEMBER:** you can always use `pwd` to verify current location
**REMEMBER:** some commands accept absolute paths via flags, and when possible, that can help you avoid changing directory with `cd`

- For Samoid, check for `Cargo.toml` before `cargo` commands
- For Husky, check for `package.json` before reading files.
