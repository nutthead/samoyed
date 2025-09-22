#!/usr/bin/env sh
# Integration test helper functions for Samoyed
# Adapted from Husky's test suite for testing Git hooks managers
# All tests run in the ./tmp directory to avoid modifying the Samoyed repository itself

# Exit on error and undefined variables
set -eu

# Get the absolute path to the Samoyed binary
# We build it in release mode for testing real-world performance
SAMOYED_BIN="${SAMOYED_BIN:-$(pwd)/target/release/samoyed}"

# Base directory for all test operations - MUST be in ./tmp
TEST_BASE_DIR="$(pwd)/tmp"

# Setup function - creates a clean test environment
# Creates a new git repository in ./tmp for each test
setup() {
    # Get the test name from the script filename
    test_name="$(basename -- "$0" .sh)"

    # Create test directory inside ./tmp
    test_dir="${TEST_BASE_DIR}/samoyed-test-${test_name}-$$"

    # Print test header for clarity
    echo
    echo "========================================"
    echo "TEST: $test_name"
    echo "DIR:  $test_dir"
    echo "========================================"
    echo

    # Ensure we're working in ./tmp directory
    if [ ! -d "$TEST_BASE_DIR" ]; then
        echo "ERROR: ./tmp directory does not exist"
        exit 1
    fi

    # Clean up any previous test directory with same name
    rm -rf "$test_dir"

    # Create fresh test directory
    mkdir -p "$test_dir"
    cd "$test_dir"

    # Initialize a new git repository for testing
    git init --quiet

    # Configure git user (required for commits)
    git config user.email "test@samoyed.test"
    git config user.name "Samoyed Test"

    # Create a dummy file for commits
    echo "test content" > test.txt
    git add test.txt
    git commit -m "Initial commit" --quiet
}

# Cleanup function - removes test directory after test completion
cleanup() {
    if [ -n "${test_dir:-}" ] && [ -d "${test_dir:-}" ]; then
        cd "$TEST_BASE_DIR"
        rm -rf "$test_dir"
        echo "Cleaned up: $test_dir"
    fi
}

# Initialize Samoyed in the test repository
# Usage: init_samoyed [dirname]
init_samoyed() {
    # POSIX compliant - no 'local' keyword
    dirname="${1:-.samoyed}"
    "$SAMOYED_BIN" init "$dirname"
}

# Run a command and check its exit code
# Usage: expect EXIT_CODE COMMAND
# Example: expect 0 "git commit -m 'test'"
expect() {
    # POSIX compliant - no 'local' keyword
    expected_exit_code="$1"
    command="$2"

    # Disable exit on error for this command
    set +e

    # Execute the command using eval to handle complex commands
    eval "$command"
    actual_exit_code="$?"

    # Re-enable exit on error
    set -e

    # Check if exit code matches expectation
    if [ "$actual_exit_code" != "$expected_exit_code" ]; then
        error "Expected command '$command' to exit with code $expected_exit_code (got $actual_exit_code)"
    fi

    return 0
}

# Check that git core.hooksPath is set to expected value
# Usage: expect_hooks_path_to_be PATH
expect_hooks_path_to_be() {
    # POSIX compliant - no 'local' keyword
    expected_path="$1"

    # Get current hooks path, handling case where it's not set
    set +e
    actual_path=$(git config core.hooksPath)
    set -e

    # Handle empty/unset case
    if [ -z "$actual_path" ]; then
        actual_path=""
    fi

    # Git may store the path as absolute or relative
    # Check if paths end with the same suffix (handles both cases)
    case "$actual_path" in
        *"$expected_path")
            # Path ends with expected path - this is ok
            return 0
            ;;
        "$expected_path")
            # Exact match
            return 0
            ;;
        *)
            # Also check if they resolve to the same directory
            if [ -d "$expected_path" ] && [ -d "$actual_path" ]; then
                # Compare canonical paths
                expected_canonical=$(cd "$expected_path" 2>/dev/null && pwd)
                actual_canonical=$(cd "$actual_path" 2>/dev/null && pwd)
                if [ "$expected_canonical" = "$actual_canonical" ]; then
                    return 0
                fi
            fi
            error "Expected core.hooksPath to be '$expected_path', but was '$actual_path'"
            ;;
    esac
}

# Report an error and exit with failure
# Usage: error "Error message"
error() {
    echo
    echo "❌ ERROR: $1"
    echo
    cleanup
    exit 1
}

# Report success
# Usage: ok "Success message"
ok() {
    echo "✓ ${1:-OK}"
}

# Create a hook file with given content
# Usage: create_hook HOOK_NAME CONTENT
# Example: create_hook "pre-commit" "echo 'pre-commit' && exit 1"
create_hook() {
    # POSIX compliant - no 'local' keyword
    hook_name="$1"
    content="$2"
    hook_dir="${3:-.samoyed}"
    hook_path="$hook_dir/$hook_name"

    # Ensure the hook directory exists
    if [ ! -d "$hook_dir" ]; then
        error "Hook directory '$hook_dir' does not exist"
    fi

    # Create the hook file with proper shebang
    {
        echo "#!/usr/bin/env sh"
        echo ". \"\$(dirname \"\$0\")/_/samoyed\""
        echo ""
        echo "$content"
    } > "$hook_path"

    # Make it executable (required for some tests)
    chmod +x "$hook_path"
}

# Check if a file exists
# Usage: expect_file_exists PATH
expect_file_exists() {
    # POSIX compliant - no 'local' keyword
    file_path="$1"

    if [ ! -f "$file_path" ]; then
        error "Expected file '$file_path' to exist, but it doesn't"
    fi
}

# Check if a directory exists
# Usage: expect_dir_exists PATH
expect_dir_exists() {
    # POSIX compliant - no 'local' keyword
    dir_path="$1"

    if [ ! -d "$dir_path" ]; then
        error "Expected directory '$dir_path' to exist, but it doesn't"
    fi
}

# Build Samoyed binary if not already built
# This ensures we're testing the current code
build_samoyed() {
    if [ ! -f "$SAMOYED_BIN" ]; then
        echo "Building Samoyed binary..."
        cargo build --release --quiet

        if [ ! -f "$SAMOYED_BIN" ]; then
            error "Failed to build Samoyed binary at $SAMOYED_BIN"
        fi

        ok "Samoyed binary built successfully"
    fi
}

# Verify we're not in the Samoyed repository root
# This prevents tests from accidentally modifying the Samoyed repository
verify_test_safety() {
    # POSIX compliant - no 'local' keyword
    current_dir="$(pwd)"

    # Check if we're in the Samoyed repository root
    if [ -f "$current_dir/Cargo.toml" ] && grep -q "name = \"samoyed\"" "$current_dir/Cargo.toml" 2>/dev/null; then
        if [ "$current_dir" = "$(git rev-parse --show-toplevel 2>/dev/null || echo "$current_dir")" ]; then
            error "Tests must not run in the Samoyed repository root! Run from ./tmp directory."
        fi
    fi
}

# Set up signal handlers for cleanup
trap cleanup EXIT INT TERM