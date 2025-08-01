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
/// with comprehensive error handling and detailed diagnostic messages.
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
/// use samoyed::git::check_git_repository;
/// use samoyed::environment::SystemFileSystem;
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
/// * `hooks_path` - Path to the hooks directory (e.g., ".samoyed/_")
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
/// use samoyed::git::set_hooks_path;
/// use samoyed::environment::SystemCommandRunner;
///
/// let runner = SystemCommandRunner;
/// match set_hooks_path(&runner, ".samoyed/_") {
///     Ok(()) => println!("Hooks path configured"),
///     Err(e) => eprintln!("Configuration failed: {}", e),
/// }
/// ```
pub fn set_hooks_path(runner: &dyn CommandRunner, hooks_path: &str) -> Result<(), GitError> {
    // Determine the git command name based on platform
    let git_cmd = get_git_command();

    // First, validate that Git is available by running git --version
    match runner.run_command(&git_cmd, &["--version"]) {
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
    let output = runner.run_command(&git_cmd, &["config", "core.hooksPath", hooks_path]);

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

/// Detects the current operating system at compile time for platform-specific installation suggestions.
///
/// This function uses Rust's compile-time configuration attributes (`cfg!`) to determine
/// the target operating system. The detection happens at compile time, not runtime, which means
/// the binary will always return the OS it was compiled for, not necessarily the OS it's running on
/// (though in practice these are usually the same).
///
/// # Returns
///
/// * `Some("linux")` - When compiled for Linux targets
/// * `Some("macos")` - When compiled for macOS targets  
/// * `Some("windows")` - When compiled for Windows targets
/// * `None` - When compiled for other platforms (e.g., BSD, Solaris)
///
/// # Example
///
/// ```rust,ignore
/// match detect_os() {
///     Some("linux") => println!("Install with: apt-get install git"),
///     Some("macos") => println!("Install with: brew install git"),
///     Some("windows") => println!("Download from: https://git-scm.com"),
///     _ => println!("Please install Git for your platform"),
/// }
/// ```
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

/// Get the appropriate git command name for the current platform
fn get_git_command() -> String {
    // On Windows, both "git" and "git.exe" should work, but we'll use "git"
    // consistently across all platforms. The CommandRunner implementations
    // should handle platform-specific command resolution.
    "git".to_string()
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
#[path = "unit_tests/git_tests.rs"]
mod tests;
