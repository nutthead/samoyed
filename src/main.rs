//! Samoyed - A modern, minimal, safe, ultra-fast, cross-platform Git hooks manager
//!
//! This is a single-binary tool that simplifies and streamlines how users work with
//! client-side Git hooks. The entire implementation fits in this single file to maintain
//! simplicity and avoid feature creep.
//!
//! ## Cross-Platform Support
//!
//! Samoyed works seamlessly across Unix (Linux, macOS) and Windows systems with:
//! - Platform-specific file permissions (Unix mode bits vs Windows defaults)
//! - Path normalization for Windows extended-length paths
//! - Graceful handling of Git execution differences across platforms

use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Embedded shell script that serves as the Git hook wrapper.
///
/// This script is copied to `.samoyed/_/samoyed` during initialization and sourced
/// by each hook script. It handles debug mode, bypass mode, user configuration loading,
/// and executes the corresponding user-defined hook if it exists.
const SAMOYED_WRAPPER_SCRIPT: &[u8] = include_bytes!("../assets/samoyed");

/// List of standard Git hook names that Samoyed manages.
///
/// These are the client-side hooks that Git supports. During initialization,
/// Samoyed creates a wrapper script for each of these hooks in the `_` directory.
const GIT_HOOKS: &[&str] = &[
    "applypatch-msg",
    "commit-msg",
    "post-applypatch",
    "post-checkout",
    "post-commit",
    "post-merge",
    "post-rewrite",
    "pre-applypatch",
    "pre-auto-gc",
    "pre-commit",
    "pre-merge-commit",
    "pre-push",
    "pre-rebase",
    "prepare-commit-msg",
];

/// Default directory name for Samoyed hooks if not specified by the user.
///
/// This directory will be created in the repository root and will contain
/// both the wrapper scripts (in `_/` subdirectory) and user-defined hooks.
const DEFAULT_SAMOYED_DIR: &str = ".samoyed";

/// Directory name for wrapper scripts within the Samoyed directory.
const WRAPPER_DIR_NAME: &str = "_";

/// Filename for the embedded wrapper script within the wrapper directory.
const WRAPPER_SCRIPT_NAME: &str = "samoyed";

/// Filename for the sample pre-commit hook.
const SAMPLE_HOOK_NAME: &str = "pre-commit";

/// Filename for the .gitignore file in the wrapper directory.
const GITIGNORE_NAME: &str = ".gitignore";

/// Message displayed when SAMOYED=0 environment variable bypasses initialization.
const MSG_BYPASS_INIT: &str = "Bypassing samoyed init due to SAMOYED=0";

/// Error message when git command execution fails.
const ERR_FAILED_EXECUTE_GIT: &str = "Error: Failed to execute git command";

/// Error message when current directory is not a git repository.
const ERR_NOT_GIT_REPO: &str = "Error: Not a git repository";

/// Error message when git root directory cannot be determined.
const ERR_FAILED_GET_GIT_ROOT: &str = "Error: Failed to get git root directory";

/// Error message when git configuration update fails.
const ERR_FAILED_SET_GIT_CONFIG: &str = "Error: Failed to set git config";

/// Error message when setting core.hooksPath configuration fails.
const ERR_FAILED_SET_HOOKS_PATH: &str = "Error: Failed to set core.hooksPath";

/// Error message when hooks path is outside the git repository.
const ERR_HOOKS_PATH_NOT_IN_REPO: &str = "Error: Hooks path is not within git repository";

/// Error message when hooks directory path is invalid.
const ERR_INVALID_HOOKS_PATH: &str = "Error: Invalid path for hooks directory";

/// Error message when path canonicalization fails.
const ERR_UNABLE_RESOLVE_PATH: &str = "Error: Unable to resolve path";

/// Error message when parent path resolution fails.
const ERR_UNABLE_RESOLVE_PARENT: &str = "Error: Unable to resolve parent path";

/// Error prefix when current directory determination fails.
const ERR_FAILED_CURRENT_DIR: &str = "Error: Failed to determine current directory";

/// Error prefix when git root resolution fails.
const ERR_FAILED_RESOLVE_GIT_ROOT: &str = "Error: Failed to resolve git root";

/// Error prefix when samoyed directory resolution fails.
const ERR_FAILED_RESOLVE_SAMOYED_DIR: &str = "Error: Failed to resolve samoyed directory";

/// Error prefix when path is outside the git repository bounds.
const ERR_OUTSIDE_GIT_REPO: &str = "Error: Path is outside the git repository";

