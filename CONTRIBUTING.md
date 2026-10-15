# Contributing to Local Chat

Report reproducible bugs, propose features, improve documentation, or open a focused pull request.
For vulnerabilities, follow [the security policy](SECURITY.md).

## Set up a development copy

Install Node.js 24 LTS, Python 3.11 or newer, the Rust toolchain pinned in `rust-toolchain.toml`, and [Tauri's platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run dev:app
```

The development app uses a separate bundle ID and data directory. Do not point tests at your real chat database.
Python runs maintenance and test scripts. Users of packaged Local Chat do not need Python.

## Make a change

1. Open an issue before a large feature or architectural change. Small fixes can go directly to a PR.
2. Create a branch from `main` and keep the change focused.
3. Add a regression test for changed behavior where practical.
4. Run the relevant checks below.
5. Open a PR with the problem, behavior, verification, and a short release note.

Maintainers apply release labels. Features use `enhancement`, fixes use `bug`, and internal changes use `maintenance`.
Use `breaking-change` for incompatible behavior and `skip-changelog` only for changes with no useful release-note entry.
Squash PR titles should describe the resulting change. Preserve authorship and `Co-authored-by` trailers when combining contributions.

## Run checks

```sh
npm run lint
npm run test:all
```

`test:all` runs Rust tests, Python contract tests, real updater-signature checks, and the Bonjour regression on macOS.
Real discovery process tests are separate because they require a usable physical interface:

```sh
npm run test:integration
```

See [testing](docs/testing.md) for each suite, manual native checks, and pending physical tests.
Formatting uses `npm run format`. Commit both lockfiles when dependencies change.

## Keep the product's boundaries

Rust owns persistence, validation, identity, and networking. Keep OS operations in platform adapters.
Every history export must respect its locally assigned network context. A familiar peer does not authorize old history export.
Preserve exact code content and creator-only room management. Keep frames, queues, and visible message pages bounded.

Use the existing design tokens and [approved reference](docs/reference/README.md). Use synthetic data in screenshots.
Read [architecture](docs/architecture.md), [protocol](docs/protocol.md), and [network contexts](docs/network-contexts.md) before changing these contracts.

## Review and release

PRs run CI for macOS ARM64, Windows x64, and Linux x64. Report which platforms you exercised yourself.
A passing build does not certify physical LAN, firewall, sleep, or desktop behavior.
Only maintainers publish version tags and configure signing secrets. See [releasing](docs/releases.md).

Contributions are distributed under the repository's [MIT license](LICENSE). Follow the [code of conduct](CODE_OF_CONDUCT.md).
