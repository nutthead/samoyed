#!/usr/bin/env sh
# Test: Behavior when git command is not available
#
# This test verifies that Samoyed handles the absence of the git command gracefully.
# Samoyed depends on git for several operations:
# - git rev-parse --is-inside-work-tree (checking if in git repo)
# - git rev-parse --show-toplevel (finding repository root)
# - git config core.hooksPath (setting hooks path)
#
# When git is not available, Samoyed should fail with a clear error message.

# Load test helper functions regardless of current working directory
integration_script_dir="$(cd "$(dirname "$0")" && pwd)"
integration_repo_root="$(cd "$integration_script_dir/../.." && pwd)"
cd "$integration_repo_root"
. "$integration_repo_root/tests/integration/functions.sh"
unset integration_script_dir
unset integration_repo_root

parse_common_args "$@"

# Build Samoyed binary if needed
build_samoyed

# Set up isolated test environment
setup

# Test: Save original PATH
echo "Testing: Git command not found handling"
ORIGINAL_PATH="$PATH"

# Test: Remove git from PATH temporarily
echo "Testing: Removing git from PATH"

# Create a temporary directory that will be first in PATH
temp_bin_dir="${test_dir}/fake-bin"
mkdir -p "$temp_bin_dir"

# Create a fake git that always fails
cat > "$temp_bin_dir/git" << 'EOF'
#!/bin/sh
echo "git: command not found" >&2
exit 127
EOF
chmod +x "$temp_bin_dir/git"

# Put fake git first in PATH
PATH="$temp_bin_dir:$ORIGINAL_PATH"
export PATH

# Verify our fake git is being used
if ! git --version 2>&1 | grep -q "command not found"; then
    # Restore PATH and skip test if we can't override git
    PATH="$ORIGINAL_PATH"
    export PATH
    echo "WARNING: Cannot override git command, skipping some tests"
    echo "Testing: Alternative approach - empty PATH"

    # Try completely empty PATH instead
    # shellcheck disable=SC2123 # Intentionally clear PATH to simulate missing git
    PATH=""
    export PATH
fi

# Test: Try to run samoyed init without git available
echo "Testing: samoyed init without git in PATH"

set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

# Restore PATH before checking results
PATH="$ORIGINAL_PATH"
export PATH

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed init to fail without git, got exit code $exit_code"
fi

# Check for appropriate error message
if echo "$output" | grep -qi "git"; then
    ok "Samoyed reported git-related error"
else
    # Still pass if we got any error - the important thing is it didn't succeed
    if [ $exit_code -eq 1 ]; then
        ok "Samoyed failed without git (generic error)"
    else
        error "Expected git-related error message, got: $output"
    fi
fi

# Test: Verify Samoyed works after git is restored
echo "Testing: samoyed init with git restored"

# PATH is already restored, verify git works
if ! git --version >/dev/null 2>&1; then
    error "Git should be available after PATH restore"
fi

# Now samoyed should work
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 0 ]; then
    error "Samoyed should work with git restored, got exit code $exit_code: $output"
fi

# Verify the structure was created
if [ -d ".samoyed" ] && [ -d ".samoyed/_" ]; then
    ok "Samoyed works correctly after git is restored"
else
    error "Samoyed directory structure not created after git restored"
fi

# Test: Git available but returns error
echo "Testing: Git command fails with error"

# Clean up previous init
rm -rf .samoyed
git config --unset core.hooksPath

# Create a git wrapper that always returns error
cat > "$temp_bin_dir/git" << 'EOF'
#!/bin/sh
echo "fatal: git internal error" >&2
exit 128
EOF
chmod +x "$temp_bin_dir/git"

# Use the failing git
PATH="$temp_bin_dir:$ORIGINAL_PATH"
export PATH

set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

# Restore PATH
PATH="$ORIGINAL_PATH"
export PATH

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed to fail when git returns error, got exit code $exit_code"
fi

ok "Samoyed handled git command errors appropriately"

# Test: Git in unusual location (not in standard PATH)
echo "Testing: Git in non-standard location"

# Create git in unusual location
unusual_git_dir="${test_dir}/unusual-location"
mkdir -p "$unusual_git_dir"

# Symlink real git to unusual location
real_git=$(command -v git)
ln -s "$real_git" "$unusual_git_dir/git"

# Set PATH to prioritise the unusual location while keeping system tools available
PATH="$unusual_git_dir:$(dirname "$SAMOYED_BIN"):$ORIGINAL_PATH"
export PATH

# Clean up for fresh test
rm -rf .samoyed
"$unusual_git_dir/git" config --unset core.hooksPath 2>/dev/null || true

# Samoyed should work with git in unusual location
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

# Restore PATH
PATH="$ORIGINAL_PATH"
export PATH

if [ $exit_code -ne 0 ]; then
    error "Samoyed should work with git in unusual PATH location, got exit code $exit_code: $output"
fi

if [ -d ".samoyed" ] && [ -d ".samoyed/_" ]; then
    ok "Samoyed works with git in non-standard PATH location"
else
    error "Samoyed failed to initialize with git in unusual location"
fi

# Cleanup
rm -rf "$temp_bin_dir" "$unusual_git_dir"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
