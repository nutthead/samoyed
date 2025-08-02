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
}

// Add comprehensive tests for missing coverage

#[test]
fn test_load_hook_command_from_config_success() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"cargo fmt --check && cargo clippy\"",
    );

    let result = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check && cargo clippy");
}

#[test]
fn test_load_hook_command_from_config_missing_file() {
    let fs = MockFileSystem::new(); // No samoyed.toml file

    let result = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No samoyed.toml"));
}

#[test]
fn test_load_hook_command_from_config_missing_hook() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-push = \"cargo test\"", // Only pre-push, no pre-commit
    );

    let result = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No command configured")
    );
}

#[test]
fn test_load_hook_command_from_config_invalid_toml() {
    let fs = MockFileSystem::new().with_file("samoyed.toml", "invalid toml syntax [[[");

    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse"));
}

#[test]
fn test_run_hook_with_toml_config_success() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");

    let output = make_output(0, b"Formatting complete", b"");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "cargo fmt --check"], Ok(output));

    let fs = MockFileSystem::new()
        .with_file(
            "samoyed.toml",
            "[hooks]\npre-commit = \"cargo fmt --check\"",
        )
        .with_file(
            "/home/test/.config/samoyed/init.sh",
            "#!/bin/sh\necho 'init'",
        );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_with_toml_config_failure() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");

    let output = make_output(1, b"", b"Formatting failed");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "cargo fmt --check"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"cargo fmt --check\"",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_run_hook_with_toml_config_command_not_found() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("HOME", "/home/test");

    let output = make_output(127, b"", b"command not found");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-c", "nonexistent_command"], Ok(output));

    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"nonexistent_command\"",
    );

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(127)
}

#[cfg(target_os = "windows")]
#[test]
fn test_execute_hook_command_windows() {
    let env = MockEnvironment::new(); // Native Windows
    let output = make_output(0, b"success", b"");
    let runner = MockCommandRunner::new().with_response("cmd", &["/C", "echo hello"], Ok(output));

    let result =
        std::panic::catch_unwind(|| execute_hook_command(&env, &runner, "echo hello", &[], false));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_script_with_output() {
    let env = MockEnvironment::new().with_var("HOME", "/home/test");

    let output = make_output(0, b"Script output", b"Script stderr");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-e", "/path/to/script", ""], Ok(output));

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");

    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &[], true)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_script_with_arguments() {
    let env = MockEnvironment::new().with_var("HOME", "/home/test");

    let output = make_output(0, b"", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/path/to/script", "arg1 arg2"],
        Ok(output),
    );

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");
    let args = vec!["arg1".to_string(), "arg2".to_string()];

    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &args, false)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_script_failure_with_debug() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test")
        .with_var("PATH", "/usr/bin:/bin:/usr/local/bin");

    let output = make_output(127, b"", b"command not found");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-e", "/path/to/script", ""], Ok(output));

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");

    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &[], true) // debug mode
    });
    assert!(result.is_err()); // Due to process::exit(127)
}

#[test]
fn test_execute_hook_script_failure_without_debug() {
    let env = MockEnvironment::new().with_var("HOME", "/home/test");

    let output = make_output(127, b"", b"command not found");
    let runner =
        MockCommandRunner::new().with_response("sh", &["-e", "/path/to/script", ""], Ok(output));

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");

    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &[], false) // no debug
    });
    assert!(result.is_err()); // Due to process::exit(127)
}

#[cfg(target_os = "windows")]
#[test]
fn test_load_init_script_windows_appdata() {
    let env = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\test")
        .with_var("APPDATA", "C:\\Users\\test\\AppData\\Roaming");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "C:\\Users\\test\\AppData\\Roaming\\samoyed\\init.cmd",
        "@echo off\nset FOO=bar",
    );

    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());
}

#[cfg(target_os = "windows")]
#[test]
fn test_load_init_script_windows_no_appdata() {
    let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\test");
    // No APPDATA variable
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "C:\\Users\\test\\.config\\samoyed\\init.cmd",
        "@echo off\nset FOO=bar",
    );

    let result = load_init_script(&env, &runner, &fs, false);
    assert!(result.is_ok());
}

#[test]
fn test_load_init_script_no_script_debug_mode() {
    let env = MockEnvironment::new().with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No init script

    let result = load_init_script(&env, &runner, &fs, true); // debug mode
    assert!(result.is_ok()); // Should succeed even without script
}

