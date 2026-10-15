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

The dedicated `Local Chat Test.app` uses `app.localchat.desktop.qa`, isolated credentials and synthetic signed fixture content. It is used for window geometry, theme, and controlled startup delay/failure checks and is excluded from the deliverables. Production has no test profile switch or control endpoint. Test helper programs are Cargo examples, so Tauri does not bundle them as app executables.

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

Windows and Linux have separate pending physical-desktop checklists in platform-status.md. Their adapters now pass native CI tests and packaging checks. Physical LAN and desktop behavior on those systems remains unverified.

## Startup loading update, 15 October 2026

The packaged dedicated native build was exercised with a 12,000 ms initialization delay after reading the saved appearance. Light and Dark loading screens used the correct native title bar and content colors. Both completed into the real fixture conversation at the same 1280×720 size. Before/after native captures are 2560×1440 pixels. System loading and direct startup were also inspected at the 800×600 minimum size. Incomplete conversation controls were absent from the loading accessibility tree.

A controlled first-attempt failure replaced the spinner with an explanation, expandable details, and Try again. Tab reached the retry button; Return retried and restored the conversation. Closing during the injected delay followed by Finder reopen restored the initialized conversation. Native Quit during another delay exited before the engine existed. The test configuration was returned to zero delay with failure injection disabled. These controls compile only into `test-profile` builds.

The fast dedicated launch measured 356.75 ms to frontend-ready and skipped the loader. The final installed Developer ID signed release was opened from Finder and measured 467.32 ms to frontend-ready, also skipping the loader. Existing rooms and messages remained visible, and discovery connected to the isolated verification peer. No new production messages were sent during this check. The peer was then stopped. These are individual warm samples on this Mac, not latency percentiles or a high-frame-rate first-paint recording.

`npm test` passed all 12 existing backend tests. Strict Svelte/TypeScript checks and Clippy passed with zero warnings. The final app and DMG were rebuilt, the installed binary matched the mounted DMG, and signature verification passed. The bundle contained only the production executable and none of the isolated data, service, window, or startup test markers. Notarization was skipped because no notarization credentials were available. Raw results are in `docs/evidence/startup-*`; screenshots are in `docs/screenshots/startup-*`. Physical-network and other pending checks above remain pending.

To reproduce controlled startup checks, build the dedicated app with `npm run tauri build -- --ci --features test-profile --config src-tauri/tauri.test.conf.json --bundles app`. In its isolated `test-profile` data directory, create `startup.json` containing `{"delay_ms":12000,"fail_once":false}`. Use `fail_once:true` to exercise recovery, then restore `{"delay_ms":0,"fail_once":false}`. Production builds neither read this file nor expose these controls.

## Public repository and native CI, 15 October 2026

Public MIT repository: https://github.com/youssefsz/local-chat. Native CI run 38709993126 passed all three jobs on commit 47de9a3. macOS ARM64 passed 12 Rust tests and generated a DMG; Windows x64 passed 14 tests and generated an explicitly unsigned NSIS installer; Ubuntu x64 passed 15 tests and generated AppImage and Debian packages. Every job passed strict frontend diagnostics, Rust formatting and Clippy, version checks, real binary architecture checks, production test-control exclusion, and installer staging. See evidence/github-ci.json for job links and the distinction between ad-hoc CI builds and signed/notarized release builds.

The new OS-specific tests exercise network observer initialization, conservative physical/tunnel/ambiguous-address rejection, and Linux unprivileged interface binding. They do not certify physical cross-device discovery, Windows firewall prompts, Secret Service desktop unlocking, tray availability, VPN transitions, or installation on real Windows/Linux desktops. Those checks remain pending in platform-status.md.

The isolated development command launched the Vite server and native frontend with `app.localchat.desktop.dev` data. Its frontend-ready timing was 1,832 ms and its native show call was recorded at 1,845 ms. The directory contained no message events and was separate from production history. The development process/server were stopped after verification. This is a development startup sample, not a release-performance measurement. Local macOS Bonjour regression checks also passed with 2 and 5 separate peers, including ownership, duplicate names, interrupted process restart, and catch-up. Existing automated transport tests still cover groups of 2, 5 and 20.

The release contract suite checks version synchronization, valid/invalid beta/RC tag forms and rejection of the wrong binary architecture. Two authentication boundary tests confirm that rejected Apple credentials cannot replace GitHub secrets and that a verified login updates only the intended secrets. Cached installer output is removed before packaging, so a later beta version cannot accidentally upload an old installer. Release publishing requires all platform manifests, creates SHA256SUMS, stages a draft, and publishes only after all uploads succeed. A matching unpublished draft can be resumed after a failed upload; published releases are never overwritten. See releases.md for reproduction and secure credential setup.


## Published notarized beta, 15 October 2026

