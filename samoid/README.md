# Samoid

[![Test Suite](https://github.com/nutthead/samoid/actions/workflows/test.yml/badge.svg)](https://github.com/nutthead/samoid/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/nutthead/samoid/graph/badge.svg?token=YOUR_CODECOV_TOKEN)](https://codecov.io/gh/nutthead/samoid)
[![Security Audit](https://img.shields.io/badge/security-audit_passing-green)](https://github.com/nutthead/samoid/actions/workflows/test.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-blue)](https://www.rust-lang.org)

A modern, fast, and secure Git hooks manager written in Rust. Samoid is a reimplementation of Husky with improved performance, better error handling, and enhanced security features.

## Features

- 🚀 **Fast**: Built with Rust for optimal performance
- 🔒 **Secure**: Comprehensive path validation and security checks
- 🛡️ **Robust**: Detailed error handling with actionable suggestions
- 🧪 **Well-tested**: 100% test coverage with comprehensive integration tests
- 🌍 **Cross-platform**: Supports Linux, macOS, and Windows
- 📦 **Zero dependencies**: No runtime dependencies beyond Git

## Installation

```bash
cargo install samoid
```

## Quick Start

Initialize Git hooks in your repository:

```bash
samoid init
```

This will:
1. Configure Git to use `.samoid/_` as the hooks directory
2. Create the hooks directory structure
3. Install hook files that delegate to the `samoid-hook` runner

## Usage

### Basic Commands

```bash
# Initialize hooks (one-time setup)
samoid init

# Install hooks with custom directory
samoid init --hooks-dir custom-hooks
```

### Environment Variables

- `SAMOID=0` - Skip hook installation entirely
- `SAMOID_DEBUG=1` - Enable debug logging

## Architecture

Samoid uses a dual-binary architecture:

- **`samoid`**: CLI interface for initialization and management
- **`samoid-hook`**: Lightweight hook runner executed by Git

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

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Husky](https://typicode.github.io/husky/) by typicode
- Built with ❤️ in Rust