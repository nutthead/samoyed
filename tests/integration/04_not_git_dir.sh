#!/usr/bin/env sh
# Test: Behavior when not in a git repository
#
# This test verifies that Samoyed handles non-git directories gracefully.
# Unlike Husky which silently succeeds, Samoyed explicitly checks for a git
# repository and returns an error when not in one. This is by design to
# prevent accidental misconfiguration.
#
# Tests:
# 1. Running samoyed init outside a git repo should fail with clear error
# 2. Running samoyed init in a directory with .git file (not directory) should fail
# 3. Proper error messages are displayed

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

# Create a test directory that is NOT a git repository
test_name="$(basename "$0" .sh)"
test_root_dir=$(create_temp_dir "samoyed-not-git-${test_name}")
test_dir="${test_root_dir}/workspace"

echo
echo "========================================"
echo "TEST: $test_name"
echo "DIR:  $test_dir"
echo "========================================"
echo

# Create fresh test directory WITHOUT git init
mkdir -p "$test_dir"
cd "$test_dir"

# Test: Attempt to initialize Samoyed outside a git repository
echo "Testing: samoyed init outside git repository"

# Samoyed should fail with exit code 1 and appropriate error message
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed init to fail with exit code 1, got $exit_code"
fi

# Check for appropriate error message
if echo "$output" | grep -q "Not a git repository"; then
    ok "Samoyed correctly detected non-git directory"
else
    error "Expected 'Not a git repository' error message, got: $output"
fi

# Test: Create a .git file (not directory) and test
echo "Testing: samoyed init with .git file (not directory)"

# Create a .git file instead of directory (like git submodules or worktrees)
echo "gitdir: /some/other/location" >.git

# Samoyed should still fail since this isn't a valid git repository
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed init to fail with .git file, got exit code $exit_code"
fi

if echo "$output" | grep -q "Not a git repository"; then
    ok "Samoyed correctly rejected .git file"
else
    error "Expected 'Not a git repository' error with .git file, got: $output"
fi

# Test: Create an empty .git directory and test
echo "Testing: samoyed init with empty .git directory"

# Remove the .git file and create empty .git directory
rm -f .git
mkdir .git

# This is still not a valid git repository
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed init to fail with empty .git dir, got exit code $exit_code"
fi

if echo "$output" | grep -q "Not a git repository"; then
    ok "Samoyed correctly rejected empty .git directory"
else
    error "Expected 'Not a git repository' with empty .git dir, got: $output"
fi

# Test: Verify Samoyed works after creating valid git repo
echo "Testing: samoyed init after proper git initialization"

# Clean up and create proper git repository
rm -rf .git
git init --quiet
git config user.email "test@samoyed.test"
git config user.name "Samoyed Test"

# Now samoyed should work
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 0 ]; then
    error "Expected samoyed init to succeed in valid git repo, got exit code $exit_code: $output"
fi

# Verify the structure was created
if [ -d ".samoyed" ] && [ -d ".samoyed/_" ]; then
    ok "Samoyed successfully initialized after git init"
else
    error "Samoyed directory structure not created after git init"
fi

# Test: Nested directory in non-git parent
echo "Testing: Subdirectory of non-git directory"

# Create a new non-git directory with subdirectories
cd "$test_root_dir"
non_git_dir="${test_root_dir}/non-git-${test_name}-$$"
rm -rf "$non_git_dir"
mkdir -p "$non_git_dir/src/components"
cd "$non_git_dir/src/components"

# Try to init from deep subdirectory of non-git directory
set +e
output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?
set -e

if [ $exit_code -ne 1 ]; then
    error "Expected samoyed to fail in subdir of non-git dir, got exit code $exit_code"
fi

if echo "$output" | grep -q "Not a git repository"; then
    ok "Samoyed correctly failed in subdirectory of non-git directory"
else
    error "Expected 'Not a git repository' in subdir, got: $output"
fi

# Cleanup
cd "$test_root_dir"
rm -rf "$non_git_dir"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
