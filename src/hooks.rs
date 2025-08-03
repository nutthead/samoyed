//! Git hook file creation and management
//!
//! This module handles the creation and configuration of Git hook files.
//! It creates the hook directory structure, individual hook scripts, and
//! the hook runner that executes user-defined hook commands.
//!
//! # Hook Structure
//!
//! The module creates:
//! - A hooks directory (e.g., `.samoyed/_`)
//! - A `.gitignore` file to exclude hooks from version control
//! - Individual hook scripts for all standard Git hooks
//! - A hook runner script (`h`) that executes the actual hook logic

use crate::environment::FileSystem;
use std::path::Path;

/// List of all standard Git hooks that Samoid manages
///
/// This array contains the names of all Git hooks that Samoid will create
/// during installation. Each hook is a simple shell script that delegates
/// to the main hook runner.
///
/// # Supported Hooks
///
/// - **pre-commit**: Runs before a commit is created
/// - **pre-merge-commit**: Runs before a merge commit
/// - **prepare-commit-msg**: Prepares the default commit message
/// - **commit-msg**: Validates or modifies commit messages
/// - **post-commit**: Runs after a commit is created
/// - **applypatch-msg**: Can edit the commit message file for patches
/// - **pre-applypatch**: Runs before a patch is applied
/// - **post-applypatch**: Runs after a patch is applied
/// - **pre-rebase**: Runs before a rebase operation
/// - **post-rewrite**: Runs after commits are rewritten
/// - **post-checkout**: Runs after a successful checkout
/// - **post-merge**: Runs after a successful merge
/// - **pre-push**: Runs before a push operation
/// - **pre-auto-gc**: Runs before automatic garbage collection
pub const STANDARD_HOOKS: &[&str] = &[
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-rebase",
    "post-rewrite",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-auto-gc",
];

