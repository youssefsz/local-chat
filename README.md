# Local Chat

[![CI](https://github.com/youssefsz/local-chat/actions/workflows/ci.yml/badge.svg)](https://github.com/youssefsz/local-chat/actions/workflows/ci.yml)


Local Chat shares text, links and code between nearby computers without an account or a separate server. Rooms are public to the current local network. History and search stay on each computer.

Download [Local Chat 0.1.0-beta.6](https://github.com/youssefsz/local-chat/releases/tag/v0.1.0-beta.6) for Apple Silicon macOS, Windows x64, or Linux x64. The Mac app and DMG are Developer ID signed and notarized. Windows is intentionally unsigned and may show a SmartScreen warning. Linux downloads are AppImage and Debian packages.

The approved interface is preserved in `docs/reference/`. The application uses Tauri 2, Svelte 5, strict TypeScript and Rust, with SQLite/FTS5 and direct pinned-TLS peer sessions. macOS is the first locally verified platform. CI targets Apple Silicon macOS, Windows x64, and Linux x64. Windows/Linux networking adapters are implemented; their build and physical-device verification status is recorded separately in [platform status](docs/platform-status.md).

## Development and releases

Use `npm run dev:app` for the isolated development copy. Pull requests and main pushes run the three-platform CI; manual CI runs provide test installers without publishing a release. Tags such as `v0.1.0-beta.1` build releases, with signed/notarized macOS and intentionally unsigned Windows. See [the full development, versioning, and signing guide](docs/releases.md).

## Run and build

Prerequisites: Node.js 24 LTS with npm, Rust 1.93 or a compatible newer stable toolchain, and the platform prerequisites for Tauri 2. On macOS install Xcode command-line tools. Lockfiles pin the resolved dependencies.

```sh
npm ci
npm run check
npm test
npm run lint
npm run dev:app
npm run tauri build
```

The build creates `src-tauri/target/release/bundle/macos/Local Chat.app` and `src-tauri/target/release/bundle/dmg/Local Chat_<version>_aarch64.dmg` on the available Apple Silicon Mac. Open the .app from Finder or copy it to Applications. Allow Local Network access when asked. Open Local Chat on the other computers on the same Wi-Fi or Ethernet network. General is available immediately. The app needs no internet after installation.

Choose Reconnect below the sidebar connection status or inside the People nearby panel to check for missing messages while keeping your draft and reading position. Local Chat also retries interrupted connections automatically.

Closing the window keeps reception running. Reopen from Finder, the Dock or the menu-bar icon. Choose **Quit Local Chat** to stop networking. Launch at login is separate and is not enabled automatically.

## Isolated peer tests

The test-profile feature is absent from the distributed build. Test identities use separate private files and databases, with independent listening ports. Production uses the OS credential store and normal single-instance behavior.

```sh
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --features test-profile --example local-chat-peer
python3 tools/peer_integration.py --counts 2 5 20
cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --example context_probe
cargo run --manifest-path src-tauri/Cargo.toml --release --no-default-features --features test-profile --example local-chat-bench -- .test-data/new-benchmark
```

Use a fresh benchmark output directory. The synthetic benchmark database is never bundled. Process tests use a fresh test-only discovery service and exercise actual mDNS on the selected interface and pinned TLS but do not prove communication between physical computers across a router.

## Keyboard and content

- Cmd/Ctrl+K opens global search. Room search is in the header. Arrow keys move through results; Enter opens one; Escape closes overlays.
- Text mode: Enter sends, Shift+Enter inserts a newline. Code mode: Enter inserts a newline, Cmd/Ctrl+Enter sends. IME composition never submits.
- Alt+1 through Alt+9 selects visible rooms. Tab reaches copy controls. Code copying preserves exact spaces, tabs and line endings.
- System is the fresh-install appearance. Light, Dark and System are persisted independently of the active OS theme.

Only the installation that created a room can rename, archive or restore it. Duplicate labels are allowed; room identities remain distinct. General cannot be renamed or archived. Archived and previous-network history remain searchable.

## Data and trust

On macOS the database is in the application support directory for `app.localchat.desktop`. Installation signing and TLS keys are in the OS credential store. SQLite uses WAL and parameterized queries. Message bodies are not logged by default. No SQL API is exposed to the frontend.

Automatic discovery proves possession of an installation key, not a person's identity. Router and OS network evidence conservatively select which local history may be synchronized. Old-network history never becomes eligible merely because a familiar person appears. Uncertain continuity creates fresh communication and retains earlier history read-only. Deliberate network impersonation remains outside the guarantees of unauthenticated LAN discovery.

See [architecture](docs/architecture.md), [protocol](docs/protocol.md), [network contexts](docs/network-contexts.md), [testing](docs/testing.md), [performance](docs/performance.md), [platform status](docs/platform-status.md), and [packaging](docs/packaging.md) for the implemented contracts and actual evidence.

## Initial local handoff

The original prototype app and DMG remain in `artifacts/`. Those historical artifacts use Developer ID signing and are not notarized. The installed copy now uses the published, notarized beta.4. The default local build uses ad-hoc signing; tagged GitHub releases use Developer ID signing and notarization. See [packaging](docs/packaging.md) for verification details.

Automated checks pass, including actual Bonjour/TLS process groups of 2, 5 and 20. Final installed-app communication and exact clipboard/draft behavior were exercised on this Mac. Search p95 was 59.75 ms on 100,000 synthetic messages. With one connected peer, the final release averaged 0.066% CPU and 168.63 MiB additive RSS over 30.44 seconds. Launch samples ranged from 494 ms to 7.62 seconds, with the slow run dominated by Keychain lookup.

A second physical computer, router/VPN transitions and sleep/wake remain pending. Native notification banners and the remaining manual accessibility scenarios are not certified. Read the actual outcomes in [testing](docs/testing.md) and the separate platform checklists before treating this as a broadly verified distribution release.

## License

[MIT](LICENSE).


The installed version and Check for updates are in Settings; macOS also exposes Check for Updates in the app menu. Optional daily checks use GitHub. Downloads are verified and install only when you choose Restart to update. Local chatting works without internet. Beta.5 and earlier need one manual upgrade to beta.6 to enable this feature. Linux AppImage supports in-app updates; Debian users download a newer package.
