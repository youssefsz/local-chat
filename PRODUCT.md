# Local Chat

## Platform
Desktop. CI builds macOS ARM64, Windows x64, and Linux x64. macOS is locally exercised; Windows and Linux physical desktop verification remains pending.

## Users and purpose
Ordinary people on 2–20 computers share text, HTTP/HTTPS links, and code in public named rooms on the same local network. Install, open, grant local-network access when asked, communicate. No accounts, server, codes, addresses, or internet dependency.

## Operating context
Reading, sending, finding, and exact copying are the primary tasks. All current-network rooms are public. General is permanent. Creator installation keys authorize room management. Duplicate names are allowed. Closing retains background reception; Quit stops it.

## Requirements
Rust owns identity, durable state, validation, network context, discovery, TLS sessions and synchronization. SQLite persists immutable signed events, gaps, drafts, delivery, read positions and preferences. The frontend renders bounded pages and issues commands. Old-network data remains locally readable and cannot be exported on a new or uncertain network. Never seed production with sample peers or messages.

## Visual authority
The user approved docs/reference/outputs/index.html and its screenshot. Preserve its composition and tokens. Rename Nearby to Local Chat. Remove the presentation canvas and imitation window controls. Default appearance is System, persisted as System.

## Scope
Automatic discovery, public rooms, text/links/code, copy, local search, history, catch-up, creator management, network separation, appearance and native background/notifications. No accounts, cloud, direct messages, private rooms, attachments, voice, read receipts or plugins.

## Evidence
Available hardware is one arm64 Mac. Multi-process tests cannot certify discovery across physical machines. Report measured performance, signing and notarization separately. Network impersonation cannot be absolutely prevented by unauthenticated LAN discovery.
