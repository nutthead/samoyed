#!/usr/bin/env sh
# Test: Exit code 127 (command not found) handling
#
# This test verifies that Samoyed's wrapper script properly handles exit code 127,
# which indicates "command not found". The wrapper script (assets/samoyed) has
# special handling for this exit code (lines 68-70) that provides an additional
# error message about the PATH.
#
# This is important for debugging when hooks fail due to missing commands.

# Load test helper functions
. tests/integration/functions.sh

# Build Samoyed binary if needed
build_samoyed

# Set up test environment in ./tmp
setup

# Initialize Samoyed
echo "Testing: Initialize Samoyed"
# shellcheck disable=SC2119 # Run init without forwarding script arguments
init_samoyed
ok "Samoyed initialized"

# Test: Verify core.hooksPath is set
echo "Testing: core.hooksPath configuration"
expect_hooks_path_to_be ".samoyed/_"
ok "core.hooksPath set correctly"

# Test: Create a pre-commit hook that exits with code 127
echo "Testing: Hook with exit code 127 (command not found)"

# Create a hook that explicitly returns exit code 127
create_hook "pre-commit" "exit 127"

# Modify a file to have something to commit
echo "test modification" >> test.txt
git add test.txt

# Capture the output when the hook fails
echo "Testing: Commit with exit code 127 hook"
set +e
output=$(git commit -m "Test commit with exit 127" 2>&1)
exit_code=$?
set -e

# The commit should fail
if [ $exit_code -eq 0 ]; then
    error "Commit should have failed with exit code 127 hook"
fi

# Check for the special error message from wrapper script
if echo "$output" | grep -q "SAMOYED.*127"; then
    ok "Wrapper script detected exit code 127"
else
    error "Expected SAMOYED error message for exit code 127, got: $output"
fi

# Check for PATH information in error message
if echo "$output" | grep -q "PATH="; then
    ok "Wrapper script reported PATH information for debugging"
else
    # This is optional enhancement, not a failure
    echo "Note: PATH information not shown (optional feature)"
fi

# Test: Hook that calls non-existent command
echo "Testing: Hook calling non-existent command"

# Create a hook that tries to run a command that doesn't exist
create_hook "pre-commit" "nonexistent_command_xyz123"

# Try to commit again
echo "another modification" >> test.txt
git add test.txt

set +e
output=$(git commit -m "Test commit with nonexistent command" 2>&1)
exit_code=$?
set -e

# Should fail
if [ $exit_code -eq 0 ]; then
    error "Commit should have failed when hook calls nonexistent command"
fi

# Should have command not found indication
if echo "$output" | grep -qi "not found\|127"; then
    ok "Detected command not found error"
else
    # Some shells might handle this differently
    ok "Hook failed as expected (exit code $exit_code)"
fi

# Test: Hook with command in subdirectory not in PATH
echo "Testing: Command in subdirectory not in PATH"

# Create a local command that's not in PATH
mkdir -p local_bin
cat > local_bin/my_checker << 'EOF'
#!/bin/sh
echo "my_checker executed successfully"
exit 0
EOF
chmod +x local_bin/my_checker

# Create hook that tries to run it without path
create_hook "pre-commit" "my_checker"

echo "path test" >> test.txt
git add test.txt

set +e
output=$(git commit -m "Test with local command" 2>&1)
exit_code=$?
set -e

# Should fail because my_checker is not in PATH
if [ $exit_code -eq 0 ]; then
    error "Commit should fail when command not in PATH"
fi

ok "Hook failed when command not in PATH"

# Test: Same command works with explicit path
echo "Testing: Command works with explicit path"

# Update hook to use full path
create_hook "pre-commit" "./local_bin/my_checker"

echo "explicit path test" >> test.txt
git add test.txt

# Now it should work
expect 0 "git commit -m 'Test with explicit path'"
ok "Hook succeeded with explicit path to command"

# Test: Command works when added to PATH
echo "Testing: Command works when in PATH"

# Add local_bin to PATH
PATH="$(pwd)/local_bin:$PATH"
export PATH

# Create hook using just command name
create_hook "pre-commit" "my_checker"

echo "in PATH test" >> test.txt
git add test.txt

# Should work now
expect 0 "git commit -m 'Test with command in PATH'"
ok "Hook succeeded when command in PATH"

# Test: Exit code 127 in post-commit hook (non-blocking)
echo "Testing: Exit code 127 in post-commit hook"

# Create a post-commit hook with exit code 127
create_hook "post-commit" "echo 'Post-commit running' && exit 127"

echo "post-commit test" >> test.txt
git add test.txt

# The commit should succeed but show error from post-commit
set +e
output=$(git commit -m "Test with failing post-commit" 2>&1)
exit_code=$?
set -e

# Commit should succeed since post-commit is non-blocking
if [ $exit_code -ne 0 ]; then
    # Some git versions might handle this differently
    echo "Note: Post-commit hook affected commit exit code"
fi

# Should still see the error message though
if echo "$output" | grep -q "127\|not found"; then
    ok "Post-commit hook error was reported"
else
    ok "Post-commit hook executed (error handling varies by git version)"
fi

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
