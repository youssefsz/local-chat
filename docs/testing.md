# Verification record

Tests ran on 14–15 October 2026 on one Apple M5 MacBook Air with 24 GB RAM, macOS 27.0 build 26A428. The distributed target is arm64. The UI was exercised as a real packaged Tauri application, including an installed copy opened from Finder. Browser prototype inspection was used for visual comparison only.

## Automated checks

| Check | Actual outcome |
| --- | --- |
| Strict TypeScript and Svelte diagnostics | Pass, zero errors or warnings |
| Rust Clippy with warnings denied | Pass |
| Rust domain, storage and transport suite | Pass, 12 tests |
| Actual native Bonjour processes, 2, 5 and 20 isolated peers | Pass, automatic discovery, pinned TLS, concurrent sends and retained-database restart |
| Interrupted history synchronization | Pass, 600 signed events across multiple inventory pages, then relay without original sender |
| Event validity | Pass, signature tampering, creator ownership, duplicate delivery, gaps, out-of-order events and wrong clocks |
| Context isolation | Pass, previous-context exports and pending messages remain ineligible; duplicate membership cannot reassign old events |
| Limits | Pass, oversized prefix rejected before allocation; eight maximally escaped 64 KiB messages split into bounded frames |
| Local search benchmark | Pass against initial storage target, 100,000 synthetic messages; see performance.md |

Raw outputs are in `docs/evidence/`. The process harness creates a unique test-only discovery service for each run. It uses separate identities, ports and databases, exercises SIGKILL/restart and room changes, and asserts convergence with no duplicate messages. These are real processes on one physical interface, not a claim about router traversal.

Reproduce:

```sh
npm ci
npm run check
npm test
npm run lint
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --features test-profile --example local-chat-peer
python3 tools/peer_integration.py --counts 2 5 20
npm run tauri build
```

## Native macOS checks

| Scenario | Actual outcome |
| --- | --- |
| Open packaged app from Finder | Pass, native window, menus and application icon |
| Install to user's Applications directory | Pass; retained data kept across replacements |
| Native discovery and bidirectional messages | Pass, installed production app and a separate isolated peer using the production service on the physical interface |
| Durable receipt labels | Pass, native outgoing message stored by peer and shown as Shared with connected peers |
| Local Network denial and recovery | Pass, native Bonjour reported denied access; restoring Local Chat's access in System Settings recovered the connection without deleting history |
| Create, rename, archive and restore a room | Pass in native UI; archived history remained readable and restore returned composer |
| General and creator restrictions | Enforced in Rust and transport tests; General's management controls absent |
| Text and code keyboard behavior | Pass, text Enter sends; code Enter adds newline and Cmd+Enter sends |
| Explicit HTTPS link | Pass in final installed release, opened expected Rust book URL in Edge |
| Exact native clipboard copy | Pass, spaces, tab and trailing newline copied, pasted into native composer and compared byte-for-byte with persisted draft |
| Draft room switching | Pass, exact draft restored after switching away and back |
| Draft restart persistence | Pass, dedicated native profile restored a multiline draft and its indentation after explicit Quit and relaunch |
| Search, result jump and Escape | Pass in native app; local search works without peer availability |
| System theme persistence and live OS changes | Pass, stored value remains system; macOS Dark to Light to Dark updated the running app, original OS appearance restored |
| Light and Dark layouts | Native 1280×720 and 800×600 captured; requested 1440×900 clamped by the available display to 1440×859 |
| Close to background | Pass, same process persisted one incoming message while its window was hidden |
| Reopen and single instance | Pass, Finder restored the same PID and background message |
| Explicit Quit | Pass, process exited and discovery/connections stopped |
| Native notification banner | Not visually confirmed; integration and mute/global preference code are present, background durable reception passed |
| Sleep and wake | Pending physical test |
| IME composition, reduced motion, 200% zoom | Implemented guards/styles; dedicated native interaction checks pending |
| Receive while reading old history | Pass, native view stayed in older history and showed New messages; activating it displayed the newly received event |
| Load-older anchor and selection during a burst | Implemented; dedicated native burst/selection repetition pending |

The dedicated `Local Chat Test.app` uses `app.localchat.desktop.qa`, isolated credentials and synthetic signed fixture content. It is used for window geometry and theme checks only and is excluded from the deliverables. Production has no test profile switch or control endpoint. Test helper programs are Cargo examples, so Tauri does not bundle them as app executables.

## Physical network checklist

One Mac is available. None of these pending checks is satisfied by running twenty local processes:

- [ ] Exchange messages between two physical computers across the intended router, with internet disconnected.
- [ ] Disconnect and reconnect to the same network, checking retained eligible history.
- [ ] Move between two routers and confirm old rooms and pending messages stay local.
- [ ] Repeat with the same SSID and address range on different routers.
- [ ] Move familiar devices together and verify recognition never follows peer identity alone.
- [ ] Exercise simultaneous Wi-Fi and Ethernet, plus route/interface changes.
- [ ] Activate and deactivate a real VPN; verify fail-closed behavior and physical binding.
- [ ] Sleep and wake with queued incoming messages.
- [ ] Close every peer and reopen; start separate groups and let them subsequently discover each other.
- [ ] Visually verify native notification banners, room mute, global disable and no catch-up flood.

Windows and Linux have separate pending checklists in platform-status.md. Their adapters currently fail closed. Neither platform is represented as working or tested.
