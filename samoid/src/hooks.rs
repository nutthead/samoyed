use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

pub const STANDARD_HOOKS: &[&str] = &[
    "pre-commit",
    "pre-merge-commit", 
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-rebase",
    "post-rewrite",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-auto-gc",
];

#[derive(Debug)]
pub enum HookError {
    IoError(std::io::Error),
    PermissionError(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::IoError(e) => write!(f, "IO error: {}", e),
            HookError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
        }
    }
}

impl std::error::Error for HookError {}

impl From<std::io::Error> for HookError {
    fn from(error: std::io::Error) -> Self {
        HookError::IoError(error)
    }
}

pub fn create_hook_directory(hooks_dir: &Path) -> Result<(), HookError> {
    fs::create_dir_all(hooks_dir)?;
    
    let gitignore_path = hooks_dir.join(".gitignore");
    fs::write(&gitignore_path, "*")?;
    
    Ok(())
}

pub fn create_hook_files(hooks_dir: &Path) -> Result<(), HookError> {
    let hook_content = r#"#!/usr/bin/env sh
. "$(dirname "$0")/h""#;

    for &hook_name in STANDARD_HOOKS {
        let hook_path = hooks_dir.join(hook_name);
        fs::write(&hook_path, hook_content)?;
        
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }
    
    Ok(())
}

pub fn copy_hook_runner(hooks_dir: &Path, runner_source: Option<&Path>) -> Result<(), HookError> {
    let runner_dest = hooks_dir.join("h");
    
    if let Some(source) = runner_source {
        fs::copy(source, &runner_dest)?;
    } else {
        let placeholder_runner = r#"#!/usr/bin/env sh
echo "Samoid hook runner - placeholder implementation"
exec "$@""#;
        fs::write(&runner_dest, placeholder_runner)?;
    }
    
    let mut perms = fs::metadata(&runner_dest)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&runner_dest, perms)?;
    
    Ok(())
}