use super::*;
use crate::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};

// Cross-platform exit status creation
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

// Helper function to create ExitStatus cross-platform
fn exit_status(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    return std::process::ExitStatus::from_raw(code);

    #[cfg(windows)]
    return std::process::ExitStatus::from_raw(code as u32);
}

#[test]
fn system_environment_basic_operations() {
    let env = SystemEnvironment;

    // Test getting environment variable that likely exists
    let path = env.get_var("PATH");
    assert!(path.is_some());

    // Test getting non-existent environment variable
    let nonexistent = env.get_var("SAMOYED_NONEXISTENT_VAR_12345");
    assert_eq!(nonexistent, None);
}

#[test]
fn system_command_runner() {
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
fn system_command_runner_failure() {
    let runner = SystemCommandRunner;

    // Test with a command that should fail
    let result = runner.run_command("nonexistent_command_12345", &[]);
    assert!(result.is_err());
}

#[test]
fn system_file_system_operations() {
    let fs = SystemFileSystem;

    // Test exists with a path that should exist
    assert!(fs.exists(std::path::Path::new(".")));

    // Test exists with a path that should not exist
    assert!(!fs.exists(std::path::Path::new("/nonexistent/path/12345")));
}

#[test]
fn system_file_system_write_and_read() {
    let fs = SystemFileSystem;
    let test_path = std::path::Path::new("/tmp/samoyed_test_file");

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
fn system_file_system_create_dir_all() {
    let fs = SystemFileSystem;
    let test_dir = std::path::Path::new("/tmp/samoyed_test_dir/nested/path");

    // Test directory creation
    let result = fs.create_dir_all(test_dir);
    assert!(result.is_ok());

    // Verify directory exists
    assert!(fs.exists(test_dir));

    // Clean up
    let _ = std::fs::remove_dir_all("/tmp/samoyed_test_dir");
}

#[test]
fn system_file_system_set_permissions() {
    let fs = SystemFileSystem;
    let test_path = std::path::Path::new("/tmp/samoyed_test_permissions");

    // Create a test file first
    let _ = fs.write(test_path, "test");

    // Test setting permissions
    let result = fs.set_permissions(test_path, 0o644);
    assert!(result.is_ok());

    // Clean up
    let _ = std::fs::remove_file(test_path);
}

#[test]
fn system_file_system_read_nonexistent() {
    let fs = SystemFileSystem;
    let nonexistent_path = std::path::Path::new("/tmp/samoyed_nonexistent_file_12345");

    // Test reading non-existent file
    let result = fs.read_to_string(nonexistent_path);
    assert!(result.is_err());
}

#[test]
fn system_file_system_set_permissions_nonexistent() {
    let fs = SystemFileSystem;
    let nonexistent_path = std::path::Path::new("/tmp/samoyed_nonexistent_file_12345");

    // Test setting permissions on non-existent file
    let result = fs.set_permissions(nonexistent_path, 0o644);
    assert!(result.is_err());
}

#[test]
fn mock_environment_operations() {
    let env = MockEnvironment::new().with_var("TEST_VAR", "test_value");

    // Test getting existing variable
    assert_eq!(env.get_var("TEST_VAR"), Some("test_value".to_string()));

    // Test getting non-existing variable
    assert_eq!(env.get_var("NONEXISTENT"), None);
}

#[test]
fn mock_command_runner_operations() {
    let output = std::process::Output {
        status: exit_status(0),
        stdout: b"success".to_vec(),
        stderr: vec![],
    };

    let runner = MockCommandRunner::new().with_response("test_cmd", &["arg1", "arg2"], Ok(output));

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
fn mock_command_runner_error_response() {
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
fn mock_command_runner_multiple_responses() {
    let output1 = std::process::Output {
        status: exit_status(0),
        stdout: b"first".to_vec(),
        stderr: vec![],
    };
    let output2 = std::process::Output {
        status: exit_status(1),
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
fn mock_filesystem_operations() {
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
fn mock_filesystem_path_matching() {
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
fn mock_filesystem_empty() {
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
fn mock_filesystem_overwrite() {
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