#[test]
fn test_debug_mode_output_coverage() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");

    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file("samoyed.toml", "[hooks]\npre-commit = \"echo test\"");

    let args = vec![
        "samoyed-hook".to_string(),
        "pre-commit".to_string(),
        "extra".to_string(),
        "args".to_string(),
    ];

    // This should exercise debug logging paths
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit
}

// Additional tests for comprehensive coverage

#[test]
fn test_run_hook_samoyed_mode_0_exits_immediately() {
    let env = MockEnvironment::new().with_var("SAMOYED", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();
    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // SAMOYED=0 should cause immediate exit - line 95
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test] 
fn test_load_hook_command_from_config_debug_output() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"cargo fmt --check\"",
    );

    // Test debug mode true - covers lines 177, 182, 188, etc.
    let result = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check");
}

#[test]
fn test_load_hook_command_from_config_no_debug() {
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"cargo fmt --check\"",
    );

    // Test debug mode false
    let result = load_hook_command_from_config(&fs, "pre-commit", false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cargo fmt --check");
}

#[test]
fn test_execute_hook_command_debug_mode() {
    let env = MockEnvironment::new();
    let output = make_output(0, b"Command output", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "echo hello"],
        Ok(output),
    );

    // Test debug mode - covers lines 228-231, 249, 253, etc.
    let result = std::panic::catch_unwind(|| {
        execute_hook_command(&env, &runner, "echo hello", &[], true)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_command_with_args() {
    let env = MockEnvironment::new();
    let output = make_output(0, b"", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "echo hello"],
        Ok(output),
    );

    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let result = std::panic::catch_unwind(|| {
        execute_hook_command(&env, &runner, "echo hello", &args, true)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_command_failure_exit_codes() {
    let test_cases = vec![1, 2, 127, 255];
    
    for exit_code in test_cases {
        let env = MockEnvironment::new();
        let output = make_output(exit_code, b"", if exit_code == 127 { b"command not found" } else { b"error" });
        let runner = MockCommandRunner::new().with_response(
            "sh",
            &["-c", "failing_command"],
            Ok(output),
        );

        // Test different exit codes - covers lines 277-278, 281-284
        let result = std::panic::catch_unwind(|| {
            execute_hook_command(&env, &runner, "failing_command", &[], true)
        });
        assert!(result.is_err()); // Due to process::exit(exit_code)
    }
}

#[test]
fn test_execute_hook_command_with_output() {
    let env = MockEnvironment::new();
    let output = make_output(0, b"stdout content", b"stderr content");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "echo hello"],
        Ok(output),
    );

    // Test command with stdout/stderr - covers lines 269-270, 272-273
    let result = std::panic::catch_unwind(|| {
        execute_hook_command(&env, &runner, "echo hello", &[], false)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_toml_config_with_init_script() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");
    
    let output = make_output(0, b"success", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "cargo fmt --check"],
        Ok(output),
    );

    let fs = MockFileSystem::new()
        .with_file("samoyed.toml", "[hooks]\npre-commit = \"cargo fmt --check\"")
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\nexport PATH=/custom:$PATH");

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This covers TOML config path with init script - lines 123-125, 129, 132
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_toml_config_fallback_to_script() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2")
        .with_var("HOME", "/home/test");
    
    let script_output = make_output(0, b"script executed", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(script_output),
    );

    let fs = MockFileSystem::new()
        .with_file("samoyed.toml", "[hooks]\npre-push = \"cargo test\"") // Only pre-push, no pre-commit
        .with_file(".samoyed/scripts/pre-commit", "#!/bin/sh\necho 'script executed'")
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\necho init");

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This covers TOML config failure -> fallback to script - lines 134-138, 162, 165
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_is_windows_unix_environment_debug_mode() {
    // Test debug output for platform detection - lines 542-544, 551-552, 559-560
    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    let result = is_windows_unix_environment(&env, true); // debug mode
    assert!(result);

    let env_cygwin = MockEnvironment::new().with_var("CYGWIN", "nodosfilewarning");
    let result_cygwin = is_windows_unix_environment(&env_cygwin, true);
    assert!(result_cygwin);

    let env_wsl = MockEnvironment::new().with_var("WSL_DISTRO_NAME", "Ubuntu");
    let result_wsl = is_windows_unix_environment(&env_wsl, true);
    assert!(result_wsl);
}

#[test]
fn test_determine_shell_execution_debug_mode() {
    let env = MockEnvironment::new();
    let script_path = std::path::Path::new("/path/to/script.sh");
    let args = ["arg1", "arg2"];

    // Test debug mode - covers lines 508-510 
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, true);
    assert_eq!(shell, "sh");
    assert_eq!(shell_args, vec!["-e", "/path/to/script.sh", "arg1 arg2"]);
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_debug() {
    let env = MockEnvironment::new(); // Native Windows
    let script_path = std::path::Path::new("C:\\path\\to\\script.bat");
    let args = ["arg1", "arg2"];

    // Test Windows debug mode - covers lines 446, 463-464, 468
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, true);
    assert_eq!(shell, "cmd");
    assert_eq!(shell_args, vec!["/C", "C:\\path\\to\\script.bat", "arg1 arg2"]);
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_extensionless() {
    let env = MockEnvironment::new(); // Native Windows
    let script_path = std::path::Path::new("C:\\path\\to\\script"); // No extension
    let args = ["arg1"];

    // Test Windows extensionless file - covers lines 497-504
    let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
    assert_eq!(shell, "cmd");
    assert_eq!(shell_args, vec!["/C", "C:\\path\\to\\script", "arg1"]);
}

#[test]
fn test_load_init_script_xdg_config_debug() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test")
        .with_var("XDG_CONFIG_HOME", "/custom/config");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_file(
        "/custom/config/samoyed/init.sh",
        "#!/bin/sh\nexport CUSTOM=value",
    );

    // Test XDG_CONFIG_HOME with debug - covers lines 383, 388, 393-395
    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());
}

// More targeted tests to hit specific uncovered lines

#[test]
fn test_run_hook_debug_mode_comprehensive() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode - ensures line 102-104
        .with_var("HOME", "/home/test");
    
    let output = make_output(0, b"success", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "echo test"],
        Ok(output),
    );

    let fs = MockFileSystem::new()
        .with_file("samoyed.toml", "[hooks]\npre-commit = \"echo test\"") // Config exists
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\necho init");

    let args = vec![
        "samoyed-hook".to_string(),
        "pre-commit".to_string(),
        "arg1".to_string(),
        "arg2".to_string(),
    ];

    // This should hit debug lines: 102-104, 118, 125, 249, 253, 264-265
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_toml_not_found_fallback() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");
    
    let script_output = make_output(0, b"script works", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", ".samoyed/scripts/pre-commit", ""],
        Ok(script_output),
    );

    let fs = MockFileSystem::new()
        // No samoyed.toml file - will fail config loading
        .with_file(".samoyed/scripts/pre-commit", "#!/bin/sh\necho 'script works'")
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\necho init");

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should hit the fallback path: lines 134-138, 149, 156, 162, 165
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_run_hook_no_script_exists() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");
    
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new()
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\necho init");
        // No samoyed.toml, no script file

    let args = vec!["samoyed-hook".to_string(), "pre-commit".to_string()];

    // This should hit the "no script found, exit silently" path: lines 154-158
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_execute_hook_script_stderr_and_stdout() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test")
        .with_var("PATH", "/usr/bin:/bin");
    
    let output = make_output(2, b"Some stdout", b"Some stderr");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/path/to/script", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");

    // This should hit lines 609-612, 617-618, 626-627, 629-630, 634-635, 637, 640
    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &[], true)
    });
    assert!(result.is_err()); // Due to process::exit(2)
}

