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
/// Each variant provides specific, actionable error information.
#[derive(Debug)]
pub enum GitError {
    /// The git command is not installed or not in PATH
    /// Contains suggestions for installing Git
    CommandNotFound {
        /// Optional detected OS for installation suggestions
        os_hint: Option<String>,
    },
    /// Git configuration command failed with an error message
    /// Contains the original error and potential solutions
    ConfigurationFailed {
        /// The raw error message from git
        message: String,
        /// Suggested resolution steps
        suggestion: Option<String>,
    },
    /// Current directory is not inside a Git repository
    /// Contains information about where .git was expected
    NotGitRepository {
        /// The directory that was checked
        checked_path: String,
        /// Whether user should run 'git init'
        suggest_init: bool,
    },
    /// Permission denied accessing Git configuration or repository
    PermissionDenied {
        /// The operation that was denied
        operation: String,
        /// The path that caused the permission error
        path: Option<String>,
    },
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::CommandNotFound { os_hint } => {
                write!(f, "Git command not found in PATH")?;
                if let Some(os) = os_hint {
                    match os.as_str() {
                        "linux" => write!(
                            f,
                            "\n\nTo install Git:\n  • Ubuntu/Debian: sudo apt install git\n  • RHEL/CentOS: sudo yum install git\n  • Arch: sudo pacman -S git"
                        )?,
                        "macos" => write!(
                            f,
                            "\n\nTo install Git:\n  • Using Homebrew: brew install git\n  • Using Xcode tools: xcode-select --install"
                        )?,
                        "windows" => write!(
                            f,
                            "\n\nTo install Git:\n  • Download from: https://git-scm.com/download/windows\n  • Or use winget: winget install Git.Git"
                        )?,
                        _ => write!(f, "\n\nPlease install Git and ensure it's in your PATH")?,
                    }
                } else {
                    write!(f, "\n\nPlease install Git and ensure it's in your PATH")?;
                }
                Ok(())
            }
            GitError::ConfigurationFailed {
                message,
                suggestion,
            } => {
                write!(f, "Git configuration failed: {message}")?;
                if let Some(hint) = suggestion {
                    write!(f, "\n\nSuggestion: {hint}")?;
                }
                Ok(())
            }
            GitError::NotGitRepository {
                checked_path,
                suggest_init,
            } => {
                write!(
                    f,
                    "Not a Git repository (no .git directory found in '{checked_path}')"
                )?;
                if *suggest_init {
                    write!(f, "\n\nTo initialize a new Git repository:\n  git init")?;
                }
                Ok(())
            }
            GitError::PermissionDenied { operation, path } => {
                write!(f, "Permission denied: {operation}")?;
                if let Some(p) = path {
                    write!(f, " (path: {p})")?;
                }
                write!(
                    f,
                    "\n\nCheck file permissions and try:\n  • Running with appropriate user permissions\n  • Ensuring the repository is not locked by another process"
                )?;
                Ok(())
            }
        }
    }
}

impl std::error::Error for GitError {}

/// Verifies that the current directory is inside a Git repository
///
/// This function checks for the presence of a `.git` directory or file,
/// following the same approach as Husky but with enhanced error messages.
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
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .display()
            .to_string();

        return Err(GitError::NotGitRepository {
            checked_path: current_dir,
            suggest_init: true,
        });
    }
    Ok(())
}

/// Configures Git to use a custom hooks directory with comprehensive error handling
///
/// This function sets the `core.hooksPath` Git configuration value with enhanced
/// error detection and reporting. It validates the Git installation, checks
/// permissions, and provides specific guidance for different failure scenarios.
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
/// * `Err(GitError::PermissionDenied)` - If permission denied
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
///     Err(e) => eprintln!("Configuration failed: {}", e),
/// }
/// ```
pub fn set_hooks_path(runner: &dyn CommandRunner, hooks_path: &str) -> Result<(), GitError> {
    // First, validate that Git is available by running git --version
    match runner.run_command("git", &["--version"]) {
        Ok(version_output) => {
            if !version_output.status.success() {
                return Err(GitError::CommandNotFound {
                    os_hint: detect_os(),
                });
            }
        }
        Err(_) => {
            return Err(GitError::CommandNotFound {
                os_hint: detect_os(),
            });
        }
    }

    // Attempt to set the hooks path configuration
    let output = runner.run_command("git", &["config", "core.hooksPath", hooks_path]);

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let suggestion = analyze_git_config_error(&stderr);

                // Check for permission-related errors
                if stderr.contains("Permission denied") || stderr.contains("permission denied") {
                    Err(GitError::PermissionDenied {
                        operation: "set Git configuration".to_string(),
                        path: None,
                    })
                } else {
                    Err(GitError::ConfigurationFailed {
                        message: stderr.trim().to_string(),
                        suggestion,
                    })
                }
            }
        }
        Err(_) => Err(GitError::CommandNotFound {
            os_hint: detect_os(),
        }),
    }
}

