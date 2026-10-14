# Platform status

## macOS

Available hardware: MacBook Air, Apple M5, 24 GB memory, arm64. macOS 27.0 build 26A428. Xcode command-line toolchain available. Node 24.13.0, npm 11.6.2, rustc/cargo 1.93.0. Actual outcomes are in testing.md and performance.md.

macOS has a SystemConfiguration adapter for confirmed physical gateway evidence and native network-change generations. Discovery uses the native Bonjour DNSService APIs. TCP sockets use IP_BOUND_IF, discovery is restricted to the selected interface, and default routes through VPN/virtual interfaces fail closed. Private installation identity uses Keychain through keyring. No location access is requested.

Pending physical verification includes a second computer, multiple routers with matching SSIDs/address ranges, Wi-Fi/Ethernet transitions, real VPN activation, real sleep/wake. Simulated policy and isolated-process checks do not replace these.

## Windows: not yet verified

- [ ] Implement and test the network-evidence, network-change and physical socket adapters without weakening context separation.
- [ ] Build on Windows and verify WebView2 packaging on a clean system.
- [ ] Verify incoming-connection firewall prompts and Bonjour/mDNS behavior.
- [ ] Verify secure key storage and retained installation identity across upgrades.
- [ ] Exercise menus, clipboard, notifications, tray, close/reopen and Quit.
- [ ] Install the generated package and exchange messages with macOS.
- [ ] Repeat rooms, catch-up, context transitions, keyboard and resource tests.

## Linux: not yet verified

Initial candidates are Ubuntu 24.04 LTS and Fedora Workstation. They are targets for future validation, not a current support claim.

- [ ] Implement and test network evidence, change signals and physical socket binding.
- [ ] Build with the required WebKitGTK, GTK and packaging dependencies.
- [ ] Test discovery and firewall behavior on both target distributions.
- [ ] Verify Secret Service availability and fail safely when key storage is unavailable.
- [ ] Verify notifications, clipboard and native menus under supported sessions.
- [ ] Test tray availability and provide a reliable reopening path where no tray is exposed.
- [ ] Install packages and exchange messages with macOS and Windows.
- [ ] Repeat context, catch-up, keyboard and performance checks under Wayland and X11 where supported.

The UI, SQLite schema, signed event protocol and transport code are shared. Platform work stays inside explicit adapters; Windows/Linux are not represented as working before those adapters and tests exist.
