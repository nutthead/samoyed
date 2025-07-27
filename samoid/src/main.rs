//! Command Line Interface for Samoid Git hooks manager
//!
//! This binary provides a CLI for managing Git hooks through TOML configuration.
//! Supports the `init` command and deprecated command warnings.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

mod config;
mod environment;
mod project;

use config::SamoidConfig;
use environment::{CommandRunner, Environment, FileSystem, SystemCommandRunner, SystemEnvironment, SystemFileSystem};
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
fn init_command_with_system_deps(project_type_hint: Option<String>) -> Result<()> {
    let env = SystemEnvironment;
    let runner = SystemCommandRunner;
    let fs = SystemFileSystem;
    
    init_command(&env, &runner, &fs, project_type_hint)
}

fn init_command(
    _env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    project_type_hint: Option<String>
) -> Result<()> {
    // Check if we're in a Git repository
    if !fs.exists(Path::new(".git")) {
        anyhow::bail!("Not a git repository. Run 'git init' first.");
    }

    // Create .samoid directory if it doesn't exist
    fs.create_dir_all(Path::new(".samoid")).context("Failed to create .samoid directory")?;

    // Check if samoid.toml already exists
    let config_exists = fs.exists(Path::new("samoid.toml"));

    if config_exists {
        println!("samoid.toml already exists. Updating configuration...");
    } else {
        // Detect project type
        let project_type = if let Some(hint) = project_type_hint {
            ProjectType::from_string(&hint).unwrap_or_else(|| {
                println!(
                    "Warning: Unknown project type '{}', auto-detecting...",
                    hint
                );
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

        fs.write(Path::new("samoid.toml"), &toml_content).context("Failed to write samoid.toml")?;

        println!(
            "✅ Created samoid.toml with {} defaults",
            project_type.name()
        );
    }

    // Configure Git hooks path
    let output = runner.run_command("git", &["config", "core.hooksPath", ".samoid/_"])
        .context("Failed to execute git config command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set Git hooks path: {}", stderr);
    }

    println!("✅ Configured Git to use .samoid/_ for hooks");
    println!("✅ samoid is ready! Edit samoid.toml to customize your hooks.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};
    use environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};

    #[test]
    fn test_init_command_creates_directories() {
        // Set up mocks
        let env = MockEnvironment::new();
        
        // Mock successful git command
        let output = Output {
            status: ExitStatus::from_raw(0),
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
        assert!(result.unwrap_err().to_string().contains("Not a git repository"));
    }

    #[test]
    fn test_init_command_with_project_type_hint() {
        // Set up mocks
        let env = MockEnvironment::new();
        
        // Mock successful git command
        let output = Output {
            status: ExitStatus::from_raw(0),
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
            status: ExitStatus::from_raw(1),
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
        assert!(result.unwrap_err().to_string().contains("Failed to set Git hooks path"));
    }
}
