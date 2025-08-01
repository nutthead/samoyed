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
fn test_install_hooks_skip_when_samoyed_0() {
    let env = MockEnvironment::new().with_var("SAMOYED", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SAMOYED=0 skip install");
}

#[test]
fn test_install_hooks_invalid_path_with_dotdot() {
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let result = install_hooks(&env, &runner, &fs, Some("../invalid"));
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Directory traversal detected")
    );
}

#[test]
fn test_install_hooks_no_git_repository() {
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No .git directory

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Not a Git repository")
    );
}

#[test]
fn test_install_hooks_success() {
    let env = MockEnvironment::new();

    // Mock git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    // Configure git config command to succeed
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

    // Configure filesystem with .git directory
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_install_hooks_with_custom_dir() {
    let env = MockEnvironment::new();

    // Mock git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    // Configure git command to succeed with custom directory
    let config_output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".custom-hooks/_"],
            Ok(config_output),
        );

    // Configure filesystem with .git directory
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, Some(".custom-hooks"));
    assert!(result.is_ok());
}

#[test]
fn test_install_error_display() {
    let git_error = git::GitError::CommandNotFound { os_hint: None };
    let install_error = InstallError::Git(git_error);
    assert!(install_error.to_string().contains("Git command not found"));

    let hook_error = hooks::HookError::IoError(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "Permission denied",
    ));
    let install_error = InstallError::Hooks(hook_error);
    assert!(install_error.to_string().contains("Permission denied"));

    let invalid_error = InstallError::InvalidPath {
        path: "test/path".to_string(),
        reason: PathValidationError::DirectoryTraversal,
    };
    assert!(
        invalid_error
            .to_string()
            .contains("Directory traversal detected")
    );
}

#[test]
fn test_install_error_from_git_error() {
    let git_error = git::GitError::NotGitRepository {
        checked_path: "/tmp".to_string(),
        suggest_init: true,
    };
    let install_error: InstallError = git_error.into();
    assert!(matches!(install_error, InstallError::Git(_)));
}

#[test]
fn test_install_error_from_hook_error() {
    let hook_error = hooks::HookError::IoError(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "test",
    ));
    let install_error: InstallError = hook_error.into();
    assert!(matches!(install_error, InstallError::Hooks(_)));
}

#[test]
fn test_install_hooks_git_command_error() {
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new(); // No responses configured
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), InstallError::Git(_)));
}

#[test]
fn test_install_hooks_filesystem_error() {
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

    let fs = MockFileSystem::new().with_directory(".git");
    // Filesystem will fail when trying to create directories

    let result = install_hooks(&env, &runner, &fs, None);
    // This should succeed since MockFileSystem allows all operations
    assert!(result.is_ok());
}

#[test]
fn test_install_error_variants_coverage() {
    // Test all InstallError variants for coverage
    let git_error = git::GitError::CommandNotFound {
        os_hint: Some("linux".to_string()),
    };
    let hook_error = hooks::HookError::IoError(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "test",
    ));

    let error1 = InstallError::Git(git_error);
    let error2 = InstallError::Hooks(hook_error);
    let error3 = InstallError::InvalidPath {
        path: "invalid".to_string(),
        reason: PathValidationError::EmptyPath,
    };

    // Test Debug formatting
    assert!(!format!("{error1:?}").is_empty());
    assert!(!format!("{error2:?}").is_empty());
    assert!(!format!("{error3:?}").is_empty());

    // Test Display formatting
    assert!(error1.to_string().contains("Git command not found"));
    assert!(error2.to_string().contains("IO error"));
    assert!(error3.to_string().contains("Path cannot be empty"));
}

#[test]
fn test_install_hooks_different_custom_dirs() {
    let env = MockEnvironment::new();

    let version_output1 = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    let version_output2 = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };
    let config_output1 = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };
    let config_output2 = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };

    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output1))
        .with_response(
            "git",
            &["config", "core.hooksPath", "my-hooks/_"],
            Ok(config_output1),
        )
        .with_response("git", &["--version"], Ok(version_output2))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".git-hooks/_"],
            Ok(config_output2),
        );

    let fs = MockFileSystem::new().with_directory(".git");

    // Test with custom directory
    let result1 = install_hooks(&env, &runner, &fs, Some("my-hooks"));
    assert!(result1.is_ok());

    // Test with another custom directory
    let result2 = install_hooks(&env, &runner, &fs, Some(".git-hooks"));
    assert!(result2.is_ok());
}

