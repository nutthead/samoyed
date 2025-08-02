//! Hook Execution Runtime for Samoyed (`samoyed-hook` binary)
//!
//! This module defines a separate binary (`samoyed-hook`) that serves as the Git hook executor.
//! It is NOT part of the main `samoyed` CLI binary, but rather a companion binary that gets
//! installed into `.samoyed/_/h` and is referenced by all Git hook files.
//!
//! # Binary Architecture
//!
//! Samoyed consists of two binaries:
//! - `samoyed`: The main CLI tool for installation and configuration (defined in `main.rs`)
//! - `samoyed-hook`: This hook runner binary that executes during Git operations
//!
//! When Git triggers a hook (e.g., pre-commit), it executes the hook file in `.samoyed/_/`,
//! which in turn executes this `samoyed-hook` binary with the hook name as an argument.
//!
//! # Environment Variables
//!
//! - **SAMOYED=0**: Skip all hook execution (useful for CI/deployment, rebasing)
//! - **SAMOYED=1**: Normal execution mode (default)
//! - **SAMOYED=2**: Enable debug mode with detailed script tracing
//!
//! # Execution Flow
//!
//! 1. Git triggers hook → 2. Hook file runs `samoyed-hook` → 3. This binary executes user's script
//!
//! The hook runner follows these steps:
//! 1. Parse environment variables (SAMOYED=0/1/2)
//! 2. Load initialization script from `~/.config/samoyed/init.sh` (if exists)
//! 3. Locate and execute the actual hook script from project root
//! 4. Handle errors and propagate exit codes to Git
//!
//! Implements efficient hook execution with comprehensive platform support.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process;

mod environment;
mod logging;

/// Simplified configuration structure for hook runner
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SamoyedConfig {
    /// Hook definitions (required)
    pub hooks: HashMap<String, String>,

    /// Optional settings (with defaults)
    #[serde(default)]
    pub settings: SamoyedSettings,
}

/// Settings structure with defaults
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct SamoyedSettings {
    #[serde(default)]
    pub hook_directory: Option<String>,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub fail_fast: Option<bool>,
    #[serde(default)]
    pub skip_hooks: bool,
}
use environment::{
    CommandRunner, Environment, FileSystem, SystemCommandRunner, SystemEnvironment,
    SystemFileSystem,
};
use logging::{log_command_execution, log_file_operation_with_env, sanitize_args, sanitize_path};

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    let env = SystemEnvironment;
    let runner = SystemCommandRunner;
    let fs = SystemFileSystem;

    let args: Vec<String> = env::args().collect();
    run_hook(&env, &runner, &fs, &args)
}

/// Main hook execution logic with dependency injection support
fn run_hook(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    args: &[String],
) -> Result<()> {
    // Check SAMOYED environment variable for execution mode
    let samoyed_mode = env.get_var("SAMOYED").unwrap_or_else(|| "1".to_string());

    // SAMOYED=0 means skip execution entirely
    if samoyed_mode == "0" {
        process::exit(0);
    }

    // SAMOYED=2 enables debug mode (script tracing)
    let debug_mode = samoyed_mode == "2";

    if debug_mode {
        eprintln!("samoyed: Debug mode enabled (SAMOYED=2)");
        let sanitized_args = sanitize_args(args);
        eprintln!("samoyed: Hook runner args: {sanitized_args:?}");
    }

    // Determine hook name from the first argument (e.g., pre-commit, post-commit)
    let hook_name = if args.len() < 2 {
        anyhow::bail!("No hook name provided in arguments");
    } else {
        Path::new(&args[1])
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
    };

    if debug_mode {
        eprintln!("samoyed: Detected hook name: {hook_name}");
    }

    // First, try to load and execute command from samoyed.toml
    match load_hook_command_from_config(fs, hook_name, debug_mode) {
        Ok(command) => {
            if debug_mode {
                eprintln!("samoyed: Found command in samoyed.toml: {command}");
            }

            // Load initialization script from ~/.config/samoyed/init.sh
            load_init_script(env, runner, fs, debug_mode)?;

            // Execute the command from configuration
            return execute_hook_command(env, runner, &command, &args[2..], debug_mode);
        }
        Err(e) => {
            if debug_mode {
                eprintln!("samoyed: Failed to load command from samoyed.toml: {e}");
            }
        }
    }

    // Fallback: Build the expected hook script path: .samoyed/scripts/{hook_name}
    let hook_script_path = PathBuf::from(".samoyed").join("scripts").join(hook_name);

    if debug_mode {
        log_file_operation_with_env(
            env,
            debug_mode,
            "Looking for hook script at",
            &hook_script_path,
        );
    }

    // Check if the hook script exists - if not, exit silently (this is normal)
    if !fs.exists(&hook_script_path) {
        if debug_mode {
            eprintln!("samoyed: Hook script not found, exiting silently");
        }
        process::exit(0);
    }

    // Load initialization script from ~/.config/samoyed/init.sh
    load_init_script(env, runner, fs, debug_mode)?;

    // Execute the hook script with proper environment
    execute_hook_script(env, runner, fs, &hook_script_path, &args[2..], debug_mode)
}

