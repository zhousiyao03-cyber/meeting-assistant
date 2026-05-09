#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT=$(pwd)

echo "==> Building VoiceNote.app via Tauri"
pnpm tauri build

DMG_DIR="$ROOT/src-tauri/target/release/bundle/dmg"

DMG_PATH=$(ls "$DMG_DIR"/VoiceNote*.dmg 2>/dev/null | head -1 || true)

if [[ -z "$DMG_PATH" || ! -f "$DMG_PATH" ]]; then
  echo "ERROR: DMG not found in $DMG_DIR"
  ls -la "$DMG_DIR" 2>/dev/null || true
  exit 1
fi

echo "==> DMG built: $DMG_PATH"
echo "==> Size: $(du -h "$DMG_PATH" | awk '{print $1}')"
echo
echo "Next steps:"
echo "  - If unsigned: distribute as-is. Users right-click → Open the first time."
echo "  - If Apple Dev approved: ./scripts/notarize.sh \"$DMG_PATH\""
echo "  - Upload to confide.knosi.xyz/download/VoiceNote.dmg:"
echo "      scp \"$DMG_PATH\" knosi:/srv/confide-landing/download/VoiceNote.dmg"
