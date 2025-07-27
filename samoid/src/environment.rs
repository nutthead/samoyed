use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Trait for abstracting environment variable operations
pub trait Environment {
    fn get_var(&self, key: &str) -> Option<String>;
}

/// Trait for abstracting command execution
pub trait CommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output>;
}

/// Trait for abstracting file system operations
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()>;
}

/// Production implementation that interacts with the real system
pub struct SystemEnvironment;

impl Environment for SystemEnvironment {
    fn get_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output> {
        Command::new(program).args(args).output()
    }
}

pub struct SystemFileSystem;

impl FileSystem for SystemFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        fs::write(path, contents)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    #[cfg(unix)]
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions)
    }

    #[cfg(not(unix))]
    fn set_permissions(&self, _path: &Path, _mode: u32) -> io::Result<()> {
        // On non-Unix systems, we'll just return Ok
        // In production, you might want to handle Windows permissions differently
        Ok(())
    }
}

/// Mock implementation for testing
#[allow(dead_code)]
pub mod mocks {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    pub struct MockEnvironment {
        vars: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockEnvironment {
        pub fn new() -> Self {
            Self {
                vars: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_var(self, key: &str, value: &str) -> Self {
            self.vars
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            self
        }
    }

    impl Environment for MockEnvironment {
        fn get_var(&self, key: &str) -> Option<String> {
            self.vars.lock().unwrap().get(key).cloned()
        }
    }

    pub struct MockCommandRunner {
        responses: Arc<Mutex<HashMap<String, io::Result<Output>>>>,
    }

    impl MockCommandRunner {
        pub fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_response(
            self,
            command: &str,
            args: &[&str],
            output: io::Result<Output>,
        ) -> Self {
            let key = format!("{} {}", command, args.join(" "));
            self.responses.lock().unwrap().insert(key, output);
            self
        }
    }

    impl CommandRunner for MockCommandRunner {
        fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output> {
            let key = format!("{} {}", program, args.join(" "));
            if let Some(result) = self.responses.lock().unwrap().get(&key) {
                match result {
                    Ok(output) => Ok(Output {
                        status: output.status,
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
                    }),
                    Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
                }
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "Command not found"))
            }
        }
    }

    pub struct MockFileSystem {
        files: Arc<Mutex<HashMap<PathBuf, String>>>,
        directories: Arc<Mutex<Vec<PathBuf>>>,
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                directories: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn with_file(self, path: impl Into<PathBuf>, contents: &str) -> Self {
            self.files
                .lock()
                .unwrap()
                .insert(path.into(), contents.to_string());
            self
        }

        pub fn with_directory(self, path: impl Into<PathBuf>) -> Self {
            self.directories.lock().unwrap().push(path.into());
            self
        }
    }

    impl FileSystem for MockFileSystem {
        fn exists(&self, path: &Path) -> bool {
            let files = self.files.lock().unwrap();
            let dirs = self.directories.lock().unwrap();

            files.contains_key(path) || dirs.iter().any(|d| d == path || path.starts_with(d))
        }

        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            self.directories.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }

        fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), contents.to_string());
            Ok(())
        }

        fn read_to_string(&self, path: &Path) -> io::Result<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }

        fn set_permissions(&self, _path: &Path, _mode: u32) -> io::Result<()> {
            // Mock implementation doesn't need to do anything
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn test_system_environment_basic_operations() {
        let env = SystemEnvironment;

        // Test getting environment variable that likely exists
        let path = env.get_var("PATH");
        assert!(path.is_some());

        // Test getting non-existent environment variable
        let nonexistent = env.get_var("SAMOID_NONEXISTENT_VAR_12345");
        assert_eq!(nonexistent, None);
    }

    #[test]
    fn test_system_command_runner() {
        let runner = SystemCommandRunner;

        // Test with a basic command that should exist on most systems
        let result = runner.run_command("echo", &["test"]);
        assert!(result.is_ok());

        if let Ok(output) = result {
            assert!(output.status.success());
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "test");
        }
    }

    #[test]
    fn test_system_command_runner_failure() {
        let runner = SystemCommandRunner;

        // Test with a command that should fail
        let result = runner.run_command("nonexistent_command_12345", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_system_file_system_operations() {
        let fs = SystemFileSystem;

        // Test exists with a path that should exist
        assert!(fs.exists(std::path::Path::new(".")));

        // Test exists with a path that should not exist
        assert!(!fs.exists(std::path::Path::new("/nonexistent/path/12345")));
    }

    #[test]
    fn test_mock_environment_operations() {
        let env = MockEnvironment::new().with_var("TEST_VAR", "test_value");

        // Test getting existing variable
        assert_eq!(env.get_var("TEST_VAR"), Some("test_value".to_string()));

        // Test getting non-existing variable
        assert_eq!(env.get_var("NONEXISTENT"), None);
    }

    #[test]
    fn test_mock_command_runner_operations() {
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"success".to_vec(),
            stderr: vec![],
        };

        let runner =
            MockCommandRunner::new().with_response("test_cmd", &["arg1", "arg2"], Ok(output));

        // Test configured command
        let result = runner.run_command("test_cmd", &["arg1", "arg2"]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"success");

        // Test unconfigured command
        let result = runner.run_command("unknown_cmd", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_filesystem_operations() {
        let fs = MockFileSystem::new()
            .with_file("/test/file.txt", "test content")
            .with_directory("/test/dir");

        // Test exists for file
        assert!(fs.exists(std::path::Path::new("/test/file.txt")));

        // Test exists for directory
        assert!(fs.exists(std::path::Path::new("/test/dir")));

        // Test exists for non-existent path
        assert!(!fs.exists(std::path::Path::new("/nonexistent")));

        // Test read existing file
        let content = fs.read_to_string(std::path::Path::new("/test/file.txt"));
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "test content");

        // Test read non-existent file
        let content = fs.read_to_string(std::path::Path::new("/nonexistent"));
        assert!(content.is_err());

        // Test write new file
        let result = fs.write(std::path::Path::new("/new/file.txt"), "new content");
        assert!(result.is_ok());

        // Test create directory
        let result = fs.create_dir_all(std::path::Path::new("/new/nested/dir"));
        assert!(result.is_ok());

        // Test set permissions (should always succeed for mock)
        let result = fs.set_permissions(std::path::Path::new("/test/file.txt"), 0o755);
        assert!(result.is_ok());
    }
}
