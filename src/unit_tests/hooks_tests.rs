use super::*;
use crate::environment::mocks::MockFileSystem;

#[test]
fn test_create_hook_directory() {
    let fs = MockFileSystem::new();
    let hooks_dir = std::path::Path::new(".samoyed/_");

    let result = create_hook_directory(&fs, hooks_dir);
    assert!(result.is_ok());

    // Verify the mock filesystem recorded the operations
    assert!(fs.exists(hooks_dir));
    assert!(fs.exists(&hooks_dir.join(".gitignore")));
}

#[test]
fn test_create_hook_files() {
    let fs = MockFileSystem::new();
    let hooks_dir = std::path::Path::new(".samoyed/_");

    let result = create_hook_files(&fs, hooks_dir);
    assert!(result.is_ok());

    // Verify all hooks were created
    for &hook in STANDARD_HOOKS {
        assert!(fs.exists(&hooks_dir.join(hook)));
    }
}

#[test]
fn test_create_example_hook_scripts() {
    let fs = MockFileSystem::new();
    let hooks_base_dir = std::path::Path::new(".samoyed");

    let result = create_example_hook_scripts(&fs, hooks_base_dir);
    assert!(result.is_ok());

    // Verify example scripts were created in scripts subdirectory
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
}

#[test]
fn test_create_example_hook_scripts_no_overwrite() {
    let fs = MockFileSystem::new().with_file(
        ".samoyed/scripts/pre-commit",
        "#!/bin/sh\n# User's existing script",
    );
    let hooks_base_dir = std::path::Path::new(".samoyed");

    let result = create_example_hook_scripts(&fs, hooks_base_dir);
    assert!(result.is_ok());

    // Verify existing file was not overwritten (still exists)
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
    // Verify other example was still created
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
}

#[test]
fn test_hook_error_display() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let hook_error = HookError::IoError(io_error);
    assert!(hook_error.to_string().contains("Permission denied"));
}

#[test]
fn test_hook_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let hook_error: HookError = io_error.into();
    assert!(matches!(hook_error, HookError::IoError(_)));
}

#[test]
fn test_hook_error_variants_coverage() {
    // Test all HookError variants for coverage
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let error1 = HookError::IoError(io_error);

    // Ensure all implement Debug and Display
    assert!(!format!("{error1:?}").is_empty());
    assert!(error1.to_string().contains("IO error"));
}

#[test]
fn test_standard_hooks_constant() {
    // Test that STANDARD_HOOKS contains expected hooks
    assert!(STANDARD_HOOKS.contains(&"pre-commit"));
    assert!(STANDARD_HOOKS.contains(&"post-commit"));
    assert!(STANDARD_HOOKS.contains(&"pre-push"));
    assert_eq!(STANDARD_HOOKS.len(), 14);
}

#[test]
fn test_create_example_hook_scripts_multiple_calls() {
    let fs = MockFileSystem::new();
    let hooks_base_dir = std::path::Path::new(".samoyed");

    // First call should create examples
    let result1 = create_example_hook_scripts(&fs, hooks_base_dir);
    assert!(result1.is_ok());

    // Second call should not fail (examples already exist)
    let result2 = create_example_hook_scripts(&fs, hooks_base_dir);
    assert!(result2.is_ok());

    // Verify scripts still exist
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-commit")));
    assert!(fs.exists(&hooks_base_dir.join("scripts/pre-push")));
}

#[test]
fn test_create_hook_files_with_multiple_directories() {
    let fs = MockFileSystem::new();

    // Test with different hook directories
    let hooks_dir1 = std::path::Path::new(".hooks/_");
    let result1 = create_hook_files(&fs, hooks_dir1);
    assert!(result1.is_ok());

    let hooks_dir2 = std::path::Path::new("custom/hooks");
    let result2 = create_hook_files(&fs, hooks_dir2);
    assert!(result2.is_ok());

    // Verify all hooks were created in both directories
    for &hook in STANDARD_HOOKS {
        assert!(fs.exists(&hooks_dir1.join(hook)));
        assert!(fs.exists(&hooks_dir2.join(hook)));
    }
}

#[test]
fn test_create_hook_directory_with_multiple_paths() {
    let fs = MockFileSystem::new();

    // Test creating hook directories with different paths
    let dirs = [
        std::path::Path::new(".samoyed/_"),
        std::path::Path::new(".hooks"),
        std::path::Path::new("custom/hooks/dir"),
    ];

    for dir in &dirs {
        let result = create_hook_directory(&fs, dir);
        assert!(result.is_ok());
        assert!(fs.exists(dir));
        assert!(fs.exists(&dir.join(".gitignore")));
    }
}

#[test]
fn test_create_example_hook_scripts_different_directories() {
    let fs = MockFileSystem::new();

    let hooks_base_dir1 = std::path::Path::new(".hooks1");
    let hooks_base_dir2 = std::path::Path::new(".hooks2");
    let hooks_base_dir3 = std::path::Path::new(".samoyed");

    // Test creating examples in different directories
    let result1 = create_example_hook_scripts(&fs, hooks_base_dir1);
    assert!(result1.is_ok());
    assert!(fs.exists(&hooks_base_dir1.join("scripts/pre-commit")));
    assert!(fs.exists(&hooks_base_dir1.join("scripts/pre-push")));

    let result2 = create_example_hook_scripts(&fs, hooks_base_dir2);
    assert!(result2.is_ok());
    assert!(fs.exists(&hooks_base_dir2.join("scripts/pre-commit")));
    assert!(fs.exists(&hooks_base_dir2.join("scripts/pre-push")));

    let result3 = create_example_hook_scripts(&fs, hooks_base_dir3);
    assert!(result3.is_ok());
    assert!(fs.exists(&hooks_base_dir3.join("scripts/pre-commit")));
    assert!(fs.exists(&hooks_base_dir3.join("scripts/pre-push")));
}

#[test]
fn test_hook_error_error_trait() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let hook_error = HookError::IoError(io_error);

    // Test that it implements std::error::Error
    let error_trait: &dyn std::error::Error = &hook_error;
    assert!(!error_trait.to_string().is_empty());
}

#[test]
fn test_normalize_line_endings_crlf() {
    let windows_content = "#!/bin/sh\r\necho 'hello'\r\necho 'world'\r\n";
    let normalized = normalize_line_endings(windows_content);
    assert_eq!(normalized, "#!/bin/sh\necho 'hello'\necho 'world'\n");
}

#[test]
fn test_normalize_line_endings_cr() {
    let mac_classic_content = "#!/bin/sh\recho 'hello'\recho 'world'\r";
    let normalized = normalize_line_endings(mac_classic_content);
    assert_eq!(normalized, "#!/bin/sh\necho 'hello'\necho 'world'\n");
}

#[test]
fn test_normalize_line_endings_mixed() {
    let mixed_content = "#!/bin/sh\r\necho 'hello'\recho 'world'\necho 'end'";
    let normalized = normalize_line_endings(mixed_content);
    assert_eq!(
        normalized,
        "#!/bin/sh\necho 'hello'\necho 'world'\necho 'end'"
    );
}

#[test]
fn test_normalize_line_endings_already_lf() {
    let unix_content = "#!/bin/sh\necho 'hello'\necho 'world'\n";
    let normalized = normalize_line_endings(unix_content);
    assert_eq!(normalized, unix_content); // Should be unchanged
}

#[test]
fn test_normalize_line_endings_empty() {
    let empty_content = "";
    let normalized = normalize_line_endings(empty_content);
    assert_eq!(normalized, "");
}
