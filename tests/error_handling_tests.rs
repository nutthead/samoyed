//! Error handling integration tests
//!
//! Tests for various error scenarios including Git failures, filesystem errors,
//! and edge cases to ensure proper error handling and recovery.

use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
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
fn test_git_command_failure_scenarios() {
    // Test various Git command failure scenarios
    let env = MockEnvironment::new();

    // Scenario 1: Git not found
    let runner_not_found = MockCommandRunner::new();
    // Don't configure any responses, so git command will fail with "Command not found"
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner_not_found, &fs, None);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Git command not found")
    );

    // Scenario 2: Git config fails
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    let config_fail_output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"fatal: could not lock config file\n".to_vec(),
    };
    let runner_config_fail = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_fail_output),
        );

    let result = install_hooks(&env, &runner_config_fail, &fs, None);
    assert!(result.is_err());
}

#[test]
fn test_filesystem_error_scenarios() {
    let env = MockEnvironment::new();
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    let config_output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );

    // Test with directory creation failure
    // MockFileSystem doesn't support error injection, so we can't directly test this scenario
    // In a real implementation, we would test with filesystem permissions or mocking at a lower level

    // Instead, test with missing .git directory which will cause a different error
    let fs_no_git = MockFileSystem::new(); // No .git directory

    let result = install_hooks(&env, &runner, &fs_no_git, None);
    assert!(result.is_err());
    // This will fail at the Git repository check, not filesystem creation
}

#[test]
fn test_edge_case_paths() {
    // Test various edge case paths
    let edge_cases = vec![
        ("..", "Directory traversal"),
        ("../hooks", "Directory traversal"),
        ("./foo/../..", "Directory traversal"),
        ("/absolute/path", "Absolute path"),
        ("", "empty"),
        ("path/with spaces/hooks", "spaces"),
        (
            "very/deeply/nested/path/to/hooks/directory/structure",
            "deep nesting",
        ),
    ];

    for (path, description) in edge_cases {
        let env = MockEnvironment::new();
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &MockCommandRunner::new(), &fs, Some(path));

        match description {
            "Directory traversal" => {
                assert!(result.is_err(), "Should fail for path with ..: {path}");
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("Directory traversal"),
                    "Error should mention directory traversal"
                );
            }
            "Absolute path" => {
                assert!(result.is_err(), "Should fail for absolute path: {path}");
                assert!(
                    result.unwrap_err().to_string().contains("Absolute path"),
                    "Error should mention absolute path"
                );
            }
            "empty" => {
                assert!(result.is_err(), "Should fail for empty path");
                assert!(
                    result.unwrap_err().to_string().contains("empty"),
                    "Error should mention empty path"
                );
            }
            _ => {
                // Other paths might be valid depending on implementation
                // Just verify the function handles them without panicking
                let _ = result;
            }
        }
    }
}

#[test]
fn test_error_message_quality() {
    // Test that error messages are helpful and specific
    let test_cases = vec![
        (
            MockCommandRunner::new(), // No responses configured, will return "Command not found"
            "Git command not found",
            "install Git",
        ),
        (
            MockCommandRunner::new()
                .with_response(
                    "git",
                    &["--version"],
                    Ok(Output {
                        status: exit_status(0),
                        stdout: b"git version 2.34.1".to_vec(),
                        stderr: vec![],
                    }),
                )
                .with_response(
                    "git",
                    &["config", "core.hooksPath", ".samoid/_"],
                    Ok(Output {
                        status: exit_status(128),
                        stdout: vec![],
                        stderr: b"fatal: not in a git directory\n".to_vec(),
                    }),
                ),
            "Git configuration failed",
            "not in a git directory",
        ),
    ];

    for (runner, expected_error, expected_hint) in test_cases {
        let env = MockEnvironment::new();
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains(expected_error),
            "Error should contain '{expected_error}', got: {error_message}"
        );

        // Some errors should provide helpful hints
        if !expected_hint.is_empty() {
            assert!(
                error_message
                    .to_lowercase()
                    .contains(&expected_hint.to_lowercase()),
                "Error should contain hint '{expected_hint}', got: {error_message}"
            );
        }
    }
}

#[test]
fn test_concurrent_installation_simulation() {
    // Simulate concurrent installation attempts
    let env = MockEnvironment::new();
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };

    // First installation succeeds
    let config_output1 = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner1 = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output.clone()))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output1),
        );
    let fs1 = MockFileSystem::new().with_directory(".git");

    let result1 = install_hooks(&env, &runner1, &fs1, None);
    assert!(result1.is_ok());

    // Second "concurrent" installation with lock error
    let config_output2 = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"error: could not lock config file .git/config: File exists\n".to_vec(),
    };
    let runner2 = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output2),
        );
    let fs2 = MockFileSystem::new().with_directory(".git");

    let result2 = install_hooks(&env, &runner2, &fs2, None);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("config"));
}