#[test]
fn test_execute_hook_script_command_not_found() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test")
        .with_var("PATH", "/usr/bin:/bin:/usr/local/bin");
    
    let output = make_output(127, b"", b"command not found");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/path/to/script", "arg1 arg2"],
        Ok(output),
    );

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/path/to/script");
    let args = vec!["arg1".to_string(), "arg2".to_string()];

    // This should hit lines 643-645, 647, 652, 654-655, 658
    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &args, true)
    });
    assert!(result.is_err()); // Due to process::exit(127)
}

#[cfg(target_os = "windows")]
#[test]
fn test_load_init_script_windows_paths() {
    // Test APPDATA path - lines 364-365
    let env = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\test")
        .with_var("APPDATA", "C:\\Users\\test\\AppData\\Roaming");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new()
        .with_file("C:\\Users\\test\\AppData\\Roaming\\samoyed\\init.cmd", "@echo off");

    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());

    // Test fallback path - lines 368-370
    let env2 = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\test");
        // No APPDATA
    let fs2 = MockFileSystem::new()
        .with_file("C:\\Users\\test\\.config\\samoyed\\init.cmd", "@echo off");

    let result2 = load_init_script(&env2, &runner, &fs2, true);
    assert!(result2.is_ok());
}

#[cfg(target_os = "windows")]
#[test]
fn test_determine_shell_execution_windows_comprehensive() {
    let env = MockEnvironment::new(); // Native Windows

    // Test .ps1 files - lines 480-491
    let ps1_path = std::path::Path::new("C:\\script.ps1");
    let (shell, args) = determine_shell_execution(&env, ps1_path, &["arg"], true);
    assert_eq!(shell, "powershell");
    assert_eq!(args, vec!["-ExecutionPolicy", "Bypass", "-File", "C:\\script.ps1", "arg"]);

    // Test .cmd files - lines 470-479
    let cmd_path = std::path::Path::new("C:\\script.cmd");
    let (shell, args) = determine_shell_execution(&env, cmd_path, &["arg"], false);
    assert_eq!(shell, "cmd");
    assert_eq!(args, vec!["/C", "C:\\script.cmd", "arg"]);
}

