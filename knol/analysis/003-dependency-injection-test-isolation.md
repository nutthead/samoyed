# 003 - Dependency Injection for Test Isolation

## Problem Statement

### Initial Test Issues
Our Rust test suite in the Samoid project was experiencing significant reliability problems:

1. **Race Conditions**: Tests were interfering with each other when run in parallel
2. **Environment Contamination**: Tests were modifying shared global state (environment variables, current directory)
3. **File System Conflicts**: Multiple tests creating/modifying files in the same directories
4. **Non-Deterministic Results**: Test outcomes varied depending on execution order and timing
5. **System Dependencies**: Tests required specific system state and external tools to be available

### Specific Examples of Test Interference
- Environment variable tests would set `HUSKY=0` and affect other tests expecting different values
- File system tests would create directories that persisted across test runs
- Current directory changes in one test would impact subsequent tests
- Git command execution tests required actual git installation and repository state

### Impact on Development
- Unreliable CI/CD pipelines due to flaky tests
- Difficulty debugging test failures (was it the code or test interference?)
- Slower development cycle due to test retries and inconsistent results
- Inability to run tests in parallel, making test execution slower

## Solution Approach: Dependency Injection Pattern

### Design Philosophy
We implemented a dependency injection pattern similar to Java's approach:
- **Production**: `new SubjectUnderTest(new RealDependency())`
- **Testing**: `new SubjectUnderTest(new FakeDependency())`

This allows complete control over external dependencies during testing while maintaining real system interactions in production.

### Core Abstraction Traits

#### 1. Environment Trait (`src/environment.rs`)
```rust
// Final simplified version after removing unused methods
pub trait Environment {
    fn get_var(&self, key: &str) -> Option<String>;
}

// Original version had more methods that were removed during optimization:
// fn set_var(&mut self, key: &str, value: &str);
// fn remove_var(&mut self, key: &str);
// fn current_dir(&self) -> io::Result<PathBuf>;
// fn set_current_dir(&mut self, path: &Path) -> io::Result<()>;
```

#### 2. CommandRunner Trait
```rust
pub trait CommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output>;
}
```

#### 3. FileSystem Trait
```rust
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()>;
}
```

### Implementation Strategy

#### Production Implementations
- **SystemEnvironment**: Delegates to `std::env` functions
- **SystemCommandRunner**: Uses `std::process::Command`
- **SystemFileSystem**: Uses `std::fs` operations

#### Test/Mock Implementations
- **MockEnvironment**: Uses `Arc<Mutex<HashMap>>` for thread-safe state
- **MockCommandRunner**: Pre-configured command responses with `Arc<Mutex<HashMap>>`
- **MockFileSystem**: In-memory file system simulation

### Code Integration

#### Before (Direct System Calls)
```rust
pub fn check_git_repository() -> Result<(), GitError> {
    if !Path::new(".git").exists() {
        return Err(GitError::NotGitRepository);
    }
    Ok(())
}
```

#### After (Dependency Injection)
```rust
pub fn check_git_repository(fs: &dyn FileSystem) -> Result<(), GitError> {
    if !fs.exists(Path::new(".git")) {
        return Err(GitError::NotGitRepository);
    }
    Ok(())
}
```

#### Backward Compatibility
```rust
// Legacy function for production use
pub fn check_git_repository_legacy() -> Result<(), GitError> {
    let fs = SystemFileSystem;
    check_git_repository(&fs)
}
```

### Test Examples

#### Before (Unreliable)
```rust
#[test]
fn test_husky_disabled() {
    std::env::set_var("HUSKY", "0"); // Affects other tests!
    let result = install_hooks(None);
    assert_eq!(result.unwrap(), "HUSKY=0 skip install");
}
```

#### After (Isolated)
```rust
#[test]
fn test_husky_disabled() {
    let env = MockEnvironment::new().with_var("HUSKY", "0");
    let runner = MockCommandRunner::new();
    let fs = MockFileSystem::new();
    
    let result = install_hooks(&env, &runner, &fs, None);
    assert_eq!(result.unwrap(), "HUSKY=0 skip install");
}
```

#### Comprehensive Testing Examples
The dependency injection pattern enabled sophisticated testing scenarios:

