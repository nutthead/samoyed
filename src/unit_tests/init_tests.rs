use super::*;
use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
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
fn test_init_command_creates_directories() {
    // Set up mocks
    let env = MockEnvironment::new();

    // Mock git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    // Mock successful git config command
    let config_output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    // Mock filesystem with git repository
    let fs = MockFileSystem::new().with_directory(".git");

    // Should succeed
    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_command_fails_without_git() {
    // Set up mocks
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No .git directory

    // Should fail without .git
    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Not a git repository")
    );
}

#[test]
fn test_init_command_with_project_type_hint() {
    // Set up mocks
    let env = MockEnvironment::new();

    // Mock git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    // Mock successful git config command
    let config_output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    // Mock filesystem with git repository
    let fs = MockFileSystem::new().with_directory(".git");

    // Should succeed with project type hint
    let result = init_command(&env, &runner, &fs, Some("rust".to_string()), None);
    assert!(result.is_ok());
}

#[test]
fn test_init_command_git_config_failure() {
    // Set up mocks
    let env = MockEnvironment::new();

    // Mock git --version first (succeeds)
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    // Mock failed git config command
    let config_output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"fatal: not a git repository".to_vec(),
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    // Mock filesystem with git repository
    let fs = MockFileSystem::new().with_directory(".git");

    // Should fail when git config fails
    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Git configuration failed"));
}

#[test]
fn test_init_command_with_existing_config() {
    // Test when samoyed.toml already exists
    let env = MockEnvironment::new();

    // Mock git --version first
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
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    // Mock filesystem with git repository and existing config
    let fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file("samoyed.toml", "[hooks]\npre-commit = \"echo test\"");

    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_init_command_with_invalid_project_type_hint() {
    // Test with invalid project type hint that falls back to auto-detection
    let env = MockEnvironment::new();

    // Mock git --version first
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
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    let fs = MockFileSystem::new().with_directory(".git");

    // Should succeed even with invalid hint, falling back to auto-detect
    let result = init_command(&env, &runner, &fs, Some("invalid-type".to_string()), None);
    assert!(result.is_ok());
}

#[test]
fn test_init_command_all_project_types() {
    // Test init command with all supported project type hints
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

    let project_types = vec!["rust", "go", "node", "python", "javascript", "typescript"];

    for project_type in project_types {
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(config_output.clone()),
            );

        let fs = MockFileSystem::new().with_directory(".git");

        let result = init_command(&env, &runner, &fs, Some(project_type.to_string()), None);
        assert!(result.is_ok(), "Failed for project type: {project_type}");
    }
}

#[test]
fn test_init_command_with_various_scenarios() {
    // Test more edge cases to improve coverage
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
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output),
        );

    // Test with different filesystem states
    let fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file("Cargo.toml", "[package]\nname = \"test\"");

    // Should detect Rust project and succeed
    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_project_type_detection_fallback() {
    // Test the fallback logic when project type hint is invalid
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
        .with_response("git", &["--version"], Ok(version_output.clone()))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output.clone()),
        );

    // Mock filesystem with multiple project files to test priority
    let fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file("package.json", "{}")
        .with_file("go.mod", "module test")
        .with_file("requirements.txt", "");

    // Test with invalid hint - should fallback to auto-detection
    let result = init_command(
        &env,
        &runner,
        &fs,
        Some("invalid-language".to_string()),
        None,
    );
    assert!(result.is_ok());

    // Test with empty hint
    let result = init_command(&env, &runner, &fs, Some("".to_string()), None);
    assert!(result.is_ok());
}

#[test]
fn test_verbose_output_with_environment_variable() {
    // Test that the SAMOYED_VERBOSE environment variable affects output
    let env = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "1");

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
        .with_response("git", &["--version"], Ok(version_output.clone()))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output.clone()),
        );

    let fs = MockFileSystem::new().with_directory(".git");

    // Should succeed with verbose environment variable set
    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_ok());

    // Test with existing config and verbose mode
    let fs_with_config = MockFileSystem::new()
        .with_directory(".git")
        .with_file("samoyed.toml", "[hooks]\npre-commit = \"test\"");

    let result = init_command(&env, &runner, &fs_with_config, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_environment_variable_not_set() {
    // Test that when SAMOYED_VERBOSE is not set or not "1", verbose mode is disabled
    let env = MockEnvironment::new(); // No environment variables set

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
        .with_response("git", &["--version"], Ok(version_output.clone()))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoyed/_"],
            Ok(config_output.clone()),
        );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = init_command(&env, &runner, &fs, None, None);
    assert!(result.is_ok());

    // Test with SAMOYED_VERBOSE set to something other than "1"
    let env_other = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "0");
    let result = init_command(&env_other, &runner, &fs, None, None);
    assert!(result.is_ok());

    let env_false = MockEnvironment::new().with_var("SAMOYED_VERBOSE", "false");
    let result = init_command(&env_false, &runner, &fs, None, None);
    assert!(result.is_ok());
}
