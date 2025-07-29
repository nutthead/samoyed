//! Installation-related integration tests
//!
//! Tests for core installation functionality including Git configuration,
//! directory creation, and hook file generation.

use samoid::environment::FileSystem;
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
fn test_complete_installation_flow() {
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
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);

    assert!(result.is_ok());

    // Verify directory creation
    assert!(fs.exists(std::path::Path::new(".samoid/_")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/.gitignore")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/h")));

    // Verify gitignore content
    let gitignore_content = fs
        .read_to_string(std::path::Path::new(".samoid/_/.gitignore"))
        .unwrap();
    assert_eq!(gitignore_content, "*");

    // Verify runner script permissions (Unix only)
    #[cfg(unix)]
    {
        let runner_content = fs
            .read_to_string(std::path::Path::new(".samoid/_/h"))
            .unwrap();
        assert!(runner_content.starts_with("#!/usr/bin/env sh"));
    }

    // Verify all standard Git hooks were created
    let standard_hooks = [
        "pre-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "pre-push",
        "pre-rebase",
        "post-rewrite",
        "post-checkout",
        "post-merge",
        "pre-auto-gc",
        "post-update",
        "push-to-checkout",
        "pre-applypatch",
        "applypatch-msg",
    ];

    for hook in &standard_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(fs.exists(&hook_path), "Hook {hook} should be created");
    }
}

#[test]
fn test_installation_with_multiple_custom_directories() {
    // Test multiple custom directory scenarios
    let test_cases = vec![
        ("hooks", ".samoid/_"),
        (".custom-hooks", ".custom-hooks/_"),
        ("path/to/hooks", "path/to/hooks/_"),
    ];

    for (custom_dir, expected_path) in test_cases {
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
                &["config", "core.hooksPath", expected_path],
                Ok(config_output),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(custom_dir));
        assert!(
            result.is_ok(),
            "Should succeed for custom dir: {custom_dir}"
        );

        // Verify hooks were created in custom directory
        assert!(fs.exists(&std::path::Path::new(custom_dir).join("_")));
        let pre_commit_path = std::path::Path::new(custom_dir)
            .join("_")
            .join("pre-commit");
        assert!(
            fs.exists(&pre_commit_path),
            "pre-commit should exist in custom directory"
        );
    }
}

#[test]
fn test_reinstallation_idempotency() {
    // Test that reinstalling multiple times is safe
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

    for i in 0..3 {
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(config_output.clone()),
            );
        let fs = if i == 0 {
            MockFileSystem::new().with_directory(".git")
        } else {
            // Simulate existing installation
            MockFileSystem::new()
                .with_directory(".git")
                .with_directory(".samoid/_")
                .with_file(".samoid/_/.gitignore", "*")
                .with_file(".samoid/_/h", "#!/usr/bin/env sh\n")
                .with_file(".samoid/_/pre-commit", "hook content")
        };

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(
            result.is_ok(),
            "Installation {i} should succeed: {:?}",
            result
        );

        // Verify hooks still exist and are valid
        assert!(fs.exists(std::path::Path::new(".samoid/_")));
        assert!(fs.exists(std::path::Path::new(".samoid/_/pre-commit")));
    }
}

#[test]
fn test_directory_structure_validation() {
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
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Verify complete directory structure
    assert!(fs.exists(std::path::Path::new(".samoid/_")));

    // Check all required files exist
    let required_files = [".gitignore", "h"];
    for file in &required_files {
        let file_path = std::path::Path::new(".samoid/_").join(file);
        assert!(
            fs.exists(&file_path),
            "Required file {file} should exist in .samoid/_"
        );
    }
}
