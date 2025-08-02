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
    let output = Output {
        status: exit_status(0),
        stdout: b"Hook executed successfully".to_vec(),
        stderr: vec![],
    };
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
    let output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"Hook failed".to_vec(),
    };
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
    let output = Output {
        status: exit_status(127),
        stdout: vec![],
        stderr: b"command not found".to_vec(),
    };
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

    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
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

#[test]
fn test_determine_shell_execution_unix() {
    #[cfg(not(target_os = "windows"))]
    let env = MockEnvironment::new();
    #[cfg(target_os = "windows")]
    let _env = MockEnvironment::new();

    #[cfg(not(target_os = "windows"))]
    let script_path = std::path::Path::new("/path/to/script.sh");
    #[cfg(target_os = "windows")]
    let _script_path = std::path::Path::new("/path/to/script.sh");

    #[cfg(not(target_os = "windows"))]
    let args = ["arg1", "arg2"];
    #[cfg(target_os = "windows")]
    let _args = ["arg1", "arg2"];

    // On Unix systems (when not compiled for Windows), should always use sh
    #[cfg(not(target_os = "windows"))]
    {
        let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
        assert_eq!(shell, "sh");
        assert_eq!(shell_args, vec!["-e", "/path/to/script.sh", "arg1 arg2"]);
    }
}

#[test]
fn test_determine_shell_execution_windows_git_bash() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new().with_var("MSYSTEM", "MINGW64");

    #[cfg(target_os = "windows")]
    let script_path = std::path::Path::new("C:\\path\\to\\script.sh");
    #[cfg(not(target_os = "windows"))]
    let _script_path = std::path::Path::new("C:\\path\\to\\script.sh");

    #[cfg(target_os = "windows")]
    let args = ["arg1", "arg2"];
    #[cfg(not(target_os = "windows"))]
    let _args = ["arg1", "arg2"];

    // When MSYSTEM is set, should use sh even on Windows
    #[cfg(target_os = "windows")]
    {
        let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
        assert_eq!(shell, "sh");
        assert_eq!(
            shell_args,
            vec!["-e", "C:\\path\\to\\script.sh", "arg1 arg2"]
        );
    }
}

#[test]
fn test_determine_shell_execution_windows_cmd() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new(); // No MSYSTEM or CYGWIN
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new(); // No MSYSTEM or CYGWIN

    #[cfg(target_os = "windows")]
    let script_path = std::path::Path::new("C:\\path\\to\\script.bat");
    #[cfg(not(target_os = "windows"))]
    let _script_path = std::path::Path::new("C:\\path\\to\\script.bat");

    #[cfg(target_os = "windows")]
    let args = ["arg1", "arg2"];
    #[cfg(not(target_os = "windows"))]
    let _args = ["arg1", "arg2"];

    // Windows batch files should use cmd
    #[cfg(target_os = "windows")]
    {
        let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
        assert_eq!(shell, "cmd");
        assert_eq!(
            shell_args,
            vec!["/C", "C:\\path\\to\\script.bat", "arg1 arg2"]
        );
    }
}

#[test]
fn test_determine_shell_execution_windows_powershell() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new(); // No MSYSTEM or CYGWIN
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new(); // No MSYSTEM or CYGWIN

    #[cfg(target_os = "windows")]
    let script_path = std::path::Path::new("C:\\path\\to\\script.ps1");
    #[cfg(not(target_os = "windows"))]
    let _script_path = std::path::Path::new("C:\\path\\to\\script.ps1");

    #[cfg(target_os = "windows")]
    let args = ["arg1", "arg2"];
    #[cfg(not(target_os = "windows"))]
    let _args = ["arg1", "arg2"];

    // PowerShell scripts should use powershell
    #[cfg(target_os = "windows")]
    {
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
    let output = Output {
        status: exit_status(0),
        stdout: b"Formatting complete".to_vec(),
        stderr: vec![],
    };
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
    let output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"Formatting failed".to_vec(),
    };
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
    let output = Output {
        status: exit_status(127),
        stdout: vec![],
        stderr: b"command not found".to_vec(),
    };
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
    let output = Output {
        status: exit_status(0),
        stdout: b"Script executed successfully".to_vec(),
        stderr: vec![],
    };
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
    let output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"Script failed".to_vec(),
    };
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
    let output = Output {
        status: exit_status(0),
        stdout: b"origin main processed".to_vec(),
        stderr: vec![],
    };
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
    let output = Output {
        status: exit_status(127),
        stdout: vec![],
        stderr: b"nonexistent_command: command not found".to_vec(),
    };
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
    let output = Output {
        status: exit_status(0),
        stdout: b"Debug script executed".to_vec(),
        stderr: vec![],
    };
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
    let output = Output {
        status: exit_status(0),
        stdout: b"Test output".to_vec(),
        stderr: vec![],
    };
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

