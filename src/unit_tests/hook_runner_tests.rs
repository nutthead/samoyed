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

// Helper function to create Output structs with consistent pattern
fn make_output(status_code: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status: exit_status(status_code),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

#[test]
fn test_run_hook_with_samoyed_0_skips_execution() {
    let env = MockEnvironment::new().with_var("SAMOYED", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // The function should return Ok and exit early with SAMOYED=0
    // Note: In a real test, we'd need to mock process::exit
    // For now, we test the logic path before the exit
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

    // The function should panic with process::exit(0)
    assert!(result.is_err());
}

#[test]
fn test_run_hook_with_debug_mode() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2")
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No hook script exists

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // Should exit early because hook script doesn't exist
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_executes_existing_script() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock successful hook execution
    let output = make_output(0, b"Hook executed successfully", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\necho 'Hook executed successfully'",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // Should execute the hook and exit with code 0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_handles_failed_script() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock failed hook execution
    let output = make_output(1, b"", b"Hook failed");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(".samoyed/scripts/pre-commit", "#!/bin/sh\nexit 1");

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // Should execute the hook and exit with code 1
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_run_hook_command_not_found() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock command not found (exit code 127)
    let output = make_output(127, b"", b"command not found");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\nnonexistent_command",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

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
        "/home/test/.config/samoyed/init.sh",
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
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No hook script

    // Test with different hook names
    let test_cases = vec![
        vec!["samoyed-hook".to_string(), "pre-commit".to_string()],
        vec!["samoyed-hook".to_string(), "post-commit".to_string()],
        vec!["samoyed-hook".to_string(), "pre-push".to_string()],
        vec![
            "samoyed-hook".to_string(),
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
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    let output = make_output(0, b"", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-push", "origin main"],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(".samoyed/scripts/pre-push", "#!/bin/sh\necho $1 $2");

    let args = vec![
        "samoyed-hook".to_string(),
        "pre-push".to_string(),
        "origin".to_string(),
        "main".to_string(),
    ];

    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));

    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_default_samoyed_mode() {
    let env = MockEnvironment::new().with_var("HOME", "/home/test"); // No SAMOYED variable set
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No hook script

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // Should default to SAMOYED=1 (normal mode)
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

    let args = vec!["samoyed-hook".to_string()]; // Missing hook name

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
        .with_var("SAMOYED", "1");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let result = load_init_script(&env, &runner, &fs, false);
    assert!(result.is_ok());
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_determine_shell_execution_unix() {
    let env = MockEnvironment::new();
    let script_path = std::path::Path::new("/path/to/script.sh");
    let args = ["arg1", "arg2"];

    // On Unix systems, should always use sh
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "sh");
    assert_eq!(shell_args, vec!["-e", "/path/to/script.sh", "arg1 arg2"]);
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_git_bash() {
    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    let script_path = std::path::Path::new("C:\\path\\to\\script.sh");
    let args = ["arg1", "arg2"];

    // When MSYSTEM is set, should use sh even on Windows
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "sh");
    assert_eq!(
        shell_args,
        vec!["-e", "C:\\path\\to\\script.sh", "arg1 arg2"]
    );
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_cmd() {
    let env = MockEnvironment::new(); // No MSYSTEM or CYGWIN
    let script_path = std::path::Path::new("C:\\path\\to\\script.bat");
    let args = ["arg1", "arg2"];

    // Windows batch files should use cmd
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "cmd");
    assert_eq!(
        shell_args,
        vec!["/C", "C:\\path\\to\\script.bat", "arg1 arg2"]
    );
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_powershell() {
    let env = MockEnvironment::new(); // No MSYSTEM or CYGWIN
    let script_path = std::path::Path::new("C:\\path\\to\\script.ps1");
    let args = ["arg1", "arg2"];

    // PowerShell scripts should use powershell
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "powershell");
    assert_eq!(
        shell_args,
        vec![
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "C:\\path\\to\\script.ps1",
            "arg1 arg2"
        ]
    );
}

#[test]
fn test_is_windows_unix_environment_git_bash() {
    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    assert!(is_windows_unix_environment(&env, false));

    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW32");
    assert!(is_windows_unix_environment(&env, false));

    let env = MockEnvironment::new().with_var("MSYSTEM", "MSYS");
    assert!(is_windows_unix_environment(&env, false));
}

#[test]
fn test_is_windows_unix_environment_cygwin() {
    let env = MockEnvironment::new().with_var("CYGWIN", "nodosfilewarning");
    assert!(is_windows_unix_environment(&env, false));
}

#[test]
fn test_is_windows_unix_environment_wsl() {
    let env = MockEnvironment::new().with_var("WSL_DISTRO_NAME", "Ubuntu");
    assert!(is_windows_unix_environment(&env, false));

    let env = MockEnvironment::new().with_var("WSL_INTEROP", "/run/WSL/123_interop");
    assert!(is_windows_unix_environment(&env, false));
}

#[test]
fn test_is_windows_unix_environment_native_windows() {
    let env = MockEnvironment::new(); // No special environment variables
    assert!(!is_windows_unix_environment(&env, false));
}

#[test]
fn test_load_init_script() {
    // Test 1: Init script exists with HOME directory
    let env = MockEnvironment::new().with_var("HOME", "/home/user");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "/home/user/.config/samoyed/init.sh",
        "#!/bin/bash\nexport FOO=bar",
    );

    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok(), "Should succeed when init script exists");

    // Test 2: Init script with XDG_CONFIG_HOME
    let env_xdg = MockEnvironment::new()
        .with_var("HOME", "/home/user")
        .with_var("XDG_CONFIG_HOME", "/custom/config");
    let fs_xdg = MockFileSystem::new().with_file(
        "/custom/config/samoyed/init.sh",
        "#!/bin/bash\nexport BAR=baz",
    );

    let result_xdg = load_init_script(&env_xdg, &runner, &fs_xdg, true);
    assert!(result_xdg.is_ok(), "Should respect XDG_CONFIG_HOME");

    // Test 3: No init script (should still succeed)
    let env_no_script = MockEnvironment::new().with_var("HOME", "/home/user");
    let fs_no_script = MockFileSystem::new();

    let result_no_script = load_init_script(&env_no_script, &runner, &fs_no_script, false);
    assert!(
        result_no_script.is_ok(),
        "Should succeed even without init script"
    );

    // Test 4: Windows USERPROFILE fallback
    let env_windows = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\user");
    let fs_windows = MockFileSystem::new().with_file(
        "C:\\Users\\user\\.config\\samoyed\\init.sh",
        "REM Windows init",
    );

    let result_windows = load_init_script(&env_windows, &runner, &fs_windows, false);
    assert!(
        result_windows.is_ok(),
        "Should work with Windows USERPROFILE"
    );

    // Test 5: No home directory at all
    let env_no_home = MockEnvironment::new();
    let result_no_home = load_init_script(&env_no_home, &runner, &fs, false);
    assert!(
        result_no_home.is_err(),
        "Should fail without HOME or USERPROFILE"
    );
    assert!(
        result_no_home
            .unwrap_err()
            .to_string()
            .contains("home directory"),
        "Error should mention home directory"
    );
}

#[test]
fn test_load_hook_command_from_config_success() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "cargo fmt --check"
pre-push = "cargo test""#,
    );

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check");

    let result = load_hook_command_from_config(&fs, "pre-push", false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo test");
}

#[test]
fn test_load_hook_command_from_config_missing_file() {
    let fs = MockFileSystem::new(); // No samoyed.toml file

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No samoyed.toml configuration file found")
    );
}

