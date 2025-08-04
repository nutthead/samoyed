use super::DEPRECATION_REMOVAL_DATE;

#[test]
fn deprecation_constant_defined() {
    // Test that the deprecation constant is properly defined
    assert!(!DEPRECATION_REMOVAL_DATE.is_empty());
    assert!(DEPRECATION_REMOVAL_DATE.contains("2025"));
}

#[test]
fn deprecation_constant_format() {
    // Test that the deprecation date follows expected format
    assert!(DEPRECATION_REMOVAL_DATE.len() > 10); // Should be more than just "2025"
    assert!(DEPRECATION_REMOVAL_DATE.contains("September") || 
            DEPRECATION_REMOVAL_DATE.contains("2025")); // Should contain month or year
}

// NOTE: The main() function in hook_runner.rs cannot be easily unit tested
// because it calls process::exit() and Command::new("samoyed"), which are
// not mockable in this simple shim. The functionality is tested through
// integration tests that verify the deprecation warning is shown and
// the delegation to 'samoyed hook' works correctly.
//
// The critical business logic has been moved to main.rs and is thoroughly
// tested there with dependency injection.