/// Load hook command from samoyed.toml configuration
fn load_hook_command_from_config(
    fs: &dyn FileSystem,
    hook_name: &str,
    debug_mode: bool,
) -> Result<String> {
    let config_path = Path::new("samoyed.toml");

    if debug_mode {
        eprintln!("samoyed: Checking for samoyed.toml...");
    }

    if !fs.exists(config_path) {
        if debug_mode {
            eprintln!("samoyed: No samoyed.toml found");
        }
        anyhow::bail!("No samoyed.toml configuration file found");
    }

    if debug_mode {
        eprintln!("samoyed: Reading samoyed.toml...");
    }

    let config_content = fs
        .read_to_string(config_path)
        .context("Failed to read samoyed.toml")?;

    if debug_mode {
        eprintln!("samoyed: Parsing samoyed.toml...");
    }

    let config: SamoyedConfig =
        toml::from_str(&config_content).context("Failed to parse samoyed.toml")?;

    if debug_mode {
        eprintln!("samoyed: Successfully parsed config, looking for hook '{hook_name}'");
        eprintln!(
            "samoyed: Available hooks: {:?}",
            config.hooks.keys().collect::<Vec<_>>()
        );
    }

    if let Some(command) = config.hooks.get(hook_name) {
        Ok(command.clone())
    } else {
        if debug_mode {
            eprintln!("samoyed: No command configured for hook '{hook_name}' in samoyed.toml");
        }
        anyhow::bail!("No command configured for hook '{hook_name}'");
    }
}

/// Execute a hook command from configuration
fn execute_hook_command(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    command: &str,
    hook_args: &[String],
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        eprintln!("samoyed: Executing command: {command}");
        let sanitized_hook_args = sanitize_args(hook_args);
        eprintln!("samoyed: Hook arguments: {sanitized_hook_args:?}");
    }

    // Use shell to execute the command
    let shell_command =
        if cfg!(target_os = "windows") && !is_windows_unix_environment(env, debug_mode) {
            "cmd"
        } else {
            "sh"
        };

    let shell_args = if cfg!(target_os = "windows") && !is_windows_unix_environment(env, debug_mode)
    {
        vec!["/C", command]
    } else {
        vec!["-c", command]
    };

    if debug_mode {
        log_command_execution(
            debug_mode,
            shell_command,
            &shell_args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
    }

    let output = runner
        .run_command(shell_command, &shell_args)
        .with_context(|| format!("Failed to execute hook command: {command}"))?;

    // Check exit code and provide appropriate error messages
    let exit_code = output.status.code().unwrap_or(1);

    if debug_mode {
        eprintln!("samoyed: Hook command exit code: {exit_code}");
    }

    // Print stdout and stderr from the hook
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Handle specific error cases
    if exit_code != 0 {
        eprintln!("samoyed - hook command failed (code {exit_code}): {command}");

        // Check for command not found (exit code 127)
        if exit_code == 127 {
            eprintln!("samoyed - command not found in PATH");
            if !debug_mode {
                eprintln!("samoyed - run with SAMOYED=2 for more details");
            }
        }
    }

    // Exit with the same code as the hook command
    process::exit(exit_code);
}