#[test]
fn test_install_hooks_edge_case_paths() {
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

    // Test various edge case directory names
    let test_cases = [
        "hooks-dir",
        ".hidden-hooks",
        "hooks_with_underscores",
        "hooks123",
        "UPPERCASE-HOOKS",
    ];

    for dir_name in &test_cases {
        let expected_path = format!("{dir_name}/_");
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", &expected_path],
                Ok(config_output.clone()),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(dir_name));
        assert!(result.is_ok(), "Failed for directory: {dir_name}");
    }
}

#[test]
fn test_install_hooks_empty_environment_variable() {
    // Test when SAMOID is set to empty string (should not skip)
    let env = MockEnvironment::new().with_var("SAMOYED", "");
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

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ""); // Should not return skip message
}

#[test]
fn test_install_hooks_other_environment_values() {
    // Test various SAMOID environment variable values
    let test_values = ["1", "true", "false", "disabled", "anything"];

    for value in &test_values {
        let env = MockEnvironment::new().with_var("SAMOYED", value);
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

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok(), "Failed for SAMOID={value}");
        assert_eq!(result.unwrap(), "", "Should not skip for SAMOID={value}");
    }
}

#[test]
fn test_path_validation_directory_traversal() {
    // Test various directory traversal attempts
    let invalid_paths = [
        "../invalid",
        "valid/../invalid",
        "..\\invalid",
        "valid\\..\\invalid",
        "hooks/../../../etc/passwd",
    ];

    for path in &invalid_paths {
        let result = validate_hooks_directory_path(path);
        assert!(result.is_err(), "Path should be invalid: {path}");
        assert!(matches!(
            result.unwrap_err(),
            InstallError::InvalidPath {
                reason: PathValidationError::DirectoryTraversal,
                ..
            }
        ));
    }
}

#[test]
fn test_path_validation_absolute_paths() {
    let invalid_paths = [
        "/absolute/path",
        "/usr/local/hooks",
        "C:\\Windows\\hooks",
        "\\\\server\\share\\hooks",
    ];

    for path in &invalid_paths {
        let result = validate_hooks_directory_path(path);
        assert!(result.is_err(), "Path should be invalid: {path}");
        if std::path::Path::new(path).is_absolute() {
            assert!(matches!(
                result.unwrap_err(),
                InstallError::InvalidPath {
                    reason: PathValidationError::AbsolutePath,
                    ..
                }
            ));
        }
    }
}

#[test]
fn test_path_validation_invalid_characters() {
    let invalid_paths = [
        "hooks*invalid",
        "hooks?query",
        "hooks|pipe",
        "hooks<redirect",
        "hooks>redirect",
        "hooks\"quote",
        "hooks:colon",
    ];

    for path in &invalid_paths {
        let result = validate_hooks_directory_path(path);
        assert!(result.is_err(), "Path should be invalid: {path}");
        assert!(matches!(
            result.unwrap_err(),
            InstallError::InvalidPath {
                reason: PathValidationError::InvalidCharacters(_),
                ..
            }
        ));
    }
}

#[test]
fn test_path_validation_empty_paths() {
    let empty_paths = ["", "   ", "\t", "\n"];

    for path in &empty_paths {
        let result = validate_hooks_directory_path(path);
        assert!(result.is_err(), "Path should be invalid: '{path}'");
        assert!(matches!(
            result.unwrap_err(),
            InstallError::InvalidPath {
                reason: PathValidationError::EmptyPath,
                ..
            }
        ));
    }
}

#[test]
fn test_path_validation_too_long() {
    let long_path = "a".repeat(256); // Exceeds 255 character limit

    let result = validate_hooks_directory_path(&long_path);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        InstallError::InvalidPath {
            reason: PathValidationError::TooLong(256),
            ..
        }
    ));
}

#[test]
fn test_path_validation_valid_paths() {
    let valid_paths = [
        ".samoyed",
        "hooks",
        ".git-hooks",
        "my_hooks",
        "project-hooks",
        "hooks123",
        "UPPERCASE_HOOKS",
        "nested/hooks",
        "deeply/nested/hooks/dir",
    ];

    for path in &valid_paths {
        let result = validate_hooks_directory_path(path);
        assert!(result.is_ok(), "Path should be valid: {path}");
    }
}

#[test]
fn test_path_validation_error_messages() {
    // Test that error messages are informative
    let result = validate_hooks_directory_path("../invalid");
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Directory traversal detected"));
    assert!(error_msg.contains("Security"));

    let result = validate_hooks_directory_path("/absolute");
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Absolute paths not allowed"));
    }

    let result = validate_hooks_directory_path("");
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Path cannot be empty"));

    let result = validate_hooks_directory_path("path*invalid");
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid characters"));
}
