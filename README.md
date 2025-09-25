# Samoyed

> A single-binary, minimal, ultra-fast Git hooks manager for every platform.

Samoyed keeps Git hook management small, transparent, and safe. It ships as one Rust binary plus a POSIX wrapper script, so developers can install it quickly, version it with their repositories, and stay in control of what runs on commit.

![Samoyed](.assets/samoyed.jpeg)

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Development](#development)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Background

Samoyed was built to strip Git hook tooling down to the essentials:

- **One file of Rust code (`src/main.rs`)** manages CLI parsing, repository safety checks, and file generation.
- **One POSIX shell wrapper (`assets/samoyed`)** bootstraps every Git hook and keeps behaviour consistent across macOS, Linux, and Windows (via WSL or compatible shells).
- **Zero runtime dependencies.** The compiled binary embeds the wrapper with `include_bytes!`, so distributing Samoyed is as simple as copying the executable.

Version 0.2.0 doubles down on clarity: the `samoyed init` command seeds every Git hook, wires them through the shared wrapper, and leaves a template pre-commit script ready for teams to adapt. Environment variables such as `SAMOYED=0` (bypass) and `SAMOYED=2` (debug) give developers predictable escape hatches without extra plugins.

This represents a fundamental architectural simplification from version 0.1.17, which scattered functionality across 23 separate Rust modules totaling nearly 6,000 lines of code. The current single-file implementation achieves the same functionality in just about 1000 lines---an ~80% reduction in code size. By consolidating everything into `src/main.rs`, the codebase becomes dramatically easier to understand, debug, and maintain, while eliminating the cognitive overhead of navigating complex module hierarchies and cross-file dependencies.

## Install

Samoyed is a Cargo binary. Install straight from source:

```sh
# Clone the repository
git clone https://github.com/nutthead/samoyed.git
cd samoyed

# Build and place the binary on your PATH
cargo install --path .
```

## Usage

```sh
# inside a Git repository
samoyed init [samoyed-dirname]
```

- `samoyed-dirname` defaults to `.samoyed`. The directory must resolve inside the repository; otherwise the command aborts with a clear error.
- When the command succeeds, Samoyed creates `samoyed-dirname`, seeds a `_` subdirectory with all 14 client-side Git hooks, installs the shared wrapper at `samoyed-dirname/_/samoyed`, writes a `.gitignore`, and sets `core.hooksPath` to point at the new `_` directory.
- A starter `samoyed-dirname/pre-commit` script is created with comments explaining where to add project-specific checks.

### Bypass and debug modes

- `SAMOYED=0 samoyed init …` prints a bypass message and leaves the repository untouched.
- `SAMOYED=0 git commit …` skips every hook sourced by the wrapper.
- `SAMOYED=2 git commit …` enables `set -x` inside the wrapper for quick troubleshooting.

## Configuration

Samoyed looks for an optional user init script at `${XDG_CONFIG_HOME:-$HOME/.config}/samoyed/init.sh`. If present, it is sourced before any hook runs---use it to export shared environment variables or early exits. Because hooks are standard shell scripts, teams can customise individual hooks directly in `.samoyed/<hook>`.

Common workflows:

- Run `cargo check` followed by `cargo fmt` in `.samoyed/pre-commit`, then re-stage formatted files with `git add --update '*.rs'`.
- Place project-wide environment set-up (for example, secrets loading) in the optional init script, so every hook inherits the same session state.

## Development

Clone the repository and work inside a Nix shell (`nix develop`) or your local Rust toolchain.

Core commands:

```sh
cargo check           # Fast type checks
cargo fmt             # Format Rust code
cargo build --release # Build the optimised binary used in production
cargo test            # Run unit tests
```

Integration tests live under `tests/integration`. Each script provisions a disposable repository via `mktemp`; run them individually, optionally preserving the workspace for inspection:

```sh
cd tests/integration
./01_default.sh            # Run a single scenario
./08_samoyed_0.sh --keep   # Keep the temporary repo after the test exits
```

## Maintainers

- [Behrang Saeedzadeh](https://github.com/behrangsa)

## Contributing

Issues and pull requests are welcome. Before submitting a change, please ensure:

1. `cargo fmt`, `cargo check`, and `cargo test` pass locally.
2. Relevant integration scripts succeed.
3. Commit messages follow Conventional Commit style (`feat:`, `fix:`, `chore:`, …).

## License

[MIT](LICENSE).