/// Loads and prepares the user's initialization script for hook execution.
///
/// This function locates and validates the optional Samoyed initialization script that users
/// can create to set up their hook environment. The script is expected to be located at
/// `~/.config/samoyed/init.sh` (following XDG Base Directory specification) or
/// `$XDG_CONFIG_HOME/samoyed/init.sh` if the environment variable is set.
///
/// # Purpose
///
/// The initialization script allows users to:
/// - Set environment variables needed by all hooks
/// - Define shell functions used across multiple hooks
/// - Configure PATH or other shell settings
/// - Load project-specific configurations
///
/// # Current Implementation Status
///
/// **Note**: Currently, this function only detects the presence of the init script but does
/// not execute it. Full shell sourcing integration is planned for a future release. This
/// limitation exists because properly sourcing a shell script into the current process
/// environment requires complex shell integration that varies by platform.
///
/// # Parameters
///
/// * `env` - Environment abstraction for reading environment variables
/// * `_runner` - Command runner (unused in current implementation, reserved for future use)
/// * `fs` - Filesystem abstraction for checking file existence
/// * `debug_mode` - When true, outputs detailed diagnostic information
///
/// # Returns
///
/// Always returns `Ok(())` as this is an optional enhancement. Missing init scripts are not
/// considered errors.
///
/// # Example Init Script
///
/// ```bash
/// # ~/.config/samoyed/init.sh
/// export NODE_OPTIONS="--max-old-space-size=4096"
/// export PATH="$HOME/.local/bin:$PATH"
///
/// # Define a helper function for all hooks
/// notify_slack() {
///     curl -X POST -H 'Content-type: application/json' \
///         --data "{\"text\":\"$1\"}" \
///         "$SLACK_WEBHOOK_URL"
/// }
/// ```
///
/// # Platform Considerations
///
/// - **Linux/macOS**: Uses `$HOME/.config/samoyed/init.sh` by default
/// - **Windows**: Falls back to `$USERPROFILE/.config/samoyed/init.sh`
/// - Respects `$XDG_CONFIG_HOME` if set (XDG Base Directory specification)
fn load_init_script(
    env: &dyn Environment,
    _runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    debug_mode: bool,
) -> Result<()> {
    // Determine the init script path: ~/.config/samoyed/init.sh
    let home_dir = env
        .get_var("HOME")
        .or_else(|| env.get_var("USERPROFILE")) // Windows fallback
        .context("Could not determine home directory")?;

    // Determine configuration directory based on platform and environment
    let config_dir = if cfg!(target_os = "windows") && !is_windows_unix_environment(env, debug_mode)
    {
        // Native Windows: use %APPDATA%/samoyed or fall back to %USERPROFILE%/.config/samoyed
        env.get_var("APPDATA")
            .map(|appdata| format!("{appdata}/samoyed"))
            .unwrap_or_else(|| format!("{home_dir}/.config/samoyed"))
    } else {
        // Unix-like systems (including WSL, Git Bash): use XDG Base Directory
        env.get_var("XDG_CONFIG_HOME")
            .map(|xdg| format!("{xdg}/samoyed"))
            .unwrap_or_else(|| format!("{home_dir}/.config/samoyed"))
    };

    // Choose script name based on environment
    let script_name =
        if cfg!(target_os = "windows") && !is_windows_unix_environment(env, debug_mode) {
            "init.cmd" // Use batch file on native Windows
        } else {
            "init.sh" // Use shell script on Unix-like systems
        };

    let init_script_path = PathBuf::from(config_dir).join(script_name);

    if debug_mode {
        log_file_operation_with_env(
            env,
            debug_mode,
            "Checking for init script at",
            &init_script_path,
        );
    }

    // If the init script exists, source it using shell
    if fs.exists(&init_script_path) {
        if debug_mode {
            log_file_operation_with_env(env, debug_mode, "Loading init script", &init_script_path);
        }

        // Note: We can't actually source the script into our environment easily
        // In a real implementation, this would require more complex shell integration
        // For now, we'll document this limitation and focus on hook execution
        if debug_mode {
            eprintln!("samoyed: Init script found but sourcing not implemented yet");
        }
    } else if debug_mode {
        eprintln!("samoyed: No init script found");
    }

    Ok(())
}

