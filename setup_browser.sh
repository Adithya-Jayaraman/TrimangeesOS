#!/usr/bin/env bash
# ============================================================
# Trimangees Browser Setup Script
# Run this once after cloning to wire up the Electron browser
# ============================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BROWSER_DIR="$SCRIPT_DIR/browser"

echo "==> Trimangees Browser Setup"
echo "    Looking for browser files in: $BROWSER_DIR"

# Check the browser directory exists
if [ ! -d "$BROWSER_DIR" ]; then
  echo ""
  echo "ERROR: $BROWSER_DIR not found."
  echo ""
  echo "Please create a 'browser/' folder next to this script and put"
  echo "the following files inside it:"
  echo "  - browser.html"
  echo "  - main.js"
  echo "  - preload.js"
  echo "  - package.json"
  exit 1
fi

# Check required files
for f in browser.html main.js preload.js package.json; do
  if [ ! -f "$BROWSER_DIR/$f" ]; then
    echo "ERROR: missing $BROWSER_DIR/$f"
    exit 1
  fi
done
echo "    All browser files found."

# Install npm dependencies
echo "==> Installing Electron..."
cd "$BROWSER_DIR"
npm install

echo "==> Testing browser launch..."
if command -v electron &>/dev/null; then
  echo "    electron found in PATH."
else
  echo "    electron not in PATH — will use npx electron."
fi

# Write env file for the shell
ENV_FILE="$SCRIPT_DIR/shell/.trimangees_env"
cat > "$ENV_FILE" << EOF
TRIMANGEES_BROWSER_DIR=$BROWSER_DIR
TRIMANGEES_ASSETS_DIR=$SCRIPT_DIR/apps
EOF
echo "==> Wrote $ENV_FILE"

echo ""
echo "Done! Start the OS with:"
echo "  cd shell && source ../.trimangees_env && cargo run"