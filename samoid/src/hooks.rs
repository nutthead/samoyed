use crate::environment::FileSystem;
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
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::IoError(e) => write!(f, "IO error: {}", e),
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
    }

    #[test]
    fn test_hook_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let hook_error: HookError = io_error.into();
        assert!(matches!(hook_error, HookError::IoError(_)));
    }

    #[test]
    fn test_hook_error_variants_coverage() {
        // Test all HookError variants for coverage
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error1 = HookError::IoError(io_error);

        // Ensure all implement Debug and Display
        assert!(!format!("{:?}", error1).is_empty());
        assert!(error1.to_string().contains("IO error"));
    }

    #[test]
    fn test_standard_hooks_constant() {
        // Test that STANDARD_HOOKS contains expected hooks
        assert!(STANDARD_HOOKS.contains(&"pre-commit"));
        assert!(STANDARD_HOOKS.contains(&"post-commit"));
        assert!(STANDARD_HOOKS.contains(&"pre-push"));
        assert_eq!(STANDARD_HOOKS.len(), 14);
    }

    #[test]
    fn test_copy_hook_runner_with_read_error() {
        let fs = MockFileSystem::new(); // No files, so read will fail
        let hooks_dir = std::path::Path::new(".samoid/_");
        let source_path = std::path::Path::new("/nonexistent/file.sh");

        let result = copy_hook_runner(&fs, hooks_dir, Some(source_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_hook_files_with_multiple_directories() {
        let fs = MockFileSystem::new();

        // Test with different hook directories
        let hooks_dir1 = std::path::Path::new(".hooks/_");
        let result1 = create_hook_files(&fs, hooks_dir1);
        assert!(result1.is_ok());

        let hooks_dir2 = std::path::Path::new("custom/hooks");
        let result2 = create_hook_files(&fs, hooks_dir2);
        assert!(result2.is_ok());

        // Verify all hooks were created in both directories
        for &hook in STANDARD_HOOKS {
            assert!(fs.exists(&hooks_dir1.join(hook)));
            assert!(fs.exists(&hooks_dir2.join(hook)));
        }
    }

    #[test]
    fn test_create_hook_directory_with_multiple_paths() {
        let fs = MockFileSystem::new();

        // Test creating hook directories with different paths
        let dirs = [
            std::path::Path::new(".samoid/_"),
            std::path::Path::new(".hooks"),
            std::path::Path::new("custom/hooks/dir"),
        ];

        for dir in &dirs {
            let result = create_hook_directory(&fs, dir);
            assert!(result.is_ok());
            assert!(fs.exists(dir));
            assert!(fs.exists(&dir.join(".gitignore")));
        }
    }

    #[test]
    fn test_copy_hook_runner_multiple_scenarios() {
        let fs = MockFileSystem::new()
            .with_file("/custom/runner1.sh", "#!/bin/sh\necho 'runner1'")
            .with_file("/custom/runner2.sh", "#!/bin/sh\necho 'runner2'");

        let hooks_dir1 = std::path::Path::new(".hooks1/_");
        let hooks_dir2 = std::path::Path::new(".hooks2/_");
        let hooks_dir3 = std::path::Path::new(".hooks3/_");

        // Test with custom source 1
        let result1 = copy_hook_runner(
            &fs,
            hooks_dir1,
            Some(std::path::Path::new("/custom/runner1.sh")),
        );
        assert!(result1.is_ok());
        assert!(fs.exists(&hooks_dir1.join("h")));

        // Test with custom source 2
        let result2 = copy_hook_runner(
            &fs,
            hooks_dir2,
            Some(std::path::Path::new("/custom/runner2.sh")),
        );
        assert!(result2.is_ok());
        assert!(fs.exists(&hooks_dir2.join("h")));

        // Test with no source (placeholder)
        let result3 = copy_hook_runner(&fs, hooks_dir3, None);
        assert!(result3.is_ok());
        assert!(fs.exists(&hooks_dir3.join("h")));
    }

    #[test]
    fn test_hook_error_error_trait() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let hook_error = HookError::IoError(io_error);

        // Test that it implements std::error::Error
        let error_trait: &dyn std::error::Error = &hook_error;
        assert!(!error_trait.to_string().is_empty());
    }
}
