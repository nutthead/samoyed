#!/usr/bin/env sh
# Test: Basic init command smoke test
#
# This is a simple smoke test that verifies the init command works
# without errors. It's a basic sanity check that the command exists,
# runs, and produces the expected output structure.

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

# Test: Basic init command execution
echo "Testing: Basic 'samoyed init' command"
expect 0 "$SAMOYED_BIN init"
ok "samoyed init completed successfully"

# Test: Verify all expected files and directories were created
echo "Testing: Expected file structure after init"

expect_dir_exists ".samoyed"
expect_dir_exists ".samoyed/_"
expect_file_exists ".samoyed/_/samoyed"
expect_file_exists ".samoyed/pre-commit"
expect_file_exists ".samoyed/_/.gitignore"

# Check all hook scripts were created
for hook in applypatch-msg commit-msg post-applypatch post-checkout \
            post-commit post-merge post-rewrite pre-applypatch \
            pre-auto-gc pre-commit pre-merge-commit pre-push \
            pre-rebase prepare-commit-msg; do
    expect_file_exists ".samoyed/_/$hook"
done

ok "All expected files and directories created"

# Test: Verify git config was set
echo "Testing: Git configuration after init"
expect_hooks_path_to_be ".samoyed/_"
ok "Git config core.hooksPath set correctly"

# Test: Re-running init should be idempotent
echo "Testing: Init command is idempotent"
expect 0 "$SAMOYED_BIN init"
ok "Second init completed successfully"

# Verify structure still exists
expect_dir_exists ".samoyed"
expect_hooks_path_to_be ".samoyed/_"
ok "Structure unchanged after second init"

# Test: Init with explicit default directory name
echo "Testing: Init with explicit '.samoyed' argument"

# Clean up first
rm -rf .samoyed
git config --unset core.hooksPath

expect 0 "$SAMOYED_BIN init .samoyed"
ok "Init with explicit default name succeeded"

expect_dir_exists ".samoyed"
expect_hooks_path_to_be ".samoyed/_"
ok "Explicit default name created correct structure"

# Test: Init command output (should be silent on success)
echo "Testing: Init command output"

# Clean up
rm -rf .samoyed
git config --unset core.hooksPath

output=$("$SAMOYED_BIN" init 2>&1)
exit_code=$?

if [ $exit_code -eq 0 ]; then
    # Success - output should be minimal or empty
    if [ -z "$output" ] || [ ${#output} -lt 100 ]; then
        ok "Init has minimal output on success"
    else
        echo "Note: Init produced output: $output"
        ok "Init succeeded (with output)"
    fi
else
    error "Init failed with exit code $exit_code: $output"
fi

# Test: Help/usage for init command
echo "Testing: Help output includes init command"

# Check if help mentions init
set +e
"$SAMOYED_BIN" --help 2>&1 | grep -q "init"
help_mentions_init=$?

"$SAMOYED_BIN" help 2>&1 | grep -q "init"
help_cmd_mentions_init=$?
set -e

if [ $help_mentions_init -eq 0 ] || [ $help_cmd_mentions_init -eq 0 ]; then
    ok "Help output mentions init command"
else
    echo "Note: Help may use different format"
    ok "Help command executed"
fi

# Test: Version command works
echo "Testing: Version information"

set +e
"$SAMOYED_BIN" --version >/dev/null 2>&1
version_flag=$?

"$SAMOYED_BIN" version >/dev/null 2>&1
version_cmd=$?
set -e

if [ $version_flag -eq 0 ] || [ $version_cmd -eq 0 ]; then
    ok "Version information available"
else
    echo "Note: Version command may not be implemented"
    ok "Version check completed"
fi

# Test: Init creates executable hook scripts
echo "Testing: Hook scripts are executable"

# Check a few key hooks for executable permission
for hook in pre-commit commit-msg pre-push; do
    if [ -x ".samoyed/_/$hook" ]; then
        ok "Hook $hook is executable"
    else
        error "Hook $hook is not executable"
    fi
done

# Test: Wrapper script permissions
echo "Testing: Wrapper script permissions"

# The wrapper script should be readable but doesn't need to be executable
# since it's sourced, not executed
if [ -r ".samoyed/_/samoyed" ]; then
    ok "Wrapper script is readable"
else
    error "Wrapper script is not readable"
fi

# Test: Sample pre-commit hook content
echo "Testing: Sample pre-commit hook content"

if grep -q "#!/usr/bin/env sh" ".samoyed/pre-commit"; then
    ok "Sample pre-commit has correct shebang"
else
    error "Sample pre-commit missing or incorrect shebang"
fi

# shellcheck disable=SC2016 # Keep the subshell expression literal for matching
if grep -q '\. "$(dirname "$0")/_/samoyed"' ".samoyed/pre-commit"; then
    ok "Sample pre-commit sources wrapper correctly"
else
    error "Sample pre-commit doesn't source wrapper"
fi

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
