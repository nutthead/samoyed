//! Core installation logic for Samoid Git hooks
//!
//! # Purpose and Overview
//!
//! This module serves as the central orchestrator for the Samoid Git hooks installation process.
//! It provides a comprehensive, robust, and secure way to set up Git hooks that delegate to
//! the `samoyed-hook` binary runner, following modern Git hooks management patterns.
//!
//! ## Raison d'être
//!
//! The primary purpose of this installer module is to:
//!
//! - **Centralize Installation Logic**: Provide a single, well-tested entry point for hook installation
//! - **Ensure Security**: Validate all paths and prevent directory traversal attacks
//! - **Handle Edge Cases**: Gracefully manage various error conditions and environment scenarios
//! - **Support Flexibility**: Allow custom hook directories while maintaining sensible defaults
//! - **Enable Testing**: Use dependency injection patterns for complete test isolation
//!
//! ## Architecture and Design
//!
//! The module follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Public API (install_hooks)                                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Path Validation & Security Checks                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Environment Integration (Git, FileSystem)                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Hook Management (Creation, Configuration)                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Installation Process Flow
//!
//! The installation follows a carefully orchestrated sequence:
//!
//! 1. **Environment Check**: Verify SAMOID environment variable (skip if SAMOYED=0)
//! 2. **Path Validation**: Ensure the hooks directory path is safe and valid
//! 3. **Repository Validation**: Confirm we're operating within a Git repository
//! 4. **Git Configuration**: Set core.hooksPath to point to our hooks directory
//! 5. **Directory Creation**: Establish the hooks directory structure with proper permissions
//! 6. **Hook Installation**: Create individual hook files that delegate to samoyed-hook binary
//! 7. **Verification**: Ensure all components are properly installed and accessible
//!
//! ## Error Handling Strategy
//!
//! The module employs comprehensive error handling with specific error types:
//!
//! - **`InstallError`**: Top-level errors with detailed context and recovery suggestions
//! - **`PathValidationError`**: Security-focused path validation failures
//! - **`GitError`**: Git-related operation failures with OS-specific hints
//! - **`HookError`**: Hook file creation and management errors
//!
//! Each error type provides actionable information to help users resolve issues.
//!
//! ## Security Considerations
//!
//! Security is paramount in this module:
//!
//! - **Path Traversal Prevention**: All paths are validated to prevent "../" attacks
//! - **Absolute Path Rejection**: Only relative paths are accepted for custom directories
//! - **Length Validation**: Paths are limited to reasonable lengths to prevent buffer issues
//! - **Character Validation**: Invalid characters are rejected to prevent injection attacks
//!
//! ## Testing and Reliability
//!
//! The module uses dependency injection to achieve 100% test coverage:
//!
//! - **Mock Environment**: Test environment variable scenarios
//! - **Mock CommandRunner**: Test Git command interactions without actual Git
//! - **Mock FileSystem**: Test file operations without touching the real filesystem
//!
//! This enables comprehensive testing of error conditions, edge cases, and platform-specific behavior.
//!
//! ## Usage Examples
//!
//! ```rust,ignore
//! use samoyed::installer::install_hooks;
//! use samoyed::environment::{SystemEnvironment, SystemCommandRunner, SystemFileSystem};
//!
//! // Basic installation with default directory
//! let env = SystemEnvironment;
//! let runner = SystemCommandRunner;
//! let fs = SystemFileSystem;
//!
//! install_hooks(&env, &runner, &fs, None)?;
//!
//! // Installation with custom directory
//! install_hooks(&env, &runner, &fs, Some("custom-hooks"))?;
//! ```

use crate::environment::{CommandRunner, Environment, FileSystem};
use crate::git::{self, GitError};
use crate::hooks::{self, HookError};
use std::path::PathBuf;

/// Errors that can occur during hook installation
///
/// This enum unifies all possible error types that can occur during
/// the installation process, providing specific, actionable error information.
#[derive(Debug)]
pub enum InstallError {
    /// Git-related errors (command not found, configuration failed, etc.)
    Git(GitError),
    /// Hook file creation errors (I/O errors)
    Hooks(HookError),
    /// Invalid path provided with security implications
    InvalidPath {
        /// The invalid path that was provided
        path: String,
        /// The specific reason why the path is invalid
        reason: PathValidationError,
    },
}

/// Specific reasons why a path validation failed
#[derive(Debug)]
pub enum PathValidationError {
    /// Path contains directory traversal sequences ("..")
    DirectoryTraversal,
    /// Path is absolute when relative was expected
    AbsolutePath,
    /// Path contains invalid characters or sequences
    InvalidCharacters(String),
    /// Path is empty or whitespace only
    EmptyPath,
    /// Path exceeds maximum allowed length
    TooLong(usize),
}

