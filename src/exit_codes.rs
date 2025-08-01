//! Exit code definitions and error mapping for Samoid
//!
//! This module provides standard exit codes following the sysexits.h convention
//! and utilities for mapping errors to appropriate exit codes.

/// Exit codes following the sysexits.h convention
///
/// Complete set of exit codes for future use
#[allow(dead_code)]
/// Successful termination.
///
/// This follows the BSD sysexits.h convention.
pub const EX_OK: i32 = 0;
/// Command line usage error.
///
/// This follows the BSD sysexits.h convention.
pub const EX_USAGE: i32 = 64;
/// Data format error.
///
/// This follows the BSD sysexits.h convention.
pub const EX_DATAERR: i32 = 65;
/// Exit code indicating that an input file cannot be opened.
///
/// This follows the BSD sysexits.h convention.
pub const EX_NOINPUT: i32 = 66;
#[allow(dead_code)]
/// Addressee unknown.
///
/// This follows the BSD sysexits.h convention.
pub const EX_NOUSER: i32 = 67;
#[allow(dead_code)]
/// Host name unknown.
///
/// This follows the BSD sysexits.h convention.
pub const EX_NOHOST: i32 = 68;
/// Service unavailable.
///
/// This follows the BSD sysexits.h convention.
pub const EX_UNAVAILABLE: i32 = 69;
/// Internal software error.
///
/// This follows the BSD sysexits.h convention.
pub const EX_SOFTWARE: i32 = 70;
#[allow(dead_code)]
/// System error (e.g., can't fork).
///
/// This follows the BSD sysexits.h convention.
pub const EX_OSERR: i32 = 71;
#[allow(dead_code)]
/// Critical OS file missing.
///
/// This follows the BSD sysexits.h convention.
pub const EX_OSFILE: i32 = 72;
#[allow(dead_code)]
/// Can't create (user) output file.
///
/// This follows the BSD sysexits.h convention.
pub const EX_CANTCREATE: i32 = 73;
/// Input/output error.
///
/// This follows the BSD sysexits.h convention.
pub const EX_IOERR: i32 = 74;
/// Temporary failure; user is invited to retry.
///
/// This follows the BSD sysexits.h convention.
pub const EX_TEMPFAIL: i32 = 75;
#[allow(dead_code)]
/// Remote error in protocol.
///
/// This follows the BSD sysexits.h convention.
pub const EX_PROTOCOL: i32 = 76;
/// Permission denied.
///
/// This follows the BSD sysexits.h convention.
pub const EX_NOPERM: i32 = 77;
/// Configuration error.
///
/// This follows the BSD sysexits.h convention.
pub const EX_CONFIG: i32 = 78;

/// Determines the appropriate exit code based on the error type
///
/// This function maps different error scenarios to standard exit codes
/// following the sysexits.h convention, providing meaningful exit codes
/// for shell scripts and CI/CD systems.
///
/// # Arguments
///
/// * `error` - The error to analyze and map to an exit code
///
/// # Returns
///
/// An appropriate exit code based on the error content
///
/// # Example
///
/// ```rust,ignore
/// use anyhow::anyhow;
/// use samoyed::exit_codes::determine_exit_code;
///
/// let error = anyhow!("Permission denied");
/// let exit_code = determine_exit_code(&error);
/// assert_eq!(exit_code, 77); // EX_NOPERM
/// ```
pub fn determine_exit_code(error: &anyhow::Error) -> i32 {
    let error_str = error.to_string();

    // Check for specific error patterns and map to appropriate exit codes
    if error_str.contains("Git command not found") {
        EX_UNAVAILABLE // Service unavailable - Git is required but not available
    } else if error_str.contains("Not a Git repository") {
        EX_NOINPUT // Cannot open input - Git repository is required but not found
    } else if error_str.contains("Permission denied") {
        EX_NOPERM // Permission denied
    } else if error_str.contains("Directory traversal") || error_str.contains("Invalid path") {
        EX_DATAERR // Data format error - invalid path provided
    } else if error_str.contains("Path cannot be empty") || error_str.contains("Invalid characters")
    {
        EX_USAGE // Command line usage error - invalid arguments
    } else if error_str.contains("could not lock config file") {
        EX_TEMPFAIL // Temporary failure - user can retry
    } else if error_str.contains("Configuration failed") {
        EX_CONFIG // Configuration error
    } else if error_str.contains("IO error") || error_str.contains("Failed to") {
        EX_IOERR // Input/output error
    } else {
        EX_SOFTWARE // Internal software error - fallback for unknown errors
    }
}

#[cfg(test)]
mod tests {
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
}
