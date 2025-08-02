# Samoyed

[![Test Suite](https://github.com/nutthead/samoyed/actions/workflows/test.yml/badge.svg)](https://github.com/nutthead/samoyed/actions/workflows/test.yml)&nbsp;&nbsp;[![codecov](https://codecov.io/gh/nutthead/samoyed/graph/badge.svg?token=8ROK706MYS)](https://codecov.io/gh/nutthead/samoyed)&nbsp;&nbsp;[![Security Audit](https://img.shields.io/badge/security-audit_passing-green)](https://github.com/nutthead/samoyed/actions/workflows/test.yml)&nbsp;&nbsp;[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-blue)](https://www.rust-lang.org)

A modern, fast, and secure Git hooks manager written in Rust. Samoyed is inspired by Husky with improved performance, better error handling, and enhanced security features.

You don’t have to fuss with that pesky `package.json` file in your projects anymore! 🤌

![Samoyed](docs/images/samoyed.webp)

## Test Coverage

![Grid](https://codecov.io/gh/nutthead/samoyed/graphs/tree.svg?token=8ROK706MYS)

## Features

- 🚀 **Fast**: Built with Rust for optimal performance
- 🔒 **Secure**: Comprehensive path validation and security checks
- 🛡️ **Robust**: Detailed error handling with actionable suggestions
- 🧪 **Well-tested**: Comprehensive test coverage with extensive integration tests
- 🌍 **Cross-platform**: Supports Linux, macOS, and Windows
- 📦 **Minimal dependencies**: Small set of essential Rust dependencies

## Installation

```bash
cargo install samoyed
```

## Quick Start

Initialize Git hooks in your repository:

```bash
samoyed init
```

This will:
1. Configure Git to use `.samoyed/_` as the hooks directory
2. Create the hooks directory structure
3. Install hook files that delegate to the `samoyed-hook` runner

## Usage

### Basic Commands

```bash
# Initialize hooks (one-time setup)
samoyed init

# Install hooks with custom directory
samoyed init --hooks-dir custom-hooks
```

### Environment Variables

- `SAMOYED=0` - Skip hook installation entirely
- `SAMOYED_DEBUG=1` - Enable debug logging

## Architecture

Samoyed uses a dual-binary architecture:

- **`samoyed`**: CLI interface for initialization and management
- **`samoyed-hook`**: Lightweight hook runner executed by Git

This separation ensures minimal overhead during Git operations while providing rich functionality for setup and management.

## Development

### Prerequisites

- Rust 1.85+ (Rust 2024 edition)
- Git

### Building

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Testing

The project uses comprehensive testing with dependency injection:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test installation_tests
cargo test --test validation_tests
cargo test --test error_handling_tests

# Run platform-specific tests
cargo test --test linux_tests    # Linux only
cargo test --test macos_tests    # macOS only
cargo test --test windows_tests  # Windows only
```

### Code Coverage

Generate coverage reports:

```bash
cargo tarpaulin --out html --output-dir target/coverage
```

## Contributing

1. Let's [discuss](https://github.com/nutthead/samoyed/discussions)
2. Fork the repository
3. Create a feature branch
4. Make your changes
5. Add tests for new functionality
6. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Husky](https://typicode.github.io/husky/)
- Built with 🤟 🫡 in Rust
