# OpenDesk

[![CI](https://github.com/erenkan/opendesk/actions/workflows/ci.yml/badge.svg)](https://github.com/erenkan/opendesk/actions/workflows/ci.yml)

Cross-platform menubar/tray app to control Linak DPG1C standing desks over
Bluetooth Low Energy. Built with Tauri v2 (Rust) + React/TypeScript.

- Tray-only app (no Dock icon on macOS, no taskbar entry on Linux)
- Live height readout + hold-to-move arrows + user presets
- Sit-stand reminder with native desktop notifications
- Backend-driven BLE session with auto-reconnect watchdog

## Install

Grab the latest build for your platform from
[Releases](https://github.com/erenkan/opendesk/releases). macOS and Linux
are the supported targets.

### macOS

`.dmg` ships separate bundles for Apple Silicon and Intel. Builds are
Developer ID-signed and notarized — drag to `/Applications` and launch
normally. App auto-updates from GitHub Releases (Settings → Check for
updates).

### Linux

Download the `.AppImage` (portable) or `.deb` (Debian / Ubuntu). GNOME
needs `gnome-shell-extension-appindicator` for the tray icon to appear.
KDE is fine out of the box.

## Compatible desks

OpenDesk's BLE layer is written around the Linak DPG1C protocol. Controllers
sharing the same chipset / command set (IKEA Idasen is a well-known example)
work out of the box. Other brands need a dedicated protocol module — see
[Adding a new desk](#adding-a-new-desk).

| Brand / model       | Status       | Notes                                              |
|---------------------|--------------|----------------------------------------------------|
| Linak DPG1C         | ✅ verified  | Reference implementation                           |
| IKEA Idasen         | ✅ verified  | Linak-compatible firmware                          |
| Other Linak OEMs    | 🧪 likely    | Worth trying; open an issue if name matching fails |
| JIECANG / Flexispot | ❌ not yet   | Needs protocol module — PRs welcome                |
| Uplift / Desky      | ❌ not yet   | Needs protocol module — PRs welcome                |

`✅ verified` = tested on real hardware by a maintainer.
`🧪 likely` = same chipset family, should work; unconfirmed.
`❌ not yet` = different BLE protocol, no module exists.

## Adding a new desk

OpenDesk welcomes per-brand PRs — `ble/linak.rs` is the reference you copy.
Start with a [New desk brand](.github/ISSUE_TEMPLATE/new-desk.yml) issue to
share the protocol research first (service UUID, command bytes, position
encoding, handshake if any); then a PR touches:

- `src-tauri/src/ble/<brand>.rs` — new protocol module, mirror `linak.rs`'s shape
- `src-tauri/src/ble/mod.rs` — `pub mod <brand>;`
- `src-tauri/src/ble/manager.rs::find_desk` — add name / service-UUID matcher
- Unit tests in the new module: encode/decode round-trip, boundary clamping
- The compatibility table above — a row for your desk

For BLE sniffing tips and an annotated walkthrough of `linak.rs`, see the
issue template and the module's own docs.

## Platform requirements

| Platform | Minimum |
|----------|---------|
| macOS    | 11 Big Sur |
| Linux    | BlueZ 5.50+, user in the `bluetooth` group |

Linux users on GNOME need the `gnome-shell-extension-appindicator` extension
for the tray icon to show up. KDE works out of the box.

## Development

Prerequisites: Rust (stable, 1.77+), Node 18+, pnpm, and the Tauri system
dependencies for your OS — see <https://v2.tauri.app/start/prerequisites/>.
macOS contributors editing the app icon additionally need Xcode (for
`actool`) and [Icon Composer](https://developer.apple.com/icon-composer/)
to regenerate `src-tauri/icons/AppIcon.icon`.

```bash
pnpm install
pnpm tauri dev        # hot-reload dev build
```

On macOS 26 Tahoe (Darwin 25+) the raw binary panics during launch (tao
#1171). Use the bundled `.app` for day-to-day testing:

```bash
pnpm app:rebuild      # kill running, rebuild, register, launch .app
pnpm app:logs         # tail backend log at ~/Library/Logs/app.opendesk.menubar/
```

## Scripts

| Script | What it does |
|--------|---------------|
| `pnpm dev` | Vite dev server only (no Tauri) |
| `pnpm build` | `tsc && vite build` — production web bundle |
| `pnpm tauri dev` | Tauri dev window + hot reload |
| `pnpm app:dev` | First-time build + register + launch `.app` (includes `mac:icon`) |
| `pnpm app:rebuild` | Kill, rebuild, re-register, relaunch `.app` (includes `mac:icon`) |
| `pnpm app:logs` | Tail the backend log file |
| `pnpm icons` | Re-render `icon.svg`/`tray-icon.svg` → PNGs → Tauri icon fan-out |
| `pnpm mac:icon` | Compile `AppIcon.icon` via `actool`, inject `Assets.car` into the debug bundle, patch `Info.plist` (Liquid Glass pipeline — macOS-only, no-op elsewhere) |
| `pnpm mac:icon:release` | Same for the release-profile bundle |

## Permissions

On first launch macOS will prompt for Bluetooth access. If you deny it and
change your mind, grant it again at **System Settings → Privacy &
Security → Bluetooth → OpenDesk**. Without permission, scans return an empty
list silently — OpenDesk surfaces a toast after 10 s if no peripherals
appear.

## Project layout

```
src/                     # React/TypeScript frontend
  components/popover/    # Popover UI (NSPanel on macOS, window elsewhere)
  hooks/                 # React hooks (useDesk, useAutoSession, ...)
  lib/                   # IPC wrapper, constants, presets
src-tauri/
  src/
    ble/                 # BLE state machine + Linak protocol
    commands.rs          # #[tauri::command] pass-throughs
    events.rs            # Event payloads (stays in sync with src/lib/desk.ts)
    notification.rs      # macOS UNUserNotificationCenter delivery (with icon attachment)
    panel.rs             # macOS NSPanel setup + tray-relative positioning
    reminder.rs          # Stand reminder tokio task
    state.rs             # AppState + reconnect watchdog
    tray.rs              # Tray icon + menu + click handler
  icons/                 # Master SVGs + generated PNG/ICNS/ICO
    AppIcon.icon/        # Icon Composer source for macOS 26 Liquid Glass
  capabilities/          # Tauri v2 capability manifests
scripts/
  render-icons.mjs       # SVG → PNG via @resvg/resvg-js
  mac-compile-icon.sh    # actool (`.icon` → Assets.car) + Info.plist patch
```

## Known issues

- **macOS sleep/wake**: CoreBluetooth disconnects silently on system sleep
  without firing `didDisconnectPeripheral`. The watchdog in
  `state.rs::run_reconnect_loop` polls `is_connected()` every 5 s and
  reconnects automatically when the link goes stale.
- **First-connect on macOS**: the initial scan may take up to 12 s if the
  desk controller is idle. Subsequent reconnects use a cached handle and
  complete in ~1 s.
- **macOS 26 Tahoe Liquid Glass icon** is built via a local workaround
  (`scripts/mac-compile-icon.sh` wraps `actool`) because Tauri ≤ 2.10.1
  can't bundle `.icon` files yet. Native support is merged upstream
  ([tauri-apps/tauri#14207](https://github.com/tauri-apps/tauri/issues/14207))
  but unreleased as of 2026-04. Once shipped, drop the script and add
  `icons/AppIcon.icon` to `tauri.conf.json`'s `bundle.icon` array.
- **macOS notifications**: `tauri-plugin-notification` uses the deprecated
  `NSUserNotification` API which drops the bundle icon on LSUIElement apps.
  We route reminders through our own `send_native_notification` command
  (`UNUserNotificationCenter` + `UNNotificationAttachment`) on macOS.
  Non-macOS keeps the plugin path.
- **Linux GNOME**: without the AppIndicator extension the tray icon is
  invisible. Install it separately.

## Contributing

Issues and pull requests welcome. This project is early — the architecture
is documented in [`src-tauri/src/ble/manager.rs`](src-tauri/src/ble/manager.rs)
(state machine + move-loop) and [`src/hooks/useDesk.ts`](src/hooks/useDesk.ts)
(the single IPC subscription point). Any BLE protocol changes go through
`src-tauri/src/ble/linak.rs`, which is pure and has unit tests
(`cargo test -p opendesk --lib`).

## License

MIT — see [`LICENSE`](LICENSE).