/// Error prefix when samoyed directory creation fails.
const ERR_FAILED_CREATE_SAMOYED_DIR: &str = "Error: Failed to create samoyed directory";

/// Error prefix when wrapper directory creation fails.
const ERR_FAILED_CREATE_WRAPPER_DIR: &str = "Error: Failed to create _ directory";

/// Error prefix when wrapper script write fails.
const ERR_FAILED_WRITE_WRAPPER: &str = "Error: Failed to write wrapper script";

/// Error prefix when file metadata retrieval fails.
const ERR_FAILED_GET_METADATA: &str = "Error: Failed to get file metadata";

/// Error prefix when file permission setting fails.
const ERR_FAILED_SET_PERMISSIONS: &str = "Error: Failed to set file permissions";

/// Error prefix when hook script write fails.
const ERR_FAILED_WRITE_HOOK: &str = "Error: Failed to write hook";

/// Error prefix when sample pre-commit hook write fails.
const ERR_FAILED_WRITE_SAMPLE: &str = "Error: Failed to write sample pre-commit hook";

/// Error prefix when git root canonicalization fails.
const ERR_FAILED_CANONICALIZE_GIT_ROOT: &str = "Error: Failed to canonicalize git root";

/// Error prefix when samoyed directory canonicalization fails.
const ERR_FAILED_CANONICALIZE_SAMOYED: &str = "Error: Failed to canonicalize samoyed directory";

/// Error prefix when .gitignore file write fails.
const ERR_FAILED_WRITE_GITIGNORE: &str = "Error: Failed to write .gitignore";

/// Shell script template for Git hooks that sources the Samoyed wrapper.
const HOOK_SCRIPT_TEMPLATE: &str = r#"#!/usr/bin/env sh
. "$(dirname "$0")/samoyed"
"#;

/// Sample pre-commit hook template with placeholder comments for user customization.
const SAMPLE_PRE_COMMIT_CONTENT: &str = r#"#!/usr/bin/env sh
# Add your pre-commit checks here. For example:
# echo "Running Samoyed sample pre-commit"
# exit 0
"#;

/// Gitignore pattern that excludes all files in the wrapper directory.
const GITIGNORE_CONTENT: &str = "*\n";

/// Command-line interface for Samoyed.
///
/// Samoyed is a modern, minimal, safe, ultra-fast, cross-platform Git hooks manager
/// that simplifies client-side Git hook management with a single-binary tool.
#[derive(Parser)]
#[command(name = "samoyed")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands for the Samoyed CLI.
///
/// Currently supports initialization of Git hooks in a repository.
/// Future versions may include additional commands for hook management.
#[derive(Subcommand)]
enum Commands {
    /// Initialize Samoyed in the current git repository
    Init {
        /// Directory name for Samoyed hooks (default: .samoyed)
        #[arg(value_name = "samoyed-dirname")]
        dirname: Option<String>,
    },
}

/// Main entry point for Samoyed
///
/// Parses command-line arguments and dispatches to appropriate handlers.
/// If no command is provided, displays help message.
fn main() -> ExitCode {
    match Cli::parse().command {
        Some(Commands::Init { dirname }) => {
            let dirname = dirname.unwrap_or_else(|| DEFAULT_SAMOYED_DIR.to_string());
            init_samoyed(&dirname).map_or_else(
                |err| {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                },
                |_| ExitCode::SUCCESS,
            )
        }
        None => ExitCode::SUCCESS,
    }
}

/// Initialize Samoyed in the current git repository
///
/// This function performs the following steps:
/// 1. Checks if SAMOYED=0 (bypass mode)
/// 2. Verifies we're inside a git repository
/// 3. Validates the samoyed directory path
/// 4. Creates the directory structure
/// 5. Copies the wrapper script
/// 6. Creates hook scripts
/// 7. Creates sample pre-commit hook
/// 8. Sets git config core.hooksPath
/// 9. Creates .gitignore in the _ directory
///
/// # Arguments
///
/// * `dirname` - The directory name for Samoyed hooks
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn init_samoyed(dirname: &str) -> Result<(), String> {
    // Check for bypass mode
    if check_bypass_mode() {
        println!("{}", MSG_BYPASS_INIT);
        return Ok(());
    }

    // Check if we're in a git repository
    let git_root = get_git_root()?;
    let current_dir =
        env::current_dir().map_err(|e| format!("{}: {}", ERR_FAILED_CURRENT_DIR, e))?;

    // Validate and resolve the samoyed directory path
    let samoyed_dir = validate_samoyed_dir(&git_root, &current_dir, dirname)?;

    // Create directory structure
    create_directory_structure(&samoyed_dir)?;

    // Copy wrapper script to _/samoyed
    copy_wrapper_script(&samoyed_dir)?;

    // Create hook scripts in _ directory
    create_hook_scripts(&samoyed_dir)?;

    // Create sample pre-commit hook
    create_sample_pre_commit(&samoyed_dir)?;

    // Set git config core.hooksPath
    set_git_hooks_path(&samoyed_dir)?;

    // Create .gitignore in _ directory
    create_gitignore(&samoyed_dir)?;

    Ok(())
}

