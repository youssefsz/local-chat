# Local Chat website

Production: [local-chat.youssef.tn](https://local-chat.youssef.tn).

Plain HTML, CSS, and browser JavaScript, with a small Cloudflare Worker for release downloads. The website lives independently of the desktop app's npm package.

```sh
cd website
npm ci
npm run build
npm test
npm run dev
```

Edit page bodies and metadata in `pages/`. `build.mjs` adds the shared header, footer, canonical URLs, social metadata, sitemap, robots file, and a CSP hash for structured data. Generated `dist/` is ignored. There are no external fonts, analytics scripts, browser storage, or tracking cookies.

## Deployment

Cloudflare Workers Builds connects `youssefsz/local-chat` to `local-chat-website` on `main`. Root directory: `website`. Build command: `npm run build && npm test`. Deploy command: `npx wrangler deploy`. Include watch path: `website/**`. Preview builds are disabled. The custom domain is `local-chat.youssef.tn`, declared in `wrangler.jsonc`.

Cloudflare manages the existing GitHub connection and build credential. No Cloudflare token is stored in this repository or GitHub Actions. GitHub's Website workflow checks pull requests; Cloudflare deploys merged website changes. A website-only edit does not publish an app release. App release tags still use the release workflow.

`html_handling: drop-trailing-slash` serves `/privacy`, redirects `/privacy.html` and `/privacy/`, and returns a real 404 for missing pages. Production headers prohibit framing, inline executable scripts, third-party resources, camera, microphone, and geolocation. `observability.enabled` is false; Cloudflare still processes normal delivery and security data.

## Downloads

`/api/release` reads GitHub's latest published stable release, validates all four package names and destinations, and caches public metadata for five minutes. `/download/macos`, `/download/windows`, `/download/linux`, and `/download/debian` return temporary redirects. No installer passes through the Worker. On GitHub API errors, downloads fall back to the public GitHub release page and metadata returns 503. Beta and draft releases are excluded.

Publishing a stable GitHub release updates the links without rebuilding the site. The release page renders highlights as text and links to GitHub for full Markdown, contributor credits, and history.

## Assets and design

The logo is copied from `assets/icon/master.svg`. Screenshots are native captures of an isolated 0.0.1 preview profile with synthetic workshop messages, honest waiting-for-peers status, and an empty composer. No real conversations are published. Apple and Linux marks come from [Simple Icons](https://github.com/simple-icons/simple-icons), under its CC0 asset license and respective brand rights. The Windows mark uses the four-panel platform silhouette. These marks identify compatibility and do not imply endorsement.

The site extends the app's system fonts, warm neutral surfaces, terracotta actions, and green-gray dark screenshot stage. The home page demonstrates exact code copying, then shows the actual app with a Light/Dark selector. It has no sign-up, pricing fiction, testimonials, customer counters, or network simulation. The design-taste direction is “The product performs”; the brief's app identity overrides the skill's generic palette bans. Existing screenshots replace its suggested stock imagery. Motion is limited to press and hover feedback so content never waits for an entrance animation.

Reference configuration: [Cloudflare HTML handling](https://developers.cloudflare.com/workers/static-assets/routing/advanced/html-handling/) and [build watch paths](https://developers.cloudflare.com/workers/ci-cd/builds/build-watch-paths/).
