import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir, stat } from 'node:fs/promises';
const root = new URL('../dist/', import.meta.url);
test('every HTML page has metadata, navigation, and valid local links', async () => {
  const files = (await readdir(root)).filter((name) => name.endsWith('.html'));
  assert.equal(files.length, 7);
  for (const name of files) {
    const html = await readFile(new URL(name, root), 'utf8');
    assert.equal((html.match(/<h1[ >]/g) || []).length, 1, name);
    for (const required of [
      'name="description"',
      'rel="canonical"',
      'property="og:image"',
      'id="main"',
      'href="/privacy"',
      'href="/terms"',
    ])
      assert.ok(html.includes(required), `${name}: ${required}`);
    for (const [, href] of html.matchAll(/(?:href|src)="(\/[^"#]*)"/g)) {
      assert.ok(!href.endsWith('.html'), href);
      if (href.startsWith('/download/')) continue;
      const file =
        href === '/'
          ? 'index.html'
          : href.slice(1) + (href.includes('.') ? '' : '.html');
      assert.ok((await stat(new URL(file, root))).isFile(), `${name}: ${href}`);
    }
  }
  const headers = await readFile(new URL('_headers', root), 'utf8');
  assert.ok(!headers.includes('SCHEMA_HASH'));
  assert.ok(headers.includes("frame-ancestors 'none'"));
});