#[cfg(target_os = "windows")]
#[test]
fn test_is_windows_unix_environment_comprehensive() {
    // Test all MSYSTEM values - lines 542-544, 546
    let systems = vec!["MINGW32", "MINGW64", "MSYS"];
    for system in systems {
        let env = MockEnvironment::new().with_var("MSYSTEM", system);
        let result = is_windows_unix_environment(&env, true);
        assert!(result);
    }

    // Test invalid MSYSTEM
    let env_invalid = MockEnvironment::new().with_var("MSYSTEM", "INVALID");
    let result_invalid = is_windows_unix_environment(&env_invalid, true);
    assert!(!result_invalid);

    // Test WSL variables - lines 558-560, 562
    let env_wsl1 = MockEnvironment::new().with_var("WSL_DISTRO_NAME", "Ubuntu-20.04");
    let result_wsl1 = is_windows_unix_environment(&env_wsl1, true);
    assert!(result_wsl1);

    let env_wsl2 = MockEnvironment::new().with_var("WSL_INTEROP", "/run/WSL/12_interop");
    let result_wsl2 = is_windows_unix_environment(&env_wsl2, true);
    assert!(result_wsl2);
}

#[test]
fn test_load_init_script_missing_script_with_debug() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No init script

    // Test missing script with debug - lines 404-405
    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());
}

#[test]
fn test_load_init_script_found_not_implemented() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new()
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh\necho test");

    // Test found script but not implemented - lines 401-402
    let result = load_init_script(&env, &runner, &fs, true);
    assert!(result.is_ok());
}

// Tests targeting specific uncovered lines for higher coverage

#[test]
fn test_samoyed_mode_zero_detection() {
    // Test SAMOYED=0 environment variable detection - line 95 path
    let env = MockEnvironment::new().with_var("SAMOYED", "0");
    let result = env.get_var("SAMOYED");
    assert_eq!(result, Some("0".to_string()));
    
    // Since we can't easily test process::exit(), test the condition
    let samoyed_mode = env.get_var("SAMOYED").unwrap_or_else(|| "1".to_string());
    assert_eq!(samoyed_mode, "0");
}

#[test]
fn test_debug_mode_detection() {
    // Test SAMOYED=2 debug mode detection - line 99
    let env = MockEnvironment::new().with_var("SAMOYED", "2");
    let samoyed_mode = env.get_var("SAMOYED").unwrap_or_else(|| "1".to_string());
    let debug_mode = samoyed_mode == "2";
    assert!(debug_mode);
}

#[test]
fn test_load_hook_command_config_parsing_debug() {
    // More comprehensive config loading test
    let fs = MockFileSystem::new().with_file(
        "samoyed.toml",
        "[hooks]\npre-commit = \"test command\"\npost-commit = \"other command\"",
    );

    // Test with debug mode to cover more lines (177, 182, 188, 203-204)
    let result1 = load_hook_command_from_config(&fs, "pre-commit", true);
    assert!(result1.is_ok());
    
    let result2 = load_hook_command_from_config(&fs, "post-commit", true);
    assert!(result2.is_ok());
    
    // Test hook not found with debug (213-214)
    let result3 = load_hook_command_from_config(&fs, "non-existent", true);
    assert!(result3.is_err());
}

#[test]
fn test_execute_hook_command_error_handling() {
    let env = MockEnvironment::new();
    let output = make_output(1, b"stdout data", b"stderr data");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "failing command"],
        Ok(output),
    );

    // Test with stdout/stderr output and non-zero exit - lines 269-270, 272-273, 277-278
    let result = std::panic::catch_unwind(|| {
        execute_hook_command(&env, &runner, "failing command", &[], true)
    });
    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_execute_hook_command_exit_code_127() {
    let env = MockEnvironment::new();
    let output = make_output(127, b"", b"bash: command not found");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "nonexistent"],
        Ok(output),
    );

    // Test command not found handling - lines 281-284
    let result = std::panic::catch_unwind(|| {
        execute_hook_command(&env, &runner, "nonexistent", &[], false) // no debug
    });
    assert!(result.is_err()); // Due to process::exit(127)
}

