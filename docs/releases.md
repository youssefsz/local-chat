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

Use `0.1.0-beta.1`, `0.1.0-beta.2`, then optionally `0.1.0-rc.1`. The first non-beta release is `0.1.0`. Pre-release versions do not consume `0.1.0`. Published tags and assets are never overwritten. Prepare a version in a PR and merge it after CI passes.

```sh
npm run version:app -- 0.1.0-beta.7
npm run release:check
# Commit the synchronized manifest and lockfile changes, then merge/push main.
git tag -a v0.1.0-beta.7 -m "Local Chat 0.1.0-beta.7"
git push origin v0.1.0-beta.7
```

`Release` checks that the tag matches all manifests and points to a commit on main. It rebuilds and tests every target. The macOS release must be Developer ID signed and notarized; missing credentials fail the release rather than silently producing an ad-hoc package. A final job creates reviewed highlights plus categorized GitHub PR notes and verified contributor credits, stages a draft, uploads installers, updater packages, signatures, update.json and SHA256SUMS, then publishes. A failed upload leaves a draft. Beta/RC tags are prereleases and do not replace the latest regular release. A regular tag publishes a regular release.

Version tags are separate Git references, not commits pushed “into” main. Do not move or delete an already published tag to rerun it; fix the issue and use the next beta number. To retry an infrastructure failure before publication, rerun the failed workflow. If an interrupted publisher left an unpublished draft, rerunning resumes it only when its provenance marker matches the exact commit and its assets belong to this build. Published releases, drafts from other commits, and drafts with unexpected assets are never overwritten.

## Configure macOS signing once

Use the project's authorized Developer ID Application identity. A certificate on a developer's Mac is not automatically available to GitHub's temporary runners. Signing setup is maintainer-only.

1. Open Keychain Access → My Certificates. Select this **Developer ID Application** identity, including its private key, and export only that identity as a password-protected `.p12` file outside the repository. Do not export every identity in the keychain.
2. At [account.apple.com](https://account.apple.com/), sign in with your Apple Developer account. Open **Sign-In and Security → App-Specific Passwords → Generate an app-specific password**, and name it Local Chat notarization. Use the password Apple generates in the terminal prompt. Your Mac login password, normal Apple account password, and certificate export password do not work for notarization. Apple ID notarization uses the existing Developer Program team; it does not create an App Store listing.
3. From your local terminal, run:

   ```sh
   npm run signing:setup -- --certificate /absolute/path/LocalChat-DeveloperID.p12
   ```

   The helper prompts for the certificate export password, Apple ID email, and app-specific password. Password input is hidden. It sends six values directly to GitHub's encrypted repository Secrets using the authenticated `gh` CLI. It does not print credential values or write them into this repository.
4. Confirm that these six secret **names** appear under the repository's Settings → Secrets and variables → Actions: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_TEAM_ID`, `APPLE_ID`, `APPLE_PASSWORD`.
5. Push the first beta tag after CI passes. Release CI imports the identity into a temporary keychain, signs/notarizes with Tauri, verifies the stapled app ticket and Gatekeeper assessment, then deletes the temporary keychain. GitHub runners are disposable. Keep the exported identity in your own secure backup.

Never paste credentials into an issue, chat, Git commit, release note, or workflow file. Never add certificate files to source control. The public-tree check blocks recognizable credential patterns and private file types, but does not replace careful review of what you publish.

If Apple rejects the login, generate a new app-specific password and run `npm run notarization:setup`. This updates only the two notarization login secrets, retaining the signing identity. Both setup modes check authentication with Apple's notarization service before updating GitHub. A failed login check leaves existing GitHub secrets unchanged.

Tagged macOS builds notarize/staple the application through Tauri, then separately notarize/staple the signed DMG that users actually download. The pipeline checks Gatekeeper acceptance of both before publishing.

Windows signing is intentionally not configured. It can be added later without changing the architecture matrix or versioning convention. Unsigned Windows downloads may show a SmartScreen warning. Beta.6 introduced the in-app updater. Users of beta.5 and earlier manually install an updater-enabled release once.


## Updater signing and channels

Run `npm run updater:setup` once from an authenticated checkout. It creates or reuses the updater identity under `~/.codex/local-chat-signing/LocalChat-Updater.key`, restricts local key access, sets the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`, and writes only the public key to `src-tauri/updater.pub` and configuration. It rejects a mismatched existing identity before changing CI secrets. The committed key must match configuration; release checks enforce the signature/version/HTTPS policy. Keep a secure backup of the private key outside Git. Never rotate it accidentally: installed apps trust the embedded public key.

Every tagged release generates the updater archive or reuses the platform installer, plus a `.sig` file. Windows remains Authenticode-unsigned; updater signatures still authenticate its package. CI uses `local-chat-update-verify` with the same signature library as Tauri to reject wrong keys, modified packages and version substitution. Signing secrets are absent from ordinary CI.

The publisher uploads `update.json`, signatures, packages and SHA256SUMS, publishes the complete release, and then updates the separate `updates` branch. Applications read `https://raw.githubusercontent.com/youssefsz/local-chat/updates/beta.json` or `stable.json`. Version tags remain immutable. GitHub's ordinary latest-release URL is not used because prereleases must be delivered on the beta channel. Stable releases update both channels when newer. The publisher checks versions so delayed jobs cannot replace a newer feed, and uses a serialized publication group. Public feeds contain release metadata only.

Check `Check for Updates…` in the app menu or Settings. Downloads stay staged until the user chooses Restart to update. A completed download is retained across Quit and verified again on the next launch. Linux AppImage updates require a writable installation; Debian users follow the package download path. Actual installation/restart on Windows and Linux desktops remains a separate manual check.

## Preview release notes

Write reviewed highlights in `docs/release-notes/VERSION.md`. Commit the version preparation on its branch, then preview before tagging:

```sh
python3 tools/release_notes.py --version 0.1.0-beta.7 --commit HEAD --output artifacts/release-notes-preview.md
```

This command reads GitHub history and writes a local preview. It does not publish or edit a release. The current version's highlight file must exist.
The publisher uses the latest older published ancestor release as the explicit comparison base, excluding failed tags and unpublished drafts. GitHub groups PRs using `.github/release.yml`. Direct commit authors linked by GitHub and explicit co-author names receive credit too; emails are never printed in the notes. Keep PR titles and release labels accurate.

Download links, upgrade instructions, known platform limitations, and a full comparison link are included. The publisher freezes the notes when it creates a draft. Retrying that draft retains its reviewed text. Published assets and updater metadata remain unchanged. A maintainer can separately edit a release description after reviewing a preview, while retaining the provenance marker and all original assets.

## Repository maintenance

Contributors use branches and PRs. Main requires the three native CI checks; the repository owner retains an administrative recovery path. Never run untrusted PR code with release secrets. Ordinary CI has read-only permissions. Private vulnerability reports use GitHub Security, linked from SECURITY.md. Generated test summaries are workflow artifacts, not committed evidence files.