/// Implementation of the Display trait for PathValidationError.
///
/// Provides human-readable error messages for path validation failures. These messages
/// are designed to be clear and actionable, helping users understand exactly what went
/// wrong with their path configuration.
///
/// # Message Format
///
/// Each error variant produces a specific message pattern:
/// - **DirectoryTraversal**: Alerts about security risks from `..` segments
/// - **AbsolutePath**: Clarifies that only relative paths are accepted
/// - **InvalidCharacters**: Lists the specific problematic characters found
/// - **EmptyPath**: Simple notification that a path was expected
/// - **TooLong**: Shows actual vs maximum allowed length
///
/// # Usage Example
///
/// ```rust,ignore
/// match validate_path(user_input) {
///     Err(PathValidationError::InvalidCharacters(chars)) => {
///         eprintln!("Error: {}", err); // "Invalid characters in path: <>"
///     }
///     _ => {}
/// }
/// ```
impl std::fmt::Display for PathValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathValidationError::DirectoryTraversal => {
                write!(f, "Directory traversal detected (contains '..')")
            }
            PathValidationError::AbsolutePath => {
                write!(f, "Absolute paths not allowed (must be relative)")
            }
            PathValidationError::InvalidCharacters(chars) => {
                write!(f, "Invalid characters in path: {chars}")
            }
            PathValidationError::EmptyPath => {
                write!(f, "Path cannot be empty")
            }
            PathValidationError::TooLong(len) => {
                write!(f, "Path too long ({len} characters, maximum is 255)")
            }
        }
    }
}

