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
/// * `force` - Optional force regeneration parameter (e.g., "_" to force hook regeneration)
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
    _force: Option<String>,
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
#[path = "unit_tests/init_tests.rs"]
mod tests;
