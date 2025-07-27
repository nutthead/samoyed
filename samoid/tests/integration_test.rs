use samoid::environment::FileSystem;
use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};

#[test]
fn test_core_installation() {
    let env = MockEnvironment::new();

    // Mock successful git commands
    let init_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let config_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new()
        .with_response("git", &["init"], Ok(init_output))
        .with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(config_output),
        );

    // Mock filesystem with git repository
    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    // Verify the mock filesystem recorded the expected operations
    assert!(fs.exists(std::path::Path::new(".samoid/_")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/.gitignore")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/h")));
    assert!(fs.exists(std::path::Path::new(".samoid/_/pre-commit")));
}

#[test]
fn test_installation_with_custom_directory() {
    let env = MockEnvironment::new();

    let config_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".custom-hooks/_"],
        Ok(config_output),
    );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, Some(".custom-hooks"));
    assert!(result.is_ok());

    assert!(fs.exists(std::path::Path::new(".custom-hooks/_")));
    assert!(fs.exists(std::path::Path::new(".custom-hooks/_/.gitignore")));
    assert!(fs.exists(std::path::Path::new(".custom-hooks/_/h")));
}

#[test]
fn test_installation_skipped_when_husky_disabled() {
    let env = MockEnvironment::new().with_var("HUSKY", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "HUSKY=0 skip install");
}

#[test]
fn test_installation_fails_with_invalid_path() {
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();

    let result = install_hooks(&env, &runner, &fs, Some("../invalid"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), ".. not allowed");
}

#[test]
fn test_installation_fails_without_git_repo() {
    let env = MockEnvironment::new();
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new(); // No .git directory

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), ".git can't be found");
}

#[test]
fn test_all_standard_hooks_created() {
    let env = MockEnvironment::new();

    let config_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(config_output),
    );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    let expected_hooks = [
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

    for hook in &expected_hooks {
        let hook_path = std::path::Path::new(".samoid/_").join(hook);
        assert!(fs.exists(&hook_path), "Hook {} should exist", hook);
    }
}

#[test]
fn test_hook_runner_content() {
    let env = MockEnvironment::new();

    let config_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(config_output),
    );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    let runner_path = std::path::Path::new(".samoid/_/h");
    assert!(fs.exists(runner_path));
}

#[test]
fn test_gitignore_content() {
    let env = MockEnvironment::new();

    let config_output = Output {
        status: ExitStatus::from_raw(0),
        stdout: vec![],
        stderr: vec![],
    };
    let runner = MockCommandRunner::new().with_response(
        "git",
        &["config", "core.hooksPath", ".samoid/_"],
        Ok(config_output),
    );

    let fs = MockFileSystem::new().with_directory(".git");

    let result = install_hooks(&env, &runner, &fs, None);
    assert!(result.is_ok());

    let gitignore_path = std::path::Path::new(".samoid/_/.gitignore");
    assert!(fs.exists(gitignore_path));
}
