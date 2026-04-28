#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "[meeting-recorder] Building Swift sidecar (arm64 release)..."
swift build -c release --arch arm64

BIN_DIR=$(swift build -c release --arch arm64 --show-bin-path 2>/dev/null || echo ".build/arm64-apple-macosx/release")
SRC_BIN="$BIN_DIR/MeetingRecorderSidecar"

mkdir -p dist
cp "$SRC_BIN" "dist/meeting-recorder-aarch64-apple-darwin"
chmod +x "dist/meeting-recorder-aarch64-apple-darwin"
ln -sfn "meeting-recorder-aarch64-apple-darwin" "dist/meeting-recorder"

echo "[meeting-recorder] Built: $(pwd)/dist/meeting-recorder-aarch64-apple-darwin"
