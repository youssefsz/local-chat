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

The initial mdns-sd prototype passed isolated process tests, including 20 peers, but its raw multicast discovery did not advertise reliably from the Finder-launched application on this macOS 27 host. The macOS adapter now uses the system DNSService APIs through a bounded native adapter. Bonjour scopes registration, browsing, resolution and address lookup to the selected physical interface. Tokio, TLS, signed events, storage and context policy are unchanged. Windows and Linux discovery adapters remain explicitly unverified.

Each integration run selects a fresh test-only DNS-SD service. Test-profile builds default to `_localchat-test._tcp`; production always uses `_localchat._tcp`. The test-only environment option never compiles into production. An explicit manual test can opt a test peer into the production service. No test HTTP or browser control endpoint is distributed.
