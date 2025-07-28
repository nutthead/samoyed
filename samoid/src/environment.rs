//! Environment abstractions for dependency injection
//!
//! This module provides traits and implementations for abstracting system operations,
//! enabling comprehensive testing through dependency injection. The design allows
//! production code to use real system operations while tests can use mock implementations.
//!
//! # Architecture
//!
//! The module follows a trait-based design pattern with:
//! - Traits defining the interface (`Environment`, `CommandRunner`, `FileSystem`)
//! - Production implementations (`SystemEnvironment`, `SystemCommandRunner`, `SystemFileSystem`)
//! - Mock implementations for testing (in the `mocks` submodule)

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Trait for abstracting environment variable operations
///
/// This trait allows code to access environment variables in a testable way.
/// Production code uses `SystemEnvironment` which reads from the actual environment,
/// while tests use `MockEnvironment` with predetermined values.
pub trait Environment {
    /// Retrieves the value of an environment variable
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the environment variable to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The value if the variable exists
    /// * `None` - If the variable doesn't exist or cannot be read
    fn get_var(&self, key: &str) -> Option<String>;
}

/// Trait for abstracting command execution
///
/// This trait provides a testable interface for running system commands.
/// Production code uses `SystemCommandRunner` which executes real processes,
/// while tests use `MockCommandRunner` with predetermined responses.
pub trait CommandRunner {
    /// Executes a command with arguments and captures its output
    ///
    /// # Arguments
    ///
    /// * `program` - The command/program to execute
    /// * `args` - Command line arguments to pass to the program
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - Command output including status, stdout, and stderr
    /// * `Err(io::Error)` - If the command cannot be executed
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output>;
}

/// Trait for abstracting file system operations
///
/// This trait provides a testable interface for file system operations.
/// Production code uses `SystemFileSystem` for real file operations,
/// while tests use `MockFileSystem` with in-memory storage.
pub trait FileSystem {
    /// Checks if a path exists in the file system
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// * `true` if the path exists (file or directory)
    /// * `false` if the path doesn't exist
    fn exists(&self, path: &Path) -> bool;

    /// Creates a directory and all necessary parent directories
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the directory was created or already exists
    /// * `Err(io::Error)` - If directory creation fails
    #[allow(
        dead_code,
        reason = "Used through trait object in hooks.rs and main.rs"
    )]
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Writes string contents to a file
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to write to
    /// * `contents` - The string contents to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(io::Error)` - If writing fails
    #[allow(
        dead_code,
        reason = "Used through trait object in hooks.rs and main.rs"
    )]
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;

    /// Reads the entire contents of a file as a string
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to read from
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The file contents
    /// * `Err(io::Error)` - If reading fails or file doesn't exist
    #[allow(
        dead_code,
        reason = "Used through trait object in test implementations"
    )]
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// Sets Unix file permissions
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to modify
    /// * `mode` - Unix permission mode (e.g., 0o755)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If permissions were set successfully
    /// * `Err(io::Error)` - If setting permissions fails
    #[allow(
        dead_code,
        reason = "Used through trait object in hooks.rs for file permissions"
    )]
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()>;
}

/// Production implementation that interacts with the real system environment
///
/// This struct provides access to actual environment variables through
/// the standard library's `std::env` module.
pub struct SystemEnvironment;

impl Environment for SystemEnvironment {
    fn get_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Production implementation for executing system commands
///
/// This struct executes real system commands using `std::process::Command`.
/// Commands are run synchronously and their output is captured.
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output> {
        Command::new(program).args(args).output()
    }
}

/// Production implementation for file system operations
///
/// This struct performs real file system operations using the standard library's
/// `std::fs` module. All operations interact with the actual file system.
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

