#!/usr/bin/env sh
# Test: Custom directory support
#
# This test verifies that Samoyed can be initialized with a custom directory
# name instead of the default .samoyed directory. This is important for users
# who want to organize their hooks differently or avoid conflicts with other tools.
#
# Tests:
# 1. Initialize with custom directory name
# 2. Verify structure is created in the custom location
# 3. Verify core.hooksPath points to the custom location
# 4. Verify hooks work from the custom location

# Load test helper functions
. tests/integration/functions.sh

# Build Samoyed binary if needed
build_samoyed

# Set up test environment in ./tmp
setup

# Test: Initialize Samoyed with custom directory "my-hooks"
echo "Testing: samoyed init with custom directory 'my-hooks'"
init_samoyed "my-hooks"
ok "Samoyed initialized with custom directory"

# Test: Verify custom directory structure was created
echo "Testing: Custom directory structure"
expect_dir_exists "my-hooks"
expect_dir_exists "my-hooks/_"
ok "Custom directory structure created"

# Test: Verify wrapper script exists in custom location
echo "Testing: Wrapper script in custom location"
expect_file_exists "my-hooks/_/samoyed"
ok "Wrapper script exists in custom location"

# Test: Verify core.hooksPath points to custom location
echo "Testing: core.hooksPath with custom directory"
expect_hooks_path_to_be "my-hooks/_"
ok "core.hooksPath set to custom directory"

# Test: Create and test a pre-commit hook in custom location
echo "Testing: Pre-commit hook in custom directory"
create_hook "pre-commit" "echo 'custom dir pre-commit' && exit 1" "my-hooks"

# Modify file for commit
echo "test modification" >> test.txt
git add test.txt

# Test that hook from custom directory blocks commit
expect 1 "git commit -m 'Test commit with custom dir hook'"
ok "Hook from custom directory blocked commit"

# Test: Initialize with nested custom directory
echo "Testing: Nested custom directory 'hooks/samoyed'"

# Clean up previous init
rm -rf my-hooks
git config --unset core.hooksPath

# Create nested directory structure
mkdir -p hooks

# Initialize in nested location
init_samoyed "hooks/samoyed"
ok "Samoyed initialized in nested directory"

# Test: Verify nested directory structure
echo "Testing: Nested directory structure"
expect_dir_exists "hooks/samoyed"
expect_dir_exists "hooks/samoyed/_"
ok "Nested directory structure created"

# Test: Verify core.hooksPath for nested directory
echo "Testing: core.hooksPath with nested directory"
expect_hooks_path_to_be "hooks/samoyed/_"
ok "core.hooksPath set to nested directory"

# Test: Create and test hook in nested location
echo "Testing: Pre-commit hook in nested directory"
create_hook "pre-commit" "echo 'nested dir pre-commit' && exit 1" "hooks/samoyed"

# Modify file again
echo "another modification" >> test.txt
git add test.txt

# Test that hook from nested directory blocks commit
expect 1 "git commit -m 'Test commit with nested dir hook'"
ok "Hook from nested directory blocked commit"

# Test: Special characters in directory name (dots, dashes, underscores)
echo "Testing: Directory with special characters '.hooks_dir-test'"

# Clean up previous init
rm -rf hooks
git config --unset core.hooksPath

# Initialize with special characters in name
init_samoyed ".hooks_dir-test"
ok "Samoyed initialized with special character directory"

# Test: Verify directory with special characters
echo "Testing: Directory structure with special characters"
expect_dir_exists ".hooks_dir-test"
expect_dir_exists ".hooks_dir-test/_"
ok "Directory with special characters created"

# Test: Verify core.hooksPath with special characters
echo "Testing: core.hooksPath with special character directory"
expect_hooks_path_to_be ".hooks_dir-test/_"
ok "core.hooksPath set correctly with special characters"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"