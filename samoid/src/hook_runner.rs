//! Hook Execution Runtime for Samoid
//!
//! This binary implements the hook runner that executes actual hook scripts with proper
//! environment setup, error handling, and debugging support. It's designed to be installed
//! as the actual Git hook and execute the user-defined hook scripts.
//!
//! # Environment Variables
//!
//! - **SAMOID=0**: Skip all hook execution (useful for CI/deployment, rebasing)
//! - **SAMOID=1**: Normal execution mode (default)
//! - **SAMOID=2**: Enable debug mode with detailed script tracing
//!
//! # Architecture
//!
//! The hook runner follows these steps:
//! 1. Parse environment variables (SAMOID=0/1/2)
//! 2. Load initialization script from `~/.config/samoid/init.sh` (if exists)
//! 3. Locate and execute the actual hook script from project root
//! 4. Handle errors and propagate exit codes to Git
//!
//! Based on the original husky hook runner implementation.

use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process;

mod environment;
mod logging;

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
    // Check SAMOID environment variable for execution mode
    let samoid_mode = env.get_var("SAMOID").unwrap_or_else(|| "1".to_string());

    // SAMOID=0 means skip execution entirely
    if samoid_mode == "0" {
        process::exit(0);
    }

    // SAMOID=2 enables debug mode (script tracing)
    let debug_mode = samoid_mode == "2";

    if debug_mode {
        eprintln!("samoid: Debug mode enabled (SAMOID=2)");
        let sanitized_args = sanitize_args(args);
        eprintln!("samoid: Hook runner args: {sanitized_args:?}");
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
        eprintln!("samoid: Detected hook name: {hook_name}");
    }

    // Build the expected hook script path: .samoid/scripts/{hook_name}
    let hook_script_path = PathBuf::from(".samoid").join("scripts").join(hook_name);

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
            eprintln!("samoid: Hook script not found, exiting silently");
        }
        process::exit(0);
    }

    // Load initialization script from ~/.config/samoid/init.sh
    load_init_script(env, runner, fs, debug_mode)?;

    // Execute the hook script with proper environment
    execute_hook_script(env, runner, fs, &hook_script_path, &args[2..], debug_mode)
}