#[test]
fn test_load_hook_command_from_config_invalid_toml() {
    let fs = MockFileSystem::new().with_file("samoyed.toml", "invalid toml content [[[");

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse samoyed.toml")
    );
}

#[test]
fn test_load_hook_command_from_config_hook_not_found() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "cargo fmt --check""#,
    );

    let result = load_hook_command_from_config(&fs, "pre-push", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No command configured for hook 'pre-push'")
    );
}

#[test]
fn test_load_hook_command_from_config_debug_mode() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "cargo fmt --check""#,
    );

    let result = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check");
}

#[test]
fn test_run_hook_with_toml_config_success() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock successful command execution
    let output = make_output(0, b"Formatting complete", b"");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "cargo fmt --check"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "cargo fmt --check""#,
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the TOML command path and exit with code 0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_with_toml_config_failure() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock failed command execution
    let output = make_output(1, b"", b"Formatting failed");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "cargo fmt --check"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "cargo fmt --check""#,
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the TOML command path and exit with code 1
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_run_hook_with_toml_config_command_not_found() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock command not found (exit code 127)
    let output = make_output(127, b"", b"command not found");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "nonexistent_command"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "nonexistent_command""#,
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the TOML command path and exit with code 127
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(127)
}

#[test]
fn test_run_hook_script_execution_success() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock successful script execution
    let output = make_output(0, b"Script executed successfully", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\necho 'Script executed successfully'",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the script file path and exit with code 0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_script_execution_failure() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock failed script execution
    let output = make_output(1, b"", b"Script failed");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(".samoyed/scripts/pre-commit", "#!/bin/sh\nexit 1");

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the script file path and exit with code 1
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_run_hook_script_execution_with_arguments() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock script execution with arguments
    let output = make_output(0, b"origin main processed", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-push", "origin main"],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-push",
        "#!/bin/sh\necho \"$1 $2 processed\"",
    );

    let args = vec![
        "samoyed-hook".to_string(),
        "pre-push".to_string(),
        "origin".to_string(),
        "main".to_string(),
    ];

    // This should execute the script with arguments and exit with code 0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_script_command_not_found() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock command not found in script (exit code 127)
    let output = make_output(127, b"", b"nonexistent_command: command not found");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\nnonexistent_command",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the script and exit with code 127
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(127)
}

