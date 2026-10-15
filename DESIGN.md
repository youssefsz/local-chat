---
name: Local Chat
description: A compact native desktop workspace for local conversations.
colors:
  "surface": "#fff"
  "sidebar": "#f5f4f0"
  "subtle": "#faf9f7"
  "hover": "#eeece7"
  "line": "#e9e7e2"
  "ink": "#292b29"
  "secondary": "#646861"
  "muted": "#696e64"
  "accent": "#a94329"
  "accent-hover": "#913820"
  "accent-wash": "#f0e1d8"
  "green": "#367151"
  "code": "#252a29"
  "code-ink": "#e6eae4"
  "focus": "#a94329"
  "surface-dark": "#202422"
  "sidebar-dark": "#1b1f1d"
  "subtle-dark": "#272c29"
  "hover-dark": "#303630"
  "line-dark": "#353c36"
  "ink-dark": "#e4e9e2"
  "secondary-dark": "#b0b8ad"
  "muted-dark": "#a0aa9e"
  "accent-dark": "#edab8b"
  "accent-hover-dark": "#f5bc9f"
  "accent-wash-dark": "#41342c"
  "green-dark": "#9ac8a6"
  "code-dark": "#151a17"
  "code-ink-dark": "#e6eae4"
  "focus-dark": "#edab8b"
  "avatar-default-bg": "#e7e5d9"
  "avatar-default-ink": "#5e5d50"
  "avatar-you-bg": "#e9e4dd"
  "avatar-you-ink": "#6b6255"
typography:
  intro:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "21px"
    fontWeight: 600
    letterSpacing: "-.5px"
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "17px"
    fontWeight: 650
    lineHeight: 1.2
    letterSpacing: "-.3px"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.7
  author:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "13px"
    fontWeight: 650
  button:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "12px"
    fontWeight: 550
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif"
    fontSize: "11px"
    fontWeight: 600
    letterSpacing: ".015em"
  code:
    fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace"
    fontSize: "11.5px"
    fontWeight: 400
    lineHeight: 1.95
rounded:
  "badge": "4px"
  "control": "6px"
  "message": "7px"
  "base": "8px"
  "avatar": "10px"
  "dialog": "12px"
  "composer": "10px"
spacing:
  "4": "4px"
  "8": "8px"
  "10": "10px"
  "12": "12px"
  "13": "13px"
  "15": "15px"
  "16": "16px"
  "20": "20px"
  "24": "24px"
  "37": "37px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.surface}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "9px 15px"
  button-secondary:
    backgroundColor: "{colors.hover}"
    textColor: "{colors.ink}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "9px 15px"
  send-button:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "0 11px"
    height: "30px"
  send-button-hover:
    backgroundColor: "{colors.accent-hover}"
  icon-button:
    textColor: "{colors.secondary}"
    rounded: "{rounded.control}"
    width: "34px"
    height: "34px"
  text-field:
    backgroundColor: "{colors.subtle}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "10px 11px"
    width: "100%"
  room-row:
    textColor: "{colors.secondary}"
    rounded: "{rounded.control}"
    padding: "10px 12px"
    width: "100%"
  room-row-active:
    backgroundColor: "{colors.accent-wash}"
    textColor: "{colors.accent}"
  unread-badge:
    backgroundColor: "{colors.line}"
    textColor: "{colors.secondary}"
    rounded: "{rounded.badge}"
    padding: "2px 4px"
    height: "18px"
  code-block:
    backgroundColor: "{colors.code}"
    textColor: "{colors.code-ink}"
    rounded: "{rounded.base}"
  composer:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.composer}"
  avatar-dark:
    backgroundColor: "{colors.hover-dark}"
    textColor: "{colors.ink-dark}"
  avatar-you-dark:
    backgroundColor: "{colors.accent-wash-dark}"
    textColor: "{colors.accent-dark}"
  avatar:
    rounded: "{rounded.avatar}"
    width: "33px"
    height: "33px"
---

# Design System: Local Chat

## Overview

**Creative North Star: "Quiet communal studio"**

Local Chat keeps the approved reference's warm neutral sidebar, clear conversation pane, compact controls, and restrained terracotta accent. It is an Operate desktop interface for reading, sending, finding, and copying. System type, initial avatars, and authored SVG stroke icons keep those tasks familiar.

The production implementation is Tauri with Svelte and plain CSS. The application fills its native window and uses operating-system window controls. The approved HTML under `docs/reference/outputs/index.html` remains the immutable visual authority. This document records the production rules in `src/styles/app.css`, `src/App.svelte`, and `src/components/MessageView.svelte`; product and platform evidence remain in `PRODUCT.md` and `docs/testing.md`.

