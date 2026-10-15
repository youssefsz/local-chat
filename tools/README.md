# Maintenance tools

These scripts support contributors and CI. They are not installed with Local Chat, and packaged-app users do not need Python.
Python scripts require Python 3.11 or newer. Publishing and credential setup also require the GitHub CLI.

| Tool | Purpose |
| --- | --- |
| `release.py` | Synchronize and validate npm, Cargo, Tauri versions, and release tags |
| `prepare_artifacts.py` | Inspect real architectures, reject test controls, verify signatures, and stage installers |
| `publish_release.py` | Publish complete releases and advance update feeds after successful uploads |
| `release_notes.py` | Preview highlights, categorized GitHub PR notes, verified contributor credits, and downloads |
| `update_feed.py` | Build deterministic metadata and advance channels without replacing newer versions |
| `configure_signing.py` | Validate Apple login and send signing credentials directly to repository secrets |
| `configure_updater.py` | Configure the separate updater identity outside Git |
| `macos_ci_signing.py` | Import the release identity into a temporary CI keychain |
| `notarize_dmg.py` | Notarize, staple, and assess the downloaded Mac container |
| `verify_macos_bundle.py` | Verify the actual Mac bundle and exclude test-only controls |
| `check_public_tree.py` | Reject tracked credentials, private data, generated evidence, and machine-specific home paths |
| `check_docs.py` | Check local documentation links and README image paths |
| `run_tests.py` | Run named suites and write data-free CI summaries |
| `measure_macos.py` | Sample process CPU and additive RSS on macOS |

Standalone Python and C test programs live in [tests](../tests/README.md).
The C program injects OS poll failures into the actual Bonjour adapter. It is compiled only during its test.
Runtime native C adapters remain under `src-tauri/src/platform`.

See [release operations](../docs/releases.md) before running publishing or credential commands.
