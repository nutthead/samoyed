//! Secure logging utilities for Samoid
//!
//! This module provides logging functions that automatically sanitize sensitive
//! information to prevent accidental exposure of secrets, personal information,
//! or other sensitive data in logs.

use crate::environment::Environment;
use std::path::Path;

/// Sanitizes a file path for safe logging by redacting sensitive portions
///
/// This function removes or redacts potentially sensitive information from
/// file paths to prevent exposure in logs while maintaining enough information
/// for debugging purposes.
///
/// # Security Features
///
/// - Redacts home directory paths to prevent exposing usernames
/// - Limits path depth to prevent exposing full directory structures
/// - Redacts common sensitive directories (/.ssh, /.config, etc.)
/// - Preserves relative paths and project-specific information
///
/// # Arguments
///
/// * `env` - Environment provider for accessing environment variables
/// * `path` - The file path to sanitize
///
/// # Returns
///
/// A sanitized string representation of the path safe for logging
pub fn sanitize_path_with_env<P: AsRef<Path>>(env: &dyn Environment, path: P) -> String {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // Handle absolute paths (including Unix-style paths on Windows)
    if path.is_absolute() || path_str.starts_with('/') {
        // Check for home directory patterns
        if let Some(home) = env.get_var("HOME").or_else(|| env.get_var("USERPROFILE")) {
            // Normalize paths for cross-platform comparison
            let normalized_path = path_str.replace('\\', "/");
            let normalized_home = home.replace('\\', "/");

            if normalized_path.starts_with(&normalized_home) {
                let relative = normalized_path
                    .strip_prefix(&normalized_home)
                    .unwrap_or(&normalized_path);
                return format!("~{relative}");
            }
        }

        // Check for common sensitive directories and redact them
        let sensitive_patterns = [
            "/etc/passwd",
            "/etc/shadow",
            "/etc/hosts",
            "/.ssh/",
            "/.gnupg/",
            "/proc/",
            "/sys/",
        ];

        for pattern in &sensitive_patterns {
            if path_str.contains(pattern) {
                return "[REDACTED_SENSITIVE_PATH]".to_string();
            }
        }

        // For other absolute paths, show only the last few components
        let components: Vec<_> = path.components().collect();
        if components.len() > 3 {
            let last_three: Vec<String> = components
                .iter()
                .rev()
                .take(3)
                .rev()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect();
            return format!(".../{}", last_three.join("/"));
        }
    }

    // For relative paths, return as-is (they're generally safe)
    path_str.to_string()
}

/// Convenience function for sanitizing paths without environment context
///
/// This is a simpler version that doesn't redact home directories but still
/// protects against other sensitive path exposure.
pub fn sanitize_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    // Check for common sensitive directories and redact them
    let sensitive_patterns = [
        "/etc/passwd",
        "/etc/shadow",
        "/etc/hosts",
        "/.ssh/",
        "/.gnupg/",
        "/proc/",
        "/sys/",
    ];

    for pattern in &sensitive_patterns {
        if path_str.contains(pattern) {
            return "[REDACTED_SENSITIVE_PATH]".to_string();
        }
    }

    // For absolute paths, show only the last few components
    if path.is_absolute() {
        let components: Vec<_> = path.components().collect();
        if components.len() > 3 {
            let last_three: Vec<String> = components
                .iter()
                .rev()
                .take(3)
                .rev()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect();
            return format!(".../{}", last_three.join("/"));
        }
    }

    // For relative paths, return as-is (they're generally safe)
    path_str.to_string()
}

/// Sanitizes command arguments for safe logging
///
/// This function removes or redacts potentially sensitive information from
/// command arguments while preserving enough context for debugging.
///
/// # Security Features
///
/// - Redacts arguments that look like passwords, tokens, or keys
/// - Masks environment variable values
/// - Preserves command structure for debugging
///
/// # Arguments
///
/// * `args` - The command arguments to sanitize
///
/// # Returns
///
/// A sanitized vector of arguments safe for logging
pub fn sanitize_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            // Check for sensitive patterns
            let lower_arg = arg.to_lowercase();

            // Redact obvious secrets
            if lower_arg.contains("password")
                || lower_arg.contains("token")
                || lower_arg.contains("secret")
                || lower_arg.contains("key=")
                || lower_arg.starts_with("--password")
                || lower_arg.starts_with("--token")
            {
                "[REDACTED]".to_string()
            }
            // Redact long base64-like strings (potential tokens)
            else if arg.len() > 32
                && arg
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                "[REDACTED_TOKEN]".to_string()
            }
            // Sanitize file paths in arguments
            else if arg.contains('/') || arg.contains('\\') {
                sanitize_path(arg)
            } else {
                arg.clone()
            }
        })
        .collect()
}

/// Sanitizes environment variable name-value pairs for logging
///
/// This function redacts sensitive environment variables while preserving
/// non-sensitive ones for debugging purposes.
///
/// # Arguments
///
/// * `name` - The environment variable name
/// * `value` - The environment variable value
///
/// # Returns
///
/// A sanitized value safe for logging, or None if the variable should be completely hidden
#[allow(dead_code)] // Function is part of public API but not used internally yet
pub fn sanitize_env_var(name: &str, value: &str) -> Option<String> {
    let lower_name = name.to_lowercase();

    // Completely hide sensitive environment variables
    let sensitive_vars = [
        "password",
        "secret",
        "token",
        "key",
        "api_key",
        "auth",
        "ssh_",
        "gpg_",
        "pgp_",
        "private",
        "cert",
        "credential",
    ];

    for sensitive in &sensitive_vars {
        if lower_name.contains(sensitive) {
            return None; // Don't log at all
        }
    }

    // Redact but show presence of semi-sensitive variables
    let semi_sensitive = [
        "path", "home", "user", "pwd", "tmp", "temp", "config", "cache", "data",
    ];

    for semi in &semi_sensitive {
        if lower_name.contains(semi) {
            return Some(format!("[REDACTED_{}_VALUE]", name.to_uppercase()));
        }
    }

    // Safe to log non-sensitive environment variables
    Some(value.to_string())
}

/// Secure debug logging macro that automatically sanitizes arguments
///
/// This macro provides debug logging with automatic sanitization of potentially
/// sensitive information. Use this instead of direct eprintln! for debug output.
#[macro_export]
macro_rules! debug_log {
    ($enabled:expr, $($arg:tt)*) => {
        if $enabled {
            eprintln!($($arg)*);
        }
    };
}

/// Secure logging for file operations with environment context
pub fn log_file_operation_with_env(
    env: &dyn Environment,
    debug_mode: bool,
    operation: &str,
    path: &Path,
) {
    if debug_mode {
        eprintln!(
            "samoyed: {} file: {}",
            operation,
            sanitize_path_with_env(env, path)
        );
    }
}

/// Secure logging for file operations
#[allow(dead_code)] // Function is part of public API but not used internally yet
pub fn log_file_operation(debug_mode: bool, operation: &str, path: &Path) {
    if debug_mode {
        eprintln!("samoyed: {} file: {}", operation, sanitize_path(path));
    }
}

/// Secure logging for command execution
pub fn log_command_execution(debug_mode: bool, command: &str, args: &[String]) {
    if debug_mode {
        let sanitized_args = sanitize_args(args);
        eprintln!("samoyed: Executing command: {command} {sanitized_args:?}");
    }
}

#[cfg(test)]
#[path = "unit_tests/logging_tests.rs"]
mod tests;
