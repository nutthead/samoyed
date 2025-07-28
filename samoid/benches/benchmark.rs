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

criterion_group!(
    benches,
    benchmark_mock_installation,
    benchmark_mock_installation_with_custom_dir,
    benchmark_mock_filesystem_operations,
    benchmark_skip_installation,
    benchmark_large_mock_filesystem
);
criterion_main!(benches);
