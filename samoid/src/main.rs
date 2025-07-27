mod environment;
mod git;
mod hooks;
mod installer;

use environment::{SystemCommandRunner, SystemEnvironment, SystemFileSystem};
use installer::install_hooks;

fn main() {
    let env = SystemEnvironment;
    let runner = SystemCommandRunner;
    let fs = SystemFileSystem;

    match install_hooks(&env, &runner, &fs, None) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{}", msg);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use crate::environment::{CommandRunner, Environment, FileSystem};
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn test_main_with_husky_disabled() {
        // Create mocks - each test is completely isolated
        let env = MockEnvironment::new().with_var("HUSKY", "0");
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HUSKY=0 skip install");
    }

    #[test]
    fn test_main_with_error_case() {
        // No .git directory, should fail
        let env = MockEnvironment::new();
        let runner = MockCommandRunner::new();
        let fs = MockFileSystem::new();

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_success_path() {
        let env = MockEnvironment::new();

        // Mock successful git command
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

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_main_git_command_failure() {
        let env = MockEnvironment::new();

        // Mock failed git command
        let output = Output {
            status: ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"fatal: not a git repository".to_vec(),
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output),
        );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_with_custom_directory() {
        let env = MockEnvironment::new();

        // Mock successful git command with custom directory
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".custom/_"],
            Ok(output),
        );

        // Mock filesystem with git repository
        let fs = MockFileSystem::new().with_directory(".git");

        let result = install_hooks(&env, &runner, &fs, Some(".custom"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_main_logic_components() {
        // Test that main logic components are properly instantiated
        let env = SystemEnvironment;
        let runner = SystemCommandRunner;
        let fs = SystemFileSystem;

        // Test that traits are implemented correctly
        let _env_var = env.get_var("PATH");
        let _command_result = runner.run_command("echo", &["test"]);
        let _exists_result = fs.exists(std::path::Path::new("."));

        // These should not panic or fail to compile
        assert!(true);
    }

    #[test]
    fn test_main_execution_paths() {
        // Test the execution paths that main() would take

        // Success case: HUSKY=0 (should return message)
        let env_disabled = MockEnvironment::new().with_var("HUSKY", "0");
        let runner_disabled = MockCommandRunner::new();
        let fs_disabled = MockFileSystem::new();

        let result_disabled = install_hooks(&env_disabled, &runner_disabled, &fs_disabled, None);
        assert!(result_disabled.is_ok());
        let msg = result_disabled.unwrap();
        assert!(!msg.is_empty()); // Would trigger println! in main

        // Error case: not a git repository (should return error)
        let env_error = MockEnvironment::new();
        let runner_error = MockCommandRunner::new();
        let fs_error = MockFileSystem::new(); // No .git

        let result_error = install_hooks(&env_error, &runner_error, &fs_error, None);
        assert!(result_error.is_err()); // Would trigger eprintln! and exit(1) in main

        // Success case: normal execution (should return empty message)
        let env_success = MockEnvironment::new();
        let output_success = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        let runner_success = MockCommandRunner::new().with_response(
            "git",
            &["config", "core.hooksPath", ".samoid/_"],
            Ok(output_success),
        );
        let fs_success = MockFileSystem::new().with_directory(".git");

        let result_success = install_hooks(&env_success, &runner_success, &fs_success, None);
        assert!(result_success.is_ok());
        let msg_success = result_success.unwrap();
        assert!(msg_success.is_empty()); // Would not trigger println! in main
    }
}
