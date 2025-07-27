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
pub trait Environment {
    fn get_var(&self, key: &str) -> Option<String>;
    fn set_var(&mut self, key: &str, value: &str);
    fn remove_var(&mut self, key: &str);
    fn current_dir(&self) -> io::Result<PathBuf>;
    fn set_current_dir(&mut self, path: &Path) -> io::Result<()>;
}
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
- **Total Tests**: 51 (20 unit tests + 8 integration tests + 23 additional)
- **Execution Time**: Reduced from ~30s to ~2s (15x faster)
- **Parallel Execution**: Now safe and reliable

### Coverage Analysis
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

The dependency injection refactoring successfully transformed our unreliable test suite into a fast, deterministic, and maintainable testing infrastructure. The pattern provides a solid foundation for continued development while ensuring high code quality through reliable automated testing.

The investment in proper abstraction and test isolation pays dividends in development velocity, code confidence, and maintenance ease. This approach should be considered for any Rust project dealing with external dependencies in tests.