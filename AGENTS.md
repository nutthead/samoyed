# Repository Guidelines

## Project Structure & Module Organization
Samoyed is a single-binary Rust CLI; core logic lives in `src/main.rs` with inline tests under `mod tests`. The Git hook wrapper resides in `assets/samoyed` and is embedded via `include_bytes!`. `clippy.toml` tracks lint thresholds, while `flake.nix` and `flake.lock` pin the development environment. `target/` is cargo output and must remain untracked.

## Build, Test, and Development Commands
Use `cargo build` for a debug build; add `--release` when benchmarking install performance. `cargo run -- init` exercises the CLI locally against a repo. `cargo test` runs unit tests that rely on `tempfile` helpers. `cargo fmt` enforces formatting, and `cargo clippy --all-targets --all-features` must stay warning-free. Inside a Nix shell (`nix develop`) you also get `cargo tarpaulin` for coverage.

## Coding Style & Naming Conventions
Follow Rust 2024 defaults (four-space indent, trailing commas). Favor descriptive `snake_case` for functions and variables, use `UpperCamelCase` for enums and structs, and keep CLI subcommands lowercase per `clap` conventions. Let `cargo fmt` apply formatting. Keep functions focused; clippy’s cognitive complexity cap is 21, so refactor early if warnings appear. Maintain module-level `//!` docs for high-level context.

## Testing Guidelines
Add unit tests inside the existing `#[cfg(test)]` module unless integration coverage demands a dedicated `tests/` tree. Name tests after the behavior under evaluation (`test_create_sample_pre_commit`). Use temporary directories and avoid modifying the workspace. Tests must run serially—`cargo test -- --test-threads=1` prevents the intermittent failures seen with the default parallel runner. Coverage runs inherit the same setting; `cargo tarpaulin` emits HTML (`tarpaulin-report.html`), JSON (`tarpaulin-report.json`), Cobertura XML (`cobertura.xml`), and LCOV (`lcov.info`) under `target/tarpaulin/`. Samoyed touches `.git/config` and writes hook files, so never run `samoyed init` or coverage experiments in this repo; instead, create a throwaway git repo under `tmp/` (`cd tmp && git init testbed && cd testbed`) before executing CLI checks. Keep `tmp/.gitkeep` in place so the sandbox directory stays tracked.

## Commit & Pull Request Guidelines
Commits follow Conventional Commits (`feat:`, `fix!:`, `chore:`) with optional `!` for breaking changes and concise, imperative descriptions. Group related edits; avoid catch-all commits. Pull requests should summarize motivation, list testing performed, and reference issues. Include CLI output or screenshots when behavior changes. Ensure formatting, clippy, and tests pass; call out skipped checks explicitly.
