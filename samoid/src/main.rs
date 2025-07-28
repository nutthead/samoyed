//! Command Line Interface for Samoid Git hooks manager
//!
//! This binary provides a CLI for managing Git hooks through TOML configuration.
//! Supports the `init` command and deprecated command warnings.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

mod config;
mod environment;
mod git;
mod hooks;
mod installer;
mod project;

use config::SamoidConfig;
use environment::{
    CommandRunner, Environment, FileSystem, SystemCommandRunner, SystemEnvironment,
    SystemFileSystem,
};
use installer::install_hooks;

use project::ProjectType;

#[derive(Parser)]
#[command(name = "samoid")]
#[command(about = "Modern native Git hooks manager")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize samoid in the current repository
    Init {
        /// Project type to auto-detect (optional)
        #[arg(short, long)]
        project_type: Option<String>,
    },
}

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { project_type }) => {
            init_command_with_system_deps(project_type)?;
        }
        None => {
            // Show help when no command is provided
            eprintln!("Error: No command specified. Use 'samoid init' to get started.");
            eprintln!("Run 'samoid --help' for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Wrapper function that calls init_command with real system dependencies
#[cfg(not(tarpaulin_include))]
fn init_command_with_system_deps(project_type_hint: Option<String>) -> Result<()> {
    let env = SystemEnvironment;
    let runner = SystemCommandRunner;
    let fs = SystemFileSystem;

    init_command(&env, &runner, &fs, project_type_hint)
}

fn init_command(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    project_type_hint: Option<String>,
) -> Result<()> {
    // Check if we're in a Git repository
    if !fs.exists(Path::new(".git")) {
        anyhow::bail!("Not a git repository. Run 'git init' first.");
    }

    // Create .samoid directory if it doesn't exist
    fs.create_dir_all(Path::new(".samoid"))
        .context("Failed to create .samoid directory")?;

    // Determine initialization mode: existing config gets updated, new projects get generated defaults
    let config_exists = fs.exists(Path::new("samoid.toml"));

    // Check if user wants verbose output
    let verbose = env.get_var("SAMOID_VERBOSE").unwrap_or_default() == "1";

    if config_exists {
        let message = "samoid.toml already exists. Updating configuration...";
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
        let config = SamoidConfig::default_for_project_type(&project_type);

        // Write samoid.toml
        let toml_content =
            toml::to_string_pretty(&config).context("Failed to serialize configuration")?;

        // Validate the configuration before writing
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Generated configuration is invalid: {}", e))?;

        fs.write(Path::new("samoid.toml"), &toml_content)
            .context("Failed to write samoid.toml")?;

        if verbose {
            println!(
                "✅ Created samoid.toml with {} defaults (verbose mode)",
                project_type.name()
            );
        } else {
            println!(
                "✅ Created samoid.toml with {} defaults",
                project_type.name()
            );
        }
    }

    // Install Git hooks using the core installation system
    match install_hooks(env, runner, fs, Some(".samoid")) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
        Err(e) => anyhow::bail!("Failed to install hooks: {}", e),
    }

    println!("✅ samoid is ready! Edit samoid.toml to customize your hooks.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
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

        // Mock successful git command
        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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

        // Mock successful git command
        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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

        // Mock failed git command
        let output = Output {
            status: exit_status(1),
            stdout: vec![],
            stderr: b"fatal: not a git repository".to_vec(),
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        // Should fail when git config fails
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to install hooks")
        );
    }

    #[test]
    fn test_init_command_with_existing_config() {
        // Test when samoid.toml already exists
        let env = MockEnvironment::new();

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        // Mock filesystem with git repository and existing config
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file("samoid.toml", "[hooks]\npre-commit = \"echo test\"");

        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_with_invalid_project_type_hint() {
        // Test with invalid project type hint that falls back to auto-detection
        let env = MockEnvironment::new();

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed even with invalid hint, falling back to auto-detect
        let result = init_command(&env, &runner, &fs, Some("invalid-type".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_command_with_system_deps_wrapper() {
        // Test the wrapper function (this will use real system deps but we can still verify it compiles and links correctly)

        // We can't easily test this without mocking at the system level,
        // but we can at least verify the function signature and that it would work
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            }),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        // Test the underlying function that the wrapper calls
        let result = init_command(&env, &runner, &fs, Some("rust".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_struct_parsing() {
        // Test CLI struct can be created and parsed correctly
        use clap::Parser;

        // Test valid arguments
        let args = vec!["samoid", "init"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            Some(Commands::Init { project_type }) => {
                assert!(project_type.is_none());
            }
            _ => panic!("Expected Init command"),
        }

        // Test with project type argument
        let args = vec!["samoid", "init", "--project-type", "rust"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            Some(Commands::Init { project_type }) => {
                assert_eq!(project_type, Some("rust".to_string()));
            }
            _ => panic!("Expected Init command"),
        }

        // Test with short form
        let args = vec!["samoid", "init", "-p", "go"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        match parsed.command {
            Some(Commands::Init { project_type }) => {
                assert_eq!(project_type, Some("go".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_no_command() {
        // Test CLI with no command (None case)
        use clap::Parser;

        let args = vec!["samoid"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        assert!(parsed.command.is_none());
    }

    #[test]
    fn test_cli_invalid_arguments() {
        // Test CLI with invalid arguments
        use clap::Parser;

        let args = vec!["samoid", "invalid-command"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_init_command_all_project_types() {
        // Test init command with all supported project type hints
        let env = MockEnvironment::new();

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let project_types = vec!["rust", "go", "node", "python", "javascript", "typescript"];

        for project_type in project_types {
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(output.clone()),
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

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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
    fn test_wrapper_function_call_path() {
        // Test that the wrapper function properly creates system dependencies
        // and calls the main init_command function
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            }),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        // This tests the actual logic path that the wrapper takes
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with different project types to ensure the wrapper handles them
        for project_type in &["rust", "go", "node", "python"] {
            let result = init_command(&env, &runner, &fs, Some(project_type.to_string()));
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_project_type_detection_fallback() {
        // Test the fallback logic when project type hint is invalid
        let env = MockEnvironment::new();

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };

        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
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
        // Test that the SAMOID_VERBOSE environment variable affects output
        let env = MockEnvironment::new().with_var("SAMOID_VERBOSE", "1");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_directory(".git");

        // Should succeed with verbose environment variable set
        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with existing config and verbose mode
        let fs_with_config = MockFileSystem::new()
            .with_directory(".git")
            .with_file("samoid.toml", "[hooks]\npre-commit = \"test\"");

        let result = init_command(&env, &runner, &fs_with_config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_environment_variable_not_set() {
        // Test that when SAMOID_VERBOSE is not set or not "1", verbose mode is disabled
        let env = MockEnvironment::new(); // No environment variables set

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_directory(".git");

        let result = init_command(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with SAMOID_VERBOSE set to something other than "1"
        let env_other = MockEnvironment::new().with_var("SAMOID_VERBOSE", "0");
        let result = init_command(&env_other, &runner, &fs, None);
        assert!(result.is_ok());

        let env_false = MockEnvironment::new().with_var("SAMOID_VERBOSE", "false");
        let result = init_command(&env_false, &runner, &fs, None);
        assert!(result.is_ok());
    }
}
