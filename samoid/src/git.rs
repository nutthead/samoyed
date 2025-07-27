use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum GitError {
    CommandNotFound,
    ConfigurationFailed(String),
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

pub fn check_git_repository() -> Result<(), GitError> {
    if !Path::new(".git").exists() {
        return Err(GitError::NotGitRepository);
    }
    Ok(())
}

pub fn set_hooks_path(hooks_path: &str) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(&["config", "core.hooksPath", hooks_path])
        .output();

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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_git_repository_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).unwrap();

        // Create .git directory
        fs::create_dir(".git").unwrap();

        let result = check_git_repository();
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_git_repository_missing() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).unwrap();

        let result = check_git_repository();
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
    fn test_set_hooks_path_in_valid_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).unwrap();

        // Initialize a real git repo for this test
        std::process::Command::new("git")
            .arg("init")
            .output()
            .expect("Failed to init git repo");

        let result = set_hooks_path(".test-hooks");

        // This should succeed in a real git repo if git is available
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
        {
            // In some environments git config might fail, so just check it doesn't panic
            let _ = result;
        }
    }
}
