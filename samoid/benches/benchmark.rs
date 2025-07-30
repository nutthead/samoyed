//! Performance benchmarks for Samoid
//!
//! These benchmarks measure the performance of core operations to ensure
//! Samoid meets performance requirements and detects regressions.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use samoid::environment::FileSystem;
use samoid::environment::mocks::{MockCommandRunner, MockEnvironment, MockFileSystem};
use samoid::install_hooks;
use std::process::{ExitStatus, Output};

// Cross-platform exit status creation
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

// Helper function to create ExitStatus cross-platform
fn exit_status(code: i32) -> ExitStatus {
    #[cfg(unix)]
    return ExitStatus::from_raw(code);

    #[cfg(windows)]
    return ExitStatus::from_raw(code as u32);
}

fn benchmark_mock_installation(c: &mut Criterion) {
    c.bench_function("mock_installation", |b| {
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
            let fs = MockFileSystem::new().with_directory(".git");

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

fn benchmark_real_hook_execution_overhead(c: &mut Criterion) {
    use std::process::Command;
    use std::time::Duration;
    
    c.bench_function("real_hook_execution_overhead", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            
            for _ in 0..iters {
                let start = std::time::Instant::now();
                
                // Test pure samoid-hook startup overhead with empty hook
                let output = Command::new("./target/release/samoid-hook")
                    .arg("pre-commit")
                    .env("SAMOID", "1")
                    .output();
                
                let elapsed = start.elapsed();
                
                // Only count successful executions in timing
                if let Ok(result) = output {
                    if result.status.success() || result.status.code() == Some(1) {
                        // Status code 1 is expected for missing hook script - that's fine for overhead measurement
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
                
                let output = Command::new("./target/release/samoid-hook")
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

fn benchmark_filesystem_operations_real(c: &mut Criterion) {
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;
    
    c.bench_function("filesystem_operations_real", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let test_path = temp_dir.path();
            
            // Simulate real filesystem operations during hook installation
            let git_dir = test_path.join(".git");
            let samoid_dir = test_path.join(".samoid");
            let hooks_dir = samoid_dir.join("_");
            
            // Create directories
            black_box(fs::create_dir_all(&git_dir));
            black_box(fs::create_dir_all(&hooks_dir));
            
            // Check existence (common operations)
            black_box(git_dir.exists());
            black_box(samoid_dir.exists());
            black_box(hooks_dir.exists());
            
            // Write hook files
            for hook in ["pre-commit", "pre-push", "commit-msg"].iter() {
                let hook_file = hooks_dir.join(hook);
                black_box(fs::write(&hook_file, "#!/bin/sh\n./samoid-hook $0 \"$@\"\n"));
            }
            
            // Read operations
            for hook in ["pre-commit", "pre-push", "commit-msg"].iter() {
                let hook_file = hooks_dir.join(hook);
                black_box(fs::read_to_string(&hook_file));
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
