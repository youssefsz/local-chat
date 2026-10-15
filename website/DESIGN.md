---
name: Local Chat website
description: The desktop app's visual language, extended to its public website.
colors:
  bg: "#f5f4f0"
  surface: "#fff"
  ink: "#292b29"
  secondary: "#646861"
  accent: "#a94329"
  hover: "#913820"
  wash: "#f0e1d8"
  line: "#e9e7e2"
  dark: "#202422"
  dark-ink: "#e4e9e2"
typography:
  display:
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "clamp(44px, 5.4vw, 84px)"
    fontWeight: 550
    lineHeight: 1.06
    letterSpacing: "-0.035em"
  headline:
    fontSize: "clamp(30px, 3.1vw, 48px)"
    fontWeight: 550
    lineHeight: 1.06
    letterSpacing: "-0.035em"
  body:
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif'
    fontSize: "17px"
    lineHeight: 1.6
  button:
    fontSize: "16px"
    fontWeight: 550
    lineHeight: 1.5
rounded:
  control: "6px"
spacing:
  margin: "clamp(20px, 4vw, 64px)"
  16: "16px"
  24: "24px"
  32: "32px"
  48: "48px"
  80: "80px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.surface}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "12px 22px"
  button-primary-hover:
    backgroundColor: "{colors.hover}"
    textColor: "{colors.surface}"
---

# Design System: Local Chat website

## Overview

The website extends the existing app world recorded in [the root design system](../DESIGN.md). Warm paper, system fonts, terracotta actions, and native app captures carry Local Chat's identity. The root product and design documents remain authoritative for the desktop app.

The selected direction is "The product performs." The home page uses Persuade mode for small groups sharing a local network. A centered headline, supporting copy, and download action lead directly into a large native app capture. The original factual hero copy remains intact. Getting started, release notes, privacy, and terms use Read mode with clear headings and bounded text columns. These are website choices, not changes to the app's Operate layout.

