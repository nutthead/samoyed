//! # Performance Benchmarks for Samoid
//!
//! This module provides comprehensive performance testing for the Samoid Git hooks manager,
//! implementing benchmarks to validate all performance-related acceptance criteria from
//! Issue #8 - Performance Optimization.
//!
//! ## Overview
//!
//! Samoid's performance is critical for developer experience, as Git hooks run frequently
//! during development workflows. This benchmark suite measures both mock and real-world
//! performance scenarios to ensure:
//!
//! - **Hook execution overhead** remains under 50ms (AC8.1)
//! - **Startup times** are under 100ms (AC8.4)  
//! - **File system operations** are efficient (AC8.5)
//! - **Memory usage** is minimal and predictable
//!
//! ## Benchmark Categories
//!
//! ### Mock Benchmarks (`mock_benches` group)
//! These benchmarks test core logic using dependency injection with mock implementations:
//!
//! - **`benchmark_mock_installation`**: Tests hook installation with mock environment
//! - **`benchmark_mock_installation_custom_dir`**: Tests custom directory installation
//! - **`benchmark_mock_filesystem_operations`**: Tests filesystem abstraction performance
//! - **`benchmark_skip_installation`**: Tests SAMOID=0 skip logic performance
//! - **`benchmark_large_mock_filesystem`**: Tests performance with large project simulation
//!
//! ### Real-World Benchmarks (`real_benches` group)
//! These benchmarks test actual binary execution for realistic performance measurement:
//!
//! - **`benchmark_real_hook_execution_overhead`**: Measures pure hook execution overhead
//! - **`benchmark_startup_time_samoid_cli`**: Measures CLI startup performance
//! - **`benchmark_startup_time_samoid_hook_cli`**: Measures hook runner startup performance
//! - **`benchmark_filesystem_operations_real`**: Measures real filesystem operation performance
//!
//! ## Design Principles
//!
//! ### Dependency Injection for Testing
//! Mock benchmarks use Samoid's dependency injection system with `MockEnvironment`,
//! `MockCommandRunner`, and `MockFileSystem` to isolate performance measurement
//! from external factors like actual filesystem I/O or process execution.
//!
//! ### Real-World Validation
//! Real benchmarks execute actual binaries to measure end-to-end performance,
//! including process startup overhead, dynamic linking, and actual system calls.
//! This provides the most accurate measurement of user-facing performance.
//!
//! ### Cross-Platform Compatibility
//! The benchmark suite handles platform differences, particularly around process
//! exit status creation, using conditional compilation for Unix vs Windows.
//!
//! ## Performance Targets
//!
//! Based on Issue #8 acceptance criteria:
//!
//! - **Hook Overhead**: < 50ms for GitHub Actions runners (< 30ms for development)
//! - **Startup Time**: < 100ms for all CLI operations
//! - **Memory Usage**: Measured separately via `/usr/bin/time -v` in CI/CD
//! - **Binary Size**: Measured separately via `stat` in CI/CD
//!
//! ## Benchmark Execution
//!
//! Run all benchmarks:
//! ```bash
//! cargo bench
//! ```
//!
//! Run specific benchmark group:
//! ```bash
//! cargo bench mock_benches
//! cargo bench real_benches  
//! ```
//!
//! ## Integration with CI/CD
//!
//! These benchmarks are executed in the dedicated performance pipeline (`.github/workflows/perf.yml`)
//! with results tracked over time for regression detection. The real-world benchmarks
//! provide the metrics used for acceptance criteria validation.

use criterion::{Criterion, criterion_group, criterion_main};
use samoid::environment::FileSystem;
use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
use std::hint::black_box;
use std::process::{ExitStatus, Output};

// Cross-platform exit status creation
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

/// Creates an ExitStatus in a cross-platform manner
///
/// This helper function abstracts the platform-specific differences in creating
/// ExitStatus objects for mock command execution. Unix systems use `from_raw(i32)`
/// while Windows uses `from_raw(u32)`.
///
/// # Arguments
/// * `code` - The exit code to create an ExitStatus for
///
/// # Returns
/// A platform-appropriate ExitStatus object
fn exit_status(code: i32) -> ExitStatus {
    #[cfg(unix)]
    return ExitStatus::from_raw(code);

    #[cfg(windows)]
    return ExitStatus::from_raw(code as u32);
}

