#!/bin/bash
#
# Automated test execution script for Samoid
# This script runs all test suites and generates comprehensive reports
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🧪 Samoid Test Suite Runner"
echo "=========================="
echo

cd "$PROJECT_ROOT"

# Function to print status
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "success" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "error" ]; then
        echo -e "${RED}❌ $message${NC}"
    elif [ "$status" = "warning" ]; then
        echo -e "${YELLOW}⚠️ $message${NC}"
    else
        echo -e "$message"
    fi
}

# Function to run command with error handling
run_command() {
    local description=$1
    shift
    echo "Running: $description"
    if "$@"; then
        print_status success "$description completed"
        return 0
    else
        print_status error "$description failed"
        return 1
    fi
}

# Check prerequisites
echo "📋 Checking prerequisites..."
if ! command -v cargo >/dev/null 2>&1; then
    print_status error "Cargo not found. Please install Rust."
    exit 1
fi

if ! command -v git >/dev/null 2>&1; then
    print_status error "Git not found. Please install Git."
    exit 1
fi

print_status success "Prerequisites check passed"
echo

# Clean previous builds
echo "🧹 Cleaning previous builds..."
run_command "Clean build artifacts" cargo clean
echo

# Build the project
echo "🔨 Building project..."
run_command "Build project" cargo build
run_command "Build project (release)" cargo build --release
echo

# Run unit tests
echo "🔬 Running unit tests..."
run_command "Unit tests" cargo test --lib
echo

# Run integration tests
echo "🔗 Running integration tests..."
run_command "Integration tests" cargo test --test integration_test
run_command "Custom integration tests" cargo test --test integration_tests
echo

# Run all tests together
echo "🚀 Running all tests..."
run_command "All tests" cargo test
echo

# Run tests with verbose output (for debugging)
echo "📝 Running tests with verbose output..."
run_command "Verbose tests" cargo test -- --nocapture
echo

# Check code formatting
echo "📐 Checking code formatting..."
if cargo fmt -- --check >/dev/null 2>&1; then
    print_status success "Code formatting check passed"
else
    print_status warning "Code formatting issues found. Run 'cargo fmt' to fix."
fi
echo

# Run clippy lints
echo "📎 Running Clippy lints..."
if cargo clippy -- -D warnings >/dev/null 2>&1; then
    print_status success "Clippy lints passed"
else
    print_status warning "Clippy found issues. Run 'cargo clippy' for details."
fi
echo

# Generate documentation
echo "📚 Generating documentation..."
run_command "Generate docs" cargo doc --no-deps
echo

# Run benchmarks (if available)
echo "⚡ Running benchmarks..."
if cargo bench --help >/dev/null 2>&1; then
    run_command "Benchmarks" cargo bench
else
    print_status warning "Benchmarks not available (criterion dependency needed)"
fi
echo

# Generate coverage report
echo "📊 Generating coverage report..."
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    run_command "Coverage report" cargo tarpaulin --out Html --output-dir target/tarpaulin --timeout 120
    if [ -f "target/tarpaulin/tarpaulin-report.html" ]; then
        print_status success "Coverage report generated at target/tarpaulin/tarpaulin-report.html"
    fi
else
    print_status warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
fi
echo

# Cross-platform testing (if on Unix)
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🌐 Running cross-platform tests..."
    run_command "Cross-platform tests" cargo test --target x86_64-unknown-linux-gnu
    echo
fi

# Performance regression check
echo "📈 Performance regression check..."
if [ -f "target/criterion/benchmark/base/estimates.json" ]; then
    print_status success "Benchmark baseline exists"
else
    print_status warning "No benchmark baseline. Run benchmarks to establish baseline."
fi
echo

# Test summary
echo "📋 Test Summary"
echo "==============="
echo

# Count total test files
TEST_FILES=$(find tests/ -name "*.rs" | wc -l)
SRC_FILES=$(find src/ -name "*.rs" | wc -l)
echo "📁 Source files: $SRC_FILES"
echo "🧪 Test files: $TEST_FILES"

# Get test count (approximate)
TEST_COUNT=$(cargo test --dry-run 2>/dev/null | grep -c "test result:" || echo "Unknown")
echo "🔢 Estimated test count: $TEST_COUNT"

print_status success "All test suites completed successfully!"
echo
echo "🎉 Test execution finished. Check the output above for any issues."
echo "📊 Coverage report: target/tarpaulin/tarpaulin-report.html"
echo "📚 Documentation: target/doc/samoid/index.html"