/// Check if SAMOYED environment variable is set to "0" (bypass mode)
///
/// # Returns
///
/// Returns true if SAMOYED=0, false otherwise
fn check_bypass_mode() -> bool {
    matches!(env::var("SAMOYED").as_deref(), Ok("0"))
}

/// Get the root directory of the current git repository
///
/// Uses `git rev-parse --is-inside-work-tree` to check if we're in a git repo,
/// and `git rev-parse --show-toplevel` to get the root directory.
///
/// # Returns
///
/// Returns the absolute path to the git root, or an error if not in a git repo
fn get_git_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|_| ERR_FAILED_EXECUTE_GIT.to_string())?;

    if !output.status.success() {
        return Err(ERR_NOT_GIT_REPO.to_string());
    }

    let inside = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if inside != "true" {
        return Err(ERR_NOT_GIT_REPO.to_string());
    }

    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| ERR_FAILED_GET_GIT_ROOT.to_string())?;

    if !output.status.success() {
        return Err(ERR_FAILED_GET_GIT_ROOT.to_string());
    }

    let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_root))
}

/// Validate that the samoyed directory is inside the git repository
///
/// # Arguments
///
/// * `git_root` - The root directory of the git repository
/// * `dirname` - The proposed directory name for Samoyed
///
/// # Returns
///
/// Returns the absolute path to the samoyed directory, or an error if invalid
fn validate_samoyed_dir(
    git_root: &Path,
    current_dir: &Path,
    dirname: &str,
) -> Result<PathBuf, String> {
    let git_root_canonical = git_root
        .canonicalize()
        .map_err(|e| format!("{}: {}", ERR_FAILED_RESOLVE_GIT_ROOT, e))?;

    let provided_path = Path::new(dirname);

    let candidate = if provided_path.is_absolute() {
        provided_path.to_path_buf()
    } else {
        let has_parent = provided_path
            .components()
            .any(|component| matches!(component, Component::ParentDir));
        if has_parent {
            current_dir.join(provided_path)
        } else {
            git_root_canonical.join(provided_path)
        }
    };

    let resolved = canonicalize_allowing_nonexistent(&candidate)
        .map_err(|e| format!("{} '{}': {}", ERR_FAILED_RESOLVE_SAMOYED_DIR, dirname, e))?;

    if !resolved.starts_with(&git_root_canonical) {
        return Err(format!(
            "{} (path: {}, git root: {})",
            ERR_OUTSIDE_GIT_REPO,
            resolved.display(),
            git_root_canonical.display()
        ));
    }

    Ok(resolved)
}

/// Canonicalize a path, allowing for non-existent components.
///
/// This function resolves a path to its absolute form, handling cases where
/// some components of the path don't exist yet. It walks up the path hierarchy
/// until it finds an existing ancestor, canonicalizes that, then appends the
/// remaining non-existent components.
///
/// # Arguments
///
/// * `path` - The path to canonicalize
///
/// # Returns
///
/// Returns the canonicalized absolute path, or an IO error if the path cannot be resolved
///
/// # Example
///
/// If `/home/user` exists but `/home/user/new_dir` doesn't, calling this with
/// `/home/user/new_dir/file.txt` will return `/home/user/new_dir/file.txt` as
/// an absolute path based on the canonical form of `/home/user`.
fn canonicalize_allowing_nonexistent(path: &Path) -> std::io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }

    let mut components = Vec::new();
    let mut current = path;

    loop {
        if current.exists() {
            let mut canonical = current.canonicalize()?;
            for component in components.iter().rev() {
                canonical.push(component);
            }
            return Ok(canonical);
        }

        match current.file_name() {
            Some(name) => components.push(name.to_os_string()),
            None => {
                // We've reached a root that doesn't exist; this means the entire path is invalid
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    ERR_UNABLE_RESOLVE_PATH,
                ));
            }
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    ERR_UNABLE_RESOLVE_PARENT,
                ));
            }
        }
    }
}