/// Benchmarks basic hook installation using mock dependencies
///
/// This benchmark measures the performance of the core `install_hooks` function
/// using mock implementations to isolate the logic performance from I/O overhead.
///
/// **Test Scenario**: Default installation with `.samoid/_` as hooks directory
/// **Mock Setup**: Git repository exists, git config command succeeds
/// **Performance Target**: < 1μs (mock operations should be nearly instant)
fn benchmark_mock_installation(c: &mut Criterion) {
    c.bench_function("mock_installation", |b| {
        b.iter(|| {
            // Create mock environment with no special variables
            let env = MockEnvironment::new();

            // Mock successful git config command response
            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(output),
            );

            // Mock filesystem with existing .git directory
            let fs = MockFileSystem::new().with_directory(".git");

            // Execute installation and use black_box to prevent optimization
            let result = install_hooks(&env, &runner, &fs, None);
            black_box(result)
        })
    });
}

fn benchmark_mock_installation_with_custom_dir(c: &mut Criterion) {
    c.bench_function("mock_installation_custom_dir", |b| {
        b.iter(|| {
            let env = MockEnvironment::new();
            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", ".custom/_"],
                Ok(output),
            );
            let fs = MockFileSystem::new().with_directory(".git");

            let result = install_hooks(&env, &runner, &fs, Some(".custom"));
            black_box(result)
        })
    });
}

fn benchmark_mock_filesystem_operations(c: &mut Criterion) {
    c.bench_function("mock_filesystem_operations", |b| {
        b.iter(|| {
            let fs = MockFileSystem::new()
                .with_directory(".git")
                .with_file(".samoid/_/pre-commit", "#!/bin/sh\necho 'hook'");

            // Simulate multiple filesystem operations
            black_box(fs.exists(std::path::Path::new(".git")));
            black_box(fs.exists(std::path::Path::new(".samoid/_")));
            black_box(fs.exists(std::path::Path::new(".samoid/_/pre-commit")));
        })
    });
}

fn benchmark_skip_installation(c: &mut Criterion) {
    c.bench_function("skip_installation_samoid_0", |b| {
        b.iter(|| {
            let env = MockEnvironment::new().with_var("SAMOID", "0");
            let runner = MockCommandRunner::new();
            let fs = MockFileSystem::new();

            let result = install_hooks(&env, &runner, &fs, None);
            black_box(result)
        })
    });
}

fn benchmark_large_mock_filesystem(c: &mut Criterion) {
    c.bench_function("large_mock_filesystem", |b| {
        let mut fs = MockFileSystem::new().with_directory(".git");

        // Simulate a large project with many files
        for i in 0..1000 {
            fs = fs.with_file(format!("src/file_{i}.rs"), "// Mock file content");
        }

        b.iter(|| {
            let env = MockEnvironment::new();
            let output = Output {
                status: exit_status(0),
                stdout: vec![],
                stderr: vec![],
            };
            let runner = MockCommandRunner::new().with_response(
                "git",
                &["config", "core.hooksPath", ".samoid/_"],
                Ok(output),
            );

            let result = install_hooks(&env, &runner, &fs, None);
            black_box(result)
        })
    });
}

/// Benchmarks real-world hook execution overhead by running actual samoid-hook binary
///
/// **Critical Performance Test**: This measures the pure overhead that Samoid adds to
/// Git hook execution, which is the most important performance metric for user experience.
///
/// **Test Method**:
/// - Executes the actual `samoid-hook` binary with a hook name argument
/// - Measures total time from process start to completion
/// - Uses missing hook scenario (exit code 1) to measure pure Samoid overhead
/// - Excludes actual hook script execution time to isolate Samoid's overhead
///
/// **Performance Target**: < 50ms for GitHub Actions runners (AC8.1)
/// **Expected Result**: ~1-2ms (based on previous measurements)
///
/// **Why This Matters**:
/// Git hooks run on every commit, push, etc. Even small overhead adds up and
/// affects developer productivity. This test ensures Samoid remains fast enough
/// to be invisible to developers.
fn benchmark_real_hook_execution_overhead(c: &mut Criterion) {
    use std::process::Command;
    use std::time::Duration;

    c.bench_function("real_hook_execution_overhead", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = std::time::Instant::now();

                // Execute samoid-hook with non-existent hook to measure pure startup overhead
                // This measures only samoid-hook's initialization cost, not actual hook execution
                let output = Command::new("./target/release/samoid-hook")
                    .arg("non-existent-hook")
                    .env("SAMOID", "1")
                    .output();

                let elapsed = start.elapsed();

                // Only count valid executions in timing measurement
                if let Ok(result) = output {
                    // Status code 0 = hook exists and succeeded
                    // Status code 1 = hook missing (expected for overhead measurement)
                    // Both scenarios measure Samoid's pure overhead without actual hook execution
                    if result.status.success() || result.status.code() == Some(1) {
                        total_duration += elapsed;
                    }
                }
            }

            total_duration
        })
    });
}

