//! Command Line Interface for Samoid Git hooks manager
//!
//! This binary provides a CLI for managing Git hooks through TOML configuration.
//! Supports the `init` command and deprecated command warnings.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;

mod config;
mod environment;
mod exit_codes;
mod git;
mod hooks;
mod init;
mod installer;
mod project;

use environment::{SystemCommandRunner, SystemEnvironment, SystemFileSystem};
use exit_codes::{determine_exit_code, EX_USAGE};

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
            if let Err(e) = init_command_with_system_deps(project_type) {
                let exit_code = determine_exit_code(&e);
                eprintln!("Error: {e}");
                process::exit(exit_code);
            }
        }
        None => {
            // Show help when no command is provided
            eprintln!("Error: No command specified. Use 'samoid init' to get started.");
            eprintln!("Run 'samoid --help' for usage information.");
            process::exit(EX_USAGE); // Command line usage error
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

    init::init_command(&env, &runner, &fs, project_type_hint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_struct_parsing() {
        // Test CLI struct can be created and parsed correctly
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
        let args = vec!["samoid"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok());

        let parsed = cli.unwrap();
        assert!(parsed.command.is_none());
    }

    #[test]
    fn test_cli_invalid_arguments() {
        // Test CLI with invalid arguments
        let args = vec!["samoid", "invalid-command"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }
}
