#!/usr/bin/env sh
# Test: Running Samoyed from a subdirectory
#
# This test verifies that Samoyed correctly finds the git repository root
# when executed from a subdirectory. This is crucial because developers
# often work in subdirectories of their projects.
#
# Samoyed uses 'git rev-parse --show-toplevel' to find the repository root,
# so it should work correctly regardless of the current working directory.

# Load test helper functions
. tests/integration/functions.sh

# Build Samoyed binary if needed
build_samoyed

# Set up test environment in ./tmp
setup

# Test: Create a subdirectory structure
echo "Testing: Creating subdirectory structure"
mkdir -p src/components/ui
mkdir -p docs/api
ok "Subdirectory structure created"

# Test: Initialize Samoyed from a deep subdirectory
echo "Testing: Initialize from src/components/ui subdirectory"
cd src/components/ui

# Run samoyed init from subdirectory - should create .samoyed at repo root
# shellcheck disable=SC2119 # Run init without forwarding script arguments
init_samoyed
ok "Samoyed initialized from subdirectory"

# Go back to repo root to verify
cd ../../..

# Test: Verify .samoyed was created at repository root, not in subdirectory
echo "Testing: Samoyed directory created at repository root"
expect_dir_exists ".samoyed"
expect_dir_exists ".samoyed/_"
ok "Samoyed directory created at correct location (repo root)"

# Test: Verify subdirectories don't have .samoyed
echo "Testing: No .samoyed in subdirectories"
if [ -d "src/components/ui/.samoyed" ]; then
    error ".samoyed incorrectly created in subdirectory"
fi
ok "No .samoyed in subdirectories"

# Test: Verify core.hooksPath is set correctly
echo "Testing: core.hooksPath from subdirectory init"
expect_hooks_path_to_be ".samoyed/_"
ok "core.hooksPath set correctly from subdir init"

# Test: Run init again from different subdirectory with custom dir
echo "Testing: Re-init from docs/api with custom directory"

# Clean up previous init
rm -rf .samoyed
git config --unset core.hooksPath

cd docs/api
init_samoyed ".hooks"
ok "Samoyed re-initialized from different subdirectory"

# Go back to root
cd ../..

# Test: Verify custom directory at root
echo "Testing: Custom directory at repository root"
expect_dir_exists ".hooks"
expect_dir_exists ".hooks/_"
ok "Custom directory created at repo root"

# Test: Verify hooks work when commit is made from subdirectory
echo "Testing: Hooks execution from subdirectory"
create_hook "pre-commit" "echo 'pre-commit from root' && exit 1" ".hooks"

# Go to subdirectory and try to commit
cd src/components

# Create a file in subdirectory
echo "component code" > Button.js
git add Button.js

# Test that hook blocks commit from subdirectory
expect 1 "git commit -m 'Add Button component'"
ok "Hook executed correctly from subdirectory"

# Test: Verify hook with success to allow commit
create_hook "pre-commit" "echo 'pre-commit check passed' && exit 0" "../../.hooks"
expect 0 "git commit -m 'Add Button component'"
ok "Successful hook allows commit from subdirectory"

# Test: Complex case - init with path to parent directory's custom location
echo "Testing: Init with relative path from subdirectory"

# Clean up
cd ../..
rm -rf .hooks
git config --unset core.hooksPath

# Create a complex structure
mkdir -p project/src
cd project/src

# Initialize with a path that goes up and back down
init_samoyed "../.project-hooks"
ok "Samoyed initialized with relative parent path"

# Go back to root
cd ../..

# Test: Verify the structure was created correctly
echo "Testing: Complex relative path structure"
expect_dir_exists "project/.project-hooks"
expect_dir_exists "project/.project-hooks/_"
ok "Complex relative path handled correctly"

# Test: Verify core.hooksPath with complex path
echo "Testing: core.hooksPath with complex relative path"
expect_hooks_path_to_be "project/.project-hooks/_"
ok "core.hooksPath set correctly with complex path"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
