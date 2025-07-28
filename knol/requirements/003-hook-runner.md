# Requirement 003: Hook Execution Runtime

## Basic Information
- **ID**: 3
- **Title**: Hook Execution Runtime
- **Type**: Functional
- **Priority**: High
- **Status**: Approved
- **Phase**: Construction

## Description
Implement the hook runner that executes actual hook scripts with proper environment setup, error handling, and debugging support.

## Source
Adapted from `husky/husky:1-23` for language-agnostic Rust implementation

## Rationale
The hook runner is the core execution engine that actually runs user-defined hook scripts when Git triggers them. Unlike Husky's Node.js-specific approach, this implementation is language-agnostic.

## Acceptance Criteria
- [ ] Execute hook scripts from project root directory
- [ ] Load initialization script from `~/.config/samoid/init.sh`
- [ ] Handle `SAMOID=0` environment variable to skip execution
- [ ] Support `SAMOID=2` for debug mode with script tracing
- [ ] Exit with hook script's exit code
- [ ] Display informative error messages for failed hooks
- [ ] Show "command not found" message when PATH is incorrect
- [ ] Skip execution if hook script file doesn't exist

## Environment Variable Controls
- **SAMOID=0**: Skip all hook execution (useful for CI/deployment, rebasing)
- **SAMOID=2**: Enable debug mode with detailed script tracing
- **SAMOID=1** (default): Normal execution mode

## PATH Management Philosophy
Projects handle their own PATH requirements through:
- Inline environment variables in hook commands
- Global init scripts (`~/.config/samoid/init.sh`)
- Project-specific shell scripts
- Standard shell environment setup

No automatic PATH modification - keeps samoid language-agnostic.

## Dependencies
- Shell execution capabilities
- Environment variable manipulation
- File system access
- Cross-platform path handling

## Effort
6 story points (reduced - no Node.js PATH complexity)

## Planned For Iteration
Sprint 1

## Labels
- `runtime`
- `execution`
- `environment`
- `🔨 phase: construction`

## Traceability

### Use Cases
- Git triggers hook during commit/push/merge operations
- Developer tests hooks manually
- CI/CD environments run hooks with custom configuration
- Developer skips hooks during rebase/merge operations
- Multi-language projects need language-agnostic hook execution

### Test Cases
- Test hook script execution with various exit codes
- Test SAMOID=0 skip functionality
- Test SAMOID=2 debug mode
- Test init.sh script loading
- Test error message display for failed hooks
- Test missing hook file handling
- Test cross-platform execution (Windows/macOS/Linux)
- Test environment variable handling

### Design Elements
- Process execution module
- Environment management utilities
- Configuration loading system
- Error reporting framework