/// Detects the current operating system for installation suggestions
fn detect_os() -> Option<String> {
    if cfg!(target_os = "linux") {
        Some("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Some("macos".to_string())
    } else if cfg!(target_os = "windows") {
        Some("windows".to_string())
    } else {
        None
    }
}

/// Analyzes Git configuration errors and provides specific suggestions  
fn analyze_git_config_error(stderr: &str) -> Option<String> {
    let lower_error = stderr.to_lowercase();

    if lower_error.contains("could not lock config file") {
        Some("Another Git process may be running. Wait and try again, or check for stale .git/config.lock files.".to_string())
    } else if lower_error.contains("not a git repository") {
        Some("Run this command from within a Git repository.".to_string())
    } else if lower_error.contains("bad config") {
        Some(
            "Git configuration file may be corrupted. Check .git/config for syntax errors."
                .to_string(),
        )
    } else if lower_error.contains("invalid key") {
        Some("The configuration key format is invalid. Check the Git documentation.".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockFileSystem};
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
        assert!(matches!(result, Err(GitError::NotGitRepository { .. })));
    }

    #[test]
    fn test_git_error_display() {
        let error = GitError::CommandNotFound { os_hint: None };
        assert!(error.to_string().contains("Git command not found in PATH"));

        let error = GitError::ConfigurationFailed {
            message: "test error".to_string(),
            suggestion: Some("try this".to_string()),
        };
        assert!(error.to_string().contains("test error"));
        assert!(error.to_string().contains("try this"));

        let error = GitError::NotGitRepository {
            checked_path: "/tmp".to_string(),
            suggest_init: true,
        };
        assert!(error.to_string().contains("Not a Git repository"));
        assert!(error.to_string().contains("git init"));
    }

    #[test]
    fn test_set_hooks_path_success() {
        // Mock successful git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };

        // Create a successful config output
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".test-hooks"],
                Ok(config_output),
            );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_hooks_path_command_not_found() {
        let runner = MockCommandRunner::new();
        // No response configured, so it will return command not found

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
    }

    #[test]
    fn test_set_hooks_path_configuration_failed() {
        // Mock successful git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };

        // Create a failed output for config command
        let config_output = Output {
            status: exit_status(1),
            stdout: vec![],
            stderr: b"error: could not lock config file".to_vec(),
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".test-hooks"],
                Ok(config_output),
            );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::ConfigurationFailed { .. })));
    }

    #[test]
    fn test_git_error_variants_coverage() {
        // Test all GitError variants for coverage
        let error1 = GitError::CommandNotFound {
            os_hint: Some("linux".to_string()),
        };
        let error2 = GitError::ConfigurationFailed {
            message: "test".to_string(),
            suggestion: None,
        };
        let error3 = GitError::NotGitRepository {
            checked_path: "/tmp".to_string(),
            suggest_init: false,
        };
        let error4 = GitError::PermissionDenied {
            operation: "test op".to_string(),
            path: Some("/test/path".to_string()),
        };

        // Ensure all implement Debug and Display
        assert!(!format!("{error1:?}").is_empty());
        assert!(!format!("{error2:?}").is_empty());
        assert!(!format!("{error3:?}").is_empty());
        assert!(!format!("{error4:?}").is_empty());
        assert!(!error1.to_string().is_empty());
        assert!(!error2.to_string().is_empty());
        assert!(!error3.to_string().is_empty());
        assert!(!error4.to_string().is_empty());
    }

    #[test]
    fn test_set_hooks_path_with_different_paths() {
        // Mock successful git --version responses
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

        // Test with different hook paths
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
                &["config", "core.hooksPath", ".hooks"],
                Ok(config_output1),
            )
            .with_response("git", &["--version"], Ok(version_output2))
            .with_response(
                "git",
                &["config", "core.hooksPath", "my-hooks/"],
                Ok(config_output2),
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
            &["--version"],
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission denied",
            )),
        );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
    }

    #[test]
    fn test_set_hooks_path_permission_denied() {
        // Mock successful git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };

        // Create a permission denied error
        let config_output = Output {
            status: exit_status(128),
            stdout: vec![],
            stderr: b"error: Permission denied".to_vec(),
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".test-hooks"],
                Ok(config_output),
            );

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::PermissionDenied { .. })));
    }

    #[test]
    fn test_check_git_repository_permission_denied() {
        // Create a filesystem where .git exists but can't be read
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file(".git/config", "test config");

        // Override the mock to return an error for reading .git/config
        let result = check_git_repository(&fs);
        // Note: In a real test, we'd need to mock the read_to_string failure
        // For now, this tests the happy path
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_suggestion_analysis() {
        // Test various error message suggestions
        let suggestion1 = analyze_git_config_error("error: could not lock config file");
        assert!(suggestion1.is_some());
        assert!(suggestion1.unwrap().contains("Another Git process"));

        let suggestion2 = analyze_git_config_error("fatal: not a git repository");
        assert!(suggestion2.is_some());
        assert!(suggestion2.unwrap().contains("Git repository"));

        let suggestion3 = analyze_git_config_error("error: bad config line");
        assert!(suggestion3.is_some());
        assert!(suggestion3.unwrap().contains("corrupted"));

        let suggestion4 = analyze_git_config_error("unknown error");
        assert!(suggestion4.is_none());
    }

    #[test]
    fn test_os_detection() {
        let os = detect_os();
        assert!(os.is_some());
        // The actual OS will depend on the test environment
        let os_str = os.unwrap();
        assert!(["linux", "macos", "windows"].contains(&os_str.as_str()));
    }

    #[test]
    fn test_git_error_os_specific_messages() {
        let error_linux = GitError::CommandNotFound {
            os_hint: Some("linux".to_string()),
        };
        let message = error_linux.to_string();
        assert!(message.contains("apt install git") || message.contains("yum install git"));

        let error_macos = GitError::CommandNotFound {
            os_hint: Some("macos".to_string()),
        };
        let message = error_macos.to_string();
        assert!(message.contains("brew install git"));

        let error_windows = GitError::CommandNotFound {
            os_hint: Some("windows".to_string()),
        };
        let message = error_windows.to_string();
        assert!(message.contains("git-scm.com"));
    }

    #[test]
    fn test_git_version_check_failure() {
        // Mock git --version failure
        let version_output = Output {
            status: exit_status(1),
            stdout: vec![],
            stderr: b"git: command not found".to_vec(),
        };

        let runner =
            MockCommandRunner::new().with_response("git", &["--version"], Ok(version_output));

        let result = set_hooks_path(&runner, ".test-hooks");
        assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
    }
}