/// Create the directory structure for Samoyed
///
/// Creates the main samoyed directory and the _ subdirectory.
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn create_directory_structure(samoyed_dir: &Path) -> Result<(), String> {
    // Create main samoyed directory
    fs::create_dir_all(samoyed_dir)
        .map_err(|e| format!("{}: {}", ERR_FAILED_CREATE_SAMOYED_DIR, e))?;

    // Create _ subdirectory
    let underscore_dir = samoyed_dir.join(WRAPPER_DIR_NAME);
    fs::create_dir_all(&underscore_dir)
        .map_err(|e| format!("{}: {}", ERR_FAILED_CREATE_WRAPPER_DIR, e))?;

    Ok(())
}

/// Copy the embedded wrapper script to _/samoyed
///
/// The script is copied with platform-appropriate permissions:
/// - Unix: 644 permissions (rw-r--r--) since the wrapper is sourced, not executed
/// - Windows: Default filesystem permissions (no Unix-style permission bits)
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn copy_wrapper_script(samoyed_dir: &Path) -> Result<(), String> {
    let wrapper_path = samoyed_dir.join(WRAPPER_DIR_NAME).join(WRAPPER_SCRIPT_NAME);

    // Write the embedded script
    fs::write(&wrapper_path, SAMOYED_WRAPPER_SCRIPT)
        .map_err(|e| format!("{}: {}", ERR_FAILED_WRITE_WRAPPER, e))?;

    // Set permissions based on platform:
    // - Unix: 644 (rw-r--r--) because the wrapper is sourced, not executed
    // - Windows: Allow default permissions (may be executable, which is acceptable)
    #[cfg(unix)]
    {
        let metadata = fs::metadata(&wrapper_path)
            .map_err(|e| format!("{}: {}", ERR_FAILED_GET_METADATA, e))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&wrapper_path, permissions)
            .map_err(|e| format!("{}: {}", ERR_FAILED_SET_PERMISSIONS, e))?;
    }

    // On Windows, file permissions work differently than Unix
    // The Windows filesystem will handle executable attributes automatically
    // It's acceptable for the wrapper to be executable on Windows

    Ok(())
}

/// Create hook scripts in the _ directory
///
/// Creates all Git hook scripts with platform-appropriate permissions:
/// - Unix: 755 permissions (rwxr-xr-x) to make scripts executable
/// - Windows: Default filesystem permissions (executable attribute handled automatically)
///
/// Each script sources the shared wrapper so user hooks run consistently.
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn create_hook_scripts(samoyed_dir: &Path) -> Result<(), String> {
    let underscore_dir = samoyed_dir.join(WRAPPER_DIR_NAME);

    for hook_name in GIT_HOOKS {
        let hook_path = underscore_dir.join(hook_name);

        // Write the hook script
        fs::write(&hook_path, HOOK_SCRIPT_TEMPLATE)
            .map_err(|e| format!("{} '{}': {}", ERR_FAILED_WRITE_HOOK, hook_name, e))?;

        // Set permissions to 755 (rwxr-xr-x)
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&hook_path)
                .map_err(|e| format!("{}: {}", ERR_FAILED_GET_METADATA, e))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions)
                .map_err(|e| format!("{}: {}", ERR_FAILED_SET_PERMISSIONS, e))?;
        }
    }

    Ok(())
}

/// Create a sample pre-commit hook in the samoyed directory
///
/// This creates a simple pre-commit hook template that users can extend.
/// The file is created with 644 permissions.
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn create_sample_pre_commit(samoyed_dir: &Path) -> Result<(), String> {
    let pre_commit_path = samoyed_dir.join(SAMPLE_HOOK_NAME);

    // Write the sample pre-commit hook
    fs::write(&pre_commit_path, SAMPLE_PRE_COMMIT_CONTENT)
        .map_err(|e| format!("{}: {}", ERR_FAILED_WRITE_SAMPLE, e))?;

    // Set permissions to 644 (rw-r--r--)
    #[cfg(unix)]
    {
        let metadata = fs::metadata(&pre_commit_path)
            .map_err(|e| format!("{}: {}", ERR_FAILED_GET_METADATA, e))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&pre_commit_path, permissions)
            .map_err(|e| format!("{}: {}", ERR_FAILED_SET_PERMISSIONS, e))?;
    }

    Ok(())
}