/// Errors that can occur during hook file operations
///
/// Currently wraps I/O errors that may occur when creating directories
/// or writing hook files.
#[derive(Debug)]
pub enum HookError {
    /// An I/O error occurred during file system operations
    IoError(std::io::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for HookError {}

impl From<std::io::Error> for HookError {
    fn from(error: std::io::Error) -> Self {
        HookError::IoError(error)
    }
}

/// Normalizes line endings to Unix-style (LF) for cross-platform compatibility
///
/// This function ensures that all generated files use consistent LF line endings,
/// regardless of the platform they're created on. This is important because:
///
/// - Git Bash and Unix shells expect LF endings
/// - Git's `core.autocrlf` settings can cause issues with mixed line endings
/// - Hook scripts should be executable on all platforms
///
/// # Arguments
///
/// * `content` - The string content to normalize
///
/// # Returns
///
/// A string with all line endings converted to LF (`\n`)
///
/// # Example
///
/// ```
/// # use samoyed::hooks::normalize_line_endings;
/// let windows_content = "#!/bin/sh\r\necho 'hello'\r\n";
/// let normalized = normalize_line_endings(windows_content);
/// assert_eq!(normalized, "#!/bin/sh\necho 'hello'\n");
/// ```
pub fn normalize_line_endings(content: &str) -> String {
    // Replace CRLF (\r\n) and standalone CR (\r) with LF (\n)
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Creates the hooks directory and its `.gitignore` file
///
/// This function creates the directory where all hook files will be stored
/// and adds a `.gitignore` file containing `*` to prevent the hooks from
/// being committed to version control.
///
/// # Arguments
///
/// * `fs` - File system abstraction for directory and file operations
/// * `hooks_dir` - Path to the hooks directory to create
///
/// # Returns
///
/// * `Ok(())` - If the directory and `.gitignore` were created successfully
/// * `Err(HookError)` - If any file system operation fails
///
/// # Example
///
/// ```
/// use samoyed::hooks::create_hook_directory;
/// use samoyed::environment::SystemFileSystem;
/// use std::path::Path;
///
/// let fs = SystemFileSystem;
/// let hooks_dir = Path::new(".samoyed/_");
/// create_hook_directory(&fs, hooks_dir).expect("Failed to create hooks directory");
/// ```
pub fn create_hook_directory(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    fs.create_dir_all(hooks_dir)?;

    let gitignore_path = hooks_dir.join(".gitignore");
    let gitignore_content = normalize_line_endings("*");
    fs.write(&gitignore_path, &gitignore_content)?;

    Ok(())
}

/// Creates all standard Git hook files
///
/// This function creates a hook file for each standard Git hook. Each hook
/// file is a simple shell script that delegates to the `samoyed-hook` binary
/// runner. All hook files are made executable (mode 0755).
///
/// # Arguments
///
/// * `fs` - File system abstraction for file operations
/// * `hooks_dir` - Directory where hook files should be created
///
/// # Returns
///
/// * `Ok(())` - If all hook files were created successfully
/// * `Err(HookError)` - If any file operation fails
///
/// # Hook File Format
///
/// Each hook file contains:
/// ```bash
/// #!/usr/bin/env sh
/// exec samoyed-hook "$(basename "$0")" "$@"
/// ```
///
/// This delegates to the `samoyed-hook` binary, passing the hook name and all arguments.
pub fn create_hook_files(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    let hook_content = r#"#!/usr/bin/env sh
exec samoyed-hook "$(basename "$0")" "$@""#;

    // Normalize line endings to LF for cross-platform compatibility
    let normalized_content = normalize_line_endings(hook_content);

    for &hook_name in STANDARD_HOOKS {
        let hook_path = hooks_dir.join(hook_name);
        fs.write(&hook_path, &normalized_content)?;
        fs.set_permissions(&hook_path, 0o755)?;
    }

    Ok(())
}

/// Creates example hook scripts for common Git hooks
///
/// This function creates example hook scripts in a scripts subdirectory that users
/// can customize. These are the actual scripts that will be executed by the hook
/// runner when the corresponding Git hook is triggered.
///
/// # Arguments
///
/// * `fs` - File system abstraction for file operations
/// * `hooks_base_dir` - Base directory (e.g., `.samoyed`) where scripts should be created
///
/// # Returns
///
/// * `Ok(())` - If the example scripts were created successfully
/// * `Err(HookError)` - If any file operation fails
///
/// # Created Examples
///
/// Creates example scripts for:
/// - `pre-commit`: Basic formatting and linting example
/// - `pre-push`: Basic testing example
/// - Other hooks get placeholder examples
pub fn create_example_hook_scripts(
    fs: &dyn FileSystem,
    hooks_base_dir: &Path,
) -> Result<(), HookError> {
    // Create the scripts directory
    let scripts_dir = hooks_base_dir.join("scripts");
    fs.create_dir_all(&scripts_dir)?;
    // Create a few example hook scripts that users can customize
    let examples = [
        (
            "pre-commit",
            r#"#!/usr/bin/env sh
#
# ┌─────────────────────────────────────────────────────────────────────────────┐
# │                        SAMOYED HOOK ARCHITECTURE                            │
# └─────────────────────────────────────────────────────────────────────────────┘
#
# This script represents the FALLBACK mechanism in Samoyed's two-tier lookup:
#
# ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
# │ Git Commit  │────▶│ .samoyed/_/      │────▶│ samoyed-hook    │
# └─────────────┘     │   pre-commit     │     │   binary        │
#                     └──────────────────┘     └─────────────────┘
#                                                        │
#                      ┌─────────────────────────────────┼─────────────────────────────────┐
#                      │                                 ▼                                 │
#                      │  ┌─────────────────────────────────────────────────────────────┐  │
#                      │  │              TWO-TIER LOOKUP SYSTEM                         │  │
#                      │  └─────────────────────────────────────────────────────────────┘  │
#                      │                                                                   │
#                      │  1 PRIMARY: Check samoyed.toml                                    │
#                      │     ┌─────────────────┐                                           │
#                      │     │ samoyed.toml    │  ✓ Found: Execute command via shell       │
#                      │     │ [hooks]         │  ✕ Not found: Continue to fallback        │
#                      │     │ pre-commit = …  │                                           │
#                      │     └─────────────────┘                                           │
#                      │                                                                   │
#                      │  2 FALLBACK: Execute this script file                             │
#                      │     ┌─────────────────┐                                           │
#                      │     │ .samoyed/       │  ✓ Found: Execute script file             │
#                      │     │   scripts/      │  ✕ Not found: Exit silently (success)     │
#                      │     │   pre-commit    │                                           │
#                      │     └─────────────────┘                                           │
#                      └───────────────────────────────────────────────────────────────────┘
#
# ⚡ WHEN IS THIS SCRIPT EXECUTED?
# This script runs when:
# - No command is defined for 'pre-commit' in samoyed.toml, OR
# - You prefer using script files for complex multi-line logic
#
# 🎛 CONFIGURATION OPTIONS:
# Option 1 - samoyed.toml (Recommended for simple commands):
#   [hooks]
#   pre-commit = "cargo fmt --check && cargo clippy -- -D warnings"
#
# Option 2 - This script file (For complex workflows):
#   Customize the script below for advanced logic, conditionals, or multi-step processes
#
# 🖥️ ENVIRONMENT VARIABLES:
# - SAMOYED=0  Skip all hook execution
# - SAMOYED=1  Normal execution (default)
# - SAMOYED=2  Debug mode with detailed tracing
#
# Example pre-commit hook
# Add your formatting, linting, or other pre-commit checks here

echo "Running pre-commit checks..."

# Example: Run formatter (uncomment and customize as needed)
# cargo fmt --check
# npm run format:check
# black --check .

echo "Pre-commit checks passed!"
"#,
        ),
        (
            "pre-push",
            r#"#!/usr/bin/env sh
#
# ┌─────────────────────────────────────────────────────────────────────────────┐
# │                        SAMOYED HOOK ARCHITECTURE                            │
# └─────────────────────────────────────────────────────────────────────────────┘
#
# This script represents the FALLBACK mechanism in Samoyed's two-tier lookup:
#
# ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
# │ Git Push    │────▶│ .samoyed/_/      │────▶│ samoyed-hook    │
# └─────────────┘     │   pre-push       │     │   binary        │
#                     └──────────────────┘     └─────────────────┘
#                                                        │
#                      ┌─────────────────────────────────┼─────────────────────────────────┐
#                      │                                 ▼                                 │
#                      │  ┌─────────────────────────────────────────────────────────────┐  │
#                      │  │              TWO-TIER LOOKUP SYSTEM                         │  │
#                      │  └─────────────────────────────────────────────────────────────┘  │
#                      │                                                                   │
#                      │  1 PRIMARY: Check samoyed.toml                                    │
#                      │     ┌─────────────────┐                                           │
#                      │     │ samoyed.toml    │  ✓ Found: Execute command via shell       │
#                      │     │ [hooks]         │  ✕ Not found: Continue to fallback        │
#                      │     │ pre-push = …    │                                           │
#                      │     └─────────────────┘                                           │
#                      │                                                                   │
#                      │  2 FALLBACK: Execute this script file                             │
#                      │     ┌─────────────────┐                                           │
#                      │     │ .samoyed/       │  ✓ Found: Execute script file             │
#                      │     │   scripts/      │  ✕ Not found: Exit silently (success)     │
#                      │     │   pre-push      │                                           │
#                      │     └─────────────────┘                                           │
#                      └───────────────────────────────────────────────────────────────────┘
#
# ⚡ WHEN IS THIS SCRIPT EXECUTED?
# This script runs when:
# - No command is defined for 'pre-push' in samoyed.toml, OR
# - You prefer using script files for complex multi-line logic
#
# 🎛 CONFIGURATION OPTIONS:
# Option 1 - samoyed.toml (Recommended for simple commands):
#   [hooks]
#   pre-push = "cargo test --release"
#
# Option 2 - This script file (For complex workflows):
#   Customize the script below for advanced logic, conditionals, or multi-step processes
#
# 🖥️ ENVIRONMENT VARIABLES:
# - SAMOYED=0  Skip all hook execution
# - SAMOYED=1  Normal execution (default)
# - SAMOYED=2  Debug mode with detailed tracing
#
# Example pre-push hook
# Add your test runs or other pre-push validations here

echo "Running pre-push validations..."

# Example: Run tests (uncomment and customize as needed)
# cargo test
# npm test
# pytest

echo "Pre-push validations passed!"
"#,
        ),
    ];

    for (hook_name, content) in &examples {
        let script_path = scripts_dir.join(hook_name);
        // Only create if it doesn't already exist (don't overwrite user customizations)
        if !fs.exists(&script_path) {
            let normalized_content = normalize_line_endings(content);
            fs.write(&script_path, &normalized_content)?;
            fs.set_permissions(&script_path, 0o755)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "unit_tests/hooks_tests.rs"]
mod tests;
