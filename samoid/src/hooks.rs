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
            HookError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for HookError {}

impl From<std::io::Error> for HookError {
    fn from(error: std::io::Error) -> Self {
        HookError::IoError(error)
    }
}

/// Normalizes line endings to Unix-style (LF) for cross-platform compatibility
///
/// This function ensures that all generated files use consistent LF line endings,
/// regardless of the platform they're created on. This is important because:
///
/// - Git Bash and Unix shells expect LF endings
/// - Git's `core.autocrlf` settings can cause issues with mixed line endings
/// - Hook scripts should be executable on all platforms
///
/// # Arguments
///
/// * `content` - The string content to normalize
///
/// # Returns
///
/// A string with all line endings converted to LF (`\n`)
///
/// # Example
///
/// ```
/// # use samoid::hooks::normalize_line_endings;
/// let windows_content = "#!/bin/sh\r\necho 'hello'\r\n";
/// let normalized = normalize_line_endings(windows_content);
/// assert_eq!(normalized, "#!/bin/sh\necho 'hello'\n");
/// ```
pub fn normalize_line_endings(content: &str) -> String {
    // Replace CRLF (\r\n) and standalone CR (\r) with LF (\n)
    content.replace("\r\n", "\n").replace('\r', "\n")
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
    let gitignore_content = normalize_line_endings("*");
    fs.write(&gitignore_path, &gitignore_content)?;

    Ok(())
}

/// Creates all standard Git hook files
///
/// This function creates a hook file for each standard Git hook. Each hook
/// file is a simple shell script that delegates to the `samoid-hook` binary
/// runner. All hook files are made executable (mode 0755).
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
/// exec samoid-hook "$(basename "$0")" "$@"
/// ```
///
/// This delegates to the `samoid-hook` binary, passing the hook name and all arguments.
pub fn create_hook_files(fs: &dyn FileSystem, hooks_dir: &Path) -> Result<(), HookError> {
    let hook_content = r#"#!/usr/bin/env sh
exec samoid-hook "$(basename "$0")" "$@""#;

    // Normalize line endings to LF for cross-platform compatibility
    let normalized_content = normalize_line_endings(hook_content);

    for &hook_name in STANDARD_HOOKS {
        let hook_path = hooks_dir.join(hook_name);
        fs.write(&hook_path, &normalized_content)?;
        fs.set_permissions(&hook_path, 0o755)?;
    }

    Ok(())
}

