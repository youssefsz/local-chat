# Packaging and distribution

Production uses the bundle ID `app.localchat.desktop`. Development and QA use distinct IDs and data directories.
Only macOS ARM64, Windows x64, and Linux x64 are published. See [platform status](platform-status.md) for physical validation limits.

| Build | macOS | Windows | Linux |
| --- | --- | --- | --- |
| Local or ordinary CI | Ad-hoc signing; no notarization | Unsigned NSIS | AppImage and Debian |
| Tagged release | Developer ID, notarized app and DMG | Unsigned NSIS with updater signature | Signed updater AppImage and Debian package |

Updater signatures are separate from OS code signing. They bind each update package to its announced version.
The publisher verifies every platform manifest, uploads a draft, and publishes only after all packages and checksums exist.
It advances update feeds after publication. See [release operations](releases.md).

## Verify a Mac download

Compare the downloaded file with its release's `SHA256SUMS`. Mount the DMG read-only, then run:

```sh
python3 tools/verify_macos_bundle.py "PATH_TO_MOUNTED_APP"
xcrun stapler validate "PATH_TO_MOUNTED_APP"
spctl --assess --type execute --verbose=4 "PATH_TO_MOUNTED_APP"
```

The verifier checks code signing, bundle ID, native declarations, executable inventory, and exclusion of test-only markers.
Release staging separately verifies the archived updater app and the DMG's notarization ticket.
Published beta.6 passed these checks on CI and the available Mac. Saved history and drafts survived installation.

The production app contains one executable. Peer, fixture, benchmark, and signature-verifier examples stay outside the bundle.
Never distribute a build made with `test-profile`, and never upload signing material or chat databases as artifacts.

## macOS toolchain workaround

The pinned Rust toolchain failed to load stripped proc-macro dylibs on macOS 27.
Release builds retain debug information and avoid stripping build dependencies while keeping LTO optimization.
See the [upstream issue](https://github.com/rust-lang/rust/issues/157750).
Remove the workaround only after testing a toolchain with the fix.
