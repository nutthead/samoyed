use crate::environment::{CommandRunner, Environment, FileSystem};
use crate::git::{self, GitError};
use crate::hooks::{self, HookError};
use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub enum InstallError {
    Git(GitError),
    Hooks(HookError),
    InvalidPath(String),
    Skipped(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Git(e) => write!(f, "{}", e),
            InstallError::Hooks(e) => write!(f, "{}", e),
            InstallError::InvalidPath(msg) => write!(f, "{}", msg),
            InstallError::Skipped(msg) => write!(f, "{}", msg),
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

/// Install hooks with dependency injection
pub fn install_hooks(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    custom_dir: Option<&str>,
) -> Result<String, InstallError> {
    // Check HUSKY environment variable
    if env.get_var("HUSKY").unwrap_or_default() == "0" {
        return Ok("HUSKY=0 skip install".to_string());
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

/// Legacy function for backward compatibility (used by main)
pub fn install_hooks_legacy(custom_dir: Option<&str>) -> Result<String, InstallError> {
    if env::var("HUSKY").unwrap_or_default() == "0" {
        return Ok("HUSKY=0 skip install".to_string());
    }

    let hooks_dir_name = custom_dir.unwrap_or(".samoid");

    if hooks_dir_name.contains("..") {
        return Err(InstallError::InvalidPath(".. not allowed".to_string()));
    }

    git::check_git_repository_legacy()?;

    let hooks_path = format!("{}/_", hooks_dir_name);
    git::set_hooks_path_legacy(&hooks_path)?;

    let hooks_dir = PathBuf::from(&hooks_path);

    hooks::create_hook_directory_legacy(&hooks_dir)?;
    hooks::copy_hook_runner_legacy(&hooks_dir, None)?;
    hooks::create_hook_files_legacy(&hooks_dir)?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn test_install_hooks_skip_when_husky_0() {
        let env = MockEnvironment::new().with_var("HUSKY", "0");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HUSKY=0 skip install");
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

        let skipped_error = InstallError::Skipped("skipped".to_string());
        assert_eq!(skipped_error.to_string(), "skipped");
    }

    #[test]
    fn test_install_error_from_git_error() {
        let git_error = git::GitError::NotGitRepository;
        let install_error: InstallError = git_error.into();
        assert!(matches!(install_error, InstallError::Git(_)));
    }

    #[test]
    fn test_install_error_from_hook_error() {
        let hook_error = hooks::HookError::PermissionError("test".to_string());
        let install_error: InstallError = hook_error.into();
        assert!(matches!(install_error, InstallError::Hooks(_)));
    }
}
