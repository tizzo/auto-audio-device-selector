# Audio Device Selector (Raycast)

A personal Raycast extension that drives the `audio-device-monitor` binary from this
repo so you can switch and inspect macOS audio devices without leaving Raycast.

This extension is **not published to the Raycast Store** — it is imported locally as a
developer extension.

## Prerequisites

- [Raycast](https://raycast.com)
- Node.js 22+ and npm
- The `audio-device-monitor` binary installed and on your `PATH`
  (it currently lives at `/usr/local/bin/audio-device-monitor`). Build/install it
  from the repo root:

  ```bash
  cargo build --release
  cp target/release/audio-device-monitor /usr/local/bin/
  ```

## Install (local developer extension)

```bash
cd raycast
npm install
npm run dev      # imports the extension into Raycast and starts hot-reload
```

`npm run dev` registers the extension in Raycast; the commands appear immediately.
Leave it running while iterating. Once you stop it, the commands remain installed.
Run `npm run build` to verify a production build.

## Commands

| Command | Mode | What it does |
| --- | --- | --- |
| **Apply Preferences** | View | Runs `apply-preferences` and shows what changed. |
| **Show Current Devices** | View | Shows the current default output/input devices. |
| **Switch Input Device** | View | Lists input devices; select one to set it as the default input. |
| **Switch Output Device** | View | Lists output devices; select one to set it as the default output. |
| **Audio Devices Menu Bar** | Menu bar | Menu-bar item showing the current input device with quick-switch submenus for input and output. |

### Enabling / disabling the menu bar item

Toggle the menu-bar item from its command settings in Raycast (find the *Audio Devices
Menu Bar* command and enable/disable it, or use *Configure Command*). This is Raycast's
native mechanism for menu-bar commands.

The menu-bar item refreshes on its own interval (every 10s), and the other commands
(*Apply Preferences*, *Switch Input/Output Device*, and the menu-bar quick-switch
submenus) also trigger an immediate background refresh so the displayed device stays in
sync right after a change. The menu also exposes a manual **Refresh** action showing when
the device list was last updated.

## Preferences

- **Binary Path** (extension): path to the `audio-device-monitor` binary. Defaults to
  `audio-device-monitor` (resolved from `PATH`); set an absolute path if Raycast can't
  find it.

## How it works

Each command shells out to the binary with the `--json` flag and parses the structured
output. Because Raycast runs with a minimal `PATH`, the extension prepends the common
Homebrew/`/usr/local/bin` locations before spawning the binary.