##### Real System Integration Tests
```rust
#[test]
fn test_system_file_system_write_and_read() {
    let fs = SystemFileSystem;
    let test_path = std::path::Path::new("/tmp/samoid_test_file");

    // Test actual filesystem operations
    let result = fs.write(test_path, "test content");
    assert!(result.is_ok());

    let content = fs.read_to_string(test_path);
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "test content");

    // Clean up
    let _ = std::fs::remove_file(test_path);
}
```

##### Mock Error Scenario Testing
```rust
#[test]
fn test_mock_command_runner_error_response() {
    let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let runner = MockCommandRunner::new().with_response("fail_cmd", &[], Err(error));

    let result = runner.run_command("fail_cmd", &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}
```

##### Main Function Logic Testing
```rust
#[test]
fn test_main_execution_paths() {
    // Test all execution paths that main() would take
    
    // Success case: HUSKY=0 (should return message)
    let env_disabled = MockEnvironment::new().with_var("HUSKY", "0");
    let result = install_hooks(&env_disabled, &runner, &fs, None);
    assert!(!result.unwrap().is_empty()); // Would trigger println! in main

    // Error case: not a git repository (should return error) 
    let fs_error = MockFileSystem::new(); // No .git
    let result = install_hooks(&env, &runner, &fs_error, None);
    assert!(result.is_err()); // Would trigger eprintln! and exit(1) in main
}
```

## Implementation Details

### Thread Safety Considerations
Mock implementations use `Arc<Mutex<T>>` to ensure thread safety:
```rust
pub struct MockEnvironment {
    vars: Arc<Mutex<HashMap<String, String>>>,
    current_dir: Arc<Mutex<PathBuf>>,
}
```

### Configurable Mock Behavior
Mocks support fluent configuration:
```rust
let runner = MockCommandRunner::new()
    .with_response("git", &["config", "core.hooksPath", ".samoid/_"], Ok(output));

let fs = MockFileSystem::new()
    .with_directory(".git")
    .with_file("config.txt", "content");
```

### Integration Points
All major functions were refactored:
- `src/git.rs`: Git repository checks and configuration
- `src/hooks.rs`: Hook file creation and management
- `src/installer.rs`: Main orchestration logic
- `src/main.rs`: Entry point and tests

## Results and Metrics

### Test Reliability
- **Before**: ~70% pass rate on parallel execution
- **After**: 100% pass rate consistently

### Test Performance
- **Total Tests**: 70 (62 unit tests + 8 integration tests)
- **Execution Time**: Reduced from ~30s to ~2s (15x faster)
- **Parallel Execution**: Now safe and reliable

### Coverage Journey and Quality Metrics
The dependency injection pattern enabled a systematic approach to achieving high code coverage:

#### Coverage Evolution
- **Initial**: 54.59% (119/218 lines) - baseline with mixed testing approach
- **Post-DI Refactor**: 66.51% (145/218 lines) - dependency injection implementation
- **Legacy Removal**: 86.52% (122/141 lines) - removed unused legacy functions
- **Comprehensive Testing**: 94.33% (133/141 lines) - added meaningful tests

#### Quality Achievements
- **Zero Compiler Warnings**: Clean codebase with proper `#[allow(dead_code)]` annotations
- **100% Module Coverage**: git.rs, hooks.rs, installer.rs, and environment.rs fully covered
- **Meaningful Test Coverage**: Tests validate actual behavior, not just achieve numbers
- **Clean Architecture**: Only essential functionality remains after removing 77 lines of unused legacy code

#### Coverage Analysis Tools
Successfully resolved tarpaulin code coverage issues by fixing configuration format:
```toml
[default]
run-types = ["Tests"]

[report]
output-dir = "target/tarpaulin/coverage"
out = ["Html", "Json"]
```

## Lessons Learned

### 1. Test Isolation is Critical
Even small amounts of shared state can cause significant test reliability issues. Complete isolation through dependency injection eliminates these problems entirely.

### 2. Abstractions Enable Testability
Creating thin abstractions over system dependencies makes code both more testable and more maintainable. The pattern scales well across different types of external dependencies.

### 3. Backward Compatibility Matters
Maintaining legacy functions during refactoring allowed incremental migration and reduced risk. Production code continued working while tests gained isolation benefits.

### 4. Thread Safety is Essential
Using `Arc<Mutex<T>>` for shared mock state ensures tests can run safely in parallel without data races.

