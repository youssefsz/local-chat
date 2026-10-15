# Performance measurements

Hardware: Apple M5 MacBook Air, 24 GB RAM, arm64. OS: macOS 27.0 build 26A428. Toolchain: Rust 1.93.0, Node 24.13.0. The application is a release build with LTO. Debug information remains enabled because stripping compiler plug-ins breaks the installed Rust toolchain on this macOS version.

## Search and pagination

`local-chat-bench` creates a fresh SQLite database using the application schema and FTS5 triggers, with 100,000 clearly synthetic text messages. Queries alternate between 1,000 matching rows and 99,000 matching rows, return at most 50 results, and include matching excerpts. The benchmark runs 200 search queries and 200 queries for the latest 80-message page.

| Measurement | Actual | Initial target |
| --- | ---: | ---: |
| Local search p95 | 59.75 ms | below 150 ms |
| Local search median | 35.96 ms | no separate target |
| 80-message storage query p95 | 0.071 ms | storage component only |
| Fixture creation | 3.11 s | no target |

The storage page timing is not the complete UI room-switch time. One measured native room switch was 20 ms, including IPC, rendering and local read-state update. WebKit coarsens the timer; this is one sample, not a percentile claim. This is not a full room-switch distribution.

Reproduce with a fresh output directory:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --release --no-default-features --features test-profile --example local-chat-bench -- .test-data/new-benchmark
```

Historical raw measurements are retained in Git history and the maintainer archive. New runs write ignored local output.

## Idle resources

The read-only `tools/measure_macos.py` script records process IDs before launch, then samples the app and new WebKit processes. The available sample covers 30.12 seconds, an open General room, retained local history and zero connected peers.

| Measurement | Actual | Initial target |
| --- | ---: | ---: |
| CPU, one-core equivalent average | 0.30% | below 1% |
| Average additive RSS | 161.97 MiB | about 200 MB |
| Peak additive RSS | 169.48 MiB | about 200 MB |

RSS adds shared pages and is not physical memory footprint. Attribution includes the Local Chat process plus WebKit GPU, WebContent and Networking processes created after the baseline. This is a small idle scenario, not a 20-peer or 100,000-message steady-state measurement. A final signed-release sample with one connected peer and the open General room covered 30.44 seconds: 0.066% one-core CPU average, 168.63 MiB average additive RSS and 169.22 MiB peak. Neither sample represents 20 peers or 100,000 messages in the live UI.

## Launch variability and initial target miss

Measured launch from Rust `run()` to frontend-ready was 7.62 seconds, above the approximate 1.5-second warm-launch target. The phase measurements locate the delay in the OS credential lookup: setup entered at 259 ms, the installation identity was ready at 7,288 ms, and setup ended at 7,324 ms. The frontend then became ready at 7,618 ms. The private key remains in Keychain. No insecure file fallback was added to conceal the delay.

The final Developer ID signed release, opened from Finder with warm OS and Keychain caches, measured 494 ms to frontend-ready. Setup entered at 237 ms, identity was ready at 248 ms, and setup ended at 265 ms. This later sample meets the initial target, but one fast sample does not erase the earlier 7.62-second miss or establish a reliable launch percentile. Keychain lookup latency varied sharply between runs. Both measurements exclude Finder or LaunchServices time before the process starts.

## Bounds and implementation

The frontend renders at most 160 messages, loads history in 80-message pages, and limits search to 50 results. Outgoing change capacity is 256, event batches are at most 8, and encoded frames are at most 1 MiB. Oversized encoded batches split into individual bounded event frames. Matching-summary digests are cached until eligible event membership changes. Presence updates do not reload conversation pages. Historical import emits one history refresh per batch and no catch-up notifications. No timer runs per message.

## Startup loading update

Secure identity initialization now runs off the native event loop. During a dedicated 12-second injected delay, the dark loading window showed at 633 ms and the light loading window at 454 ms from Rust `run()`. The local UI became ready at 12,278.79 ms and 12,197.66 ms respectively. The injected delay demonstrates responsiveness while initialization waits; it does not measure Keychain latency. The same window retained its dimensions on completion.

With delay injection disabled, the dedicated System launch was ready at 356.75 ms and showed at 358 ms. The final installed signed production release, opened from Finder, was ready at 467.32 ms and showed at 471 ms. Both recorded `startup_loading_shown=false`. The loading threshold is 180 ms after frontend mount, with no minimum display duration. Native show-call timing is not compositor first paint, and these single warm samples do not establish a percentile or eliminate OS credential latency. The historical measurements above remain valid for the previous synchronous setup. See [testing](testing.md) for the test boundaries.
