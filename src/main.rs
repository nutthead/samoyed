//! Command Line Interface for Samoyed Git hooks manager
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
use exit_codes::{EX_USAGE, determine_exit_code};

#[derive(Parser)]
#[command(name = "samoyed")]
#[command(about = "Modern native Git hooks manager")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize samoyed in the current repository
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
            eprintln!("Error: No command specified. Use 'samoyed init' to get started.");
            eprintln!("Run 'samoyed --help' for usage information.");
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
#[path = "unit_tests/main_tests.rs"]
mod tests;