/// Creates example hook scripts for common Git hooks
///
/// This function creates example hook scripts in a scripts subdirectory that users
/// can customize. These are the actual scripts that will be executed by the hook
/// runner when the corresponding Git hook is triggered.
///
/// # Arguments
///
/// * `fs` - File system abstraction for file operations
/// * `hooks_base_dir` - Base directory (e.g., `.samoid`) where scripts should be created
///
/// # Returns
///
/// * `Ok(())` - If the example scripts were created successfully
/// * `Err(HookError)` - If any file operation fails
///
/// # Created Examples
///
/// Creates example scripts for:
/// - `pre-commit`: Basic formatting and linting example
/// - `pre-push`: Basic testing example
/// - Other hooks get placeholder examples
pub fn create_example_hook_scripts(
    fs: &dyn FileSystem,
    hooks_base_dir: &Path,
) -> Result<(), HookError> {
    // Create the scripts directory
    let scripts_dir = hooks_base_dir.join("scripts");
    fs.create_dir_all(&scripts_dir)?;
    // Create a few example hook scripts that users can customize
    let examples = [
        (
            "pre-commit",
            r#"#!/usr/bin/env sh
# Example pre-commit hook
# Add your formatting, linting, or other pre-commit checks here

echo "Running pre-commit checks..."

# Example: Run formatter (uncomment and customize as needed)
# cargo fmt --check
# npm run format:check
# black --check .

echo "Pre-commit checks passed!"
"#,
        ),
        (
            "pre-push",
            r#"#!/usr/bin/env sh
# Example pre-push hook
# Add your test runs or other pre-push validations here

echo "Running pre-push validations..."

# Example: Run tests (uncomment and customize as needed)  
# cargo test
# npm test
# pytest

echo "Pre-push validations passed!"
"#,
        ),
    ];

    for (hook_name, content) in &examples {
        let script_path = scripts_dir.join(hook_name);
        // Only create if it doesn't already exist (don't overwrite user customizations)
        if !fs.exists(&script_path) {
            let normalized_content = normalize_line_endings(content);
            fs.write(&script_path, &normalized_content)?;
            fs.set_permissions(&script_path, 0o755)?;
        }
    }

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
    fn test_create_example_hook_scripts() {
        let fs = MockFileSystem::new();
        let hooks_base_dir = std::path::Path::new(".samoid");

        let result = create_example_hook_scripts(&fs, hooks_base_dir);
        assert!(result.is_ok());

        // Verify example scripts were created in scripts subdirectory
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
    }

    #[test]
    fn test_create_example_hook_scripts_no_overwrite() {
        let fs = MockFileSystem::new().with_file(
            ".samoid/scripts/pre-commit",
            "#!/bin/sh\n# User's existing script",
        );
        let hooks_base_dir = std::path::Path::new(".samoid");

        let result = create_example_hook_scripts(&fs, hooks_base_dir);
        assert!(result.is_ok());

        // Verify existing file was not overwritten (still exists)
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
        // Verify other example was still created
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
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
        assert!(!format!("{error1:?}").is_empty());
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
    fn test_create_example_hook_scripts_multiple_calls() {
        let fs = MockFileSystem::new();
        let hooks_base_dir = std::path::Path::new(".samoid");

        // First call should create examples
        let result1 = create_example_hook_scripts(&fs, hooks_base_dir);
        assert!(result1.is_ok());

        // Second call should not fail (examples already exist)
        let result2 = create_example_hook_scripts(&fs, hooks_base_dir);
        assert!(result2.is_ok());

        // Verify scripts still exist
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
        assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
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
    fn test_create_example_hook_scripts_different_directories() {
        let fs = MockFileSystem::new();

        let hooks_base_dir1 = std::path::Path::new(".hooks1");
        let hooks_base_dir2 = std::path::Path::new(".hooks2");
        let hooks_base_dir3 = std::path::Path::new(".samoid");

        // Test creating examples in different directories
        let result1 = create_example_hook_scripts(&fs, hooks_base_dir1);
        assert!(result1.is_ok());
        assert!(fs.exists(&hooks_base_dir1.join("scripts/pre-commit")));
        assert!(fs.exists(&hooks_base_dir1.join("scripts/pre-push")));

        let result2 = create_example_hook_scripts(&fs, hooks_base_dir2);
        assert!(result2.is_ok());
        assert!(fs.exists(&hooks_base_dir2.join("scripts/pre-commit")));
        assert!(fs.exists(&hooks_base_dir2.join("scripts/pre-push")));

        let result3 = create_example_hook_scripts(&fs, hooks_base_dir3);
        assert!(result3.is_ok());
        assert!(fs.exists(&hooks_base_dir3.join("scripts/pre-commit")));
        assert!(fs.exists(&hooks_base_dir3.join("scripts/pre-push")));
    }

    #[test]
    fn test_hook_error_error_trait() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let hook_error = HookError::IoError(io_error);

        // Test that it implements std::error::Error
        let error_trait: &dyn std::error::Error = &hook_error;
        assert!(!error_trait.to_string().is_empty());
    }

    #[test]
    fn test_normalize_line_endings_crlf() {
        let windows_content = "#!/bin/sh\r\necho 'hello'\r\necho 'world'\r\n";
        let normalized = normalize_line_endings(windows_content);
        assert_eq!(normalized, "#!/bin/sh\necho 'hello'\necho 'world'\n");
    }

    #[test]
    fn test_normalize_line_endings_cr() {
        let mac_classic_content = "#!/bin/sh\recho 'hello'\recho 'world'\r";
        let normalized = normalize_line_endings(mac_classic_content);
        assert_eq!(normalized, "#!/bin/sh\necho 'hello'\necho 'world'\n");
    }

    #[test]
    fn test_normalize_line_endings_mixed() {
        let mixed_content = "#!/bin/sh\r\necho 'hello'\recho 'world'\necho 'end'";
        let normalized = normalize_line_endings(mixed_content);
        assert_eq!(
            normalized,
            "#!/bin/sh\necho 'hello'\necho 'world'\necho 'end'"
        );
    }

    #[test]
    fn test_normalize_line_endings_already_lf() {
        let unix_content = "#!/bin/sh\necho 'hello'\necho 'world'\n";
        let normalized = normalize_line_endings(unix_content);
        assert_eq!(normalized, unix_content); // Should be unchanged
    }

    #[test]
    fn test_normalize_line_endings_empty() {
        let empty_content = "";
        let normalized = normalize_line_endings(empty_content);
        assert_eq!(normalized, "");
    }
}
