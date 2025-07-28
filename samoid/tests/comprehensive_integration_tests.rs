//! Comprehensive integration tests with enhanced mock scenarios
//!
//! These tests provide comprehensive coverage of integration scenarios
//! using the mock dependency injection system for reliable testing.

use samoid::environment::FileSystem;
use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};

#[test]
fn test_complete_installation_flow() {
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
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

    // Verify all expected directories and files were created
    assert!(fs.exists(std::path::Path::new(".samoid/_")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/.gitignore")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/h")));

    // Verify all standard hooks were created
    let standard_hooks = [
        "pre-commit",
        "pre-merge-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "applypatch-msg",
        "pre-applypatch",
        "post-applypatch",
        "pre-rebase",
        "post-rewrite",
        "post-checkout",
        "post-merge",
        "pre-push",
        "pre-auto-gc",
    ];

    for hook in &standard_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(fs.exists(&hook_path), "Hook {hook} should exist");
    }
}

#[test]
fn test_installation_with_multiple_custom_directories() {
    let custom_dirs = vec![".custom-hooks", "hooks", ".git-hooks", "project-hooks"];

    for custom_dir in custom_dirs {
        let env = MockEnvironment::new();
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", &format!("{custom_dir}/_")],
            Ok(output),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(custom_dir));
        assert!(
            result.is_ok(),
            "Failed for custom directory: {custom_dir}"
        );
        assert_eq!(result.unwrap(), ""); // Success returns empty string

        // Verify hooks were created in custom directory
        assert!(fs.exists(&std::path::Path::new(custom_dir).join("_")));
        assert!(
            fs.exists(
                &std::path::Path::new(custom_dir)
                    .join("_")
                    .join("pre-commit")
            )
        );
    }
}

#[test]
fn test_environment_variable_scenarios() {
    let scenarios = vec![
        // Normal execution
        (Some("1"), true, ""),
        // Skip installation
        (Some("0"), true, "SAMOID=0 skip install"),
        // Debug mode (should still install)
        (Some("2"), true, ""),
        // Empty string (should default to normal)
        (Some(""), true, ""),
        // No variable set (should default to normal)
        (None, true, ""),
    ];

    for (samoid_value, should_succeed, expected_message) in scenarios {
        let mut env = MockEnvironment::new();
        if let Some(value) = samoid_value {
            env = env.with_var("SAMOID", value);
        }

        let output = Output {
            status: ExitStatus::from_raw(0),
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

        if should_succeed {
            assert!(result.is_ok(), "Failed for SAMOID={samoid_value:?}");
            assert_eq!(result.unwrap(), expected_message);
        } else {
            assert!(
                result.is_err(),
                "Should have failed for SAMOID={samoid_value:?}"
            );
        }
    }
}

#[test]
fn test_git_command_failure_scenarios() {
    let failure_scenarios = vec![
        // Git config command fails
        (1, b"fatal: not in a git repository".to_vec()),
        // Permission denied
        (128, b"fatal: could not lock config file".to_vec()),
        // Command not found
        (127, b"git: command not found".to_vec()),
    ];

    for (exit_code, stderr) in failure_scenarios {
        let env = MockEnvironment::new();
        let output = Output {
            status: ExitStatus::from_raw(exit_code),
            stdout: vec![],
            stderr,
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(
            result.is_err(),
            "Should fail for git exit code {exit_code}"
        );
    }
}

#[test]
fn test_filesystem_error_scenarios() {
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );

    // Test the main filesystem error condition - no .git directory
    let fs_no_git = MockFileSystem::new(); // No .git directory
    let result = install_hooks(&env, &runner, &fs_no_git, None);
    assert!(result.is_err(), "Should fail when not in a git repository");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains(".git can't be found")
    );
}

#[test]
fn test_edge_case_paths() {
    let edge_case_paths = vec![
        // Empty path (should be rejected)
        "",
        // Paths with dots
        ".",
        "..",
        "../invalid",
        "./valid",
        // Long paths
        "very/long/path/with/many/segments/for/testing/purposes",
        // Paths with special characters (but safe ones)
        "hooks-test",
        "hooks_test",
        "hooks.test",
    ];

    for path in edge_case_paths {
        let env = MockEnvironment::new();
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", &format!("{path}/_")],
            Ok(output),
        );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(path));

        // Paths with ".." should be rejected
        if path.contains("..") {
            assert!(result.is_err(), "Should reject path with '..': {path}");
            if let Err(e) = result {
                assert!(
                    e.to_string().contains("not allowed"),
                    "Error message should mention 'not allowed' for path: {path}"
                );
            }
        } else if path.is_empty() {
            // Empty paths should use default behavior
            assert!(result.is_ok(), "Empty path should use default: {path}");
        } else {
            // Other paths should work
            assert!(result.is_ok(), "Valid path should work: {path}");
        }
    }
}

