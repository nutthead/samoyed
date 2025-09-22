#!/usr/bin/env sh
# Integration test helper functions for Samoyed
# Adapted from Husky's test suite for testing Git hooks managers
# All tests run in the ./tmp directory to avoid modifying the Samoyed repository itself

# Exit on error and undefined variables
set -eu

# Helper functions run in POSIX shells without "local".  We prefix
# helper-scoped variables with the function name and explicitly unset
# anything that should not leak into the global namespace.  The
# `test_dir` variable is intentionally global so that setup/cleanup and
# individual tests can share the active repository path.

# Get the absolute path to the Samoyed binary
# We build it in release mode for testing real-world performance
SAMOYED_BIN="${SAMOYED_BIN:-$(pwd)/target/release/samoyed}"

# Base directory for all test operations - MUST be in ./tmp
TEST_BASE_DIR="$(pwd)/tmp"

# Setup function - creates a clean test environment
# Creates a new git repository in ./tmp for each test
setup() {
    # Get the test name from the script filename
    setup_test_name="$(basename "$0" .sh)"

    # Create test directory inside ./tmp
    test_dir="${TEST_BASE_DIR}/samoyed-test-${setup_test_name}-$$"

    # Print test header for clarity
    echo
    echo "========================================"
    echo "TEST: $setup_test_name"
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

    unset setup_test_name
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
    "$SAMOYED_BIN" init "$@"
}

# Run a command and check its exit code
# Usage: expect EXIT_CODE COMMAND
# Example: expect 0 "git commit -m 'test'"
expect() {
    expect_expected_code="$1"
    expect_command="$2"

    # Disable exit on error for this command
    set +e

    # Execute the command using eval to handle complex commands
    eval "$expect_command"
    expect_actual_code="$?"

    # Re-enable exit on error
    set -e

    # Check if exit code matches expectation
    if [ "$expect_actual_code" != "$expect_expected_code" ]; then
        error "Expected command '$expect_command' to exit with code $expect_expected_code (got $expect_actual_code)"
    fi

    unset expect_expected_code
    unset expect_command
    unset expect_actual_code
    return 0
}

# Check that git core.hooksPath is set to expected value
# Usage: expect_hooks_path_to_be PATH
expect_hooks_path_to_be() {
    expect_hooks_expected_path="$1"

    # Get current hooks path, handling case where it's not set
    set +e
    expect_hooks_actual_path=$(git config core.hooksPath)
    set -e

    # Handle empty/unset case
    if [ -z "$expect_hooks_actual_path" ]; then
        expect_hooks_actual_path=""
    fi

    # Git may store the path as absolute or relative
    # Check if paths end with the same suffix (handles both cases)
    case "$expect_hooks_actual_path" in
        *"$expect_hooks_expected_path")
            # Path ends with expected path - this is ok
            unset expect_hooks_expected_path
            unset expect_hooks_actual_path
            return 0
            ;;
        "$expect_hooks_expected_path")
            # Exact match
            unset expect_hooks_expected_path
            unset expect_hooks_actual_path
            return 0
            ;;
        *)
            # Also check if they resolve to the same directory
            if [ -d "$expect_hooks_expected_path" ] && [ -d "$expect_hooks_actual_path" ]; then
                # Compare canonical paths
                expect_hooks_expected_canonical=$(cd "$expect_hooks_expected_path" 2>/dev/null && pwd)
                expect_hooks_actual_canonical=$(cd "$expect_hooks_actual_path" 2>/dev/null && pwd)
                if [ "$expect_hooks_expected_canonical" = "$expect_hooks_actual_canonical" ]; then
                    unset expect_hooks_expected_canonical
                    unset expect_hooks_actual_canonical
                    unset expect_hooks_expected_path
                    unset expect_hooks_actual_path
                    return 0
                fi
                unset expect_hooks_expected_canonical
                unset expect_hooks_actual_canonical
            fi
            error "Expected core.hooksPath to be '$expect_hooks_expected_path', but was '$expect_hooks_actual_path'"
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
    create_hook_name="$1"
    create_hook_content="$2"
    create_hook_dir="${3:-.samoyed}"
    create_hook_path="$create_hook_dir/$create_hook_name"

    # Ensure the hook directory exists
    if [ ! -d "$create_hook_dir" ]; then
        error "Hook directory '$create_hook_dir' does not exist"
    fi

    # Create the hook file with proper shebang
    {
        echo "#!/usr/bin/env sh"
        echo ". \"\$(dirname \"\$0\")/_/samoyed\""
        echo ""
        echo "$create_hook_content"
    } > "$create_hook_path"

    # Make it executable (required for some tests)
    chmod +x "$create_hook_path"

    unset create_hook_name
    unset create_hook_content
    unset create_hook_dir
    unset create_hook_path
}

# Check if a file exists
# Usage: expect_file_exists PATH
expect_file_exists() {
    expect_file_path="$1"

    if [ ! -f "$expect_file_path" ]; then
        error "Expected file '$expect_file_path' to exist, but it doesn't"
    fi

    unset expect_file_path
}

# Check if a directory exists
# Usage: expect_dir_exists PATH
expect_dir_exists() {
    expect_dir_path="$1"

    if [ ! -d "$expect_dir_path" ]; then
        error "Expected directory '$expect_dir_path' to exist, but it doesn't"
    fi

    unset expect_dir_path
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
    verify_current_dir="$(pwd)"

    # Check if we're in the Samoyed repository root
    if [ -f "$verify_current_dir/Cargo.toml" ] && grep -q "name = \"samoyed\"" "$verify_current_dir/Cargo.toml" 2>/dev/null; then
        verify_repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "$verify_current_dir")"
        if [ "$verify_current_dir" = "$verify_repo_root" ]; then
            error "Tests must not run in the Samoyed repository root! Run from ./tmp directory."
        fi
    fi

    unset verify_repo_root
    unset verify_current_dir
}

# Set up signal handlers for cleanup
trap cleanup EXIT INT TERM
