mod environment;
mod git;
mod hooks;
mod installer;

use installer::install_hooks_legacy;

fn main() {
    match install_hooks_legacy(None) {
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
    use crate::installer::install_hooks;
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
}
