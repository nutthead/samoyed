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

pub fn install_hooks(custom_dir: Option<&str>) -> Result<String, InstallError> {
    if env::var("HUSKY").unwrap_or_default() == "0" {
        return Ok("HUSKY=0 skip install".to_string());
    }

    let hooks_dir_name = custom_dir.unwrap_or(".samoid");

    if hooks_dir_name.contains("..") {
        return Err(InstallError::InvalidPath(".. not allowed".to_string()));
    }

    git::check_git_repository()?;

    let hooks_path = format!("{}/_", hooks_dir_name);
    git::set_hooks_path(&hooks_path)?;

    let hooks_dir = PathBuf::from(&hooks_path);

    hooks::create_hook_directory(&hooks_dir)?;

    hooks::copy_hook_runner(&hooks_dir, None)?;

    hooks::create_hook_files(&hooks_dir)?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_install_hooks_skip_when_husky_0() {
        // Set HUSKY=0 environment variable
        unsafe {
            env::set_var("HUSKY", "0");
        }

        let result = install_hooks(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HUSKY=0 skip install");

        // Clean up
        unsafe {
            env::remove_var("HUSKY");
        }
    }

    #[test]
    fn test_install_hooks_invalid_path_with_dotdot() {
        let result = install_hooks(Some("../invalid"));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, InstallError::InvalidPath(_)));
        assert_eq!(error.to_string(), ".. not allowed");
    }

    #[test]
    fn test_install_hooks_no_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = install_hooks(None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InstallError::Git(_)));
    }

    #[test]
    fn test_install_hooks_success_with_custom_dir() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create .git directory
        std::fs::create_dir(".git").unwrap();

        // Initialize git repo to make git commands work
        std::process::Command::new("git").arg("init").output().ok();

        let result = install_hooks(Some(".custom-hooks"));

        // Should succeed if git is available
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
        {
            assert!(result.is_ok());
        }
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
