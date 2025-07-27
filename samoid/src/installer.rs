use std::env;
use std::path::PathBuf;
use crate::git::{self, GitError};
use crate::hooks::{self, HookError};

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