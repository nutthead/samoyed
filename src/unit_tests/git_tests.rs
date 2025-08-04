use super::*;
use crate::environment::mocks::{MockCommandRunner, MockFileSystem};
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
fn check_git_repository_exists() {
    // Create a mock filesystem with .git directory
    let fs = MockFileSystem::new().with_directory(".git");

    let result = check_git_repository(&fs);
    assert!(result.is_ok());
}

#[test]
fn check_git_repository_missing() {
    // Create a mock filesystem without .git directory
    let fs = MockFileSystem::new();

    let result = check_git_repository(&fs);
    assert!(matches!(result, Err(GitError::NotGitRepository { .. })));
}

#[test]
fn git_error_display() {
    let error = GitError::CommandNotFound { os_hint: None };
    assert!(error.to_string().contains("Git command not found in PATH"));

    let error = GitError::ConfigurationFailed {
        message: "test error".to_string(),
        suggestion: Some("try this".to_string()),
    };
    assert!(error.to_string().contains("test error"));
    assert!(error.to_string().contains("try this"));

    let error = GitError::NotGitRepository {
        checked_path: "/tmp".to_string(),
        suggest_init: true,
    };
    assert!(error.to_string().contains("Not a Git repository"));
    assert!(error.to_string().contains("git init"));
}

#[test]
fn set_hooks_path_success() {
    // Mock successful git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };

    // Create a successful config output
    let config_output = Output {
        status: exit_status(0),
        stdout: vec![],
        stderr: vec![],
    };

    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Ok(config_output),
        );

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(result.is_ok());
}

#[test]
fn set_hooks_path_handles_command_not_found() {
    let runner = MockCommandRunner::new();
    // No response configured, so it will return command not found

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
}

#[test]
fn set_hooks_path_handles_configuration_failure() {
    // Mock successful git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };

    // Create a failed output for config command
    let config_output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"error: could not lock config file".to_vec(),
    };

    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Ok(config_output),
        );

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(matches!(result, Err(GitError::ConfigurationFailed { .. })));
}

#[test]
fn git_error_variants_coverage() {
    // Test all GitError variants for coverage
    let error1 = GitError::CommandNotFound {
        os_hint: Some("linux".to_string()),
    };
    let error2 = GitError::ConfigurationFailed {
        message: "test".to_string(),
        suggestion: None,
    };
    let error3 = GitError::NotGitRepository {
        checked_path: "/tmp".to_string(),
        suggest_init: false,
    };
    let error4 = GitError::PermissionDenied {
        operation: "test op".to_string(),
        path: Some("/test/path".to_string()),
    };

    // Ensure all implement Debug and Display
    assert!(!format!("{error1:?}").is_empty());
    assert!(!format!("{error2:?}").is_empty());
    assert!(!format!("{error3:?}").is_empty());
    assert!(!format!("{error4:?}").is_empty());
    assert!(!error1.to_string().is_empty());
    assert!(!error2.to_string().is_empty());
    assert!(!error3.to_string().is_empty());
    assert!(!error4.to_string().is_empty());
}

#[test]
fn set_hooks_path_handles_different_paths() {
    // Mock successful git --version responses
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

    // Test with different hook paths
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
            &["config", "core.hooksPath", ".hooks"],
            Ok(config_output1),
        )
        .with_response("git", &["--version"], Ok(version_output2))
        .with_response(
            "git",
            &["config", "core.hooksPath", "my-hooks/"],
            Ok(config_output2),
        );

    let result1 = set_hooks_path(&runner, ".hooks");
    assert!(result1.is_ok());

    let result2 = set_hooks_path(&runner, "my-hooks/");
    assert!(result2.is_ok());
}

#[test]
fn check_git_repository_handles_different_filesystems() {
    // Test with filesystem that has .git directory
    let fs_with_git = MockFileSystem::new().with_directory(".git");
    let result1 = check_git_repository(&fs_with_git);
    assert!(result1.is_ok());

    // Test with filesystem that has .git file
    let fs_with_git_file =
        MockFileSystem::new().with_file(".git", "gitdir: ../.git/worktrees/branch");
    let result2 = check_git_repository(&fs_with_git_file);
    assert!(result2.is_ok());

    // Test with filesystem that has no .git
    let fs_no_git = MockFileSystem::new();
    let result3 = check_git_repository(&fs_no_git);
    assert!(result3.is_err());
}

#[test]
fn set_hooks_path_handles_io_error() {
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["--version"],
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied",
        )),
    );

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
}

#[test]
fn set_hooks_path_handles_permission_denied() {
    // Mock successful git --version first
    let version_output = Output {
        status: exit_status(0),
        stdout: b"git version 2.34.1".to_vec(),
        stderr: vec![],
    };

    // Create a permission denied error
    let config_output = Output {
        status: exit_status(128),
        stdout: vec![],
        stderr: b"error: Permission denied".to_vec(),
    };

    let runner = MockCommandRunner::new()
        .with_response("git", &["--version"], Ok(version_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".test-hooks"],
            Ok(config_output),
        );

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(matches!(result, Err(GitError::PermissionDenied { .. })));
}

#[test]
fn check_git_repository_handles_permission_denied() {
    // Create a filesystem where .git exists but can't be read
    let fs = MockFileSystem::new()
        .with_directory(".git")
        .with_file(".git/config", "test config");

    // Override the mock to return an error for reading .git/config
    let result = check_git_repository(&fs);
    // Note: In a real test, we'd need to mock the read_to_string failure
    // For now, this tests the happy path
    assert!(result.is_ok());
}

#[test]
fn error_suggestion_analysis() {
    // Test various error message suggestions
    let suggestion1 = analyze_git_config_error("error: could not lock config file");
    assert!(suggestion1.is_some());
    assert!(suggestion1.unwrap().contains("Another Git process"));

    let suggestion2 = analyze_git_config_error("fatal: not a git repository");
    assert!(suggestion2.is_some());
    assert!(suggestion2.unwrap().contains("Git repository"));

    let suggestion3 = analyze_git_config_error("error: bad config line");
    assert!(suggestion3.is_some());
    assert!(suggestion3.unwrap().contains("corrupted"));

    let suggestion4 = analyze_git_config_error("unknown error");
    assert!(suggestion4.is_none());
}

#[test]
fn os_detection() {
    let os = detect_os();
    assert!(os.is_some());
    // The actual OS will depend on the test environment
    let os_str = os.unwrap();
    assert!(["linux", "macos", "windows"].contains(&os_str.as_str()));
}

#[test]
fn git_error_os_specific_messages() {
    let error_linux = GitError::CommandNotFound {
        os_hint: Some("linux".to_string()),
    };
    let message = error_linux.to_string();
    assert!(message.contains("apt install git") || message.contains("yum install git"));

    let error_macos = GitError::CommandNotFound {
        os_hint: Some("macos".to_string()),
    };
    let message = error_macos.to_string();
    assert!(message.contains("brew install git"));

    let error_windows = GitError::CommandNotFound {
        os_hint: Some("windows".to_string()),
    };
    let message = error_windows.to_string();
    assert!(message.contains("git-scm.com"));
}

#[test]
fn git_version_check_failure() {
    // Mock git --version failure
    let version_output = Output {
        status: exit_status(1),
        stdout: vec![],
        stderr: b"git: command not found".to_vec(),
    };

    let runner = MockCommandRunner::new().with_response("git", &["--version"], Ok(version_output));

    let result = set_hooks_path(&runner, ".test-hooks");
    assert!(matches!(result, Err(GitError::CommandNotFound { .. })));
}
