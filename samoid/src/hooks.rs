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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_hook_directory_success() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("test_hooks");

        let result = create_hook_directory(&hooks_dir);
        assert!(result.is_ok());

        assert!(hooks_dir.exists());
        let gitignore_path = hooks_dir.join(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert_eq!(content, "*");
    }

    #[test]
    fn test_create_hook_files_success() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("test_hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let result = create_hook_files(&hooks_dir);
        assert!(result.is_ok());

        // Check that all standard hooks were created
        for &hook_name in STANDARD_HOOKS {
            let hook_path = hooks_dir.join(hook_name);
            assert!(hook_path.exists(), "Hook {} should exist", hook_name);

            let content = fs::read_to_string(&hook_path).unwrap();
            assert_eq!(content, "#!/usr/bin/env sh\n. \"$(dirname \"$0\")/h\"");

            // Check permissions (on Unix systems)
            #[cfg(unix)]
            {
                let metadata = fs::metadata(&hook_path).unwrap();
                let permissions = metadata.permissions();
                assert_eq!(permissions.mode() & 0o777, 0o755);
            }
        }
    }

    #[test]
    fn test_copy_hook_runner_with_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("test_hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let result = copy_hook_runner(&hooks_dir, None);
        assert!(result.is_ok());

        let runner_path = hooks_dir.join("h");
        assert!(runner_path.exists());

        let content = fs::read_to_string(&runner_path).unwrap();
        assert!(content.contains("Samoid hook runner - placeholder implementation"));

        // Check permissions
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&runner_path).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o755);
        }
    }

    #[test]
    fn test_copy_hook_runner_with_source() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("test_hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Create a source file
        let source_path = temp_dir.path().join("source_runner");
        fs::write(&source_path, "#!/bin/sh\necho 'custom runner'").unwrap();

        let result = copy_hook_runner(&hooks_dir, Some(&source_path));
        assert!(result.is_ok());

        let runner_path = hooks_dir.join("h");
        assert!(runner_path.exists());

        let content = fs::read_to_string(&runner_path).unwrap();
        assert_eq!(content, "#!/bin/sh\necho 'custom runner'");
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