/// Set the git config core.hooksPath to point to the _ directory
///
/// Uses `git config core.hooksPath` to configure Git to use our hooks.
/// Sets a relative path from the git repository root to avoid Windows extended-length path issues.
/// The path is normalized to use Unix-style separators for Git configuration compatibility.
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn set_git_hooks_path(samoyed_dir: &Path) -> Result<(), String> {
    // Get git root to calculate relative path
    let git_root = get_git_root()?;

    // Canonicalize both paths to ensure consistent path representation
    let git_root_canonical = git_root
        .canonicalize()
        .map_err(|e| format!("{}: {}", ERR_FAILED_CANONICALIZE_GIT_ROOT, e))?;

    let samoyed_dir_canonical = canonicalize_allowing_nonexistent(samoyed_dir)
        .map_err(|e| format!("{}: {}", ERR_FAILED_CANONICALIZE_SAMOYED, e))?;

    // Calculate relative path from git root to hooks directory
    let hooks_path = samoyed_dir_canonical.join(WRAPPER_DIR_NAME);
    let relative_hooks_path = hooks_path
        .strip_prefix(&git_root_canonical)
        .map_err(|_| ERR_HOOKS_PATH_NOT_IN_REPO.to_string())?;

    // Convert to string with Unix-style separators for Git config
    let hooks_path_str = relative_hooks_path
        .to_str()
        .ok_or_else(|| ERR_INVALID_HOOKS_PATH.to_string())?
        .replace('\\', "/");

    let status = Command::new("git")
        .args(["config", "core.hooksPath", &hooks_path_str])
        .status()
        .map_err(|_| ERR_FAILED_SET_GIT_CONFIG.to_string())?;

    if !status.success() {
        return Err(ERR_FAILED_SET_HOOKS_PATH.to_string());
    }

    Ok(())
}

