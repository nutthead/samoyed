use std::process::Command;
use std::path::Path;

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