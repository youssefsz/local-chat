# Packaging and distribution

Bundle identifier: `app.localchat.desktop`. Product name: Local Chat. Version: 0.1.0. The available build targets aarch64 macOS, minimum declared version 12.0. Intel macOS has not been built or exercised. A universal binary requires installing the x86_64 target and building `--target universal-apple-darwin`, followed by tests on both architectures.

The custom Info.plist includes NSLocalNetworkUsageDescription and NSBonjourServices for `_localchat._tcp`. Window controls, native menus and the template menu-bar icon use Tauri's OS integration. Distribution contains no test-profile feature or test control server.

The default reproducible configuration uses ad-hoc signing. The handoff artifact is signed with the available user-owned Developer ID Application certificate for Youssef Dhibi, team 5S4ZT4SRV9. It is locally tested and not notarized. No notarization credentials were available, and no upload was performed. These are separate statuses. No credentials are embedded in the repository.

For a distribution release, merge a local Tauri config containing `bundle.macOS.signingIdentity` set to the intended Developer ID Application identity, using `npm run tauri build -- --config path/to/signing.json`, and provide the Tauri-supported notarization environment in the build environment. Typical Apple ID notarization uses APPLE_ID, APPLE_PASSWORD and APPLE_TEAM_ID. Credentials must stay outside source control. Verify the resulting bundle using codesign and spctl, staple the notarization ticket, mount the DMG, install via Finder and repeat the desktop checklist. Do not upload a build or select another person's signing identity implicitly.

## macOS 27 toolchain note

The installed Rust 1.93 toolchain failed to load stripped proc-macro dylibs on macOS 27. Release builds keep debug information and disable stripping for build dependencies. This matches the upstream [Rust issue about misaligned stripped dylibs](https://github.com/rust-lang/rust/issues/157750). The release remains optimized with LTO. Remove the workaround only after testing a toolchain containing the upstream fix.

## Handoff artifact checks

The final `.app` and DMG are in `artifacts/`. The DMG was mounted read-only, its application signature verified, and its executable SHA-256 matched the standalone app. Both use hardened runtime signing with the intended Developer ID certificate. The installer is signed and not notarized. The installed copy in the user's Applications directory was opened through Finder and retained its existing SQLite history and installation identity.

`python3 tools/verify_macos_bundle.py "path/to/Local Chat.app"` verifies the signature, bundle identifier, Bonjour declaration, executable inventory and exclusion of test-only runtime markers. Recorded outputs are in `docs/evidence/macos-bundle.json` and `dmg-bundle.json`. The app contains one executable; the peer, fixture and benchmark programs are Cargo examples and never bundled. A DMG checksum is in `artifacts/SHA256SUMS`.