/// Determines the appropriate shell command and arguments for executing a hook script
/// based on the current platform and environment.
///
/// # Platform-Specific Behavior
///
/// - **Unix-like systems (Linux, macOS)**: Uses `/bin/sh` with `-e` flag for error handling
/// - **Windows without Git Bash**: Uses `cmd.exe /C` for batch files or `powershell -File` for PowerShell scripts
/// - **Windows with Git Bash**: Detects Git Bash via environment variables and prefers `sh.exe`
///
/// # Environment Detection
///
/// The function detects Windows Unix-like environments by checking:
/// - `MSYSTEM` environment variable (Git Bash: MINGW32, MINGW64, MSYS)
/// - `CYGWIN` environment variable (Cygwin environments)
/// - File system checks for `/proc/version` containing "Microsoft" or "WSL" (WSL detection)
///
/// # Arguments
///
/// * `env` - Environment provider for reading environment variables
/// * `script_path` - Path to the hook script to execute
/// * `args` - Arguments to pass to the hook script
/// * `debug_mode` - Whether to output debug information
///
/// # Returns
///
/// A tuple containing:
/// - Shell command to execute (e.g., "sh", "cmd", "powershell")
/// - Vector of arguments to pass to the shell
fn determine_shell_execution(
    env: &dyn Environment,
    script_path: &Path,
    args: &[&str],
    debug_mode: bool,
) -> (String, Vec<String>) {
    // Check if we're on Windows
    if cfg!(target_os = "windows") {
        // Check for Unix-like environments on Windows
        if is_windows_unix_environment(env, debug_mode) {
            if debug_mode {
                eprintln!("samoyed: Detected Unix-like environment on Windows, using sh");
            }
            return (
                "sh".to_string(),
                vec![
                    "-e".to_string(),
                    script_path.to_string_lossy().to_string(),
                    args.join(" "),
                ],
            );
        }

        // Native Windows execution
        if debug_mode {
            eprintln!("samoyed: Native Windows detected, determining shell by file extension");
        }

        // Determine shell based on file extension
        if let Some(ext) = script_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "bat" | "cmd" => {
                    return (
                        "cmd".to_string(),
                        vec![
                            "/C".to_string(),
                            script_path.to_string_lossy().to_string(),
                            args.join(" "),
                        ],
                    );
                }
                "ps1" => {
                    return (
                        "powershell".to_string(),
                        vec![
                            "-ExecutionPolicy".to_string(),
                            "Bypass".to_string(),
                            "-File".to_string(),
                            script_path.to_string_lossy().to_string(),
                            args.join(" "),
                        ],
                    );
                }
                _ => {}
            }
        }

        // Default to cmd.exe for extensionless files on Windows
        return (
            "cmd".to_string(),
            vec![
                "/C".to_string(),
                script_path.to_string_lossy().to_string(),
                args.join(" "),
            ],
        );
    }

    // Unix-like systems (Linux, macOS, etc.)
    if debug_mode {
        eprintln!("samoyed: Unix-like system detected, using sh");
    }
    (
        "sh".to_string(),
        vec![
            "-e".to_string(),
            script_path.to_string_lossy().to_string(),
            args.join(" "),
        ],
    )
}

