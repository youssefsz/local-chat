# Development, CI, and releases

Supported build architectures are Apple Silicon macOS (`aarch64-apple-darwin`), Windows x64 (`x86_64-pc-windows-msvc`), and Linux x64 (`x86_64-unknown-linux-gnu`). No other architecture is published. Windows installers are intentionally unsigned. Linux packages are AppImage and Debian. A generated installer is not certification of physical LAN or desktop behavior on every system; platform-status.md records that separately.

## Local development

Install Node 24 LTS, Rust through rustup, and the [Tauri native prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system. The repository pins Rust 1.93.0 and both package lockfiles.

```sh
npm ci
npm run dev:app
```

This launches **Local Chat Dev** with live frontend updates, a distinct bundle identifier and an isolated data directory. It uses a file-backed test identity and the isolated `_localchat-test._tcp` discovery service, so it can run beside the installed production app. Run a second development copy on another computer to test development discovery. Production uses Keychain/Credential Manager/Secret Service, the production discovery service, and normal single-instance behavior. Never enable `test-profile` for a distributed build.

For an isolated packaged UI fixture use `src-tauri/tauri.test.conf.json`; this is separate from development and production. Development requires no distribution signing credentials.

## CI without a release

`CI` runs on pull requests and pushes to main. It is also available through GitHub Actions → CI → Run workflow. All three native runners check synchronized versions, Svelte/TypeScript, Rust formatting, Clippy, and backend tests, then build and inspect the production installers. Download the `release-*` workflow artifacts to test packaging. These temporary artifacts are retained for 14 days; the macOS CI package is ad-hoc signed and not notarized. Nothing is published to GitHub Releases by CI, and no release number is consumed.

All external actions are pinned to reviewed commit SHAs. Pull-request CI receives no signing secrets and has read-only repository permissions. Architecture and production test-control exclusion are verified against actual binaries. Windows Authenticode is checked to remain unsigned.

## Version and tag releases

Release helper scripts require Python 3.11 or later and the authenticated GitHub CLI. CI explicitly installs Python 3.13 in build, validation, and publication jobs.

Use `0.1.0-beta.1`, `0.1.0-beta.2`, then optionally `0.1.0-rc.1`. The first non-beta release is `0.1.0`. Pre-release versions do not consume `0.1.0`. Published tags and assets are never overwritten.

```sh
npm run version:app -- 0.1.0-beta.2
npm run release:check
# Commit the synchronized manifest and lockfile changes, then merge/push main.
git tag -a v0.1.0-beta.2 -m "Local Chat 0.1.0-beta.2"
git push origin v0.1.0-beta.2
```

`Release` checks that the tag matches all manifests and points to a commit on main. It rebuilds and tests every target. The macOS release must be Developer ID signed and notarized; missing credentials fail the release rather than silently producing an ad-hoc package. A final job stages a draft release, uploads all four installers and SHA256SUMS, then publishes. A failed upload leaves a draft. Beta/RC tags are prereleases and do not replace the latest regular release. A regular tag publishes a regular release.

Version tags are separate Git references, not commits pushed “into” main. Do not move or delete an already published tag to rerun it; fix the issue and use the next beta number. To retry an infrastructure failure before publication, rerun the failed workflow. If an interrupted publisher left an unpublished draft, rerunning resumes it only when its provenance marker matches the exact commit and its assets belong to this build. Published releases, drafts from other commits, and drafts with unexpected assets are never overwritten.

## Configure macOS signing once

The current signing identity is `Developer ID Application: Youssef Dhibi (5S4ZT4SRV9)`. A certificate on a developer's Mac is not automatically available to GitHub's temporary runners.

1. Open Keychain Access → My Certificates. Select this **Developer ID Application** identity, including its private key, and export only that identity as a password-protected `.p12` file outside the repository. Do not export every identity in the keychain.
2. At [account.apple.com](https://account.apple.com/), sign in and generate an app-specific password for Local Chat notarization. This is different from your normal Apple account password. Apple ID notarization uses the existing Developer Program team; it does not create an App Store listing.
3. From your local terminal, run:

   ```sh
   npm run signing:setup -- --certificate /absolute/path/LocalChat-DeveloperID.p12
   ```

   The helper prompts for the certificate export password, Apple ID email, and app-specific password. Password input is hidden. It sends six values directly to GitHub's encrypted repository Secrets using the authenticated `gh` CLI. It does not print credential values or write them into this repository.
4. Confirm that these six secret **names** appear under the repository's Settings → Secrets and variables → Actions: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID`, `APPLE_ID`, `APPLE_PASSWORD`.
5. Push the first beta tag after CI passes. Release CI imports the identity into a temporary keychain, signs/notarizes with Tauri, verifies the stapled app ticket and Gatekeeper assessment, then deletes the temporary keychain. GitHub runners are disposable. Keep the exported identity in your own secure backup.

Never paste credentials into an issue, chat, Git commit, release note, or workflow file. Never add certificate files to source control. The public-tree check blocks recognizable credential patterns and private file types, but does not replace careful review of what you publish.

Windows signing is intentionally not configured. It can be added later without changing the architecture matrix or versioning convention. Unsigned Windows downloads may show a SmartScreen warning. No automatic app updater is included; users download and install a later release.
