disposition: fix

Inputs not supplied: a new five-block direction contract, quality-bar card, measured spec, comp-diff reports, and build state; this is the expressly approved prototype-preservation task, so no new concept roll, seed, generated comp, or comp-build gates apply. The supplied Operate reference is the quality authority. No detector ran for this native desktop surface. Review is limited to the named screenshots and sampled primary source; no live UI or browser was used.

## persistence

Fail: PRODUCT.md exists and correctly records native desktop scope, the approved reference, Rust ownership, and truthful production state. DESIGN.md retains prototype-only claims about sample people, browser-local messages, the provisional name, and `.impeccable/design.json`, despite its production-adaptations appendix. Reconcile its main component rules with the shipped implementation. The approved HTML and preview exist under docs/reference/outputs and were inspected without editing. All eight required native captures are valid JPEGs at exactly 2× their named logical dimensions: 1280×720, 1440×859, 800×600 in both themes, plus both 1280×800 production-peer captures. The requested 1440×900 native window was limited by the Mac work area to 1440×859; this review does not certify the unavailable height or Windows/Linux.

## fidelity

The approved preview's salient inventory is a compact warm/dark sidebar, room list and search, author initials and metadata, left-aligned flowing messages, an inset dark code panel with Copy code, a fixed bottom composer, quiet network/profile footer, and an optional people panel. Native controls replace the presentation frame.

| Element | Verdict | Evidence |
| --- | --- | --- |
| Sidebar/conversation topology and density | match | 246px sidebar at normal desktop widths; controlled message/composer width; no card-per-message treatment. |
| Native frame and Local Chat branding | adaptation | brief.txt §§2 and 5 expressly require the name change, actual OS controls, and removal of the outer presentation canvas. |
| TYPE | match | System sans, compact 11–14px metadata/body scale, 17px room title, system monospace snippets preserve the reference's desktop character. |
| MATERIAL | match | The reference is crisp UI geometry and flat tonal surfaces; thin borders and restrained radii remain. No raster illustration or decorative physical material is required. |
| GROUND | match | Pixel samples of the native captures and approved desktop baselines agree: light main RGB 255/255/255 and sidebar 245/244/240; dark main 34/36/35 and sidebar 29/31/30. JPEG/color output explains the tiny difference from CSS literals equally on both sides. |
| Selected room, icons, restrained accent | match | Terracotta selection in light, peach/green-gray roles in dark, consistent authored stroke icons. |
| Message avatars | contradicted | dark-1280x720.jpg has blank own-message avatars while fixture-author initials remain; dark-800x600.jpg reduces two-character initials to clipped strokes. The light captures and dark-1440x859.jpg show the intended full initials. |
| Message flow and fixed composer | match | Conversation content scrolls above the composer at every supplied size; code and links remain within the reading pane. |
| Explicit URL rendering | adaptation | brief.txt permits URL/hostname display and forbids arbitrary automatic preview fetching; MessageView.svelte uses explicit clickable HTTP/HTTPS text. |
| Delivery and nearby people | adaptation | PRODUCT.md and brief.txt require real backend state instead of sample peers; fixture captures label their content synthetic, and production-peer captures show the connected peer and actual delivery vocabulary. |
| Compact sidebar at 800×600 | contradicted | Both themes split the search shortcut over two lines and break “People nearby” and “0 connected” into fragmented labels. This loses the approved compact control rhythm. |
| Optional people panel | adaptation | Present in App.svelte using backend peers; the supplied viewport captures show its closed state. Its open-state visual finish was not evidenced in this packet. |

## ceiling

Operate familiarity and restraint are preserved; adding ornamental devices, display fonts, or motion would weaken this surface. The remaining ceiling is stable native theme repainting and intentional compact-width labels. The first viewport communicates room, nearby state, conversation, and send action immediately. Production semantics correctly replace the prototype's simulation promise. New concept-seed/FORM checks are inapplicable because the user prohibited a new concept round.

## material_fixes

1. Fidelity/state: repair disappearing or clipped avatar initials across native Light → Dark → Light changes; investigate src/App.svelte:94 (`applyTheme` and native `setTheme`), src/components/MessageView.svelte:38 (avatar text), and src/styles/app.css:364 (`.avatar` paint/layout). Preserve explicit foreground/background and the 33px footprint; verify full one- and two-character initials in visible native pixels after repeated switches at 1280×720 and 800×600. AX text alone is insufficient evidence.
2. Fidelity/responsive: fix src/styles/app.css:228/246/257 and the ≤850px rules, plus src/App.svelte:615/687 if shorter narrow-width copy is needed. Keep the shortcut unbroken with no flex shrink, give search text an intentional truncation or compact label, and keep people/count labels whole rather than splitting their words. Preserve the existing sidebar/conversation proportions and fixed composer.
3. Persistence/truth: update DESIGN.md's prototype-only prose to describe the production app; remove claims that people remain samples, messages live in the browser, the name is provisional, and an absent `.impeccable/design.json` owns elevation. Keep the immutable historical prototype documentation under docs/reference unchanged.

## keep

Keep the approved warm light and green-gray dark grounds, system type, compact room navigation, left-aligned message flow, exact code presentation, visible copy actions, and fixed composer; make only the bounded repairs above.
