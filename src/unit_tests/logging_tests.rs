use super::*;

#[test]
fn test_sanitize_path_home_directory() {
    use crate::environment::mocks::MockEnvironment;

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
