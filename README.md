# Samoyed

[![Crates.io Version](https://img.shields.io/crates/v/samoyed)](https://crates.io/crates/samoyed)

> A single-binary, minimalist, ultra-fast Git hooks manager for every platform.

Samoyed keeps Git hook management small, transparent, and safe. It ships as one Rust binary plus a POSIX wrapper script, so developers can install it quickly, version it with their repositories, and stay in control of what runs on commit.

![Samoyed](.assets/samoyed.jpeg)

## Why Samoyed?

- **Single binary** — Zero runtime dependencies. One Rust executable embeds everything.
- **Transparent** — All code in one file (`src/main.rs`). No hidden complexity.
- **Cross-platform** — Works on Linux, macOS, and Windows (WSL). POSIX wrapper ensures consistency.
- **Developer-friendly** — `SAMOYED=0` to bypass, `SAMOYED=2` to debug. Simple escape hatches when you need them.
- **80% smaller** — 0.2.x radically simplifies the code from 6000+ lines across 23 modules to ~1000 lines of code in one file.

## Quick Start

Get started in under a minute:

```sh
# Install Samoyed
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | bash

# Navigate to your Git repository
cd your-project

# Initialize hooks
samoyed init

# Edit the starter pre-commit hook
$EDITOR .samoyed/pre-commit

# Test it
git commit --allow-empty -m "test hooks"
```

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Background](#background)
- [Development](#development)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Install

### Quick Install (Linux and macOS)

The fastest way to install Samoyed is with a single command:

```sh
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | bash
```

This downloads the latest pre-built binary for your platform, verifies its checksum, and places it in `~/.local/bin`. To customize the installation directory:

```sh
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh | INSTALL_DIR=~/bin bash
```

If you prefer to inspect the installer before running it:

```sh
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/install.sh -o install.sh
less install.sh
bash install.sh
```

### Using Cargo

If you have Rust installed, use `cargo install`:

```sh
cargo install samoyed
```

### Building from Source

Clone the repository and build locally:

```sh
git clone https://github.com/nutthead/samoyed.git
cd samoyed
cargo install --path .
```

### Windows

Windows users should install via [WSL (Windows Subsystem for Linux)](https://learn.microsoft.com/en-us/windows/wsl/install) and follow the Linux installation instructions above, or use `cargo install samoyed` in a native Windows terminal with the Rust toolchain installed.

### Uninstalling

To remove Samoyed:

```sh
curl -fsSL https://raw.githubusercontent.com/nutthead/samoyed/main/uninstall.sh | bash
```

Or manually:

```sh
rm ~/.local/bin/samoyed
# Remove PATH entry from your shell config if added by the installer
```

## Usage

### Initialize Hooks

Inside your Git repository, run:

```sh
samoyed init [samoyed-dirname]
```

The `samoyed-dirname` defaults to `.samoyed` and must reside within the repository. On success, Samoyed creates:

```
.samoyed/
├── _/                    # Hook wrappers (git-ignored)
│   ├── pre-commit
│   ├── commit-msg
│   ├── pre-push
│   ├── ... (all 14 hooks)
│   └── samoyed          # POSIX wrapper script
├── pre-commit           # Your editable hook (starter template)
└── .gitignore           # Ignores the _/ directory
```

Git's `core.hooksPath` is configured to point to `.samoyed/_/`, routing all hook events through the wrapper.

### Creating Your First Hook

The starter `pre-commit` script includes helpful comments. Edit it to add project-specific checks:

```sh
#!/bin/sh
# .samoyed/pre-commit

# Example: Format Rust code before commit
if ! cargo fmt --check; then
  echo "Running cargo fmt..."
  cargo fmt
  git add --update '*.rs'
fi

# Example: Run linter
cargo clippy -- -D warnings

# Example: Ensure tests pass
cargo test --all
```

Make it executable and commit it to version control:

```sh
chmod +x .samoyed/pre-commit
git add .samoyed/
git commit -m "Add pre-commit hook for Rust formatting and tests"
```

Now every commit will automatically run these checks.

### Bypass and Debug Modes

**Bypass all hooks** when you need to commit without running checks:

```sh
SAMOYED=0 git commit -m "emergency fix"
```

**Enable debug mode** to see exactly what the wrapper is doing:

```sh
SAMOYED=2 git commit -m "debug this hook"
```

This runs the wrapper with `set -x`, printing each command as it executes.

**Bypass during initialization** to prepare hooks without activating them:

```sh
SAMOYED=0 samoyed init
```

## Configuration

### User Init Script

Samoyed sources an optional initialization script at `${XDG_CONFIG_HOME:-$HOME/.config}/samoyed/init.sh` before running any hook. Use this for environment setup shared across all hooks:

```sh
# ~/.config/samoyed/init.sh

# Load secrets
export DATABASE_URL="$(cat ~/.secrets/db_url)"

# Set project defaults
export RUST_BACKTRACE=1

# Skip hooks in CI (optional)
if [ -n "$CI" ]; then
  exit 0
fi
```

### Per-Hook Customization

Because hooks are standard shell scripts, customize them directly in `.samoyed/<hook>`:

```sh
# .samoyed/pre-push - runs before git push

# Ensure main branch is never pushed directly
branch=$(git symbolic-ref HEAD | sed -e 's,.*/\(.*\),\1,')
if [ "$branch" = "main" ]; then
  echo "Direct pushes to main are forbidden"
  exit 1
fi

# Run full test suite before pushing
cargo test --all --release
```

## Background

Samoyed was built to strip Git hook tooling down to the essentials:

- **One file of Rust code (`src/main.rs`)** manages CLI parsing, repository safety checks, and file generation.
- **One POSIX shell wrapper (`assets/samoyed`)** bootstraps every Git hook and keeps behaviour consistent across macOS, Linux, and Windows (via WSL or compatible shells).
- **Zero runtime dependencies.** The compiled binary embeds the wrapper---`assets/samoyed`---with `include_bytes!`, so distributing Samoyed is as simple as copying the executable.

In 0.2.x, Samoyed doubles down on clarity: the `samoyed init` command seeds every Git hook, wires them through the shared wrapper, and leaves a template pre-commit script ready for teams to adapt. Environment variables such as `SAMOYED=0` (bypass) and `SAMOYED=2` (debug) give developers predictable escape hatches without extra plugins.

This represents a fundamental architectural simplification from version 0.1.17, which scattered functionality across 23 separate Rust modules totaling nearly 6,000 lines of code. The current single-file implementation achieves the same functionality<sup>*</sup> in just about 1000 lines of code---an ~80% reduction in code size. By consolidating everything into `src/main.rs`, the codebase becomes dramatically easier to understand, debug, and maintain, while eliminating the cognitive overhead of navigating complex module hierarchies and cross-file dependencies.

<sup>*</sup>Support for `samoyed.toml` is removed in version 0.2.0. However I will re-introduce a well-thought-out option for configuring hooks _"declaratively"_ in a future release.

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

- [Behrang Saeedzadeh](https://www.behrang.org)

## Contributing

Issues and pull requests are welcome. Before submitting a change, please ensure:

1. `cargo fmt`, `cargo check`, and `cargo test` pass locally.
2. Relevant integration scripts succeed.
3. Commit messages follow Conventional Commit style (`feat:`, `fix:`, `chore:`, …).

## License

[MIT](LICENSE)
