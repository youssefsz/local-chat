# Architecture

`send_message(room, content, kind)` validates and persists an immutable signed event and pending state in one transaction. A change event refreshes the visible page; bounded peer sessions export eligible events. Durable acknowledgements update delivery. No UI SQL API exists.

## Ownership and contracts

- `domain` owns signed event encoding, size limits, author identity and room authority. Ed25519 author keys sign an explicit ordered JSON tuple. Event IDs are the SHA-256 of canonical bytes. Each installation persists a monotonic sequence; Lamport values order independently of display clocks.
- `storage` owns SQLite WAL migrations, context assignment, event ingestion/materialization, FTS5, drafts, read positions, mute and preferences. Incoming events receive a local context assignment. Remote namespaces never select a local export context.
- `network` normalizes platform DNS-SD discovery, then owns pinned TLS, framing, challenge authentication, live sessions, bounded reconciliation and retry. Every session captures a context generation; a generation change cancels sessions before exports resume.
- `platform` owns physical-interface evidence, secure credentials and native integration. Unsupported context adapters fail conservatively and must be completed before platform certification.
- `app` coordinates Rust state and Tauri events. The frontend keeps selection, open dialogs and scroll anchors; persisted preferences and drafts remain in Rust.

## Topology decision

Direct peer sessions require at most 19 connections per device at the requested 20-device scale. This avoids coordinator election, leader failover and one laptop owning availability. An elected coordinator would reduce connection count but needs leader migration and durable replication anyway. Choose direct peers; measure 20 peers before changing topology. The lexicographically smaller installation ID initiates a connection. Each connection is bidirectional.

## Context and namespace

Device identity, local network context and event namespace are different identifiers. Each local context has an origin namespace. Received valid events keep their origin namespace and are assigned to the receiving local context. General is a system room within each local context. User room IDs are random stable identifiers. Independent groups can merge their catalogs when discovered because neither General nor a room label acts as a global unique network identity.

A new network creates a separate local context. Only the current context is eligible for export, pending delivery or catalogs. Familiar peer identity never grants continuity. Uncertain evidence creates a fresh context while previous history remains read-only. See network-contexts.md.

## Failure paths

An invalid signature or oversized frame closes the session. A slow peer cannot grow an unbounded queue. Database failure prevents an acknowledgement. Unknown room events remain in the event store and materialize when the signed create arrives. Owner changes use `(lamport, author, event_id)` ordering. Archive affects composition, not acceptance of delayed historical messages. Reconciliation uses actual sequence membership, including gaps, and resumes after interruption.

## Main risks

mDNS can be blocked by router isolation or OS permissions. First-contact discovery identifies a key, not a person. Router hardware identity can be impersonated. Uncertain context recognition sacrifices automatic historical continuity to avoid accidental export. Physical LAN, permission, sleep and resource measurements require actual tests, recorded separately.

## macOS discovery decision

The initial mdns-sd prototype passed isolated process tests, including 20 peers, but its raw multicast discovery did not advertise reliably from the Finder-launched application on this macOS 27 host. The macOS adapter now uses the system DNSService APIs through a bounded native adapter. Bonjour scopes registration, browsing, resolution and address lookup to the selected physical interface. Tokio, TLS, signed events, storage and context policy are unchanged. Windows and Linux use `mdns-sd` scoped to the selected IPv4 interface index. Their physical-machine verification remains pending; native CI build and test results are recorded separately.

Each integration run selects a fresh test-only DNS-SD service. Test-profile builds default to `_localchat-test._tcp`; production always uses `_localchat._tcp`. The test-only environment option never compiles into production. An explicit manual test can opt a test peer into the production service. No test HTTP or browser control endpoint is distributed.

## Startup and window reveal

The main native window starts hidden. `app/startup.rs` owns `loading → ready | failed` and the saved appearance. Storage and secure identity initialization run on a blocking worker, leaving the native event loop free while macOS accesses the login keychain. The frontend subscribes before querying startup status, applies the saved appearance to both the document and native title bar, then reveals the same window. A 180 ms threshold from frontend mount suppresses the loading indicator on fast initialization; there is no minimum display time. The conversation shell retains layout while hidden and inert until initial local state is ready. No animation or resize accompanies the handoff.

`show_window` reveals the themed loading or error state once; `ready` reveals an immediately usable shell if needed and starts discovery once. Completion respects a window the user closed during startup. Early Dock or second-instance reopen events cannot expose an unthemed webview. Quit and Exit tolerate an engine that has not initialized yet. Failed initialization offers a retry; loading and ready states cannot start duplicate workers. A retry reuses an already initialized engine after a frontend failure. There is no insecure identity fallback.

A separate splash window would add a second window lifecycle and geometry handoff. Blocking setup prevented the loading screen from rendering during keychain delays. The chosen same-window flow preserves the approved workspace and keeps network discovery independent of startup readiness. Reading persisted appearance precedes reveal; an inaccessible database produces a recovery screen using System appearance.

Only dedicated `test-profile` builds read `startup.json` with bounded `delay_ms` and `fail_once` controls. They use the existing isolated data directory and test identity. Production bundle verification rejects those markers. Local timing metadata distinguishes the native show call from usable local UI; neither is a measurement of compositor first paint.

## Portable platform boundary and releases

Keep the macOS native Bonjour adapter because its Finder-launched permission behavior was verified. Windows/Linux use a separate scoped `mdns-sd` adapter, not a replacement of the macOS path. `netdev` reads the default route, addresses and gateway; Linux sysfs and Windows GetIfEntry2 independently confirm physical hardware. Unknown/virtual/tunnel interfaces and ambiguous addresses fail closed. A private physical-evidence flag replaces macOS-specific interface-name assumptions in context fingerprinting. Missing gateway hardware cannot recognize retained history.

Linux RTNETLINK and Windows IP Helper callbacks invalidate the generation on link, address and route changes. Export guards compare this generation on every frame, and the coordinator cancels sessions before querying new evidence. Observer failure returns generation zero and prevents sharing. Linux sockets bind with SO_BINDTODEVICE on supported kernels; Windows sets IP_UNICAST_IF with the interface index in network byte order and also binds the selected local IPv4 address. Binding errors never fall back to a wildcard socket. Discovery has a bounded output queue and publishes only protocol/version/identity/certificate records. OS-specific validation and observer/socket tests run on their corresponding CI runners; physical-router and firewall tests remain separate.

CI and release workflows share one three-target build matrix. Ordinary CI has no signing secrets and uploads temporary test installers. Release tags must match npm, Cargo and Tauri versions and point to main. Tagged macOS builds require Developer ID signing and notarization. Windows is explicitly unsigned. The publisher checks a manifest for every platform, creates checksums, uploads a draft, then publishes only when all installers exist. Published versions are never overwritten. See releases.md for operations and credential setup.
