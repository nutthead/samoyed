use super::*;
use anyhow::anyhow;

#[test]
fn test_determine_exit_code() {
    // Test various error messages map to correct exit codes
    let git_not_found = anyhow!("Git command not found in PATH");
    assert_eq!(determine_exit_code(&git_not_found), EX_UNAVAILABLE);

    let not_git_repo = anyhow!("Not a Git repository (no .git directory found)");
    assert_eq!(determine_exit_code(&not_git_repo), EX_NOINPUT);

    let permission_denied = anyhow!("Permission denied: set Git configuration");
    assert_eq!(determine_exit_code(&permission_denied), EX_NOPERM);

    let invalid_path = anyhow!("Invalid path '../invalid': Directory traversal detected");
    assert_eq!(determine_exit_code(&invalid_path), EX_DATAERR);

    let empty_path = anyhow!("Path cannot be empty");
    assert_eq!(determine_exit_code(&empty_path), EX_USAGE);

    let config_lock = anyhow!("error: could not lock config file");
    assert_eq!(determine_exit_code(&config_lock), EX_TEMPFAIL);

    let config_failed = anyhow!("Configuration failed: bad config");
    assert_eq!(determine_exit_code(&config_failed), EX_CONFIG);

    let io_error = anyhow!("IO error: Failed to write file");
    assert_eq!(determine_exit_code(&io_error), EX_IOERR);

    let unknown_error = anyhow!("Some unknown error occurred");
    assert_eq!(determine_exit_code(&unknown_error), EX_SOFTWARE);
}
