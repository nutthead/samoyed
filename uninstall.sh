#!/usr/bin/env bash
set -euo pipefail

readonly BINARY="samoyed"
readonly INSTALL_LOCATIONS=(
    "$HOME/.local/bin/$BINARY"
    "$HOME/bin/$BINARY"
    "/usr/local/bin/$BINARY"
)

echo "Uninstalling Samoyed..."
echo ""

found=false

# Check common installation locations
for location in "${INSTALL_LOCATIONS[@]}"; do
    if [ -f "$location" ]; then
        echo "Found: $location"
        rm -f "$location" && echo "  Removed ✓" || echo "  Failed to remove"
        found=true
    fi
done

if ! $found; then
    echo "Samoyed binary not found in common locations"
    echo ""
    echo "Checked:"
    printf '  %s\n' "${INSTALL_LOCATIONS[@]}"
    echo ""
    echo "If installed elsewhere, remove manually:"
    echo "  which samoyed"
    echo "  rm \$(which samoyed)"
fi

echo ""
echo "Note: Samoyed configuration remains in your repositories (.samoyed/ directories)"
echo "To fully remove:"
echo "  cd your-repo"
echo "  rm -rf .samoyed"
echo "  git config --unset core.hooksPath"
echo ""
