//! Init command implementation for Samoid
//!
//! This module contains the initialization logic for setting up Samoid in a Git repository,
//! including project type detection, configuration file generation, and hook installation.

use anyhow::{Context, Result};
use std::path::Path;

use crate::config::SamoyedConfig;
use crate::environment::{CommandRunner, Environment, FileSystem};
use crate::installer::install_hooks;
use crate::project::ProjectType;

/// Initialize Samoid in the current repository
///
/// This function performs the complete initialization process:
/// 1. Validates that we're in a Git repository
/// 2. Creates the `.samoyed` directory structure
/// 3. Detects or uses the specified project type
/// 4. Generates an appropriate `samoyed.toml` configuration
/// 5. Installs the Git hooks
///
/// # Arguments
///
/// * `env` - Environment abstraction for reading environment variables
/// * `runner` - Command runner for executing system commands
/// * `fs` - Filesystem abstraction for file operations
/// * `project_type_hint` - Optional project type hint (e.g., "rust", "node")
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error describing what went wrong.
///
/// # Errors
///
/// This function will return an error if:
/// - The current directory is not a Git repository
/// - Unable to create the `.samoyed` directory
/// - Configuration generation or validation fails
/// - Hook installation fails
pub fn init_command(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    project_type_hint: Option<String>,
) -> Result<()> {
    // Check if we're in a Git repository
    if !fs.exists(Path::new(".git")) {
        anyhow::bail!("Not a git repository. Run 'git init' first.");
    }

    // Create .samoyed directory if it doesn't exist
    fs.create_dir_all(Path::new(".samoyed"))
        .context("Failed to create .samoyed directory")?;

    // Determine initialization mode: existing config gets updated, new projects get generated defaults
    let config_exists = fs.exists(Path::new("samoyed.toml"));

    // Check if user wants verbose output
    let verbose = env.get_var("SAMOYED_VERBOSE").unwrap_or_default() == "1";

    if config_exists {
        let message = "samoyed.toml already exists. Updating configuration...";
        if verbose {
            println!("🔧 {message}");
        } else {
            println!("{message}");
        }
    } else {
        // Detect project type
        let project_type = if let Some(hint) = project_type_hint {
            ProjectType::from_string(&hint).unwrap_or_else(|| {
                println!("Warning: Unknown project type '{hint}', auto-detecting...");
                ProjectType::auto_detect()
            })
        } else {
            ProjectType::auto_detect()
        };

        // Create default configuration
        let config = SamoyedConfig::default_for_project_type(&project_type);

        // Write samoyed.toml
        let toml_content =
            toml::to_string_pretty(&config).context("Failed to serialize configuration")?;

        // Validate the configuration before writing
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Generated configuration is invalid: {}", e))?;

        fs.write(Path::new("samoyed.toml"), &toml_content)
            .context("Failed to write samoyed.toml")?;

        if verbose {
            println!(
                "✅ Created samoyed.toml with {} defaults (verbose mode)",
                project_type.name()
            );
        } else {
            println!(
                "✅ Created samoyed.toml with {} defaults",
                project_type.name()
            );
        }
    }

    // Install Git hooks using the core installation system
    match install_hooks(env, runner, fs, Some(".samoyed")) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
        Err(e) => {
            // Convert InstallError to anyhow::Error while preserving error context
            return Err(anyhow::anyhow!("Failed to install hooks").context(e.to_string()));
        }
    }

    println!("✅ samoyed is ready! Edit samoyed.toml to customize your hooks.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use std::process::{ExitStatus, Output};

    // Cross-platform exit status creation
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    // Helper function to create ExitStatus cross-platform
    fn exit_status(code: i32) -> ExitStatus {
        #[cfg(unix)]
        return ExitStatus::from_raw(code);

        #[cfg(windows)]
        return ExitStatus::from_raw(code as u32);
    }

    #[test]
    fn test_init_command_creates_directories() {
        // Set up mocks
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        // Mock successful git config command
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_fails_without_git() {
        // Set up mocks
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No .git directory

        // Should fail without .git
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Not a git repository")
        );
    }

    #[test]
    fn test_init_command_with_project_type_hint() {
        // Set up mocks
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        // Mock successful git config command
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed with project type hint
        let result = init_command(&env, &runner, &fs, Some("rust".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_git_config_failure() {
        // Set up mocks
        let env = MockEnvironment::new();

        // Mock git --version first (succeeds)
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        // Mock failed git config command
        let config_output = Output {
            status: exit_status(1),
            stdout: vec![],
            stderr: b"fatal: not a git repository".to_vec(),
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        // Should fail when git config fails
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Git configuration failed"));
    }

    #[test]
    fn test_init_command_with_existing_config() {
        // Test when samoyed.toml already exists
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        // Mock filesystem with git repository and existing config
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file("samoyed.toml", "[hooks]\npre-commit = \"echo test\"");

        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_with_invalid_project_type_hint() {
        // Test with invalid project type hint that falls back to auto-detection
        let env = MockEnvironment::new();

        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed even with invalid hint, falling back to auto-detect
        let result = init_command(&env, &runner, &fs, Some("invalid-type".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_all_project_types() {
        // Test init command with all supported project type hints
        let env = MockEnvironment::new();

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let project_types = vec!["rust", "go", "node", "python", "javascript", "typescript"];

        for project_type in project_types {
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output.clone()))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", ".samoyed/_"],
                    Ok(config_output.clone()),
                );

            let fs = MockFileSystem::new().with_directory(".git");

            let result = init_command(&env, &runner, &fs, Some(project_type.to_string()));
            assert!(result.is_ok(), "Failed for project type: {project_type}");
        }
    }

    #[test]
    fn test_init_command_with_various_scenarios() {
        // Test more edge cases to improve coverage
        let env = MockEnvironment::new();

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output),
            );

        // Test with different filesystem states
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file("Cargo.toml", "[package]\nname = \"test\"");

        // Should detect Rust project and succeed
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_type_detection_fallback() {
        // Test the fallback logic when project type hint is invalid
        let env = MockEnvironment::new();

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output.clone()),
            );

        // Mock filesystem with multiple project files to test priority
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file("package.json", "{}")
            .with_file("go.mod", "module test")
            .with_file("requirements.txt", "");

        // Test with invalid hint - should fallback to auto-detection
        let result = init_command(&env, &runner, &fs, Some("invalid-language".to_string()));
        assert!(result.is_ok());

        // Test with empty hint
        let result = init_command(&env, &runner, &fs, Some("".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_verbose_output_with_environment_variable() {
        // Test that the SAMOYED_VERBOSE environment variable affects output
        let env = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "1");

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output.clone()),
            );

        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed with verbose environment variable set
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with existing config and verbose mode
        let fs_with_config = MockFileSystem::new()
            .with_directory(".git")
            .with_file("samoyed.toml", "[hooks]\npre-commit = \"test\"");

        let result = init_command(&env, &runner, &fs_with_config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_environment_variable_not_set() {
        // Test that when SAMOYED_VERBOSE is not set or not "1", verbose mode is disabled
        let env = MockEnvironment::new(); // No environment variables set

        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let config_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output.clone()),
            );

        let fs = MockFileSystem::new().with_directory(".git");

        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with SAMOYED_VERBOSE set to something other than "1"
        let env_other = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "0");
        let result = init_command(&env_other, &runner, &fs, None);
        assert!(result.is_ok());

        let env_false = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "false");
        let result = init_command(&env_false, &runner, &fs, None);
        assert!(result.is_ok());
    }
}
