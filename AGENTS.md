# Local Chat implementation instructions

Read PRODUCT.md, DESIGN.md, docs/architecture.md and docs/network-contexts.md before changing behavior. Preserve the approved HTML under docs/reference without editing it. Match its density, proportions, tokens and system fonts; this is an Operate desktop UI.

Rust owns all durable and network state. No frontend SQL access, private keys or network permission decisions. Every event query and export must include the locally assigned network context. Never trust an incoming context identifier as export authorization. Pause all sessions before changing context. Only the room creator key can manage a user room. General cannot be managed. Preserve exact code content.

Use bounded message pages, network frames and queues. No message bodies or secrets in logs. Pin dependencies and commit both lockfiles. Keep platform operations in platform adapters. Windows/Linux are unverified until exercised there.

Verification commands and actual outcomes live in docs/testing.md. Keep test profiles isolated from production data and single-instance behavior. Never compile test control endpoints into distributed builds. Update evidence and open limitations honestly.
