#!/usr/bin/env sh
# Test: Basic Samoyed initialization and hook execution
#
# This test verifies the core functionality of Samoyed:
# 1. Initializing Samoyed with default settings
# 2. Verifying the directory structure is created correctly
# 3. Checking that core.hooksPath is set properly
# 4. Creating a pre-commit hook that fails
# 5. Verifying that the failing hook blocks commits
#
# This is the most fundamental test - if this fails, nothing else will work.

# Load test helper functions
. tests/integration/functions.sh

# Build Samoyed binary if needed
build_samoyed

# Set up test environment in ./tmp
setup

# Test: Initialize Samoyed with default directory
echo "Testing: samoyed init (default directory)"
init_samoyed
ok "Samoyed initialized successfully"

# Test: Verify directory structure was created
echo "Testing: Directory structure"
expect_dir_exists ".samoyed"
expect_dir_exists ".samoyed/_"
ok "Directory structure created"

# Test: Verify wrapper script exists
echo "Testing: Wrapper script"
expect_file_exists ".samoyed/_/samoyed"
ok "Wrapper script exists"

# Test: Verify all git hooks were created
echo "Testing: Git hook scripts"
for hook in applypatch-msg commit-msg post-applypatch post-checkout \
            post-commit post-merge post-rewrite pre-applypatch \
            pre-auto-gc pre-commit pre-merge-commit pre-push \
            pre-rebase prepare-commit-msg; do
    expect_file_exists ".samoyed/_/$hook"
done
ok "All git hooks created"

# Test: Verify sample pre-commit hook was created
echo "Testing: Sample pre-commit hook"
expect_file_exists ".samoyed/pre-commit"
ok "Sample pre-commit hook created"

# Test: Verify core.hooksPath is set correctly
echo "Testing: core.hooksPath configuration"
expect_hooks_path_to_be ".samoyed/_"
ok "core.hooksPath set correctly"

# Test: Create a failing pre-commit hook
echo "Testing: Failing pre-commit hook blocks commits"
create_hook "pre-commit" "echo 'pre-commit hook executed' && exit 1"

# Modify a file to have something to commit
echo "modified content" >> test.txt
git add test.txt

# Test: Verify that the failing hook blocks the commit
expect 1 "git commit -m 'Test commit that should be blocked'"
ok "Failing pre-commit hook blocked commit"

# Test: Create a successful pre-commit hook
echo "Testing: Successful pre-commit hook allows commits"
create_hook "pre-commit" "echo 'pre-commit hook executed' && exit 0"

# Test: Verify that the successful hook allows the commit
expect 0 "git commit -m 'Test commit that should succeed'"
ok "Successful pre-commit hook allowed commit"

# Test: Verify .gitignore was created in _ directory
echo "Testing: .gitignore in _ directory"
expect_file_exists ".samoyed/_/.gitignore"
if grep -q "^\*$" ".samoyed/_/.gitignore"; then
    ok ".gitignore contains wildcard to ignore all files"
else
    error ".gitignore does not contain expected wildcard"
fi

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"