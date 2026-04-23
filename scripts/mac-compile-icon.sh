#!/usr/bin/env bash
#
# Compile the macOS 26 Tahoe Liquid Glass icon and inject it into the
# already-bundled .app. Runs AFTER `tauri build` because Tauri v2 doesn't
# know about `.icon` yet (tauri-apps/tauri#14207).
#
# On non-macOS hosts this is a no-op so cross-platform contributors can run
# the same build scripts unchanged.

set -euo pipefail

if [[ "$(uname)" != "Darwin" ]]; then
  echo "mac-compile-icon: host is not macOS — skipping"
  exit 0
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICON_SRC="${PROJECT_ROOT}/src-tauri/icons/AppIcon.icon"
PROFILE="${1:-debug}"
APP="${PROJECT_ROOT}/src-tauri/target/${PROFILE}/bundle/macos/OpenDesk.app"

if [[ ! -d "${ICON_SRC}" ]]; then
  echo "mac-compile-icon: no ${ICON_SRC} — skipping (build without Liquid Glass)"
  exit 0
fi

if [[ ! -d "${APP}" ]]; then
  echo "mac-compile-icon: ${APP} not found — did you run \`tauri build\`?"
  exit 1
fi

if ! command -v xcrun >/dev/null 2>&1; then
  echo "mac-compile-icon: xcrun missing (install Xcode or Command Line Tools)"
  exit 1
fi

TMP_PLIST="$(mktemp -t actool-plist.XXXXXX).plist"
trap 'rm -f "${TMP_PLIST}"' EXIT

echo "mac-compile-icon: compiling ${ICON_SRC} → ${APP}/Contents/Resources/"
xcrun actool "${ICON_SRC}" \
  --compile "${APP}/Contents/Resources/" \
  --platform macosx \
  --minimum-deployment-target 11.0 \
  --app-icon AppIcon \
  --output-partial-info-plist "${TMP_PLIST}" \
  >/dev/null

PLIST="${APP}/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "${PLIST}"
if ! /usr/libexec/PlistBuddy -c "Print :CFBundleIconName" "${PLIST}" >/dev/null 2>&1; then
  /usr/libexec/PlistBuddy -c "Add :CFBundleIconName string AppIcon" "${PLIST}"
else
  /usr/libexec/PlistBuddy -c "Set :CFBundleIconName AppIcon" "${PLIST}"
fi

# Tauri-bundled icon.icns would otherwise race against actool's AppIcon.icns
# on macOS <26 resolution and on Tahoe leaves a squircle-jailed fallback.
rm -f "${APP}/Contents/Resources/icon.icns"

echo "mac-compile-icon: done (Assets.car + AppIcon.icns installed, plist patched)"
