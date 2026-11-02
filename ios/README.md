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
**iOS IPA** workflow; it runs on a macOS runner, so no Mac is needed.

## Install and first launch

1. Open `Horizon.ipa` in TrollStore.
2. Launch Horizon. The first launch downloads and prepares the vanilla resources on
   the device (~500 MiB, several minutes, blank window — progress is in `logs/latest.log`),
   then quits by itself.
3. Launch again to play.

The app container is the game folder: `saves/`, `options.txt`, `logs/` and `.local/` live in
`Documents`, shown in the Files app.

## Controls

The engine's touch layer is on: the joystick and the buttons are the same ones
the Android build draws.

| Input | Action |
|---|---|
| Joystick (lower left) | Walk; push past the edge to sprint |
| Jump / sneak / sprint buttons | As labelled |
| Drag anywhere else | Look around |
| Tap or hold | Mine (hold) or use the held item |
| Two-finger tap | Right-click: place a block, use an item |
| Tap the hotbar | Select that slot |
| Inventory / pause buttons | Open the inventory or the pause menu |
| Tap in a menu | Click |

Chat, options, perspective and debug buttons open their screens. Text fields need a
Bluetooth keyboard: there is no on-screen keyboard yet.

## Memory ceiling

`memory-limit.txt` in the game folder (Files app → On My iPhone → Horizon), one number of MiB:

```
2048
```

Past it an allocation fails and the game aborts, the way `-Xmx` runs out in the JVM. Without
the file there is no ceiling. `HORIZON_MEMORY_LIMIT_MB` overrides it.

## What this build is

The desktop engine compiled for iOS: Metal renderer, sound through CoreAudio, single player
and multiplayer. Controllers (`gilrs`) and code mods (wasmtime's JIT, which iOS forbids) are
off; the clipboard stays inside the game.
