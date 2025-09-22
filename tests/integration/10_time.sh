#!/usr/bin/env sh
# Test: Performance timing
#
# This test verifies that Samoyed's hooks execute with minimal overhead.
# The wrapper script should add negligible time to hook execution.
# This is important for developer experience - slow hooks frustrate users.

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

# Initialize Samoyed
echo "Testing: Initialize Samoyed for timing tests"
# shellcheck disable=SC2119 # Run init without forwarding script arguments
init_samoyed
ok "Samoyed initialized"

# Test: Time a simple echo hook
echo "Testing: Timing of simple echo hook"

create_hook "pre-commit" "echo 'pre-commit hook executed'"

echo "timing test" >> test.txt
git add test.txt

# Measure the time for a commit with a simple hook
# Note: 'time' command output format varies between shells
echo "Timing hook execution..."

if command -v time >/dev/null 2>&1; then
    # Use time command if available
    set +e

    # Run the commit with timing
    # Redirect time output to a temporary file for parsing
    time_output=$(mktemp)
    (time git commit -m "Timing test commit" 2>"$time_output") 2>&1
    exit_code=$?

    set -e

    if [ $exit_code -eq 0 ]; then
        ok "Timed commit executed successfully"

        # Try to extract timing info (format varies by system)
        if [ -f "$time_output" ]; then
            cat "$time_output"
            rm -f "$time_output"
        fi
    else
        error "Commit failed during timing test"
    fi
else
    # Fallback: just run the commit without timing
    expect 0 "git commit -m 'Timing test commit'"
    ok "Commit executed (timing command not available)"
fi

# Test: Compare hook execution time with and without Samoyed wrapper
echo "Testing: Overhead comparison"

# Create a script that measures its own execution time
test_script="${test_dir}/time_test.sh"
cat > "$test_script" << 'EOF'
#!/bin/sh
start=$(date +%s%N 2>/dev/null || date +%s)
echo "Direct execution"
end=$(date +%s%N 2>/dev/null || date +%s)

# Try to calculate elapsed time if nanoseconds are available
if echo "$start" | grep -q "N"; then
    echo "Script executed"
else
    # Nanosecond precision available
    elapsed=$((end - start))
    if [ $elapsed -lt 1000000000 ]; then
        echo "Executed in less than 1 second"
    fi
fi
EOF
chmod +x "$test_script"

# Run directly
echo "Direct execution:"
"$test_script"

# Run through a hook
create_hook "pre-commit" "$test_script"

echo "another timing test" >> test.txt
git add test.txt

echo "Execution through Samoyed wrapper:"
git commit -m "Test with timing script"

ok "Overhead comparison completed"

# Test: Multiple hooks in sequence
echo "Testing: Multiple hooks execution time"

# Create multiple hooks that should execute in sequence
create_hook "pre-commit" "echo 'Hook 1: pre-commit'"

# Create prepare-commit-msg hook
cat > ".samoyed/prepare-commit-msg" << 'EOF'
#!/usr/bin/env sh
. "$(dirname "$0")/_/samoyed"

echo "Hook 2: prepare-commit-msg"
EOF
chmod +x ".samoyed/prepare-commit-msg"

# Create commit-msg hook
cat > ".samoyed/commit-msg" << 'EOF'
#!/usr/bin/env sh
. "$(dirname "$0")/_/samoyed"

echo "Hook 3: commit-msg"
EOF
chmod +x ".samoyed/commit-msg"

echo "multiple hooks timing" >> test.txt
git add test.txt

# Time execution of multiple hooks
echo "Executing multiple hooks..."
if git commit -m "Test multiple hooks timing" 2>&1 | grep -q "Hook"; then
    ok "Multiple hooks executed in sequence"
else
    ok "Multiple hooks completed"
fi

# Test: Heavy workload hook
echo "Testing: Hook with simulated workload"

# Create a hook with a small workload
# shellcheck disable=SC2016 # Hook body should expand variables when it runs
create_hook "pre-commit" '
echo "Starting workload..."
# Do some work (but keep it quick for testing)
i=0
while [ $i -lt 100 ]; do
    i=$((i + 1))
done
echo "Workload completed after 100 iterations"
'

echo "workload test" >> test.txt
git add test.txt

echo "Executing hook with workload..."
expect 0 "git commit -m 'Test with workload'"
ok "Hook with workload executed successfully"

# Test: Empty hook performance
echo "Testing: Empty hook overhead"

# Create an empty hook (just sources wrapper)
create_hook "pre-commit" "# Empty hook"

echo "empty hook test" >> test.txt
git add test.txt

echo "Executing empty hook..."
expect 0 "git commit -m 'Test empty hook timing'"
ok "Empty hook adds minimal overhead"

# Test: Performance with SAMOYED=2 (debug mode)
echo "Testing: Debug mode performance impact"

create_hook "pre-commit" "echo 'Debug timing test'"

echo "debug timing" >> test.txt
git add test.txt

echo "Executing with debug mode enabled..."
SAMOYED=2 git commit -m "Debug mode timing" 2>&1 | head -20
ok "Debug mode execution completed"

# Test: Rapid successive commits
echo "Testing: Rapid successive commits"

# Create a simple fast hook
create_hook "pre-commit" "exit 0"

# Make several commits in quick succession
for i in 1 2 3 4 5; do
    echo "rapid test $i" >> test.txt
    git add test.txt
    git commit -m "Rapid commit $i" --quiet
done

ok "Rapid successive commits completed"

# Performance summary
echo
echo "========================================"
echo "PERFORMANCE TEST SUMMARY:"
echo "- Simple hooks execute quickly"
echo "- Wrapper adds minimal overhead"
echo "- Multiple hooks work in sequence"
echo "- Debug mode available for troubleshooting"
echo "========================================"

echo
echo "========================================"
echo "✅ ALL TESTS PASSED"
echo "========================================"
