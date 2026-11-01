#!/bin/sh
# Builds Horizon.app and Horizon.ipa for TrollStore. Needs macOS with Xcode (the iOS
# SDK); nothing is signed, TrollStore fakesigns the IPA when it installs it.
#
#   sh tools/ios_ipa.sh
#
# Controllers (gilrs) and code mods (wasmtime's JIT) are off on iOS: the first has no
# backend here, the second cannot allocate executable memory. Sound is on (CoreAudio).
set -eu

target=aarch64-apple-ios
out=build/ios
name=Horizon

cargo build --profile fast --no-default-features --features audio \
    -p horizon-client --target "$target"

rm -rf "$out/$name.app"
mkdir -p "$out/$name.app"
cp "target/$target/fast/horizon-client" "$out/$name.app/horizon-client"
cp ios/Info.plist "$out/$name.app/Info.plist"
cp crates/horizon-theme/icons/horizon-64.png "$out/$name.app/Icon.png"
printf 'APPL????' > "$out/$name.app/PkgInfo"
# The game binary and its shaders are the only things in the bundle; the ~500 MiB of
# Minecraft resources are copied into Documents, not shipped.
strip -x "$out/$name.app/horizon-client" 2>/dev/null || true

rm -f "$out/$name.ipa"
(cd "$out" && zip -qry "$name.ipa" "$name.app")
echo "$out/$name.ipa"
