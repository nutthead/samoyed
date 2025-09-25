# Repository Guidelines

## Project Structure & Module Organization
- Single-binary Rust CLI; core logic in `src/main.rs` with inline tests under `mod tests`.
- Git hook wrapper lives in `assets/samoyed` and is embedded via `include_bytes!`.
- Tooling/config: `clippy.toml` (lint thresholds), `flake.nix`/`flake.lock` (dev env).
- `target/` is Cargo output; keep it untracked.
- `tmp/` is untracked scratch space for local experiments. Examples:
  - `gh run view [<run-id>] --log  tmp/run_log_4.txt`.
  - `gh run download [<run-id>] --name tmp/security-reports`.

## Build, Test, and Development Commands
- Build: `cargo build` (debug) or `cargo build --release` (production).
- Run locally: `cargo run -- init` against a throwaway repo, never this workspace.
- Format: `cargo fmt`.
- Lint: `cargo clippy --all-targets --all-features` (must be warning-free).
- Tests: `cargo test -- --test-threads=1` (tests rely on `tempfile`).
- Nix shell: `nix develop`; coverage: `cargo tarpaulin -Html -Json -Xml -Lcov`.
- Example safe run: `TMP=$(mktemp -d); git init "$TMP"; (cd "$TMP" && cargo run -- init)`.

## Coding Style & Naming Conventions
- Rust 2024 defaults: four-space indent, trailing commas; let `cargo fmt` apply formatting.
- Naming: `snake_case` for functions/vars, `UpperCamelCase` for structs/enums; CLI subcommands lowercase (clap).
- Keep functions focused; refactor if clippy flags cognitive complexity (>21).

## Testing Guidelines
- Keep unit tests inside `#[cfg(test)]` in `src/main.rs`; name by behavior (e.g., `test_create_sample_pre_commit`).
- Use temporary directories; do not modify the workspace.
- Run tests serially: `cargo test -- --test-threads=1`.
- Coverage reports (when in `nix develop`) land under `target/tarpaulin/` as `tarpaulin-report.html`, `tarpaulin-report.json`, `cobertura.xml`, and `lcov.info`.
- Important: Samoyed touches `.git/config` and writes hook files---do not run `samoyed init`, tests, or coverage in this repo’s working copy.

## Commit & Pull Request Guidelines
- Conventional Commits (`feat:`, `fix!:`, `chore:`). Keep messages concise and imperative; group related edits.
- PRs: describe motivation, list testing performed, link issues, and include CLI output/screenshots for behavior changes.
- Ensure `cargo fmt`, clippy, and tests pass; call out any skipped checks explicitly.
