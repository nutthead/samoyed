#!/usr/bin/env sh
# Test: Compatibility with set -u (treat unset variables as errors)
#
# This test verifies that Samoyed's wrapper script works correctly when users
# have 'set -u' in their shell configuration. The 'set -u' option causes the
# shell to exit with an error when an unset variable is referenced.
#
# Samoyed's wrapper script uses ${SAMOYED-} with default expansion (line 49)
# and ${XDG_CONFIG_HOME:-$HOME/.config} to handle this properly.

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

# Verify core.hooksPath is set
echo "Testing: core.hooksPath configuration"
expect_hooks_path_to_be ".samoyed/_"
ok "core.hooksPath set correctly"

# Test: Create a simple pre-commit hook
echo "Testing: Basic hook without set -u"
create_hook "pre-commit" "echo 'pre-commit executed'"

echo "test" >> test.txt
git add test.txt

# Should work without set -u
expect 0 "git commit -m 'Test without set -u'"
ok "Hook works without set -u"

# Test: Create init.sh with set -u
echo "Testing: User init.sh with set -u"

# Create config directory structure
config_dir="${test_dir}/config"
mkdir -p "$config_dir/samoyed"

# Create init.sh with set -u
cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
# User configuration with strict mode
set -u

# This should not cause errors even though SAMOYED might be unset
echo "User init.sh loaded with set -u"
EOF

# Test with XDG_CONFIG_HOME pointing to our config
echo "test with set -u" >> test.txt
git add test.txt

# Run with custom config
XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Test with set -u in init.sh'"
ok "Hook works with set -u in user init.sh"

# Test: init.sh that uses unset variables (should fail)
echo "Testing: init.sh with unset variable reference"

cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
set -u
# This will fail because UNDEFINED_VAR is not set
echo "Value is: $UNDEFINED_VAR"
EOF

echo "unset var test" >> test.txt
git add test.txt

# This should fail due to unset variable in init.sh
set +e
XDG_CONFIG_HOME="$config_dir" git commit -m "Test with unset var" 2>/dev/null
exit_code=$?
set -e

if [ $exit_code -eq 0 ]; then
    error "Commit should have failed with unset variable in init.sh"
fi
ok "Hook correctly failed with unset variable error"

# Test: init.sh with proper variable handling under set -u
echo "Testing: init.sh with proper variable handling"

cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
set -u

# Proper way to handle potentially unset variables
MY_VAR="${MY_VAR:-default_value}"
echo "MY_VAR is: $MY_VAR"

# Check if variable is set before using
if [ "${OPTIONAL_VAR+set}" = "set" ]; then
    echo "OPTIONAL_VAR is set to: $OPTIONAL_VAR"
else
    echo "OPTIONAL_VAR is not set"
fi

# Export a variable for the hook
export HOOK_ENV="from_init"
EOF

# Create a hook that uses the exported variable
# shellcheck disable=SC2016 # The variable expands later when the hook runs
create_hook "pre-commit" 'echo "HOOK_ENV is: ${HOOK_ENV:-not_set}"'

echo "proper handling test" >> test.txt
git add test.txt

# Should work with proper variable handling
XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Test with proper var handling'"
ok "Hook works with proper variable handling under set -u"

# Test: Verify SAMOYED variable is handled correctly with set -u
echo "Testing: SAMOYED variable handling with set -u"

cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
set -u

# The wrapper script should handle SAMOYED correctly
# even when set -u is active
echo "Init script with set -u loaded"
EOF

# Test without SAMOYED set
unset SAMOYED
echo "SAMOYED unset test" >> test.txt
git add test.txt

XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Test SAMOYED unset with set -u'"
ok "Works with SAMOYED unset under set -u"

# Test with SAMOYED=0 (bypass mode)
export SAMOYED=0
echo "SAMOYED=0 test" >> test.txt
git add test.txt

# Create a failing hook to verify bypass works
create_hook "pre-commit" "echo 'This should be bypassed' && exit 1"

XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Test SAMOYED=0 with set -u'"
ok "SAMOYED=0 bypass works with set -u"

unset SAMOYED

# Test: Multiple environment variables with set -u
echo "Testing: Multiple env vars with set -u"

cat > "$config_dir/samoyed/init.sh" << 'EOF'
#!/bin/sh
set -u
set -e

# Test various environment variable patterns
HOME="${HOME:-/tmp}"
PATH="${PATH:-/usr/bin:/bin}"
USER="${USER:-unknown}"
CUSTOM="${CUSTOM:-default}"

echo "Environment configured with set -u"
EOF

echo "multi env test" >> test.txt
git add test.txt

# Create successful hook
create_hook "pre-commit" "echo 'Hook running' && exit 0"

XDG_CONFIG_HOME="$config_dir" expect 0 "git commit -m 'Test multiple env vars'"
ok "Multiple environment variables handled correctly with set -u"

# Test: Verify hook execution continues to work normally
echo "Testing: Normal hook execution after set -u tests"

# Remove custom config
rm -rf "$config_dir"

echo "final test" >> test.txt
git add test.txt

# Should work normally without custom config
expect 0 "git commit -m 'Final test after set -u tests'"
ok "Normal execution works after set -u tests"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
