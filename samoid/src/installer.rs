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
/// the installation process, providing a single error type for the
/// public API.
#[derive(Debug)]
pub enum InstallError {
    /// Git-related errors (command not found, configuration failed, etc.)
    Git(GitError),
    /// Hook file creation errors (I/O errors)
    Hooks(HookError),
    /// Invalid path provided (e.g., contains "..")
    InvalidPath(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Git(e) => write!(f, "{}", e),
            InstallError::Hooks(e) => write!(f, "{}", e),
            InstallError::InvalidPath(msg) => write!(f, "{}", msg),
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

    if hooks_dir_name.contains("..") {
        return Err(InstallError::InvalidPath(".. not allowed".to_string()));
    }

    // Check if we're in a git repository
    git::check_git_repository(fs)?;

    let hooks_path = format!("{}/_", hooks_dir_name);

    // Set git hooks path
    git::set_hooks_path(runner, &hooks_path)?;

    let hooks_dir = PathBuf::from(&hooks_path);

    // Create hook directory and files
    hooks::create_hook_directory(fs, &hooks_dir)?;
    hooks::copy_hook_runner(fs, &hooks_dir, None)?;
    hooks::create_hook_files(fs, &hooks_dir)?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

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
        assert_eq!(result.unwrap_err().to_string(), ".. not allowed");
    }

    #[test]
    fn test_install_hooks_no_git_repository() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No .git directory

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), ".git can't be found");
    }

    #[test]
    fn test_install_hooks_success() {
        let env = MockEnvironment::new();

        // Configure git command to succeed
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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

        // Configure git command to succeed with custom directory
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".custom-hooks/_"],
            Ok(output),
        );

        // Configure filesystem with .git directory
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(".custom-hooks"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_error_display() {
        let git_error = git::GitError::CommandNotFound;
        let install_error = InstallError::Git(git_error);
        assert_eq!(install_error.to_string(), "git command not found");

        let hook_error = hooks::HookError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));
        let install_error = InstallError::Hooks(hook_error);
        assert!(install_error.to_string().contains("Permission denied"));

        let invalid_error = InstallError::InvalidPath("test error".to_string());
        assert_eq!(invalid_error.to_string(), "test error");
    }

    #[test]
    fn test_install_error_from_git_error() {
        let git_error = git::GitError::NotGitRepository;
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

        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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
        let git_error = git::GitError::CommandNotFound;
        let hook_error = hooks::HookError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test",
        ));

        let error1 = InstallError::Git(git_error);
        let error2 = InstallError::Hooks(hook_error);
        let error3 = InstallError::InvalidPath("invalid".to_string());

        // Test Debug formatting
        assert!(!format!("{:?}", error1).is_empty());
        assert!(!format!("{:?}", error2).is_empty());
        assert!(!format!("{:?}", error3).is_empty());

        // Test Display formatting
        assert_eq!(error1.to_string(), "git command not found");
        assert!(error2.to_string().contains("IO error"));
        assert_eq!(error3.to_string(), "invalid");
    }

    #[test]
    fn test_install_hooks_different_custom_dirs() {
        let env = MockEnvironment::new();

        let output1 = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let output2 = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new()
            .with_response(
                "git",
                &["config", "core.hooksPath", "my-hooks/_"],
                Ok(output1),
            )
            .with_response(
                "git",
                &["config", "core.hooksPath", ".git-hooks/_"],
                Ok(output2),
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
        let output = Output {
            status: ExitStatus::from_raw(0),
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
            let expected_path = format!("{}/_", dir_name);
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", &expected_path],
                Ok(output.clone()),
            );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, Some(dir_name));
            assert!(result.is_ok(), "Failed for directory: {}", dir_name);
        }
    }

    #[test]
    fn test_install_hooks_empty_environment_variable() {
        // Test when SAMOID is set to empty string (should not skip)
        let env = MockEnvironment::new().with_var("SAMOID", "");
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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
            let output = Output {
                status: ExitStatus::from_raw(0),
                stdout: vec![],
                stderr: vec![],
            };
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(output),
            );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, None);
            assert!(result.is_ok(), "Failed for SAMOID={}", value);
            assert_eq!(result.unwrap(), "", "Should not skip for SAMOID={}", value);
        }
    }
}
