#!/usr/bin/env bash
# Build the pure-Rust binary for iOS and wrap it in an .app bundle.
#
#   scripts/ios.sh sim            # build for the simulator and launch it
#   scripts/ios.sh device         # build for a real iPhone (needs signing)
#
# Requires Xcode command-line tools and the matching Rust targets:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# For a device, set CODESIGN_IDENTITY to a development certificate, e.g.
#   CODESIGN_IDENTITY="Apple Development: You (TEAMID)" scripts/ios.sh device
set -euo pipefail

kind="${1:-sim}"
case "$kind" in
  sim)    target="aarch64-apple-ios-sim" ;;
  device) target="aarch64-apple-ios" ;;
  *) echo "usage: $0 [sim|device]" >&2; exit 2 ;;
esac

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release -p ames-app --target "$target"

app="target/ios/$kind/Ames Window.app"
rm -rf "$app"
mkdir -p "$app"
cp "target/$target/release/ames-window" "$app/"
cp ios/Info.plist "$app/"

if [[ "$kind" == "device" ]]; then
  : "${CODESIGN_IDENTITY:?set CODESIGN_IDENTITY to sign for a device}"
  codesign --force --sign "$CODESIGN_IDENTITY" "$app"
  echo "Built and signed: $app"
  echo "Install with Xcode (Window > Devices and Simulators) or:"
  echo "  xcrun devicectl device install app --device <udid> \"$app\""
  exit 0
fi

# Simulator: boot the first available iPhone if none is running.
booted="$(xcrun simctl list devices booted -j | python3 -c 'import json,sys; d=json.load(sys.stdin)["devices"]; print(next((x["udid"] for v in d.values() for x in v), ""))')"
if [[ -z "$booted" ]]; then
  booted="$(xcrun simctl list devices available -j | python3 -c 'import json,sys; d=json.load(sys.stdin)["devices"]; print(next(x["udid"] for v in d.values() for x in v if "iPhone" in x["name"]))')"
  xcrun simctl boot "$booted"
fi
open -a Simulator
xcrun simctl install "$booted" "$app"
xcrun simctl launch "$booted" com.brucedombrowski.ames-window
echo "Launched on simulator $booted"
