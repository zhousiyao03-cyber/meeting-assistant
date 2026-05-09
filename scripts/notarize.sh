#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${1:-}" ]]; then
  echo "Usage: $0 <path-to-dmg>"
  exit 1
fi
DMG="$1"

# Required env vars (from your Apple Developer Account):
#   APPLE_ID                — your Apple developer account email
#   APPLE_TEAM_ID           — Team ID (Member Center > Account)
#   APP_SPECIFIC_PASSWORD   — generate at appleid.apple.com (App-Specific Passwords)

: "${APPLE_ID:?Set APPLE_ID env var (Apple developer account email)}"
: "${APPLE_TEAM_ID:?Set APPLE_TEAM_ID env var}"
: "${APP_SPECIFIC_PASSWORD:?Set APP_SPECIFIC_PASSWORD env var}"

echo "==> Submitting $DMG to Apple notarization service (this may take 5-15 minutes)"
xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APP_SPECIFIC_PASSWORD" \
  --wait

echo "==> Stapling notarization ticket"
xcrun stapler staple "$DMG"

echo "==> ✅ Notarization complete: $DMG"
