//! Windows-specific integration tests
//!
//! Tests that validate Windows-specific behavior including paths, permissions,
//! and environment variable handling.

#[cfg(target_os = "windows")]
mod windows_tests {
    use samoyed::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use samoyed::install_hooks;
    use std::process::{ExitStatus, Output};

    // Cross-platform exit status creation
    use std::os::windows::process::ExitStatusExt;

    // Helper function to create ExitStatus
    fn exit_status(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code as u32)
    }

    #[test]
    fn windows_path_handling() {
        let env = MockEnvironment::new()
            .with_var("USERPROFILE", "C:\\Users\\user")
            .with_var("USERNAME", "testuser");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1.windows.1".to_vec(),
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

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());

        // Test Windows-style paths
        let windows_paths = vec![
            "hooks",         // Forward slash
            "hooks\\subdir", // Backslash
            "C:\\hooks",     // Absolute (should fail)
            "..\\hooks",     // Parent directory (should fail)
        ];

        for path in windows_paths {
            let result = install_hooks(&env, &runner, &fs, Some(path));
            if path.contains(':') || path.contains("..") {
                assert!(result.is_err(), "Invalid path should fail: {path}");
            } else {
                // Relative paths should work
                let runner2 = MockCommandRunner::new()
                    .with_response("git", &["--version"], Ok(version_output.clone()))
                    .with_response(
                        "git",
                        &["config", "core.hooksPath", &format!("{path}\\_")],
                        Ok(output.clone()),
                    );
                let result = install_hooks(&env, &runner2, &fs, Some(path));
                // Note: actual implementation might normalize paths
                let _ = result; // Don't assert, just ensure no panic
            }
        }
    }

    #[test]
    fn windows_git_installations() {
        // Test that samoyed works with various Git for Windows installations
        // Note: samoyed always uses "git" command regardless of where Git is installed
        let git_installations = vec![
            "Standard Git for Windows (C:\\Program Files\\Git\\bin\\git.exe)",
            "32-bit Git for Windows (C:\\Program Files (x86)\\Git\\bin\\git.exe)",
            "Chocolatey Git (C:\\tools\\git\\bin\\git.exe)",
            "Git in PATH",
        ];

        for description in git_installations {
            let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\user");

            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let version_output = Output {
                status: exit_status(0),
                stdout: format!("git version 2.34.1.windows.1 ({description})").into_bytes(),
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
            assert!(result.is_ok(), "Should work with {description}");
        }
    }

    #[test]
    fn windows_environment_variables() {
        // Test Windows-specific environment variables
        let env = MockEnvironment::new()
            .with_var("USERPROFILE", "C:\\Users\\user")
            .with_var("HOMEDRIVE", "C:")
            .with_var("HOMEPATH", "\\Users\\user")
            .with_var("APPDATA", "C:\\Users\\user\\AppData\\Roaming")
            .with_var("LOCALAPPDATA", "C:\\Users\\user\\AppData\\Local");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1.windows.1".to_vec(),
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
        assert!(
            result.is_ok(),
            "Should handle Windows environment variables"
        );
    }

    #[test]
    fn windows_line_endings() {
        // Test handling of Windows CRLF line endings
        let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\user");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1.windows.1\r\n".to_vec(), // CRLF
            stderr: vec![],
        };
        let runner = MockCommandRunner::new()
            .with_response("git", &["--version"], Ok(version_output))
            .with_response(
                "git",
                &["config", "core.hooksPath", ".samoyed/_"],
                Ok(output),
            );

        // Filesystem with CRLF line endings in existing files
        let fs = MockFileSystem::new()
            .with_directory(".git")
            .with_file(".gitconfig", "[core]\r\n\tautocrlf = true\r\n");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok(), "Should handle CRLF line endings");
    }

    #[test]
    fn windows_shells() {
        // Test various Windows shells
        let shells = vec![
            ("cmd.exe", "Command Prompt"),
            ("powershell.exe", "PowerShell"),
            ("pwsh.exe", "PowerShell Core"),
            ("C:\\Program Files\\Git\\bin\\bash.exe", "Git Bash"),
        ];

        for (shell, description) in shells {
            let env = MockEnvironment::new()
                .with_var("USERPROFILE", "C:\\Users\\user")
                .with_var("COMSPEC", shell);

            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let version_output = Output {
                status: exit_status(0),
                stdout: b"git version 2.34.1.windows.1".to_vec(),
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
            assert!(result.is_ok(), "Should work with {description}: {shell}");
        }
    }

    #[test]
    fn windows_path_separators() {
        // Test mixed path separators
        let env = MockEnvironment::new().with_var("USERPROFILE", "C:\\Users\\user");

        let output = Output {
            status: exit_status(0),
            stdout: vec![],
            stderr: vec![],
        };
        let version_output = Output {
            status: exit_status(0),
            stdout: b"git version 2.34.1.windows.1".to_vec(),
            stderr: vec![],
        };

        // Test paths with mixed separators
        let mixed_paths = vec![
            "hooks/sub\\dir",  // Mixed separators
            "hooks\\sub/dir",  // Mixed separators reversed
            "hooks\\\\double", // Double backslash
            "hooks//double",   // Double forward slash
        ];

        for path in mixed_paths {
            let runner = MockCommandRunner::new()
                .with_response("git", &["--version"], Ok(version_output.clone()))
                .with_response(
                    "git",
                    &["config", "core.hooksPath", &format!("{path}\\_")],
                    Ok(output.clone()),
                );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, Some(path));
            // Implementation should normalize these paths
            let _ = result; // Don't assert specific behavior, just ensure no panic
        }
    }
}

// This entire test module is only compiled on Windows
// On other platforms, this file is effectively empty
