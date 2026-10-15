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
  code: "#252a29"
  code-ink: "#e6eae4"
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
  code:
    fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace"
    fontSize: "12px"
    lineHeight: 1.85
rounded:
  control: "6px"
  code: "8px"
  demo: "10px"
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

The website extends the existing app world recorded in [the root design system](../DESIGN.md). Warm paper, system fonts, terracotta actions, compact app fragments, and green-gray dark panels carry Local Chat's identity. The root product and design documents remain authoritative for the desktop app.

The selected direction is "The product performs." The home page uses Persuade mode for small groups sharing a local network. Visitors can copy a labeled example, inspect the real desktop app, and download it. Getting started, release notes, privacy, and terms use Read mode with clear headings and bounded text columns. These are website choices, not changes to the app's Operate layout.

The implementation is plain HTML, CSS, and browser JavaScript. There is no approved website comp. The current pages and stylesheet record the implemented composition; the approved app HTML under `docs/reference` stays untouched.

## Colors

Terracotta identifies primary download actions, active navigation, link hover, and keyboard focus. Its darker hover value gives pressed controls a familiar app treatment. The pale terracotta wash separates the network explanation and marks text selection.

Warm paper is the default page background; white contains the example conversation. Dark green-gray creates a full-width stage for screenshots of both app appearances. Ink and secondary text separate headings from supporting copy; quiet lines belong to app fragment chrome and the mobile navigation overlay. Code retains the app's dark panel and pale text.

**The inherited identity rule.** Keep the app's system fonts and palette when extending this website. The user's requirement to match the app takes precedence over generic font or palette exclusions.

## Typography

The system sans-serif stack handles display, navigation, and reading. System monospace handles code. No font download is required. Display headings use the frontmatter scale and tight tracking; body text keeps the app's familiar forms at a larger reading size.

The wordmark is 21px at weight 650. Introductory copy is 20px or 21px. Supporting text is 13px to 15px. Reading headings use 30px with a 1.2 line height, and prose is at most 66ch wide. Code uses the frontmatter role inside the example. At 600px and below, body text becomes 16px, the home headline becomes 50px, other page titles become 44px, and code becomes 11px.

## Layout

The home page pairs copy with the interactive example in a 5:6 grid with a 6vw gap. Related explanations and the closing download action repeat that alignment. Large section intervals give the public page room to read while the demonstration keeps the app's compact message density. Horizontal margins follow the frontmatter margin token. The body caps at 1920px on wide displays.

At 900px and below, the hero stacks and its example shifts inward by 15%. Supporting sections keep two columns until 600px. At 600px and below, margins become 20px, home sections stack, the example loses its offset, and the header exposes a native Menu disclosure alongside Download. The desktop navigation links remain available inside the menu.

The screenshot always scales as a complete image. Mobile never crops the desktop composition to make it appear larger. Both the image link and the visible "View full-size screenshot" link open the selected capture.

Guide and release pages use a 1:2 grid. Guide navigation sticks near the top while reading on desktop; at 600px it returns to normal flow above the article. Legal text uses the same bounded prose column and loses its desktop left offset on mobile. Download rows pair platform marks and platform information; mobile places each download button below its description.

## Elevation & Depth

Pages use flat fills and open spacing. The demonstration uses one quiet border. Screenshots retain their native window chrome. Only the open mobile navigation panel adds a small shadow, `0 8px 24px #292b2914`, to separate its overlay from the page.

## Shapes

Controls and screenshots use the control radius. The example conversation and softly squared initials use the demo radius; code uses the code radius. Keep platform marks and the Local Chat logo intact. Ordinary explanatory sections remain open page content.

## Components

### Navigation and actions

Download links use the primary button with a 48px minimum height. Hover deepens the terracotta fill. Press moves the button down one pixel. Links, buttons, and disclosure summaries receive a visible two-pixel terracotta keyboard outline with a five-pixel offset. The first keyboard link skips to main content. Preserve semantic anchors, native buttons, and native disclosure elements so keyboard operation follows browser conventions.

The footer keeps GitHub, Privacy, Terms, and Support available across pages. Download platform marks identify compatibility. Apple and Linux assets come from Simple Icons; Windows uses the four-panel silhouette. Asset provenance and deployment configuration live in [README.md](README.md).

### Copy example

The home conversation fragment is explicitly labeled as an example. Its persistent "Copy code" button copies the code element's exact text, including whitespace. Success changes the button to "Copied" for 1500ms and announces confirmation through a status region. Clipboard failure announces how to select and copy manually. The code remains selectable, and narrow screens wrap its presentation without changing copied content. The fragment does not simulate discovery or send messages.

### Native app captures

Light and Dark buttons form a labeled group with `aria-pressed` state. Selection updates the image, descriptive alternative text, image link, and visible full-size link. It changes the displayed capture only; the website keeps its light page appearance and saves no preference.

`public/assets/app-light.jpg` and `app-dark.jpg` are native macOS captures from an isolated 0.0.1 preview profile. Each is 2560 by 1600 pixels, displayed with 1280 by 800 intrinsic HTML dimensions. They show synthetic Alex workshop messages, General and Workshop rooms, an honest zero connected peers count, waiting-for-peers delivery labels, and a blank composer. They publish no real conversation. The captured pointer in the native title bar is part of the source capture. These images demonstrate appearance, not cross-platform or physical multi-computer verification.

### Reading and release content

FAQ and platform help use native disclosures. Guides use in-page anchors and separated sections. The release page renders latest stable metadata and highlights as text, with full notes on GitHub. Keep release fallback links readable when live metadata is unavailable; current version and deployment behavior belong to the implementation and README.

### Motion

Motion is limited to button hover and press feedback with 140ms transitions. Content appears immediately. Reduced motion removes transitions, animations, and press movement while preserving text feedback and all controls.

## Do's and Don'ts

- Do preserve the inherited system typography, warm neutrals, and terracotta actions.
- Do distinguish the interactive example from real app captures and label sample content.
- Do keep full desktop screenshots visible on mobile with an explicit full-size link.
- Do retain keyboard focus, copy feedback, disclosure semantics, and useful release fallbacks.
- Don't publish private messages or treat synthetic screenshots as connection evidence.
- Don't add fabricated testimonials, user counts, pricing, or signup steps.
- Don't replace native screenshots with an imitation operating-system window or alter the approved app reference.
- Don't infer Windows or Linux desktop verification from macOS imagery.
