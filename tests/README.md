# Test layout

| Location | Purpose |
| --- | --- |
| `src-tauri/src/**` | Colocated Rust tests for domain rules, storage, sessions, adapters, and updates |
| `tests/contracts` | Python tests for release versions, architecture, signing setup, feeds, resume, and attribution |
| `tests/integration` | Real signature checks, C Bonjour fault injection, macOS peer processes, and the manual updater fixture |

Run `npm run test:all` for all non-interactive suites. Run `npm run test:integration` for macOS physical-interface process checks.
Python 3.11 or newer is required. Signature checks also need npm dependencies and Cargo.
The Bonjour test compiles its C program with Clang against the real native adapter and requires macOS.

Test programs use synthetic content and isolated stores. Never point them at production data.
The updater fixture is a manual server, not a test that CI invokes automatically.
Read [testing](../docs/testing.md) for scope, results, and pending physical checks.
