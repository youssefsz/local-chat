---
name: Nearby
description: A quiet shared workspace for nearby conversations.
colors:
  "canvas": "#eae9e5"
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
  "canvas-dark": "#151817"
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
  "avatar-nina-bg": "#e6e7da"
  "avatar-nina-ink": "#666a43"
  "avatar-oliver-bg": "#dfe8ed"
  "avatar-oliver-ink": "#4e6a7a"
  "avatar-maya-bg": "#eee2da"
  "avatar-maya-ink": "#925d47"
  "avatar-alex-bg": "#e7e0ed"
  "avatar-alex-ink": "#756482"
  "avatar-sam-bg": "#dce8df"
  "avatar-sam-ink": "#4b7259"
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
  "window": "13px"
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
  shared-link:
    textColor: "{colors.ink}"
    rounded: "{rounded.base}"
    padding: "11px 13px"
  code-block:
    backgroundColor: "{colors.code}"
    textColor: "{colors.code-ink}"
    rounded: "{rounded.base}"
  avatar:
    rounded: "{rounded.avatar}"
    width: "33px"
    height: "33px"
---

# Design System: Nearby

## Overview

**Creative North Star: "Quiet communal studio"**

Nearby uses a quiet communal studio direction. Warm neutral panels, compact controls, and a terracotta accent support long conversations without turning the interface into a dashboard. System type, initial avatars, and inline icons keep the standalone prototype small and familiar across desktop platforms.

The implemented HTML is the source for the values below. The palette and component rules describe the current prototype; the Nearby name remains provisional. Product requirements live in PRODUCT.md, and the selected direction is recorded in work/direction.md.

**Key Characteristics:**

- Warm sidebar beside a clear reading pane.
- Terracotta for actions, selection, links, and focus.
- Left-aligned messages with stable author initials and visible copy controls.
- Light, dark, and system appearance without external fonts or images.

## Colors

The light theme combines warm paper tones with dark green-gray text. The dark theme keeps the same roles and shifts terracotta to a pale peach for contrast. Frontmatter keys ending in `-dark` record the matching `data-theme="dark"` overrides.

### Primary

`accent` is terracotta for Send, primary dialog actions, active rooms, links, and caret color. `accent-hover` deepens the Send button on hover. `accent-wash` gives selected rooms and search-result destinations a quiet fill. `focus` supplies the keyboard outline.

### Neutral

`canvas` frames the app. `surface` fills the conversation and dialogs; `sidebar` separates navigation; `subtle` and `hover` distinguish rows and interactive states. `line` supplies borders. `ink`, `secondary`, and `muted` distinguish message text, supporting copy, and timestamps. `code` and `code-ink` keep snippets readable in both themes.

`green` marks online status. The six `avatar-*-bg` and `avatar-*-ink` pairs identify sample authors and stay constant across themes. These colors identify people rather than a second action hierarchy. Sidecar tonal ramps are generated panel previews; the frontmatter records the actual application colors.

**The Accent Rule.** Use the action accent for actionable or selected content. Keep ordinary message text in `ink`.

## Typography

Use the system sans-serif stack for all interface text. Use the system monospace stack for snippets and code-entry mode. The `intro`, `title`, `body`, `author`, `button`, `label`, and `code` roles in frontmatter record the implemented hierarchy; there is no display-font treatment.

Message body text uses a generous line height and a maximum width of 72ch. Compact metadata uses 11px text, and timestamps use tabular numerals. The wordmark uses 21px type at weight 650 with -.8px tracking. On narrow screens, the room title becomes 16px and the composer input becomes 16px to remain usable on mobile browsers.

## Layout

The app is a full-height desktop window with a title bar, room sidebar, conversation, and optional people panel. The conversation scrolls independently while the header and composer remain available. Messages and composer share a maximum width of 850px.

