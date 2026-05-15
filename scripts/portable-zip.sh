#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE_DIR="$ROOT/src-tauri/target/release"
BUNDLE_DIR="$RELEASE_DIR/hermes-remote-manager-portable"
OUTPUT_DIR="$ROOT/artifacts"

mkdir -p "$OUTPUT_DIR" "$BUNDLE_DIR"

# Copy binary (Tauri 2.x uses kebab-case for the binary name)
if [ -f "$RELEASE_DIR/hermes-remote-manager" ]; then
    cp "$RELEASE_DIR/hermes-remote-manager" "$BUNDLE_DIR/"
elif [ -f "$RELEASE_DIR/hermes_remote_manager" ]; then
    cp "$RELEASE_DIR/hermes_remote_manager" "$BUNDLE_DIR/"
else
    echo "❌ Binary not found in $RELEASE_DIR"
    exit 1
fi

# Copy resources (app.asar, locales, icons...)
if [ -d "$RELEASE_DIR/resources" ]; then
    cp -r "$RELEASE_DIR/resources" "$BUNDLE_DIR/"
fi

# Copy sidecar files (.exe sidecar if exists)
if [ -d "$RELEASE_DIR/hermes-remote-manager" ]; then
    cp -r "$RELEASE_DIR/hermes-remote-manager/"* "$BUNDLE_DIR/" 2>/dev/null || true
fi

# Copy LICENSE if exists
if [ -f "$ROOT/LICENSE" ]; then
    cp "$ROOT/LICENSE" "$BUNDLE_DIR/" 2>/dev/null || true
fi

# Create zip
ZIP_NAME="hermes-remote-manager-v${VERSION}-portable.zip"
cd "$OUTPUT_DIR"
if command -v 7z &>/dev/null; then
    7z a "$ZIP_NAME" "$BUNDLE_DIR/*" > /dev/null
elif command -v zip &>/dev/null; then
    zip -r "$ZIP_NAME" -j "$BUNDLE_DIR"/* > /dev/null
else
    echo "❌ Neither 7z nor zip available"
    exit 1
fi

echo "✅ Created: $OUTPUT_DIR/$ZIP_NAME"

# Cleanup
rm -rf "$BUNDLE_DIR"