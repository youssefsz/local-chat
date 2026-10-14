# Network contexts

Before historical synchronization, context policy must be validated with isolated test peers.

macOS selects one physical default-route interface. SystemConfiguration supplies the confirmed interface, ARPResolvedIPAddress and ARPResolvedHardwareAddress for the primary network service. The adapter checks those against the selected route and uses no SSID, BSSID or location access. Native SCDynamicStore change notifications invalidate each session generation immediately; the coordinator reevaluates the route before opening replacement sessions. Discovery advertisements, listening sockets and dialing are restricted to that interface and its IPv4 subnet. VPN/virtual routes never become a sharing interface. A stable context needs a physical interface, local subnet/mask, default gateway IP and resolved gateway hardware address. No SSID or location permission is requested. The evidence digest includes the gateway MAC, gateway IP and subnet/mask, not a peer list. Sockets also bind the physical interface with IP_BOUND_IF. A familiar peer never authorizes an export. Raw evidence stays local.

Without gateway hardware evidence, use a new session context. Keep previous context history local and read-only. An interface, address, route or evidence change cancels connections and discovery before reevaluation. Resume recognized history only after all required evidence agrees. A network adapter reports unavailable when there is no permitted physical route. The listener never binds wildcard addresses in production.

Event namespaces are random per local context, not network fingerprints. Session peers may hold different local context IDs; each imports current eligible data into its own active context. This lets separate fresh groups converge without sharing their local recognition evidence. A duplicate event already assigned to a previous local context is acknowledged without reassigning it. It cannot resurrect export permission or prevent fresh communication.

This conservative combination distinguishes ordinary networks with identical SSIDs or IP ranges but different gateways. It cannot prove continuity against deliberate router MAC/IP/subnet impersonation, or two distinct networks configured with all the same evidence. That limitation follows from unauthenticated LAN discovery. Ambiguity means old history stays local.

## Required scenarios

Automated adapter-policy fixtures cover same-evidence reconnect, different gateways, identical SSID/range with different hardware, familiar groups moving together, simultaneous interfaces, VPN selection, absent evidence and export generation changes. Physical Wi-Fi/Ethernet changes, VPN activation, sleep/wake and two-router discovery remain manual until actually exercised. See testing.md for outcomes.
