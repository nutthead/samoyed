//! Linux-specific integration tests
//!
//! Tests that validate Linux-specific behavior including paths, permissions,
//! and environment variable handling.

#[cfg(target_os = "linux")]
mod linux_tests {
    use samoyed::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use samoyed::install_hooks;
    use std::process::{ExitStatus, Output};

    // Cross-platform exit status creation
    use std::os::unix::process::ExitStatusExt;

    // Helper function to create ExitStatus
    fn exit_status(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code)
    }

    #[test]
    fn test_linux_path_handling() {
        let env = MockEnvironment::new()
            .with_var("HOME", "/home/user")
            .with_var("USER", "testuser");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        // Mock git --version first
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(output.clone()),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test with absolute paths (should fail)
        let result = install_hooks(&env, &runner, &fs, Some("/etc/hooks"));
        assert!(result.is_err());

        // Test with relative paths (should succeed)
        let version_output2 = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let runner2 = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output2))
            .with_response("git", &["config", "core.hooksPath", "hooks/_"], Ok(output));
        let result = install_hooks(&env, &runner2, &fs, Some("hooks"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_xdg_config_home_handling() {
        // Test XDG Base Directory specification compliance
        let env_with_xdg = MockEnvironment::new()
            .with_var("HOME", "/home/user")
            .with_var("XDG_CONFIG_HOME", "/home/user/.config");

        let env_without_xdg = MockEnvironment::new().with_var("HOME", "/home/user");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output.clone()))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(output.clone()),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        // Both should succeed
        let result1 = install_hooks(&env_with_xdg, &runner, &fs, None);
        assert!(result1.is_ok());

        let runner2 = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(output),
            );
        let result2 = install_hooks(&env_without_xdg, &runner2, &fs, None);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_file_permissions_linux() {
        let env = MockEnvironment::new().with_var("HOME", "/home/user");
        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1".to_vec(),
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(output),
            );
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // On Linux, verify hook files would have executable permissions
        // Note: MockFileSystem doesn't track permissions, but in real Linux
        // the hooks should be created with 0o755 permissions
    }

    #[test]
    fn test_linux_shell_compatibility() {
        // Test shell-specific behavior
        let shells = vec![
            ("SHELL", "/bin/bash"),
            ("SHELL", "/bin/zsh"),
            ("SHELL", "/bin/sh"),
            ("SHELL", "/usr/bin/fish"),
        ];

        for (var, shell_path) in shells {
            let env = MockEnvironment::new()
                .with_var("HOME", "/home/user")
                .with_var(var, shell_path);

            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let version_output = Output {
                status: exit_status(0),
                stdout: b"git version 2.34.1".to_vec(),
                stderr: vec![],
            };
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", ".samoyed/_"],
                    Ok(output),
                );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, None);
            assert!(result.is_ok(), "Should work with shell: {shell_path}");
        }
    }

    #[test]
    fn test_linux_git_variations() {
        // Test different git installations common on Linux
        let git_paths = vec![
            "/usr/bin/git",
            "/usr/local/bin/git",
            "/opt/git/bin/git",
            "git", // System PATH
        ];

        // Note: The actual implementation always calls "git", not the specific paths
        // This test verifies that different git installations work by simulating
        // the "git" command being available, regardless of the installation path
        for git_path in git_paths {
            let env = MockEnvironment::new().with_var("HOME", "/home/user");
            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let version_output = Output {
                status: exit_status(0),
                stdout: format!("git version 2.34.1 (from {git_path})").into_bytes(),
                stderr: vec![],
            };
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", ".samoyed/_"],
                    Ok(output),
                );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, None);
            assert!(result.is_ok(), "Should work with git at: {git_path}");
        }
    }
}

// This entire test module is only compiled on Linux
// On other platforms, this file is effectively empty