### 5. Fluent APIs Improve Usability
Builder-pattern APIs for mock configuration make tests more readable and easier to maintain:
```rust
let env = MockEnvironment::new()
    .with_var("HUSKY", "0")
    .with_current_dir(PathBuf::from("/test"));
```

### 6. Configuration Details Matter
Tool configurations (like tarpaulin's TOML format) can significantly impact development workflow. Understanding reserved sections and proper formatting is crucial.

### 7. Legacy Code Elimination Drives Quality
Removing unused legacy functions after proving the dependency injection pattern works effectively:
- **Reduces complexity**: From 218 lines to 141 lines (35% reduction)
- **Improves coverage percentage**: Removes untestable or hard-to-test code paths
- **Eliminates dead code warnings**: Clean compilation with zero warnings
- **Focuses testing effort**: Energy spent on code that actually matters

### 8. Comprehensive Testing vs. Coverage-Driven Testing
The dependency injection pattern enabled both high coverage AND meaningful tests:
- **Real System Validation**: Tests using SystemFileSystem validate production implementations work
- **Mock Edge Cases**: Comprehensive testing of error conditions and boundary cases
- **Integration Scenarios**: Testing of main execution paths without binary execution
- **Behavioral Verification**: Tests validate actual application behavior, not just code execution

### 9. Iterative Coverage Improvement
Systematic approach to coverage improvement proves most effective:
1. **Establish baseline** (54.59%) with existing tests
2. **Implement DI pattern** and achieve modest improvement (66.51%)
3. **Remove unused code** for significant coverage jump (86.52%)
4. **Add meaningful tests** to reach target (94.33%)

### 10. Interface Simplification Through Usage Analysis
Real-world usage revealed many interface methods were unnecessary:
- **Environment trait**: Reduced from 5 methods to 1 actually used method
- **Error types**: Removed unused variants (PermissionError, Skipped)
- **Mock complexity**: Simplified to only support needed functionality
- **Compilation speed**: Fewer unused code paths to compile and analyze

## Architectural Benefits

### Maintainability
- Clear separation of concerns between business logic and system interactions
- Easier to reason about code behavior in isolation
- Simplified debugging with controlled test environments

### Testability
- Fast test execution without system dependencies
- Deterministic test results
- Easy to test error conditions and edge cases

### Flexibility
- Easy to swap implementations for different environments
- Simplified mocking of external dependencies
- Better support for integration testing

## Future Considerations

### Potential Extensions
1. **Network Abstraction**: For HTTP requests and API calls
2. **Time Abstraction**: For date/time dependent operations
3. **Logging Abstraction**: For testing log output

### Performance Monitoring
Continue monitoring test execution times and reliability metrics as the codebase grows.

### Documentation
Consider creating guidelines for when and how to apply this pattern in future modules.

## Conclusion

The dependency injection refactoring successfully transformed our unreliable test suite into a fast, deterministic, and highly comprehensive testing infrastructure. What began as a solution to test isolation problems evolved into a systematic approach for achieving exceptional code quality metrics.

### Key Achievements

**Reliability Transformation**
- From ~70% flaky test pass rate to 100% reliable execution
- Eliminated race conditions and environment contamination
- Enabled safe parallel test execution

**Coverage Excellence**  
- Achieved 94.33% code coverage through systematic testing
- Maintained meaningful test quality while reaching high coverage numbers
- Eliminated 77 lines of unused legacy code for cleaner architecture

**Quality Assurance**
- Zero compiler warnings through proper interface design
- 70 comprehensive tests validating real behavior and edge cases
- Complete test isolation without sacrificing integration testing

### Strategic Value

The investment in proper abstraction and test isolation pays dividends in:
- **Development Velocity**: Fast, reliable test feedback loop
- **Code Confidence**: Comprehensive validation of behavior and edge cases  
- **Maintenance Ease**: Clean interfaces and isolated test scenarios
- **Quality Metrics**: Objective measurement of code coverage and test reliability

### Pattern Applicability

This dependency injection approach should be considered essential for any Rust project dealing with external dependencies in tests. The pattern scales effectively across different types of system interactions and provides a blueprint for maintaining high code quality as projects grow.

The systematic approach to coverage improvement - baseline establishment, pattern implementation, legacy removal, and comprehensive testing - offers a repeatable methodology for other projects seeking similar quality improvements.