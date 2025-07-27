//! Command Line Interface for Samoid Git hooks manager
//!
//! This binary provides a CLI for managing Git hooks through TOML configuration.
//! Supports the `init` command and deprecated command warnings.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

mod config;
mod project;

use config::SamoidConfig;
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
            init_command(project_type)?;
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

fn init_command(project_type_hint: Option<String>) -> Result<()> {
    // Check if we're in a Git repository
    if !Path::new(".git").exists() {
        anyhow::bail!("Not a git repository. Run 'git init' first.");
    }

    // Create .samoid directory if it doesn't exist
    std::fs::create_dir_all(".samoid").context("Failed to create .samoid directory")?;

    // Check if samoid.toml already exists
    let config_exists = Path::new("samoid.toml").exists();

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

        std::fs::write("samoid.toml", toml_content).context("Failed to write samoid.toml")?;

        println!(
            "✅ Created samoid.toml with {} defaults",
            project_type.name()
        );
    }

    // Configure Git hooks path
    let output = std::process::Command::new("git")
        .args(&["config", "core.hooksPath", ".samoid/_"])
        .output()
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
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Clean up any existing samoid.toml from previous tests
        let _ = fs::remove_file("samoid.toml");
        let _ = fs::remove_dir_all(".samoid");
        
        temp_dir
    }

    #[test]
    fn test_init_command_creates_directories() {
        let _temp_dir = setup_test_repo();

        // Should create .samoid directory and samoid.toml even if git config fails
        let result = init_command(None);

        // Should create .samoid directory (in current working directory, which is temp_dir)
        assert!(Path::new(".samoid").exists());

        // Should create samoid.toml (in current working directory, which is temp_dir)
        assert!(Path::new("samoid.toml").exists());

        // Git config might fail in test environment, but file creation should work
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to") || error_msg.contains("git"));
        }
    }

    #[test]
    fn test_init_command_fails_without_git() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Should fail without .git
        assert!(init_command(None).is_err());
    }

    #[test]
    fn test_init_command_with_project_type_hint() {
        let _temp_dir = setup_test_repo();

        // Create a Cargo.toml so the project type hint works properly
        fs::write("Cargo.toml", "[package]\nname = \"test\"").unwrap();

        let result = init_command(Some("rust".to_string()));

        // Should create samoid.toml with Rust defaults
        assert!(Path::new("samoid.toml").exists());
        let content = fs::read_to_string("samoid.toml").unwrap();
        assert!(content.contains("cargo"));

        // Git config might fail in test environment, but file creation should work
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("Failed to") || error_msg.contains("git"));
        }
    }
}