/// Mock implementations for testing
///
/// This module provides thread-safe mock implementations of all environment traits.
/// The mocks use `Arc<Mutex<T>>` for interior mutability, allowing them to be
/// shared across threads and modified during tests.
///
/// # Example
///
/// ```
/// use samoid::environment::mocks::{MockEnvironment, MockCommandRunner, MockFileSystem};
/// use samoid::environment::{Environment, CommandRunner, FileSystem};
/// use std::process::{Output, ExitStatus};
/// use std::os::unix::process::ExitStatusExt;
///
/// // Create a mock environment with a specific variable
/// let env = MockEnvironment::new().with_var("SAMOID", "0");
/// assert_eq!(env.get_var("SAMOID"), Some("0".to_string()));
///
/// // Create a mock command runner with a predefined response
/// let output = Output {
///     status: ExitStatus::from_raw(0),
///     stdout: b"success".to_vec(),
///     stderr: vec![],
/// };
/// let runner = MockCommandRunner::new()
///     .with_response("git", &["status"], Ok(output));
///
/// // Create a mock filesystem with files and directories
/// let fs = MockFileSystem::new()
///     .with_file("/test.txt", "content")
///     .with_directory("/test_dir");
/// ```
#[allow(dead_code)]
pub mod mocks {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Mock implementation of the Environment trait for testing
    ///
    /// Stores environment variables in a thread-safe HashMap, allowing
    /// tests to control environment variable values without affecting
    /// the actual system environment.
    pub struct MockEnvironment {
        /// Thread-safe storage for mock environment variables
        vars: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockEnvironment {
        /// Creates a new empty mock environment
        ///
        /// # Returns
        ///
        /// A new `MockEnvironment` with no variables set
        pub fn new() -> Self {
            Self {
                vars: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Adds an environment variable to the mock environment
        ///
        /// This method uses the builder pattern, allowing chaining of multiple
        /// variable definitions.
        ///
        /// # Arguments
        ///
        /// * `key` - The environment variable name
        /// * `value` - The environment variable value
        ///
        /// # Returns
        ///
        /// Self for method chaining
        ///
        /// # Example
        ///
        /// ```
        /// # use samoid::environment::mocks::MockEnvironment;
        /// let env = MockEnvironment::new()
        ///     .with_var("SAMOID", "0")
        ///     .with_var("CI", "true");
        /// ```
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

    /// Mock implementation of the CommandRunner trait for testing
    ///
    /// Stores predefined responses for specific command invocations.
    /// Commands are matched by combining the program name and arguments.
    pub struct MockCommandRunner {
        /// Thread-safe storage for command responses keyed by "program arg1 arg2"
        responses: Arc<Mutex<HashMap<String, io::Result<Output>>>>,
    }

    impl MockCommandRunner {
        /// Creates a new mock command runner with no configured responses
        ///
        /// Commands not configured will return a "Command not found" error.
        ///
        /// # Returns
        ///
        /// A new `MockCommandRunner` with no responses configured
        pub fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Configures a response for a specific command invocation
        ///
        /// This method uses the builder pattern for chaining multiple responses.
        /// The command and arguments must match exactly for the response to be returned.
        ///
        /// # Arguments
        ///
        /// * `command` - The command/program name
        /// * `args` - The exact arguments expected
        /// * `output` - The result to return when this command is run
        ///
        /// # Returns
        ///
        /// Self for method chaining
        ///
        /// # Example
        ///
        /// ```ignore
        /// let runner = MockCommandRunner::new()
        ///     .with_response("git", &["status"], Ok(success_output))
        ///     .with_response("git", &["commit"], Err(io_error));
        /// ```
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

    /// Mock implementation of the FileSystem trait for testing
    ///
    /// Stores files and directories in memory, providing a complete
    /// file system abstraction without touching the real file system.
    pub struct MockFileSystem {
        /// Thread-safe storage for file contents keyed by path
        files: Arc<Mutex<HashMap<PathBuf, String>>>,
        /// Thread-safe storage for directory paths
        directories: Arc<Mutex<Vec<PathBuf>>>,
    }

    impl MockFileSystem {
        /// Creates a new empty mock file system
        ///
        /// # Returns
        ///
        /// A new `MockFileSystem` with no files or directories
        pub fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                directories: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Adds a file with contents to the mock file system
        ///
        /// This method uses the builder pattern for chaining multiple files.
        ///
        /// # Arguments
        ///
        /// * `path` - The file path
        /// * `contents` - The file contents
        ///
        /// # Returns
        ///
        /// Self for method chaining
        ///
        /// # Example
        ///
        /// ```
        /// # use samoid::environment::mocks::MockFileSystem;
        /// let fs = MockFileSystem::new()
        ///     .with_file("/config.json", "{\"key\": \"value\"}")
        ///     .with_file("/script.sh", "#!/bin/bash\necho hello");
        /// ```
        pub fn with_file(self, path: impl Into<PathBuf>, contents: &str) -> Self {
            self.files
                .lock()
                .unwrap()
                .insert(path.into(), contents.to_string());
            self
        }

        /// Adds a directory to the mock file system
        ///
        /// This method uses the builder pattern for chaining multiple directories.
        /// The mock implementation treats directories as existing if they're in the
        /// directory list or if any path starts with the directory path.
        ///
        /// # Arguments
        ///
        /// * `path` - The directory path
        ///
        /// # Returns
        ///
        /// Self for method chaining
        ///
        /// # Example
        ///
        /// ```
        /// # use samoid::environment::mocks::MockFileSystem;
        /// let fs = MockFileSystem::new()
        ///     .with_directory(".git")
        ///     .with_directory("src/components");
        /// ```
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
    fn test_system_file_system_write_and_read() {
        let fs = SystemFileSystem;
        let test_path = std::path::Path::new("/tmp/samoid_test_file");

        // Test write operation
        let result = fs.write(test_path, "test content");
        assert!(result.is_ok());

        // Test read operation
        let content = fs.read_to_string(test_path);
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "test content");

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_system_file_system_create_dir_all() {
        let fs = SystemFileSystem;
        let test_dir = std::path::Path::new("/tmp/samoid_test_dir/nested/path");

        // Test directory creation
        let result = fs.create_dir_all(test_dir);
        assert!(result.is_ok());

        // Verify directory exists
        assert!(fs.exists(test_dir));

        // Clean up
        let _ = std::fs::remove_dir_all("/tmp/samoid_test_dir");
    }

    #[test]
    fn test_system_file_system_set_permissions() {
        let fs = SystemFileSystem;
        let test_path = std::path::Path::new("/tmp/samoid_test_permissions");

        // Create a test file first
        let _ = fs.write(test_path, "test");

        // Test setting permissions
        let result = fs.set_permissions(test_path, 0o644);
        assert!(result.is_ok());

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_system_file_system_read_nonexistent() {
        let fs = SystemFileSystem;
        let nonexistent_path = std::path::Path::new("/tmp/samoid_nonexistent_file_12345");

        // Test reading non-existent file
        let result = fs.read_to_string(nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_system_file_system_set_permissions_nonexistent() {
        let fs = SystemFileSystem;
        let nonexistent_path = std::path::Path::new("/tmp/samoid_nonexistent_file_12345");

        // Test setting permissions on non-existent file
        let result = fs.set_permissions(nonexistent_path, 0o644);
        assert!(result.is_err());
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
    fn test_mock_command_runner_error_response() {
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let runner = MockCommandRunner::new().with_response("fail_cmd", &[], Err(error));

        // Test command that returns an error
        let result = runner.run_command("fail_cmd", &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(err.to_string().contains("Access denied"));
    }

    #[test]
    fn test_mock_command_runner_multiple_responses() {
        let output1 = std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"first".to_vec(),
            stderr: vec![],
        };
        let output2 = std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: vec![],
            stderr: b"error".to_vec(),
        };

        let runner = MockCommandRunner::new()
            .with_response("cmd1", &["arg"], Ok(output1))
            .with_response("cmd2", &["arg"], Ok(output2));

        // Test first command
        let result1 = runner.run_command("cmd1", &["arg"]);
        assert!(result1.is_ok());
        let out1 = result1.unwrap();
        assert!(out1.status.success());
        assert_eq!(out1.stdout, b"first");

        // Test second command
        let result2 = runner.run_command("cmd2", &["arg"]);
        assert!(result2.is_ok());
        let out2 = result2.unwrap();
        assert!(!out2.status.success());
        assert_eq!(out2.stderr, b"error");
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

    #[test]
    fn test_mock_filesystem_path_matching() {
        let fs = MockFileSystem::new()
            .with_directory("/root")
            .with_file("/root/child/file.txt", "content");

        // Test path prefix matching for directories
        assert!(fs.exists(std::path::Path::new("/root")));
        assert!(fs.exists(std::path::Path::new("/root/child/file.txt")));

        // Test paths that don't exist
        assert!(!fs.exists(std::path::Path::new("/other")));
        assert!(!fs.exists(std::path::Path::new("/different/path")));

        // Note: The mock filesystem's exists() implementation checks:
        // 1. If the path is a file (exact match)
        // 2. If the path starts with any directory (prefix match)
        // So "/root/child" would return true because it starts with "/root"
    }

    #[test]
    fn test_mock_filesystem_empty() {
        let fs = MockFileSystem::new();

        // Test operations on empty filesystem
        assert!(!fs.exists(std::path::Path::new("/anything")));

        let result = fs.read_to_string(std::path::Path::new("/anything"));
        assert!(result.is_err());

        // Write and create operations should succeed
        let write_result = fs.write(std::path::Path::new("/new.txt"), "content");
        assert!(write_result.is_ok());

        let dir_result = fs.create_dir_all(std::path::Path::new("/new/dir"));
        assert!(dir_result.is_ok());
    }

    #[test]
    fn test_mock_filesystem_overwrite() {
        let fs = MockFileSystem::new().with_file("/test.txt", "original");

        // Verify original content
        let content = fs.read_to_string(std::path::Path::new("/test.txt"));
        assert_eq!(content.unwrap(), "original");

        // Overwrite the file
        let result = fs.write(std::path::Path::new("/test.txt"), "updated");
        assert!(result.is_ok());

        // Verify updated content
        let content = fs.read_to_string(std::path::Path::new("/test.txt"));
        assert_eq!(content.unwrap(), "updated");
    }
}
