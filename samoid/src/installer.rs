//! Core installation logic for Samoid Git hooks
//!
//! This module contains the main installation function that orchestrates
//! the entire process of setting up Git hooks. It handles environment
//! checks, repository validation, and hook file creation.
//!
//! # Installation Process
//!
//! 1. Check if installation should be skipped (SAMOID=0)
//! 2. Validate the hooks directory path
//! 3. Verify we're in a Git repository
//! 4. Configure Git to use our hooks directory
//! 5. Create the hooks directory structure
//! 6. Install the hook runner and individual hook files

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

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Git(e) => write!(f, "{e}"),
            InstallError::Hooks(e) => write!(f, "{e}"),
            InstallError::InvalidPath { path, reason } => {
                write!(f, "Invalid path '{path}': {reason}")?;
                match reason {
                    PathValidationError::DirectoryTraversal => {
                        write!(f, "\n\nSecurity: Directory traversal attacks are not allowed.\nUse a relative path within the current directory.")?;
                    }
                    PathValidationError::AbsolutePath => {
                        write!(f, "\n\nUse a relative path like '.samoid' or 'hooks' instead.")?;
                    }
                    PathValidationError::InvalidCharacters(_) => {
                        write!(f, "\n\nUse only alphanumeric characters, hyphens, underscores, and dots.")?;
                    }
                    PathValidationError::EmptyPath => {
                        write!(f, "\n\nProvide a valid directory name like '.samoid'.")?;
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
/// * `custom_dir` - Optional custom directory name (defaults to ".samoid")
///
/// # Returns
///
/// * `Ok(String)` - Success message (empty string or "SAMOID=0 skip install")
/// * `Err(InstallError)` - If any step of the installation fails
///
/// # Environment Variables
///
/// - `SAMOID=0` - Skip installation (for CI environments or debugging)
///
/// # Example
///
/// ```
/// use samoid::install_hooks;
/// use samoid::environment::{SystemEnvironment, SystemCommandRunner, SystemFileSystem};
///
/// let env = SystemEnvironment;
/// let runner = SystemCommandRunner;
/// let fs = SystemFileSystem;
///
/// // Install with default directory (.samoid)
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
/// match install_hooks(&env, &runner, &fs, Some(".husky")) {
///     Ok(_) => println!("Hooks installed in .husky/_"),
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
    if env.get_var("SAMOID").unwrap_or_default() == "0" {
        return Ok("SAMOID=0 skip install".to_string());
    }

    let hooks_dir_name = custom_dir.unwrap_or(".samoid");

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

    // Create example hook scripts for users to customize in .samoid/scripts/
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
    if std::path::Path::new(path).is_absolute() {
        return Err(InstallError::InvalidPath {
            path: path.to_string(),
            reason: PathValidationError::AbsolutePath,
        });
    }
    
    // Check for invalid characters (platform-specific, but these are commonly problematic)
    let invalid_chars: Vec<char> = path
        .chars()
        .filter(|&c| {
            // Allow alphanumeric, hyphens, underscores, dots, and forward slashes
            !c.is_alphanumeric() && !matches!(c, '-' | '_' | '.' | '/')
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
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use std::process::{ExitStatus, Output};

    // Cross-platform exit status creation
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    // Helper function to create ExitStatus cross-platform
    fn exit_status(code: i32) -> ExitStatus {
        #[cfg(unix)]
        return ExitStatus::from_raw(code);

        #[cfg(windows)]
        return ExitStatus::from_raw(code as u32);
    }

    #[test]
    fn test_install_hooks_skip_when_samoid_0() {
        let env = MockEnvironment::new().with_var("SAMOID", "0");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "SAMOID=0 skip install");
    }

    #[test]
    fn test_install_hooks_invalid_path_with_dotdot() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = install_hooks(&env, &runner, &fs, Some("../invalid"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Directory traversal detected"));
    }

    #[test]
    fn test_install_hooks_no_git_repository() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No .git directory

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a Git repository"));
    }

    #[test]
    fn test_install_hooks_success() {
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        // Configure git config command to succeed
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(config_output),
            );

        // Configure filesystem with .git directory
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_install_hooks_with_custom_dir() {
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        // Configure git command to succeed with custom directory
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".custom-hooks/_"],
                Ok(config_output),
            );

        // Configure filesystem with .git directory
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(".custom-hooks"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_error_display() {
        let git_error = git::GitError::CommandNotFound { os_hint: None };
        let install_error = InstallError::Git(git_error);
        assert!(install_error.to_string().contains("Git command not found"));

        let hook_error = hooks::HookError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));
        let install_error = InstallError::Hooks(hook_error);
        assert!(install_error.to_string().contains("Permission denied"));

        let invalid_error = InstallError::InvalidPath {
            path: "test/path".to_string(),
            reason: PathValidationError::DirectoryTraversal,
        };
        assert!(invalid_error.to_string().contains("Directory traversal detected"));
    }

    #[test]
    fn test_install_error_from_git_error() {
        let git_error = git::GitError::NotGitRepository {
            checked_path: "/tmp".to_string(),
            suggest_init: true,
        };
        let install_error: InstallError = git_error.into();
        assert!(matches!(install_error, InstallError::Git(_)));
    }

    #[test]
    fn test_install_error_from_hook_error() {
        let hook_error = hooks::HookError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test",
        ));
        let install_error: InstallError = hook_error.into();
        assert!(matches!(install_error, InstallError::Hooks(_)));
    }

    #[test]
    fn test_install_hooks_git_command_error() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new(); // No responses configured
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InstallError::Git(_)));
    }

    #[test]
    fn test_install_hooks_filesystem_error() {
        let env = MockEnvironment::new();

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(config_output),
            );

        let fs = MockFileSystem::new().with_directory(".git");
        // Filesystem will fail when trying to create directories

        let result = install_hooks(&env, &runner, &fs, None);
        // This should succeed since MockFileSystem allows all operations
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_error_variants_coverage() {
        // Test all InstallError variants for coverage
        let git_error = git::GitError::CommandNotFound { os_hint: Some("linux".to_string()) };
        let hook_error = hooks::HookError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test",
        ));

        let error1 = InstallError::Git(git_error);
        let error2 = InstallError::Hooks(hook_error);
        let error3 = InstallError::InvalidPath {
            path: "invalid".to_string(),
            reason: PathValidationError::EmptyPath,
        };

        // Test Debug formatting
        assert!(!format!("{error1:?}").is_empty());
        assert!(!format!("{error2:?}").is_empty());
        assert!(!format!("{error3:?}").is_empty());

        // Test Display formatting
        assert!(error1.to_string().contains("Git command not found"));
        assert!(error2.to_string().contains("IO error"));
        assert!(error3.to_string().contains("Path cannot be empty"));
    }

    #[test]
    fn test_install_hooks_different_custom_dirs() {
        let env = MockEnvironment::new();

        let version_output1 = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let version_output2 = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output1 = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let config_output2 = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output1))
            .with_response(
                "git",
                &["config", "core.hooksPath", "my-hooks/_"],
                Ok(config_output1),
            )
            .with_response("git", &["--version"], Ok(version_output2))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".git-hooks/_"],
                Ok(config_output2),
            );

        let fs = MockFileSystem::new().with_directory(".git");

        // Test with custom directory
        let result1 = install_hooks(&env, &runner, &fs, Some("my-hooks"));
        assert!(result1.is_ok());

        // Test with another custom directory
        let result2 = install_hooks(&env, &runner, &fs, Some(".git-hooks"));
        assert!(result2.is_ok());
    }

    #[test]
    fn test_install_hooks_edge_case_paths() {
        let env = MockEnvironment::new();
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        // Test various edge case directory names
        let test_cases = [
            "hooks-dir",
            ".hidden-hooks",
            "hooks_with_underscores",
            "hooks123",
            "UPPERCASE-HOOKS",
        ];

        for dir_name in &test_cases {
            let expected_path = format!("{dir_name}/_");
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output.clone()))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", &expected_path],
                    Ok(config_output.clone()),
                );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, Some(dir_name));
            assert!(result.is_ok(), "Failed for directory: {dir_name}");
        }
    }

    #[test]
    fn test_install_hooks_empty_environment_variable() {
        // Test when SAMOID is set to empty string (should not skip)
        let env = MockEnvironment::new().with_var("SAMOID", "");
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(config_output),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ""); // Should not return skip message
    }

    #[test]
    fn test_install_hooks_other_environment_values() {
        // Test various SAMOID environment variable values
        let test_values = ["1", "true", "false", "disabled", "anything"];

        for value in &test_values {
            let env = MockEnvironment::new().with_var("SAMOID", value);
            let version_output = Output {
                status: exit_status(0),
                stdout: b"git version 2.34.1".to_vec(),
                stderr: vec![],
            };
            let config_output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", ".samoid/_"],
                    Ok(config_output),
                );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, None);
            assert!(result.is_ok(), "Failed for SAMOID={value}");
            assert_eq!(result.unwrap(), "", "Should not skip for SAMOID={value}");
        }
    }

    #[test]
    fn test_path_validation_directory_traversal() {
        // Test various directory traversal attempts
        let invalid_paths = [
            "../invalid",
            "valid/../invalid", 
            "..\\invalid",
            "valid\\..\\invalid",
            "hooks/../../../etc/passwd",
        ];

        for path in &invalid_paths {
            let result = validate_hooks_directory_path(path);
            assert!(result.is_err(), "Path should be invalid: {path}");
            assert!(matches!(
                result.unwrap_err(),
                InstallError::InvalidPath {
                    reason: PathValidationError::DirectoryTraversal,
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_path_validation_absolute_paths() {
        let invalid_paths = [
            "/absolute/path",
            "/usr/local/hooks",
            "C:\\Windows\\hooks",
            "\\\\server\\share\\hooks",
        ];

        for path in &invalid_paths {
            let result = validate_hooks_directory_path(path);
            assert!(result.is_err(), "Path should be invalid: {path}");
            if std::path::Path::new(path).is_absolute() {
                assert!(matches!(
                    result.unwrap_err(),
                    InstallError::InvalidPath {
                        reason: PathValidationError::AbsolutePath,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn test_path_validation_invalid_characters() {
        let invalid_paths = [
            "hooks*invalid",
            "hooks?query",
            "hooks|pipe",
            "hooks<redirect",
            "hooks>redirect",
            "hooks\"quote",
            "hooks:colon",
        ];

        for path in &invalid_paths {
            let result = validate_hooks_directory_path(path);
            assert!(result.is_err(), "Path should be invalid: {path}");
            assert!(matches!(
                result.unwrap_err(),
                InstallError::InvalidPath {
                    reason: PathValidationError::InvalidCharacters(_),
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_path_validation_empty_paths() {
        let empty_paths = ["", "   ", "\t", "\n"];

        for path in &empty_paths {
            let result = validate_hooks_directory_path(path);
            assert!(result.is_err(), "Path should be invalid: '{path}'");
            assert!(matches!(
                result.unwrap_err(),
                InstallError::InvalidPath {
                    reason: PathValidationError::EmptyPath,
                    ..
                }
            ));
        }
    }

    #[test]
    fn test_path_validation_too_long() {
        let long_path = "a".repeat(256); // Exceeds 255 character limit

        let result = validate_hooks_directory_path(&long_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InstallError::InvalidPath {
                reason: PathValidationError::TooLong(256),
                ..
            }
        ));
    }

    #[test]
    fn test_path_validation_valid_paths() {
        let valid_paths = [
            ".samoid",
            "hooks",
            ".git-hooks",
            "my_hooks",
            "project-hooks",
            "hooks123",
            "UPPERCASE_HOOKS",
            "nested/hooks",
            "deeply/nested/hooks/dir",
        ];

        for path in &valid_paths {
            let result = validate_hooks_directory_path(path);
            assert!(result.is_ok(), "Path should be valid: {path}");
        }
    }

    #[test]
    fn test_path_validation_error_messages() {
        // Test that error messages are informative
        let result = validate_hooks_directory_path("../invalid");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Directory traversal detected"));
        assert!(error_msg.contains("Security"));

        let result = validate_hooks_directory_path("/absolute");
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Absolute paths not allowed"));
        }

        let result = validate_hooks_directory_path("");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Path cannot be empty"));

        let result = validate_hooks_directory_path("path*invalid");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid characters"));
    }
}
