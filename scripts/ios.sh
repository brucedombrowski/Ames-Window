#!/usr/bin/env bash
# Build the pure-Rust binary for iOS and wrap it in an .app bundle.
#
#   scripts/ios.sh sim            # build for the simulator and launch it
#   scripts/ios.sh device         # build, sign and install on a plugged-in iPhone
#
# Requires Xcode and the matching Rust targets:
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#
# A device build needs a development provisioning profile for the bundle id
# below. Xcode mints one the first time it runs *any* project with that bundle
# id and your team on your phone; after that this script finds it on its own.
# CODESIGN_IDENTITY may be set to pick a certificate; otherwise the first
# "Apple Development" identity in the keychain is used.
set -euo pipefail

bundle_id="com.brucedombrowski.ames-window"
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

if [[ "$kind" == "sim" ]]; then
  booted="$(xcrun simctl list devices booted -j | python3 -c 'import json,sys; d=json.load(sys.stdin)["devices"]; print(next((x["udid"] for v in d.values() for x in v), ""))')"
  if [[ -z "$booted" ]]; then
    booted="$(xcrun simctl list devices available -j | python3 -c 'import json,sys; d=json.load(sys.stdin)["devices"]; print(next(x["udid"] for v in d.values() for x in v if "iPhone" in x["name"]))')"
    xcrun simctl boot "$booted"
  fi
  # Xcode 27 does not install Simulator.app where `open -a` can find it, and
  # install, launch and screenshot all work headless, so this is not fatal.
  open -a Simulator 2>/dev/null || echo "note: Simulator.app not found; running headless"
  xcrun simctl install "$booted" "$app"
  xcrun simctl launch "$booted" "$bundle_id"
  echo "Launched on simulator $booted"
  exit 0
fi

# Device: find a provisioning profile that covers the bundle id.
profile="$(python3 - "$bundle_id" <<'PY'
import glob, os, plistlib, re, subprocess, sys
from datetime import datetime, timezone
bundle = sys.argv[1]
dirs = ["~/Library/MobileDevice/Provisioning Profiles",
        "~/Library/Developer/Xcode/UserData/Provisioning Profiles"]
best = None
for d in dirs:
    for path in glob.glob(os.path.expanduser(d) + "/*.mobileprovision"):
        raw = subprocess.run(["security", "cms", "-D", "-i", path],
                             capture_output=True).stdout
        try:
            p = plistlib.loads(raw)
        except Exception:
            continue
        app_id = p.get("Entitlements", {}).get("application-identifier", "")
        pattern = re.escape(app_id.split(".", 1)[-1]).replace(r"\*", ".*")
        expires = p.get("ExpirationDate")
        if expires and expires.replace(tzinfo=timezone.utc) < datetime.now(timezone.utc):
            continue
        if re.fullmatch(pattern, bundle):
            if best is None or (expires and expires > best[1]):
                best = (path, expires)
print(best[0] if best else "")
PY
)"
if [[ -z "$profile" ]]; then
  cat >&2 <<MSG
No valid provisioning profile covers $bundle_id.
Make one once: in Xcode create any iOS app with bundle id $bundle_id,
pick your team under Signing & Capabilities, and run it on the phone.
Then run this script again.
MSG
  exit 1
fi
cp "$profile" "$app/embedded.mobileprovision"

entitlements="target/ios/$kind/entitlements.plist"
security cms -D -i "$profile" | /usr/libexec/PlistBuddy -x -c 'Print :Entitlements' /dev/stdin > "$entitlements"

identity="${CODESIGN_IDENTITY:-$(security find-identity -v -p codesigning | grep -m1 -o '"Apple Development[^"]*"' | tr -d '"')}"
: "${identity:?no Apple Development identity found; sign in to Xcode with your Apple ID}"
codesign --force --sign "$identity" --entitlements "$entitlements" "$app"
echo "Signed with: $identity"
echo "Profile:     $profile"

udid="$(xcrun devicectl list devices -j /dev/stdout 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); print(next((x["identifier"] for x in d["result"]["devices"] if x.get("connectionProperties",{}).get("tunnelState")!="unavailable"), ""))' || true)"
if [[ -z "$udid" ]]; then
  echo "Built and signed: $app"
  echo "No connected iPhone found; install with Xcode (Window > Devices and Simulators) or devicectl."
  exit 0
fi
xcrun devicectl device install app --device "$udid" "$app"
xcrun devicectl device process launch --device "$udid" "$bundle_id"
echo "Launched on device $udid"
