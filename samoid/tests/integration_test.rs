use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_core_installation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to initialize git repo");

    let result = samoid::install_hooks(None);
    assert!(result.is_ok());

    let hooks_path = ".samoid/_";
    assert!(Path::new(hooks_path).exists());
    assert!(Path::new(&format!("{}/.gitignore", hooks_path)).exists());
    assert!(Path::new(&format!("{}/h", hooks_path)).exists());
    assert!(Path::new(&format!("{}/pre-commit", hooks_path)).exists());

    let output = Command::new("git")
        .args(&["config", "core.hooksPath"])
        .output()
        .expect("Failed to get git config");

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), ".samoid/_");
}

#[test]
fn test_installation_with_custom_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to initialize git repo");

    let result = samoid::install_hooks(Some(".custom-hooks"));
    assert!(result.is_ok());

    let hooks_path = ".custom-hooks/_";
    assert!(Path::new(hooks_path).exists());
    assert!(Path::new(&format!("{}/.gitignore", hooks_path)).exists());
    assert!(Path::new(&format!("{}/h", hooks_path)).exists());

    let output = Command::new("git")
        .args(&["config", "core.hooksPath"])
        .output()
        .expect("Failed to get git config");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        ".custom-hooks/_"
    );
}

#[test]
fn test_installation_skipped_when_husky_disabled() {
    unsafe {
        env::set_var("HUSKY", "0");
    }

    let result = samoid::install_hooks(None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "HUSKY=0 skip install");

    unsafe {
        env::remove_var("HUSKY");
    }
}

#[test]
fn test_installation_fails_with_invalid_path() {
    let result = samoid::install_hooks(Some("../invalid"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), ".. not allowed");
}

#[test]
fn test_installation_fails_without_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    let result = samoid::install_hooks(None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), ".git can't be found");
}

#[test]
fn test_all_standard_hooks_created() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to initialize git repo");

    let result = samoid::install_hooks(None);
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
        let hook_path = format!(".samoid/_/{}", hook);
        assert!(Path::new(&hook_path).exists(), "Hook {} should exist", hook);

        let content = fs::read_to_string(&hook_path).unwrap();
        assert_eq!(content, "#!/usr/bin/env sh\n. \"$(dirname \"$0\")/h\"");
    }
}

#[test]
fn test_hook_runner_content() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to initialize git repo");

    let result = samoid::install_hooks(None);
    assert!(result.is_ok());

    let runner_path = ".samoid/_/h";
    assert!(Path::new(runner_path).exists());

    let content = fs::read_to_string(runner_path).unwrap();
    assert!(content.contains("Samoid hook runner - placeholder implementation"));
    assert!(content.contains("exec \"$@\""));
}

#[test]
fn test_gitignore_content() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    std::env::set_current_dir(repo_path).unwrap();

    Command::new("git")
        .args(&["init"])
        .output()
        .expect("Failed to initialize git repo");

    let result = samoid::install_hooks(None);
    assert!(result.is_ok());

    let gitignore_path = ".samoid/_/.gitignore";
    assert!(Path::new(gitignore_path).exists());

    let content = fs::read_to_string(gitignore_path).unwrap();
    assert_eq!(content, "*");
}
