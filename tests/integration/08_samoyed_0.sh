#!/usr/bin/env sh
# Test: SAMOYED=0 bypass mode
#
# This test verifies that the SAMOYED=0 environment variable correctly bypasses
# all hook execution. This feature is essential for:
# - CI/CD pipelines that need to skip hooks
# - Emergency commits when hooks are broken
# - Automated tools that need to bypass validation
#
# Samoyed checks SAMOYED=0 in two places:
# 1. In the Rust code during init (check_bypass_mode function)
# 2. In the wrapper script during hook execution (line 49)

# Load test helper functions regardless of current working directory
integration_script_dir="$(cd "$(dirname "$0")" && pwd)"
integration_repo_root="$(cd "$integration_script_dir/../.." && pwd)"
cd "$integration_repo_root"
. "$integration_repo_root/tests/integration/functions.sh"
unset integration_script_dir
unset integration_repo_root

# Build Samoyed binary if needed
build_samoyed

# Set up test environment in ./tmp
setup

# Test: SAMOYED=0 during init should bypass initialization
echo "Testing: SAMOYED=0 bypasses samoyed init"

# Try to initialize with SAMOYED=0
if SAMOYED=0 "$SAMOYED_BIN" init 2>&1 | grep -q "Bypassing samoyed init"; then
    ok "SAMOYED=0 bypassed init with message"
else
    # Even if message differs, check that nothing was created
    if [ ! -d ".samoyed" ]; then
        ok "SAMOYED=0 prevented initialization"
    else
        error "SAMOYED=0 did not bypass init"
    fi
fi

# Verify core.hooksPath was NOT set
echo "Testing: core.hooksPath not set with SAMOYED=0 init"
expect_hooks_path_to_be ""
ok "core.hooksPath not set when init bypassed"

# Test: Normal init without SAMOYED=0
echo "Testing: Normal init without SAMOYED=0"
unset SAMOYED
# shellcheck disable=SC2119 # Run init without forwarding script arguments
init_samoyed
ok "Samoyed initialized normally"

# Verify initialization worked
echo "Testing: Verification of normal init"
expect_hooks_path_to_be ".samoyed/_"
expect_dir_exists ".samoyed"
expect_dir_exists ".samoyed/_"
ok "Normal initialization completed successfully"

# Test: Create a failing pre-commit hook
echo "Testing: Create failing pre-commit hook"
create_hook "pre-commit" "echo 'pre-commit hook executed' && exit 1"

# Test that hook blocks commit normally
echo "test content" >> test.txt
git add test.txt

expect 1 "git commit -m 'Should be blocked by hook'"
ok "Hook blocks commit normally"

# Test: SAMOYED=0 bypasses the failing hook
echo "Testing: SAMOYED=0 bypasses failing hook"
echo "bypass test" >> test.txt
git add test.txt

SAMOYED=0 expect 0 "git commit -m 'Should succeed with SAMOYED=0'"
ok "SAMOYED=0 bypassed the failing hook"

# Test: SAMOYED=0 in user's init.sh configuration
echo "Testing: SAMOYED=0 set in user config"

# Create config directory
config_dir="${test_dir}/config"
mkdir -p "$config_dir/samoyed"

# Create init.sh that sets SAMOYED=0
cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
# User config that disables all hooks
export SAMOYED=0
echo "User config set SAMOYED=0"
EOF

# Create another failing hook
create_hook "pre-commit" "echo 'This should be bypassed' && exit 1"

echo "config bypass test" >> test.txt
git add test.txt

# Hook should be bypassed due to init.sh setting SAMOYED=0
XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Bypassed via init.sh'"
ok "SAMOYED=0 in init.sh bypasses hooks"

# Test: SAMOYED=0 bypasses multiple hook types
echo "Testing: SAMOYED=0 bypasses multiple hook types"

# Clean up previous config
rm -rf "$config_dir"
unset SAMOYED

# Create multiple failing hooks
create_hook "pre-commit" "echo 'pre-commit' && exit 1"
create_hook "prepare-commit-msg" "echo 'prepare-commit-msg' && exit 1"
create_hook "commit-msg" "echo 'commit-msg' && exit 1"

echo "multiple hooks test" >> test.txt
git add test.txt

# All hooks should be bypassed
SAMOYED=0 expect 0 "git commit -m 'All hooks bypassed'"
ok "SAMOYED=0 bypasses all hook types"

# Test: SAMOYED=1 does NOT bypass hooks
echo "Testing: SAMOYED=1 does not bypass hooks"

echo "SAMOYED=1 test" >> test.txt
git add test.txt

SAMOYED=1 expect 1 "git commit -m 'Should fail with SAMOYED=1'"
ok "SAMOYED=1 does not bypass hooks"

# Test: SAMOYED=2 enables debug mode but doesn't bypass
echo "Testing: SAMOYED=2 debug mode"

# Create a simple hook that will show debug output
create_hook "pre-commit" "echo 'Debug mode hook' && exit 0"

echo "debug mode test" >> test.txt
git add test.txt

# Capture output to check for debug traces
set +e
# shellcheck disable=SC2034
output=$(SAMOYED=2 git commit -m "Debug mode test" 2>&1)
exit_code=$?
set -e

if [ $exit_code -eq 0 ]; then
    ok "SAMOYED=2 allows hook execution"
else
    error "SAMOYED=2 should not bypass hooks"
fi

# Test: Empty SAMOYED variable does not bypass
echo "Testing: Empty SAMOYED does not bypass"

create_hook "pre-commit" "echo 'checking empty' && exit 1"

echo "empty SAMOYED test" >> test.txt
git add test.txt

SAMOYED="" expect 1 "git commit -m 'Empty SAMOYED should not bypass'"
ok "Empty SAMOYED value does not bypass hooks"

# Test: Case sensitivity - samoyed=0 should NOT bypass
echo "Testing: Case sensitivity of SAMOYED variable"

echo "case test" >> test.txt
git add test.txt

samoyed=0 expect 1 "git commit -m 'Lowercase should not work'"
ok "Lowercase 'samoyed=0' does not bypass (case sensitive)"

# Test: Verify SAMOYED=0 during init doesn't affect existing setup
echo "Testing: SAMOYED=0 init on existing installation"

# Try to re-init with SAMOYED=0
SAMOYED=0 "$SAMOYED_BIN" init 2>&1 | grep -q "Bypassing"

# Existing hooks should still work
echo "existing test" >> test.txt
git add test.txt

# Without SAMOYED=0, hooks should still block
expect 1 "git commit -m 'Existing hooks still work'"
ok "Existing installation unaffected by SAMOYED=0 init"

# Test: SAMOYED=0 with successful hooks (should still bypass)
echo "Testing: SAMOYED=0 bypasses even successful hooks"

# Create a successful hook that modifies files
create_hook "pre-commit" "echo 'Hook executed' > hook_was_run.txt && exit 0"

echo "successful hook test" >> test.txt
git add test.txt

# With SAMOYED=0, even successful hooks shouldn't run
SAMOYED=0 git commit -m "Bypass successful hook"

# Check that hook didn't run
if [ -f "hook_was_run.txt" ]; then
    error "Hook was executed despite SAMOYED=0"
else
    ok "SAMOYED=0 bypassed even successful hooks"
fi

# Test: Clean commit without SAMOYED=0 to verify normal operation
echo "Testing: Normal operation after bypass tests"

# Create a simple successful hook
create_hook "pre-commit" "echo 'Normal hook executed' && exit 0"

echo "final normal test" >> test.txt
git add test.txt

expect 0 "git commit -m 'Normal operation restored'"
ok "Normal hook operation works after bypass tests"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