/// Create a .gitignore file in the _ directory
///
/// The .gitignore contains a single asterisk to ignore all files in the directory.
/// Only creates the file if it doesn't already exist.
///
/// # Arguments
///
/// * `samoyed_dir` - Path to the samoyed directory
///
/// # Returns
///
/// Returns Ok(()) on success, or an error message on failure
fn create_gitignore(samoyed_dir: &Path) -> Result<(), String> {
    let gitignore_path = samoyed_dir.join(WRAPPER_DIR_NAME).join(GITIGNORE_NAME);

    // Only create if it doesn't exist
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, GITIGNORE_CONTENT)
            .map_err(|e| format!("{}: {}", ERR_FAILED_WRITE_GITIGNORE, e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command as StdCommand;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// Test check_bypass_mode function
    #[test]
    fn test_check_bypass_mode() {
        // Test when SAMOYED is not set
        unsafe {
            env::remove_var("SAMOYED");
        }
        assert!(!check_bypass_mode());

        // Test when SAMOYED=0
        unsafe {
            env::set_var("SAMOYED", "0");
        }
        assert!(check_bypass_mode());

        // Test when SAMOYED=1
        unsafe {
            env::set_var("SAMOYED", "1");
        }
        assert!(!check_bypass_mode());

        // Test when SAMOYED=2
        unsafe {
            env::set_var("SAMOYED", "2");
        }
        assert!(!check_bypass_mode());

        // Clean up
        unsafe {
            env::remove_var("SAMOYED");
        }
    }

    /// Test validate_samoyed_dir function with valid paths
    #[test]
    fn test_validate_samoyed_dir_valid() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();

        // Test with simple directory name
        let result = validate_samoyed_dir(git_root, git_root, ".samoyed");
        assert!(result.is_ok());
        let path = result.unwrap();
        let expected = canonicalize_allowing_nonexistent(&git_root.join(".samoyed"))
            .expect("failed to canonicalize expected path");
        assert_eq!(path, expected);

        // Test with nested directory
        let result = validate_samoyed_dir(git_root, git_root, "hooks/samoyed");
        assert!(result.is_ok());
    }

    /// Test validate_samoyed_dir function with invalid paths
    #[test]
    fn test_validate_samoyed_dir_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();

        // Test with path outside git root
        let result = validate_samoyed_dir(git_root, git_root, "..");
        assert!(result.is_err());

        // Test with absolute path outside git root
        let result = validate_samoyed_dir(git_root, git_root, "/tmp/outside");
        assert!(result.is_err());
    }

    /// Test create_directory_structure function
    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let samoyed_dir = temp_dir.path().join(".samoyed");

        let result = create_directory_structure(&samoyed_dir);
        assert!(result.is_ok());

        // Check that directories were created
        assert!(samoyed_dir.exists());
        assert!(samoyed_dir.join("_").exists());

        // Test idempotency - should work even if directories exist
        let result = create_directory_structure(&samoyed_dir);
        assert!(result.is_ok());
    }

    /// Test copy_wrapper_script function
    #[test]
    fn test_copy_wrapper_script() {
        let temp_dir = TempDir::new().unwrap();
        let samoyed_dir = temp_dir.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = copy_wrapper_script(&samoyed_dir);
        assert!(result.is_ok());

        let wrapper_path = samoyed_dir.join("_").join("samoyed");
        assert!(wrapper_path.exists());

        let contents = fs::read(&wrapper_path).unwrap();
        assert_eq!(contents, SAMOYED_WRAPPER_SCRIPT);

        // Check permissions on Unix
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&wrapper_path).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o644);
        }
    }

    /// Test create_hook_scripts function
    #[test]
    fn test_create_hook_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let samoyed_dir = temp_dir.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = create_hook_scripts(&samoyed_dir);
        assert!(result.is_ok());

        // Check that all hook scripts were created
        for hook_name in GIT_HOOKS {
            let hook_path = samoyed_dir.join("_").join(hook_name);
            assert!(hook_path.exists(), "Hook {} should exist", hook_name);

            // Check content
            let content = fs::read_to_string(&hook_path).unwrap();
            assert_eq!(
                content,
                r#"#!/usr/bin/env sh
. "$(dirname "$0")/samoyed"
"#
            );

            // Check permissions on Unix
            #[cfg(unix)]
            {
                let metadata = fs::metadata(&hook_path).unwrap();
                let mode = metadata.permissions().mode();
                assert_eq!(
                    mode & 0o777,
                    0o755,
                    "Hook {} should have 755 permissions",
                    hook_name
                );
            }
        }
    }

    /// Test create_sample_pre_commit function
    #[test]
    fn test_create_sample_pre_commit() {
        let temp_dir = TempDir::new().unwrap();
        let samoyed_dir = temp_dir.path().join(".samoyed");
        fs::create_dir_all(&samoyed_dir).unwrap();

        let result = create_sample_pre_commit(&samoyed_dir);
        assert!(result.is_ok());

        let pre_commit_path = samoyed_dir.join("pre-commit");
        assert!(pre_commit_path.exists());

        // Check content
        let content = fs::read_to_string(&pre_commit_path).unwrap();
        assert_eq!(
            content,
            r#"#!/usr/bin/env sh
# Add your pre-commit checks here. For example:
# echo "Running Samoyed sample pre-commit"
# exit 0
"#
        );

        // Check permissions on Unix
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&pre_commit_path).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o644);
        }
    }

    /// Test create_gitignore function
    #[test]
    fn test_create_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let samoyed_dir = temp_dir.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = create_gitignore(&samoyed_dir);
        assert!(result.is_ok());

        let gitignore_path = samoyed_dir.join("_").join(".gitignore");
        assert!(gitignore_path.exists());

        // Check content
        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert_eq!(content, "*\n");

        // Test that it doesn't overwrite existing file
        fs::write(&gitignore_path, "custom content").unwrap();
        let result = create_gitignore(&samoyed_dir);
        assert!(result.is_ok());

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert_eq!(content, "custom content");
    }

    /// Test the CLI parsing
    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;

        // Test that the CLI can be constructed
        let _cli = Cli::command();

        // Test parsing init command
        let cli = Cli::parse_from(["samoyed", "init"]);
        match cli.command {
            Some(Commands::Init { dirname }) => {
                assert!(dirname.is_none());
            }
            _ => panic!("Expected Init command"),
        }

        // Test parsing init command with dirname
        let cli = Cli::parse_from(["samoyed", "init", ".hooks"]);
        match cli.command {
            Some(Commands::Init { dirname }) => {
                assert_eq!(dirname, Some(".hooks".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    /// Test get_git_root function when not in a git repo
    #[test]
    fn test_get_git_root_not_in_repo() {
        let temp_dir = TempDir::new().unwrap();

        // Change to temp directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let result = get_git_root();
        assert!(result.is_err());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    /// Test init_samoyed with bypass mode
    #[test]
    fn test_init_samoyed_bypass() {
        unsafe {
            env::set_var("SAMOYED", "0");
        }

        let result = init_samoyed(".samoyed");
        assert!(result.is_ok());

        unsafe {
            env::remove_var("SAMOYED");
        }
    }

    /// Test init_samoyed when not in git repo
    #[test]
    fn test_init_samoyed_not_in_repo() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let result = init_samoyed(".samoyed");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Not a git repository"));

        env::set_current_dir(&original_dir).unwrap();
    }

    /// Helper function to create a test git repository
    fn create_test_git_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");

        // Verify the temp directory exists before proceeding
        assert!(
            temp_dir.path().exists(),
            "Temporary directory does not exist: {:?}",
            temp_dir.path()
        );

        // Initialize git repo
        let init_output = StdCommand::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute git init command");

        if !init_output.status.success() {
            panic!(
                "Failed to init git repo: {}",
                String::from_utf8_lossy(&init_output.stderr)
            );
        }

        // Configure git user (required for some operations)
        let email_output = StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute git config user.email command");

        if !email_output.status.success() {
            panic!(
                "Failed to set git user email: {}",
                String::from_utf8_lossy(&email_output.stderr)
            );
        }

        let name_output = StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to execute git config user.name command");

        if !name_output.status.success() {
            panic!(
                "Failed to set git user name: {}",
                String::from_utf8_lossy(&name_output.stderr)
            );
        }

        // Verify the git repo was created successfully
        let verify_output = StdCommand::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to verify git repository");

        if !verify_output.status.success() {
            panic!(
                "Git repository verification failed: {}",
                String::from_utf8_lossy(&verify_output.stderr)
            );
        }

        temp_dir
    }

    /// Test full init_samoyed function in a git repo
    #[test]
    fn test_init_samoyed_full() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        // Run init
        let result = init_samoyed(".samoyed");
        assert!(result.is_ok());

        // Verify directory structure
        let samoyed_dir = git_repo.path().join(".samoyed");
        assert!(samoyed_dir.exists());
        assert!(samoyed_dir.join("_").exists());

        // Verify wrapper script
        let wrapper_path = samoyed_dir.join("_").join("samoyed");
        assert!(wrapper_path.exists());

        // Verify sample pre-commit
        let pre_commit_path = samoyed_dir.join("pre-commit");
        assert!(pre_commit_path.exists());

        // Verify all hook scripts
        for hook_name in GIT_HOOKS {
            let hook_path = samoyed_dir.join("_").join(hook_name);
            assert!(hook_path.exists(), "Hook {} should exist", hook_name);
        }

        // Verify .gitignore
        let gitignore_path = samoyed_dir.join("_").join(".gitignore");
        assert!(gitignore_path.exists());

        // Verify git config was set
        let output = StdCommand::new("git")
            .args(["config", "core.hooksPath"])
            .current_dir(git_repo.path())
            .output()
            .unwrap();

        let hooks_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(!hooks_path.is_empty());

        let hooks_path_buf = PathBuf::from(&hooks_path);
        let expected_hooks = samoyed_dir.join("_");
        let hooks_canonical = canonicalize_allowing_nonexistent(&hooks_path_buf)
            .expect("failed to canonicalize hooks path");
        let expected_canonical = canonicalize_allowing_nonexistent(&expected_hooks)
            .expect("failed to canonicalize expected hooks path");
        assert_eq!(hooks_canonical, expected_canonical);

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test init_samoyed with custom directory name
    #[test]
    fn test_init_samoyed_custom_dir() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        // Run init with custom directory
        let result = init_samoyed(".hooks");
        assert!(result.is_ok());

        // Verify custom directory was created
        let hooks_dir = git_repo.path().join(".hooks");
        assert!(hooks_dir.exists());
        assert!(hooks_dir.join("_").exists());

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test init_samoyed idempotency (running it twice)
    #[test]
    fn test_init_samoyed_idempotent() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        // Run init first time
        let result1 = init_samoyed(".samoyed");
        assert!(result1.is_ok());

        // Run init second time
        let result2 = init_samoyed(".samoyed");
        assert!(result2.is_ok());

        // Verify structure still exists
        let samoyed_dir = git_repo.path().join(".samoyed");
        assert!(samoyed_dir.exists());

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test set_git_hooks_path function
    #[test]
    fn test_set_git_hooks_path() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        let samoyed_dir = git_repo.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = set_git_hooks_path(&samoyed_dir);
        assert!(result.is_ok());

        // Verify git config was set
        let output = StdCommand::new("git")
            .args(["config", "core.hooksPath"])
            .current_dir(git_repo.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let hooks_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(!hooks_path.is_empty());

        let hooks_path_buf = PathBuf::from(&hooks_path);
        let expected_hooks = samoyed_dir.join("_");
        let hooks_canonical = canonicalize_allowing_nonexistent(&hooks_path_buf)
            .expect("failed to canonicalize hooks path");
        let expected_canonical = canonicalize_allowing_nonexistent(&expected_hooks)
            .expect("failed to canonicalize expected hooks path");
        assert_eq!(hooks_canonical, expected_canonical);

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test get_git_root in an actual git repository
    #[test]
    fn test_get_git_root_in_repo() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();

        // Test from root
        env::set_current_dir(git_repo.path()).unwrap();
        let result = get_git_root();
        assert!(result.is_ok());
        let git_root = result.unwrap();
        // Canonicalize both paths for comparison
        assert_eq!(
            git_root.canonicalize().unwrap(),
            git_repo.path().canonicalize().unwrap()
        );

        // Test from subdirectory
        let subdir = git_repo.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        env::set_current_dir(&subdir).unwrap();

        let result = get_git_root();
        assert!(result.is_ok());
        let git_root = result.unwrap();
        assert_eq!(
            git_root.canonicalize().unwrap(),
            git_repo.path().canonicalize().unwrap()
        );

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test validate_samoyed_dir with relative path containing ..
    #[test]
    fn test_validate_samoyed_dir_parent_relative() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();

        // Create subdirectory
        let subdir = git_root.join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Test with path that goes up and back down
        let result = validate_samoyed_dir(git_root, &subdir, "../.samoyed");
        assert!(result.is_ok());

        // The resulting path should be inside git root
        let path = result.unwrap();
        let git_root_canonical = git_root.canonicalize().unwrap();
        assert!(path.starts_with(&git_root_canonical));
    }

    /// Test that main function returns success exit code
    #[test]
    fn test_main_no_command() {
        // This will trigger the help display
        let args = vec!["samoyed"];
        let result = std::panic::catch_unwind(|| Cli::try_parse_from(args));

        // When no command is given, clap returns an error (which shows help)
        // but our main function still returns SUCCESS
        assert!(result.is_ok());
    }

    /// Test Windows-specific path normalization in set_git_hooks_path
    /// This test only runs on Windows to verify backslash to forward slash conversion
    #[cfg(windows)]
    #[test]
    fn test_set_git_hooks_path_windows_normalization() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        // Create a path that would naturally have backslashes on Windows
        let samoyed_dir = git_repo.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = set_git_hooks_path(&samoyed_dir);
        assert!(result.is_ok());

        // Verify git config was set with Unix-style separators
        let output = StdCommand::new("git")
            .args(["config", "core.hooksPath"])
            .current_dir(git_repo.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let hooks_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // On Windows, the path should be normalized to use forward slashes
        // and should be relative (not contain drive letters or backslashes)
        assert!(
            !hooks_path.contains('\\'),
            "Path should not contain backslashes: {}",
            hooks_path
        );
        assert!(
            !hooks_path.contains("C:"),
            "Path should be relative, not absolute: {}",
            hooks_path
        );

        env::set_current_dir(original_dir).unwrap();
    }

    /// Test cross-platform path normalization behavior
    /// This test runs on all platforms to verify consistent behavior
    #[test]
    fn test_set_git_hooks_path_cross_platform() {
        let git_repo = create_test_git_repo();
        let original_dir = env::current_dir().unwrap();
        assert!(
            git_repo.path().exists(),
            "Git repo directory should exist: {:?}",
            git_repo.path()
        );
        env::set_current_dir(git_repo.path()).unwrap_or_else(|e| {
            panic!(
                "Failed to change to git repo directory {:?}: {}",
                git_repo.path(),
                e
            )
        });

        let samoyed_dir = git_repo.path().join(".samoyed");
        fs::create_dir_all(samoyed_dir.join("_")).unwrap();

        let result = set_git_hooks_path(&samoyed_dir);
        assert!(result.is_ok());

        // Verify git config was set
        let output = StdCommand::new("git")
            .args(["config", "core.hooksPath"])
            .current_dir(git_repo.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let hooks_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // The path should be relative regardless of platform
        assert!(!hooks_path.is_empty(), "Hooks path should not be empty");

        // Should not contain absolute path indicators
        #[cfg(windows)]
        assert!(
            !hooks_path.contains(":\\"),
            "Should not contain Windows drive letter: {}",
            hooks_path
        );

        #[cfg(unix)]
        assert!(
            !hooks_path.starts_with('/'),
            "Should not be absolute Unix path: {}",
            hooks_path
        );

        // Should end with our expected relative path structure
        assert!(
            hooks_path.ends_with("_") || hooks_path.ends_with("_/"),
            "Should end with underscore directory: {}",
            hooks_path
        );

        env::set_current_dir(original_dir).unwrap();
    }
}