/// Detects Unix-like environments running on Windows
///
/// This function checks for various indicators that suggest we're running in a
/// Unix-like environment on Windows, such as Git Bash, MSYS2, Cygwin, or WSL.
///
/// # Detection Methods
///
/// 1. **Git Bash/MSYS2**: Checks for `MSYSTEM` environment variable
/// 2. **Cygwin**: Checks for `CYGWIN` environment variable  
/// 3. **WSL**: Checks for `WSL_DISTRO_NAME` and `WSL_INTEROP` environment variables
///
/// # Arguments
///
/// * `env` - Environment provider for reading environment variables
/// * `debug_mode` - Whether to output debug information
///
/// # Returns
///
/// `true` if a Unix-like environment is detected, `false` otherwise
fn is_windows_unix_environment(env: &dyn Environment, debug_mode: bool) -> bool {
    // Check for Git Bash / MSYS2
    if let Some(msystem) = env.get_var("MSYSTEM") {
        if debug_mode {
            eprintln!("samoyed: Detected MSYSTEM={msystem}");
        }
        return matches!(msystem.as_str(), "MINGW32" | "MINGW64" | "MSYS");
    }

    // Check for Cygwin
    if env.get_var("CYGWIN").is_some() {
        if debug_mode {
            eprintln!("samoyed: Detected Cygwin environment");
        }
        return true;
    }

    // Check for WSL (Windows Subsystem for Linux)
    if env.get_var("WSL_DISTRO_NAME").is_some() || env.get_var("WSL_INTEROP").is_some() {
        if debug_mode {
            eprintln!("samoyed: Detected WSL environment");
        }
        return true;
    }

    false
}

/// Execute the actual hook script and handle exit codes
fn execute_hook_script(
    env: &dyn Environment,
    runner: &dyn CommandRunner,
    _fs: &dyn FileSystem,
    script_path: &Path,
    hook_args: &[String],
    debug_mode: bool,
) -> Result<()> {
    if debug_mode {
        log_file_operation_with_env(env, debug_mode, "Executing hook script", script_path);
        let sanitized_hook_args = sanitize_args(hook_args);
        eprintln!("samoyed: Hook arguments: {sanitized_hook_args:?}");
    }

    // Convert String args to &str for the runner interface
    let str_args: Vec<&str> = hook_args.iter().map(|s| s.as_str()).collect();

    // Determine appropriate shell and arguments based on platform and environment
    let (shell_command, shell_args) =
        determine_shell_execution(env, script_path, &str_args, debug_mode);

    if debug_mode {
        log_command_execution(debug_mode, &shell_command, &shell_args);
    }

    let output = runner
        .run_command(
            &shell_command,
            &shell_args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )
        .with_context(|| {
            format!(
                "Failed to execute hook script: {}",
                sanitize_path(script_path)
            )
        })?;

    // Check exit code and provide appropriate error messages
    let exit_code = output.status.code().unwrap_or(1);

    if debug_mode {
        eprintln!("samoyed: Hook script exit code: {exit_code}");
        if !output.stdout.is_empty() {
            eprintln!(
                "samoyed: Hook stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
        if !output.stderr.is_empty() {
            eprintln!(
                "samoyed: Hook stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // Print stdout and stderr from the hook
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Handle specific error cases
    if exit_code != 0 {
        let hook_name = script_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        eprintln!("samoyed - {hook_name} script failed (code {exit_code})");

        // Check for command not found (exit code 127)
        if exit_code == 127 {
            eprintln!("samoyed - command not found in PATH");
            if debug_mode {
                // Only show PATH in debug mode, and sanitize it
                if let Ok(path) = std::env::var("PATH") {
                    // Use platform-specific PATH separator for counting
                    let separator = if cfg!(target_os = "windows") {
                        ";"
                    } else {
                        ":"
                    };
                    let dir_count = path.split(separator).count();
                    eprintln!("samoyed - PATH contains {dir_count} directories");
                }
            } else {
                eprintln!("samoyed - run with SAMOYED=2 for more details");
            }
        }
    }

    // Exit with the same code as the hook script
    process::exit(exit_code);
}

#[cfg(test)]
#[path = "unit_tests/hook_runner_tests.rs"]
mod tests;