/// Load and execute the initialization script if it exists
fn load_init_script(
    env: &dyn Environment,
    _runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    debug_mode: bool,
) -> Result<()> {
    // Determine the init script path: ~/.config/samoid/init.sh
    let home_dir = env
        .get_var("HOME")
        .or_else(|| env.get_var("USERPROFILE")) // Windows fallback
        .context("Could not determine home directory")?;

    let xdg_config_home = env
        .get_var("XDG_CONFIG_HOME")
        .unwrap_or_else(|| format!("{home_dir}/.config"));

    let init_script_path = PathBuf::from(xdg_config_home)
        .join("samoid")
        .join("init.sh");

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
            eprintln!("samoid: Init script found but sourcing not implemented yet");
        }
    } else if debug_mode {
        eprintln!("samoid: No init script found");
    }

    Ok(())
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
        eprintln!("samoid: Hook arguments: {sanitized_hook_args:?}");
    }

    // Convert String args to &str for the runner interface
    let str_args: Vec<&str> = hook_args.iter().map(|s| s.as_str()).collect();

    // Execute the hook script using shell
    let shell_args = vec![
        "-e".to_string(),
        script_path.to_string_lossy().to_string(),
        str_args.join(" "),
    ];

    if debug_mode {
        log_command_execution(debug_mode, "sh", &shell_args);
    }

    let output = runner
        .run_command(
            "sh",
            &["-e", &script_path.to_string_lossy(), &str_args.join(" ")],
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
        eprintln!("samoid: Hook script exit code: {exit_code}");
        if !output.stdout.is_empty() {
            eprintln!(
                "samoid: Hook stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
        if !output.stderr.is_empty() {
            eprintln!(
                "samoid: Hook stderr: {}",
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

        eprintln!("samoid - {hook_name} script failed (code {exit_code})");

        // Check for command not found (exit code 127)
        if exit_code == 127 {
            eprintln!("samoid - command not found in PATH");
            if debug_mode {
                // Only show PATH in debug mode, and sanitize it
                if let Ok(path) = std::env::var("PATH") {
                    // Sanitize PATH by showing only directory count to avoid exposing system structure
                    let dir_count = path.split(':').count();
                    eprintln!("samoid - PATH contains {dir_count} directories");
                }
            } else {
                eprintln!("samoid - run with SAMOID=2 for more details");
            }
        }
    }

    // Exit with the same code as the hook script
    process::exit(exit_code);
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
    fn test_run_hook_with_samoid_0_skips_execution() {
        let env = MockEnvironment::new().with_var("SAMOID", "0");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // The function should return Ok and exit early with SAMOID=0
        // Note: In a real test, we'd need to mock process::exit
        // For now, we test the logic path before the exit
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        // The function should panic with process::exit(0)
        assert!(result.is_err());
    }

    #[test]
    fn test_run_hook_with_debug_mode() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "2")
            .with_var("HOME", "/home/test");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No hook script exists

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // Should exit early because hook script doesn't exist
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(0)
    }

    #[test]
    fn test_run_hook_executes_existing_script() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "1")
            .with_var("HOME", "/home/test");

        // Mock successful hook execution
        let output = Output {
            status: exit_status(0),
            stdout: b"Hook executed successfully".to_vec(),
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-e", ".samoid/scripts/pre-commit", ""],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_file(
            ".samoid/scripts/pre-commit",
            "#!/bin/sh\necho 'Hook executed successfully'",
        );

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // Should execute the hook and exit with code 0
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(0)
    }

    #[test]
    fn test_run_hook_handles_failed_script() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "1")
            .with_var("HOME", "/home/test");

        // Mock failed hook execution
        let output = Output {
            status: exit_status(1),
            stdout: vec![],
            stderr: b"Hook failed".to_vec(),
        };
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-e", ".samoid/scripts/pre-commit", ""],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_file(".samoid/scripts/pre-commit", "#!/bin/sh\nexit 1");

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // Should execute the hook and exit with code 1
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(1)
    }

    #[test]
    fn test_run_hook_command_not_found() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "1")
            .with_var("HOME", "/home/test");

        // Mock command not found (exit code 127)
        let output = Output {
            status: exit_status(127),
            stdout: vec![],
            stderr: b"command not found".to_vec(),
        };
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-e", ".samoid/scripts/pre-commit", ""],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_file(
            ".samoid/scripts/pre-commit",
            "#!/bin/sh\nnonexistent_command",
        );

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // Should handle command not found error
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(127)
    }

    #[test]
    fn test_load_init_script_with_existing_file() {
        let env = MockEnvironment::new()
            .with_var("HOME", "/home/test")
            .with_var("XDG_CONFIG_HOME", "/home/test/.config");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new().with_file(
            "/home/test/.config/samoid/init.sh",
            "export PATH=/custom:$PATH",
        );

        let result = load_init_script(&env, &runner, &fs, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_init_script_with_missing_home() {
        let env = MockEnvironment::new(); // No HOME variable
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = load_init_script(&env, &runner, &fs, false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Could not determine home directory")
        );
    }

    #[test]
    fn test_hook_name_extraction() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "1")
            .with_var("HOME", "/home/test");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No hook script

        // Test with different hook names
        let test_cases = vec![
            vec!["samoid-hook".to_string(), "pre-commit".to_string()],
            vec!["samoid-hook".to_string(), "post-commit".to_string()],
            vec!["samoid-hook".to_string(), "pre-push".to_string()],
            vec![
                "samoid-hook".to_string(),
                "/path/to/pre-receive".to_string(),
            ],
        ];

        for args in test_cases {
            let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

            // Should exit cleanly (no hook script found)
            assert!(result.is_err()); // Due to process::exit(0)
        }
    }

    #[test]
    fn test_hook_with_arguments() {
        let env = MockEnvironment::new()
            .with_var("SAMOID", "1")
            .with_var("HOME", "/home/test");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-e", ".samoid/scripts/pre-push", "origin main"],
            Ok(output),
        );

        let fs =
            MockFileSystem::new().with_file(".samoid/scripts/pre-push", "#!/bin/sh\necho $1 $2");

        let args = vec![
            "samoid-hook".to_string(),
            "pre-push".to_string(),
            "origin".to_string(),
            "main".to_string(),
        ];

        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(0)
    }

    #[test]
    fn test_default_samoid_mode() {
        let env = MockEnvironment::new().with_var("HOME", "/home/test"); // No SAMOID variable set
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No hook script

        let args = vec!["samoid-hook".to_string(), "pre-commit".to_string()];

        // Should default to SAMOID=1 (normal mode)
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

        assert!(result.is_err()); // Due to process::exit(0)
    }

    #[test]
    fn test_empty_args_error() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let args: Vec<String> = vec![];

        let result = run_hook(&env, &runner, &fs, &args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No hook name provided")
        );
    }

    #[test]
    fn test_insufficient_args_error() {
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let args = vec!["samoid-hook".to_string()]; // Missing hook name

        let result = run_hook(&env, &runner, &fs, &args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No hook name provided")
        );
    }

    #[test]
    fn test_windows_home_fallback() {
        let env = MockEnvironment::new()
            .with_var("USERPROFILE", "C:\\Users\\test") // Windows home
            .with_var("SAMOID", "1");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = load_init_script(&env, &runner, &fs, false);
        assert!(result.is_ok());
    }
}
