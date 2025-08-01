//! Validation and verification integration tests
//!
//! Tests for validating hook content, environment variables, and system state
//! after installation to ensure everything is correctly configured.

use samoyed::environment::FileSystem;
use samoyed::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoyed::install_hooks;
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
fn test_environment_variable_scenarios() {
    // Test various environment variable configurations
    let test_scenarios = vec![
        (vec![("SAMOYED", "0")], "skip mode"),
        (vec![("SAMOYED", "1")], "normal mode"),
        (vec![("SAMOYED", "2")], "debug mode"),
        (vec![("HOME", "/custom/home")], "custom home"),
        (vec![("CI", "true"), ("SAMOYED", "0")], "CI environment"),
    ];

    for (vars, description) in test_scenarios {
        let mut env = MockEnvironment::new();
        for (key, value) in vars {
            env = env.with_var(key, value);
        }

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
        assert!(
            result.is_ok(),
            "Should handle environment scenario: {description}"
        );
    }
}

#[test]
fn test_hook_content_validation() {
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

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Verify that hook files were created
    let pre_commit_path = std::path::Path::new(".samoyed/_/pre-commit");
    let pre_commit_content = fs
        .read_to_string(pre_commit_path)
        .expect("pre-commit hook should exist");
    assert!(pre_commit_content.starts_with("#!/usr/bin/env sh"));
    assert!(pre_commit_content.contains("samoyed-hook"));

    // Verify gitignore
    let gitignore_path = std::path::Path::new(".samoyed/_/.gitignore");
    let gitignore_content = fs
        .read_to_string(gitignore_path)
        .expect("Gitignore should exist");
    assert_eq!(gitignore_content, "*");

    // Verify hook content
    let standard_hooks = ["pre-commit", "commit-msg", "pre-push"];
    for hook in &standard_hooks {
        let hook_path = std::path::Path::new(".samoyed/_").join(hook);
        let hook_content = fs
            .read_to_string(&hook_path)
            .expect("Hook should exist and be readable");

        // Verify shebang
        assert!(
            hook_content.starts_with("#!/usr/bin/env sh"),
            "Hook {hook} should start with proper shebang"
        );

        // Verify it references the samoyed-hook runner
        assert!(
            hook_content.contains("exec samoyed-hook"),
            "Hook {hook} should exec the samoyed-hook binary"
        );
    }
}

#[test]
fn test_comprehensive_hook_coverage() {
    // Ensure all Git hooks are created
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

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Complete list of standard Git hooks (matches STANDARD_HOOKS constant)
    let all_git_hooks = [
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

    for hook in &all_git_hooks {
        let hook_path = std::path::Path::new(".samoyed/_").join(hook);
        assert!(fs.exists(&hook_path), "Git hook '{hook}' should be created");

        // Verify each hook is executable (has content)
        let content = fs
            .read_to_string(&hook_path)
            .unwrap_or_else(|_| panic!("Hook {hook} should be readable"));
        assert!(!content.is_empty(), "Hook {hook} should have content");
    }
}

#[test]
fn test_large_number_of_files_simulation() {
    // Test with many pre-existing files to ensure performance
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

    // Create filesystem with many existing files
    let mut fs = MockFileSystem::new().with_directory(".git");

    // Add 100 dummy files to simulate a large project
    for i in 0..100 {
        fs = fs.with_file(format!("src/file{i}.rs"), "// dummy content");
    }

    // Installation should still work efficiently
    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok(), "Should handle large number of files");

    // Verify hooks were still created properly
    assert!(fs.exists(std::path::Path::new(".samoyed/_/pre-commit")));
}