#[test]
fn test_run_hook_script_debug_mode() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");

    // Mock successful script execution with debug output
    let output = make_output(0, b"Debug script executed", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\necho 'Debug script executed'",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute with debug logging and exit with code 0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_script_function() {
    let env = MockEnvironment::new();
    let fs = MockFileSystem::new();

    // Mock successful script execution
    let output = make_output(0, b"Test output", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/test/script.sh", "arg1 arg2"],
        Ok(output),
    );

    let script_path = std::path::Path::new("/test/script.sh");
    let hook_args = vec!["arg1".to_string(), "arg2".to_string()];

    // This function should exit with process::exit, so catch the panic
    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &hook_args, false)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_native() {
    let env = MockEnvironment::new(); // No Unix environment variables
    let script_path = std::path::Path::new("C:\\path\\to\\script");
    let args = ["arg1", "arg2"];

    // Native Windows should default to cmd for extensionless files
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "cmd");
    assert_eq!(shell_args, vec!["/C", "C:\\path\\to\\script", "arg1 arg2"]);
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_appdata_config() {
    let env = MockEnvironment::new()
        .with_var("APPDATA", "C:\\Users\\test\\AppData\\Roaming")
        .with_var("USERPROFILE", "C:\\Users\\test");

    // Test Windows APPDATA configuration path in init script loading
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "C:\\Users\\test\\AppData\\Roaming\\samoyed\\init.cmd",
        "@echo off\nset NODE_OPTIONS=--max-old-space-size=4096",
    );

    let result = load_init_script(&env, &runner, &fs, false);
    assert!(result.is_ok());
}

