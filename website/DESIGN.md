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

Warm paper is the page background; white contains the mobile navigation overlay. Ink and secondary text separate headings from supporting copy. Quiet lines frame the screenshot and navigation overlay, and fill the selected appearance control. Light and dark app colors remain inside the native captures; changing the capture does not change the website palette.

**The inherited identity rule.** Keep the app's system fonts and palette when extending this website. The user's requirement to match the app takes precedence over generic font or palette exclusions.

## Typography

The system sans-serif stack handles display, navigation, and reading. No font download is required. Display headings use the frontmatter scale and tight tracking; body text keeps the app's familiar forms at a larger reading size.

The wordmark is 21px at weight 650. Introductory copy is 20px or 21px. Supporting text is 13px to 15px. Reading headings use 30px with a 1.2 line height, and prose is at most 66ch wide. The desktop hero headline fits on one line. At 600px and below, body text becomes 16px, the home headline becomes 50px with a 10ch maximum width and two lines, and other page titles become 44px.

## Layout

The hero stays in one centered column at every width. Its copy has an 800px maximum width, with supporting text capped at 48ch. It starts 48px below the header, with 24px between the headline and body, 24px before Download, and 16px before the compatibility note. Horizontal margins follow the frontmatter margin token. The body caps at 1920px on wide displays.

The native app capture follows the copy after 40px and caps at 1120px. It scales as a complete image with automatic height. Mobile never crops the desktop composition to make it appear larger. A caption and visible "View full-size screenshot" link sit below it, opposite the Light/Dark control group. Both the image link and full-size link open the selected capture.

The introduction, explanations, FAQ, and closing download section retain their 5:6 desktop grids with a 6vw gap. At 900px and below, those sections use equal columns with 32px or 40px gaps; the hero remains centered.

At 600px and below, margins become 20px. The hero starts 40px below the header, supporting copy becomes 18px with a 30ch maximum width and 20px top gap, and the image follows after 36px. The caption and controls stack with a 12px gap and align left beneath the full image. Supporting home sections stack. The header exposes a native Menu disclosure alongside Download, with the desktop navigation links inside the menu.

Guide and release pages use a 1:2 grid. Guide navigation sticks near the top while reading on desktop; at 600px it returns to normal flow above the article. Legal text uses the same bounded prose column and loses its desktop left offset on mobile. Download rows pair platform marks and platform information; mobile places each download button below its description.

## Elevation & Depth

Pages use flat fills and open spacing. The screenshot has one quiet border around the full app content, with no imitation title bar. Only the open mobile navigation panel adds a small shadow, `0 8px 24px #292b2914`, to separate its overlay from the page.

## Shapes

Controls and screenshots use the control radius. Keep platform marks and the Local Chat logo intact. Ordinary explanatory sections remain open page content.

## Components

### Navigation and actions

Download links use the primary button with a 48px minimum height. Hover deepens the terracotta fill. Press moves the button down one pixel. Links, buttons, and disclosure summaries receive a visible two-pixel terracotta keyboard outline with a five-pixel offset. The first keyboard link skips to main content. Preserve semantic anchors, native buttons, and native disclosure elements so keyboard operation follows browser conventions.

The footer keeps GitHub, Privacy, Terms, and Support available across pages. Download platform marks identify compatibility. Apple and Linux assets come from Simple Icons; Windows uses the four-panel silhouette. Asset provenance and deployment configuration live in [README.md](README.md).

### Native app captures

The hero uses the native app image directly, without a fabricated HTML conversation or interactive code-copy demonstration. The image loads eagerly with high fetch priority and explicit intrinsic dimensions.

Light and Dark buttons below the image form a labeled group with `aria-pressed` state. Each has a 44px minimum height; the selected button uses a neutral fill and hover uses the accent wash. Selection updates the image, descriptive alternative text, image link, and visible full-size link. It changes the displayed capture only; the website keeps its light page appearance and saves no preference.

`public/assets/app-light.jpg` and `app-dark.jpg` are native macOS captures from an isolated 0.0.1 preview profile. Each is 2560 by 1800 pixels, displayed with 1280 by 900 intrinsic HTML dimensions. They show synthetic Alex workshop messages, General and Workshop rooms, an honest zero connected peers count, waiting-for-peers delivery labels, and a blank composer. They publish no real conversation. A dedicated test-profile bundle sets `decorations: false` only for capture, excluding the operating-system title bar and control badge. Each appearance is captured immediately after a fresh launch, before computer-use input can add a pointer overlay. Both images show the full app content without cropping or retouching, with no cursor or badge. The production app retains its native window decorations. These images demonstrate appearance, not cross-platform or physical multi-computer verification.

### Reading and release content

FAQ and platform help use native disclosures. Guides use in-page anchors and separated sections. The release page renders latest stable metadata and highlights as text, with full notes on GitHub. Keep release fallback links readable when live metadata is unavailable; current version and deployment behavior belong to the implementation and README.

### Motion

Motion is limited to button hover and press feedback with 140ms transitions. Content appears immediately. Reduced motion removes transitions, animations, and press movement while preserving text feedback and all controls.

## Do's and Don'ts

- Do preserve the inherited system typography, warm neutrals, and terracotta actions.
- Do label the native capture's sample messages and keep its provenance explicit.
- Do keep full desktop screenshots visible on mobile with an explicit full-size link.
- Do retain keyboard focus, appearance state, disclosure semantics, and useful release fallbacks.
- Don't publish private messages or treat synthetic screenshots as connection evidence.
- Don't add fabricated testimonials, user counts, pricing, or signup steps.
- Don't replace native screenshots with an imitation operating-system window or alter the approved app reference.
- Don't infer Windows or Linux desktop verification from macOS imagery.