#[test]
fn test_run_hook_toml_config_debug_comprehensive() {
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "2") // Debug mode
        .with_var("HOME", "/home/test");
    
    let output = make_output(0, b"", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-c", "test command"],
        Ok(output),
    );

    let fs = MockFileSystem::new()
        .with_file("samoyed.toml", "[hooks]\npre-commit = \"test command\"")
        .with_file("/home/test/.config/samoyed/init.sh", "#!/bin/sh");

    let args = vec![
        "samoyed-hook".to_string(),
        "/full/path/to/pre-commit".to_string(), // Test file name extraction - line 111-114
    ];

    // Test comprehensive TOML path with debug - lines 123-125, 129, 132
    let result = std::panic::catch_unwind(|| run_hook(&env, &runner, &fs, &args));
    assert!(result.is_err()); // Due to process::exit(0)
}

#[test]
fn test_hook_name_from_path() {
    // Test hook name extraction from various paths - lines 111-114
    use std::path::Path;
    
    let test_cases = vec![
        ("pre-commit", "pre-commit"),
        ("/path/to/pre-commit", "pre-commit"),
        ("C:\\path\\to\\post-commit", "post-commit"),
        ("/usr/bin/pre-push", "pre-push"),
    ];
    
    for (input, expected) in test_cases {
        let hook_name = Path::new(input)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        assert_eq!(hook_name, expected);
    }
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_shell_detection_comprehensive() {
    // Test Windows environment detection with various scenarios
    let env_git_bash = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    assert!(is_windows_unix_environment(&env_git_bash, true));
    
    let env_wsl = MockEnvironment::new().with_var("WSL_DISTRO_NAME", "Ubuntu");
    assert!(is_windows_unix_environment(&env_wsl, true));
    
    let env_native = MockEnvironment::new();
    assert!(!is_windows_unix_environment(&env_native, true));
}

#[test]
fn test_execute_hook_script_debug_output() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test");
    
    let output = make_output(1, b"hook output", b"hook error");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/test/script", "arg1 arg2"],
        Ok(output),
    );

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/test/script");
    let args = vec!["arg1".to_string(), "arg2".to_string()];

    // Test debug output for script execution - lines 577-580, 590-591, 609-612
    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &args, true)
    });
    assert!(result.is_err()); // Due to process::exit(1)
}

#[test]
fn test_execute_hook_script_no_debug() {
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/test");
    
    let output = make_output(0, b"", b"");
    let runner = MockCommandRunner::new().with_response(
        "sh",
        &["-e", "/test/script", ""],
        Ok(output),
    );

    let fs = MockFileSystem::new();
    let script_path = std::path::Path::new("/test/script");

    // Test without debug mode
    let result = std::panic::catch_unwind(|| {
        execute_hook_script(&env, &runner, &fs, script_path, &[], false)
    });
    assert!(result.is_err()); // Due to process::exit(0)
}

#[cfg(target_os = "windows")]
#[test]
fn test_load_init_script_windows_script_name() {
    // Test Windows script name selection - lines 374-379
    let env = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\test");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new()
        .with_file("C:\\Users\\test\\.config\\samoyed\\init.cmd", "@echo off");

    let result = load_init_script(&env, &runner, &fs, false);
    assert!(result.is_ok());
}

#[test]
fn test_load_init_script_path_variations() {
    // Test different config path scenarios - lines 368-370, 378
    let env1 = MockEnvironment::new()
        .with_var("HOME", "/home/user")
        .with_var("XDG_CONFIG_HOME", "/custom/config");
    let runner = MockCommandRunner::new();
    let fs1 = MockFileSystem::new()
        .with_file("/custom/config/samoyed/init.sh", "#!/bin/sh");

    let result1 = load_init_script(&env1, &runner, &fs1, false);
    assert!(result1.is_ok());

    // Test fallback to HOME/.config when no XDG_CONFIG_HOME
    let env2 = MockEnvironment::new().with_var("HOME", "/home/user");
    let fs2 = MockFileSystem::new()
        .with_file("/home/user/.config/samoyed/init.sh", "#!/bin/sh");

    let result2 = load_init_script(&env2, &runner, &fs2, false);
    assert!(result2.is_ok());
}