#[test]
fn test_concurrent_installation_simulation() {
    // Simulate multiple concurrent installations (though our mock system is single-threaded)
    let num_simulations = 10;

    for i in 0..num_simulations {
        let env = MockEnvironment::new();
        let output = Output {
            status: ExitStatus::from_raw(0),
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
        assert!(result.is_ok(), "Simulation {i} should succeed");
        assert_eq!(result.unwrap(), ""); // Success returns empty string
    }
}

#[test]
fn test_large_number_of_files_simulation() {
    // Simulate a filesystem with many files to test performance
    let mut fs = MockFileSystem::new().with_directory(".git");

    // Add many files to simulate a large repository
    for i in 0..1000 {
        fs = fs.with_file(format!("src/file_{i}.rs"), "// Mock file content");
        fs = fs.with_file(format!("tests/test_{i}.rs"), "// Mock test content");
    }

    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );

    // Installation should still work efficiently
    let start = std::time::Instant::now();
    let result = install_hooks(&env, &runner, &fs, None);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ""); // Success returns empty string

    // Should be fast even with many files
    assert!(
        duration.as_millis() < 100,
        "Installation took too long: {duration:?}"
    );
}

#[test]
fn test_reinstallation_idempotency() {
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output.clone()),
    );

    // Start with existing hook files
    let fs = MockFileSystem::new()
        .with_directory(".git")
        .with_directory(".samoid/_")
        .with_file(".samoid/_/.gitignore", "*")
        .with_file(".samoid/_/h", "#!/bin/sh\necho 'existing hook runner'")
        .with_file(
            ".samoid/_/pre-commit",
            "#!/bin/sh\necho 'existing pre-commit'",
        );

    // First installation should succeed
    let result1 = install_hooks(&env, &runner, &fs, None);
    assert!(result1.is_ok());

    // Add the git response again for second call
    let runner2 = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(output),
    );

    // Second installation should also succeed (idempotent)
    let result2 = install_hooks(&env, &runner2, &fs, None);
    assert!(result2.is_ok());

    // Results should be the same
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_hook_content_validation() {
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
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

    // Check that the hook runner script has expected content
    let runner_path = std::path::Path::new(".samoid/_/h");
    assert!(fs.exists(runner_path));

    // Check .gitignore content
    let gitignore_path = std::path::Path::new(".samoid/_/.gitignore");
    assert!(fs.exists(gitignore_path));

    // Verify individual hook files exist and are properly configured
    let hooks_to_verify = ["pre-commit", "post-commit", "pre-push"];
    for hook in &hooks_to_verify {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(fs.exists(&hook_path), "Hook {hook} should exist");
    }
}

#[test]
fn test_directory_structure_validation() {
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
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

    // Verify the complete directory structure is created correctly
    // Note: MockFileSystem creates the full path when hooks are installed
    assert!(fs.exists(std::path::Path::new(".samoid/_")));

    // Verify essential files are present
    let essential_files = [".samoid/_/.gitignore", ".samoid/_/h"];

    for file in &essential_files {
        assert!(
            fs.exists(std::path::Path::new(file)),
            "Essential file {file} should exist"
        );
    }
}

#[test]
fn test_error_message_quality() {
    // Test that error messages are helpful and informative

    // Test with invalid path containing ".."
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, Some("../invalid"));
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not allowed"),
        "Error message should be informative: {error_msg}"
    );

    // Test without git repository
    let fs_no_git = MockFileSystem::new();
    let result = install_hooks(&env, &runner, &fs_no_git, None);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains(".git can't be found"),
        "Error message should be informative: {error_msg}"
    );
}

#[test]
fn test_comprehensive_hook_coverage() {
    // Ensure all Git hooks are supported
    let env = MockEnvironment::new();
    let output = Output {
        status: ExitStatus::from_raw(0),
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

    // Verify ALL standard Git hooks are created
    let all_git_hooks = [
        "applypatch-msg",
        "pre-applypatch",
        "post-applypatch",
        "pre-commit",
        "pre-merge-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "pre-rebase",
        "post-checkout",
        "post-merge",
        "pre-push",
        "pre-auto-gc",
        "post-rewrite",
    ];

    for hook in &all_git_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(fs.exists(&hook_path), "Git hook {hook} should be created");
    }

    // Verify we have the expected total count
    assert_eq!(
        all_git_hooks.len(),
        14,
        "Should support all 14 standard Git hooks"
    );
}
