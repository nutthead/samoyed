# Requirement 002: Command Line Interface

## Basic Information
- **ID**: 2
- **Title**: Command Line Interface
- **Type**: Functional
- **Priority**: High
- **Status**: Approved
- **Phase**: Construction

## Description
Implement a CLI that provides the `init` command and manages project configuration through a `samoid.toml` file, providing language-agnostic Git hook management.

## Source
Adapted from `husky/bin.js:10-26` and analysis of Husky's proven approach

## Rationale
Samoid should work with any programming language or project type, not just Node.js projects. TOML provides better type safety, comments, and language-neutral configuration. As a clean, modern alternative to Husky, Samoid focuses on simplicity without legacy baggage, using only essential hook definitions with standard shell syntax for environment variables and existing ecosystem tools for complex workflows.

## Acceptance Criteria
- [ ] Implement `samoid init` command functionality
- [ ] Create `samoid.toml` configuration file with project settings
- [ ] Create `.samoid` directory if it doesn't exist  
- [ ] Generate sample `pre-commit` hook with appropriate test command for detected project type
- [ ] Auto-detect project type (Go, Rust, Node.js, Python, etc.) for sensible defaults
- [ ] Exit with appropriate status codes (0 for success, 1 for errors)
- [ ] Handle missing `samoid.toml` gracefully with sensible defaults
- [ ] Preserve existing formatting in `samoid.toml` when updating

## samoid.toml Structure

```toml
# Core hook definitions (required)
[hooks]
pre-commit = "go fmt ./... && go vet ./..."
pre-push = "CGO_ENABLED=0 go test -v -race ./..."
commit-msg = "go run scripts/validate-commit-msg.go"

# Optional settings (only if non-default values needed)
[settings]
hook_directory = ".samoid"  # Only if custom directory needed
debug = false              # Only if enabling debug mode
fail_fast = true          # Only if changing default behavior
```

**Design Philosophy**: 
- **Ultra-minimal configuration** - Only `[hooks]` section required
- **Shell-native** - Hooks execute shell commands directly (including env vars: `CGO_ENABLED=0 command`)
- **Ecosystem-friendly** - Use existing tools (shell scripts, Makefiles, task runners) for complex workflows
- **No indirection** - Commands run directly in shell context
- **Clean slate approach** - Modern design without legacy baggage or deprecated patterns

## Dependencies
- File system access for reading/writing `samoid.toml`
- TOML parsing capabilities (using `toml` crate)
- Project type detection utilities (go.mod, Cargo.toml, package.json, etc.)

## Effort
5 story points (reduced due to simplified design)

## Planned For Iteration
Sprint 1

## Labels
- `cli`
- `user-interface`
- `initialization`
- `🔨 phase: construction`

## Traceability

### Use Cases
- New developer sets up hooks with `samoid init` on Go/Rust/Python/any project
- Teams adopting modern Git hook management solution
- Project configuration through `samoid.toml`
- Multi-language monorepos needing consistent hook management
- CI/CD pipelines requiring language-neutral Git hooks

### Test Cases
- Test `init` command creates proper `samoid.toml` configuration
- Verify sample hook file generation for different project types (Go, Rust, Node.js, Python)
- Test project type auto-detection (go.mod, Cargo.toml, package.json, requirements.txt)
- Test TOML formatting preservation
- Test missing `samoid.toml` handling with defaults
- Test custom hook directory configuration
- Test environment variable configuration for different languages
- Test error handling for invalid arguments

### Design Elements
- CLI argument parser (using clap)
- TOML configuration management (using toml crate)
- Project type detection utilities
- Configuration validation and defaults
- Language-specific default hook generators
- File system operations