Release run 38709997690 on the same commit passed validation, all three installer jobs, and publication for `v0.1.0-beta.3`. The prerelease contains exactly the Mac ARM64 DMG, unsigned Windows x64 NSIS executable, Linux x64 AppImage and Debian package, plus SHA256SUMS. No stable version has been published. Earlier beta.1 and beta.2 tags remain setup history with failed workflows and no published releases.

Apple accepted the app notarization and the separately submitted signed DMG. Both tickets were stapled and validated. Both Gatekeeper assessments reported `accepted` with `source=Notarized Developer ID` on the runner and on the available Mac. The published download matched its SHA256SUMS entry and GitHub's asset digest. The mounted bundle retained the production bundle ID, ARM64 executable, and absence of test-only controls.

The notarized application replaced the installed prototype after private app and SQLite backups. A quarantine attribute was applied for normal first-open assessment. Finder launched it under App Translocation into the real conversation interface. Every existing row in events, rooms, and drafts matched the pre-upgrade backup. No messages were sent during this check. The app remains installed and open. Raw public release metadata, checksums, job links, and verification facts are recorded in `evidence/github-release-beta3.json`; the native screenshot and private backups remain ignored local files. Physical Windows/Linux and multi-computer checks remain pending.

## Reconnect update, 15 October 2026

Three regression cases failed before their fixes: the Bonjour worker did not report injected poll failures, a lagged UI change subscriber terminated, and old TLS-session cleanup removed a newly connected peer. The final Rust suite passes 15 tests on macOS, including event-burst recovery, bounded reconnect requests, session replacement, and existing protocol/context isolation checks. `python3 tools/test_bonjour_recovery.py` passes five cases through the actual C adapter and runs on macOS CI.

Build `local-chat-peer` with the test-profile feature, then run `python3 tools/reconnect_integration.py`. Three isolated real Bonjour/TLS processes passed seven manual resets, including concurrent resets, retained identity/context/rooms/drafts and offline catch-up without duplicate messages. This test uses a unique test-only discovery service and temporary private stores.

The dedicated packaged native test app was inspected in Light and System/Dark at 800x600 and 1280x720. Its labeled Reconnect buttons below the sidebar status and in the People nearby connection panel reconnected a real isolated peer. A multiline composer draft remained exact in SQLite; before/after screenshots retained the older-message reading position. A later peer message showed New messages while preserving the view. The sidebar status opens connection details; Reconnect is a labeled secondary button below the sidebar status and inside the panel with a standard Lucide refresh-cw vector. Test appearance was restored to System. Synthetic fixture screenshots and outcomes are in screenshots/reconnect-* and evidence/reconnect.json. Production data was not used for these UI checks. Physical sleep/router and Windows/Linux desktop checks remain pending.

## Software updates, 15 October 2026

The updater-enabled dedicated native QA build was tested with an independently signed loopback package using `python3 tools/update_fixture.py --app PATH_TO_QA_CANDIDATE --folder .test-data/updates/server --data QA_TEST_PROFILE_DIRECTORY`. Build the QA candidate with `npm run tauri build -- --ci --debug --features test-profile --config src-tauri/tauri.test.conf.json --bundles app`; build a second copy with an older version override as the installed base. Both are dedicated QA apps; this does not imply the published beta.5 already had an updater. The fixture refuses production bundle IDs and production builds exclude its endpoint/key override. Change the generated mode.json between current, available, unavailable, bad-signature and wrong-version. Stop the server and remove its generated update-test.json from the isolated profile afterward.

The native Mac checks passed: current version; recoverable server failure; rejected invalid signature without an install action; version and release notes; progress and verified Ready state; Later suppression; native app-menu check; Settings return with unsaved fields intact; and a staged package restored after quitting while the fixture server returned HTTP 503. Restart to update replaced the app, relaunched into beta.6, and retained the exact multiline draft including a tab and trailing spaces. The installed executable matched the candidate byte for byte. SQLite comparisons retained all 9 events, 4 rooms and 1 draft exactly. Production conversations were not used. System appearance and the 800×600 QA geometry were restored.

The native UI was checked at 800×600 in System/Dark and 1280×720 in Light. Captures are screenshots/update-current-dark-800.jpg and screenshots/update-available-light-1280.jpg. A stacked-dialog defect and disabled native menu item were found and fixed before the final installation check. No remaining visual defect was observed within these checked states.

`npm run lint` passed strict Svelte/TypeScript and Clippy diagnostics. `npm test` passed all 17 Mac backend tests. `python3 tools/test_update_signatures.py` exercised actual Tauri signing and the verifier used by staging: valid, wrong-key, wrong-version and tampered packages. The four update-feed/release-resume tests, four release contract tests and two Apple-authentication boundary tests passed. `actionlint`, version consistency and whitespace checks passed. Evidence is recorded in evidence/updates.json. Windows/Linux native build outcomes are recorded with the release once completed; physical installation/restart on those desktops remains pending.