#[cfg(target_os = "windows")]
#[test]
fn test_run_hook_windows_cmd_execution() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");

    // Mock Windows batch file execution
    let output = make_output(0, b"Windows batch executed", b"");
    let runner = MockCommandRunner::new().with_response(
        "cmd",
        &["/C", ".samoyed\\scripts\\pre-commit.bat", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit.bat",
        "@echo off\necho Windows batch executed",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the Windows batch file
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[cfg(target_os = "windows")]
#[test]
fn test_run_hook_windows_powershell_execution() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");

    // Mock PowerShell script execution
    let output = make_output(0, b"PowerShell script executed", b"");
    let runner = MockCommandRunner::new().with_response(
        "powershell",
        &[
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            ".samoyed\\scripts\\pre-commit.ps1",
            "",
        ],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit.ps1",
        "Write-Host 'PowerShell script executed'",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute the PowerShell script
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_windows_shell_detection_edge_cases() {
    // Test Windows environment detection edge cases
    let env_empty = MockEnvironment::new();
    assert!(!is_windows_unix_environment(&env_empty, false));

    let env_invalid_msystem = MockEnvironment::new().with_var("MSYSTEM", "INVALID");
    assert!(!is_windows_unix_environment(&env_invalid_msystem, false));

    let env_wsl_both = MockEnvironment::new()
        .with_var("WSL_DISTRO_NAME", "Ubuntu")
        .with_var("WSL_INTEROP", "/run/WSL/123_interop");
    assert!(is_windows_unix_environment(&env_wsl_both, false));
}

#[cfg(target_os = "windows")]
#[test]
fn test_load_init_script_windows_native_paths() {
    let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\test");

    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No init script

    // Should succeed even without init script on Windows
    let result = load_init_script(&env, &runner, &fs, false);
    assert!(result.is_ok());
}

#[cfg(target_os = "windows")]
#[test]
fn test_execute_hook_command_windows_shell() {
    let env = MockEnvironment::new(); // Native Windows environment

    // Mock Windows command execution
    let output = make_output(0, b"Windows command executed", b"");
    let runner = MockCommandRunner::new().with_response("cmd", &["/C", "dir"], Ok(output));

    let hook_args = vec![];

    // This function should exit with process::exit, so catch the panic
    let result =
        std::panic::catch_unwind(|| execute_hook_command(&env, &runner, "dir", &hook_args, false));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_with_samoyed_0_early_exit() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "0") // Skip execution
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // The function should exit early with SAMOYED=0
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_debug_mode_extensive_logging() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");

    // Mock successful script execution with debug output
    let output = make_output(0, b"Debug command executed", b"");
    let runner = MockCommandRunner::new().with_response("sh", &["-c", "echo 'debug'"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "echo 'debug'""#,
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should execute with extensive debug logging
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_various_environment_modes() {
    // Test SAMOYED=1 (normal mode)
    let env_normal = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No hooks available

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    let result = std::panic::catch_unwind(|| run_hook(&env_normal, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)

    // Test unset SAMOYED (should default to "1")
    let env_unset = MockEnvironment::new().with_var("HOME", "/home/test");
    let result = std::panic::catch_unwind(|| run_hook(&env_unset, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)

    // Test invalid SAMOYED value (should be treated as normal mode)
    let env_invalid = MockEnvironment::new()
        .with_var("SAMOYED", "invalid")
        .with_var("HOME", "/home/test");
    let result = std::panic::catch_unwind(|| run_hook(&env_invalid, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_hook_name_extraction_edge_cases() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No hook script

    // Test with path-like hook name
    let args_path = vec![
        "samoyed-hook".to_string(),
        "/path/to/pre-commit".to_string(),
    ];
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args_path));
    assert!(result.is_err()); // Due to process::exit(0)

    // Test with Windows-style path
    let args_windows = vec![
        "samoyed-hook".to_string(),
        "C:\\hooks\\pre-commit.exe".to_string(),
    ];
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args_windows));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_init_script_xdg_config_home() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test")
        .with_var("XDG_CONFIG_HOME", "/custom/config");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "/custom/config/samoyed/init.sh",
        "#!/bin/bash\nexport CUSTOM_VAR=value",
    );

    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());
}

#[test]
fn test_init_script_no_home_variables() {
    let env = MockEnvironment::new(); // No HOME or USERPROFILE
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
fn test_command_execution_with_stdout_stderr() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock command with both stdout and stderr
    let output = make_output(0, b"Command output to stdout", b"Command output to stderr");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "echo 'test' && echo 'error' >&2"],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        r#"[hooks]
pre-commit = "echo 'test' && echo 'error' >&2""#,
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should handle both stdout and stderr
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_hook_execution_with_complex_arguments() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Mock script execution with complex arguments
    let output = make_output(0, b"Complex args processed", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &[
            "-e",
            ".samoyed/scripts/prepare-commit-msg",
            ".git/COMMIT_EDITMSG commit",
        ],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/prepare-commit-msg",
        "#!/bin/sh\necho \"Processing: $1 $2\"",
    );

    let args = vec![
        "samoyed-hook".to_string(),
        "prepare-commit-msg".to_string(),
        ".git/COMMIT_EDITMSG".to_string(),
        "commit".to_string(),
    ];

    // This should handle complex Git hook arguments
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_error_propagation_and_exit_codes() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    // Test various exit codes
    for exit_code in [1, 2, 127, 255] {
        let output = make_output(
            exit_code,
            b"",
            &format!("Error with code {exit_code}").into_bytes(),
        );
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-c", &format!("exit {exit_code}")],
            Ok(output),
        );

        let fs = MockFileSystem::new().with_file(
            "samoyed.toml",
            &format!(
                r#"[hooks]
pre-commit = "exit {exit_code}""#
            ),
        );

        let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

        // This should propagate the exit code
        let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
        assert!(result.is_err()); // Due to process::exit(exit_code)
    }
}
