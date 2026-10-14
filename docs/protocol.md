# Local Chat protocol 1

## Discovery and transport

On macOS, native Bonjour DNSService APIs advertise `_localchat._tcp.local.` on one selected physical interface. TXT fields are `protocol=localchat`, `version=1`, an Ed25519 peer public key and a SHA-256 TLS certificate fingerprint. Neither messages nor rooms appear in DNS-SD. The lower peer key initiates a bidirectional session. Addresses must be in the permitted physical subnet. The listener binds its physical IPv4 address. IPv6-only networks are not yet supported.

TCP carries TLS using rustls with its maintained ring provider. The client pins the discovered exact certificate fingerprint and verifies the TLS handshake signature using the certificate's actual public key. It does not disable signature verification. First contact is unauthenticated LAN discovery, not a WebPKI name or proof of a human identity. Persistent Ed25519 and TLS secrets are stored in the OS credential adapter. Test builds use isolated permission-restricted files.

Both sides sign a hello containing protocol version, installation ID, profile, platform, and a 32-byte TLS exporter binding. This proves possession of the installation key and prevents replay into a different TLS channel. Outgoing sessions require the discovered peer key. Inbound peers must also prove possession of their declared Ed25519 key. TLS has no generic accept-all verifier.

## Framing and limits

Big-endian u32 length prefix, followed by a UTF-8 JSON object with a `type` discriminator. Maximum frame is 1 MiB. Maximum event batch is 8. Inventory pages contain at most 512 sequence entries. Profile and room name limits are 80 Unicode scalar values, room descriptions 240, and content 65,536 UTF-8 bytes. Limits live in `domain/mod.rs`. Network parsing, event validation and Rust command validation enforce boundaries independently of UI fields.

Handshake and write timeout: 5 seconds. Idle ping: 10 seconds. No received frame for 35 seconds closes the session. Peer connection retries use exponential backoff capped at 32 seconds plus 0–499 ms jitter. Per-engine change capacity is 256; a lagged subscriber closes rather than silently dropping an event. The production connection cap is 19 outgoing peers with bounded accepted sessions. Slow writes close after timeout. Reconciliation repairs interrupted live delivery.

## Event encoding and authority

An event contains `version`, `namespace`, `id`, `author`, `profile`, `sequence`, `logical`, `timestamp`, `payload`, and `signature`. The canonical signed bytes are compact serde_json encoding of the ordered tuple:

`[version, namespace, author, profile, sequence, logical, timestamp, payload]`

Payload is a Rust tagged enum with declared field order. Version 1 has no unordered maps in signed data. The event ID is SHA-256 of these canonical bytes. Signature is Ed25519 of those bytes. Author is the 32-byte public key encoded in lowercase hex. Signature is 64 bytes encoded in lowercase hex. Namespace is a random UUID created for the originating local context. Counters are positive signed integers within the JavaScript safe integer range. Wall-clock milliseconds are display metadata. Ordering uses logical counter then author and event ID for room management; message ties use event ID.

Payloads are `create_room`, `rename_room`, `archive_room`, and `message`. Text/code is a tagged enum value. User room identity is `<full creator public key>:<random UUID>`. A create event must match that key prefix, so another installation cannot forge ownership by choosing a familiar room name or ID. General has the reserved ID `general` and cannot receive management events. Only verified creator events affect room metadata. Delayed messages survive archive. Non-create events arriving before creation remain durable and materialize deterministically when creation arrives.

## Receipt and catch-up

Local event and pending delivery are committed with message/room materialization. Only then does the UI show success and the peer session send it. The receiver verifies and transactionally commits before sending an `ack`. An acknowledgement means durable storage, never reading. The first durable remote receipt changes a local label from `waiting` to `shared`; this is not an all-devices receipt. The event store itself is the retry queue: reconnect reconciliation includes every eligible event without requiring the original author online.

`summary` carries a digest of sorted eligible event IDs. Different digests request paged `inventory` entries of `(namespace, author, sequence)`. The receiver requests actual missing membership in batches, including holes below the largest observed sequence. `need` responses export signed original events. Events and sequence uniqueness make retries idempotent; equivocation is rejected. Progress is durable event membership, so interrupted sessions resume from the actual gaps. Matching digests exchange pings without inventories. History is relayed with `live=false`, suppressing catch-up notifications.

## Context restriction

The wire namespace never selects the receiver's local context. Every query, inventory, export, acknowledgement and import uses the engine's active local context under the database lock. Connections capture a context and cancellation generation. A changed route cancels sessions before switching the active context. An event ID already retained in a different local context is acknowledged without being assigned to the new one. Old history never enters new catalogs or pending delivery. Details and recognition limits are in `network-contexts.md`.

Matching-summary digests are cached until event membership or context changes. Historical batches emit one UI history refresh instead of a notification per event, and do not announce intermediate summaries. This prevents repeated inventories during catch-up. Presence changes refresh only navigation and status; they do not reload the conversation page.

Bonjour discovery errors update the unavailable state and trigger a retry after a cooldown. Production registers only `_localchat._tcp`. Test-profile builds use separate services so test fixtures never join ordinary rooms unless explicitly requested for a manual desktop test.
