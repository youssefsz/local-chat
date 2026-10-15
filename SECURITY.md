# Security policy

## Report a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/youssefsz/local-chat/security/advisories/new).
Do not include vulnerability details in public issues or pull requests before coordinated disclosure.

Include the affected version and operating system, reproduction steps, expected security boundary, and observed impact.
Use synthetic conversations. Do not submit private chats, signing keys, passwords, or real network identifiers.
Maintainers review reports, discuss a fix privately, and coordinate publication with the reporter.
This volunteer project does not promise a response deadline or a bounty.

## Supported versions

Security fixes target the newest published release in the current beta series. Upgrade older beta builds before retesting.
There is no stable release or long-term support series yet. The [release page](https://github.com/youssefsz/local-chat/releases) identifies current downloads.

## Security boundaries

Local Chat discovers computers on the current LAN without an account or a trusted central directory.
Discovery input and peer messages are untrusted. Signed events and pinned TLS authenticate installation keys, not a person's identity.
All current-network rooms are public to peers on that network. Avoid sharing sensitive material on an untrusted LAN.

Important properties include context-bound history exports, creator-only room management, signature validation,
bounded frames and queues, secure local key storage, and verification of update packages and their announced versions.
Violations of those properties are relevant reports. The scope includes application code, native adapters, dependencies,
and the build and update pipeline. No finding class is excluded by this policy.

Network recognition cannot prove continuity against deliberate impersonation of the same gateway hardware and addressing.
This limitation is described in [network contexts](docs/network-contexts.md).
Report cases where implementation behavior exceeds documented limitations or exports history without the required evidence.

macOS releases are Developer ID signed and notarized. Windows installers are intentionally unsigned by Authenticode.
Updater packages on all supported targets have separate signatures. Physical Windows and Linux validation remains documented in [platform status](docs/platform-status.md).
