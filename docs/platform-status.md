# Platform status

## macOS

Available hardware: MacBook Air, Apple M5, 24 GB memory, arm64. macOS 27.0 build 26A428. Xcode command-line toolchain available. Node 24.13.0, npm 11.6.2, rustc/cargo 1.93.0. Actual outcomes are in testing.md and performance.md.

macOS has a SystemConfiguration adapter for confirmed physical gateway evidence and native network-change generations. Discovery uses the native Bonjour DNSService APIs. TCP sockets use IP_BOUND_IF, discovery is restricted to the selected interface, and default routes through VPN/virtual interfaces fail closed. Private installation identity uses Keychain through keyring. No location access is requested.

Pending physical verification includes a second computer, multiple routers with matching SSIDs/address ranges, Wi-Fi/Ethernet transitions, real VPN activation, real sleep/wake. Simulated policy and isolated-process checks do not replace these.

## Windows x64: native CI and physical verification

- [x] Implement platform adapters: IP Helper hardware checks and notifications, default-route/gateway evidence, scoped sockets, and IPv4 DNS-SD. OS observer and conservative-evidence tests run on Windows CI.
- [x] Native Windows x64 CI passes formatting, Clippy, 17 backend/adapter tests, production compilation, NSIS packaging, architecture/test-control checks and unsigned Authenticode verification.
- [ ] Install the Windows package and verify WebView2 on a clean desktop.
- [ ] Verify incoming-connection firewall prompts and Bonjour/mDNS behavior.
- [ ] Verify secure key storage and retained installation identity across upgrades.
- [ ] Exercise menus, clipboard, notifications, tray, close/reopen and Quit.
- [ ] Install the generated package and exchange messages with macOS.
- [ ] Repeat rooms, catch-up, context transitions, keyboard and resource tests.

## Linux x64: native CI and physical verification

CI builds on Ubuntu 22.04 x64 to produce AppImage and Debian packages. Ubuntu 24.04 LTS is the first physical-desktop validation target; Fedora AppImage validation remains pending. ARM Linux is outside the supported build matrix.

- [x] Implement platform adapters: sysfs physical-device validation, default-route/gateway evidence, RTNETLINK generation updates, scoped sockets, and IPv4 DNS-SD. Native CI exercises observer initialization, evidence rejection and unprivileged socket binding.
- [x] Native Ubuntu x64 CI passes formatting, Clippy, 18 backend/adapter tests, production compilation, AppImage/Debian packaging and architecture/test-control checks.
- [ ] Test discovery and firewall behavior on both target distributions.
- [ ] Verify Secret Service availability and fail safely when key storage is unavailable.
- [ ] Verify notifications, clipboard and native menus under supported sessions.
- [ ] Test tray availability and provide a reliable reopening path where no tray is exposed.
- [ ] Install packages and exchange messages with macOS and Windows.
- [ ] Repeat context, catch-up, keyboard and performance checks under Wayland and X11 where supported.

The UI, SQLite schema, signed event protocol and transport code are shared. Platform code stays inside explicit adapters. Native CI builds/tests and real desktop/LAN verification are distinct; pending checks above are not certified by producing an installer. Windows signing is intentionally disabled. Only macOS ARM64, Windows x64, and Linux x64 are built.