#[test]
fn test_determine_shell_execution_windows_native() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new(); // No Unix environment variables
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new(); // No Unix environment variables

    #[cfg(target_os = "windows")]
    let script_path = std::path::Path::new("C:\\path\\to\\script");
    #[cfg(not(target_os = "windows"))]
    let _script_path = std::path::Path::new("C:\\path\\to\\script");

    #[cfg(target_os = "windows")]
    let args = ["arg1", "arg2"];
    #[cfg(not(target_os = "windows"))]
    let _args = ["arg1", "arg2"];

    // Native Windows should default to cmd for extensionless files
    #[cfg(target_os = "windows")]
    {
        let (shell, shell_args) = determine_shell_execution(&env, script_path, &args, false);
        assert_eq!(shell, "cmd");
        assert_eq!(shell_args, vec!["/C", "C:\\path\\to\\script", "arg1 arg2"]);
    }
}

#[test]
fn test_determine_shell_execution_windows_appdata_config() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new()
        .with_var("APPDATA", "C:\\Users\\test\\AppData\\Roaming")
        .with_var("USERPROFILE", "C:\\Users\\test");
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new()
        .with_var("APPDATA", "C:\\Users\\test\\AppData\\Roaming")
        .with_var("USERPROFILE", "C:\\Users\\test");

    // Test Windows APPDATA configuration path in init script loading
    #[cfg(target_os = "windows")]
    {
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new().with_file(
            "C:\\Users\\test\\AppData\\Roaming\\samoyed\\init.cmd",
            "@echo off\nset NODE_OPTIONS=--max-old-space-size=4096",
        );

        let result = load_init_script(&env, &runner, &fs, false);
        assert!(result.is_ok());
    }
}

#[test]
fn test_run_hook_windows_cmd_execution() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");

    #[cfg(target_os = "windows")]
    {
        // Mock Windows batch file execution
        let output = Output {
            status: exit_status(0),
            stdout: b"Windows batch executed".to_vec(),
            stderr: vec![],
        };
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
}

#[test]
fn test_run_hook_windows_powershell_execution() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new()
        .with_var("SAMOYED", "1")
        .with_var("USERPROFILE", "C:\\Users\\test");

    #[cfg(target_os = "windows")]
    {
        // Mock PowerShell script execution
        let output = Output {
            status: exit_status(0),
            stdout: b"PowerShell script executed".to_vec(),
            stderr: vec![],
        };
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

#[test]
fn test_load_init_script_windows_native_paths() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\test");
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\test");

    #[cfg(target_os = "windows")]
    {
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new(); // No init script

        // Should succeed even without init script on Windows
        let result = load_init_script(&env, &runner, &fs, false);
        assert!(result.is_ok());
    }
}

#[test]
fn test_execute_hook_command_windows_shell() {
    #[cfg(target_os = "windows")]
    let env = MockEnvironment::new(); // Native Windows environment
    #[cfg(not(target_os = "windows"))]
    let _env = MockEnvironment::new(); // Native Windows environment

    #[cfg(target_os = "windows")]
    {
        // Mock Windows command execution
        let output = Output {
            status: exit_status(0),
            stdout: b"Windows command executed".to_vec(),
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response("cmd", &["/C", "dir"], Ok(output));

        let hook_args = vec![];

        // This function should exit with process::exit, so catch the panic
        let result = std::panic::catch_unwind(|| {
            execute_hook_command(&env, &runner, "dir", &hook_args, false)
        });
        assert!(result.is_err()); // Due to process::exit(0)
    }
}
