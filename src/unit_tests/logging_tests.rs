use super::*;
use crate::environment::mocks::MockEnvironment;
use std::path::Path;

#[test]
fn test_sanitize_path_home_directory() {
    // Mock HOME environment variable
    let env = MockEnvironment::new().with_var("HOME", "/home/testuser");

    let result = sanitize_path_with_env(&env, "/home/testuser/.config/samoyed");
    assert_eq!(result, "~/.config/samoyed");
}

#[test]
fn test_sanitize_path_relative() {
    let result = sanitize_path(".samoyed/scripts/pre-commit");
    assert_eq!(result, ".samoyed/scripts/pre-commit");
}

#[test]
fn test_sanitize_path_sensitive() {
    let result = sanitize_path("/home/user/.ssh/id_rsa");
    assert_eq!(result, "[REDACTED_SENSITIVE_PATH]");
}

#[test]
fn test_sanitize_args_passwords() {
    let args = vec![
        "git".to_string(),
        "clone".to_string(),
        "--password=secret123".to_string(),
        "repo.git".to_string(),
    ];

    let result = sanitize_args(&args);
    assert_eq!(result[2], "[REDACTED]");
    assert_eq!(result[0], "git");
    assert_eq!(result[1], "clone");
}

#[test]
fn test_sanitize_args_tokens() {
    let args = vec![
        "curl".to_string(),
        "-H".to_string(),
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
    ];

    let result = sanitize_args(&args);
    assert_eq!(result[2], "[REDACTED_TOKEN]");
}

#[test]
fn test_sanitize_env_var_sensitive() {
    let result = sanitize_env_var("API_SECRET", "secret123");
    assert_eq!(result, None);
}

#[test]
fn test_sanitize_env_var_semi_sensitive() {
    let result = sanitize_env_var("HOME", "/home/user");
    assert_eq!(result, Some("[REDACTED_HOME_VALUE]".to_string()));
}

#[test]
fn test_sanitize_env_var_safe() {
    let result = sanitize_env_var("SAMOYED_DEBUG", "1");
    assert_eq!(result, Some("1".to_string()));
}

#[test]
fn test_log_file_operation_with_env_debug_mode() {
    let env = MockEnvironment::new().with_var("HOME", "/home/testuser");
    let path = Path::new("/home/testuser/.config/samoyed/config.toml");

    // Test with debug mode enabled - function should log to stderr
    // We can't easily capture stderr in unit tests, but we can verify the function runs without panicking
    log_file_operation_with_env(&env, true, "Reading", path);

    // Test with debug mode disabled - function should not log
    log_file_operation_with_env(&env, false, "Writing", path);
}

#[test]
fn test_log_file_operation_with_env_different_paths() {
    let env = MockEnvironment::new().with_var("HOME", "/home/testuser");

    // Test with absolute path
    let abs_path = Path::new("/usr/local/bin/samoyed");
    log_file_operation_with_env(&env, true, "Creating", abs_path);

    // Test with relative path
    let rel_path = Path::new(".samoyed/scripts/pre-commit");
    log_file_operation_with_env(&env, true, "Executing", rel_path);

    // Test with sensitive path
    let sensitive_path = Path::new("/home/testuser/.ssh/id_rsa");
    log_file_operation_with_env(&env, true, "Checking", sensitive_path);
}

#[test]
fn test_log_file_operation_debug_modes() {
    let path = Path::new(".samoyed/config.toml");

    // Test with debug mode enabled
    log_file_operation(true, "Reading", path);

    // Test with debug mode disabled
    log_file_operation(false, "Writing", path);
}

#[test]
fn test_log_file_operation_different_operations() {
    let path = Path::new("/tmp/samoyed-test.txt");

    // Test different operation types
    log_file_operation(true, "Creating", path);
    log_file_operation(true, "Modifying", path);
    log_file_operation(true, "Deleting", path);
    log_file_operation(true, "Checking", path);
}

#[test]
fn test_log_command_execution_debug_modes() {
    let args = vec!["--version".to_string()];

    // Test with debug mode enabled
    log_command_execution(true, "git", &args);

    // Test with debug mode disabled
    log_command_execution(false, "git", &args);
}

#[test]
fn test_log_command_execution_with_sensitive_args() {
    // Test with sensitive arguments that should be redacted
    let sensitive_args = vec![
        "clone".to_string(),
        "--password=secret123".to_string(),
        "https://github.com/user/repo.git".to_string(),
    ];

    log_command_execution(true, "git", &sensitive_args);
}

#[test]
fn test_log_command_execution_with_tokens() {
    // Test with token-like arguments
    let token_args = vec![
        "-H".to_string(),
        "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
        "https://api.github.com/user".to_string(),
    ];

    log_command_execution(true, "curl", &token_args);
}

#[test]
fn test_log_command_execution_empty_args() {
    // Test with no arguments
    let empty_args: Vec<String> = vec![];
    log_command_execution(true, "samoyed", &empty_args);
}

#[test]
fn test_log_command_execution_complex_command() {
    // Test with complex command and multiple arguments
    let complex_args = vec![
        "fmt".to_string(),
        "--check".to_string(),
        "--all".to_string(),
        "--config".to_string(),
        ".rustfmt.toml".to_string(),
    ];

    log_command_execution(true, "cargo", &complex_args);
}