The centered hero uses the [Spline reference on Mobbin](https://mobbin.com/sites/sections/3bf8e20b-ba8a-464a-a774-bb107f225557) for composition only. It borrows no assets or typography. This is a layout refinement within the existing brand.

The implementation is plain HTML, CSS, and browser JavaScript. There is no approved website comp. The current pages and stylesheet record the implemented composition; the approved app HTML under `docs/reference` stays untouched.

## Colors

Terracotta identifies primary download actions, active navigation, link hover, and keyboard focus. Its darker hover value gives pressed controls a familiar app treatment. The pale terracotta wash separates the network explanation and marks text selection.

Warm paper is the page background; white contains the mobile navigation overlay. Ink and secondary text separate headings from supporting copy. Quiet lines frame the app preview and navigation overlay. Selected appearance tabs use terracotta text and an underline. Light and dark colors belong to the native capture and its surrounding window frame; the rest of the website keeps its light palette.

**The inherited identity rule.** Keep the app's system fonts and palette when extending this website. The user's requirement to match the app takes precedence over generic font or palette exclusions.

## Typography

The system sans-serif stack handles display, navigation, and reading. No font download is required. Display headings use the frontmatter scale and tight tracking; body text keeps the app's familiar forms at a larger reading size.

The wordmark is 21px at weight 650. Introductory copy is 20px or 21px. Supporting text is 13px to 15px. Reading headings use 30px with a 1.2 line height, and prose is at most 66ch wide. The desktop hero headline fits on one line. At 600px and below, body text becomes 16px, the home headline becomes 50px with a 10ch maximum width and two lines, and other page titles become 44px.

## Layout

The hero stays in one centered column at every width. Its copy has an 800px maximum width, with supporting text capped at 48ch. It starts 48px below the header, with 24px between the headline and body, 24px before Download, and 16px before the compatibility note. Horizontal margins follow the frontmatter margin token. The body caps at 1920px on wide displays.

The preview follows the copy after 40px and caps at 1120px. Light/Dark tabs center above the framed native app capture with a 16px gap before the frame. The image scales completely with automatic height. Mobile never crops the desktop composition to make it appear larger. The sample-content caption and an "Enlarge preview" button sit below it. Both screenshot buttons and the caption action open the selected capture in a modal viewer.

The introduction, explanations, FAQ, and closing download section retain their 5:6 desktop grids with a 6vw gap. At 900px and below, those sections use equal columns with 32px or 40px gaps; the hero remains centered.

At 600px and below, margins become 20px. The hero starts 40px below the header, supporting copy becomes 18px with a 30ch maximum width and 20px top gap, and the preview follows after 36px. The tabs remain centered above the image. The caption and Enlarge preview action stack with a 12px gap and align left beneath it. Supporting home sections stack. The header exposes a native Menu disclosure alongside Download, with the desktop navigation links inside the menu.

Guide and release pages use a 1:2 grid. Guide navigation sticks near the top while reading on desktop; at 600px it returns to normal flow above the article. Legal text uses the same bounded prose column and loses its desktop left offset on mobile. Download rows pair platform marks and platform information; mobile places each download button below its description.

## Elevation & Depth

Pages use flat fills and open spacing. The app preview has one quiet outer rim and a separator beneath its decorative window bar. It has no shadow or glow. The modal image viewer uses a dark translucent backdrop, `#202422cc`, to separate it from the page. Only the open mobile navigation panel adds a small shadow, `0 8px 24px #292b2914`.

## Shapes

Ordinary controls use the control radius. The hero download action has 10px corners around a 5px-cornered arrow inset; the modal viewer has 12px corners, becoming 10px on mobile. The app preview frame has 10px outer corners, becoming 8px at 600px and below, and clips its contents to that outline. Keep platform marks and the Local Chat logo intact. Ordinary explanatory sections remain open page content.

## Components

### Navigation and actions

Download links use the primary button with a 48px minimum height. Hover deepens the terracotta fill. Press moves the button down one pixel. Links, buttons, and disclosure summaries receive a visible two-pixel terracotta keyboard outline with a five-pixel offset. The first keyboard link skips to main content. Preserve semantic anchors, native buttons, and native disclosure elements so keyboard operation follows browser conventions.

The footer keeps GitHub, Privacy, Terms, and Support available across pages. Download platform marks identify compatibility. Apple and Linux assets come from Simple Icons; Windows uses the four-panel silhouette. Asset provenance and deployment configuration live in [README.md](README.md).

The hero download action keeps its compact 48px height and approximately 240px width. It uses 6px padding around a 36px arrow block, with 22px left padding and an 18px text-to-arrow gap. The diagonal arrow sits inside the ink-colored block, which stays neutral while hover deepens the surrounding terracotta button. The arrow moves one pixel up and right on hover.

The frame takes a terracotta outline when its screenshot button has focus, keeping keyboard focus visible outside the clipped window edges.

### Native app captures

The hero uses the native app image directly, without a fabricated HTML conversation or interactive code-copy demonstration. Both appearance panels have explicit image dimensions and descriptive alternative text. Light loads eagerly with high fetch priority; Dark is lazy-loaded.

Light and Dark are manual-activation tabs in a labeled tablist above the screenshot. Each has a 44px minimum height, a 92px minimum width, an appearance icon, and `aria-selected` state. The selected tab uses terracotta text and a two-pixel underline; hover uses ink text. Separate labeled tabpanels contain the Light and Dark captures, with the inactive panel hidden. Left and Right arrow keys wrap focus between tabs; Home and End focus the first and last tab. Enter or Space activates the focused tab. Activation changes the visible panel and surrounding frame's `data-theme`. The rest of the website keeps its light appearance and saves no preference.

`public/assets/app-light.jpg` and `app-dark.jpg` are native macOS captures from an isolated 0.0.1 preview profile. Each is 2560 by 1800 pixels, displayed with 1280 by 900 intrinsic HTML dimensions. They show synthetic Alex workshop messages, General and Workshop rooms, an honest zero connected peers count, waiting-for-peers delivery labels, and a blank composer. They publish no real conversation. A dedicated test-profile bundle sets `decorations: false` only for capture, excluding the operating-system title bar and control badge. Each appearance is captured immediately after a fresh launch, before computer-use input can add a pointer overlay. Both images show the full app content without cropping or retouching, with no cursor or badge. The production app retains its native window decorations. These images demonstrate appearance, not cross-platform or physical multi-computer verification.

### Decorative app frame

At the user's request, an HTML macOS-style frame surrounds the unchanged native JPEG. The `app-window` wrapper owns the preview's Light or Dark theme. Its window bar is `aria-hidden="true"`; the three static CSS traffic dots are decoration, with no button or window-management behavior. A centered "Local Chat" label uses 12px system text at weight 500.

The desktop bar is 34px high with 12px side padding. Its dots are 10px wide, separated by 7px, in red `#ff5f57`, yellow `#febc2e`, and green `#28c840`. At 600px and below, the bar becomes 28px high with 10px side padding, the label becomes 11px, and the dots become 8px with 5px gaps.

Light appearance uses a `#faf9f7` bar, the existing line color for its one-pixel rim and separator, and secondary text for its label. Dark appearance uses a `#272c29` bar, `#353c36` rim and separator, and `#b0b8ad` label. This frame is a decorative website surround. The raw captures stay title-bar-free, pointer-free, and uncropped, and the production native UI is unchanged.

### Image viewer

Each screenshot is a button with a zoom-in cursor and a visible enlarge cue. The cue is 44px square on desktop and 36px on mobile. Both screenshot buttons and the caption's "Enlarge preview" button open the selected native capture in an HTML `dialog`, without navigating to the raw image. The dialog has an appearance-specific title and opens with Close focused. The native JPEG remains unchanged; the decorative window frame stays outside the viewer.

The dialog caps at 1440px wide and leaves 24px around the viewport. At 600px and below it leaves 8px. A warm-paper toolbar holds 44px Zoom out, Fit, Zoom in, and Close controls above a scrollable neutral image stage. Opening starts in Fit with the full image visible. Zoom steps through 100%, 150%, and 200%, measured against the capture's 1280 by 900 logical size. The controls stop at their bounds; focus moves to Fit if a focused zoom control becomes disabled. Plus or equals zooms in, minus zooms out, and zero restores Fit. The percentage label announces changes politely.

When zoomed, the stage uses native scrolling and implements mouse drag-to-pan with grab feedback. Double-click switches between Fit and the first zoom step. These implemented behaviors do not establish physical touch-gesture support. Escape, Close, or a click that starts and ends on the backdrop dismiss the dialog. Closing restores focus to the invoking button and unlocks page scrolling. Native dialog modality keeps page controls outside the active modal. Loading has a status message; a failed image shows an inline error and Try again. Actual browser checks and outstanding interaction verification remain in `docs/testing.md`.

### Reading and release content

FAQ and platform help use native disclosures. Guides use in-page anchors and separated sections. The release page renders latest stable metadata and highlights as text, with full notes on GitHub. Keep release fallback links readable when live metadata is unavailable; current version and deployment behavior belong to the implementation and README.

### Motion

Motion is limited to button hover, press, and the inset download arrow feedback with 140ms transitions. Content appears immediately. Reduced motion removes transitions, animations, and press movement while preserving text feedback and all controls.

## Do's and Don'ts

- Do preserve the inherited system typography, warm neutrals, and terracotta actions.
- Do label the native capture's sample messages and keep its provenance explicit.
- Do keep the complete desktop capture visible on mobile and provide the Enlarge preview action.
- Do retain keyboard focus, appearance state, disclosure semantics, and useful release fallbacks.
- Don't publish private messages or treat synthetic screenshots as connection evidence.
- Don't add fabricated testimonials, user counts, pricing, or signup steps.
- Do allow the requested decorative macOS frame around the real capture, with static dots and no window-management behavior.
- Don't add operating-system control badges, cursors, or fabricated app content to the preview.
- Don't alter the approved app reference or production native window chrome to match the website frame.
- Don't infer Windows or Linux desktop verification from macOS imagery.
