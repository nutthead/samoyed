//! Git repository operations and configuration
//!
//! This module provides Git-specific functionality for Samoid, including
//! repository detection and Git configuration management. All operations
//! use dependency injection for testability.

use crate::environment::{CommandRunner, FileSystem};
use std::path::Path;

/// Errors that can occur during Git operations
///
/// This enum represents all possible failures when interacting with Git,
/// from missing Git installations to repository configuration issues.
#[derive(Debug)]
pub enum GitError {
    /// The git command is not installed or not in PATH
    CommandNotFound,
    /// Git configuration command failed with an error message
    ConfigurationFailed(String),
    /// Current directory is not inside a Git repository
    NotGitRepository,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::CommandNotFound => write!(f, "git command not found"),
            GitError::ConfigurationFailed(msg) => write!(f, "{}", msg),
            GitError::NotGitRepository => write!(f, ".git can't be found"),
        }
    }
}

impl std::error::Error for GitError {}

/// Verifies that the current directory is inside a Git repository
///
/// This function checks for the presence of a `.git` directory or file.
/// Git worktrees use a `.git` file pointing to the actual repository,
/// so this function correctly handles both cases.
///
/// # Arguments
///
/// * `fs` - File system abstraction for checking path existence
///
/// # Returns
///
/// * `Ok(())` - If we're inside a Git repository
/// * `Err(GitError::NotGitRepository)` - If no `.git` exists
///
/// # Example
///
/// ```
/// use samoid::git::check_git_repository;
/// use samoid::environment::SystemFileSystem;
///
/// let fs = SystemFileSystem;
/// match check_git_repository(&fs) {
///     Ok(()) => println!("Inside a Git repository"),
///     Err(e) => eprintln!("Not a Git repository: {}", e),
/// }
/// ```
pub fn check_git_repository(fs: &dyn FileSystem) -> Result<(), GitError> {
    if !fs.exists(Path::new(".git")) {
        return Err(GitError::NotGitRepository);
    }
    Ok(())
}

/// Configures Git to use a custom hooks directory
///
/// This function sets the `core.hooksPath` Git configuration value,
/// which tells Git where to find hook scripts. This is the core
/// mechanism that allows Samoid to manage Git hooks.
///
/// # Arguments
///
/// * `runner` - Command runner for executing git commands
/// * `hooks_path` - Path to the hooks directory (e.g., ".samoid/_")
///
/// # Returns
///
/// * `Ok(())` - If the configuration was set successfully
/// * `Err(GitError::CommandNotFound)` - If git command is not available
/// * `Err(GitError::ConfigurationFailed)` - If git config command fails
///
/// # Example
///
/// ```
/// use samoid::git::set_hooks_path;
/// use samoid::environment::SystemCommandRunner;
///
/// let runner = SystemCommandRunner;
/// match set_hooks_path(&runner, ".samoid/_") {
///     Ok(()) => println!("Hooks path configured"),
///     Err(e) => eprintln!("Failed to configure: {}", e),
/// }
/// ```
pub fn set_hooks_path(runner: &dyn CommandRunner, hooks_path: &str) -> Result<(), GitError> {
    let output = runner.run_command("git", &["config", "core.hooksPath", hooks_path]);

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(GitError::ConfigurationFailed(stderr.to_string()))
            }
        }
        Err(_) => Err(GitError::CommandNotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockFileSystem};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn test_check_git_repository_exists() {
        // Create a mock filesystem with .git directory
        let fs = MockFileSystem::new().with_directory(".git");

        let result = check_git_repository(&fs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_git_repository_missing() {
        // Create a mock filesystem without .git directory
        let fs = MockFileSystem::new();

        let result = check_git_repository(&fs);
        assert!(matches!(result, Err(GitError::NotGitRepository)));
    }

    #[test]
    fn test_git_error_display() {
        let error = GitError::CommandNotFound;
        assert_eq!(error.to_string(), "git command not found");

        let error = GitError::ConfigurationFailed("test error".to_string());
        assert_eq!(error.to_string(), "test error");

        let error = GitError::NotGitRepository;
        assert_eq!(error.to_string(), ".git can't be found");
    }

    #[test]
    fn test_set_hooks_path_success() {
        // Create a successful output
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Ok(output),
        );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_hooks_path_command_not_found() {
        let runner = MockCommandRunner::new();
        // No response configured, so it will return command not found

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::CommandNotFound)));
    }

    #[test]
    fn test_set_hooks_path_configuration_failed() {
        // Create a failed output
        let output = Output {
            status: ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"error: could not lock config file".to_vec(),
        };

        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Ok(output),
        );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::ConfigurationFailed(_))));
    }

    #[test]
    fn test_git_error_variants_coverage() {
        // Test all GitError variants for coverage
        let error1 = GitError::CommandNotFound;
        let error2 = GitError::ConfigurationFailed("test".to_string());
        let error3 = GitError::NotGitRepository;

        // Ensure all implement Debug and Display
        assert!(!format!("{:?}", error1).is_empty());
        assert!(!format!("{:?}", error2).is_empty());
        assert!(!format!("{:?}", error3).is_empty());
        assert!(!error1.to_string().is_empty());
        assert!(!error2.to_string().is_empty());
        assert!(!error3.to_string().is_empty());
    }

    #[test]
    fn test_set_hooks_path_with_different_paths() {
        // Test with different hook paths
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
            .with_response("git", &["config", "core.hooksPath", ".hooks"], Ok(output1))
            .with_response(
                "git",
                &["config", "core.hooksPath", "my-hooks/"],
                Ok(output2),
            );

        let result1 = set_hooks_path(&runner, ".hooks");
        assert!(result1.is_ok());

        let result2 = set_hooks_path(&runner, "my-hooks/");
        assert!(result2.is_ok());
    }

    #[test]
    fn test_check_git_repository_with_different_filesystems() {
        // Test with filesystem that has .git directory
        let fs_with_git = MockFileSystem::new().with_directory(".git");
        let result1 = check_git_repository(&fs_with_git);
        assert!(result1.is_ok());

        // Test with filesystem that has .git file
        let fs_with_git_file =
            MockFileSystem::new().with_file(".git", "gitdir: ../.git/worktrees/branch");
        let result2 = check_git_repository(&fs_with_git_file);
        assert!(result2.is_ok());

        // Test with filesystem that has no .git
        let fs_no_git = MockFileSystem::new();
        let result3 = check_git_repository(&fs_no_git);
        assert!(result3.is_err());
    }

    #[test]
    fn test_set_hooks_path_with_io_error() {
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission denied",
            )),
        );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::CommandNotFound)));
    }
}
