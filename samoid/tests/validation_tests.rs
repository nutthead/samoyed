//! Validation and verification integration tests
//!
//! Tests for validating hook content, environment variables, and system state
//! after installation to ensure everything is correctly configured.

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
fn test_environment_variable_scenarios() {
    // Test various environment variable configurations
    let test_scenarios = vec![
        (vec![("SAMOID", "0")], "skip mode"),
        (vec![("SAMOID", "1")], "normal mode"),
        (vec![("SAMOID", "2")], "debug mode"),
        (vec![("HOME", "/custom/home")], "custom home"),
        (vec![("CI", "true"), ("SAMOID", "0")], "CI environment"),
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
                &["config", "core.hooksPath", ".samoid/_"],
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
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Verify hook runner content
    let runner_path = std::path::Path::new(".samoid/_/h");
    let runner_content = fs.read_to_string(runner_path).expect("Runner should exist");
    assert!(runner_content.starts_with("#!/usr/bin/env sh"));
    assert!(runner_content.contains("samoid"));

    // Verify gitignore
    let gitignore_path = std::path::Path::new(".samoid/_/.gitignore");
    let gitignore_content = fs
        .read_to_string(gitignore_path)
        .expect("Gitignore should exist");
    assert_eq!(gitignore_content, "*");

    // Verify hook content
    let standard_hooks = ["pre-commit", "commit-msg", "pre-push"];
    for hook in &standard_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        let hook_content = fs
            .read_to_string(&hook_path)
            .expect("Hook should exist and be readable");
        
        // Verify shebang
        assert!(
            hook_content.starts_with("#!/usr/bin/env sh"),
            "Hook {hook} should start with proper shebang"
        );
        
        // Verify it references the runner
        assert!(
            hook_content.contains(". \"$(dirname -- \"$0\")/h\""),
            "Hook {hook} should source the runner script"
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
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Complete list of standard Git hooks
    let all_git_hooks = [
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

    for hook in &all_git_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(
            fs.exists(&hook_path),
            "Git hook '{hook}' should be created"
        );
        
        // Verify each hook is executable (has content)
        let content = fs
            .read_to_string(&hook_path)
            .expect(&format!("Hook {hook} should be readable"));
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
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );

    // Create filesystem with many existing files
    let mut fs = MockFileSystem::new().with_directory(".git");
    
    // Add 100 dummy files to simulate a large project
    for i in 0..100 {
        fs = fs.with_file(&format!("src/file{i}.rs"), "// dummy content");
    }

    // Installation should still work efficiently
    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok(), "Should handle large number of files");

    // Verify hooks were still created properly
    assert!(fs.exists(std::path::Path::new(".samoid/_/pre-commit")));
}