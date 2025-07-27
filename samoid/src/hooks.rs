use crate::environment::FileSystem;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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

pub fn create_hook_directory(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    fs.create_dir_all(hooks_dir)?;

    let gitignore_path = hooks_dir.join(".gitignore");
    fs.write(&gitignore_path, "*")?;

    Ok(())
}

pub fn create_hook_files(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    let hook_content = r#"#!/usr/bin/env sh
. "$(dirname "$0")/h""#;

    for &hook_name in STANDARD_HOOKS {
        let hook_path = hooks_dir.join(hook_name);
        fs.write(&hook_path, hook_content)?;
        fs.set_permissions(&hook_path, 0o755)?;
    }

    Ok(())
}

pub fn copy_hook_runner(
    fs: &dyn FileSystem,
    hooks_dir: &Path,
    runner_source: Option<&Path>,
) -> Result<(), HookError> {
    let runner_dest = hooks_dir.join("h");

    if let Some(source) = runner_source {
        let content = fs.read_to_string(source)?;
        fs.write(&runner_dest, &content)?;
    } else {
        let placeholder_runner = r#"#!/usr/bin/env sh
echo "Samoid hook runner - placeholder implementation"
exec "$@""#;
        fs.write(&runner_dest, placeholder_runner)?;
    }

    fs.set_permissions(&runner_dest, 0o755)?;

    Ok(())
}

/// Legacy functions for backward compatibility (used by main)
pub fn create_hook_directory_legacy(hooks_dir: &Path) -> Result<(), HookError> {
    fs::create_dir_all(hooks_dir)?;

    let gitignore_path = hooks_dir.join(".gitignore");
    fs::write(&gitignore_path, "*")?;

    Ok(())
}

pub fn create_hook_files_legacy(hooks_dir: &Path) -> Result<(), HookError> {
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

pub fn copy_hook_runner_legacy(
    hooks_dir: &Path,
    runner_source: Option<&Path>,
) -> Result<(), HookError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::MockFileSystem;

    #[test]
    fn test_create_hook_directory() {
        let fs = MockFileSystem::new();
        let hooks_dir = std::path::Path::new(".samoid/_");

        let result = create_hook_directory(&fs, hooks_dir);
        assert!(result.is_ok());

        // Verify the mock filesystem recorded the operations
        assert!(fs.exists(hooks_dir));
        assert!(fs.exists(&hooks_dir.join(".gitignore")));
    }

    #[test]
    fn test_create_hook_files() {
        let fs = MockFileSystem::new();
        let hooks_dir = std::path::Path::new(".samoid/_");

        let result = create_hook_files(&fs, hooks_dir);
        assert!(result.is_ok());

        // Verify all hooks were created
        for &hook in STANDARD_HOOKS {
            assert!(fs.exists(&hooks_dir.join(hook)));
        }
    }

    #[test]
    fn test_copy_hook_runner_with_placeholder() {
        let fs = MockFileSystem::new();
        let hooks_dir = std::path::Path::new(".samoid/_");

        let result = copy_hook_runner(&fs, hooks_dir, None);
        assert!(result.is_ok());

        assert!(fs.exists(&hooks_dir.join("h")));
    }

    #[test]
    fn test_copy_hook_runner_with_source() {
        let fs =
            MockFileSystem::new().with_file("/tmp/runner.sh", "#!/bin/sh\necho 'custom runner'");
        let hooks_dir = std::path::Path::new(".samoid/_");

        let result = copy_hook_runner(&fs, hooks_dir, Some(std::path::Path::new("/tmp/runner.sh")));
        assert!(result.is_ok());

        assert!(fs.exists(&hooks_dir.join("h")));
    }

    #[test]
    fn test_hook_error_display() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let hook_error = HookError::IoError(io_error);
        assert!(hook_error.to_string().contains("Permission denied"));

        let perm_error = HookError::PermissionError("Custom permission error".to_string());
        assert_eq!(
            perm_error.to_string(),
            "Permission error: Custom permission error"
        );
    }

    #[test]
    fn test_hook_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let hook_error: HookError = io_error.into();
        assert!(matches!(hook_error, HookError::IoError(_)));
    }
}