**Key Characteristics:**

- Warm sidebar beside a clear reading pane.
- Terracotta for actions, selection, links, and focus.
- Left-aligned messages with author initials and visible copy controls.
- Light, dark, and persisted System appearance without external fonts or images.
- Actual nearby peers, honest delivery labels, and local previous-network history.

## Colors

The light theme uses warm paper neutrals and dark green-gray text. The dark theme keeps the same roles with green-gray panels and a pale peach action accent. Frontmatter keys ending in `-dark` map to the matching `data-theme="dark"` overrides; use semantic CSS variables in components.

### Primary

`accent` identifies Send, primary dialog actions, active rooms, links, and carets. `accent-hover` changes Send on hover. `accent-wash` marks selection and search destinations. `focus` supplies keyboard outlines.

### Neutral

`surface` fills the conversation and dialogs. `sidebar` separates navigation. `subtle` and `hover` distinguish rows and interactive states; `line` supplies dividers and borders. `ink`, `secondary`, and `muted` distinguish message text, supporting text, and timestamps. `code` and `code-ink` keep snippets legible in both appearances.

`green` marks connected status alongside text or a count. The default and own-message avatar color pairs in frontmatter describe light appearance. Dark avatars use the current `hover` background and `ink` text; own-message avatars use `accent-wash` and `accent`. Dark avatar text also sets `-webkit-text-fill-color` explicitly to its foreground role. These treatments distinguish author presentation without asserting verified identity. The inherited sample-author color classes are not a production identity palette. Sidecar tonal ramps are generated previews, not additional application tokens.

**The Accent Rule.** Use the action accent for actionable or selected content. Keep ordinary message text in `ink`.

## Typography

Use the system sans-serif stack for interface text and system monospace for code and code-entry mode. The frontmatter records the implemented introduction, room title, message body, author, button, label, and code roles. There is no display-font treatment.

Message bodies have a maximum width of 72ch. Metadata uses compact 11px text; timestamps use tabular numerals. The wordmark uses 21px type at weight 650 with -.6px tracking, becoming 19px at widths of 850px or less. Composer text is 13px with a 1.6 line height. Code preserves whitespace and uses a 1.95 line height at normal desktop widths.

**The System Type Rule.** Keep the approved system fonts and compact hierarchy across navigation, conversations, and dialogs.

## Layout

The application fills the native content area with no presentation canvas, artificial window shadow, or imitation title bar. A fixed-width sidebar sits beside the flexible conversation. Navigation and conversation scroll independently; the room header and bottom composer remain available. Conversation and composer share an 850px maximum width.

- The sidebar is 246px normally, 218px at widths of 1000px or less, and 205px at widths of 850px or less. The native window configuration sets an 800px minimum width. Legacy narrower browser-preview rules do not establish a supported mobile layout.
- Conversation side padding is 37px normally, 25px at widths of 1100px or less, and 24px at widths of 850px or less. Header side padding follows 31px, 21px, then 20px at those widths. Sidebar side padding becomes 12px at 1100px or less.
- At 1000px or less, search uses the deliberate short label "Find messages". Its shortcut stays on one line without shrinking. The nearby count keeps its status dot and hides the word "connected"; labels remain whole.
- At desktop heights of 780px or less, sidebar and header spacing tighten. The nearby list visually hides entries after the second person. Conversation top padding becomes 23px and the introductory icon is hidden. The application does not render the reference's "View everyone" row; the header and network footer open the people panel.
- The people panel is 280px wide. It sits beside the conversation above 1100px and overlays the right edge at 1100px or less, aligned to the native content area's top, bottom, and right edges.

Messages use bounded backend pages; loading earlier messages retains at most 160 rendered messages and preserves the visible anchor. New messages follow the bottom only while the reader is there. Otherwise a "New messages" control preserves their place. "Load earlier messages" and "Jump to latest" expose history movement explicitly.

## Elevation & Depth

Flat tonal panels and one-pixel borders organize the workspace. Message rows have no shadow, and the application has no CSS window shadow. Floating menus, dialogs, the overlay people panel, toasts, and the new-message control use the small shadow vocabulary recorded in `.impeccable/design.json`. Dialogs dim the background with the source's translucent backdrop.

**The Flat Workspace Rule.** Reserve elevation for transient controls and overlays; keep the conversation and navigation flat.

## Shapes