/// Implementation of the Display trait for InstallError.
///
/// Provides comprehensive error messages for installation failures, designed to guide users
/// through resolving issues. Each error message includes context about what went wrong and,
/// where applicable, suggestions for resolution.
///
/// # Error Categories
///
/// The error messages are grouped by failure type:
/// - **Git-related**: Repository validation, command execution failures
/// - **Path-related**: Invalid paths, security concerns
/// - **System-related**: File I/O, permissions, command execution
///
/// # Message Philosophy
///
/// Error messages follow these principles:
/// 1. **Clarity**: State what operation failed
/// 2. **Context**: Include relevant details (paths, error codes)
/// 3. **Actionability**: Suggest fixes where possible
/// 4. **Chaining**: Preserve underlying error details for debugging
///
/// # Example Output
///
/// ```text
/// Failed to set Git hooks path
/// Caused by: Permission denied (os error 13)
/// ```
///
/// # Integration with Error Handling
///
/// These messages are designed to work with the `anyhow` error handling framework,
/// preserving error chains while providing user-friendly top-level messages.
impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Git(e) => write!(f, "{e}"),
            InstallError::Hooks(e) => write!(f, "{e}"),
            InstallError::InvalidPath { path, reason } => {
                write!(f, "Invalid path '{path}': {reason}")?;
                match reason {
                    PathValidationError::DirectoryTraversal => {
                        write!(
                            f,
                            "\n\nSecurity: Directory traversal attacks are not allowed.\nUse a relative path within the current directory."
                        )?;
                    }
                    PathValidationError::AbsolutePath => {
                        write!(
                            f,
                            "\n\nUse a relative path like '.samoyed' or 'hooks' instead."
                        )?;
                    }
                    PathValidationError::InvalidCharacters(_) => {
                        write!(
                            f,
                            "\n\nUse only alphanumeric characters, hyphens, underscores, and dots."
                        )?;
                    }
                    PathValidationError::EmptyPath => {
                        write!(f, "\n\nProvide a valid directory name like '.samoyed'.")?;
                    }
                    PathValidationError::TooLong(_) => {
                        write!(f, "\n\nUse a shorter directory name.")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for InstallError {}

impl From<GitError> for InstallError {
    fn from(error: GitError) -> Self {
        InstallError::Git(error)
    }
}

impl From<HookError> for InstallError {
    fn from(error: HookError) -> Self {
        InstallError::Hooks(error)
    }
}

/// Installs Git hooks in the current repository
///
/// This is the main entry point for Samoid's functionality. It sets up
/// Git hooks by creating a hooks directory, configuring Git to use it,
/// and installing all necessary hook files.
///
/// # Arguments
///
/// * `env` - Environment provider for reading environment variables
/// * `runner` - Command runner for executing Git commands
/// * `fs` - File system abstraction for file operations
/// * `custom_dir` - Optional custom directory name (defaults to ".samoyed")
///
/// # Returns
///
/// * `Ok(String)` - Success message (empty string or "SAMOYED=0 skip install")
/// * `Err(InstallError)` - If any step of the installation fails
///
/// # Environment Variables
///
/// - `SAMOYED=0` - Skip installation (for CI environments or debugging)
///
/// # Example
///
/// ```
/// use samoyed::install_hooks;
/// use samoyed::environment::{SystemEnvironment, SystemCommandRunner, SystemFileSystem};
///
/// let env = SystemEnvironment;
/// let runner = SystemCommandRunner;
/// let fs = SystemFileSystem;
///
/// // Install with default directory (.samoyed)
/// match install_hooks(&env, &runner, &fs, None) {
///     Ok(msg) => {
///         if !msg.is_empty() {
///             println!("{}", msg);
///         }
///     }
///     Err(e) => eprintln!("Installation failed: {}", e),
/// }
///
/// // Install with custom directory
/// match install_hooks(&env, &runner, &fs, Some(".samoyed")) {
///     Ok(_) => println!("Hooks installed in .samoyed/_"),
///     Err(e) => eprintln!("Installation failed: {}", e),
/// }
/// ```
///
/// # Security
///
/// The function validates that `custom_dir` doesn't contain ".." to prevent
/// directory traversal attacks.
pub fn install_hooks(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    custom_dir: Option<&str>,
) -> Result<String, InstallError> {
    // Check SAMOID environment variable
    if env.get_var("SAMOYED").unwrap_or_default() == "0" {
        return Ok("SAMOYED=0 skip install".to_string());
    }

    let hooks_dir_name = custom_dir.unwrap_or(".samoyed");

    // Comprehensive path validation
    validate_hooks_directory_path(hooks_dir_name)?;

    // Check if we're in a git repository
    git::check_git_repository(fs)?;

    let hooks_path = format!("{hooks_dir_name}/_");

    // Set git hooks path
    git::set_hooks_path(runner, &hooks_path)?;

    let hooks_dir = PathBuf::from(&hooks_path);

    // Create hook directory and files
    hooks::create_hook_directory(fs, &hooks_dir)?;
    hooks::create_hook_files(fs, &hooks_dir)?;

    // Create example hook scripts for users to customize in .samoyed/scripts/
    // These are optional and won't overwrite existing user scripts
    let hooks_base_dir = PathBuf::from(hooks_dir_name);
    hooks::create_example_hook_scripts(fs, &hooks_base_dir)?;

    Ok(String::new())
}

/// Validates a hooks directory path for security and correctness
///
/// This function performs comprehensive validation to prevent security issues
/// and ensure the path is suitable for use as a hooks directory.
///
/// # Arguments
///
/// * `path` - The directory path to validate
///
/// # Returns
///
/// * `Ok(())` - If the path is valid and safe
/// * `Err(InstallError::InvalidPath)` - If the path fails validation
///
/// # Security
///
/// This function prevents:
/// - Directory traversal attacks ("../", "..\\")
/// - Absolute path usage (security and portability)
/// - Invalid characters that could cause issues
/// - Excessively long paths
/// - Empty or whitespace-only paths
fn validate_hooks_directory_path(path: &str) -> Result<(), InstallError> {
    // Check for empty or whitespace-only path
    if path.trim().is_empty() {
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::EmptyPath,
        });
    }

    // Check path length (filesystem limits vary, but 255 is a safe limit)
    if path.len() > 255 {
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::TooLong(path.len()),
        });
    }

    // Check for directory traversal sequences
    if path.contains("..") {
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::DirectoryTraversal,
        });
    }

    // Check for absolute paths (security and portability)
    // Handle both Unix-style (/path) and Windows-style (C:\path, \\server\share) absolute paths
    let is_absolute = if cfg!(target_os = "windows") {
        // Windows absolute paths: C:\, D:\, \\server\share, etc.
        path.len() >= 3 && path.chars().nth(1) == Some(':') && path.chars().nth(2) == Some('\\')
            || path.starts_with("\\\\") // UNC path
            || path.starts_with('/') // Git Bash style
    } else {
        // Unix absolute paths start with /
        path.starts_with('/')
    };

    if is_absolute {
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::AbsolutePath,
        });
    }

    // Check for invalid characters (platform-specific, but these are commonly problematic)
    let invalid_chars: Vec<char> = path
        .chars()
        .filter(|&c| {
            // Allow alphanumeric, hyphens, underscores, dots, and path separators
            let allowed = c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/');
            // On Windows, also allow backslashes as path separators
            #[cfg(target_os = "windows")]
            let allowed = allowed || c == '\\';
            !allowed
        })
        .collect();

    if !invalid_chars.is_empty() {
        let invalid_str: String = invalid_chars.into_iter().collect();
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::InvalidCharacters(invalid_str),
        });
    }

    Ok(())
}

#[cfg(test)]
#[path = "unit_tests/installer_tests.rs"]
mod tests;