- The default outer inset is 24px, the app maximum width is 1490px, and the sidebar is 246px wide. At widths of at least 1500px, the inset becomes 38px and the sidebar becomes 264px.
- At widths of 1100px or less, the inset is 12px and the sidebar is 218px. The people panel overlays the right edge.
- At widths of 700px or less, the app fills the viewport. Navigation becomes a 260px drawer below the 31px title bar. Conversation side padding is 19px; the composer side padding is 13px. Copy actions stay visible, and secondary header details collapse.
- At heights of 780px or less on desktop widths, navigation and header spacing tighten. The sidebar shows two people and a View everyone control. The final compact rules use 14px above people, 4px vertical person padding, 18px above room navigation, 8px below section labels, and 12px above the footer.

## Elevation & Depth

Tonal panels and thin borders organize the main workspace. Messages have no shadow. Shadows distinguish the app window, floating menus, dialogs, people overlay, and toast; their exact values live in `.impeccable/design.json`. The mobile app frame drops its shadow and corner radius. Dialogs and the narrow navigation drawer use dim backdrops.

## Shapes

Small rounded rectangles carry the interface. Controls use the `control` radius; snippets and link previews use `base`; message hover areas use `message`. Avatars are softly squared, while online dots and the compact header avatar stack are circular. The composer has 10px corners. The app frame uses `window`, and dialogs use `dialog`. Borders are one pixel throughout the workspace.

## Components

### Buttons and fields

Primary dialog buttons and Send use the accent against the current surface color. Secondary buttons use the hover neutral. Send becomes disabled at 40% opacity when the composer is empty. Enabled buttons move down one pixel while pressed. Dialog buttons darken with a brightness filter on hover; Send uses `accent-hover`. Icon buttons reveal a neutral fill on hover.

Text fields use a subtle fill and a neutral border. The composer uses a white or dark surface and changes its border to the accent when focused. Its textarea grows up to 160px and preserves each room's draft. Enter sends; Shift+Enter inserts a line break. Plain text and code modes share the same composer.

Interactive controls retain the implemented two-pixel accent outline with a three-pixel offset. The composer expresses focus through its border.

### Room navigation and people

Room rows pair a hash icon with a truncated label and optional unread badge. The selected row uses an accent wash and stronger text. Room switching is immediate. Search opens with Cmd/Ctrl+K, and Alt+1 through Alt+9 selects visible rooms. The room menu offers rename and archive only for rooms owned by the current user.

Initial avatars identify authors without image assets. Online dots supplement text labels. The people panel retains its sample-network explanation. It uses the same avatars and text hierarchy as the sidebar.

### Messages, links, and code

Messages align by avatar and author baseline, with a subtle row fill on hover. Copy actions appear on hover or focus within the row and remain visible on narrow screens. A copied action changes to a checkmark in place for 1700ms. A toast confirms the result for 2600ms.

Link previews use a bordered row with title, domain, and an external-link icon. Code snippets use a dark container, a slim header, and a visible Copy code action. Preserve whitespace, exact copy content, and long-string wrapping. Browser-local messages carry the Saved in this preview label.

### Dialogs and motion

Search, room creation, and settings share compact dialogs with consistent fields and action placement. Search and navigation appear without entry animation. Icon-button and composer state changes use 120ms transitions. Toast opacity uses 140ms, with a 180ms upward movement governed by the implemented easing curve. Reduced-motion preference removes transitions and animations.

## Do's and Don'ts

### Do:

- Do use the semantic CSS variables so light and dark appearance stay consistent.
- Do keep message text selectable and preserve whitespace in code.
- Do expose copy controls on hover and keyboard focus, and keep them visible on narrow screens.
- Do retain explicit sample-data labels and the Saved in this preview state for browser-local messages.
- Do keep the composer and room navigation available as the viewport gets smaller.

### Don't:

- Don't introduce photographic avatars or external icon and font dependencies into this prototype.
- Don't turn every message into a separate elevated card.
- Don't imply that the preview has discovered real people or delivered messages across a network.
- Don't add decorative motion to room switching, search, or the navigation drawer.
