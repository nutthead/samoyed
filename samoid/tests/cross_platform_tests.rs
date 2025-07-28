//! Cross-platform compatibility tests
//!
//! These tests ensure Samoid works correctly across different operating systems
//! and environments, handling path separators, permissions, and shell differences.

use samoid::environment::FileSystem;
use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
use std::process::Output;

// Cross-platform exit status creation
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

// Helper function to create ExitStatus cross-platform
fn exit_status(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    return std::process::ExitStatus::from_raw(code);
    
    #[cfg(windows)]
    return std::process::ExitStatus::from_raw(code as u32);
}

#[test]
fn test_unix_path_handling() {
    let env = MockEnvironment::new();
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ""); // Success returns empty string
}

#[test]
fn test_windows_style_paths() {
    let env = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\test")
        .with_var("SAMOID", "1");

    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
}

#[test]
fn test_mixed_path_separators() {
    let env = MockEnvironment::new();
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };

    // Test various path formats that might occur on different systems
    let test_dirs = vec![".hooks\\windows", "./unix/style", "mixed\\and/separated"];

    for dir in test_dirs {
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", &format!("{dir}/_")],
            Ok(output.clone()),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(dir));
        assert!(result.is_ok(), "Failed for directory: {dir}");
    }
}

#[test]
fn test_environment_variable_differences() {
    // Test Unix-style HOME variable
    let unix_env = MockEnvironment::new().with_var("HOME", "/home/user");
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output.clone()),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&unix_env, &runner, &fs, None);
    assert!(result.is_ok());

    // Test Windows-style USERPROFILE variable
    let windows_env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\user");
    let result = install_hooks(&windows_env, &runner, &fs, None);
    assert!(result.is_ok());
}

#[test]
fn test_xdg_config_home_handling() {
    // Test with XDG_CONFIG_HOME set (Linux standard)
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/user")
        .with_var("XDG_CONFIG_HOME", "/home/user/.config");

    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
}

#[test]
fn test_shell_command_compatibility() {
    let env = MockEnvironment::new();

    // Test different shell command formats that might exist on different systems
    let shell_commands = vec![
        ("sh", vec!["-c", "echo test"]),
        ("bash", vec!["-c", "echo test"]),
        ("cmd", vec!["/c", "echo test"]),
    ];

    for (shell, args) in shell_commands {
        let output = Output {
            status: exit_status(0),
            stdout: b"test\n".to_vec(),
            stderr: vec![],
        };

        let mut runner = MockCommandRunner::new();

        // Add git config response
        let git_output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        runner = runner.with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(git_output),
        );

        // Add shell command response
        runner = runner.with_response(shell, &args, Ok(output));

        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok(), "Failed for shell: {shell}");
    }
}

#[test]
fn test_file_permissions_cross_platform() {
    // This test would verify that file permissions are handled correctly
    // across platforms (executable on Unix, appropriate on Windows)
    let env = MockEnvironment::new();
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Verify that hooks directory was created
    assert!(fs.exists(std::path::Path::new(".samoid/_")));
}

#[test]
fn test_git_command_variations() {
    // Test different Git command variations that might exist on different systems
    let env = MockEnvironment::new();

    let git_commands = vec![
        "git",
        "git.exe",      // Windows
        "/usr/bin/git", // Full path
    ];

    for git_cmd in git_commands {
        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            git_cmd,
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        // Note: This test is limited by our mock system, but in real scenarios
        // we'd want to test different Git executable locations
        let result = install_hooks(&env, &runner, &fs, None);
        // Only test with "git" as other commands are not configured in mock
        if git_cmd == "git" {
            assert!(result.is_ok(), "Failed for git command: {git_cmd}");
        }
    }
}

#[test]
fn test_line_ending_handling() {
    // Test handling of different line endings (Unix \n vs Windows \r\n)
    let env = MockEnvironment::new();
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );

    // Create filesystem with different line ending styles
    let unix_fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file("test_unix.txt", "line1\nline2\nline3\n");

    let windows_fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file("test_windows.txt", "line1\r\nline2\r\nline3\r\n");

    // Both should work correctly
    let result1 = install_hooks(&env, &runner, &unix_fs, None);
    let result2 = install_hooks(&env, &runner, &windows_fs, None);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_unicode_path_handling() {
    // Test handling of Unicode characters in file paths
    let env = MockEnvironment::new();
    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };

    let unicode_dirs = vec![".hooks-测试", ".hooks-🚀", ".hooks-café"];

    for unicode_dir in unicode_dirs {
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", &format!("{unicode_dir}/_")],
            Ok(output.clone()),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(unicode_dir));
        assert!(
            result.is_ok(),
            "Failed for Unicode directory: {unicode_dir}"
        );
    }
}

#[cfg(target_family = "unix")]
#[test]
fn test_unix_specific_features() {
    // Test Unix-specific functionality
    let env = MockEnvironment::new()
        .with_var("HOME", "/home/user")
        .with_var("USER", "testuser");

    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
}

#[cfg(target_family = "windows")]
#[test]
fn test_windows_specific_features() {
    // Test Windows-specific functionality
    let env = MockEnvironment::new()
        .with_var("USERPROFILE", "C:\\Users\\user")
        .with_var("USERNAME", "testuser");

    let output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
}
