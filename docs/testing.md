# Testing

Tests are split between colocated Rust modules and standalone programs in `tests`.
`tools` contains the runner and maintenance utilities. Generated reports belong in ignored `artifacts/test-reports`, not source control.

## Run automated checks

```sh
npm run lint
npm run test:all
```

| Command | Coverage |
| --- | --- |
| `npm test` | Rust protocol, storage, transport, platform, and update-policy tests |
| `npm run test:contracts` | Versions, architecture rejection, Apple-authentication boundaries, update feeds, release resume, and release-note attribution |
| `npm run test:signatures` | Actual Tauri signatures: valid, wrong key, wrong version, and tampered package |
| `npm run test:bonjour` | macOS C adapter recovery under interrupted and fatal poll failures |
| `npm run test:all` | All of the above; Bonjour runs on macOS only |
| `npm run test:integration` | Isolated real Bonjour and TLS peers on macOS, including reconnect and catch-up |

The process harness requires a usable physical interface and is separate from deterministic CI checks.
It builds its test-only peer example and uses independent identities, temporary databases, and a unique discovery service.
Multiple processes on one machine do not prove communication between physical computers across a router.

Rust tests use `#[cfg(test)]` beside the implementation. Standalone locations and dependencies are listed in [tests/README.md](../tests/README.md).
`npm run check` checks frontend types and Svelte diagnostics. There is no automated frontend interaction suite yet.

## CI and published release verification

The [CI workflow](https://github.com/youssefsz/local-chat/actions/workflows/ci.yml) checks all three supported build targets.
Each run uploads installer artifacts for 14 days and data-free test summaries for 30 days, including failure results where produced.
Release runs repeat tests and verify actual architectures, absence of test controls, update signatures, and package staging.
Mac releases also verify notarized app, updater archive, and DMG tickets and Gatekeeper acceptance.

[Beta.6 CI](https://github.com/youssefsz/local-chat/actions/runs/38738130533) and its [release workflow](https://github.com/youssefsz/local-chat/actions/runs/38738135486) passed on commit `35e29b0`.
The native Rust suites passed 17 tests on macOS ARM64, 19 on Windows x64, and 20 on Linux x64.
The Mac download matched checksums and GitHub digests, passed local Gatekeeper assessment, and was installed while retaining saved history and drafts.
The installed app reached the live beta feed and reported the current version.

Historical raw logs and repetitive captures were removed from the current tree during repository maintenance.
They remain in Git history and in the maintainer's private local archive. Public summaries retain the scope and limitations of the checks.

## Native UI checks

Build a dedicated synthetic profile:

```sh
npm run tauri build -- --ci --debug --features test-profile --config src-tauri/tauri.test.conf.json --bundles app
```

Its bundle ID is `app.localchat.desktop.qa`, with isolated storage and credentials. Production must never use `test-profile`.
The fixture, peer, and benchmark examples are not packaged with the application.

Previously exercised on the available Mac:

- Light, Dark, and System at the 800×600 minimum and 1280×720.
- Exact multiline drafts, tabs, spaces, code copying, local search, room management, and background reception.
- Reconnect from the sidebar and connection panel with retained drafts and reading position.
- A 12-second injected startup delay, retry after initialization failure, close/reopen, and Quit during loading.
- Update progress, current version, unavailable server, invalid signature, native menu, and retained unsaved Settings fields.
- A verified downloaded update restored after Quit while its server was unavailable, then installed and restarted with exact saved data retained.

For startup tests, place `startup.json` with `{"delay_ms":12000,"fail_once":true}` in the QA profile directory.
Restore zero delay and `fail_once:false` afterward. For window sizing, use QA-only `window.json` with `width` and `height`.

For updates, build a candidate and an older version override of the same QA app. Run:

```sh
python3 tests/integration/update_fixture.py --app PATH_TO_QA_CANDIDATE --folder .test-data/update-server --data QA_PROFILE_DIRECTORY
```

The fixture refuses production bundle IDs. Generated `mode.json` supports current, available, unavailable, bad-signature, and wrong-version.
The base fixture version is beta.5; adjust the fixture's base version when testing a different upgrade range.
Remove generated `update-test.json` and stop the server after checking. Endpoint overrides and insecure loopback transport never compile into production.

## Repository-maintenance verification

After reorganizing the tests, local `test:all` passed 17 Rust tests, 13 Python contracts, four signature scenarios, and five Bonjour fault scenarios.
The moved integration harness also passed real process groups of 2, 5, and 20, plus seven reconnect resets on three isolated peers.
Svelte diagnostics, Clippy, workflow syntax, local documentation links, and public-tree checks passed.
The published beta.6 history produced a release-note preview with verified attribution and the correct beta.5 comparison base.

## Public 0.0.1 and website verification

The [0.0.1 release workflow](https://github.com/youssefsz/local-chat/actions/runs/38783989701) passed all three platforms and published ten assets at source commit `cf568c5`. Initial Mac ticket lookups lost their Apple network connection; a rerun passed without changing the tag or removing validation. The [subsequent main CI](https://github.com/youssefsz/local-chat/actions/runs/38788077101) also passed all three targets.

Downloaded Mac files matched `SHA256SUMS`. The actual updater verifier accepted the package signature and authenticated version. The app and DMG passed local stapled-ticket checks; the app passed strict code-signature verification and Gatekeeper as Notarized Developer ID. The verified bundle replaced the installed beta while leaving its app data directory in place. Production Settings showed 0.0.1, a manual Stable check reported up to date, and its Privacy link opened the public policy in the system browser. This was a manual beta-to-stable installation, not a new automated updater-installation test.

Website commands:

```sh
npm --prefix website ci
npm --prefix website run build
npm --prefix website test
```

Five Node tests pass for page metadata/local links, complete stable-release validation, metadata caching/error behavior, allowed requests, and download redirects/fallbacks. The Workers dry run passed. The live six content routes returned 200, `.html` and trailing-slash privacy URLs redirected to `/privacy`, and missing routes returned 404. The public release endpoint returned 0.0.1; all four download endpoints redirected to the exact published installers.

A merged website change automatically triggered Cloudflare Workers Builds on `main`, deployed successfully, and changed the production content. Its watch path is `website/**`; website changes do not create an app release.

Manual browser checks covered desktop 1440px, tablet 768px, mobile 390px, and the 1280px working viewport. The revised centered hero has no horizontal overflow, retains the complete native screenshot on mobile, and uses an eager high-priority image. Light/Dark selection updates the image, pressed state, and full-size destination. The hero Download action reaches the platform download page. Light and Dark screenshot assets were recaptured from a fresh isolated preview window with decorations disabled for capture only; both 2560×1800 images were visually checked for cursor and computer-use indicator artifacts. This is manual browser evidence, not an automated frontend interaction suite. Existing platform and physical-LAN limitations below still apply.

## Pending physical and interaction checks

- Two physical computers across the intended router with internet disconnected.
- Same-network reconnect, two routers with matching SSIDs or address ranges, and familiar peers moving together.
- Wi-Fi and Ethernet transitions, real VPN activation, sleep/wake, and queued incoming messages.
- Native notification banners and mute behavior without catch-up floods.
- Dedicated IME, reduced-motion, 200% zoom, burst selection, and older-page anchor interaction checks.
- Windows and Linux installation, firewall prompts, key storage, desktop integration, LAN exchange, and updater restart.

See [platform status](platform-status.md). A green build or simulated policy test does not complete these checks.
