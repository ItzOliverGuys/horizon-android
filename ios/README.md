# Horizon on iOS

Unsigned IPA for **TrollStore**. No Apple account, certificate or provisioning profile is
needed: TrollStore fakesigns the app when it installs it, and the install does not expire.

## Build

On macOS with Xcode (the iOS SDK is required to link):

```sh
rustup target add aarch64-apple-ios
sh tools/ios_ipa.sh        # -> build/ios/Horizon.ipa
```

Or push to the `ios` branch and download the `Horizon-ipa` artifact from the
**iOS IPA** workflow; it runs on a macOS runner.

## Install

1. Open `Horizon.ipa` in TrollStore.
2. Put the prepared resources where the game finds them: connect the device, open
   Horizon in Finder or the Files app (app documents are shared), and copy the
   `.local` folder from a PC prepared install so that
   `Documents/.local/horizon-client/26.3/horizon-resources.json` exists.
   The game downloads nothing itself yet.
3. Launch Horizon.

The container is the game folder: `saves/`, `options.txt`, `logs/` and `.local/` live in
`Documents`. Copying a world into `Documents/saves/` works the same way.

## What this build is

The desktop engine compiled for iOS: Metal renderer, sound through CoreAudio, the server
thread and single player. It boots and plays, but input is still the desktop path — no
touch controls, no on-screen keyboard and no on-device resource installer yet, so it needs
a Bluetooth keyboard/mouse or a controller to be usable. Controllers (`gilrs`) and code
mods (wasmtime's JIT, which iOS forbids) are off; the clipboard stays inside the game.