Controls use the `control` radius; snippets use `base`; message hover areas use `message`. Dialogs use `dialog` and the composer uses `composer`. Message avatars use a softly squared 33px footprint, sidebar avatars use 27px with 8px corners, and profile avatars use 28px with 8px corners. Connected dots and the compact header avatar stack are circular. Message initials sit in a child text span with a unit line height. Dark appearance explicitly sets the avatar foreground, background, and WebKit text fill.

## Components

### Buttons and fields

Send and primary dialog buttons use the accent against the current surface color. Secondary buttons use the hover neutral. Disabled buttons have 40% opacity. Enabled buttons move down one pixel while pressed, except under reduced motion. Dialog buttons use a brightness change on hover; Send uses `accent-hover`. Icon buttons reveal a neutral fill on hover.

Text fields use a subtle fill and a neutral border. Interactive controls retain a two-pixel focus outline with a three-pixel offset. The composer expresses focus with its accent border. Its textarea has a 160px CSS maximum height; drafts and code mode are saved per network context and room through Rust persistence. Text mode sends with Enter and inserts a newline with Shift+Enter. Code mode inserts newlines with Enter and sends with Cmd/Ctrl+Enter. Input-method composition does not submit.

### Navigation and people

Room rows pair a hash icon with a truncated label and an optional unread count or mute icon. Duplicate room names remain valid; short ID suffixes distinguish matching labels. Selected rows use an accent wash and stronger text. Cmd/Ctrl+K opens local search; Alt+1 through Alt+9 selects visible unarchived rooms. Search can be scoped to the current room and distinguishes previous-network matches.

Room management appears only for the creator installation in the current network. General is a permanent system room. Archived rooms remain readable. Previous-network history replaces the composer with a local read-only notice.

People and connection labels come from backend state. The sidebar shows connected peers; the panel includes the local profile and discovered peer statuses. Empty states explain discovery or unavailable networking. Never seed production with sample people or conversations. Rust owns durable history and drafts in SQLite; the frontend presents state and issues commands. A display name does not verify a person's identity.

### Messages and code

Messages align by avatar and author baseline and receive a subtle row fill on hover. Plain-message copy buttons remain visible at all supported desktop widths. Code has its own persistent "Copy code" action in a dark inset panel with a slim header. Preserve exact content, line breaks, whitespace, and clipboard output. Code uses wrapping for long strings without editing stored text.

Explicit HTTP/HTTPS URLs render as clickable text and open through the native browser action. Production does not use the reference's decorative fetched link cards. Own messages show either "Saved locally · waiting for peers" or "Shared with connected peers" according to backend delivery state. Copy confirmation changes the icon or label for 1500ms and shows a 2000ms toast. Search destinations receive an accent wash and outline for 2000ms.

### Dialogs and motion

Search, room creation, and settings use compact dialogs with consistent fields and action placement. Settings preserve System as the default and as a stored preference; System follows operating-system appearance changes. Search and navigation do not have entry animation. Icon and composer state transitions use 120ms. Toast opacity uses 140ms and its small upward movement uses 180ms with the source easing curve. Reduced motion removes transitions, animations, and button movement while retaining textual feedback.

### Startup

The native window keeps its normal size throughout opening. Apply the saved Light, Dark, or System appearance before revealing any content. When local initialization takes more than 180 ms after the interface mounts, center an 18px indeterminate spinner beside “Opening Local Chat…” on the existing `surface`, using `secondary` text at 13px. Fast initialization reveals the workspace directly. There is no artificial minimum duration or startup transition. Reduced motion leaves a static indicator with the same status text. A failed startup replaces the status with a compact explanation, optional error details, and the existing primary “Try again” button. Keep incomplete workspace controls hidden and inert until ready.

## Do's and Don'ts

### Do:

- Do use semantic CSS variables so light and dark appearances stay consistent.
- Do preserve the approved proportions, system fonts, compact spacing, and native window controls.
- Do keep message text selectable and preserve exact code and copy content.
- Do keep copy actions visible and accessible by keyboard.
- Do show backend connection and delivery facts, including honest empty and read-only states.
- Do preserve room drafts and reading position while navigating bounded message pages.

### Don't:

- Don't reintroduce the outer presentation canvas or imitation window controls.
- Don't introduce photographic avatars or external icon and font dependencies.
- Don't turn every message into a separate elevated card.
- Don't fill production with sample people, sample conversations, or browser-preview storage claims.
- Don't add decorative motion to room switching, search, navigation, or synchronized messages.
- Don't treat old mobile-preview CSS or one-platform screenshots as verified cross-platform behavior.