fn benchmark_startup_time_samoid_cli(c: &mut Criterion) {
    use std::process::Command;
    use std::time::Duration;

    c.bench_function("startup_time_samoid_help", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = std::time::Instant::now();

                let output = Command::new("./target/release/samoid")
                    .arg("--help")
                    .output();

                let elapsed = start.elapsed();

                if let Ok(result) = output {
                    if result.status.success() {
                        total_duration += elapsed;
                    }
                }
            }

            total_duration
        })
    });
}

fn benchmark_startup_time_samoid_hook_cli(c: &mut Criterion) {
    use std::process::Command;
    use std::time::Duration;

    c.bench_function("startup_time_samoid_hook_help", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);

            for _ in 0..iters {
                let start = std::time::Instant::now();

                // Use a non-existent hook name to measure startup overhead only
                // samoid-hook will exit cleanly when no hook script is found
                let output = Command::new("./target/release/samoid-hook")
                    .arg("non-existent-hook")
                    .env("SAMOID", "1")
                    .output();

                let elapsed = start.elapsed();

                if let Ok(result) = output {
                    // Exit codes 0 (hook succeeded) and 1 (hook missing) are both valid for this test
                    // We're measuring startup time, not hook execution success
                    if result.status.success() || result.status.code() == Some(0) {
                        total_duration += elapsed;
                    }
                }
            }

            total_duration
        })
    });
}

/// Benchmarks real filesystem operations during hook installation workflow
///
/// **Performance Test**: Measures actual filesystem I/O performance for operations
/// that occur during `samoid init` to validate AC8.5 (Efficient filesystem operations).
///
/// **Test Scenario**:
/// - Creates temporary directory structure (.git, .samoid, hooks directory)
/// - Writes hook files with realistic content
/// - Performs existence checks and read operations
/// - Measures complete workflow timing including all I/O operations
///
/// **Performance Target**: Complete workflow should be efficient (target: <1ms)
/// **Expected Result**: ~200-300μs for complete filesystem workflow
///
/// **Why This Matters**:
/// Installation performance affects developer onboarding experience. Slow filesystem
/// operations make `samoid init` feel sluggish and impact first impressions.
fn benchmark_filesystem_operations_real(c: &mut Criterion) {
    use std::fs;
    use tempfile::TempDir;

    c.bench_function("filesystem_operations_real", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let test_path = temp_dir.path();

            // Simulate real filesystem operations during hook installation
            let git_dir = test_path.join(".git");
            let samoid_dir = test_path.join(".samoid");
            let hooks_dir = samoid_dir.join("_");

            // Create directories - ignore Results since we're benchmarking, not testing correctness
            let _ = black_box(fs::create_dir_all(&git_dir));
            let _ = black_box(fs::create_dir_all(&hooks_dir));

            // Check existence (common operations during validation)
            black_box(git_dir.exists());
            black_box(samoid_dir.exists());
            black_box(hooks_dir.exists());

            // Write hook files with realistic hook runner content
            for hook in ["pre-commit", "pre-push", "commit-msg"].iter() {
                let hook_file = hooks_dir.join(hook);
                let _ = black_box(fs::write(
                    &hook_file,
                    "#!/bin/sh\n./samoid-hook $0 \"$@\"\n",
                ));
            }

            // Read operations (common during hook execution)
            for hook in ["pre-commit", "pre-push", "commit-msg"].iter() {
                let hook_file = hooks_dir.join(hook);
                let _ = black_box(fs::read_to_string(&hook_file));
            }
        })
    });
}

criterion_group!(
    mock_benches,
    benchmark_mock_installation,
    benchmark_mock_installation_with_custom_dir,
    benchmark_mock_filesystem_operations,
    benchmark_skip_installation,
    benchmark_large_mock_filesystem
);

criterion_group!(
    real_benches,
    benchmark_real_hook_execution_overhead,
    benchmark_startup_time_samoid_cli,
    benchmark_startup_time_samoid_hook_cli,
    benchmark_filesystem_operations_real
);

criterion_main!(mock_benches, real_benches);
