//! Git hook file creation and management
//!
//! This module handles the creation and configuration of Git hook files.
//! It creates the hook directory structure, individual hook scripts, and
//! the hook runner that executes user-defined hook commands.
//!
//! # Hook Structure
//!
//! The module creates:
//! - A hooks directory (e.g., `.samoid/_`)
//! - A `.gitignore` file to exclude hooks from version control
//! - Individual hook scripts for all standard Git hooks
//! - A hook runner script (`h`) that executes the actual hook logic

use crate::environment::FileSystem;
use std::path::Path;

/// List of all standard Git hooks that Samoid manages
///
/// This array contains the names of all Git hooks that Samoid will create
/// during installation. Each hook is a simple shell script that delegates
/// to the main hook runner.
///
/// # Supported Hooks
///
/// - **pre-commit**: Runs before a commit is created
/// - **pre-merge-commit**: Runs before a merge commit
/// - **prepare-commit-msg**: Prepares the default commit message
/// - **commit-msg**: Validates or modifies commit messages
/// - **post-commit**: Runs after a commit is created
/// - **applypatch-msg**: Can edit the commit message file for patches
/// - **pre-applypatch**: Runs before a patch is applied
/// - **post-applypatch**: Runs after a patch is applied
/// - **pre-rebase**: Runs before a rebase operation
/// - **post-rewrite**: Runs after commits are rewritten
/// - **post-checkout**: Runs after a successful checkout
/// - **post-merge**: Runs after a successful merge
/// - **pre-push**: Runs before a push operation
/// - **pre-auto-gc**: Runs before automatic garbage collection
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

/// Errors that can occur during hook file operations
///
/// Currently wraps I/O errors that may occur when creating directories
/// or writing hook files.
#[derive(Debug)]
pub enum HookError {
    /// An I/O error occurred during file system operations
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

/// Creates the hooks directory and its `.gitignore` file
///
/// This function creates the directory where all hook files will be stored
/// and adds a `.gitignore` file containing `*` to prevent the hooks from
/// being committed to version control.
///
/// # Arguments
///
/// * `fs` - File system abstraction for directory and file operations
/// * `hooks_dir` - Path to the hooks directory to create
///
/// # Returns
///
/// * `Ok(())` - If the directory and `.gitignore` were created successfully
/// * `Err(HookError)` - If any file system operation fails
///
/// # Example
///
/// ```
/// use samoid::hooks::create_hook_directory;
/// use samoid::environment::SystemFileSystem;
/// use std::path::Path;
///
/// let fs = SystemFileSystem;
/// let hooks_dir = Path::new(".samoid/_");
/// create_hook_directory(&fs, hooks_dir).expect("Failed to create hooks directory");
/// ```
pub fn create_hook_directory(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    fs.create_dir_all(hooks_dir)?;

    let gitignore_path = hooks_dir.join(".gitignore");
    fs.write(&gitignore_path, "*")?;

    Ok(())
}

/// Creates all standard Git hook files
///
/// This function creates a hook file for each standard Git hook. Each hook
/// file is a simple shell script that sources and executes the main hook
/// runner (`h`). All hook files are made executable (mode 0755).
///
/// # Arguments
///
/// * `fs` - File system abstraction for file operations
/// * `hooks_dir` - Directory where hook files should be created
///
/// # Returns
///
/// * `Ok(())` - If all hook files were created successfully
/// * `Err(HookError)` - If any file operation fails
///
/// # Hook File Format
///
/// Each hook file contains:
/// ```bash
/// #!/usr/bin/env sh
/// . "$(dirname "$0")/h"
/// ```
///
/// This sources the hook runner `h` from the same directory as the hook.
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

/// Creates or copies the main hook runner script
///
/// The hook runner (`h`) is the central script that all individual hooks
/// delegate to. It can either be copied from an existing source file or
/// created with placeholder content.
///
/// # Arguments
///
/// * `fs` - File system abstraction for file operations
/// * `hooks_dir` - Directory where the runner should be created
/// * `runner_source` - Optional path to an existing runner script to copy
///
/// # Returns
///
/// * `Ok(())` - If the runner was created successfully
/// * `Err(HookError)` - If any file operation fails
///
/// # Behavior
///
/// - If `runner_source` is provided, its contents are copied
/// - If `runner_source` is `None`, a placeholder runner is created
/// - The runner is always made executable (mode 0755)
///
/// # Placeholder Runner
///
/// When no source is provided, the following placeholder is created:
/// ```bash
/// #!/usr/bin/env sh
/// echo "Samoid hook runner - placeholder implementation"
/// exec "$@"
/// ```
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
