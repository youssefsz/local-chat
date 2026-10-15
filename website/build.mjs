import { readFile, writeFile, mkdir, readdir, cp, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { createHash } from 'node:crypto';

const root = fileURLToPath(new URL('.', import.meta.url));
const out = path.join(root, 'dist');
await rm(out, { recursive: true, force: true });
await mkdir(out, { recursive: true });
await cp(path.join(root, 'public'), out, { recursive: true });
const site = 'https://local-chat.youssef.tn';
const routes = [];
let schemaHash = '';
const nav = [
  ['/', 'Home'],
  ['/get-started', 'Get started'],
  ['/releases', 'Releases'],
];
const escape = (s) =>
  s.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;');
for (const name of await readdir(path.join(root, 'pages'))) {
  if (!name.endsWith('.html')) continue;
  const input = await readFile(path.join(root, 'pages', name), 'utf8');
  const [, metadata, body] = input.match(/^<!--(.*?)-->\s*([\s\S]*)$/s);
  const meta = JSON.parse(metadata);
  const route = name === 'index.html' ? '/' : '/' + name.replace('.html', '');
  const title =
    meta.title === 'Local Chat'
      ? meta.title + ' · Chat on your local network'
      : meta.title + ' · Local Chat';
  const schema =
    route === '/'
      ? '<script type="application/ld+json">' +
        JSON.stringify({
          '@context': 'https://schema.org',
          '@type': 'SoftwareApplication',
          name: 'Local Chat',
          applicationCategory: 'CommunicationApplication',
          operatingSystem: 'macOS, Windows, Linux',
          url: site,
          downloadUrl: site + '/download',
          license: 'https://opensource.org/license/mit',
          offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
        }) +
        '</script>'
      : '';
  if (schema)
    schemaHash = createHash('sha256')
      .update(schema.match(/<script[^>]*>(.*?)<\/script>/s)[1])
      .digest('base64');
  await writeFile(
    path.join(out, name),
    `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${escape(title)}</title><meta name="description" content="${escape(meta.description)}"><link rel="canonical" href="${site}${route}"><meta name="theme-color" content="#f5f4f0"><link rel="icon" href="/assets/logo.svg" type="image/svg+xml"><meta property="og:type" content="website"><meta property="og:site_name" content="Local Chat"><meta property="og:title" content="${escape(title)}"><meta property="og:description" content="${escape(meta.description)}"><meta property="og:url" content="${site}${route}"><meta property="og:image" content="${site}/assets/app-light.png"><meta property="og:image:alt" content="Local Chat desktop app with a sample conversation"><meta name="twitter:card" content="summary_large_image"><link rel="stylesheet" href="/style.css"><script type="module" src="/site.js"></script>${meta.noindex ? '<meta name="robots" content="noindex">' : ''}${schema}</head>
<body><a class="skip" href="#main">Skip to content</a><header class="site-header"><a class="brand" href="/" aria-label="Local Chat home"><img src="/assets/logo.svg" width="36" height="36" alt=""><span>Local Chat</span></a><details class="mobile-menu"><summary>Menu</summary><nav aria-label="Mobile navigation">${nav.map(([href, label]) => `<a href="${href}">${label}</a>`).join('')}</nav></details><nav aria-label="Main navigation">${nav.map(([href, label]) => `<a href="${href}"${route === href ? ' aria-current="page"' : ''}>${label}</a>`).join('')}<a class="nav-download" href="/download"${route === '/download' ? ' aria-current="page"' : ''}>Download</a></nav></header>
<main id="main">${body}</main><footer class="site-footer"><p>Made for the same network.</p><nav aria-label="Footer"><a href="https://github.com/youssefsz/local-chat">GitHub</a><a href="/privacy">Privacy</a><a href="/terms">Terms</a><a href="https://github.com/youssefsz/local-chat/issues/new/choose">Support</a></nav><p>Free &amp; open source. MIT licensed.</p></footer></body></html>`,
  );
  if (!meta.noindex) routes.push(route);
}
await writeFile(
  path.join(out, 'sitemap.xml'),
  '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">' +
    routes.map((route) => `<url><loc>${site}${route}</loc></url>`).join('') +
    '</urlset>\n',
);
await writeFile(
  path.join(out, 'robots.txt'),
  `User-agent: *\nAllow: /\nDisallow: /api/\nSitemap: ${site}/sitemap.xml\n`,
);
const headers = await readFile(path.join(out, '_headers'), 'utf8');
await writeFile(
  path.join(out, '_headers'),
  headers.replace('SCHEMA_HASH', schemaHash),
);
console.log(`Built ${routes.length} pages with clean canonical URLs.`);
