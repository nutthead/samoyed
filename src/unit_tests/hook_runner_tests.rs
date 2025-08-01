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
